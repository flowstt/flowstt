# window-appearance Specification

## Purpose
TBD - created by archiving change update-window-appearance. Update Purpose after archive.
## Requirements
### Requirement: Gradient Background
The application SHALL display a dark gray gradient background across the entire window.

#### Scenario: Background renders on launch
- **WHEN** the application window opens
- **THEN** the background displays a smooth dark gray gradient

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

### Requirement: No Title Bar
The application window SHALL display without a native title bar (window decorations disabled).

#### Scenario: Window renders without decorations
- **WHEN** the application window opens
- **THEN** no native title bar or window frame decorations are visible

### Requirement: Custom Drag Region
The application window background SHALL be draggable to allow window repositioning without a native title bar. Interactive elements (buttons, inputs, selects, toggles, canvases) SHALL be excluded from the drag region.

#### Scenario: User drags window via background
- **WHEN** the user clicks and drags on any non-interactive background area
- **THEN** the window moves with the cursor to reposition on screen

#### Scenario: Interactive elements remain functional
- **WHEN** the user clicks on a button, input, select, toggle, or canvas
- **THEN** the element receives the click event normally without initiating window drag

#### Scenario: Windows platform support
- **WHEN** the application runs on Windows
- **THEN** the `-webkit-app-region: drag` CSS property enables native window dragging

### Requirement: Visualization Window
The system SHALL provide a separate resizable window for displaying audio visualizations (waveform, spectrogram, speech activity graph).

#### Scenario: Visualization window opens on demand
- **WHEN** the user double-clicks the mini waveform in the main window header
- **THEN** the visualization window opens displaying all three visualizations

#### Scenario: Visualization window is resizable
- **WHEN** the user drags the edges or corners of the visualization window
- **THEN** the window resizes freely in both dimensions

#### Scenario: Visualization window minimum size
- **WHEN** the user attempts to resize the visualization window below 800x600 pixels
- **THEN** the window stops resizing and maintains at least 800x600 dimensions

#### Scenario: Visualization window closed by default
- **WHEN** the application starts
- **THEN** only the main window is visible; the visualization window is not open

#### Scenario: Visualization window can be closed independently
- **WHEN** the user closes the visualization window
- **THEN** the main window remains open and functional

#### Scenario: Visualization window has no title bar
- **WHEN** the visualization window opens
- **THEN** it displays without native title bar decorations, matching the main window style

#### Scenario: Opening already-open visualization window focuses it
- **WHEN** the user double-clicks the mini waveform while the visualization window is already open
- **THEN** the existing visualization window is focused instead of opening a new window

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

### Requirement: Visualization Window Drag Region
The visualization window background SHALL be draggable to allow window repositioning without a native title bar.

#### Scenario: User drags visualization window via background
- **WHEN** the user clicks and drags on any non-interactive background area of the visualization window
- **THEN** the window moves with the cursor to reposition on screen

#### Scenario: Interactive elements in visualization window remain functional
- **WHEN** the user clicks on a canvas in the visualization window
- **THEN** the element receives the click event normally without initiating window drag

