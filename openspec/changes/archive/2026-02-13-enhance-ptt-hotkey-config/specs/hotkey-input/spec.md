## MODIFIED Requirements

### Requirement: Global Hotkey Backend Interface
The system SHALL provide a platform-agnostic interface for global hotkey capture through a `HotkeyBackend` trait. Platform-specific implementations SHALL implement this trait, enabling push-to-talk functionality across supported platforms. The backend SHALL support monitoring multiple hotkey combinations simultaneously.

#### Scenario: Backend trait defines hotkey operations
- **WHEN** the application requires global hotkey functionality
- **THEN** it uses the `HotkeyBackend` trait methods: `start()`, `stop()`, `try_recv()`, `is_running()`, and `is_available()`

#### Scenario: Backend start accepts multiple hotkey combinations
- **WHEN** the backend `start()` method is called
- **THEN** it accepts a `Vec<HotkeyCombination>` representing all configured PTT hotkey bindings

#### Scenario: Backend selected at compile time
- **WHEN** the application is compiled for a specific platform
- **THEN** the appropriate platform backend is selected via conditional compilation

#### Scenario: Backend delivers press and release events
- **WHEN** any configured hotkey combination is fully held or released
- **THEN** the backend delivers a `HotkeyEvent::Pressed` or `HotkeyEvent::Released` event

#### Scenario: Backend tracks all key state
- **WHEN** the backend is active
- **THEN** it maintains a set of currently-pressed keys and matches against all configured combinations

### Requirement: macOS Hotkey Backend (CGEventTap)
The system SHALL provide a fully functional hotkey backend for macOS using the CGEventTap API, supporting global key monitoring for hotkey combinations even when the application is not focused.

#### Scenario: macOS backend initializes CGEventTap
- **WHEN** the hotkey backend starts on macOS
- **THEN** a CGEventTap is created in passive listening mode (kCGEventTapOptionListenOnly)

#### Scenario: macOS backend detects key events
- **WHEN** any key is pressed or released anywhere in the system
- **THEN** the backend receives the key event and updates its pressed-key state

#### Scenario: macOS backend matches combinations
- **WHEN** the set of currently-pressed keys is updated
- **THEN** the backend checks if any configured hotkey combination is a subset of the pressed keys
- **AND** emits `Pressed` when a combination becomes fully held
- **AND** emits `Released` when a previously-matched combination is no longer fully held

#### Scenario: macOS backend runs on separate thread
- **WHEN** the hotkey backend is active
- **THEN** event monitoring runs on a dedicated thread to avoid blocking audio processing

#### Scenario: macOS backend stops cleanly
- **WHEN** the hotkey backend stop() is called
- **THEN** the CGEventTap is disabled and the run loop exits

### Requirement: Hotkey Configuration
The system SHALL allow configuration of one or more push-to-talk hotkey bindings, each of which is a combination of one or more simultaneously-held keys. A sensible default SHALL be provided for each platform.

#### Scenario: Default hotkey on macOS
- **WHEN** no custom hotkeys are configured on macOS
- **THEN** a single binding of the Right Option (Alt) key is used as the default PTT hotkey

#### Scenario: Default hotkey on Windows
- **WHEN** no custom hotkeys are configured on Windows
- **THEN** a single binding of the Right Alt key is used as the default PTT hotkey

#### Scenario: Multiple hotkey bindings
- **WHEN** the user configures multiple PTT hotkey bindings
- **THEN** pressing any one of the configured combinations activates push-to-talk

#### Scenario: Key combination binding
- **WHEN** the user configures a hotkey combination (e.g., Ctrl+Alt)
- **THEN** all keys in the combination must be held simultaneously to activate PTT
- **AND** releasing any key in the combination deactivates PTT

#### Scenario: Configuration persists across sessions
- **WHEN** the user configures custom hotkey bindings
- **THEN** all bindings are saved and restored on next application launch

#### Scenario: Config migration from single key format
- **WHEN** the application loads a config file with the legacy `ptt_key` field
- **THEN** it is automatically migrated to a single-element `ptt_hotkeys` array
- **AND** the migrated config is saved in the new format on next save

#### Scenario: Empty hotkey list
- **WHEN** the user removes all hotkey bindings
- **THEN** no PTT hotkey is active and push-to-talk cannot be triggered via keyboard

### Requirement: Hotkey IPC Requests
The system SHALL support IPC requests for configuring and controlling the push-to-talk hotkey system, using hotkey combinations and multiple bindings.

#### Scenario: Set transcription mode request
- **WHEN** a client sends a SetTranscriptionMode request
- **THEN** the service updates the active transcription mode and responds with success

