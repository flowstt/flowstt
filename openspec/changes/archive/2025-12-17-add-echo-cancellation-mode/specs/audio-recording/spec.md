## ADDED Requirements

### Requirement: Echo Cancellation Recording Mode
The system SHALL provide a recording mode selection that determines how audio from multiple sources is combined. The available modes are "Mixed" (combine both streams) and "Echo Cancel" (output only the primary stream with echo removed).

#### Scenario: Mixed mode selected (default)
- **WHEN** the user selects "Mixed" recording mode
- **THEN** audio capture combines the primary and secondary sources as per existing Mixed Audio Capture behavior

#### Scenario: Echo Cancel mode selected
- **WHEN** the user selects "Echo Cancel" recording mode with both primary and secondary sources active
- **THEN** the system uses the secondary source as an AEC reference signal and outputs only the echo-cancelled primary source

#### Scenario: Echo Cancel mode with single source
- **WHEN** the user attempts to select "Echo Cancel" mode with only one source active
- **THEN** the UI prevents selection or indicates that two sources are required for this mode

#### Scenario: Echo Cancel mode produces voice-only output
- **WHEN** recording in Echo Cancel mode while system audio is playing
- **THEN** the recorded output contains only the user's voice with system audio removed
