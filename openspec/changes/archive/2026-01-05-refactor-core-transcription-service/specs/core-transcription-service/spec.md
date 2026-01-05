## ADDED Requirements

### Requirement: Transcription Service Process
The system SHALL provide a standalone background service process (`flowstt-service`) that handles all voice transcription functionality. The service runs independently of any GUI and communicates with clients via IPC.

#### Scenario: Service starts as standalone process
- **WHEN** the `flowstt-service` binary is executed
- **THEN** it initializes audio backends, loads configuration, and begins listening for IPC connections

#### Scenario: Service runs without GUI
- **WHEN** the service is running
- **THEN** it provides full transcription functionality without requiring any display or GUI components

#### Scenario: Service handles multiple sequential sessions
- **WHEN** a client connects, performs transcription, disconnects, and another client connects
- **THEN** the service handles the second client correctly without requiring restart

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
The system SHALL handle client requests for device enumeration, transcription control, and status queries.

#### Scenario: Ping request
- **WHEN** a client sends a `Ping` request
- **THEN** the service responds with `Pong`

#### Scenario: List devices request
- **WHEN** a client sends a `ListDevices` request
- **THEN** the service responds with all available audio devices for the requested source type

#### Scenario: Start transcribe request
- **WHEN** a client sends a `StartTranscribe` request with valid source IDs
- **THEN** the service begins audio capture and speech-triggered transcription

#### Scenario: Stop transcribe request
- **WHEN** a client sends a `StopTranscribe` request while transcription is active
- **THEN** the service finalizes any pending segment and stops capture

#### Scenario: Get status request
- **WHEN** a client sends a `GetStatus` request
- **THEN** the service responds with current transcription state (active, in_speech, queue_depth)

#### Scenario: Invalid request handling
- **WHEN** a client sends a malformed or invalid request
- **THEN** the service responds with an `Error` response containing a descriptive message

### Requirement: Service Event Streaming
The system SHALL stream real-time events to connected clients for visualization data and transcription results.

#### Scenario: Visualization data events
- **WHEN** audio monitoring or transcription is active
- **THEN** the service emits `visualization-data` events containing waveform amplitudes, spectrogram columns, and speech metrics

#### Scenario: Transcription complete events
- **WHEN** a speech segment is transcribed
- **THEN** the service emits a `transcription-complete` event containing the transcribed text

#### Scenario: Speech state events
- **WHEN** speech detection state changes
- **THEN** the service emits `speech-started` or `speech-ended` events

#### Scenario: Events to subscribed client only
- **WHEN** multiple clients are connected
- **THEN** streaming events are sent only to the client that initiated transcription

### Requirement: Service Lifecycle Management
The system SHALL manage its lifecycle based on client connections, including auto-shutdown after a grace period when all clients disconnect.

#### Scenario: Service waits for connections on startup
- **WHEN** the service starts
- **THEN** it listens for client connections indefinitely until manually stopped

#### Scenario: Graceful shutdown on signal
- **WHEN** the service receives SIGTERM or SIGINT
- **THEN** it stops active transcription, closes connections, and exits cleanly

#### Scenario: Auto-shutdown after last client
- **WHEN** the last client disconnects and no new client connects within 30 seconds
- **THEN** the service shuts down automatically

#### Scenario: Active transcription prevents shutdown
- **WHEN** transcription is active and a client disconnects
- **THEN** transcription continues and shutdown timer does not start until transcription completes

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
