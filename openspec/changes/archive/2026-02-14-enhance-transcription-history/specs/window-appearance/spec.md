## MODIFIED Requirements

### Requirement: Fixed Window Size
The main application window SHALL be resizable with a default compact size of 900x340 pixels and a minimum size of 600x300 pixels. All content and components SHALL resize within the window, maintaining their position and margins relative to the window edges.

#### Scenario: Default window size on launch
- **WHEN** the application window opens for the first time
- **THEN** the window displays at 900x340 pixels

#### Scenario: User resizes main window
- **WHEN** the user drags the edges or corners of the main window
- **THEN** the window resizes freely in both dimensions and all content adapts to the new size

#### Scenario: Minimum size enforced
- **WHEN** the user attempts to resize the main window below 600x300 pixels
- **THEN** the window stops resizing and maintains at least 600x300 dimensions

#### Scenario: Content maintains relative layout on resize
- **WHEN** the main window is resized
- **THEN** the header, controls bar, and transcription area maintain their relative positions and margins to the window edges
- **AND** the transcription area expands or contracts to fill available space

### Requirement: Mini Waveform Display
The main window SHALL display a miniature real-time waveform visualization in the header area next to the application logo. The mini waveform SHALL only be visible when audio recording or capture is active.

#### Scenario: Mini waveform position and alignment
- **WHEN** the main window renders and audio capture is active
- **THEN** the mini waveform appears immediately to the right of the logo, vertically centered with the logo

#### Scenario: Mini waveform size
- **WHEN** the mini waveform renders
- **THEN** its height is proportional to the logo height and its width provides adequate visualization (~120 pixels wide)

#### Scenario: Mini waveform appearance
- **WHEN** the mini waveform renders
- **THEN** it displays a gray waveform line on a transparent background with no scale, axis labels, or grid lines

#### Scenario: Mini waveform animates in real-time
- **WHEN** audio monitoring or transcription is active
- **THEN** the mini waveform displays scrolling audio amplitude in real-time, matching the time window of the full waveform (~80ms)

#### Scenario: Mini waveform hidden when idle
- **WHEN** audio capture is not active (no PTT key held, no automatic speech capture in progress)
- **THEN** the mini waveform is not visible (hidden via CSS display none)

#### Scenario: Mini waveform becomes visible on capture start
- **WHEN** audio capture begins (PTT key pressed or transcribe mode activated)
- **THEN** the mini waveform becomes visible and begins animating

#### Scenario: Mini waveform hides on capture stop
- **WHEN** audio capture stops (PTT key released and transcription completes, or transcribe mode deactivated)
- **THEN** the mini waveform is hidden

#### Scenario: Mini waveform opens visualization window
- **WHEN** the user double-clicks the mini waveform
- **THEN** the visualization window opens (or focuses if already open)
