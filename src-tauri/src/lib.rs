//! FlowSTT GUI - Tauri application with vtx-engine integration.
//!
//! The audio/transcription engine (vtx-engine) runs in-process.
//! Tauri commands call AudioEngine methods directly.
//! The IPC socket server is hosted by this process for CLI client access.

mod clipboard;
mod hotkey;
mod ipc;
mod tray;

use flowstt_common::config::{Config, LogLevel, ThemeMode};
use flowstt_common::{
    runtime_mode, ConfigValues, HotkeyCombination, RecordingMode, RuntimeMode, TranscriptionMode,
};
// All shared audio/transcription types come from vtx_engine (re-exported from
// vtx_engine::common). There is no separate vtx-common crate.
use vtx_engine::{AudioDevice, AudioEngine, EngineBuilder, EngineConfig, EngineEvent, HistoryEntry, ModelStatus, TranscriptionHistory};
use std::env;
use std::sync::Arc;
use std::time::Instant;
use tauri::webview::WebviewWindowBuilder;
use tauri::WebviewUrl;
use tauri::{AppHandle, Emitter, Listener, Manager, State};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{reload, EnvFilter};

/// Detect if running on Wayland and set workaround env vars (Linux-specific)
#[cfg(target_os = "linux")]
fn configure_wayland_workarounds() {
    let is_wayland = env::var("WAYLAND_DISPLAY").is_ok()
        || env::var("XDG_SESSION_TYPE")
            .map(|v| v.to_lowercase() == "wayland")
            .unwrap_or(false);

    if is_wayland {
        // SAFETY: This is called before any threads are spawned
        unsafe {
            env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn configure_wayland_workarounds() {}

// ─── Log line payload (emitted to frontend) ──────────────────────────────────

#[derive(serde::Serialize, Clone)]
struct LogLinePayload {
    line: String,
}

// ─── TauriLogLayer ───────────────────────────────────────────────────────────

struct TauriLogLayer {
    sender: tokio::sync::mpsc::Sender<LogLinePayload>,
}

impl<S> tracing_subscriber::Layer<S> for TauriLogLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        use tracing_subscriber::field::Visit;

        struct MessageVisitor(String);
        impl Visit for MessageVisitor {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.0 = format!("{:?}", value);
                }
            }
            fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
                if field.name() == "message" {
                    self.0 = value.to_string();
                }
            }
        }

        let mut visitor = MessageVisitor(String::new());
        event.record(&mut visitor);

        let now = chrono::Utc::now();
        let level = event.metadata().level().to_string().to_uppercase();
        let target = event.metadata().target();
        let line = format!(
            "{}  {:5} {}: {}",
            now.format("%Y-%m-%dT%H:%M:%S%.6fZ"),
            level,
            target,
            visitor.0,
        );
        let payload = LogLinePayload { line };
        let _ = self.sender.try_send(payload);
    }
}

// ─── Log state ───────────────────────────────────────────────────────────────

struct LogState {
    reload_handle: Arc<reload::Handle<EnvFilter, tracing_subscriber::Registry>>,
}

fn init_logging(initial_level: &LogLevel) -> (LogState, tokio::sync::mpsc::Receiver<LogLinePayload>) {
    let mode = runtime_mode();
    let filter_str = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(initial_level.as_filter_str()));

    let (filter_layer, reload_handle) = reload::Layer::new(filter_str);

    let (tx, rx) = tokio::sync::mpsc::channel::<LogLinePayload>(1000);
    let tauri_layer = TauriLogLayer { sender: tx };

    if let Err(e) = flowstt_common::logging::ensure_log_dir() {
        eprintln!("Warning: Failed to create log directory: {}", e);
    }

    let log_path = flowstt_common::logging::app_log_path();
    let log_dir = log_path.parent().unwrap();

    let file_appender = match tracing_appender::rolling::RollingFileAppender::builder()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .max_log_files(5)
        .filename_prefix("flowstt-app")
        .filename_suffix("log")
        .build(log_dir)
    {
        Ok(appender) => appender,
        Err(e) => {
            eprintln!("Warning: Failed to create log file appender: {}", e);
            let temp_dir = std::env::temp_dir().join("flowstt-logs");
            let _ = std::fs::create_dir_all(&temp_dir);
            tracing_appender::rolling::RollingFileAppender::builder()
                .rotation(tracing_appender::rolling::Rotation::DAILY)
                .max_log_files(5)
                .filename_prefix("flowstt-app")
                .filename_suffix("log")
                .build(&temp_dir)
                .expect("Failed to create temp log file appender")
        }
    };

    let file_fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false);

    match mode {
        RuntimeMode::Production => {
            tracing_subscriber::registry()
                .with(filter_layer)
                .with(file_fmt_layer)
                .with(tauri_layer)
                .init();
        }
        RuntimeMode::Development => {
            let stdout_fmt_layer = tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_ansi(true);

            tracing_subscriber::registry()
                .with(filter_layer)
                .with(file_fmt_layer)
                .with(stdout_fmt_layer)
                .with(tauri_layer)
                .init();
        }
    }

    let log_state = LogState {
        reload_handle: Arc::new(reload_handle),
    };

    (log_state, rx)
}

// ─── Application state ───────────────────────────────────────────────────────

/// Engine held in Tauri managed state.
struct EngineState {
    engine: Arc<AudioEngine>,
}

