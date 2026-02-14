## Context

The current PTT system models a hotkey as a single `KeyCode` enum variant (one of 15 predefined keys). The user selects it from a dropdown. This proposal replaces that with a recorder-based UI supporting key combinations and multiple bindings. The change spans the shared types crate, both platform backends, the config window frontend, IPC protocol, and config persistence.

### Stakeholders
- End users who want flexible PTT hotkey configuration
- Platform backends (macOS CGEventTap, Windows Raw Input) that must track modifier/key state

## Goals / Non-Goals

### Goals
- Support key combinations (e.g., Ctrl+Alt, Alt+Shift+K) as PTT triggers
- Support multiple independent PTT hotkey bindings (any one activates PTT)
- Replace dropdown with a "press to record" hotkey capture widget
- Support any keyboard key, not just modifiers and function keys
- Maintain backward-compatible config loading (graceful migration from old format)
- Keep PTT semantics as hold-to-talk (all keys in combination must be held)

### Non-Goals
- Toggle-to-talk mode (out of scope)
- Per-hotkey action mapping (all hotkeys trigger the same PTT action)
- Linux backend implementation (remains a stub)
- Mouse button hotkeys

## Decisions

### Data Model: HotkeyCombination replaces KeyCode for PTT config

Replace the single `KeyCode` field with a new `HotkeyCombination` struct:

```rust
/// A set of keys that must all be held simultaneously to trigger PTT.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct HotkeyCombination {
    /// One or more keys that must be held together. Order does not matter.
    pub keys: Vec<KeyCode>,
}
```

The `KeyCode` enum is expanded to include all standard keyboard keys (letters, numbers, punctuation, navigation keys, etc.) beyond the current modifier+function key set.

Config stores `ptt_hotkeys: Vec<HotkeyCombination>` instead of `ptt_key: KeyCode`.

**Alternatives considered:**
- Virtual key code integers instead of enum: Rejected because enums provide type safety, readable serialization, and platform-independent representation.
- Separate modifier flags + trigger key model: Rejected because it artificially distinguishes modifiers from other keys. A flat `Vec<KeyCode>` is simpler and allows combinations like Shift+Ctrl or F13+F14 without special cases.

### Backend: Track all pressed keys, match against combinations

Platform backends shift from monitoring a single key to tracking the full set of currently-pressed keys. On each key event:
1. Update the pressed-keys set.
2. Check if any configured `HotkeyCombination` is a subset of the currently-pressed keys.
3. Emit `Pressed` when a combination becomes fully held; emit `Released` when it is no longer fully held.

The backend trait changes:
```rust
fn start(&mut self, hotkeys: Vec<HotkeyCombination>) -> Result<(), String>;
```

This is a breaking change to `HotkeyBackend::start()`.

**Alternatives considered:**
- One backend instance per hotkey: Rejected because it would require multiple CGEventTaps/Raw Input registrations and complex coordination.
- Matching in the PTT controller instead of the backend: Rejected because the backend already receives raw key events and is the natural place for state tracking.

### Hotkey Recorder: Frontend captures key combinations

The config window replaces the PTT dropdown with a recorder widget. The recorder:
1. Shows a "Record Hotkey" button per binding.
2. When clicked, enters recording mode (listens for keydown events).
3. Accumulates all simultaneously-held keys into a combination.
4. Finalizes the combination when all keys are released (debounce ~200ms after last keyup).
5. Displays the recorded combination as a human-readable string (e.g., "Ctrl + Alt + K").

Recording happens in the Tauri webview using standard DOM `keydown`/`keyup` events. The webview can capture most keys. Some OS-intercepted keys (e.g., Windows key on Windows, some function keys) may not reach the webview -- the recorder should display a warning if no keys are detected within a timeout.

**Alternatives considered:**
- Record at the backend level (Rust side): Would capture OS-intercepted keys but requires a dedicated IPC recording protocol and is significantly more complex. Can be added later if webview-level recording proves insufficient.
- Keep dropdown + add a "custom" option: Doesn't solve the fundamental UX problem of selecting combinations.

### Config Migration

When loading config, if `ptt_key` (old format) is present instead of `ptt_hotkeys`, migrate it automatically:
```rust
// Old: { "ptt_key": "right_alt" }
// New: { "ptt_hotkeys": [{ "keys": ["right_alt"] }] }
```

The migration is one-way. After saving, the new format is used exclusively.

### Config Window Sizing

The config window grows from 400x320 to approximately 480x460 to accommodate:
- A list of hotkey bindings (each showing the combination + a remove button)
- An "Add Hotkey" button that opens a recorder inline
- The recorder widget with status text

The exact size may be adjusted during implementation.

## Risks / Trade-offs

- **OS key interception**: Some keys (e.g., Windows key, Cmd, PrintScreen) are intercepted by the OS before they reach applications. The webview recorder may not capture these. Mitigation: Document known limitations; the backend can still monitor these keys even if the recorder can't capture them. A future enhancement could add backend-level recording.
- **Key conflict with typing**: Allowing any key as a PTT trigger means users could bind a letter key. Mitigation: Show a warning in the UI when the user binds a single non-modifier key that could conflict with typing.
- **Breaking config format**: Existing `ptt_key` configs become invalid. Mitigation: Automatic migration on load (described above).
- **Breaking IPC protocol**: `SetPushToTalkKey` changes signature. Mitigation: The IPC protocol is internal (service <-> Tauri app on same machine); there is no external API contract to maintain.
- **Combination ambiguity**: If the user configures both `Ctrl+Alt` and `Ctrl+Alt+K`, pressing Ctrl+Alt+K would match both. Mitigation: Use longest-match semantics -- the more specific combination takes priority. Since all hotkeys trigger the same PTT action, this is only relevant for press/release timing.

## Open Questions

- Should there be a maximum number of configured hotkey bindings? A reasonable limit (e.g., 5) would simplify the UI. To be decided during implementation.
- Should the recorder have a cancel/escape mechanism? Likely yes -- pressing Escape during recording should cancel without saving.
