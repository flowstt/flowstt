# cli-interface Specification

## Purpose
TBD - created by archiving change refactor-core-transcription-service. Update Purpose after archive.
## Requirements
### Requirement: CLI Application Binary
The system SHALL provide a command-line interface binary named `flowstt` that enables headless access to transcription functionality.

#### Scenario: CLI binary executes
- **WHEN** the user runs `flowstt --help`
- **THEN** usage information is displayed showing available commands and options

#### Scenario: Version display
- **WHEN** the user runs `flowstt version`
- **THEN** the CLI displays version information for the CLI and service

### Requirement: Device Enumeration Command
The system SHALL provide a `list devices` command to enumerate available audio sources.

#### Scenario: List all devices
- **WHEN** the user runs `flowstt list devices`
- **THEN** all available input and system audio devices are displayed with ID and name

#### Scenario: List input devices only
- **WHEN** the user runs `flowstt list devices --type input`
- **THEN** only microphone and input devices are displayed

#### Scenario: List system devices only
- **WHEN** the user runs `flowstt list devices --type system`
- **THEN** only system audio (monitor/loopback) devices are displayed

#### Scenario: No devices available
- **WHEN** the user runs `flowstt list devices` and no devices are detected
- **THEN** a message indicates no devices are available

### Requirement: Transcription Command
The system SHALL provide a `transcribe` command to start speech-triggered transcription.

#### Scenario: Start transcription with source
- **WHEN** the user runs `flowstt transcribe --source <device_id>`
- **THEN** transcription begins using the specified audio source

#### Scenario: Start transcription with dual sources
- **WHEN** the user runs `flowstt transcribe --source <id1> --secondary <id2>`
- **THEN** transcription begins capturing from both sources with mixing

#### Scenario: Start transcription with echo cancellation
- **WHEN** the user runs `flowstt transcribe --source <id1> --secondary <id2> --aec`
- **THEN** transcription begins with acoustic echo cancellation enabled

#### Scenario: Transcription output to stdout
- **WHEN** transcription produces text while running
- **THEN** transcribed segments are printed to stdout

#### Scenario: Transcription runs until interrupted
- **WHEN** transcription is active
- **THEN** it continues until the user presses Ctrl+C or runs `flowstt stop`

#### Scenario: Already transcribing error
- **WHEN** the user runs `flowstt transcribe` while transcription is already active
- **THEN** an error message indicates transcription is already in progress

### Requirement: Status Command
The system SHALL provide a `status` command to show current transcription state.

#### Scenario: Status when idle
- **WHEN** the user runs `flowstt status` and no transcription is active
- **THEN** the output indicates the service is idle

#### Scenario: Status when transcribing
- **WHEN** the user runs `flowstt status` during active transcription
- **THEN** the output shows transcription is active with speech state and queue depth

#### Scenario: Status when service not running
- **WHEN** the user runs `flowstt status` and the service is not running
- **THEN** the output indicates the service is not running

### Requirement: Stop Command
The system SHALL provide a `stop` command to halt active transcription.

#### Scenario: Stop active transcription
- **WHEN** the user runs `flowstt stop` while transcription is active
- **THEN** transcription stops and any pending segment is finalized

#### Scenario: Stop when not transcribing
- **WHEN** the user runs `flowstt stop` and no transcription is active
- **THEN** a message indicates there is nothing to stop

### Requirement: JSON Output Mode
The system SHALL support JSON output for all commands to enable scripting and automation.

#### Scenario: JSON device list
- **WHEN** the user runs `flowstt list devices --json`
- **THEN** device information is output as a JSON array

#### Scenario: JSON status
- **WHEN** the user runs `flowstt status --json`
- **THEN** status information is output as a JSON object

#### Scenario: JSON transcription output
- **WHEN** the user runs `flowstt transcribe --json`
- **THEN** each transcription segment is output as a JSON object on a new line (JSON Lines format)

#### Scenario: JSON error output
- **WHEN** an error occurs and `--json` is specified
- **THEN** the error is output as a JSON object with an error field

### Requirement: Service Auto-Spawn
The system SHALL automatically start the service if it is not running when a CLI command is executed. The CLI does not need to send readiness signals after connection; the service is immediately operational.

#### Scenario: Service spawned on first command
- **WHEN** the user runs any CLI command and the service is not running
- **THEN** the CLI spawns the service and waits for it to be ready

#### Scenario: Service found from sibling binary
- **WHEN** the CLI needs to spawn the service
- **THEN** it first looks for `flowstt-service` in the same directory as the CLI binary

#### Scenario: Service found in PATH
- **WHEN** the service binary is not alongside the CLI
- **THEN** the CLI searches PATH for `flowstt-service`

#### Scenario: Service not found error
- **WHEN** the CLI cannot locate the service binary
- **THEN** an error message indicates the service binary was not found

#### Scenario: Service spawn timeout
- **WHEN** the spawned service does not become ready within 10 seconds
- **THEN** an error message indicates service startup timed out

#### Scenario: No AppReady required after connect
- **WHEN** the CLI connects to the service
- **THEN** it does not send an AppReady request; the service is already capturing and transcribing

### Requirement: CLI Output Verbosity
The system SHALL support quiet and verbose output modes.

#### Scenario: Quiet mode suppresses non-essential output
- **WHEN** the user runs a command with `--quiet` or `-q`
- **THEN** only essential output (transcription text, errors) is displayed

