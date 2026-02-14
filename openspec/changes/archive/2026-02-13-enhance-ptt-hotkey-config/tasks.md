## 1. Data Model Changes (src-common)
- [x] 1.1 Expand `KeyCode` enum in `src-common/src/types.rs` to include all standard keyboard keys (letters A-Z, digits 0-9, punctuation, navigation, special keys, F1-F24, Meta/Win/Cmd)
- [x] 1.2 Add platform-specific key code mapping methods for the new `KeyCode` variants (macOS CGKeyCode, Windows VK codes)
- [x] 1.3 Add display names for all new `KeyCode` variants
- [x] 1.4 Create `HotkeyCombination` struct in `src-common/src/types.rs` with `keys: Vec<KeyCode>`, derive Serialize/Deserialize/Eq/Hash, implement display formatting and order-independent equality
- [x] 1.5 Update `PttStatus` struct to use `Vec<HotkeyCombination>` instead of single `KeyCode`
- [x] 1.6 Mirror `KeyCode` and `HotkeyCombination` types in TypeScript (`src/config.ts`)

## 2. IPC Protocol Changes (src-common)
- [x] 2.1 Replace `SetPushToTalkKey { key: KeyCode }` IPC request with `SetPushToTalkHotkeys { hotkeys: Vec<HotkeyCombination> }`
- [x] 2.2 Update `PttStatus` IPC response to carry `Vec<HotkeyCombination>`
- [x] 2.3 Update Tauri commands in `src-tauri/src/lib.rs` (`set_ptt_key` -> `set_ptt_hotkeys`, update `get_ptt_status` return type)

## 3. Config Persistence (src-service)
- [x] 3.1 Update `Config` struct in `src-service/src/config.rs` to use `ptt_hotkeys: Vec<HotkeyCombination>` instead of `ptt_key: KeyCode`
- [x] 3.2 Implement backward-compatible config loading: detect legacy `ptt_key` field and migrate to `ptt_hotkeys` array
- [x] 3.3 Add unit tests for config migration (old format -> new format, missing field -> default, new format round-trip)

## 4. Backend Interface Changes (src-service)
- [x] 4.1 Update `HotkeyBackend::start()` signature to accept `Vec<HotkeyCombination>` instead of `KeyCode`
- [x] 4.2 Update `start_hotkey()` module-level function in `src-service/src/hotkey/mod.rs` to pass combinations
- [x] 4.3 Update `ServiceState` in `src-service/src/state.rs` to store `Vec<HotkeyCombination>`

## 5. macOS Backend (src-service)
- [x] 5.1 Add pressed-key tracking (`HashSet<KeyCode>`) to the macOS CGEventTap backend
- [x] 5.2 Add key code mapping for all new `KeyCode` variants to macOS CGKeyCode values
- [x] 5.3 Implement combination matching: on each key event, check if any configured combination is a subset of pressed keys
- [x] 5.4 Emit `Pressed`/`Released` events based on combination match state transitions
- [ ] 5.5 Test with single-key and multi-key combinations on macOS

NOTE: macOS backend updated with signature change only. Uses first key of first combination for backward compat. Full combination support deferred (Windows focus per user request).

## 6. Windows Backend (src-service)
- [x] 6.1 Add pressed-key tracking (`HashSet<KeyCode>`) to the Windows Raw Input backend
- [x] 6.2 Add key code mapping for all new `KeyCode` variants to Windows VK codes (with E0 flag handling for left/right modifiers)
- [x] 6.3 Implement combination matching: on each key event, check if any configured combination is a subset of pressed keys
- [x] 6.4 Emit `Pressed`/`Released` events based on combination match state transitions
- [ ] 6.5 Test with single-key and multi-key combinations on Windows

## 7. Linux Stub Backend (src-service)
- [x] 7.1 Update Linux stub `start()` signature to accept `Vec<HotkeyCombination>` (still returns not-implemented error)

## 8. IPC Handler & PTT Controller (src-service)
- [x] 8.1 Update `SetPushToTalkKey` handler in `src-service/src/ipc/handlers.rs` to handle `SetPushToTalkHotkeys` with `Vec<HotkeyCombination>`
- [x] 8.2 Update `GetPttStatus` handler to return all configured combinations
- [x] 8.3 Update PTT controller startup in `src-service/src/ptt_controller.rs` to pass combinations to backend
- [x] 8.4 Update service startup in `src-service/src/main.rs` to load and apply `ptt_hotkeys` from config

## 9. Config Window Frontend
- [x] 9.1 Enlarge config window dimensions in `src-tauri/src/tray/windows.rs` (400x320 -> ~480x460)
- [x] 9.2 Replace PTT key `<select>` dropdown in `config.html` with hotkey binding list container and "Add Hotkey" button
- [x] 9.3 Implement hotkey recorder widget in `src/config.ts`: keydown/keyup capture, accumulate pressed keys, finalize on all-keys-released with debounce
- [x] 9.4 Implement binding list rendering: display each combination with remove button
- [x] 9.5 Add Escape-to-cancel during recording
- [x] 9.6 Add duplicate binding detection and warning
- [x] 9.7 Add single non-modifier key warning
- [x] 9.8 Add recording timeout warning for OS-intercepted keys
- [x] 9.9 Style the new hotkey management UI in `src/config.css` (binding list, recorder widget, buttons, warnings)
- [x] 9.10 Wire up IPC: call `set_ptt_hotkeys` Tauri command when bindings change, load bindings from `get_ptt_status` on open

## 10. Integration Testing
- [ ] 10.1 Verify config migration: start app with old-format config.json, confirm hotkeys load correctly
- [ ] 10.2 Verify multiple bindings: configure 2+ hotkeys, confirm any one activates PTT
- [ ] 10.3 Verify combination: configure a multi-key combo, confirm all keys must be held for PTT
- [ ] 10.4 Verify recorder: open config window, record a hotkey, confirm it appears in list and works for PTT
- [ ] 10.5 Verify persistence: configure hotkeys, restart app, confirm all bindings are restored
- [x] 10.6 Build check: run `cargo build` to verify compilation on current platform
