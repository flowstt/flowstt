## Context
The CLI communicates with the service over IPC for all operations. Configuration is persisted to `config.json` by the service. The GUI sets config values via IPC requests (`SetTranscriptionMode`, `SetPushToTalkHotkeys`). The CLI needs a `config` subcommand that can get and set these same values.

## Goals / Non-Goals
- Goals:
  - CLI users can read and write all persisted config values
  - Extensible key-value pattern that trivially supports new config keys
  - Consistent with existing CLI patterns (--json, --quiet, exit codes)
- Non-Goals:
  - Setting runtime-only state (sources, AEC) -- already covered by `transcribe` flags
  - Config file format migration or schema changes
  - Interactive configuration wizard

## Decisions

### 1. Subcommand structure: `config <action> [key] [value]`
- `flowstt config show` -- display all config values
- `flowstt config get <key>` -- display a single config value
- `flowstt config set <key> <value>` -- update a config value
- This flat key-value pattern is simple to extend: adding a new config field only requires adding a key name to the match arms.
- Alternatives considered:
  - Nested subcommands per config area (`config mode set`, `config hotkey add`): more structured but harder to extend and inconsistent with the flat config file layout.
  - Positional-only args (`config transcription_mode automatic`): ambiguous whether this is get or set.

### 2. Online-first with service fallback
- `config get`/`show`: sends a new `GetConfig` IPC request to the running service. If the service is not running, reads `config.json` directly from disk.
- `config set`: sends the appropriate existing IPC request (`SetTranscriptionMode` or `SetPushToTalkHotkeys`) to the running service, which persists the change. If the service is not running, writes directly to `config.json`.
- Rationale: When the service is running, it owns the config state; bypassing it could cause stale state. Allowing offline edits is important for headless setup.
- Alternatives considered:
  - Always require running service: too restrictive for initial setup.
  - Always edit file directly: would desync from running service state.

### 3. New `GetConfig` IPC request
- A single `GetConfig` request returns all persisted config fields in one response, avoiding multiple round-trips.
- The response type `ConfigValues` mirrors the `Config` struct fields.
- Existing `SetTranscriptionMode` and `SetPushToTalkHotkeys` requests are reused for writes -- no new set requests needed.

### 4. Config key naming
- Keys use snake_case matching the JSON config field names: `transcription_mode`, `ptt_hotkeys`.
- Values for `transcription_mode`: `automatic`, `push_to_talk` (matching serde serialization).
- Values for `ptt_hotkeys`: JSON array string, e.g. `'[{"keys":["left_control","left_alt"]}]'`.

## Risks / Trade-offs
- Offline config edits bypass validation -- mitigated by reusing the same serde types for direct file read/write.
- PTT hotkeys value requires JSON input which is less user-friendly -- acceptable for a CLI tool; a future `config hotkey add/remove` convenience wrapper could be added later.

## Open Questions
- None; the design is straightforward given existing patterns.
