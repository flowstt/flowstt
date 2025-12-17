# Change: Add Echo Cancellation Recording Mode

## Why
Users need to record clean voice-only audio while listening to system audio (music, videos, etc.). The current "Mixed" mode with echo cancellation still mixes both streams together, which includes the system audio in the output. A new mode is needed that uses echo cancellation to remove system audio from the microphone input and outputs only the cleaned primary source.

## What Changes
- Add a new "Recording Mode" toggle to select between "Mixed" (current behavior) and "Echo Cancel" mode
- In "Echo Cancel" mode:
  - Use the secondary source (system audio) as the AEC reference signal
  - Apply echo cancellation to the primary source (microphone)
  - Output only the echo-cancelled primary source (no mixing)
- The existing AEC toggle continues to control whether echo cancellation is applied in either mode
- When AEC toggle is disabled in "Echo Cancel" mode, output the raw primary source only

## Impact
- Affected specs: audio-recording, audio-processing
- Affected code: `pipewire_audio.rs` (AudioMixer), `lib.rs` (Tauri commands), `main.ts` (UI)
