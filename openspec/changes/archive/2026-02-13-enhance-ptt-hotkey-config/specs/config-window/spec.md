## MODIFIED Requirements

### Requirement: Configuration Window
The system SHALL provide a configuration window for adjusting audio and input settings. The window is accessible from the system tray context menu. The window SHALL be sized to accommodate the hotkey management interface.

#### Scenario: Config window opens from tray
- **WHEN** the user clicks "Settings" in the tray context menu
- **THEN** the configuration window opens centered on screen

#### Scenario: Config window appearance
- **WHEN** the configuration window is visible
- **THEN** it has the same dark theme as other application windows
- **AND** it has rounded corners with a subtle border
- **AND** it has a custom close button (no native title bar)
- **AND** it does not appear in the Windows taskbar

#### Scenario: Config window enlarged dimensions
- **WHEN** the configuration window is created
- **THEN** its dimensions are approximately 480x460 logical pixels to accommodate the hotkey binding list and recorder widget

#### Scenario: Config window is draggable
- **WHEN** the user clicks and drags on any non-interactive background area of the configuration window
- **THEN** the window moves with the cursor to reposition on screen

#### Scenario: Config window close
- **GIVEN** the configuration window is visible
- **WHEN** the user clicks the close button
- **THEN** the configuration window closes
- **AND** the main application continues running

### Requirement: PTT Key Configuration
The configuration window SHALL provide a hotkey management interface for recording, displaying, and managing multiple push-to-talk hotkey bindings. The PTT key dropdown is replaced by a hotkey recorder and binding list.

#### Scenario: Hotkey bindings displayed on open
- **WHEN** the configuration window opens
- **THEN** all currently configured PTT hotkey bindings are displayed in a list
- **AND** each binding shows the key combination in human-readable format (e.g., "Ctrl + Alt + K")
- **AND** each binding has a remove button

#### Scenario: Add hotkey binding via recorder
- **WHEN** the user clicks the "Add Hotkey" button
- **THEN** a hotkey recorder widget activates and displays recording status (e.g., "Press keys...")
- **AND** the recorder captures all keys pressed simultaneously
- **AND** the combination is finalized when all keys are released
- **AND** the new binding is added to the list and takes effect immediately

#### Scenario: Remove hotkey binding
- **WHEN** the user clicks the remove button on a hotkey binding
- **THEN** that binding is removed from the list
- **AND** the change takes effect immediately without requiring a save action

#### Scenario: Cancel hotkey recording
- **WHEN** the user presses Escape during hotkey recording
- **THEN** the recording is cancelled without adding a new binding
- **AND** the recorder returns to its idle state

#### Scenario: Duplicate binding prevented
- **WHEN** the user records a hotkey combination that already exists in the binding list
- **THEN** the duplicate is rejected with visual feedback
- **AND** the existing binding is preserved unchanged

#### Scenario: Recording timeout
- **WHEN** the recorder is active and no keys are detected within a reasonable timeout
- **THEN** a warning is displayed indicating the key may not be capturable from the configuration interface

#### Scenario: Single non-modifier key warning
- **WHEN** the user records a single non-modifier key (e.g., a letter or number)
- **THEN** a warning is displayed indicating this may conflict with normal typing

#### Scenario: PTT hotkeys changed in config window
- **WHEN** the user adds or removes a PTT hotkey binding in the configuration window
- **THEN** the change takes effect immediately without requiring a save action
- **AND** the hotkey backend is reconfigured with the updated set of bindings

#### Scenario: No hotkey bindings configured
- **WHEN** all hotkey bindings have been removed
- **THEN** the list area displays a message indicating no hotkeys are configured
- **AND** push-to-talk is inactive until a binding is added
