## Context

FlowSTT is a service-architecture desktop app where the background service (`flowstt-service`) handles all transcription. When a segment completes, the service broadcasts a `TranscriptionComplete` event via IPC. Currently there is no clipboard integration or input simulation -- users must manually copy and paste transcribed text.

The clipboard write, foreground window detection, and paste simulation all require platform-specific OS APIs. The service already depends on platform crates (`windows` on Windows, `objc2-app-kit` on macOS) and follows a backend-trait pattern for platform abstractions (see `src-service/src/platform/backend.rs` and `src-service/src/hotkey/backend.rs`).

## Goals / Non-Goals

- Goals:
  - Copy transcribed text to the system clipboard immediately after each transcription segment completes
  - Simulate a paste keystroke (Ctrl+V / Cmd+V) into the active foreground application
  - Suppress paste simulation when a FlowSTT window is the foreground window
  - Provide configuration for enabling/disabling auto-paste and tuning paste delay
  - Support Windows, macOS, and Linux

- Non-Goals:
  - Rich text or HTML clipboard content (plain text only)
  - Pasting via application-specific APIs (e.g., accessibility APIs, direct text insertion)
  - Undo integration in the target application
  - Clipboard history or clipboard manager integration
  - GUI settings panel for these options (CLI/config file only for now)

## Decisions

### 1. Implementation lives in the service, not the GUI

- **Decision**: Clipboard write and paste simulation happen in `flowstt-service`, not in the Tauri GUI app.
- **Rationale**: The service runs headlessly and is the single point where transcription results originate. Placing the logic here ensures clipboard/paste works identically whether the user is running the GUI, CLI, or service-only. The `TranscriptionEventBroadcaster::on_transcription_complete` callback in `src-service/src/audio_loop.rs:224` is the natural hook point.

### 2. Platform abstraction via trait + module pattern

- **Decision**: Create a `ClipboardPaste` trait in `src-service/src/clipboard/backend.rs` with platform implementations in `windows.rs`, `macos.rs`, and `linux.rs`, following the existing pattern in `src-service/src/hotkey/`.
- **Rationale**: Consistent with existing codebase conventions. Each platform has very different APIs for clipboard, foreground detection, and input simulation.

### 3. Windows implementation using existing `windows` crate

- **Decision**: Use Win32 APIs already available through the `windows` crate dependency:
  - Clipboard: `OpenClipboard`, `EmptyClipboard`, `SetClipboardData`, `CloseClipboard` (requires `Win32_System_DataExchange`)
  - Foreground detection: `GetForegroundWindow`, `GetWindowThreadProcessId` (already available via `Win32_UI_WindowsAndMessaging`)
  - FlowSTT detection: compare foreground window's process ID to the service's PID, or check the window class/title for the Tauri app. Since the service and GUI are separate processes, detect the GUI by checking if the foreground window's executable name matches `flowstt-app.exe`.
  - Paste simulation: `SendInput` with `INPUT_KEYBOARD` for Ctrl+V (already available via `Win32_UI_Input_KeyboardAndMouse`)
- **Rationale**: No new crate dependencies needed on Windows. The `windows` crate just needs the `Win32_System_DataExchange` feature added.

### 4. macOS implementation

- **Decision**: Use AppKit (`NSPasteboard`) for clipboard, `NSWorkspace.shared.frontmostApplication` for foreground detection (via existing `objc2-app-kit`), and `CGEvent` for paste simulation.
- **Rationale**: `objc2-app-kit` is already a dependency. CGEvent is the standard way to simulate keystrokes on macOS. May need `core-graphics` crate or direct CGEvent FFI.

### 5. Linux implementation

- **Decision**: Start with a stub/best-effort approach using `xdotool` and `xclip`/`wl-copy` CLI tools via `std::process::Command`, or use `arboard` crate for clipboard.
- **Rationale**: Linux has fragmented display server (X11 vs Wayland) and clipboard models. A full native implementation is complex. CLI tool delegation is pragmatic for the initial implementation. Can be upgraded to native APIs later.

### 6. Foreground detection identifies FlowSTT by executable name

- **Decision**: Check if the foreground window belongs to an executable named `flowstt-app` (the Tauri GUI binary). The service process itself has no windows, so it won't be detected as foreground.
- **Rationale**: Simple, reliable, no coordination needed between service and GUI. Works even if the GUI is restarted.

### 7. Configurable paste delay with sensible default

- **Decision**: `auto_paste_delay_ms: u32` config field, default 50ms. Delay is applied between clipboard write and Ctrl+V simulation.
- **Rationale**: Some applications need time to register clipboard changes. 50ms is a safe default that won't be perceptible. Power users can tune it up or down.

### 8. Clipboard write is synchronous in the transcription callback

- **Decision**: The clipboard write + paste simulation runs synchronously within `on_transcription_complete`. Since this callback runs on the dedicated transcription worker thread (not the audio thread), blocking briefly for clipboard and input simulation is acceptable.
- **Rationale**: The transcription worker thread processes segments sequentially and sleeps between segments. Adding ~50-100ms for clipboard + paste doesn't affect audio capture or speech detection (those run on separate threads).

## Risks / Trade-offs

- **Paste into wrong application**: If the user switches windows between transcription start and completion, the paste may go to an unintended window. This is inherent to the approach and acceptable for a v1.
  - Mitigation: The delay is short (transcription typically completes within seconds of speech ending). Users will learn the timing.

- **Application-specific paste issues**: Some applications may not respond to Ctrl+V (e.g., terminals expecting Ctrl+Shift+V, or applications with custom paste handling).
  - Mitigation: Auto-paste can be disabled via config. Document known incompatibilities.

- **Security considerations**: Simulating keystrokes requires elevated privileges on some systems (macOS Accessibility permissions for CGEvent).
  - Mitigation: FlowSTT already requires Accessibility permissions on macOS for global hotkey capture. Document this clearly.

- **Linux fragmentation**: X11 vs Wayland differences for clipboard and input simulation.
  - Mitigation: Start with best-effort CLI-tool approach; upgrade to native APIs when needed.

## Open Questions

- None at this time. Scope is well-defined by user decisions.
