# audio-recording Specification

## Purpose
TBD - created by archiving change add-whisper-stt-scaffolding. Update Purpose after archive.
## Requirements
### Requirement: Audio Device Enumeration
The system SHALL enumerate all available audio input devices and system audio sources, presenting them for user selection based on the selected source type.

#### Scenario: Devices listed on load
- **WHEN** the application starts
- **THEN** a dropdown displays all available audio input devices by name for the default source type (Input)

#### Scenario: No devices available
- **WHEN** no audio input devices are detected for the selected source type
- **THEN** the UI displays a message indicating no devices found and disables recording

#### Scenario: Source type change
- **WHEN** the user changes the source type
- **THEN** the device dropdown is repopulated with devices appropriate for the new source type

### Requirement: Audio Recording Control
The system SHALL allow the user to start and stop audio recording from the selected input device.

#### Scenario: Start recording
- **WHEN** user clicks the record button with a device selected
- **THEN** audio capture begins from the selected device and the button indicates recording state

#### Scenario: Stop recording
- **WHEN** user clicks the record button while recording
- **THEN** audio capture stops and the recorded audio is prepared for transcription

### Requirement: Audio Format Conversion
The system SHALL convert recorded audio to 16kHz mono format for Whisper compatibility.

#### Scenario: High sample rate input
- **WHEN** the input device provides audio at a sample rate other than 16kHz
- **THEN** the audio is resampled to 16kHz before transcription

#### Scenario: Stereo input
- **WHEN** the input device provides stereo audio
- **THEN** the audio is converted to mono before transcription

### Requirement: Audio Source Type Selection
The system SHALL allow users to select the audio source type: input device (microphone), system audio (desktop output), or mixed (both combined).

#### Scenario: Input source type selected
- **WHEN** the user selects "Input" as the source type
- **THEN** the device dropdown displays available microphone and input devices

#### Scenario: System source type selected
- **WHEN** the user selects "System" as the source type
- **THEN** the device dropdown displays available system audio sources (monitor devices)

#### Scenario: Mixed source type selected
- **WHEN** the user selects "Mixed" as the source type
- **THEN** the device dropdown displays available input devices and the system captures from both the selected input and the default system audio output

### Requirement: System Audio Device Enumeration
The system SHALL enumerate available system audio sources (monitor/loopback devices) on Linux systems using PipeWire or PulseAudio.

#### Scenario: Monitor sources available
- **WHEN** the system has active audio output devices with PipeWire or PulseAudio
- **THEN** corresponding monitor sources are listed as system audio devices

#### Scenario: No monitor sources available
- **WHEN** no system audio output devices are active
- **THEN** the system audio device list is empty and the UI indicates no system audio sources found

#### Scenario: Monitor source naming
- **WHEN** enumerating system audio devices
- **THEN** devices are displayed with user-friendly names derived from the output device name

### Requirement: System Audio Recording
The system SHALL capture audio from system audio sources (monitor devices) for monitoring and recording, using the same processing pipeline as input devices.

#### Scenario: Start system audio monitoring
- **WHEN** user starts monitoring with a system audio source selected
- **THEN** audio capture begins from the monitor device and visualization displays the system audio

#### Scenario: Start system audio recording
- **WHEN** user starts recording with a system audio source selected
- **THEN** audio is captured from the monitor device and prepared for transcription

#### Scenario: System audio format conversion
- **WHEN** the system audio source provides audio at a sample rate other than 16kHz
- **THEN** the audio is resampled to 16kHz before transcription

### Requirement: Mixed Audio Capture
The system SHALL support capturing audio from both an input device and system audio simultaneously, combining them into a single stream. When both sources are active, acoustic echo cancellation SHALL be applied to the microphone input to remove system audio that is picked up acoustically before mixing.

#### Scenario: Mixed mode start
- **WHEN** user starts monitoring or recording in mixed mode
- **THEN** audio is captured from both the selected input device and system audio output simultaneously

#### Scenario: Mixed mode audio combination
- **WHEN** capturing in mixed mode
- **THEN** input and system audio samples are mixed with equal gain (0.5 each) to prevent clipping

#### Scenario: Mixed mode visualization
- **WHEN** monitoring in mixed mode
- **THEN** the waveform and spectrogram display the combined audio from both sources

#### Scenario: Mixed mode transcription
- **WHEN** recording completes in mixed mode
- **THEN** the combined audio is transcribed, capturing speech from both microphone and system audio

#### Scenario: Echo cancellation applied in mixed mode
- **WHEN** capturing in mixed mode with both microphone and system audio active
- **THEN** acoustic echo cancellation is applied to the microphone signal using system audio as the reference before mixing, removing speaker feedback from the microphone input

#### Scenario: Echo cancellation improves transcription
- **WHEN** system audio is playing while user speaks into microphone in mixed mode
- **THEN** the user's speech is clearly captured without duplication of the system audio content

### Requirement: Echo Cancellation Recording Mode
The system SHALL provide a recording mode selection that determines how audio from multiple sources is combined. The available modes are "Mixed" (combine both streams) and "Echo Cancel" (output only the primary stream with echo removed).

#### Scenario: Mixed mode selected (default)
- **WHEN** the user selects "Mixed" recording mode
- **THEN** audio capture combines the primary and secondary sources as per existing Mixed Audio Capture behavior

#### Scenario: Echo Cancel mode selected
- **WHEN** the user selects "Echo Cancel" recording mode with both primary and secondary sources active
- **THEN** the system uses the secondary source as an AEC reference signal and outputs only the echo-cancelled primary source

#### Scenario: Echo Cancel mode with single source
- **WHEN** the user attempts to select "Echo Cancel" mode with only one source active
- **THEN** the UI prevents selection or indicates that two sources are required for this mode

#### Scenario: Echo Cancel mode produces voice-only output
- **WHEN** recording in Echo Cancel mode while system audio is playing
- **THEN** the recorded output contains only the user's voice with system audio removed

