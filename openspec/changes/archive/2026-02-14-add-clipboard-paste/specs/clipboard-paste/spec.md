## ADDED Requirements

### Requirement: Automatic Clipboard Copy
The system SHALL copy the transcribed text to the system clipboard immediately after each transcription segment completes. The clipboard write SHALL occur regardless of which window is currently focused.

#### Scenario: Text copied to clipboard after transcription
- **WHEN** a transcription segment completes with non-empty text
- **THEN** the transcribed text is written to the system clipboard as plain text

#### Scenario: Clipboard written when FlowSTT is focused
- **WHEN** a transcription segment completes and a FlowSTT window is the foreground window
- **THEN** the transcribed text is still written to the system clipboard

#### Scenario: Empty or no-speech transcription does not write clipboard
- **WHEN** a transcription segment completes with empty text or "(No speech detected)"
- **THEN** the clipboard is not modified

#### Scenario: Clipboard write failure is non-fatal
- **WHEN** clipboard write fails (e.g., clipboard locked by another application)
- **THEN** the failure is logged as a warning and transcription continues normally

### Requirement: Automatic Paste Simulation
The system SHALL simulate a paste keystroke (Ctrl+V on Windows/Linux, Cmd+V on macOS) into the active foreground application after writing transcribed text to the clipboard. Paste simulation SHALL NOT occur when a FlowSTT window is the foreground application.

#### Scenario: Auto-paste into external application
- **WHEN** a transcription segment completes, the text is copied to the clipboard, and the foreground application is not FlowSTT
- **THEN** the system simulates a paste keystroke into the foreground application after the configured delay

#### Scenario: Paste suppressed when FlowSTT is active
- **WHEN** a transcription segment completes and a FlowSTT window (GUI) is the foreground application
- **THEN** no paste keystroke is simulated

#### Scenario: Paste suppressed when auto-paste is disabled
- **WHEN** a transcription segment completes and `auto_paste_enabled` is set to `false` in configuration
- **THEN** the text is copied to the clipboard but no paste keystroke is simulated

#### Scenario: Configurable delay before paste
- **WHEN** the system is about to simulate a paste keystroke
- **THEN** it waits for `auto_paste_delay_ms` milliseconds (default: 50) after the clipboard write before sending the keystroke

#### Scenario: Paste simulation failure is non-fatal
- **WHEN** paste keystroke simulation fails (e.g., permission denied, display server unavailable)
- **THEN** the failure is logged as a warning and the clipboard still contains the transcribed text

### Requirement: Foreground Window Detection
The system SHALL detect the current foreground application to determine whether paste simulation should be suppressed. Detection SHALL identify FlowSTT windows by matching the foreground window's owning executable name against the FlowSTT GUI binary name (`flowstt-app`).

#### Scenario: FlowSTT GUI detected as foreground on Windows
- **WHEN** the foreground window belongs to a process with executable name `flowstt-app.exe`
- **THEN** the system identifies the foreground as a FlowSTT window

#### Scenario: FlowSTT GUI detected as foreground on macOS
- **WHEN** the frontmost application has a bundle or executable name matching `flowstt-app`
- **THEN** the system identifies the foreground as a FlowSTT window

#### Scenario: FlowSTT GUI detected as foreground on Linux
- **WHEN** the focused window belongs to a process with executable name `flowstt-app`
- **THEN** the system identifies the foreground as a FlowSTT window

#### Scenario: No foreground window detected
- **WHEN** the foreground window cannot be determined (e.g., desktop is focused, error querying)
- **THEN** the system proceeds with paste simulation (default to pasting)

### Requirement: Auto-Paste Configuration
The system SHALL provide configuration options to control automatic paste behavior. Configuration is persisted in the service config file.

#### Scenario: Auto-paste enabled by default
- **WHEN** the service starts with no prior configuration for auto-paste
- **THEN** `auto_paste_enabled` defaults to `true`

#### Scenario: Paste delay configurable
- **WHEN** the user sets `auto_paste_delay_ms` in the configuration
- **THEN** the system uses the specified delay between clipboard write and paste simulation

#### Scenario: Default paste delay
- **WHEN** the service starts with no prior configuration for paste delay
- **THEN** `auto_paste_delay_ms` defaults to `50`

#### Scenario: Toggle auto-paste via IPC
- **WHEN** a client sends a `SetAutoPaste` request with an `enabled` flag
- **THEN** the service updates the `auto_paste_enabled` setting and persists it to the config file

### Requirement: Platform Clipboard and Input Backends
The system SHALL implement clipboard write, foreground detection, and paste simulation using platform-native APIs on each supported operating system.

#### Scenario: Windows clipboard and paste
- **WHEN** the service runs on Windows
- **THEN** clipboard is accessed via Win32 `OpenClipboard`/`SetClipboardData` APIs, foreground detection uses `GetForegroundWindow`/`GetWindowThreadProcessId`, and paste is simulated via `SendInput` with `INPUT_KEYBOARD`

#### Scenario: macOS clipboard and paste
- **WHEN** the service runs on macOS
- **THEN** clipboard is accessed via `NSPasteboard`, foreground detection uses `NSWorkspace.shared.frontmostApplication`, and paste is simulated via `CGEvent` keyboard events

#### Scenario: Linux clipboard and paste
- **WHEN** the service runs on Linux
- **THEN** clipboard and paste are handled via available system tools (e.g., `xclip`/`xdotool` for X11, `wl-copy`/`wtype` for Wayland) with graceful fallback if tools are not installed
