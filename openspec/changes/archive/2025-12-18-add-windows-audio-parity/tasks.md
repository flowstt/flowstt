## 1. Phase 1: System Audio Capture (Loopback)

- [x] 1.1 Add `enumerate_render_devices()` function to enumerate output devices using `eRender` endpoint type
- [x] 1.2 Update `list_system_devices()` to call `enumerate_render_devices()` and return loopback sources
- [x] 1.3 Add device type marker to `PlatformAudioDevice` to distinguish input vs loopback in capture logic
- [x] 1.4 Modify `start_capture()` to detect loopback device and use `AUDCLNT_STREAMFLAGS_LOOPBACK` flag
- [x] 1.5 Test loopback capture with system audio playback
- [x] 1.6 Verify visualization shows system audio waveform/spectrogram
- [x] 1.7 Verify transcription works on system audio

## 2. Phase 2: Multi-Source Capture Infrastructure

- [x] 2.1 Create `CaptureStream` struct encapsulating a single WASAPI capture session
- [x] 2.2 Create `MultiCaptureManager` struct to coordinate multiple capture streams
- [x] 2.3 Implement thread-per-stream architecture with `mpsc` channels for sample delivery
- [x] 2.4 Update `start_capture_sources()` to create two streams when both source IDs provided
- [x] 2.5 Add stream synchronization handling for different buffer callback rates
- [x] 2.6 Update `stop_capture()` to stop all active streams cleanly
- [x] 2.7 Test concurrent capture from microphone and system audio

## 3. Phase 3: Audio Mixer Port

- [x] 3.1 Create `AudioMixer` struct with `buffer1` and `buffer2` for stream samples
- [x] 3.2 Implement thread-safe `push_stream1()` and `push_stream2()` methods via channel-based communication
- [x] 3.3 Implement `try_mix_and_send()` with frame-based processing (480 samples = 10ms at 48kHz)
- [x] 3.4 Handle buffer underruns gracefully (partial frames, timing variations)
- [x] 3.5 Integrate mixer into capture thread pipeline
- [x] 3.6 Test mixed audio output from both sources

## 4. Phase 4: Echo Cancellation Integration

- [x] 4.1 Update `Cargo.toml` to make `aec3` dependency cross-platform (Linux and Windows targets)
- [x] 4.2 Add `VoipAec3` initialization to mixer when dual-source capture starts
- [x] 4.3 Process microphone samples through AEC with system audio as reference signal
- [x] 4.4 Wire up `aec_enabled` flag to control AEC processing in mixer
- [x] 4.5 Test echo cancellation with speaker audio being picked up by microphone

## 5. Phase 5: Recording Mode Support

- [x] 5.1 Wire up `recording_mode` flag in mixer processing logic
- [x] 5.2 Implement Mixed mode: combine echo-cancelled mic + system audio at 0.5 gain each
- [x] 5.3 Implement EchoCancel mode: output only echo-cancelled microphone signal
- [x] 5.4 Verify UI recording mode selector works on Windows
- [x] 5.5 Test both recording modes produce expected output

## 6. Phase 6: Testing and Polish

- [x] 6.1 Test with various audio devices (USB, built-in, virtual)
- [x] 6.2 Test with various sample rates (44.1kHz, 48kHz, 96kHz)
- [x] 6.3 Test edge cases: device disconnection during capture
- [x] 6.4 Test edge cases: format changes during capture
- [x] 6.5 Profile performance and optimize if needed (CPU usage during capture)
- [x] 6.6 Verify no memory leaks with extended capture sessions
- [x] 6.7 Run full build to ensure no compilation errors
- [x] 6.8 Update any platform-specific documentation if needed
