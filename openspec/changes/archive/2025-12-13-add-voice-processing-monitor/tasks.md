## 1. Backend: Audio Processor Architecture
- [x] 1.1 Create `src-tauri/src/processor.rs` with `AudioProcessor` trait
- [x] 1.2 Implement `SilenceDetector` struct with RMS-based detection and console logging
- [x] 1.3 Add `is_processing_enabled` flag and processor field to `AudioStreamState`
- [x] 1.4 Add processing hook in `process_audio_samples` function

## 2. Backend: Tauri Commands
- [x] 2.1 Add `set_processing_enabled` command to toggle processing state
- [x] 2.2 Add `is_processing_enabled` query command
- [x] 2.3 Register new commands in `invoke_handler`

## 3. Frontend: UI Toggle
- [x] 3.1 Add voice processing toggle element to `index.html`
- [x] 3.2 Add CSS styling for the new toggle (consistent with monitor toggle)
- [x] 3.3 Add `isProcessingEnabled` state variable in `main.ts`
- [x] 3.4 Implement `toggleProcessing` function to call backend command
- [x] 3.5 Wire up toggle event listener

## 4. Validation
- [x] 4.1 Test: Toggle processing while not monitoring (no errors, no processing)
- [x] 4.2 Test: Start monitoring with processing enabled (processing starts)
- [x] 4.3 Test: Enable processing while monitoring (processing starts immediately)
- [x] 4.4 Test: Silence detection logs correctly on state transitions
- [x] 4.5 Test: No duplicate console logs for sustained silence/sound