/// IPC server handle and hotkey listener state.
struct AppState {
    ipc_server_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
    hotkey_listener: Mutex<Option<hotkey::HotkeyListener>>,
    /// Current transcription mode — used to decide how tray icon reflects state.
    transcription_mode: Mutex<TranscriptionMode>,
}

// ─── Forward EngineEvents to Tauri frontend ───────────────────────────────────

/// Forward an engine event to the Tauri frontend.
/// VisualizationData events are discarded (no visualization window).
fn forward_engine_event(app_handle: &AppHandle, event: &EngineEvent, is_ptt_mode: bool) {
    match event {
        EngineEvent::VisualizationData(data) => {
            let _ = app_handle.emit("visualization-data", data);
        }
        EngineEvent::TranscriptionComplete(result) => {
            // vtx-engine emits results without id/timestamp — enrich them
            // here so the frontend and history store have complete records.
            let id = uuid::Uuid::new_v4().to_string();
            let timestamp = chrono::Utc::now().to_rfc3339();

            let enriched = vtx_engine::TranscriptionResult {
                id: Some(id.clone()),
                text: result.text.clone(),
                timestamp: Some(timestamp.clone()),
                duration_ms: result.duration_ms,
                audio_path: result.audio_path.clone(),
                timestamp_offset_ms: result.timestamp_offset_ms,
            };

            // Persist to history
            if let Ok(mut history) = TranscriptionHistory::open("FlowSTT", 500) {
                history.append(HistoryEntry {
                    id: id.clone(),
                    text: result.text.clone(),
                    timestamp: timestamp.clone(),
                    wav_path: result.audio_path.clone(),
                });
            }

            let _ = app_handle.emit("transcription-complete", &enriched);

            // Copy to clipboard and optionally paste into the foreground app.
            // Config is loaded from disk so runtime changes take effect immediately.
            // Run on a blocking thread to avoid holding up the engine event loop.
            {
                let text = result.text.clone();
                std::thread::spawn(move || {
                    let config = flowstt_common::config::Config::load();
                    clipboard::copy_and_paste(
                        &text,
                        config.auto_paste_enabled,
                        config.auto_paste_delay_ms,
                        config.restore_clipboard_enabled,
                    );
                });
            }

            // On Windows, WebView2 can enter a frozen rendering state when
            // Alt (the default PTT key) is released while the window is focused.
            #[cfg(target_os = "windows")]
            {
                for label in ["main", "setup"] {
                    if let Some(win) = app_handle.get_webview_window(label) {
                        if let Ok(hwnd) = win.hwnd() {
                            unsafe {
                                let _ = windows::Win32::UI::WindowsAndMessaging::SendMessageW(
                                    windows::Win32::Foundation::HWND(hwnd.0),
                                    windows::Win32::UI::WindowsAndMessaging::WM_CANCELMODE,
                                    None,
                                    None,
                                );
                            }
                        }
                    }
                }
            }
        }
        EngineEvent::SpeechStarted => {
            let _ = app_handle.emit("speech-started", ());
        }
        EngineEvent::SpeechEnded { duration_ms } => {
            let _ = app_handle.emit("speech-ended", duration_ms);
        }
        EngineEvent::CaptureStateChanged { capturing, error } => {
            #[derive(serde::Serialize, Clone)]
            struct CaptureState {
                capturing: bool,
                error: Option<String>,
            }
            let _ = app_handle.emit(
                "capture-state-changed",
                CaptureState {
                    capturing: *capturing,
                    error: error.clone(),
                },
            );
            // In automatic mode, the tray icon reflects capture state (active = transcribing).
            // In PTT mode, the tray icon is driven by RecordingStarted/RecordingStopped instead.
            if !is_ptt_mode {
                tray::update_tray_icon(app_handle, *capturing);
            }
        }
        EngineEvent::ModelDownloadProgress { percent } => {
            let _ = app_handle.emit("model-download-progress", percent);
        }
        EngineEvent::ModelDownloadComplete { success } => {
            let _ = app_handle.emit("model-download-complete", success);
        }
        EngineEvent::AudioLevelUpdate { device_id, level_db } => {
            #[derive(serde::Serialize, Clone)]
            struct AudioLevel {
                device_id: String,
                level_db: f32,
            }
            let _ = app_handle.emit(
                "audio-level-update",
                AudioLevel {
                    device_id: device_id.clone(),
                    level_db: *level_db,
                },
            );
        }
        EngineEvent::TranscriptionSegment(_) => {
            // Not used in live dictation mode; ignore.
        }
        EngineEvent::RecordingStarted => {
            let _ = app_handle.emit("recording-started", ());
            if is_ptt_mode {
                tray::update_tray_icon(app_handle, true);
            }
        }
        EngineEvent::RecordingStopped { duration_ms } => {
            let _ = app_handle.emit("recording-stopped", duration_ms);
            if is_ptt_mode {
                tray::update_tray_icon(app_handle, false);
            }
        }
        EngineEvent::PlaybackComplete => {
            // No playback UI; ignore.
        }
    }
}

// ─── Tauri commands ───────────────────────────────────────────────────────────

/// List all available audio sources
#[tauri::command]
async fn list_all_sources(engine: State<'_, EngineState>) -> Result<Vec<AudioDevice>, String> {
    let mut devices = engine.engine.list_input_devices();
    devices.extend(engine.engine.list_system_devices());
    Ok(devices)
}

