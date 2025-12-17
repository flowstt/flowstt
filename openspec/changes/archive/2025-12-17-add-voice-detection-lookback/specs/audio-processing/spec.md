# audio-processing Spec Delta

## ADDED Requirements

### Requirement: Voice Detection Lookback Buffer
The system SHALL maintain a ring buffer of recent audio samples in the speech detector to enable retroactive determination of the true speech start point. The buffer SHALL hold at least 200ms of audio at the current sample rate.

#### Scenario: Ring buffer accumulates samples
- **WHEN** the speech detector processes audio samples
- **THEN** samples are added to the ring buffer, overwriting oldest samples when full

#### Scenario: Ring buffer sized for lookback
- **WHEN** the speech detector is initialized at 48kHz sample rate
- **THEN** the ring buffer holds at least 9,600 samples (200ms)

#### Scenario: Ring buffer resets on speech end
- **WHEN** a `speech-ended` event is emitted
- **THEN** the ring buffer continues accumulating for the next speech detection

### Requirement: Lookback Speech Start Analysis
The system SHALL analyze the ring buffer when speech is confirmed to determine where speech actually began, which is typically earlier than the confirmation point. The lookback analysis SHALL use an amplitude threshold lower than the normal detection threshold to catch the initial onset of speech.

#### Scenario: Lookback finds true start
- **WHEN** speech is confirmed after onset time accumulation
- **THEN** the system scans backward through the ring buffer to find the first sample where amplitude exceeded the lookback threshold (-55dB)

#### Scenario: Lookback threshold is more sensitive
- **WHEN** lookback analysis scans the buffer
- **THEN** it uses a threshold of -55dB (more sensitive than the -42dB voiced threshold or -52dB whisper threshold)

#### Scenario: Lookback finds no earlier start
- **WHEN** the ring buffer contains no samples above the lookback threshold before the onset period
- **THEN** the lookback start is set to the beginning of the ring buffer contents

#### Scenario: Lookback respects buffer bounds
- **WHEN** lookback analysis completes
- **THEN** the identified start point is within the valid ring buffer range

### Requirement: Speech Started Event with Lookback Audio
The system SHALL include lookback audio samples in the `speech-started` event payload, providing downstream consumers (such as transcription) with audio from the true start of speech rather than the confirmation point.

#### Scenario: Event includes lookback samples
- **WHEN** a `speech-started` event is emitted
- **THEN** the event payload includes the audio samples from the lookback start point to the current position

#### Scenario: Lookback samples are chronological
- **WHEN** lookback samples are included in the event
- **THEN** they are ordered from oldest (true start) to newest (current position)

#### Scenario: Lookback samples format
- **WHEN** lookback samples are included in the event
- **THEN** they are mono f32 samples at the detector's sample rate

### Requirement: Lookback Speech Detection Metrics
The system SHALL emit speech detection metrics that distinguish between lookback-determined speech start and confirmed speech detection, enabling visualization of both states.

#### Scenario: Lookback start metric emitted
- **WHEN** speech is confirmed and lookback analysis determines the true start
- **THEN** the system emits metrics indicating the lookback speech state for the duration between true start and confirmation

#### Scenario: Metrics include lookback state
- **WHEN** speech detection metrics are emitted
- **THEN** they include an `is_lookback_speech` flag distinguishing lookback-determined speech from confirmed speech

#### Scenario: Lookback offset reported
- **WHEN** speech is confirmed
- **THEN** the metrics include the lookback offset in milliseconds (time between true start and confirmation)
