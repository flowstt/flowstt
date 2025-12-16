# Change: Add Echo Cancellation for Mixed Audio Sources

## Why
When capturing both microphone input and system audio simultaneously, the microphone picks up the system audio output through speakers or acoustic feedback. This causes the system audio to appear twice in the mix (once from the monitor capture, once from the microphone), degrading transcription quality and intelligibility. Acoustic echo cancellation (AEC) removes the system audio component from the microphone signal before mixing.

## What Changes
- Add `aec3` crate dependency - a pure Rust port of WebRTC's AEC3 algorithm (replaced `aec-rs` due to quality issues with Speex at 48kHz)
- Modify the audio mixer to apply AEC when both microphone and system audio sources are active
- Use VoipAec3 wrapper with native 48kHz support and interleaved f32 samples
- Configure AEC with 10ms frames (480 samples per channel at 48kHz)
- AEC processes microphone audio using system audio as the reference signal to cancel echo
- Replace the "Processing" toggle with an "Echo Cancel" toggle that enables/disables AEC
- Remove the separate `is_processing_enabled` state - audio processing (speech detection, visualization) is now always active when monitoring or recording
- Speech detection events are always emitted during monitoring/recording (no toggle required)

## Impact
- Affected specs: `audio-recording` (Mixed Audio Capture requirement), `audio-processing` (Echo Cancellation Toggle)
- Affected code:
  - `src-tauri/src/pipewire_audio.rs` (AudioMixer struct and mixing logic)
  - `src-tauri/src/audio.rs` (remove `is_processing_enabled` state)
  - `src-tauri/src/lib.rs` (replace `set_processing_enabled`/`is_processing_enabled` with AEC commands)
  - `src/main.ts` (replace Processing toggle logic with Echo Cancel toggle)
  - `index.html` (rename toggle label)
- New dependency: `aec3` crate (pure Rust WebRTC AEC3 port)
