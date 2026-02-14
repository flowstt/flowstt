## ADDED Requirements

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
