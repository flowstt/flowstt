## ADDED Requirements

### Requirement: Transcription Queue
The system SHALL maintain a queue of audio segments awaiting transcription. The queue allows recording to continue while transcription processes previous segments asynchronously.

#### Scenario: Segment queued for transcription
- **WHEN** a speech segment is finalized in transcribe mode
- **THEN** the segment is added to the transcription queue

#### Scenario: Queue processes segments sequentially
- **WHEN** multiple segments are in the transcription queue
- **THEN** segments are processed in FIFO order (oldest first)

#### Scenario: Queue bounded to prevent memory growth
- **WHEN** the transcription queue reaches its maximum capacity (10 segments)
- **THEN** new segments wait until space is available (recording continues, queue blocks)

#### Scenario: Queue drains on transcribe mode stop
- **WHEN** the user disables transcribe mode
- **THEN** remaining queued segments continue to be transcribed until the queue is empty

### Requirement: Async Transcription Worker
The system SHALL run a dedicated worker thread that processes queued transcription segments independently of the recording and monitoring pipeline.

#### Scenario: Worker processes queue
- **WHEN** segments are present in the transcription queue
- **THEN** the worker retrieves and transcribes segments one at a time

#### Scenario: Worker emits results
- **WHEN** the worker completes transcription of a segment
- **THEN** a transcription-complete event is emitted with the transcribed text

#### Scenario: Worker handles empty segments
- **WHEN** a queued segment contains no detectable speech
- **THEN** the worker emits "(No speech detected)" as the transcription result

#### Scenario: Worker tolerates transcription lag
- **WHEN** speech segments are produced faster than transcription can process them
- **THEN** the worker continues processing at its own pace while segments accumulate in the queue

## MODIFIED Requirements

### Requirement: Local Whisper Transcription
The system SHALL transcribe recorded audio to text using a local Whisper model. On Windows and macOS, transcription uses the whisper.cpp shared library loaded via FFI. On Linux, transcription uses the whisper-rs crate. In transcribe mode, segments are processed asynchronously from a queue.

#### Scenario: Successful transcription
- **WHEN** recording stops and audio data is available
- **THEN** the audio is transcribed and the resulting text is displayed in the UI

#### Scenario: Transcription in progress
- **WHEN** transcription is processing
- **THEN** the UI displays a loading indicator

#### Scenario: Windows/macOS library loading
- **WHEN** transcription is requested on Windows or macOS
- **THEN** the whisper.cpp shared library (whisper.dll or libwhisper.dylib) is loaded from the application bundle

#### Scenario: Linux transcription
- **WHEN** transcription is requested on Linux
- **THEN** transcription is performed using the whisper-rs crate

#### Scenario: Queue-based transcription in transcribe mode
- **WHEN** transcribe mode is active and a speech segment is queued
- **THEN** the transcription worker processes the segment from the queue and emits the result
