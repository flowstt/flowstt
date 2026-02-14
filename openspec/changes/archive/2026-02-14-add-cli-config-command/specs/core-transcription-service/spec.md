## MODIFIED Requirements

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
