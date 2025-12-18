# Change: Add Automatic Transcription Mode

## Why
The current manual recording workflow requires users to click Record/Stop for each speech segment. For continuous use cases like meeting transcription or voice notes, automatic speech-triggered recording and transcription would eliminate this manual interaction and create a hands-free experience.

## What Changes
- Replace the "Record" button with a "Transcribe" toggle
- When Transcribe is active, automatically begin recording when speech is detected
- Automatically stop recording and submit for transcription when speech ends
- Implement a transcription queue to handle multiple speech segments in parallel with recording
- Allow transcription to run asynchronously and lag behind live speech
- Continue saving WAV files to the configured directory (cleanup handled separately)

## Impact
- Affected specs: audio-recording, speech-transcription, audio-visualization
- Affected code:
  - `src/main.ts` - Replace Record button with Transcribe toggle, handle transcribe mode state
  - `index.html` - Update UI controls
  - `src-tauri/src/lib.rs` - Add transcribe mode commands, transcription queue management
  - `src-tauri/src/audio.rs` - Add speech-triggered recording state management
  - `src-tauri/src/processor.rs` - Integrate speech events with recording triggers
