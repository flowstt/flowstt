//! FlowSTT-specific types.
//!
//! Audio/transcription types that duplicate vtx-common are removed from here;
//! they are re-exported directly from vtx-common in lib.rs.
//! Only types unique to FlowSTT's IPC protocol and app layer are defined here.

use serde::{Deserialize, Serialize};
use vtx_engine::HotkeyCombination;

/// Transcription mode — an app-level concept that determines how FlowSTT
/// decides when to record audio for transcription.
///
/// vtx-engine itself has no notion of transcription modes. It provides:
/// - **VAD (automatic)**: continuously detects speech and transcribes segments
/// - **Manual recording**: `start_recording()` / `stop_recording()` API
///
/// FlowSTT maps this enum to the appropriate engine API calls.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptionMode {
    /// VAD-triggered — speech detection determines segment boundaries
    #[default]
    Automatic,
    /// Push-to-Talk — user holds a hotkey to record, audio is transcribed on release
    PushToTalk,
}

// Re-export from vtx-engine so internal modules can use crate-internal types
pub use vtx_engine::{
    AudioDevice, AudioSourceType, HistoryEntry, KeyCode, ModelStatus, RecordingMode,
    TranscriptionResult,
};

/// Runtime mode - determines behavior for service lifecycle management.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeMode {
    /// Development mode - service persists independently for debugging
    Development,
    /// Production mode - service lifecycle coupled to owner client
    #[default]
    Production,
}

impl RuntimeMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            RuntimeMode::Development => "development",
            RuntimeMode::Production => "production",
        }
    }
}

/// Persisted configuration values returned by the GetConfig IPC request.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfigValues {
    /// Current transcription mode (Automatic or PushToTalk) — app-level concept
    pub transcription_mode: TranscriptionMode,
    /// Configured push-to-talk hotkey combinations
    pub ptt_hotkeys: Vec<HotkeyCombination>,
    /// Configured auto-mode toggle hotkeys
    #[serde(default)]
    pub auto_toggle_hotkeys: Vec<HotkeyCombination>,
    /// Whether auto-paste into the foreground application is enabled
    #[serde(default = "default_auto_paste_enabled")]
    pub auto_paste_enabled: bool,
    /// Delay in milliseconds between clipboard write and paste simulation
    #[serde(default = "default_auto_paste_delay_ms")]
    pub auto_paste_delay_ms: u32,
    /// Whether to save and restore clipboard contents around each transcription paste
    #[serde(default = "default_restore_clipboard_enabled")]
    pub restore_clipboard_enabled: bool,
    /// Microphone input gain in dB (−20.0 to +20.0 dB, default 0.0)
    #[serde(default = "default_mic_gain")]
    pub mic_gain: f32,
    /// Preferred primary audio input device ID
    #[serde(default)]
    pub preferred_source1_id: Option<String>,
    /// Preferred reference (system) audio device ID
    #[serde(default)]
    pub preferred_source2_id: Option<String>,
}

fn default_auto_paste_enabled() -> bool {
    true
}
fn default_auto_paste_delay_ms() -> u32 {
    50
}
fn default_restore_clipboard_enabled() -> bool {
    true
}
fn default_mic_gain() -> f32 {
    0.0
}

/// Push-to-talk status information.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PttStatus {
    /// Current transcription mode
    pub mode: TranscriptionMode,
    /// Configured PTT hotkey combinations
    pub hotkeys: Vec<HotkeyCombination>,
    /// Configured auto-mode toggle hotkeys
    #[serde(default)]
    pub auto_toggle_hotkeys: Vec<HotkeyCombination>,
    /// Whether auto mode is currently active
    #[serde(default)]
    pub auto_mode_active: bool,
    /// Whether PTT key is currently pressed
    pub is_active: bool,
    /// Whether PTT is available on this platform
    pub available: bool,
    /// Error message if PTT is unavailable (e.g., missing permissions)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Whether macOS Accessibility permission is currently granted.
    /// Always true on non-macOS platforms (permission not applicable).
    #[serde(default = "default_true")]
    pub accessibility_permission_granted: bool,
}

fn default_true() -> bool {
    true
}

/// Status of the transcription system.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TranscribeStatus {
    /// Whether audio capture is running (sources configured and valid)
    pub capturing: bool,
    /// Whether currently capturing speech
    pub in_speech: bool,
    /// Number of segments waiting to be transcribed
    pub queue_depth: usize,
    /// Error message if capture failed (e.g., invalid source)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Currently configured primary audio source ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source1_id: Option<String>,
    /// Currently configured secondary audio source ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source2_id: Option<String>,
    /// Current transcription mode
    pub transcription_mode: TranscriptionMode,
}

/// CUDA/GPU acceleration status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CudaStatus {
    /// Whether the binary was built with CUDA support
    pub build_enabled: bool,
    /// Whether CUDA is available at runtime
    pub runtime_available: bool,
    /// System info string from whisper.cpp
    pub system_info: String,
}

/// A single column of spectrogram data ready for rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectrogramColumn {
    /// RGB triplets for each pixel row (height * 3 bytes)
    pub colors: Vec<u8>,
}

/// Visualization data for real-time audio display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationData {
    /// Waveform amplitude values (downsampled for display)
    pub waveform: Vec<f32>,
    /// Spectrogram column (RGB color values, if ready)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spectrogram: Option<SpectrogramColumn>,
    /// Speech detection metrics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speech_metrics: Option<SpeechMetrics>,
}

/// Speech detection metrics for visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechMetrics {
    /// RMS amplitude in dB
    pub amplitude_db: f32,
    /// Zero-crossing rate (0.0-1.0)
    pub zcr: f32,
    /// Spectral centroid in Hz
    pub centroid_hz: f32,
    /// Whether speech is currently detected
    pub is_speaking: bool,
    /// Whether voiced onset is pending
    pub voiced_onset_pending: bool,
    /// Whether whisper onset is pending
    pub whisper_onset_pending: bool,
    /// Whether a transient was detected
    pub is_transient: bool,
    /// Whether this is lookback-determined speech
    pub is_lookback_speech: bool,
    /// Whether this is a word break
    pub is_word_break: bool,
}
