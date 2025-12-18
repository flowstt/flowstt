## 1. Backend: Ring Buffer for Continuous Capture
- [ ] 1.1 Create `SegmentRingBuffer` struct sized for 30 seconds of audio at 48kHz stereo
- [ ] 1.2 Implement continuous write method that advances write position and wraps
- [ ] 1.3 Implement segment extraction method that copies samples between two indices
- [ ] 1.4 Handle wraparound case when segment spans buffer end/start boundary
- [ ] 1.5 Add method to calculate sample index from lookback offset
- [ ] 1.6 Add method to calculate current segment length (from segment_start_idx to write_pos)
- [ ] 1.7 Implement overflow detection (segment approaching 90% of buffer capacity)
- [ ] 1.8 Implement overflow handling: extract current segment, reset segment_start_idx, remain in speech state

## 2. Backend: Transcription Queue Infrastructure
- [ ] 2.1 Create `TranscriptionQueue` struct with bounded `VecDeque` for audio segments
- [ ] 2.2 Implement `QueuedSegment` struct to hold extracted audio data and metadata
- [ ] 2.3 Add worker thread that processes queued segments sequentially
- [ ] 2.4 Emit `transcription-complete` event for each processed segment
- [ ] 2.5 Add methods to enqueue segments and check queue status

## 3. Backend: Transcribe Mode State Management
- [ ] 3.1 Add `TranscribeState` struct with ring buffer, active flag, in_speech flag, segment_start_idx
- [ ] 3.2 Add transcribe state to `AppState` managed by Tauri
- [ ] 3.3 Create `start_transcribe_mode` Tauri command that initializes ring buffer and starts capture
- [ ] 3.4 Create `stop_transcribe_mode` Tauri command that extracts any pending segment and stops capture
- [ ] 3.5 Create `is_transcribe_active` Tauri command

## 4. Backend: Continuous Capture Integration
- [ ] 4.1 Modify audio processing callback to write samples to ring buffer when transcribe active
- [ ] 4.2 Ensure ring buffer writes occur regardless of speech state (continuous)
- [ ] 4.3 Add mutex/lock strategy to allow concurrent writes and extraction
- [ ] 4.4 Check for buffer overflow before each write, extract if needed
- [ ] 4.5 Verify no sample drops during segment extraction or overflow handling

## 5. Backend: Speech Event Integration
- [ ] 5.1 Subscribe to speech-started events when transcribe mode is active
- [ ] 5.2 On speech-started: calculate segment_start_idx including lookback offset
- [ ] 5.3 Subscribe to speech-ended events when transcribe mode is active
- [ ] 5.4 On speech-ended: extract segment from ring buffer, save WAV, queue for transcription
- [ ] 5.5 Reset in_speech flag after extraction, ready for next segment

## 6. Frontend: UI Control Changes
- [ ] 6.1 Replace "Record" button with "Transcribe" toggle switch
- [ ] 6.2 Remove Record button click handler and recording state
- [ ] 6.3 Add Transcribe toggle change handler that calls backend commands
- [ ] 6.4 Update Monitor toggle to be disabled when Transcribe is active
- [ ] 6.5 Disable source selection while Transcribe is active

## 7. Frontend: State Management Updates
- [ ] 7.1 Add `isTranscribing` state variable
- [ ] 7.2 Update status display for transcribe mode ("Listening...", "Recording speech...", etc.)
- [ ] 7.3 Show queue depth in status when segments are pending
- [ ] 7.4 Continue handling `transcription-complete` events to append text
- [ ] 7.5 Continue handling `recording-saved` events for notifications

## 8. Integration and Testing
- [ ] 8.1 Test continuous capture does not drop samples between segments
- [ ] 8.2 Test lookback audio is correctly included at segment start
- [ ] 8.3 Test multiple rapid speech segments extract correctly from ring buffer
- [ ] 8.4 Test long utterances approaching buffer limit trigger segment split
- [ ] 8.5 Test segment split produces multiple WAV files and transcriptions
- [ ] 8.6 Test no audio is dropped during segment split
- [ ] 8.7 Test transcription results appear in correct order
- [ ] 8.8 Test WAV files are saved for each segment
- [ ] 8.9 Test enable/disable Transcribe toggle in various states
- [ ] 8.10 Verify cargo build succeeds on target platform
