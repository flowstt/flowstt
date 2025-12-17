## MODIFIED Requirements

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