/// Set audio sources and start capture (persists selection to config)
#[tauri::command]
async fn set_sources(
    source1_id: Option<String>,
    source2_id: Option<String>,
    engine: State<'_, EngineState>,
) -> Result<(), String> {
    // Persist the selected device IDs so they survive restart
    let mut config = Config::load();
    config.preferred_source1_id = source1_id.clone();
    config.preferred_source2_id = source2_id.clone();
    config
        .save()
        .map_err(|e| format!("Failed to save config: {}", e))?;

    engine.engine.start_capture(source1_id, source2_id).await
}

/// Set echo cancellation enabled/disabled (persisted via config)
#[tauri::command]
async fn set_aec_enabled(enabled: bool) -> Result<(), String> {
    // AEC is now controlled via EngineConfig.recording_mode.
    // For backward compatibility: enabling AEC sets EchoCancel mode; disabling sets Mixed.
    let mut config = Config::load();
    config.recording_mode = if enabled {
        RecordingMode::EchoCancel
    } else {
        RecordingMode::Mixed
    };
    config.save().map_err(|e| format!("Failed to save config: {}", e))
}

/// Set recording mode
#[tauri::command]
async fn set_recording_mode(mode: RecordingMode) -> Result<(), String> {
    let mut config = Config::load();
    config.recording_mode = mode;
    config.save().map_err(|e| format!("Failed to save config: {}", e))
}

/// Check Whisper model status
#[tauri::command]
async fn check_model_status(engine: State<'_, EngineState>) -> Result<ModelStatus, String> {
    Ok(engine.engine.check_model_status())
}

/// Download the Whisper model
#[tauri::command]
async fn download_model(engine: State<'_, EngineState>) -> Result<(), String> {
    engine.engine.download_model().await
}

/// Local CUDA status struct for frontend compatibility
#[derive(serde::Serialize)]
struct LocalCudaStatus {
    build_enabled: bool,
    runtime_available: bool,
    system_info: String,
}

/// Get CUDA/GPU acceleration status
#[tauri::command]
async fn get_cuda_status(engine: State<'_, EngineState>) -> Result<LocalCudaStatus, String> {
    let status = engine.engine.check_gpu_status()?;
    Ok(LocalCudaStatus {
        build_enabled: status.cuda_available || status.metal_available,
        runtime_available: status.cuda_available || status.metal_available,
        system_info: status.system_info,
    })
}

/// Status struct for frontend
#[derive(serde::Serialize)]
struct LocalStatus {
    capturing: bool,
    in_speech: bool,
    queue_depth: usize,
    error: Option<String>,
    source1_id: Option<String>,
    source2_id: Option<String>,
    transcription_mode: TranscriptionMode,
}

/// Get current status
#[tauri::command]
async fn get_status(engine: State<'_, EngineState>) -> Result<LocalStatus, String> {
    let status = engine.engine.get_status();
    let config = Config::load();
    Ok(LocalStatus {
        capturing: status.capturing,
        in_speech: status.in_speech,
        queue_depth: status.queue_depth,
        error: status.error,
        source1_id: status.source1_id,
        source2_id: status.source2_id,
        transcription_mode: config.transcription_mode,
    })
}

/// Push-to-talk status for frontend
#[derive(serde::Serialize)]
struct LocalPttStatus {
    mode: TranscriptionMode,
    hotkeys: Vec<HotkeyCombination>,
    auto_toggle_hotkeys: Vec<HotkeyCombination>,
    auto_mode_active: bool,
    is_active: bool,
    available: bool,
    error: Option<String>,
}

/// Set the transcription mode and start/stop the hotkey listener accordingly.
#[tauri::command]
async fn set_transcription_mode(
    mode: TranscriptionMode,
    app_handle: AppHandle,
    engine: State<'_, EngineState>,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = Config::load();
    config.transcription_mode = mode;
    config.save().map_err(|e| format!("Failed to save config: {}", e))?;
    apply_transcription_mode(mode, &engine.engine, &app_state, &config.ptt_hotkeys, Some(&app_handle)).await;
    Ok(())
}

/// Set the push-to-talk hotkey combinations
#[tauri::command]
async fn set_ptt_hotkeys(
    hotkeys: Vec<HotkeyCombination>,
    app_handle: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = Config::load();
    config.ptt_hotkeys = hotkeys.clone();
    config.save().map_err(|e| format!("Failed to save config: {}", e))?;
    // Update the running listener if one exists
    if let Some(ref listener) = *app_state.hotkey_listener.lock().await {
        listener.update_combos(hotkeys);
    }
    let _ = app_handle.emit("ptt-hotkeys-changed", ());
    Ok(())
}

/// Get push-to-talk status
#[tauri::command]
async fn get_ptt_status(engine: State<'_, EngineState>) -> Result<LocalPttStatus, String> {
    let config = Config::load();
    Ok(LocalPttStatus {
        mode: config.transcription_mode,
        hotkeys: config.ptt_hotkeys,
        auto_toggle_hotkeys: config.auto_toggle_hotkeys,
        auto_mode_active: false,
        is_active: engine.engine.is_recording(),
        available: true,
        error: None,
    })
}

/// Set the auto-mode toggle hotkeys
#[tauri::command]
async fn set_auto_toggle_hotkeys(hotkeys: Vec<HotkeyCombination>) -> Result<(), String> {
    let mut config = Config::load();
    config.auto_toggle_hotkeys = hotkeys;
    config.save().map_err(|e| format!("Failed to save config: {}", e))
}

