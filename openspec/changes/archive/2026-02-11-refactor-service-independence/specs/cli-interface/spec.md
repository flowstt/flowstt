## MODIFIED Requirements

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
