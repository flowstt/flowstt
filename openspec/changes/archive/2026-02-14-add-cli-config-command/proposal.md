# Change: Add CLI Configuration Command

## Why
The CLI has no way to read or modify persisted configuration values. The GUI front-end can set transcription mode and PTT hotkeys via IPC, but CLI users must manually edit `config.json` on disk. Adding a `config` subcommand gives CLI users full parity with the GUI for persisted settings, and the extensible `config get/set` pattern makes it trivial to add new configuration keys in the future.

## What Changes
- Add a `config` subcommand to the CLI with `get`, `set`, and `show` actions
- `config get <key>` reads a single persisted config value from the running service (or from the config file if the service is not running)
- `config set <key> <value>` updates a persisted config value via IPC (or writes directly to the config file if the service is not running)
- `config show` displays all current persisted configuration values
- Supported keys: `transcription_mode`, `ptt_hotkeys`
- New IPC request/response pair (`GetConfig` / `ConfigValues`) to retrieve all persisted config in one round-trip
- Respects existing `--json` and `--quiet` output flags

## Impact
- Affected specs: `cli-interface`
- Affected code: `src-cli/` (commands, argument parsing), `src-common/` (IPC request/response types), `src-service/` (IPC handler for GetConfig)
