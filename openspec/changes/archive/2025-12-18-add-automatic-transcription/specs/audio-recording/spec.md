## ADDED Requirements

### Requirement: Automatic Transcription Mode
The system SHALL provide an automatic transcription mode where audio is captured continuously and speech segments are extracted for transcription based on speech detection events. When enabled, the system monitors for speech activity and extracts each speech segment for transcription without manual intervention.

#### Scenario: Transcribe mode enabled
- **WHEN** the user enables the Transcribe toggle
- **THEN** the system begins continuous audio capture, monitoring for speech activity

#### Scenario: Continuous capture while transcribe active
- **WHEN** transcribe mode is active
- **THEN** audio samples are continuously written to a ring buffer regardless of speech state

#### Scenario: Speech triggers segment marking
- **WHEN** transcribe mode is active and the speech detector emits a speech-started event
- **THEN** the system marks the segment start position (including lookback samples) without interrupting capture

#### Scenario: Speech end triggers segment extraction
- **WHEN** transcribe mode is active and the speech detector emits a speech-ended event
- **THEN** the system extracts (copies) the segment from the ring buffer, saves it to a WAV file, and queues it for transcription

#### Scenario: Capture continues after segment extraction
- **WHEN** a speech segment is extracted from the ring buffer
- **THEN** audio capture continues uninterrupted, ready to capture the next segment

#### Scenario: Transcribe mode disabled
- **WHEN** the user disables the Transcribe toggle
- **THEN** the system stops audio capture and any in-progress segment is finalized and queued

### Requirement: Speech Segment Ring Buffer
The system SHALL maintain a ring buffer for continuous audio capture during transcribe mode. The ring buffer allows segment extraction without interrupting the audio stream, ensuring no samples are dropped between speech segments.

#### Scenario: Ring buffer sized for long utterances
- **WHEN** transcribe mode is initialized
- **THEN** the ring buffer is sized to hold at least 30 seconds of audio at the capture sample rate

#### Scenario: Samples continuously written
- **WHEN** audio samples arrive from the capture backend
- **THEN** samples are written to the ring buffer at the current write position, overwriting old samples when the buffer wraps

#### Scenario: Segment extraction copies samples
- **WHEN** a speech segment is extracted
- **THEN** samples are copied from the ring buffer into a new owned buffer for transcription, leaving the ring buffer intact

#### Scenario: Lookback samples included via ring buffer
- **WHEN** a speech-started event provides a lookback offset
- **THEN** the segment start position is set to include lookback samples already present in the ring buffer

#### Scenario: Buffer overflow triggers segment split
- **WHEN** a speech segment approaches ring buffer capacity (90% full) while speech continues
- **THEN** the current segment is extracted and queued, and a new segment begins at the current position without dropping any audio samples

#### Scenario: Split segment continues speech state
- **WHEN** a segment is split due to buffer overflow
- **THEN** the system remains in speech state, ready to extract the continuation when speech ends or another overflow occurs

### Requirement: Speech Segment Recording
The system SHALL capture speech segments as independent audio recordings, with each segment starting from the lookback-determined speech start point. Segment boundaries are determined by speech detection events within the continuous audio stream.

#### Scenario: Segment includes lookback audio
- **WHEN** a speech-started event triggers segment marking
- **THEN** the segment start position includes samples from the lookback period (capturing the true start of speech)

#### Scenario: Segment ends at speech end
- **WHEN** a speech-ended event is received
- **THEN** the segment ends at the current ring buffer position and is extracted for transcription

#### Scenario: Segment saved to WAV file
- **WHEN** a speech segment is extracted
- **THEN** the audio is saved to a WAV file in the configured recordings directory with a timestamped filename

#### Scenario: Long speech produces multiple segments
- **WHEN** continuous speech exceeds the ring buffer capacity
- **THEN** the speech is split into multiple segments, each saved as a separate WAV file and queued for transcription independently

## MODIFIED Requirements

### Requirement: Audio Recording Control
The system SHALL allow the user to control audio capture through the Transcribe toggle. When Transcribe is active, audio capture runs continuously and speech segments are automatically extracted based on speech detection. Manual recording control is no longer supported.

#### Scenario: Start transcribe mode
- **WHEN** user enables the Transcribe toggle with a device selected
- **THEN** continuous audio capture begins and speech-triggered segment extraction becomes active

#### Scenario: Stop transcribe mode
- **WHEN** user disables the Transcribe toggle
- **THEN** any in-progress speech segment is finalized, audio capture stops, and the ring buffer is released
