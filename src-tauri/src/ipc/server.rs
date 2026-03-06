//! FlowSTT IPC socket server.
//!
//! Hosts the named pipe / Unix socket that CLI clients connect to.
//! Handles the FlowSTT IPC request/response protocol using vtx-engine.

use std::sync::Arc;
use tracing::{debug, error, info, warn};
use vtx_engine::AudioEngine;
use vtx_common::AudioSourceType;
use flowstt_common::config::Config;
use flowstt_common::ipc::{Request, Response};
use flowstt_common::{ConfigValues, runtime_mode};
use super::history;

// ─── Platform IPC ─────────────────────────────────────────────────────────────

/// Start the IPC server and return a JoinHandle.
pub async fn start(engine: Arc<AudioEngine>) -> Result<tokio::task::JoinHandle<()>, String> {
    let socket_path = flowstt_common::ipc::get_socket_path();

    // Clean up stale socket on Unix
    #[cfg(unix)]
    {
        if socket_path.exists() {
            let _ = std::fs::remove_file(&socket_path);
        }
        if let Some(parent) = socket_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
    }

    let handle = tokio::spawn(async move {
        if let Err(e) = run_server(engine, socket_path).await {
            error!("[IPC] Server exited with error: {}", e);
        }
    });

    Ok(handle)
}

#[cfg(unix)]
async fn run_server(engine: Arc<AudioEngine>, socket_path: std::path::PathBuf) -> Result<(), String> {
    use tokio::net::UnixListener;
    let listener = UnixListener::bind(&socket_path)
        .map_err(|e| format!("Failed to bind IPC socket {:?}: {}", socket_path, e))?;
    info!("[IPC] Listening on {:?}", socket_path);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let engine_clone = engine.clone();
                tokio::spawn(async move {
                    handle_connection_unix(stream, engine_clone).await;
                });
            }
            Err(e) => {
                warn!("[IPC] Accept error: {}", e);
            }
        }
    }
}

#[cfg(target_os = "windows")]
async fn run_server(engine: Arc<AudioEngine>, _socket_path: std::path::PathBuf) -> Result<(), String> {
    use tokio::net::windows::named_pipe::{ServerOptions};
    let pipe_name = r"\\.\pipe\flowstt-service";
    info!("[IPC] Listening on {}", pipe_name);

    loop {
        let server = ServerOptions::new()
            .first_pipe_instance(false)
            .create(pipe_name)
            .map_err(|e| format!("Failed to create named pipe: {}", e))?;

        server.connect().await
            .map_err(|e| format!("Named pipe connect failed: {}", e))?;

        let engine_clone = engine.clone();
        tokio::spawn(async move {
            handle_connection_windows(server, engine_clone).await;
        });
    }
}

// ─── Connection handling ──────────────────────────────────────────────────────

#[cfg(unix)]
async fn handle_connection_unix(stream: tokio::net::UnixStream, engine: Arc<AudioEngine>) {
    use flowstt_common::ipc::protocol::{read_json, write_json};
    use tokio::io::split;

    let (mut reader, mut writer) = split(stream);
    debug!("[IPC] New connection");

    loop {
        match read_json::<_, Request>(&mut reader).await {
            Ok(request) => {
                debug!("[IPC] Request: {:?}", request);
                let response = handle_request(request, &engine).await;
                if let Err(e) = write_json(&mut writer, &response).await {
                    debug!("[IPC] Write error: {}", e);
                    break;
                }
            }
            Err(flowstt_common::ipc::protocol::IpcError::ConnectionClosed) => {
                debug!("[IPC] Client disconnected");
                break;
            }
            Err(e) => {
                warn!("[IPC] Read error: {}", e);
                break;
            }
        }
    }
}

