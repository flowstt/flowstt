## 1. Remove `app_ready` gate from service state
- [x] 1.1 Remove `app_ready` field from `ServiceState` in `src-service/src/state.rs`
- [x] 1.2 Simplify `should_capture()` to only check `has_primary_source()` (remove `app_ready` condition)
- [x] 1.3 Remove all reads/writes of `app_ready` across the service codebase

## 2. Remove `AppReady` and `AppDisconnect` IPC requests
- [x] 2.1 Remove `AppReady` and `AppDisconnect` variants from the `Request` enum in `src-common/src/ipc/requests.rs`
- [x] 2.2 Remove corresponding handler logic from `src-service/src/ipc/handlers.rs`
- [x] 2.3 Remove `AppReady`/`AppDisconnect` sends from the GUI client in `src-tauri/src/lib.rs` (replaced `app_ready` command with `connect_events`)
- [x] 2.4 Remove `AppReady`/`AppDisconnect` sends from the CLI client in `src-cli/src/client.rs` (not present - no changes needed)

## 3. Unify startup: auto-configure and start capture in both modes
- [x] 3.1 Refactor `main.rs` startup to auto-select default input device for both Automatic and PTT modes (replaced mode-specific block with unified auto-configure block that calls `start_capture()`)
- [x] 3.2 In Automatic mode, call the equivalent of `start_capture()` during startup (made `start_capture()` public, called from unified startup block)
- [x] 3.3 Handle the case where no audio devices are found: log warning, set `source1_id` to `None`, and wait for a client to configure via `SetSources`
- [x] 3.4 Verify PTT mode startup is unchanged (start_capture() already handles PTT mode correctly)

## 4. Add logging fallback for events with no subscribers
- [x] 4.1 In `broadcast_event()` (or the broadcast channel send path), detect when no receivers are subscribed
- [x] 4.2 Log transcription-complete events at `info` level with the transcribed text
- [x] 4.3 Log high-frequency events (visualization data, speech state) at `debug` level
- [x] 4.4 Log other events (capture state changed, PTT pressed/released, mode changed) at `info` level

## 5. Remove auto-shutdown on client disconnect
- [x] 5.1 Remove or disable the 30-second auto-shutdown timer that triggers when the last client disconnects (no explicit timer existed; `AppDisconnect` handler was the mechanism, now removed)
- [x] 5.2 Ensure the service continues running and transcribing after all clients disconnect

## 6. Update clients
- [x] 6.1 Update the GUI client to not send `AppReady` on connect or `AppDisconnect` on close (replaced with `connect_events` command, updated frontend TypeScript)
- [x] 6.2 Update the CLI client to not send `AppReady` on connect (CLI never sent these requests - no changes needed)
- [x] 6.3 Verify both clients still work correctly (build succeeds, no compilation errors)

## 7. Verify and test
- [x] 7.1 Build the project (`cargo build`) and fix any compilation errors
- [ ] 7.2 Verify the service starts and begins capturing in Automatic mode without any client connected
- [ ] 7.3 Verify the service starts and responds to PTT key presses without any client connected
- [ ] 7.4 Verify a client connecting mid-session receives events correctly
- [ ] 7.5 Verify `SetSources` from a client overrides the auto-selected device
- [ ] 7.6 Verify the service continues running after all clients disconnect
