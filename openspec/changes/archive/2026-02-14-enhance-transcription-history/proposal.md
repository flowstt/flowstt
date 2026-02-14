# Change: Enhance Transcription History and Main Window Display

## Why
The main window currently shows transcription as a single continuous text buffer with no persistence, no segment boundaries, and no ability to review, copy, or manage individual transcription segments. WAV files are saved to a user-visible Documents directory with no automatic cleanup. The window is also fixed-size and cannot be resized, and the mini-visualizer is always visible even when idle.

## What Changes
- **BREAKING** Main window becomes resizable; content and components scale with window dimensions while maintaining relative margins
- Mini-visualizer is hidden unless audio recording is active (PTT key held or auto-transcription capturing speech)
- Transcription display changes from a single continuous text buffer to a scrolling history of individual segments
- Each segment shows: timestamp, transcribed text, copy button, delete button, and a play button (if WAV file exists)
- Remove the gradient fade/shadow effect at the top of the transcription panel
- Persistent transcription history stored in a JSON state file in the OS-standard application data directory
- History entries include transcribed text, timestamp, and path to the cached WAV file
- WAV files relocated from `~/Documents/Recordings/` to the OS-standard application data/state directory
- Automatic cleanup of WAV files older than 24 hours
- Delete button removes the segment from history and deletes the associated WAV file

## Impact
- Affected specs: `window-appearance`, `audio-visualization`, `audio-recording`, `transcription-history` (new)
- Affected code:
  - `src-tauri/tauri.conf.json` (window config)
  - `src-tauri/src/tray/windows.rs` (window recreation)
  - `src/main.ts` (transcription display logic, mini-visualizer visibility)
  - `src/styles.css` (layout, segment styling, fade removal)
  - `index.html` (transcription area structure)
  - `src-service/src/transcription/transcribe_state.rs` (WAV save path)
  - `src-service/src/config.rs` or new history module (history persistence, cleanup)
  - `src-common/src/ipc/requests.rs` and `responses.rs` (new IPC messages for history)
  - `src-tauri/src/lib.rs` (new Tauri commands for history and audio playback)
