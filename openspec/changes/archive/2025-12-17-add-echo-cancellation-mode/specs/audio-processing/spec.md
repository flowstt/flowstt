## MODIFIED Requirements

### Requirement: Echo Cancellation Toggle
The system SHALL provide a toggle to enable or disable acoustic echo cancellation. When enabled, echo cancellation is applied during dual-source audio capture. The effect of echo cancellation depends on the selected recording mode: in Mixed mode, echo is removed from the microphone before mixing; in Echo Cancel mode, only the echo-cancelled microphone signal is output.

#### Scenario: Toggle enabled in Mixed mode
- **WHEN** the user enables the echo cancellation toggle in Mixed recording mode with both sources active
- **THEN** acoustic echo cancellation is applied to the microphone signal using system audio as reference, then both streams are mixed

#### Scenario: Toggle enabled in Echo Cancel mode
- **WHEN** the user enables the echo cancellation toggle in Echo Cancel recording mode with both sources active
- **THEN** acoustic echo cancellation is applied to the microphone signal using system audio as reference, and only the echo-cancelled microphone signal is output (no mixing)

#### Scenario: Toggle disabled in Echo Cancel mode
- **WHEN** the user disables the echo cancellation toggle in Echo Cancel recording mode
- **THEN** only the raw primary source (microphone) signal is output without processing

#### Scenario: Toggle enabled in single-source mode
- **WHEN** the user enables the echo cancellation toggle but only one audio source is active
- **THEN** the toggle state is saved but no echo cancellation is applied (not needed for single source)

#### Scenario: Toggle disabled
- **WHEN** the user disables the echo cancellation toggle in Mixed mode
- **THEN** audio is mixed without echo cancellation (simple additive mixing)

#### Scenario: Toggle state persists across source changes
- **WHEN** the user changes audio sources while echo cancellation is enabled
- **THEN** echo cancellation continues to apply if the new configuration has both sources active
