## ADDED Requirements

### Requirement: Transcribe Toggle Control
The system SHALL display a Transcribe toggle switch in the controls bar that enables or disables automatic speech-triggered transcription mode.

#### Scenario: Transcribe toggle displayed
- **WHEN** the application loads
- **THEN** a Transcribe toggle switch is visible in the control buttons area

#### Scenario: Transcribe toggle enables transcription mode
- **WHEN** the user enables the Transcribe toggle
- **THEN** monitoring starts (if not already active), speech detection triggers recording, and the toggle shows active state

#### Scenario: Transcribe toggle disables transcription mode
- **WHEN** the user disables the Transcribe toggle
- **THEN** speech-triggered recording stops and any in-progress segment is finalized

#### Scenario: Transcribe toggle disabled without source
- **WHEN** no audio source is selected
- **THEN** the Transcribe toggle is disabled

### Requirement: Transcribe Mode Status Display
The system SHALL display status messages that reflect the current transcribe mode state and activity.

#### Scenario: Status shows listening
- **WHEN** transcribe mode is active and idle (not currently capturing speech)
- **THEN** the status displays "Listening..."

#### Scenario: Status shows speech capture in progress
- **WHEN** transcribe mode is active and speech is being captured
- **THEN** the status displays "Recording speech..."

#### Scenario: Status shows transcription pending
- **WHEN** segments are queued for transcription
- **THEN** the status indicates pending transcriptions (e.g., "Transcribing... (2 pending)")

## MODIFIED Requirements

### Requirement: Audio Monitor Mode
The system SHALL allow the user to monitor audio input without recording, for verifying microphone function. When Transcribe mode is enabled, monitoring is implicitly active.

#### Scenario: Start monitoring
- **WHEN** the user clicks the Monitor button while idle
- **THEN** audio streaming begins, the waveform displays live input, and no audio is accumulated for transcription

#### Scenario: Stop monitoring
- **WHEN** the user clicks the Monitor button while monitoring
- **THEN** audio streaming stops and the waveform returns to idle state

#### Scenario: Transcribe mode activates monitoring
- **WHEN** the user enables Transcribe mode while monitoring is inactive
- **THEN** monitoring is automatically enabled and remains active while Transcribe is enabled

#### Scenario: Monitor toggle disabled when transcribe active
- **WHEN** Transcribe mode is active
- **THEN** the Monitor toggle is disabled (monitoring is implicitly on)

#### Scenario: Stopping monitoring stops transcribe
- **WHEN** the user attempts to stop monitoring while Transcribe mode is active
- **THEN** both monitoring and Transcribe mode are disabled
