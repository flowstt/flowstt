# Change: Add Voice Processing in Monitor Mode

## Why
Users need the ability to analyze incoming audio data while monitoring. This establishes a foundation for multiple processing types (silence detection, voice activity detection, etc.) that will be added over time.

## What Changes
- Add a new "Voice Processing" toggle in the UI (separate from Monitor toggle)
- Create an extensible audio processing system in the backend
- Implement silence detection as the first processor example
- Log processing results to console

## Impact
- Affected specs: New `audio-processing` capability
- Affected code:
  - `src-tauri/src/audio.rs` - Add processing hook in audio callback
  - `src-tauri/src/lib.rs` - Add command to toggle processing
  - `src/main.ts` - Add UI toggle and state management
  - `index.html` - Add toggle element
