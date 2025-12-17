# audio-visualization Spec Delta

## MODIFIED Requirements

### Requirement: Speech Activity Display
The system SHALL display a real-time speech activity visualization below the waveform and spectrogram, showing speech detection state and the underlying detection algorithm components. The speech activity graph SHALL be delayed by the lookback duration (200ms) to allow lookback-determined speech starts to be displayed at their correct temporal position.

#### Scenario: Speech activity graph renders during monitoring
- **WHEN** the user starts audio monitoring
- **THEN** a scrolling speech activity graph appears showing detection metrics over time, updating at 60fps

#### Scenario: Speech activity graph scrolls right to left
- **WHEN** new speech detection metrics arrive
- **THEN** they appear on the right edge of the display and scroll leftward as newer metrics arrive

#### Scenario: Speech activity graph renders during recording
- **WHEN** the user is recording audio
- **THEN** the speech activity visualization is active, showing detection metrics for audio being captured

#### Scenario: Speech activity graph clears on stop when not monitoring
- **WHEN** recording stops and monitoring was not active before recording started
- **THEN** the speech activity display shows an idle state (cleared canvas with background color)

#### Scenario: Speech activity graph continues on stop when monitoring was active
- **WHEN** recording stops and monitoring was active before recording started
- **THEN** the speech activity graph continues displaying live detection metrics

#### Scenario: Speech activity graph is delayed
- **WHEN** speech detection metrics are received
- **THEN** they are buffered for 200ms before being rendered, allowing lookback results to be inserted at the correct position

#### Scenario: Waveform and spectrogram remain real-time
- **WHEN** audio is being monitored
- **THEN** the waveform and spectrogram display with minimal latency while the speech activity graph is intentionally delayed

## ADDED Requirements

### Requirement: Lookback Speech Visualization
The system SHALL visually distinguish lookback-determined speech regions from confirmed speech regions in the speech activity graph, using different colors to show where speech actually started versus when it was confirmed.

#### Scenario: Lookback speech displayed with distinct color
- **WHEN** speech is confirmed and lookback determines an earlier start point
- **THEN** the region between the lookback start and confirmation point is displayed in a distinct lookback color

#### Scenario: Confirmed speech displayed with standard color
- **WHEN** speech is confirmed
- **THEN** the region from confirmation onward is displayed in the standard speech color

#### Scenario: Both regions visible simultaneously
- **WHEN** the speech activity graph scrolls
- **THEN** both lookback and confirmed speech regions are visible, showing the temporal relationship between true start and confirmation

#### Scenario: Lookback region precedes confirmed region
- **WHEN** speech is visualized in the delayed graph
- **THEN** the lookback-colored region appears to the left of (earlier than) the confirmed speech region

### Requirement: Speech Activity Delay Buffer
The system SHALL buffer speech detection metrics for the lookback duration before rendering, enabling retroactive insertion of lookback speech state at the correct temporal position.

#### Scenario: Metrics buffered before rendering
- **WHEN** speech detection metrics are received
- **THEN** they are held in a delay buffer for 200ms before being rendered to the canvas

#### Scenario: Lookback state inserted retroactively
- **WHEN** speech is confirmed with a lookback offset
- **THEN** the lookback speech state is inserted into the delay buffer at the position corresponding to the true speech start

#### Scenario: Buffer maintains temporal ordering
- **WHEN** metrics are rendered from the delay buffer
- **THEN** they are rendered in chronological order with lookback insertions at correct positions
