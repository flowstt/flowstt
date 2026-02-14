# transcription-history Specification

## Purpose
TBD - created by archiving change enhance-transcription-history. Update Purpose after archive.
## Requirements
### Requirement: Persistent Transcription History
The system SHALL maintain a persistent transcription history in a JSON file stored in the OS-standard application data directory. Each history entry SHALL include a unique identifier, the transcribed text, an ISO 8601 timestamp, and an optional path to the associated cached WAV file.

#### Scenario: History file location
- **WHEN** the service starts
- **THEN** the history file is located at `<data_dir>/history.json`
- **AND** on Windows this is `%APPDATA%/flowstt/history.json`
- **AND** on Linux this is `~/.local/share/flowstt/history.json`
- **AND** on macOS this is `~/Library/Application Support/flowstt/history.json`

#### Scenario: History entry created on transcription complete
- **WHEN** a speech segment is transcribed successfully
- **THEN** a new entry is appended to the history file containing the unique ID, transcribed text, ISO 8601 timestamp, and the WAV file path (if saved successfully)

#### Scenario: History file created if missing
- **WHEN** the service starts and no history file exists
- **THEN** an empty history file is created

#### Scenario: History survives service restart
- **WHEN** the service restarts
- **THEN** all previously saved history entries are available for retrieval

#### Scenario: Corrupted history file handled gracefully
- **WHEN** the history file exists but contains invalid JSON
- **THEN** the service logs a warning and starts with an empty history (the corrupted file is backed up)

### Requirement: Transcription History Retrieval
The system SHALL allow clients to retrieve the full transcription history via an IPC request.

#### Scenario: Client requests history on startup
- **WHEN** a client sends a `GetHistory` request
- **THEN** the service responds with the complete list of history entries ordered by timestamp (oldest first)

#### Scenario: Empty history response
- **WHEN** a client requests history and no entries exist
- **THEN** the service responds with an empty list

### Requirement: Transcription History Entry Deletion
The system SHALL allow clients to delete individual history entries. Deleting an entry SHALL remove it from the history file and delete the associated WAV file if one exists.

#### Scenario: Delete entry with WAV file
- **WHEN** a client sends a `DeleteHistoryEntry` request for an entry that has an associated WAV file
- **THEN** the entry is removed from the history file
- **AND** the associated WAV file is deleted from disk

#### Scenario: Delete entry without WAV file
- **WHEN** a client sends a `DeleteHistoryEntry` request for an entry with no associated WAV file (expired or never saved)
- **THEN** the entry is removed from the history file

#### Scenario: Delete non-existent entry
- **WHEN** a client sends a `DeleteHistoryEntry` request for an ID that does not exist
- **THEN** the service responds with an error indicating the entry was not found

### Requirement: Transcription Complete Event Payload
The system SHALL include segment metadata in the transcription-complete event payload, enabling the frontend to display history entries without a separate round-trip.

#### Scenario: Event includes segment metadata
- **WHEN** a transcription completes and the result event is emitted
- **THEN** the event payload includes the history entry ID, transcribed text, ISO 8601 timestamp, and WAV file path (or null)

#### Scenario: Frontend receives enriched event
- **WHEN** the frontend receives a transcription-complete event
- **THEN** it can directly render a new history segment row using the included metadata

### Requirement: WAV File Automatic Cleanup
The system SHALL automatically delete cached WAV files that are older than 24 hours. When a WAV file is cleaned up, the corresponding history entry's WAV path SHALL be set to null (the text entry is retained).

#### Scenario: Cleanup runs on service startup
- **WHEN** the service starts
- **THEN** it scans the recordings directory and deletes WAV files with a modification time older than 24 hours

#### Scenario: History entries updated after cleanup
- **WHEN** WAV files are deleted during cleanup
- **THEN** any history entries referencing the deleted WAV files have their `wav_path` set to null
- **AND** the history file is saved with the updated entries

#### Scenario: Text entries retained after WAV cleanup
- **WHEN** a WAV file is cleaned up due to age
- **THEN** the corresponding history entry (text, timestamp, ID) remains in the history file

#### Scenario: No WAV files to clean up
- **WHEN** the service starts and all WAV files are less than 24 hours old
- **THEN** no files are deleted and the history file is not modified

### Requirement: WAV File Playback Support
The system SHALL allow clients to retrieve the file path for a history entry's associated WAV file, enabling audio playback in the frontend.

#### Scenario: WAV path available
- **WHEN** a client requests the WAV path for a history entry that has an associated WAV file
- **THEN** the service responds with the absolute file path

#### Scenario: WAV path unavailable
- **WHEN** a client requests the WAV path for a history entry with no WAV file (null path)
- **THEN** the service responds indicating no WAV file is available

#### Scenario: Frontend plays WAV file
- **WHEN** the user clicks the play button on a transcription segment
- **THEN** the frontend uses the WAV file path to play the audio via an HTML5 audio element