/// Toggle between Automatic and PushToTalk modes
#[tauri::command]
async fn toggle_auto_mode(
    app_handle: AppHandle,
    engine: State<'_, EngineState>,
    app_state: State<'_, AppState>,
) -> Result<TranscriptionMode, String> {
    let mut config = Config::load();
    config.transcription_mode = match config.transcription_mode {
        TranscriptionMode::Automatic => TranscriptionMode::PushToTalk,
        TranscriptionMode::PushToTalk => TranscriptionMode::Automatic,
    };
    let new_mode = config.transcription_mode;
    config.save().map_err(|e| format!("Failed to save config: {}", e))?;
    apply_transcription_mode(new_mode, &engine.engine, &app_state, &config.ptt_hotkeys, Some(&app_handle)).await;
    Ok(new_mode)
}

/// Apply a transcription mode change: start/stop the hotkey listener and
/// enable/disable VAD-driven transcription as appropriate.
async fn apply_transcription_mode(
    mode: TranscriptionMode,
    engine: &Arc<AudioEngine>,
    app_state: &AppState,
    ptt_hotkeys: &[HotkeyCombination],
    app_handle: Option<&AppHandle>,
) {
    // Update the shared mode so the event-forwarder task can read it.
    {
        let mut mode_lock = app_state.transcription_mode.lock().await;
        *mode_lock = mode;
    }

    let mut listener_lock = app_state.hotkey_listener.lock().await;
    match mode {
        TranscriptionMode::PushToTalk => {
            // Stop any in-progress recording, disable VAD transcription
            engine.stop_recording();
            engine.set_transcription_enabled(false);

            // Reset tray icon to non-recording since PTT is not being held
            if let Some(ah) = app_handle {
                tray::update_tray_icon(ah, false);
            }

            // Start hotkey listener if not already running
            if listener_lock.is_none() {
                let listener = hotkey::HotkeyListener::start(
                    engine.clone(),
                    ptt_hotkeys.to_vec(),
                );
                *listener_lock = Some(listener);
                info!("[Mode] Switched to PTT — hotkey listener started");
            } else {
                info!("[Mode] Switched to PTT — hotkey listener already running");
            }
        }
        TranscriptionMode::Automatic => {
            // Stop hotkey listener and any in-progress manual recording
            if let Some(listener) = listener_lock.take() {
                listener.stop();
            }
            engine.stop_recording();
            engine.set_transcription_enabled(true);

            // Tray icon reflects capture state in automatic mode
            if let Some(ah) = app_handle {
                tray::update_tray_icon(ah, engine.is_capturing());
            }

            info!("[Mode] Switched to Automatic — VAD transcription enabled");
        }
    }
}

/// Get all persisted configuration values
#[tauri::command]
async fn get_config() -> Result<ConfigValues, String> {
    let config = Config::load();
    Ok(ConfigValues {
        transcription_mode: config.transcription_mode,
        ptt_hotkeys: config.ptt_hotkeys,
        auto_toggle_hotkeys: config.auto_toggle_hotkeys,
        auto_paste_enabled: config.auto_paste_enabled,
        auto_paste_delay_ms: config.auto_paste_delay_ms,
        restore_clipboard_enabled: config.restore_clipboard_enabled,
        mic_gain: config.mic_gain,
        preferred_source1_id: config.preferred_source1_id,
        preferred_source2_id: config.preferred_source2_id,
    })
}

/// Enable or disable clipboard save/restore around transcription paste
#[tauri::command]
async fn set_restore_clipboard(enabled: bool) -> Result<(), String> {
    let mut config = Config::load();
    config.restore_clipboard_enabled = enabled;
    config.save().map_err(|e| format!("Failed to save config: {}", e))
}

/// Set the microphone input gain multiplier (1.0–4.0)
#[tauri::command]
async fn set_mic_gain(gain: f32) -> Result<(), String> {
    let mut config = Config::load();
    config.mic_gain = gain;
    config.save().map_err(|e| format!("Failed to save config: {}", e))
}

/// History entry struct for frontend compatibility
#[derive(serde::Serialize, serde::Deserialize)]
struct LocalHistoryEntry {
    id: String,
    text: String,
    timestamp: String,
    wav_path: Option<String>,
}

/// Get transcription history
#[tauri::command]
async fn get_history() -> Result<Vec<LocalHistoryEntry>, String> {
    let history = ipc::history::load_history()?;
    Ok(history
        .into_iter()
        .map(|e| LocalHistoryEntry {
            id: e.id,
            text: e.text,
            timestamp: e.timestamp,
            wav_path: e.wav_path,
        })
        .collect())
}

/// Delete a history entry
#[tauri::command]
async fn delete_history_entry(id: String) -> Result<(), String> {
    ipc::history::delete_history_entry(&id)
}

/// Get the current theme mode from the config file.
#[tauri::command]
fn get_theme_mode() -> Result<ThemeMode, String> {
    let config = Config::load();
    Ok(config.theme_mode)
}

/// Set the theme mode and persist to the config file.
#[tauri::command]
fn set_theme_mode(mode: ThemeMode, app_handle: AppHandle) -> Result<(), String> {
    let mut config = Config::load();
    config.theme_mode = mode.clone();
    config
        .save()
        .map_err(|e| format!("Failed to save config: {}", e))?;
    app_handle
        .emit("theme-changed", &mode)
        .map_err(|e| format!("Failed to emit theme event: {}", e))?;
    Ok(())
}

/// Check if first-time setup is needed.
#[tauri::command]
fn needs_setup() -> bool {
    Config::needs_setup()
}

/// Get the current runtime mode.
#[tauri::command]
fn get_runtime_mode() -> String {
    runtime_mode().as_str().to_string()
}

