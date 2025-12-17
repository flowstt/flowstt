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
The application window SHALL be non-resizable with a fixed size of 800x600 pixels.

#### Scenario: User attempts to resize window
- **WHEN** the user attempts to resize the window by dragging edges or corners
- **THEN** the window remains at its fixed 800x600 size

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

