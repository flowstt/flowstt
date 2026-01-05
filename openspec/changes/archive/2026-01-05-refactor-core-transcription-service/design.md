# Design: Core Transcription Service Refactoring

## Context
FlowSTT currently implements all voice transcription capabilities directly within the Tauri GUI application (`src-tauri/`). This architecture prevents:
- Headless operation (servers, containers, remote systems)
- CLI-driven automation and scripting
- Integration with other applications
- Running transcription without the GUI overhead

The omnirec project demonstrates a proven pattern for this separation: a background service (`src-service/`) handles core functionality, a shared library (`src-common/`) defines the IPC protocol, a CLI (`src-cli/`) provides headless access, and the GUI (`src-tauri/`) becomes a thin client.

## Goals
- Extract voice transcription into a standalone service runnable without GUI
- Provide CLI access to all transcription functionality (except visualizations)
- Maintain full GUI feature parity through IPC communication
- Enable scripting and automation via JSON output mode
- Keep the codebase maintainable with clear separation of concerns

## Non-Goals
- Real-time audio visualization in CLI (waveform, spectrogram) - requires GUI
- Cross-machine networking (service runs locally, same machine)
- Changing the core transcription algorithm or Whisper integration
- Supporting multiple simultaneous clients (single-client model like omnirec)

## Architecture

```
                           +-----------------------+
                           |      src-common       |
                           | (flowstt-common crate)|
                           +----------+------------+
                                      |
              +-----------------------+-----------------------+
              |                       |                       |
   +----------v-----------+ +---------v---------+  +----------v-----------+
   |      src-service     | |     src-cli       |  |      src-tauri       |
   | (flowstt-service bin)| |  (flowstt bin)    |  | (flowstt-app bin)    |
   +----------------------+ +-------------------+  +----------------------+
              |                       |                       |
              |         Unix Socket / Named Pipe              |
              +<----------------------------------------------+
```

### Binary Naming Convention

| Crate | Binary Name | Purpose |
|-------|-------------|---------|
| `src-cli` | `flowstt` | Primary CLI executable (like `omnirec`) |
| `src-service` | `flowstt-service` | Background service (like `omnirec-service`) |
| `src-tauri` | `flowstt-app` | GUI application (like `omnirec-app`) |

The CLI is the primary user-facing binary and gets the short name. The service and GUI binaries are suffixed to indicate their role.

### Component Responsibilities

**src-common** (crate: `flowstt-common`, no binary)
- IPC message types (Request, Response enums)
- Audio device representations (AudioDevice, AudioSourceType)
- Transcription configuration types
- Path validation and security utilities
- Platform-agnostic socket path determination

**src-service** (crate: `flowstt-service`, binary: `flowstt-service`)
- Platform audio backends (PipeWire, WASAPI, CoreAudio)
- Audio processing pipeline (resampling, echo cancellation)
- Speech detection (amplitude, ZCR, spectral centroid)
- Whisper FFI and transcription queue
- IPC server listening on Unix socket (Linux/macOS) or named pipe (Windows)
- Event emission for visualization data (consumed by GUI client)

**src-cli** (crate: `flowstt-cli`, binary: `flowstt`)
- Argument parsing via clap
- IPC client connecting to service
- Human-readable and JSON output modes
- Auto-spawn service if not running (looks for `flowstt-service`)
- Signal handling for graceful shutdown

**src-tauri** (crate: `flowstt-app`, binary: `flowstt-app`)
- Tauri GUI shell and window management
- IPC client connecting to service (or spawns `flowstt-service`)
- Visualization rendering (frontend receives events via IPC)
- Window appearance, drag regions, styling

### IPC Protocol

Follows omnirec pattern: length-prefixed JSON messages over Unix socket/named pipe.

```
Request:
  [4 bytes: message length LE u32][JSON payload]

Response:
  [4 bytes: message length LE u32][JSON payload]
```

**Request Types:**
- `Ping` - Health check
- `ListDevices { source_type: Option<AudioSourceType> }` - Enumerate devices
- `StartTranscribe { source1_id, source2_id, aec_enabled, mode }` - Begin
- `StopTranscribe` - End transcription
- `GetStatus` - Current state
- `GetModelStatus` - Whisper model availability
- `DownloadModel` - Trigger model download
- `GetCudaStatus` - GPU acceleration status

