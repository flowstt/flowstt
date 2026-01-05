//! IPC response types.

use serde::{Deserialize, Serialize};

use crate::types::{
    AudioDevice, CudaStatus, ModelStatus, TranscribeStatus, TranscriptionResult,
    VisualizationData,
};

/// IPC response from service to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    // === Success Responses ===
    /// List of audio devices
    Devices { devices: Vec<AudioDevice> },

    /// Current transcription status
    Status(TranscribeStatus),

    /// Whisper model status
    ModelStatus(ModelStatus),

    /// CUDA/GPU status
    CudaStatus(CudaStatus),

    /// Subscribed to events
    Subscribed,

    /// Generic success
    Ok,

    /// Pong response to ping
    Pong,

    // === Error Response ===
    /// Error occurred
    Error { message: String },

    // === Event Responses (after Subscribe) ===
    /// Real-time event
    Event { event: EventType },
}

/// Event types streamed to subscribed clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum EventType {
    /// Visualization data update
    VisualizationData(VisualizationData),

    /// Transcription result for a segment
    TranscriptionComplete(TranscriptionResult),

    /// Speech started
    SpeechStarted,

    /// Speech ended
    SpeechEnded { duration_ms: u64 },

    /// Transcription mode started
    TranscribeStarted,

    /// Transcription mode stopped
    TranscribeStopped,

    /// Model download progress
    ModelDownloadProgress { percent: u8 },

    /// Model download complete
    ModelDownloadComplete { success: bool },

    /// Service is shutting down
    Shutdown,
}

impl Response {
    /// Create an error response.
    pub fn error(message: impl Into<String>) -> Self {
        Response::Error {
            message: message.into(),
        }
    }

    /// Create a success response.
    pub fn ok() -> Self {
        Response::Ok
    }

    /// Check if this response indicates an error.
    pub fn is_error(&self) -> bool {
        matches!(self, Response::Error { .. })
    }
}