#### Scenario: Set PTT hotkeys request
- **WHEN** a client sends a SetPushToTalkHotkeys request with a list of hotkey combinations
- **THEN** the service updates all hotkey bindings, restarts monitoring with the new combinations, and responds with success

#### Scenario: Get PTT status request
- **WHEN** a client sends a GetPttStatus request
- **THEN** the service responds with the current transcription mode and all configured hotkey combinations

#### Scenario: Get transcription mode request
- **WHEN** a client sends a GetTranscriptionMode request
- **THEN** the service responds with the current transcription mode and hotkey configuration

### Requirement: Hotkey Event Key Codes
The system SHALL use platform-independent key code representation for configuring push-to-talk hotkeys. The key code set SHALL include all standard keyboard keys.

#### Scenario: Key code enumeration
- **WHEN** a key is used in a PTT hotkey combination
- **THEN** it is represented using a `KeyCode` enum that maps to platform-specific codes

#### Scenario: Extended key support
- **WHEN** the user records a hotkey
- **THEN** all standard keyboard keys are supported including: modifier keys (Alt, Control, Shift, Meta/Win/Cmd), function keys (F1-F24), letter keys (A-Z), number keys (0-9), punctuation keys, navigation keys (Home, End, PageUp, PageDown, arrows), and special keys (CapsLock, Tab, Escape, Space, Enter, Backspace, Delete, Insert, PrintScreen, ScrollLock, Pause)

#### Scenario: Key code serialization
- **WHEN** key configuration is saved or transmitted via IPC
- **THEN** key codes serialize to human-readable snake_case names (e.g., "right_alt", "f13", "key_a", "digit_1")

#### Scenario: Hotkey combination serialization
- **WHEN** a hotkey combination is serialized
- **THEN** it is represented as an object with a `keys` array of key code strings

## REMOVED Requirements

### Requirement: Windows Hotkey Backend (Stub)
**Reason**: The Windows backend is no longer a stub. It has been fully implemented using the Raw Input API. This stub requirement is replaced by the new "Windows Hotkey Backend (Raw Input)" requirement under ADDED.
**Migration**: See the ADDED "Windows Hotkey Backend (Raw Input)" requirement below.

## ADDED Requirements

### Requirement: Hotkey Combination Type
The system SHALL provide a `HotkeyCombination` type representing a set of keys that must all be held simultaneously to trigger PTT.

#### Scenario: Single key combination
- **WHEN** a `HotkeyCombination` contains one key
- **THEN** it behaves identically to the legacy single-key PTT model

#### Scenario: Multi-key combination
- **WHEN** a `HotkeyCombination` contains multiple keys
- **THEN** all keys must be held simultaneously for the combination to be considered active

#### Scenario: Combination equality ignores key order
- **WHEN** two `HotkeyCombination` values contain the same keys in different order
- **THEN** they are considered equal

#### Scenario: Display format
- **WHEN** a `HotkeyCombination` is displayed to the user
- **THEN** keys are shown joined by " + " separators (e.g., "Ctrl + Alt + K")
- **AND** modifier keys are listed before non-modifier keys

### Requirement: Windows Hotkey Backend (Raw Input)
The system SHALL provide a hotkey backend for Windows using the Raw Input API, supporting global key monitoring for hotkey combinations even when the application is not focused.

#### Scenario: Windows backend initializes Raw Input
- **WHEN** the hotkey backend starts on Windows
- **THEN** a hidden message-only window is created with Raw Input registered for keyboard devices using `RIDEV_INPUTSINK` for global capture

#### Scenario: Windows backend detects key events
- **WHEN** any key is pressed or released anywhere in the system
- **THEN** the backend receives the key event and updates its pressed-key state

#### Scenario: Windows backend matches combinations
- **WHEN** the set of currently-pressed keys is updated
- **THEN** the backend checks if any configured hotkey combination is a subset of the pressed keys
- **AND** emits `Pressed` when a combination becomes fully held
- **AND** emits `Released` when a previously-matched combination is no longer fully held

#### Scenario: Windows backend distinguishes left/right modifiers
- **WHEN** modifier key events are received
- **THEN** the backend distinguishes between left and right variants using VK codes and the E0 extended key flag

#### Scenario: Windows backend runs on separate thread
- **WHEN** the hotkey backend is active
- **THEN** the message pump runs on a dedicated thread to avoid blocking audio processing

#### Scenario: Windows backend stops cleanly
- **WHEN** the hotkey backend stop() is called
- **THEN** the Raw Input registration is removed and the message loop exits