/// Cancel any pending Win32 menu activation mode on a window.
#[tauri::command]
fn cancel_menu_mode(_window: tauri::WebviewWindow) {
    #[cfg(target_os = "windows")]
    {
        if let Ok(hwnd) = _window.hwnd() {
            unsafe {
                let _ = windows::Win32::UI::WindowsAndMessaging::SendMessageW(
                    windows::Win32::Foundation::HWND(hwnd.0),
                    windows::Win32::UI::WindowsAndMessaging::WM_CANCELMODE,
                    None,
                    None,
                );
            }
        }
    }
}

/// Complete the first-time setup wizard.
#[tauri::command]
async fn complete_setup(
    transcription_mode: TranscriptionMode,
    hotkeys: Vec<HotkeyCombination>,
    source1_id: Option<String>,
    source2_id: Option<String>,
    app_handle: AppHandle,
    engine: State<'_, EngineState>,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = Config::default_with_hotkeys();
    config.transcription_mode = transcription_mode;
    config.ptt_hotkeys = hotkeys.clone();
    config.preferred_source1_id = source1_id.clone();
    config.preferred_source2_id = source2_id.clone();
    config
        .save()
        .map_err(|e| format!("Failed to save config: {}", e))?;

    // Start capture with selected sources
    if source1_id.is_some() || source2_id.is_some() {
        engine.engine.start_capture(source1_id, source2_id).await?;
    }

    // Apply transcription mode (starts hotkey listener for PTT, enables VAD
    // transcription for automatic)
    apply_transcription_mode(
        transcription_mode,
        &engine.engine,
        &app_state,
        &hotkeys,
        Some(&app_handle),
    )
    .await;

    app_handle
        .emit("setup-complete", ())
        .map_err(|e| format!("Failed to emit event: {}", e))?;

    Ok(())
}

/// Start a manual recording session (for push-to-talk).
///
/// Audio is accumulated until `stop_recording` is called, then submitted
/// for transcription.
#[tauri::command]
async fn start_recording(engine: State<'_, EngineState>) -> Result<(), String> {
    engine.engine.start_recording();
    Ok(())
}

/// Stop the manual recording session and submit audio for transcription.
#[tauri::command]
async fn stop_recording(engine: State<'_, EngineState>) -> Result<(), String> {
    engine.engine.stop_recording();
    Ok(())
}

/// Check if a manual recording session is active.
#[tauri::command]
async fn is_recording(engine: State<'_, EngineState>) -> Result<bool, String> {
    Ok(engine.engine.is_recording())
}

/// Start a test audio capture on a device for level metering.
#[tauri::command]
async fn test_audio_device(
    device_id: String,
    engine: State<'_, EngineState>,
) -> Result<(), String> {
    engine.engine.start_test_capture(device_id)
}

/// Stop any active test audio capture.
#[tauri::command]
async fn stop_test_audio_device(engine: State<'_, EngineState>) -> Result<(), String> {
    engine.engine.stop_test_capture()
}

/// Open System Settings to the Accessibility pane on macOS.
#[tauri::command]
async fn open_accessibility_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        ipc::hotkey::request_accessibility_permission();
    }
    Ok(())
}

/// Check whether this process has macOS Accessibility permission.
#[tauri::command]
async fn check_accessibility_permission() -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        Ok(ipc::hotkey::check_accessibility_permission())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(true)
    }
}

/// Return the raw text of the current session's log file.
#[tauri::command]
fn get_log_history() -> String {
    let log_dir = flowstt_common::logging::app_log_path()
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::temp_dir().join("flowstt-logs"));

    let most_recent = std::fs::read_dir(&log_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            name.starts_with("flowstt-app.") && name.ends_with(".log")
        })
        .filter_map(|e| {
            let meta = e.metadata().ok()?;
            let modified = meta.modified().ok()?;
            Some((e.path(), modified))
        })
        .max_by_key(|(_, modified)| *modified)
        .map(|(path, _)| path);

    match most_recent {
        Some(path) => std::fs::read_to_string(&path).unwrap_or_default(),
        None => String::new(),
    }
}

/// Get the current log level from config.
#[tauri::command]
fn get_log_level() -> Result<String, String> {
    let config = Config::load();
    Ok(config.log_level.as_filter_str().to_string())
}

/// Set the minimum log level at runtime and persist to config.
#[tauri::command]
fn set_log_level(level: String, state: State<LogState>) -> Result<(), String> {
    let log_level: LogLevel = match level.as_str() {
        "error" => LogLevel::Error,
        "warn" => LogLevel::Warn,
        "info" => LogLevel::Info,
        "debug" => LogLevel::Debug,
        "trace" => LogLevel::Trace,
        other => return Err(format!("Unknown log level: {}", other)),
    };

    state
        .reload_handle
        .reload(EnvFilter::new(log_level.as_filter_str()))
        .map_err(|e| format!("Failed to reload log filter: {}", e))?;

    let mut config = Config::load();
    config.log_level = log_level;
    config
        .save()
        .map_err(|e| format!("Failed to save config: {}", e))?;

    Ok(())
}

