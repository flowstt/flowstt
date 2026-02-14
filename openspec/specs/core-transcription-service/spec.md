# core-transcription-service Specification

## Purpose
TBD - created by archiving change refactor-core-transcription-service. Update Purpose after archive.
## Requirements
### Requirement: Transcription Service Process
The system SHALL provide a standalone background service process (`flowstt-service`) that handles all voice transcription functionality. The service runs independently of any GUI or CLI client, automatically starting audio capture and the transcription engine on startup using the default audio input device. Clients are optional and serve as observers and configuration overrides.

#### Scenario: Service starts as standalone process
- **WHEN** the `flowstt-service` binary is executed
- **THEN** it initializes audio backends, loads configuration, selects the default audio input device, and begins audio capture and transcription immediately

#### Scenario: Service starts in Automatic mode without clients
- **WHEN** the service starts with transcription mode set to Automatic and no clients are connected
- **THEN** it auto-selects the default input device, starts continuous audio capture with VAD-based speech detection, and begins transcribing detected speech segments

#### Scenario: Service starts in PTT mode without clients
- **WHEN** the service starts with transcription mode set to Push-to-Talk and no clients are connected
- **THEN** it auto-selects the default input device, starts hotkey monitoring, and begins capturing and transcribing audio when the PTT key is pressed

#### Scenario: Service runs without GUI
- **WHEN** the service is running
- **THEN** it provides full transcription functionality without requiring any display or GUI components

#### Scenario: Service handles multiple sequential sessions
- **WHEN** a client connects, performs transcription, disconnects, and another client connects
- **THEN** the service handles the second client correctly without requiring restart

#### Scenario: No audio devices available on startup
- **WHEN** the service starts and no audio input devices are detected
- **THEN** the service starts but logs a warning and waits for a client to configure audio sources via SetSources

### Requirement: IPC Communication Protocol
The system SHALL use platform-native IPC mechanisms (Unix socket on Linux/macOS, named pipe on Windows) for client-service communication using length-prefixed JSON messages.

#### Scenario: Unix socket on Linux
- **WHEN** the service starts on Linux
- **THEN** it creates a Unix socket at `$XDG_RUNTIME_DIR/flowstt/service.sock`

#### Scenario: Unix socket on macOS
- **WHEN** the service starts on macOS
- **THEN** it creates a Unix socket at `$TMPDIR/flowstt/service.sock`

#### Scenario: Named pipe on Windows
- **WHEN** the service starts on Windows
- **THEN** it creates a named pipe at `\\.\pipe\flowstt-service`

#### Scenario: Message format
- **WHEN** a client sends a request or the service sends a response
- **THEN** the message is formatted as a 4-byte little-endian length prefix followed by JSON payload

#### Scenario: Stale socket cleanup
- **WHEN** the service starts and finds an existing socket file with no listening process
- **THEN** it removes the stale socket and creates a new one

### Requirement: Service Request Handling
The system SHALL handle client requests for device enumeration, transcription control, configuration queries, and status queries. The `AppReady` and `AppDisconnect` requests are removed; the service does not require client readiness signals.

#### Scenario: Ping request
- **WHEN** a client sends a `Ping` request
- **THEN** the service responds with `Pong`

#### Scenario: List devices request
- **WHEN** a client sends a `ListDevices` request
- **THEN** the service responds with all available audio devices for the requested source type

#### Scenario: Set sources overrides auto-selected device
- **WHEN** a client sends a `SetSources` request with valid source IDs while the service is already capturing
- **THEN** the service restarts capture using the specified audio sources instead of the auto-selected defaults

#### Scenario: Stop transcribe request
- **WHEN** a client sends a `StopTranscribe` request while transcription is active
- **THEN** the service finalizes any pending segment and stops capture

#### Scenario: Get status request
- **WHEN** a client sends a `GetStatus` request
- **THEN** the service responds with current transcription state (active, in_speech, queue_depth)

#### Scenario: Get config request
- **WHEN** a client sends a `GetConfig` request
- **THEN** the service responds with all persisted configuration values (transcription_mode, ptt_hotkeys)

#### Scenario: Invalid request handling
- **WHEN** a client sends a malformed or invalid request
- **THEN** the service responds with an `Error` response containing a descriptive message

