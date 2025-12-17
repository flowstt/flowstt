# audio-processing Specification

## Purpose
TBD - created by archiving change add-voice-processing-monitor. Update Purpose after archive.
## Requirements
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

### Requirement: Silence Detection Processor
The system SHALL include a silence detection processor that identifies periods of silence in the audio stream.

#### Scenario: Silence detected
- **WHEN** the RMS amplitude of audio samples falls below the silence threshold (-40dB)
- **THEN** the processor logs "Silence detected" to the console

#### Scenario: Sound detected after silence
- **WHEN** the RMS amplitude rises above the silence threshold after a period of silence
- **THEN** the processor logs "Sound detected" to the console

#### Scenario: No duplicate logs
- **WHEN** the audio remains in the same state (silent or not silent)
- **THEN** the processor does not log repeated messages

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

### Requirement: Configurable Speech Detection Parameters
The system SHALL allow configuration of speech detection sensitivity through threshold, hold time, and feature range parameters.

#### Scenario: Default parameters
- **WHEN** the speech detector is created without explicit configuration
- **THEN** it uses default voiced threshold (-40dB), whisper threshold (-50dB), hold time (300ms), voiced onset time (100ms), and whisper onset time (150ms)

#### Scenario: Custom threshold
- **WHEN** a custom threshold is configured
- **THEN** speech detection uses the specified threshold for amplitude comparison

#### Scenario: Dual-mode validation
- **WHEN** audio is analyzed for speech detection
- **THEN** features are validated against both voiced and whisper mode criteria

### Requirement: Zero-Crossing Rate Analysis
The system SHALL compute the zero-crossing rate of audio samples to distinguish voiced speech from impulsive transient sounds and to identify whispered speech characteristics.

#### Scenario: ZCR calculation
- **WHEN** an audio buffer is processed
- **THEN** the system calculates the normalized zero-crossing rate (crossings per sample)

#### Scenario: Voiced speech ZCR
- **WHEN** the ZCR falls within the voiced speech range (0.01-0.20)
- **THEN** the sample passes the ZCR criterion for voiced speech detection

#### Scenario: Whisper speech ZCR
- **WHEN** the ZCR falls within the whisper range (0.10-0.40)
- **THEN** the sample passes the ZCR criterion for whisper speech detection

#### Scenario: Transient ZCR
- **WHEN** the ZCR exceeds 0.40 (characteristic of clicks and impulsive sounds)
- **THEN** the sample is flagged for transient rejection evaluation

### Requirement: Spectral Centroid Estimation
The system SHALL estimate the spectral centroid of audio samples using a computationally efficient approximation to identify speech-band frequency content without requiring FFT.

#### Scenario: Centroid calculation
- **WHEN** an audio buffer is processed
- **THEN** the system calculates an approximate spectral centroid in Hz using the first-difference method

#### Scenario: Voiced speech centroid
- **WHEN** the spectral centroid falls within the voiced speech band (250-4000 Hz)
- **THEN** the sample passes the spectral criterion for voiced speech detection

#### Scenario: Whisper speech centroid
- **WHEN** the spectral centroid falls within the whisper band (400-6000 Hz)
- **THEN** the sample passes the spectral criterion for whisper speech detection

#### Scenario: Transient centroid
- **WHEN** the spectral centroid exceeds 5500 Hz combined with high ZCR
- **THEN** the sample is classified as a transient and rejected

### Requirement: Transient Sound Rejection
The system SHALL explicitly reject impulsive transient sounds such as keyboard clicks, mouse clicks, and similar brief noises that could otherwise trigger false speech detection.

#### Scenario: Transient detection
- **WHEN** audio has both ZCR > 0.40 AND spectral centroid > 5500 Hz
- **THEN** the audio is classified as a transient regardless of amplitude

#### Scenario: Transient resets onset
- **WHEN** a transient is detected during speech onset accumulation
- **THEN** the onset timer is reset and no speech event is emitted

#### Scenario: Transient during speech
- **WHEN** a brief transient occurs during confirmed speech (within hold time)
- **THEN** the transient does not end the speech session prematurely

### Requirement: Whisper Detection Mode
The system SHALL include a dedicated whisper detection mode with parameters tuned for soft, breathy speech that has different acoustic characteristics than voiced speech.

#### Scenario: Whisper mode activation
- **WHEN** audio amplitude is between -50dB and -40dB with whisper-range features
- **THEN** the whisper detection mode evaluates the audio

#### Scenario: Whisper onset time
- **WHEN** whisper-mode audio is detected
- **THEN** a longer onset time (150ms vs 100ms) is required to confirm speech, filtering brief noises

#### Scenario: Whisper to voiced transition
- **WHEN** the user transitions from whispering to normal speech
- **THEN** the speech session continues without interruption

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