/// Download all log files as a zip archive via a native save dialog.
#[tauri::command]
async fn download_logs(app_handle: AppHandle) -> Result<(), String> {
    use std::io::Write;
    use tauri_plugin_dialog::DialogExt;

    let log_dir = flowstt_common::logging::app_log_path()
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::temp_dir().join("flowstt-logs"));

    let entries: Vec<std::path::PathBuf> = match std::fs::read_dir(&log_dir) {
        Ok(dir) => dir
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("log"))
            .collect(),
        Err(_) => vec![],
    };

    if entries.is_empty() {
        return Err("no_logs".to_string());
    }

    let mut zip_buf = std::io::Cursor::new(Vec::<u8>::new());
    {
        let mut zip = zip::ZipWriter::new(&mut zip_buf);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for path in &entries {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if let Ok(contents) = std::fs::read(path) {
                    let _ = zip.start_file(name, options);
                    let _ = zip.write_all(&contents);
                }
            }
        }
        zip.finish().map_err(|e| format!("Zip error: {}", e))?;
    }

    let zip_bytes = zip_buf.into_inner();
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let default_name = format!("flowstt-logs-{}.zip", today);

    let (tx, rx) = tokio::sync::oneshot::channel::<Option<std::path::PathBuf>>();
    app_handle
        .dialog()
        .file()
        .set_file_name(&default_name)
        .save_file(move |path| {
            let _ = tx.send(path.and_then(|p| p.into_path().ok()));
        });

    match rx.await {
        Ok(Some(dest)) => {
            std::fs::write(&dest, &zip_bytes)
                .map_err(|e| format!("Failed to write zip: {}", e))?;
        }
        Ok(None) => {}
        Err(_) => return Err("Dialog channel error".to_string()),
    }

    Ok(())
}

// ─── App updater commands ────────────────────────────────────────────────────

#[derive(serde::Serialize, Clone)]
struct UpdateInfo {
    available: bool,
    version: Option<String>,
    date: Option<String>,
    notes: Option<String>,
}

