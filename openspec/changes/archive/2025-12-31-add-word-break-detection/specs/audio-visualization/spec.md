## MODIFIED Requirements

### Requirement: Speech Detection Metrics Visualization
The system SHALL display individual speech detection algorithm components as colored line graphs within the speech activity display, including word break indicators.

#### Scenario: Amplitude line displayed
- **WHEN** speech detection metrics are received
- **THEN** the RMS amplitude (in dB) is plotted as a gold/yellow line

#### Scenario: Zero-crossing rate line displayed
- **WHEN** speech detection metrics are received
- **THEN** the zero-crossing rate is plotted as a cyan line

#### Scenario: Spectral centroid line displayed
- **WHEN** speech detection metrics are received
- **THEN** the spectral centroid (in Hz) is plotted as a magenta line

#### Scenario: Onset state indicators displayed
- **WHEN** speech detection metrics are received
- **THEN** voiced onset pending state is indicated with a green marker and whisper onset pending state with a blue marker

#### Scenario: Transient detection indicator displayed
- **WHEN** a transient sound is detected (keyboard click, etc.)
- **THEN** the transient state is indicated with a red marker

#### Scenario: Word break markers displayed
- **WHEN** speech detection metrics indicate a word break
- **THEN** a vertical bar is drawn at that position within the speech state bar region

## ADDED Requirements

### Requirement: Word Break Visualization
The system SHALL display detected word breaks as vertical bar markers overlaying the speech state bar area in the speech activity graph.

#### Scenario: Word break bar rendered
- **WHEN** the speech activity graph renders metrics with `is_word_break: true`
- **THEN** a vertical bar is drawn spanning the full height of the speech state bar region at that time position

#### Scenario: Word break bar styling
- **WHEN** word break bars are rendered
- **THEN** they use a semi-transparent white or light gray color (e.g., rgba(255, 255, 255, 0.6)) to contrast with the green/blue speech bars

#### Scenario: Word break bar width
- **WHEN** word break bars are rendered
- **THEN** they are 1-2 pixels wide to appear as distinct vertical lines

#### Scenario: Multiple word breaks visible
- **WHEN** the speech activity graph scrolls and multiple word breaks are within the visible window
- **THEN** all word break bars are rendered at their correct temporal positions

#### Scenario: Word breaks only in speech regions
- **WHEN** word break markers are rendered
- **THEN** they only appear within regions where speech is active (green or blue speech bar areas)
