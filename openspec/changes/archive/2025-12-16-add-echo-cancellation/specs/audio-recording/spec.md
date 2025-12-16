## MODIFIED Requirements

### Requirement: Mixed Audio Capture
The system SHALL support capturing audio from both an input device and system audio simultaneously, combining them into a single stream. When both sources are active, acoustic echo cancellation SHALL be applied to the microphone input to remove system audio that is picked up acoustically before mixing.

#### Scenario: Mixed mode start
- **WHEN** user starts monitoring or recording in mixed mode
- **THEN** audio is captured from both the selected input device and system audio output simultaneously

#### Scenario: Mixed mode audio combination
- **WHEN** capturing in mixed mode
- **THEN** input and system audio samples are mixed with equal gain (0.5 each) to prevent clipping

#### Scenario: Mixed mode visualization
- **WHEN** monitoring in mixed mode
- **THEN** the waveform and spectrogram display the combined audio from both sources

#### Scenario: Mixed mode transcription
- **WHEN** recording completes in mixed mode
- **THEN** the combined audio is transcribed, capturing speech from both microphone and system audio

#### Scenario: Echo cancellation applied in mixed mode
- **WHEN** capturing in mixed mode with both microphone and system audio active
- **THEN** acoustic echo cancellation is applied to the microphone signal using system audio as the reference before mixing, removing speaker feedback from the microphone input

#### Scenario: Echo cancellation improves transcription
- **WHEN** system audio is playing while user speaks into microphone in mixed mode
- **THEN** the user's speech is clearly captured without duplication of the system audio content
