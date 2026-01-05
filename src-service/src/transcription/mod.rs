//! Voice transcription module for FlowSTT.
//!
//! This module provides automatic transcription of audio using whisper.cpp via FFI.
//!
//! # Components
//!
//! - [`whisper_ffi`]: Low-level FFI bindings to whisper.cpp
//! - [`transcriber`]: High-level transcription API
//! - [`queue`]: Async transcription queue with worker thread
//! - [`transcribe_state`]: State management for continuous transcription mode

pub mod queue;
pub mod transcribe_state;
pub mod transcriber;
pub mod whisper_ffi;

// Re-export main types
pub use queue::{QueuedSegment, TranscriptionCallback, TranscriptionQueue};
pub use transcribe_state::{SegmentRingBuffer, TranscribeState, TranscribeStateCallback};
pub use transcriber::{download_model, Transcriber};
pub use whisper_ffi::{
    full_default_params, get_system_info, init_library, Context, WhisperFullParams,
    WhisperSamplingStrategy, WhisperVadParams,
};
