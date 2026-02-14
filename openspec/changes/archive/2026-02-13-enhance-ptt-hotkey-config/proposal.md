# Change: Enhanced PTT Hotkey Configuration

## Why
The current PTT hotkey system only supports a single key selected from a fixed dropdown list. Users need the ability to record custom hotkeys (including key combinations like Ctrl+Alt), configure multiple alternative PTT hotkeys, and use any key on the keyboard -- not just modifiers and function keys. The config window dropdown is insufficient for this level of flexibility.

## What Changes
- **BREAKING**: Replace single `KeyCode` enum with a `HotkeyCombination` type that represents one or more simultaneously-held keys
- **BREAKING**: Replace single `ptt_key` config field with a `ptt_hotkeys` array supporting multiple hotkey bindings
- Replace the PTT key dropdown in the config window with a hotkey recorder widget that captures actual key presses
- Add UI for managing multiple PTT hotkeys (add/remove bindings)
- Expand the set of supported keys beyond modifiers and function keys to include all keyboard keys
- Enlarge the config window to accommodate the new hotkey management interface
- Update IPC messages, config persistence, and backend interfaces for the new data model
- Update platform backends (macOS CGEventTap, Windows Raw Input) to monitor for key combinations and multiple bindings

## Impact
- Affected specs: `hotkey-input`, `config-window`
- Affected code:
  - `src-common/src/types.rs` (KeyCode enum, PttStatus struct)
  - `src-common/src/ipc/requests.rs` (SetPushToTalkKey request)
  - `src-common/src/ipc/responses.rs` (PttStatus response)
  - `src-service/src/hotkey/backend.rs` (HotkeyBackend trait)
  - `src-service/src/hotkey/mod.rs` (module-level API)
  - `src-service/src/hotkey/windows.rs` (Windows backend)
  - `src-service/src/hotkey/macos.rs` (macOS backend)
  - `src-service/src/config.rs` (Config struct, persistence)
  - `src-service/src/ipc/handlers.rs` (SetPushToTalkKey handler)
  - `src-service/src/ptt_controller.rs` (hotkey event polling)
  - `src-service/src/state.rs` (ServiceState.ptt_key)
  - `config.html` (config window layout)
  - `src/config.ts` (config window logic)
  - `src/config.css` (config window styling)
  - `src-tauri/src/lib.rs` (Tauri commands)
  - `src-tauri/src/tray/windows.rs` (config window dimensions)
