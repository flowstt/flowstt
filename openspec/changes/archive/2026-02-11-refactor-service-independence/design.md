## Context
The FlowSTT service (`flowstt-service`) is architecturally a standalone daemon, but in Automatic transcription mode it waits for a client to signal readiness (`AppReady`) and configure audio sources (`SetSources`) before starting capture. This creates an implicit dependency on clients and prevents the service from operating as a true headless daemon. PTT mode already bypasses this gate by auto-configuring on startup, creating an inconsistency between the two modes.

The refactoring makes both modes behave uniformly: the service is self-sufficient from the moment it starts.

## Goals / Non-Goals
- Goals:
  - Service starts transcription engine and audio capture on startup without waiting for clients
  - Service listens for PTT key presses immediately in PTT mode (already works, but unify the pattern)
  - Service logs transcription results and events when no clients are subscribed
  - Clients become optional observers that can change settings and receive events
  - Consistent startup behavior between Automatic and PTT modes
- Non-Goals:
  - Changing the IPC protocol framing (length-prefixed JSON stays)
  - Adding new transcription modes or audio features
  - Changing the security/peer verification model
  - Implementing client reference counting (single `app_ready` removal is sufficient)

## Decisions
- **Decision**: Remove `app_ready` field from `ServiceState` and the `AppReady`/`AppDisconnect` IPC requests entirely
  - Alternatives considered:
    - Keep `app_ready` but auto-set it on startup: Leaves dead code and a confusing concept. The field serves no purpose if always true.
    - Convert to a client reference count: Over-engineered for the current use case. The service should simply run regardless.
  - Rationale: Clean removal is simplest. Clients that previously sent `AppReady` can simply stop sending it; the service will already be operational.

- **Decision**: Auto-select default input device on startup for both modes
  - The service already does this for PTT mode in `main.rs:152-166`. Extend the same pattern to Automatic mode.
  - If no devices are found, the service starts but logs a warning and waits for a client to configure sources via `SetSources`.

- **Decision**: Log when events have no subscribers rather than silently dropping
  - The `broadcast_event()` function already uses a `tokio::sync::broadcast` channel. When there are no receivers, the send returns an error. Currently this is silently ignored. Change this to log a concise message (at `debug` level for high-frequency events like visualization, `info` level for transcription results).

- **Decision**: `SetSources` remains available for clients to override the auto-selected device
  - Clients can still configure specific audio sources. This allows the GUI to let users pick non-default devices.
  - When `SetSources` is received, capture restarts with the new configuration.

## Risks / Trade-offs
- **Risk**: Service starts capturing audio immediately without user awareness
  - Mitigation: This is the desired behavior for a background transcription daemon. The service already requires explicit launch (or auto-spawn by a client). Users opting to run the service standalone expect immediate operation.
- **Risk**: Removing `AppReady`/`AppDisconnect` breaks existing GUI and CLI clients
  - Mitigation: The GUI client (`src-tauri/src/ipc_client.rs`) sends `AppReady` on connect and `AppDisconnect` on close. After removal, the service should respond with `Response::Ok` to unknown/deprecated requests, or the clients should be updated to stop sending them. Since all components are in the same repo and released together, updating clients simultaneously is straightforward.
- **Risk**: Auto-shutdown timer (30s after last client disconnect) conflicts with independent operation
  - Mitigation: The auto-shutdown behavior in the `Service Lifecycle Management` spec requirement should be modified. When running independently, the service should not auto-shutdown just because a client disconnects -- it should continue running until explicitly stopped or signaled.

## Migration Plan
1. Remove `app_ready` from `ServiceState` and `should_capture()`
2. Remove `AppReady` and `AppDisconnect` variants from the `Request` enum (respond with error for backward compat if needed)
3. Extend the startup sequence in `main.rs` to auto-select default device and start capture in both modes
4. Add logging fallback in `broadcast_event()` for when no clients are subscribed
5. Update GUI and CLI clients to remove `AppReady`/`AppDisconnect` sends
6. Update the auto-shutdown behavior to not trigger on client disconnect when the service was started independently

## Open Questions
- None; the approach is straightforward given the existing PTT mode precedent.
