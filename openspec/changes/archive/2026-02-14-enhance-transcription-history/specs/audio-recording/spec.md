## MODIFIED Requirements

### Requirement: Speech Segment Recording
The system SHALL capture speech segments as independent audio recordings, with each segment starting from the lookback-determined speech start point. Segment boundaries are determined by speech detection events within the continuous audio stream. WAV files SHALL be saved in the OS-standard application data directory under a `recordings` subdirectory.

#### Scenario: Segment includes lookback audio
- **WHEN** a speech-started event triggers segment marking
- **THEN** the segment start position includes samples from the lookback period (capturing the true start of speech)

#### Scenario: Segment ends at speech end
- **WHEN** a speech-ended event is received
- **THEN** the segment ends at the current ring buffer position and is extracted for transcription

#### Scenario: Segment saved to WAV file in app data directory
- **WHEN** a speech segment is extracted
- **THEN** the audio is saved to a WAV file in the application data directory (`<data_dir>/recordings/`) with a timestamped filename
- **AND** on Windows this is `%APPDATA%/flowstt/recordings/`
- **AND** on Linux this is `~/.local/share/flowstt/recordings/`
- **AND** on macOS this is `~/Library/Application Support/flowstt/recordings/`

#### Scenario: Long speech produces multiple segments
- **WHEN** continuous speech exceeds the ring buffer capacity
- **THEN** the speech is split into multiple segments, each saved as a separate WAV file and queued for transcription independently
