# Change: Add Clipboard Copy and Automatic Paste

## Why
After transcription completes, users must manually select and copy the transcribed text, then switch to their target application and paste. This friction defeats the purpose of a hands-free voice transcription tool. Automatically placing text on the clipboard and pasting it into the active application makes the transcription-to-text flow seamless.

## What Changes
- After each transcription segment completes, the service copies the transcribed text to the system clipboard
- The service detects the foreground window and, if it is not a FlowSTT window, simulates a Ctrl+V (Cmd+V on macOS) keystroke to paste the text
- Clipboard copy always happens regardless of which window is focused; only the paste simulation is suppressed when FlowSTT is the active window
- Auto-paste is enabled by default; a new configuration option (`auto_paste_enabled`) controls the behavior
- A configurable delay (`auto_paste_delay_ms`) between clipboard write and paste simulation allows tuning for application compatibility (default: 50ms)
- Platform-specific implementations for clipboard access, foreground window detection, and input simulation (Windows via Win32 APIs, macOS via AppKit/CGEvent, Linux via X11/Wayland)

## Impact
- Affected specs: `clipboard-paste` (new capability)
- Affected code:
  - `src-service/src/clipboard/` (new module: clipboard write, foreground detection, paste simulation)
  - `src-service/src/audio_loop.rs` (hook into `TranscriptionEventBroadcaster::on_transcription_complete`)
  - `src-service/src/config.rs` (new config fields: `auto_paste_enabled`, `auto_paste_delay_ms`)
  - `src-service/src/state.rs` (expose new config in state if needed)
  - `src-common/src/types.rs` (add config fields to `ConfigValues`)
  - `src-common/src/ipc/requests.rs` (new IPC request for toggling auto-paste)
  - `src-service/Cargo.toml` (platform dependencies: `clipboard-win` or raw Win32, `cocoa`/`objc2` on macOS, `x11-clipboard`/`wl-clipboard` on Linux)
