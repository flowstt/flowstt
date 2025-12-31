## ADDED Requirements

### Requirement: Word Break Detection
The system SHALL detect brief energy dips within confirmed speech regions that indicate word boundaries. Word break detection only operates during active speech (after speech-started, before speech-ended).

#### Scenario: Word break detected during speech
- **WHEN** speech is active AND amplitude drops below 50% of the recent speech average for 15-200ms
- **THEN** the system identifies a word break at the midpoint of the gap

#### Scenario: Very brief dip ignored
- **WHEN** amplitude drops below threshold for less than 15ms
- **THEN** no word break is detected (filters consonant stops and transients)

#### Scenario: Long gap treated as pause
- **WHEN** amplitude drops below threshold for more than 200ms
- **THEN** no word break is detected (gap is a speech pause, not a word boundary)

#### Scenario: No detection during silence
- **WHEN** speech is not active (before speech-started or after speech-ended)
- **THEN** no word break detection occurs

#### Scenario: Threshold adapts to speech level
- **WHEN** the speaker's volume varies during speech
- **THEN** the word break threshold adapts based on recent speech amplitude (100ms window)

### Requirement: Word Break Events
The system SHALL emit events when word breaks are detected, providing position and duration information for downstream consumers.

#### Scenario: Word break event emitted
- **WHEN** a word break is detected
- **THEN** the system emits a `word-break` event containing the offset from speech start and gap duration

#### Scenario: Event payload structure
- **WHEN** a word-break event is emitted
- **THEN** it contains `offset_ms` (milliseconds from speech start) and `gap_duration_ms` (duration of the gap)

### Requirement: Word Break Metrics
The system SHALL include word break state in speech detection metrics for visualization purposes.

#### Scenario: Metrics include word break flag
- **WHEN** speech detection metrics are emitted during a detected word break gap
- **THEN** the metrics include `is_word_break: true`

#### Scenario: Metrics show no word break normally
- **WHEN** speech detection metrics are emitted outside of a word break gap
- **THEN** the metrics include `is_word_break: false`
