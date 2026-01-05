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
The system SHALL automatically start the service if it is not running when a CLI command is executed.

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