#[cfg(target_os = "windows")]
async fn handle_connection_windows(
    pipe: tokio::net::windows::named_pipe::NamedPipeServer,
    engine: Arc<AudioEngine>,
) {
    use flowstt_common::ipc::protocol::{read_json, write_json};
    use tokio::io::split;

    let (mut reader, mut writer) = split(pipe);
    debug!("[IPC] New named pipe connection");

    loop {
        match read_json::<_, Request>(&mut reader).await {
            Ok(request) => {
                let response = handle_request(request, &engine).await;
                if let Err(e) = write_json(&mut writer, &response).await {
                    debug!("[IPC] Write error: {}", e);
                    break;
                }
            }
            Err(flowstt_common::ipc::protocol::IpcError::ConnectionClosed) => {
                debug!("[IPC] Client disconnected");
                break;
            }
            Err(e) => {
                warn!("[IPC] Read error: {}", e);
                break;
            }
        }
    }
}

// ─── Request dispatch ────────────────────────────────────────────────────────

async fn handle_request(request: Request, engine: &Arc<AudioEngine>) -> Response {
    match request {
        Request::Ping => Response::Pong,

        Request::ListDevices { source_type } => {
            let devices = match source_type {
                None => {
                    let mut all = engine.list_input_devices();
                    all.extend(engine.list_system_devices());
                    all
                }
                Some(AudioSourceType::Input) => engine.list_input_devices(),
                Some(AudioSourceType::System) => engine.list_system_devices(),
                Some(AudioSourceType::Mixed) => {
                    let mut all = engine.list_input_devices();
                    all.extend(engine.list_system_devices());
                    all
                }
            };
            Response::Devices { devices }
        }

        Request::SetSources { source1_id, source2_id } => {
            match engine.start_capture(source1_id, source2_id).await {
                Ok(()) => Response::Ok,
                Err(e) => Response::error(e),
            }
        }

        Request::GetStatus => {
            let status = engine.get_status();
            let config = Config::load();
            Response::Status(flowstt_common::TranscribeStatus {
                capturing: status.capturing,
                in_speech: status.in_speech,
                queue_depth: status.queue_depth,
                error: status.error,
                source1_id: status.source1_id,
                source2_id: status.source2_id,
                transcription_mode: config.transcription_mode,
            })
        }

        Request::GetModelStatus => {
            let status = engine.check_model_status();
            Response::ModelStatus(status)
        }

        Request::DownloadModel => {
            match engine.download_model().await {
                Ok(()) => Response::Ok,
                Err(e) => Response::error(e),
            }
        }

        Request::GetCudaStatus => {
            match engine.check_gpu_status() {
                Ok(gpu) => Response::CudaStatus(flowstt_common::CudaStatus {
                    build_enabled: gpu.cuda_available || gpu.metal_available,
                    runtime_available: gpu.cuda_available || gpu.metal_available,
                    system_info: gpu.system_info,
                }),
                Err(e) => Response::error(e),
            }
        }

        Request::GetConfig => {
            let config = Config::load();
            Response::ConfigValues(ConfigValues {
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

        Request::SetTranscriptionMode { mode } => {
            let mut config = Config::load();
            config.transcription_mode = mode;
            match config.save() {
                Ok(()) => Response::Ok,
                Err(e) => Response::error(format!("Failed to save config: {}", e)),
            }
        }

        Request::SetPushToTalkHotkeys { hotkeys } => {
            let mut config = Config::load();
            config.ptt_hotkeys = hotkeys;
            match config.save() {
                Ok(()) => Response::Ok,
                Err(e) => Response::error(format!("Failed to save config: {}", e)),
            }
        }

        Request::GetPttStatus => {
            let config = Config::load();
            Response::PttStatus(flowstt_common::PttStatus {
                mode: config.transcription_mode,
                hotkeys: config.ptt_hotkeys,
                auto_toggle_hotkeys: config.auto_toggle_hotkeys,
                auto_mode_active: false,
                is_active: false,
                available: true,
                error: None,
                accessibility_permission_granted: true,
            })
        }

        Request::SetAutoToggleHotkeys { hotkeys } => {
            let mut config = Config::load();
            config.auto_toggle_hotkeys = hotkeys;
            match config.save() {
                Ok(()) => Response::Ok,
                Err(e) => Response::error(format!("Failed to save config: {}", e)),
            }
        }

        Request::GetAutoToggleHotkeys => {
            let config = Config::load();
            Response::ConfigValues(ConfigValues {
                auto_toggle_hotkeys: config.auto_toggle_hotkeys,
                ..Default::default()
            })
        }

        Request::ToggleAutoMode => {
            let mut config = Config::load();
            config.transcription_mode = match config.transcription_mode {
                flowstt_common::TranscriptionMode::Automatic => flowstt_common::TranscriptionMode::PushToTalk,
                flowstt_common::TranscriptionMode::PushToTalk => flowstt_common::TranscriptionMode::Automatic,
            };
            match config.save() {
                Ok(()) => Response::Ok,
                Err(e) => Response::error(format!("Failed to save config: {}", e)),
            }
        }

        Request::SetAecEnabled { enabled } => {
            let mut config = Config::load();
            config.recording_mode = if enabled {
                vtx_common::RecordingMode::EchoCancel
            } else {
                vtx_common::RecordingMode::Mixed
            };
            match config.save() {
                Ok(()) => Response::Ok,
                Err(e) => Response::error(format!("Failed to save config: {}", e)),
            }
        }

        Request::SetRecordingMode { mode } => {
            let mut config = Config::load();
            config.recording_mode = mode;
            match config.save() {
                Ok(()) => Response::Ok,
                Err(e) => Response::error(format!("Failed to save config: {}", e)),
            }
        }

        Request::SetAutoPaste { enabled } => {
            let mut config = Config::load();
            config.auto_paste_enabled = enabled;
            match config.save() {
                Ok(()) => Response::Ok,
                Err(e) => Response::error(format!("Failed to save config: {}", e)),
            }
        }

        Request::SetRestoreClipboard { enabled } => {
            let mut config = Config::load();
            config.restore_clipboard_enabled = enabled;
            match config.save() {
                Ok(()) => Response::Ok,
                Err(e) => Response::error(format!("Failed to save config: {}", e)),
            }
        }

        Request::SetMicGain { gain } => {
            let mut config = Config::load();
            config.mic_gain = gain;
            match config.save() {
                Ok(()) => Response::Ok,
                Err(e) => Response::error(format!("Failed to save config: {}", e)),
            }
        }

        Request::GetHistory => {
            match history::load_history() {
                Ok(entries) => Response::History { entries },
                Err(e) => Response::error(e),
            }
        }

        Request::DeleteHistoryEntry { id } => {
            match history::delete_history_entry(&id) {
                Ok(()) => Response::Ok,
                Err(e) => Response::error(e),
            }
        }

        Request::TestAudioDevice { device_id } => {
            match engine.start_test_capture(device_id) {
                Ok(()) => Response::Ok,
                Err(e) => Response::error(e),
            }
        }

        Request::StopTestAudioDevice => {
            match engine.stop_test_capture() {
                Ok(()) => Response::Ok,
                Err(e) => Response::error(e),
            }
        }

        Request::CheckAccessibilityPermission => {
            Response::AccessibilityPermission {
                granted: super::hotkey::check_accessibility_permission(),
            }
        }

        Request::RequestAccessibilityPermission => {
            super::hotkey::request_accessibility_permission();
            Response::AccessibilityPermission {
                granted: super::hotkey::check_accessibility_permission(),
            }
        }

        Request::SubscribeEvents => {
            // Event streaming is not yet implemented in the new server.
            // CLI `transcribe` command uses this; return Subscribed for compatibility.
            Response::Subscribed
        }

        Request::Shutdown => {
            info!("[IPC] Shutdown requested via IPC");
            engine.shutdown();
            Response::Ok
        }

        Request::GetRuntimeMode => {
            Response::RuntimeMode {
                mode: runtime_mode().as_str().to_string(),
            }
        }
    }
}
