# Change: Refactor service to operate independently of clients

## Why
The service currently requires a client (GUI or CLI) to send `AppReady` and `SetSources` before it starts transcription in Automatic mode. This couples the service lifecycle to client connections and prevents headless/daemon operation. The service should be fully self-sufficient -- starting the transcription engine and listening for PTT key presses immediately on startup, regardless of whether any clients are connected.

## What Changes
- **BREAKING**: Remove the `app_ready` gate from the service startup flow; the service auto-configures and begins capture on startup using the default audio device
- **BREAKING**: Remove `AppReady` and `AppDisconnect` IPC requests; clients no longer control service readiness
- Unify PTT and Automatic mode startup: both modes auto-select the default input device and begin operation immediately
- When no clients are subscribed to events, log a message instead of broadcasting (e.g., "Transcription complete (no clients): <text>")
- Clients become purely observational/configurational -- they can change settings, subscribe to events, and receive transcription results, but the service does not depend on them to function
- The `should_capture()` check is simplified to only require a configured primary source (no `app_ready` flag)

## Impact
- Affected specs: `core-transcription-service`, `cli-interface`
- Affected code: `src-service/src/main.rs`, `src-service/src/state.rs`, `src-service/src/ipc/handlers.rs`, `src-common/src/ipc/requests.rs`, `src-common/src/ipc/responses.rs`, `src-tauri/src/ipc_client.rs`, `src-cli/src/client.rs`