#### Scenario: Verbose mode shows detailed output
- **WHEN** the user runs a command with `--verbose` or `-v`
- **THEN** additional information (connection status, timing) is displayed

#### Scenario: Default output
- **WHEN** the user runs a command without verbosity flags
- **THEN** normal human-readable output is displayed

### Requirement: CLI Exit Codes
The system SHALL use meaningful exit codes to indicate command outcome.

#### Scenario: Success exit code
- **WHEN** a command completes successfully
- **THEN** the CLI exits with code 0

#### Scenario: General error exit code
- **WHEN** a command fails due to an error
- **THEN** the CLI exits with code 1

#### Scenario: Service connection failure exit code
- **WHEN** the CLI cannot connect to or spawn the service
- **THEN** the CLI exits with code 2

#### Scenario: Invalid arguments exit code
- **WHEN** the user provides invalid command-line arguments
- **THEN** the CLI exits with code 64 (EX_USAGE)

### Requirement: Model Management Commands
The system SHALL provide commands to check and download the Whisper model.

#### Scenario: Check model status
- **WHEN** the user runs `flowstt model status`
- **THEN** the output shows whether the model is available and its location

#### Scenario: Download model
- **WHEN** the user runs `flowstt model download`
- **THEN** the model is downloaded to the default cache location

#### Scenario: Model already downloaded
- **WHEN** the user runs `flowstt model download` and the model exists
- **THEN** a message indicates the model is already available

### Requirement: GPU Status Command
The system SHALL provide a command to show GPU acceleration status.

#### Scenario: Show CUDA status
- **WHEN** the user runs `flowstt gpu status`
- **THEN** the output shows whether CUDA is enabled at build time and available at runtime

#### Scenario: Show system info
- **WHEN** the user runs `flowstt gpu status --verbose`
- **THEN** the output includes the full Whisper system info string showing all available backends

### Requirement: Configuration Show Command
The system SHALL provide a `config show` command that displays all persisted configuration values.

#### Scenario: Show all config with service running
- **WHEN** the user runs `flowstt config show` and the service is running
- **THEN** all persisted configuration values are retrieved from the service via IPC and displayed

#### Scenario: Show all config with service not running
- **WHEN** the user runs `flowstt config show` and the service is not running
- **THEN** configuration values are read directly from the config file on disk and displayed

#### Scenario: Show config JSON output
- **WHEN** the user runs `flowstt config show --json`
- **THEN** all persisted configuration values are output as a JSON object

#### Scenario: Show config with no config file
- **WHEN** the user runs `flowstt config show`, the service is not running, and no config file exists
- **THEN** default configuration values are displayed

### Requirement: Configuration Get Command
The system SHALL provide a `config get <key>` command that displays the current value of a single persisted configuration key.

#### Scenario: Get transcription mode
- **WHEN** the user runs `flowstt config get transcription_mode`
- **THEN** the current transcription mode value is displayed (e.g., `automatic` or `push_to_talk`)

#### Scenario: Get PTT hotkeys
- **WHEN** the user runs `flowstt config get ptt_hotkeys`
- **THEN** the current PTT hotkey bindings are displayed in human-readable format

#### Scenario: Get config key with JSON output
- **WHEN** the user runs `flowstt config get <key> --json`
- **THEN** the value is output as a JSON value

#### Scenario: Get unknown config key
- **WHEN** the user runs `flowstt config get <unknown_key>`
- **THEN** an error message lists the valid configuration keys
- **AND** the CLI exits with code 64 (EX_USAGE)

#### Scenario: Get config with service not running
- **WHEN** the user runs `flowstt config get <key>` and the service is not running
- **THEN** the value is read directly from the config file on disk

### Requirement: Configuration Set Command
The system SHALL provide a `config set <key> <value>` command that updates a persisted configuration value.

#### Scenario: Set transcription mode
- **WHEN** the user runs `flowstt config set transcription_mode automatic`
- **THEN** the transcription mode is updated to Automatic
- **AND** the change is persisted to the config file

#### Scenario: Set transcription mode to push-to-talk
- **WHEN** the user runs `flowstt config set transcription_mode push_to_talk`
- **THEN** the transcription mode is updated to PushToTalk
- **AND** the change is persisted to the config file

#### Scenario: Set PTT hotkeys
- **WHEN** the user runs `flowstt config set ptt_hotkeys '<json_array>'`
- **THEN** the PTT hotkey bindings are updated to the specified combinations
- **AND** the change is persisted to the config file

#### Scenario: Set config with service running
- **WHEN** the user runs `flowstt config set <key> <value>` and the service is running
- **THEN** the value is sent to the service via the appropriate IPC request
- **AND** the service updates its in-memory state and persists the change

#### Scenario: Set config with service not running
- **WHEN** the user runs `flowstt config set <key> <value>` and the service is not running
- **THEN** the value is written directly to the config file on disk

#### Scenario: Set invalid value
- **WHEN** the user runs `flowstt config set <key> <invalid_value>`
- **THEN** an error message describes the expected value format
- **AND** the CLI exits with code 64 (EX_USAGE)

#### Scenario: Set unknown config key
- **WHEN** the user runs `flowstt config set <unknown_key> <value>`
- **THEN** an error message lists the valid configuration keys
- **AND** the CLI exits with code 64 (EX_USAGE)

#### Scenario: Set config success confirmation
- **WHEN** a `config set` command completes successfully
- **THEN** a confirmation message is displayed showing the key and new value

