## MODIFIED Requirements

### Requirement: System Audio Device Enumeration
The system SHALL enumerate available system audio sources (monitor/loopback devices) using the platform-appropriate audio backend. On Linux, this uses PipeWire or PulseAudio monitor sources. On Windows, this uses WASAPI loopback mode to enumerate render endpoints as capturable system audio sources. On macOS, the stub backend returns an empty list until full support is implemented.

#### Scenario: Monitor sources available (Linux)
- **WHEN** the system has active audio output devices on Linux with PipeWire or PulseAudio
- **THEN** corresponding monitor sources are listed as system audio devices

#### Scenario: Loopback sources available (Windows)
- **WHEN** the system has active audio render endpoints on Windows
- **THEN** corresponding loopback sources are listed as system audio devices with user-friendly names

#### Scenario: No monitor sources available
- **WHEN** no system audio output devices are active or the platform backend does not support system audio
- **THEN** the system audio device list is empty and the UI indicates no system audio sources found

#### Scenario: Monitor source naming
- **WHEN** enumerating system audio devices
- **THEN** devices are displayed with user-friendly names derived from the output device name

## ADDED Requirements

### Requirement: Windows Audio Backend (Full)
The system SHALL provide a fully functional audio backend for Windows using WASAPI, supporting all audio capture features including input device capture, system audio capture (loopback), multi-source mixing, and echo cancellation. This achieves feature parity with the Linux PipeWire backend.

#### Scenario: Windows backend compiles
- **WHEN** the application is compiled on Windows
- **THEN** compilation succeeds using the WASAPI backend

#### Scenario: Windows input device enumeration
- **WHEN** device enumeration is requested on Windows
- **THEN** available input devices (microphones) are returned with their names and IDs

#### Scenario: Windows system audio enumeration
- **WHEN** system audio device enumeration is requested on Windows
- **THEN** available render endpoints are returned as loopback sources with their names and IDs

#### Scenario: Windows single-source capture starts
- **WHEN** the user starts capture with a single input device selected on Windows
- **THEN** audio capture begins from the selected device and samples are delivered via the backend interface

#### Scenario: Windows single-source capture stops
- **WHEN** the user stops capture on Windows
- **THEN** audio capture stops and resources are released

#### Scenario: Windows loopback capture starts
- **WHEN** the user starts capture with a system audio source selected on Windows
- **THEN** audio capture begins from the selected render endpoint using WASAPI loopback mode

#### Scenario: Windows multi-source capture starts
- **WHEN** the user starts capture with both an input device and system audio source on Windows
- **THEN** audio capture begins from both sources simultaneously using separate capture threads

#### Scenario: Windows multi-source audio mixing
- **WHEN** capturing from both input and system sources on Windows
- **THEN** samples from both sources are mixed using frame-based processing (10ms frames at 48kHz)

#### Scenario: Windows echo cancellation applied
- **WHEN** capturing from both sources on Windows with echo cancellation enabled
- **THEN** the AEC3 algorithm is applied to the microphone signal using system audio as reference

#### Scenario: Windows recording mode Mixed
- **WHEN** capturing from both sources on Windows in Mixed recording mode
- **THEN** echo-cancelled microphone and system audio are combined with soft clipping to prevent distortion

#### Scenario: Windows recording mode EchoCancel
- **WHEN** capturing from both sources on Windows in EchoCancel recording mode
- **THEN** only the echo-cancelled microphone signal is output (no system audio in output)

#### Scenario: Windows backend provides consistent sample format
- **WHEN** the Windows backend delivers audio samples
- **THEN** samples are provided as stereo f32 interleaved format at 48kHz (resampled if device uses different rate)

## REMOVED Requirements

### Requirement: Windows Audio Backend (Stub)
**Reason**: Replaced by "Windows Audio Backend (Full)" which provides complete feature support.
**Migration**: All scenarios from the stub requirement are preserved in the new Full requirement, with additional scenarios for the previously stubbed features.
