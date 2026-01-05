# Change: Refactor Core Voice Transcription into Separate Service with CLI

## Why
The current FlowSTT implementation tightly couples voice transcription capabilities with the Tauri GUI application, making it impossible to use transcription functionality from scripts, automation pipelines, or headless environments. Separating the core transcription engine into a standalone service (following the omnirec pattern) enables CLI access while preserving the GUI experience.

## What Changes
- **NEW** `src-service/` crate (binary: `flowstt-service`) containing the core transcription engine:
  - Audio capture backends (extracted from current `src-tauri/src/platform/`)
  - Audio processing pipeline (resampling, speech detection, echo cancellation)
  - Whisper transcription (queue, worker, FFI bindings)
  - IPC server for CLI and GUI communication
- **NEW** `src-common/` crate (`flowstt-common`) for shared types and IPC protocol:
  - Request/Response message types
  - Audio device representations
  - Transcription configuration
  - Security and validation utilities
- **NEW** `src-cli/` crate (binary: `flowstt`) providing command-line interface:
  - `flowstt list devices` - enumerate audio sources
  - `flowstt transcribe` - start transcription with configurable sources
  - `flowstt status` - show current transcription status
  - `flowstt stop` - stop active transcription
  - JSON output mode for scripting
- **MODIFIED** `src-tauri/` (binary: `flowstt-app`) to become a thin GUI wrapper:
  - Connects to the service via IPC (like omnirec-app)
  - Retains visualization and UI logic only
  - Service (`flowstt-service`) auto-spawned on GUI launch if not running

## Impact
- Affected specs:
  - `audio-recording` - audio capture moves to service
  - `audio-processing` - processing pipeline moves to service
  - `speech-transcription` - transcription engine moves to service
  - (NEW) `core-transcription-service` - new capability
  - (NEW) `cli-interface` - new capability
- Affected code:
  - `src-tauri/src/platform/` - moves to `src-service/`
  - `src-tauri/src/audio.rs` - moves to `src-service/`
  - `src-tauri/src/processor.rs` - moves to `src-service/`
  - `src-tauri/src/transcribe*.rs` - moves to `src-service/`
  - `src-tauri/src/whisper_ffi.rs` - moves to `src-service/`
  - `src-tauri/src/lib.rs` - becomes IPC client
- **NOT affected** (remain in GUI):
  - `src-tauri/src/main.rs` - Tauri entry point
  - All frontend TypeScript/CSS/HTML
  - Visualization rendering (waveform, spectrogram, speech activity)
  - Window appearance and styling
