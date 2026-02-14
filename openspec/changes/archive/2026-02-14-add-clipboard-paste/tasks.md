## 1. Configuration
- [x] 1.1 Add `auto_paste_enabled: bool` (default `true`) and `auto_paste_delay_ms: u32` (default `50`) to config in `src-common/src/config.rs`
- [x] 1.2 Add corresponding fields to `ConfigValues` in `src-common/src/types.rs`
- [x] 1.3 Add `SetAutoPaste { enabled: bool }` request variant to `src-common/src/ipc/requests.rs`
- [x] 1.4 Add handler for `SetAutoPaste` in `src-service/src/ipc/handlers.rs`

## 2. Clipboard and Paste Trait
- [x] 2.1 Create `src-service/src/clipboard/mod.rs` with `ClipboardPaster` trait defining `write_clipboard(&self, text: &str) -> Result<()>`, `is_flowstt_foreground(&self) -> bool`, and `simulate_paste(&self) -> Result<()>`
- [x] 2.2 Create `src-service/src/clipboard/mod.rs` with platform dispatch function returning the appropriate backend

## 3. Windows Backend
- [x] 3.1 Add `Win32_System_DataExchange` and `Win32_System_Memory` features to the `windows` crate dependency in `src-service/Cargo.toml`
- [x] 3.2 Create `src-service/src/clipboard/windows.rs` implementing clipboard write via `OpenClipboard`/`SetClipboardData`/`CloseClipboard`
- [x] 3.3 Implement foreground detection via `GetForegroundWindow` + `GetWindowThreadProcessId` + process executable name check
- [x] 3.4 Implement paste simulation via `SendInput` with Ctrl key down, V key down, V key up, Ctrl key up

## 4. macOS Backend
- [x] 4.1 Create `src-service/src/clipboard/macos.rs` implementing clipboard write via `pbcopy`
- [x] 4.2 Implement foreground detection via `osascript` querying System Events frontmost process
- [x] 4.3 Implement paste simulation via `osascript` sending Cmd+V keystroke

## 5. Linux Backend
- [x] 5.1 Create `src-service/src/clipboard/linux.rs` implementing clipboard write via `xclip`/`wl-copy` subprocess
- [x] 5.2 Implement foreground detection via `xdotool getactivewindow getwindowpid` (X11) or best-effort Wayland approach
- [x] 5.3 Implement paste simulation via `xdotool key ctrl+v` (X11) or `wtype` (Wayland)
- [x] 5.4 Handle missing tools gracefully with warnings

## 6. Integration Hook
- [x] 6.1 In `TranscriptionEventBroadcaster::on_transcription_complete` (`src-service/src/audio_loop.rs`), after broadcasting the IPC event, call the clipboard/paste logic
- [x] 6.2 Read `auto_paste_enabled` and `auto_paste_delay_ms` from config to control behavior
- [x] 6.3 Skip clipboard write for empty text or "(No speech detected)" results
- [x] 6.4 Write text to clipboard, check foreground window, wait configured delay, simulate paste if appropriate
- [x] 6.5 Log warnings on clipboard/paste failures without interrupting transcription flow

## 7. Testing and Verification
- [ ] 7.1 Manual test: PTT hold -> speak -> release -> verify text appears in clipboard
- [ ] 7.2 Manual test: With external text editor focused, verify transcribed text is auto-pasted
- [ ] 7.3 Manual test: With FlowSTT GUI focused, verify no paste simulation occurs but clipboard is written
- [ ] 7.4 Manual test: Set `auto_paste_enabled = false`, verify clipboard write but no paste
- [ ] 7.5 Manual test: Adjust `auto_paste_delay_ms` and verify behavior
- [x] 7.6 Verify `cargo build` succeeds on Windows (primary platform)
- [x] 7.7 Verify `cargo clippy` passes without new warnings
