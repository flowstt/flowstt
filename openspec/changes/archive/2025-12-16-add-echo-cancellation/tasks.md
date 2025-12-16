## 1. Add Dependency
- [x] 1.1 Add `aec3` crate to `src-tauri/Cargo.toml` (replaced `aec-rs` due to quality issues)

## 2. Simplify Audio Processing State
- [x] 2.1 Remove `is_processing_enabled` field from `AudioStreamState` in `audio.rs`
- [x] 2.2 Remove `is_processing_enabled()` and `set_processing_enabled()` methods from `RecordingState`
- [x] 2.3 Update `process_audio_samples()` to always run speech processor when monitoring/recording
- [x] 2.4 Remove `set_processing_enabled` and `is_processing_enabled` Tauri commands from `lib.rs`

## 3. Add AEC State Management
- [x] 3.1 Add `is_aec_enabled` field to app state (shared with PipeWire thread)
- [x] 3.2 Add `set_aec_enabled` and `is_aec_enabled` Tauri commands
- [x] 3.3 Pass AEC enabled flag to AudioMixer

## 4. Implement AEC in AudioMixer
- [x] 4.1 Add AEC3 instance (VoipAec3) to `AudioMixer` struct
- [x] 4.2 Initialize AEC3 when `num_streams == 2` (mixed mode)
- [x] 4.3 Process in 10ms frames (480 samples × channels) for AEC3
- [x] 4.4 Modify `try_mix_and_send()` to apply AEC before mixing when enabled and both streams active
- [x] 4.5 Process microphone samples through AEC using system audio as reference

## 5. Update Frontend
- [x] 5.1 Rename `processingToggle` to `aecToggle` in `main.ts`
- [x] 5.2 Replace `toggleProcessing()` with `toggleAec()` function
- [x] 5.3 Update toggle to call `set_aec_enabled` instead of `set_processing_enabled`
- [x] 5.4 Remove conditional speech event listener setup (always set up during init)
- [x] 5.5 Update `index.html` toggle label from "Processing" to "Echo Cancel"
- [x] 5.6 Remove `isProcessingEnabled` state variable

## 6. Testing and Validation
- [x] 6.1 Test mixed-mode capture with echo cancellation enabled
- [x] 6.2 Verify transcription quality improvement with system audio playing
- [x] 6.3 Verify single-source capture still works (AEC not applied)
- [x] 6.4 Verify speech events are emitted during monitoring/recording without toggle
- [x] 6.5 Verify AEC toggle only affects mixed-mode behavior
