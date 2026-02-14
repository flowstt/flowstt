## MODIFIED Requirements

### Requirement: Streaming Transcription Display
The system SHALL display transcription output as a scrolling history of individual transcription segments. Each segment represents one PTT hold-transcribe-release cycle or one automatic speech detection segment. New segments are appended to the bottom of the display.

#### Scenario: Segment appended on new transcription
- **WHEN** new transcription text is received from the speech-to-text engine
- **THEN** a new segment row is added to the bottom of the transcription history display

#### Scenario: Each segment displayed as a distinct row
- **WHEN** multiple transcription segments have been received
- **THEN** each segment is displayed as a separate row with visible separation between segments

#### Scenario: Segment includes timestamp
- **WHEN** a transcription segment is displayed
- **THEN** the segment row includes a timestamp indicating when the transcription was captured

#### Scenario: Segment includes copy button
- **WHEN** a transcription segment is displayed
- **THEN** the segment row includes a copy button that copies the segment text to the clipboard when clicked

#### Scenario: Segment includes delete button
- **WHEN** a transcription segment is displayed
- **THEN** the segment row includes a delete button that removes the segment from the display and from the persistent history file
- **AND** deletes the associated cached WAV file if one exists

#### Scenario: Segment includes play button when WAV exists
- **WHEN** a transcription segment is displayed and the associated cached WAV file exists on disk
- **THEN** the segment row includes a play button that plays the WAV audio when clicked

#### Scenario: Segment hides play button when WAV missing
- **WHEN** a transcription segment is displayed and no associated WAV file exists (deleted or expired)
- **THEN** no play button is shown for that segment

#### Scenario: Auto-scroll to newest segment
- **WHEN** a new segment is added to the transcription display
- **THEN** the display automatically scrolls to show the most recent segment at the bottom

#### Scenario: Scrollable history
- **WHEN** the transcription display contains more segments than fit in the visible area
- **THEN** the user can scroll up to view older segments

### Requirement: Mini Waveform Rendering
The system SHALL render a simplified real-time waveform in the mini waveform canvas that shows audio activity without detailed metrics. The mini waveform SHALL only be visible when audio capture is active.

#### Scenario: Mini waveform receives visualization data
- **WHEN** visualization data events are emitted during monitoring
- **THEN** the mini waveform renderer receives and processes the waveform amplitude data

#### Scenario: Mini waveform draws gray line
- **WHEN** the mini waveform renders audio data
- **THEN** it draws the waveform as a gray (#888888) line without glow effects

#### Scenario: Mini waveform has transparent background
- **WHEN** the mini waveform renders
- **THEN** the canvas background is transparent, allowing the header background to show through

#### Scenario: Mini waveform has no decorations
- **WHEN** the mini waveform renders
- **THEN** no grid lines, axis labels, scale indicators, or margins are drawn

#### Scenario: Mini waveform matches full waveform time window
- **WHEN** the mini waveform renders
- **THEN** it displays the same ~80ms time window as the full waveform visualization

#### Scenario: Mini waveform scrolls right to left
- **WHEN** new audio samples arrive
- **THEN** they appear on the right edge of the mini waveform and scroll leftward

#### Scenario: Mini waveform updates at 60fps
- **WHEN** audio monitoring is active
- **THEN** the mini waveform animation loop runs at 60fps for smooth visualization

#### Scenario: Mini waveform hidden when idle
- **WHEN** audio capture is not active
- **THEN** the mini waveform canvas is hidden (display none) rather than showing a flat line

## REMOVED Requirements

### Requirement: Transcription Panel Fade Effect
**Reason**: The user explicitly requested removal of the shadow/fade effect at the top of the transcription panel. The scrolling history display with distinct segment rows eliminates the need for a fade-to-background gradient.
**Migration**: Remove the `.result-fade` div from `index.html` and its CSS styles from `styles.css`.

### Requirement: Transcription Buffer Limit
**Reason**: Replaced by persistent transcription history. The display no longer uses a single in-memory text buffer with a 2000-character cap. Instead, individual segments are stored in a persistent history file and displayed as separate rows.
**Migration**: Remove the `transcriptionBuffer` variable and truncation logic from `main.ts`. History is now managed by the service and loaded on frontend startup.