### Requirement: Service Event Streaming
The system SHALL stream real-time events to connected clients for visualization data and transcription results. When no clients are subscribed, the service SHALL log events instead of discarding them silently.

#### Scenario: Visualization data events
- **WHEN** audio monitoring or transcription is active
- **THEN** the service emits `visualization-data` events containing waveform amplitudes, spectrogram columns, and speech metrics

#### Scenario: Transcription complete events
- **WHEN** a speech segment is transcribed
- **THEN** the service emits a `transcription-complete` event containing the transcribed text

#### Scenario: Speech state events
- **WHEN** speech detection state changes
- **THEN** the service emits `speech-started` or `speech-ended` events

#### Scenario: Events to subscribed clients
- **WHEN** multiple clients are connected and subscribed
- **THEN** streaming events are sent to all subscribed clients

#### Scenario: No clients subscribed logs transcription results
- **WHEN** a transcription completes and no clients are subscribed to events
- **THEN** the service logs the transcription result at info level (e.g., "Transcription complete (no clients): <text>")

#### Scenario: No clients subscribed logs high-frequency events at debug level
- **WHEN** visualization data or other high-frequency events are generated with no subscribed clients
- **THEN** the service logs a debug-level message rather than silently discarding the event

### Requirement: Service Lifecycle Management
The system SHALL manage its lifecycle independently of client connections. The service runs until explicitly stopped via signal or IPC shutdown request.

#### Scenario: Service waits for connections on startup
- **WHEN** the service starts
- **THEN** it begins transcription immediately and listens for optional client connections

#### Scenario: Graceful shutdown on signal
- **WHEN** the service receives SIGTERM or SIGINT
- **THEN** it stops active transcription, closes connections, and exits cleanly

#### Scenario: Shutdown via IPC
- **WHEN** a client sends a Shutdown request
- **THEN** the service stops transcription, broadcasts a shutdown event, and exits cleanly

#### Scenario: Client disconnect does not stop service
- **WHEN** the last client disconnects
- **THEN** the service continues running and transcribing; capture is not interrupted

#### Scenario: Service continues after all clients disconnect
- **WHEN** all clients have disconnected and new speech is detected
- **THEN** the service transcribes the speech and logs the result

### Requirement: Service Security
The system SHALL verify client identity using platform peer credential mechanisms to prevent unauthorized access.

#### Scenario: Peer verification on Unix
- **WHEN** a client connects on Linux or macOS
- **THEN** the service verifies the client's UID matches the service's UID before accepting requests

#### Scenario: Peer verification on Windows
- **WHEN** a client connects on Windows
- **THEN** the service verifies the client process belongs to the same user session

#### Scenario: Unauthorized connection rejected
- **WHEN** a connection fails peer verification
- **THEN** the service closes the connection without processing any requests

### Requirement: Service Audio Backend Integration
The system SHALL initialize and manage platform-specific audio backends for capture and processing.

#### Scenario: Linux backend initialization
- **WHEN** the service starts on Linux
- **THEN** it initializes the PipeWire audio backend

#### Scenario: Windows backend initialization
- **WHEN** the service starts on Windows
- **THEN** it initializes the WASAPI audio backend

#### Scenario: macOS backend initialization
- **WHEN** the service starts on macOS
- **THEN** it initializes the CoreAudio and ScreenCaptureKit backends

#### Scenario: Backend initialization failure
- **WHEN** audio backend initialization fails
- **THEN** the service starts but responds to device/transcription requests with appropriate error messages

### Requirement: Service Transcription Pipeline
The system SHALL maintain the transcription pipeline including speech detection, segment buffering, and Whisper processing.

#### Scenario: Speech detection in service
- **WHEN** transcription is active
- **THEN** the service runs speech detection on captured audio and emits state change events

#### Scenario: Segment extraction in service
- **WHEN** speech ends or duration threshold is reached
- **THEN** the service extracts the audio segment and queues it for transcription

#### Scenario: Whisper transcription in service
- **WHEN** segments are queued
- **THEN** the service processes them sequentially through Whisper and emits transcription results

#### Scenario: Model loading in service
- **WHEN** the first transcription is requested and the model is not loaded
- **THEN** the service loads the Whisper model before processing

