## REMOVED Requirements

### Requirement: Voice Processing Toggle
**Reason**: Audio processing (speech detection, visualization) is now always active when monitoring or recording. The toggle has been repurposed to control echo cancellation instead of speech processing.
**Migration**: Remove `is_processing_enabled` state and related Tauri commands. Speech detection runs automatically during capture.

## MODIFIED Requirements

### Requirement: Extensible Audio Processor Architecture
The system SHALL provide a trait-based architecture for audio processors, allowing new processor types to be added without modifying the core audio pipeline. Processors MAY emit events to the frontend via an `AppHandle` parameter. Audio processors run automatically whenever monitoring or recording is active.

#### Scenario: Processor receives samples during monitoring
- **WHEN** monitoring or recording is active
- **THEN** the active processor receives audio samples and an AppHandle reference in the audio callback

#### Scenario: Processor executes without blocking
- **WHEN** a processor processes samples
- **THEN** processing completes within the audio callback without causing audio dropouts

#### Scenario: Processor emits event
- **WHEN** a processor determines an event should be emitted
- **THEN** the processor uses the provided AppHandle to emit the event to the frontend

### Requirement: Speech Detection Events
The system SHALL emit events when speech activity transitions occur, indicating when the user starts and stops speaking. Speech detection SHALL use multi-feature analysis including amplitude, zero-crossing rate, and spectral characteristics to distinguish speech from non-speech audio. The detector SHALL support both voiced and whispered speech through dual-mode detection. Speech detection runs automatically when monitoring or recording is active.

#### Scenario: Voiced speech starts
- **WHEN** audio meets voiced speech criteria (amplitude > -40dB, ZCR 0.01-0.20, spectral centroid 250-4000 Hz) for the configured onset time (100ms)
- **THEN** the system emits a `speech-started` event to the frontend

#### Scenario: Whispered speech starts
- **WHEN** audio meets whisper speech criteria (amplitude > -50dB, ZCR 0.10-0.40, spectral centroid 400-6000 Hz) for the whisper onset time (150ms)
- **THEN** the system emits a `speech-started` event to the frontend

#### Scenario: Speech ends after hold time
- **WHEN** audio amplitude falls below the detection threshold and remains below for the configured hold time (default 300ms)
- **THEN** the system emits a `speech-ended` event to the frontend

#### Scenario: Brief pause during speech
- **WHEN** audio amplitude briefly falls below threshold but returns above threshold before hold time elapses
- **THEN** no `speech-ended` event is emitted (debouncing prevents false triggers)

#### Scenario: Keyboard click rejected
- **WHEN** a brief impulsive sound like a keyboard click produces high amplitude with ZCR > 0.40 and spectral centroid > 5500 Hz
- **THEN** the transient is rejected and no speech-started event is emitted

#### Scenario: Low rumble rejected
- **WHEN** low-frequency ambient noise produces amplitude above threshold but spectral centroid below 250 Hz
- **THEN** the sound is rejected as non-speech

#### Scenario: Soft whispered speech detected
- **WHEN** the user speaks softly or whispers with amplitude between -50dB and -40dB
- **THEN** the whisper detection mode captures the speech after the whisper onset time

## ADDED Requirements

### Requirement: Echo Cancellation Toggle
The system SHALL provide a toggle to enable or disable acoustic echo cancellation. When enabled, echo cancellation is applied during mixed-mode audio capture to remove system audio feedback from the microphone input.

#### Scenario: Toggle enabled in mixed mode
- **WHEN** the user enables the echo cancellation toggle and both microphone and system audio sources are active
- **THEN** acoustic echo cancellation is applied to the microphone signal using system audio as reference

#### Scenario: Toggle enabled in single-source mode
- **WHEN** the user enables the echo cancellation toggle but only one audio source is active
- **THEN** the toggle state is saved but no echo cancellation is applied (not needed for single source)

#### Scenario: Toggle disabled
- **WHEN** the user disables the echo cancellation toggle
- **THEN** audio is mixed without echo cancellation (simple additive mixing)

#### Scenario: Toggle state persists across source changes
- **WHEN** the user changes audio sources while echo cancellation is enabled
- **THEN** echo cancellation continues to apply if the new configuration has both sources active
