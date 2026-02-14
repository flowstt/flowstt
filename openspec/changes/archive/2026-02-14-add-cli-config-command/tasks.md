## 1. IPC Protocol (src-common)
- [x] 1.1 Add `GetConfig` variant to the `Request` enum in `src-common/src/ipc/requests.rs`
- [x] 1.2 Add `ConfigValues` variant to the `Response` enum in `src-common/src/ipc/responses.rs` containing `transcription_mode` and `ptt_hotkeys` fields

## 2. Service Handler (src-service)
- [x] 2.1 Add `GetConfig` handler in `src-service/src/ipc/handlers.rs` that reads from `ServiceState` and returns `ConfigValues`

## 3. CLI Config Subcommand (src-cli)
- [x] 3.1 Add `Config` variant to the `Commands` enum in `src-cli/src/main.rs` with `ConfigAction` subcommand enum (`Show`, `Get { key }`, `Set { key, value }`)
- [x] 3.2 Implement `config show` -- connect to service and send `GetConfig`, format output (table for human, JSON for --json); fall back to reading config file directly if service is not running
- [x] 3.3 Implement `config get <key>` -- validate key name, retrieve config (via IPC or file), display the single value
- [x] 3.4 Implement `config set <key> <value>` -- validate key and value, send appropriate IPC request (`SetTranscriptionMode` or `SetPushToTalkHotkeys`) if service is running; otherwise load config file, update field, and save
- [x] 3.5 Add config file direct read/write utility in CLI for offline mode (reuse `Config` struct from service or extract to common)

## 4. Config Struct Sharing
- [x] 4.1 Evaluate whether the `Config` struct should move from `src-service/src/config.rs` to `src-common` so the CLI can reuse it for offline read/write; if not, duplicate the minimal load/save logic in CLI

## 5. Testing
- [x] 5.1 Add unit tests for config key validation and value parsing in the CLI
- [x] 5.2 Add unit test for `GetConfig` handler in the service
- [x] 5.3 Verify `config show`, `config get`, and `config set` work with `--json` flag
- [x] 5.4 Verify offline mode (service not running) for get and set operations

## 6. Documentation
- [x] 6.1 Update `--help` text for the new `config` subcommand and its actions
