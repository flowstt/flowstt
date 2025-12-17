## 1. Backend: Add Recording Mode State

- [x] 1.1 Add `RecordingMode` enum (Mixed, EchoCancel) in `audio.rs`
- [x] 1.2 Add recording mode field to shared audio state
- [x] 1.3 Add `set_recording_mode` Tauri command in `lib.rs`
- [x] 1.4 Pass recording mode to `AudioMixer` via shared `Arc<Mutex<bool>>` or similar

## 2. Backend: Implement Echo Cancel Mode in Mixer

- [x] 2.1 Add `recording_mode` field to `AudioMixer` struct
- [x] 2.2 Modify `try_mix_and_send()` to check recording mode
- [x] 2.3 In Echo Cancel mode with AEC enabled: output only AEC-processed mic samples
- [x] 2.4 In Echo Cancel mode with AEC disabled: output only raw mic samples
- [x] 2.5 Ensure channel count and sample format match expected output

## 3. Frontend: Add Recording Mode UI

- [x] 3.1 Add recording mode toggle/select element to HTML (next to AEC toggle)
- [x] 3.2 Add CSS styling for the new control
- [x] 3.3 Add recording mode state variable in `main.ts`
- [x] 3.4 Add change handler that calls `set_recording_mode` command
- [x] 3.5 Disable "Echo Cancel" option when only one source is selected
- [x] 3.6 Update status messages to reflect current mode

## 4. Validation

- [ ] 4.1 Test Mixed mode with AEC off - both streams mixed
- [ ] 4.2 Test Mixed mode with AEC on - echo cancelled before mixing
- [ ] 4.3 Test Echo Cancel mode with AEC off - primary stream only
- [ ] 4.4 Test Echo Cancel mode with AEC on - echo cancelled primary stream only
- [ ] 4.5 Test mode switching during active monitoring/recording
- [ ] 4.6 Test UI correctly disables Echo Cancel when single source selected