#[tauri::command]
#[cfg(desktop)]
async fn check_for_updates(app: AppHandle) -> Result<UpdateInfo, String> {
    use tauri_plugin_updater::UpdaterExt;
    match app.updater().map_err(|e| e.to_string())?.check().await {
        Ok(Some(update)) => Ok(UpdateInfo {
            available: true,
            version: Some(update.version.clone()),
            date: update.date.map(|d| d.to_string()),
            notes: update.body.clone(),
        }),
        Ok(None) => Ok(UpdateInfo {
            available: false,
            version: None,
            date: None,
            notes: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
#[cfg(not(desktop))]
async fn check_for_updates(_app: AppHandle) -> Result<UpdateInfo, String> {
    Ok(UpdateInfo {
        available: false,
        version: None,
        date: None,
        notes: None,
    })
}

#[tauri::command]
#[cfg(desktop)]
async fn install_update(app: AppHandle) -> Result<(), String> {
    use tauri_plugin_updater::UpdaterExt;

    let update = app
        .updater()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No update available".to_string())?;

    let app_handle = app.clone();
    update
        .download_and_install(
            move |chunk_length, content_length| {
                let _ = app_handle.emit(
                    "update-download-progress",
                    serde_json::json!({
                        "chunkLength": chunk_length,
                        "contentLength": content_length
                    }),
                );
            },
            || {},
        )
        .await
        .map_err(|e| e.to_string())?;

    #[allow(unreachable_code)]
    {
        app.restart();
        Ok(())
    }
}

#[tauri::command]
#[cfg(not(desktop))]
async fn install_update(_app: AppHandle) -> Result<(), String> {
    Err("Updates not supported on this platform".to_string())
}

/// Connect events -- no-op (events forwarded from engine broadcast loop)
#[tauri::command]
async fn connect_events() -> Result<(), String> {
    debug!("[Startup] connect_events: no-op (events forwarded from vtx-engine broadcast loop)");
    Ok(())
}

/// Log a startup diagnostic message from the frontend.
#[tauri::command]
fn startup_log(message: String) {
    info!("[Startup/JS] {}", message);
}

/// Log a message from the frontend to the log file.
#[tauri::command]
fn log_to_file(level: String, message: String) {
    match level.as_str() {
        "error" => error!("{}", message),
        "warn" => warn!("{}", message),
        "info" => info!("{}", message),
        "debug" => debug!("{}", message),
        _ => info!("{}", message),
    }
}

// ─── Window helpers ──────────────────────────────────────────────────────────

pub fn open_log_viewer_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("logs") {
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }

    let _ = WebviewWindowBuilder::new(app, "logs", WebviewUrl::App("logs.html".into()))
        .title("FlowSTT Logs")
        .inner_size(900.0, 600.0)
        .min_inner_size(600.0, 400.0)
        .resizable(true)
        .decorations(false)
        .transparent(false)
        .shadow(true)
        .skip_taskbar(true)
        .center()
        .build();
}

// ─── Application entry point ─────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_t0 = Instant::now();

    let headless = std::env::args().any(|arg| arg == "--headless");
    // --test-mode flag is no longer supported (test_mode was part of flowstt-engine)

    let initial_config = Config::load();

    let (log_state, log_rx) = init_logging(&initial_config.log_level);

    info!(
        "[Startup] run() entered (headless={})",
        headless
    );
    configure_wayland_workarounds();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .manage(AppState {
            ipc_server_handle: Mutex::new(None),
            hotkey_listener: Mutex::new(None),
            transcription_mode: Mutex::new(initial_config.transcription_mode),
        })
        .manage(log_state)
        .setup(move |app| {
            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;

            // Spawn the log-line forwarder task
            {
                let app_handle = app.handle().clone();
                let mut rx = log_rx;
                tauri::async_runtime::spawn(async move {
                    while let Some(payload) = rx.recv().await {
                        let _ = app_handle.emit("log-line", payload);
                    }
                });
            }

            debug!(
                "[Startup] setup() hook called (+{}ms from run())",
                app_t0.elapsed().as_millis()
            );

            // Set the VTX resource dir on Windows for whisper.cpp binary loading
            #[cfg(windows)]
            if let Ok(resource_dir) = app.path().resource_dir() {
                env::set_var("VTX_RESOURCE_DIR", resource_dir);
            }

            let app_handle = app.handle().clone();

            // Migrate whisper model from legacy location if needed.
            // Pre-vtx-engine builds stored the model at {cache}/whisper/;
            // vtx-engine uses {cache}/FlowSTT/whisper/.
            migrate_whisper_model();

            // Build vtx-engine with config from FlowSTT app config
            let engine_config = build_engine_config(&initial_config);
            let (engine, event_rx) = tauri::async_runtime::block_on(async {
                EngineBuilder::from_config(engine_config)
                    .app_name("FlowSTT")
                    .build()
                    .await
            })
            .map_err(|e| {
                error!("[Startup] Failed to build engine: {}", e);
                e
            })?;

            let engine = Arc::new(engine);

            // Register engine as managed state
            app.manage(EngineState { engine: engine.clone() });

            // Spawn the EngineEvent forwarding task
            {
                let app_handle_clone = app_handle.clone();
                let mut rx = event_rx;
                let initial_is_ptt = initial_config.transcription_mode == TranscriptionMode::PushToTalk;
                tauri::async_runtime::spawn(async move {
                    loop {
                        match rx.recv().await {
                            Ok(event) => {
                                // Read the current mode from AppState (may change at runtime).
                                let is_ptt = app_handle_clone
                                    .try_state::<AppState>()
                                    .map(|s| {
                                        s.transcription_mode
                                            .try_lock()
                                            .map(|m| *m == TranscriptionMode::PushToTalk)
                                            .unwrap_or(initial_is_ptt)
                                    })
                                    .unwrap_or(initial_is_ptt);
                                forward_engine_event(&app_handle_clone, &event, is_ptt);
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                                warn!("[Engine] Event receiver lagged, dropped {} events", n);
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                                debug!("[Engine] Event broadcast channel closed");
                                break;
                            }
                        }
                    }
                });
            }

            // Initialize the IPC socket server for CLI clients
            let ipc_handle = tauri::async_runtime::block_on(async {
                ipc::server::start(engine.clone()).await
            });

            match ipc_handle {
                Ok(handle) => {
                    let state: State<AppState> = app.state();
                    let mut lock = tauri::async_runtime::block_on(state.ipc_server_handle.lock());
                    *lock = Some(handle);
                    info!("[Startup] IPC server started");
                }
                Err(e) => {
                    warn!("[Startup] IPC server failed to start: {}", e);
                }
            }

            // Set up the system tray
            if let Err(e) = tray::setup_tray(app) {
                warn!("[FlowSTT] Failed to set up system tray: {}", e);
            }

            // Restore always-on-top state from config
            {
                let config = Config::load();
                if config.always_on_top {
                    if let Some(main_win) = app.get_webview_window("main") {
                        if let Err(e) = main_win.set_always_on_top(true) {
                            warn!("[Startup] Failed to restore always-on-top: {}", e);
                        }
                    }
                }
            }

            // Auto-start capture if setup is complete
            let first_run = Config::needs_setup();
            if !first_run {
                let config = Config::load();
                let engine_clone = engine.clone();

                // In PTT mode, disable VAD-driven transcription so speech
                // detection does not auto-submit segments.  Recording is
                // exclusively driven by the global hotkey listener below.
                let is_ptt = config.transcription_mode == TranscriptionMode::PushToTalk;
                if is_ptt {
                    engine_clone.set_transcription_enabled(false);
                    info!("[Startup] PTT mode — VAD transcription disabled");
                }

                tauri::async_runtime::block_on(async move {
                    let input_devices = engine_clone.list_input_devices();
                    info!(
                        "[Startup] {} input device(s) available, preferred_source1_id={:?}",
                        input_devices.len(),
                        config.preferred_source1_id,
                    );
                    let source1_id = if let Some(pref) = config.preferred_source1_id.as_deref() {
                        input_devices.iter().find(|d| d.id == pref).map(|d| d.id.clone())
                            .or_else(|| {
                                warn!("[Startup] Preferred source1 '{}' not found, falling back", pref);
                                input_devices.first().map(|d| d.id.clone())
                            })
                    } else {
                        input_devices.first().map(|d| d.id.clone())
                    };

                    let source2_id = config.preferred_source2_id.as_deref().and_then(|pref| {
                        engine_clone.list_system_devices()
                            .into_iter()
                            .find(|d| d.id == pref)
                            .map(|d| d.id)
                    });

                    if let Some(ref s1) = source1_id {
                        info!("[Startup] Starting capture with source1={}, source2={:?}", s1, source2_id);
                        match engine_clone.start_capture(source1_id, source2_id).await {
                            Ok(()) => info!("[Startup] Auto-capture started successfully"),
                            Err(e) => error!("[Startup] Auto-capture failed: {}", e),
                        }
                    } else {
                        warn!("[Startup] No audio input devices found — PTT will not work");
                    }
                });

                // Start global hotkey listener for PTT
                if is_ptt {
                    let listener = hotkey::HotkeyListener::start(
                        engine.clone(),
                        config.ptt_hotkeys.clone(),
                    );
                    let app_state: State<AppState> = app.state();
                    *tauri::async_runtime::block_on(app_state.hotkey_listener.lock()) =
                        Some(listener);
                    info!("[Startup] PTT hotkey listener started");
                }
            }

            // First-run detection: show setup wizard
            if first_run && !headless {
                info!("[Startup] First run detected - showing setup wizard");

                if let Some(main_win) = app.get_webview_window("main") {
                    let _ = main_win.hide();
                }

                let _setup_win =
                    WebviewWindowBuilder::new(app, "setup", WebviewUrl::App("setup.html".into()))
                        .title("FlowSTT Setup")
                        .inner_size(600.0, 500.0)
                        .min_inner_size(500.0, 400.0)
                        .center()
                        .decorations(true)
                        .transparent(false)
                        .shadow(true)
                        .visible(true)
                        .build()
                        .expect("Failed to create setup window");

                let app_handle_inner = app.handle().clone();
                app.listen("setup-complete", move |_event| {
                    info!("[Startup] Setup complete - transitioning to main window");

                    if let Some(setup_win) = app_handle_inner.get_webview_window("setup") {
                        let _ = setup_win.destroy();
                    }

                    if let Some(main_win) = app_handle_inner.get_webview_window("main") {
                        let _ = main_win.show();
                        let _ = main_win.set_focus();
                    }
                });
            } else if headless {
                info!("[Startup] Headless mode - hiding main window");
                if let Some(main_win) = app.get_webview_window("main") {
                    let _ = main_win.hide();
                }
            }

            // Deferred background update check (release builds only)
            #[cfg(all(desktop, not(debug_assertions)))]
            {
                let app_handle_upd = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    use tauri_plugin_updater::UpdaterExt;
                    match app_handle_upd.updater() {
                        Ok(updater) => match updater.check().await {
                            Ok(Some(update)) => {
                                info!("[Updater] Update available: v{}", update.version);
                                let payload = UpdateInfo {
                                    available: true,
                                    version: Some(update.version.clone()),
                                    date: update.date.map(|d| d.to_string()),
                                    notes: update.body.clone(),
                                };
                                let _ = app_handle_upd.emit("update-available", payload);
                            }
                            Ok(None) => debug!("[Updater] App is up to date"),
                            Err(e) => warn!("[Updater] Update check failed: {}", e),
                        },
                        Err(e) => warn!("[Updater] Failed to get updater: {}", e),
                    }
                });
            }

            debug!(
                "[Startup] setup() hook done (+{}ms from run())",
                app_t0.elapsed().as_millis()
            );
            Ok(())
        })
        .on_window_event(|_window, _event| {
            #[cfg(windows)]
            if let tauri::WindowEvent::CloseRequested { api, .. } = _event {
                if _window.label() == "main" {
                    api.prevent_close();
                    let _ = _window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            startup_log,
            log_to_file,
            get_log_history,
            get_log_level,
            set_log_level,
            download_logs,
            list_all_sources,
            set_sources,
            set_aec_enabled,
            set_recording_mode,
            check_model_status,
            download_model,
            get_status,
            get_cuda_status,
            set_transcription_mode,
            set_ptt_hotkeys,
            get_ptt_status,
            set_auto_toggle_hotkeys,
            toggle_auto_mode,
            get_config,
            set_restore_clipboard,
            set_mic_gain,
            start_recording,
            stop_recording,
            is_recording,
            get_history,
            delete_history_entry,
            connect_events,
            get_theme_mode,
            set_theme_mode,
            needs_setup,
            get_runtime_mode,
            cancel_menu_mode,
            complete_setup,
            test_audio_device,
            stop_test_audio_device,
            check_accessibility_permission,
            open_accessibility_settings,
            check_for_updates,
            install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    info!("FlowSTT stopped");
}

/// Migrate the whisper model from the legacy location to the new app-scoped
/// location if the model exists at the old path but not the new one.
///
/// Legacy path:  `{cache_dir}/whisper/ggml-base.en.bin`
/// New path:     `{cache_dir}/FlowSTT/whisper/ggml-base.en.bin`
fn migrate_whisper_model() {
    let Some(base) = directories::BaseDirs::new().map(|d| d.cache_dir().to_path_buf()) else {
        return;
    };

    let model_name = "ggml-base.en.bin";
    let legacy_path = base.join("whisper").join(model_name);
    let new_dir = base.join("FlowSTT").join("whisper");
    let new_path = new_dir.join(model_name);

    if new_path.exists() || !legacy_path.exists() {
        return;
    }

    info!(
        "[Migration] Copying whisper model from {} to {}",
        legacy_path.display(),
        new_path.display()
    );

    if let Err(e) = std::fs::create_dir_all(&new_dir) {
        warn!("[Migration] Failed to create directory {}: {}", new_dir.display(), e);
        return;
    }

    match std::fs::copy(&legacy_path, &new_path) {
        Ok(bytes) => info!("[Migration] Model copied successfully ({} bytes)", bytes),
        Err(e) => warn!("[Migration] Failed to copy model: {}", e),
    }
}

/// Build an EngineConfig from the FlowSTT AppConfig.
fn build_engine_config(config: &Config) -> EngineConfig {
    let mut engine_config = EngineConfig::load("FlowSTT").unwrap_or_default();
    // Apply FlowSTT app-config overrides that affect engine behaviour
    engine_config.recording_mode = config.recording_mode;
    // word_break_segmentation_enabled defaults to true in EngineConfig::default()
    engine_config
}