**Response Types:**
- `Pong` - Health check response
- `Devices { devices: Vec<AudioDevice> }` - Device list
- `Ok` - Success acknowledgment
- `Status { active, in_speech, queue_depth }` - Current state
- `ModelStatus { available, path }` - Model info
- `CudaStatus { build_enabled, runtime_available, system_info }` - GPU info
- `Error { message }` - Error details

**Event Streaming:**
For real-time visualization, the service streams events to connected clients:
- `visualization-data` - Waveform amplitudes, spectrogram columns, speech metrics
- `transcription-complete` - Transcribed text for a segment
- `speech-started` / `speech-ended` - Speech state transitions

## Decisions

### Decision: Separate Service Process (not library)
**Rationale:** Following omnirec's pattern, using a separate process with IPC provides:
- Clean process isolation (transcription crashes don't affect CLI/GUI)
- Shared state between CLI and GUI (both can query status)
- Graceful service lifecycle management
- Easier debugging and monitoring

**Alternatives considered:**
- Shared library: Would require embedding in both CLI and GUI, no shared state
- In-process threading: Coupling remains, crashes affect caller

### Decision: Unix Socket (Linux/macOS) + Named Pipe (Windows)
**Rationale:** Platform-native IPC mechanisms provide:
- Low latency for real-time audio events
- No network stack overhead
- Peer credential verification for security
- Proven by omnirec implementation

**Alternatives considered:**
- TCP: Unnecessary network overhead, firewall complexity
- gRPC: Heavy dependency for local IPC
- Shared memory: Complex synchronization for this use case

### Decision: Service Auto-Spawn from Clients
**Rationale:** Both CLI and GUI should be usable standalone without manual service management:
- If service not running, client spawns it
- Service runs in background (daemonized)
- Clean shutdown when all clients disconnect (with grace period)

### Decision: Single-Client Event Subscription
**Rationale:** Only one client (either GUI or CLI with `--follow`) receives streaming events at a time:
- Simplifies service implementation
- Matches omnirec's model
- GUI typically wants events; CLI typically doesn't (or uses polling)

## Migration Plan

### Phase 1: Extract Common Types
1. Create `src-common/` crate
2. Define IPC protocol types
3. Add shared audio device types
4. No behavioral changes yet

### Phase 2: Create Service Core
1. Create `src-service/` crate structure
2. Move platform backends from `src-tauri/src/platform/`
3. Move audio processing from `src-tauri/src/audio.rs`, `processor.rs`
4. Move transcription from `src-tauri/src/transcribe*.rs`, `whisper_ffi.rs`
5. Implement IPC server
6. Service can run standalone at this point

### Phase 3: Create CLI Client
1. Create `src-cli/` crate
2. Implement IPC client
3. Add clap argument parsing
4. Implement all commands (list, transcribe, status, stop)
5. Add JSON output mode

### Phase 4: Refactor GUI as Client
1. Replace direct audio/transcription calls with IPC
2. Keep visualization rendering in frontend
3. Auto-spawn service on launch
4. Events forwarded to frontend via Tauri events

### Rollback
Each phase is independently reversible:
- Phase 1: Remove `src-common/`, no impact
- Phase 2: Delete `src-service/`, GUI continues working with old code
- Phase 3: Remove `src-cli/`, no impact on GUI
- Phase 4: Revert `src-tauri/` changes, remove service dependency

## Risks / Trade-offs

### Risk: IPC Latency for Real-Time Audio
**Mitigation:** Use non-blocking I/O with small buffer sizes. Events are batched (like current Tauri events) - the serialization overhead is minimal for the data sizes involved (~1KB per visualization event).

### Risk: Service Process Management Complexity
**Mitigation:** Follow omnirec's proven patterns for:
- Socket/pipe cleanup on crash
- Stale socket detection and removal
- Graceful shutdown with drain period

### Risk: Platform-Specific IPC Differences
**Mitigation:** Abstract behind common trait (already done in omnirec). Code review Windows named pipe handling carefully.

## Open Questions

1. **Model download progress:** Should CLI show download progress bar? Requires streaming response capability.
   - *Likely answer:* Use simple polling with status messages, avoid complexity.

2. **Service auto-shutdown:** How long should service wait after last client disconnects?
   - *Likely answer:* 30 seconds grace period, matching omnirec.

3. **Concurrent CLI invocations:** What happens if user runs `flowstt transcribe` twice?
   - *Likely answer:* Second invocation reports "already transcribing" or attaches to existing session.
