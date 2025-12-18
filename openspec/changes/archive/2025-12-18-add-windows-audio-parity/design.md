## Context

FlowSTT requires feature parity between Linux and Windows audio backends. The Linux implementation uses PipeWire and provides:
- System audio capture via sink monitors
- Multi-source capture with mixing
- Echo cancellation using the `aec3` crate

The Windows implementation uses WASAPI and currently only supports single-source microphone capture. This design document outlines the technical approach for achieving full feature parity.

### Stakeholders
- End users on Windows who need system audio capture and mixing
- Developers maintaining cross-platform audio code

### Constraints
- Must use WASAPI (Windows Audio Session API) as the audio backend
- Must maintain the existing `AudioBackend` trait interface
- Echo cancellation must use the same `aec3` crate for consistency

## Goals / Non-Goals

### Goals
- Enable system audio (loopback) capture on Windows
- Support concurrent capture from two audio sources
- Implement audio mixing equivalent to Linux
- Integrate echo cancellation for Windows
- Maintain API compatibility with the `AudioBackend` trait

### Non-Goals
- Changing the Linux implementation
- Supporting more than two simultaneous sources
- Adding new audio processing features beyond echo cancellation
- Supporting exclusive mode capture (shared mode only)

## Decisions

### Decision 1: Thread-per-stream architecture for multi-source capture

**What:** Each audio source (microphone and system loopback) runs in its own dedicated capture thread, with samples sent via channels to a mixer thread.

**Why:** This mirrors the Linux architecture where PipeWire handles stream threading internally. It provides clean separation of concerns and simplifies error handling per-stream.

**Alternatives considered:**
- Single thread with `WaitForMultipleObjects`: More complex, requires careful event handling for multiple audio clients
- Async/await with Windows async APIs: Would require significant refactoring and different threading model

### Decision 2: WASAPI loopback mode for system audio capture

**What:** Use `AUDCLNT_STREAMFLAGS_LOOPBACK` flag when initializing audio clients for system audio devices.

**Why:** This is the standard WASAPI mechanism for capturing audio being played to an output device. It attaches a capture stream to a render endpoint.

**Implementation:**
```rust
// Enumerate render endpoints for loopback sources
enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)

// Initialize for loopback capture
audio_client.Initialize(
    AUDCLNT_SHAREMODE_SHARED,
    AUDCLNT_STREAMFLAGS_LOOPBACK | AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
    buffer_duration,
    0,
    mix_format,
    None,
)
```

### Decision 3: Cross-platform `aec3` dependency

**What:** Move the `aec3` crate dependency from Linux-only to all platforms.

**Why:** The `aec3` crate is a pure Rust port of WebRTC's AEC3 algorithm with no platform-specific code. Making it available on Windows enables echo cancellation without code duplication.

**Current (Cargo.toml):**
```toml
[target.'cfg(target_os = "linux")'.dependencies]
aec3 = "0.1"
```

**Proposed:**
```toml
[dependencies]
aec3 = "0.1"
```

### Decision 4: Port AudioMixer from Linux implementation

**What:** Create a `WasapiAudioMixer` struct that mirrors the functionality of the Linux `AudioMixer`.

**Why:** The mixing logic (frame-based processing, AEC integration, recording mode support) is identical. Porting ensures consistent behavior across platforms.

**Key components to port:**
- `buffer1` and `buffer2` for stream sample accumulation
- `push_stream1()` and `push_stream2()` for thread-safe sample delivery
- `try_mix_and_send()` for frame-based mixing (480 samples at 48kHz = 10ms)
- Recording mode handling (Mixed vs EchoCancel)

## Risks / Trade-offs

### Risk 1: WASAPI loopback device limitations
- **Risk:** Some audio devices or drivers may not support loopback capture
- **Mitigation:** Graceful error handling with clear user messaging; test on multiple systems

### Risk 2: Stream synchronization drift
- **Risk:** Different capture threads may have varying callback rates, causing audio drift
- **Mitigation:** Use timestamped buffers and frame-based mixing (same approach as Linux)

### Risk 3: AEC3 performance on Windows
- **Risk:** Echo cancellation may have different performance characteristics on Windows
- **Mitigation:** Use identical parameters as Linux; profile if issues arise

### Trade-off: Thread overhead vs complexity
- **Trade-off:** Using separate threads per stream adds thread management overhead but simplifies the capture logic
- **Justification:** Matches Linux architecture, easier to maintain and debug

## Implementation Phases

### Phase 1: System Audio Capture (Loopback)
- Add `enumerate_render_devices()` function
- Update `list_system_devices()` to return render devices
- Modify capture initialization to detect and handle loopback mode

### Phase 2: Multi-Source Capture Infrastructure
- Create `CaptureStream` struct for encapsulating single-stream capture
- Create `MultiCaptureManager` for coordinating multiple streams
- Implement channel-based sample delivery from capture threads

### Phase 3: Audio Mixer Port
- Create `WasapiAudioMixer` with thread-safe sample buffering
- Implement frame-based mixing with configurable output mode
- Integrate mixer into capture pipeline

### Phase 4: Echo Cancellation Integration
- Update Cargo.toml for cross-platform `aec3`
- Initialize AEC3 in mixer when dual-source capture starts
- Wire up `aec_enabled` flag

### Phase 5: Recording Mode Support
- Implement Mixed mode output (combined streams)
- Implement EchoCancel mode output (voice-only)
- Wire up `recording_mode` flag

### Phase 6: Testing and Polish
- Test with various audio devices and sample rates
- Handle edge cases (device disconnection, format changes)
- Performance profiling

## Open Questions

None - the gap analysis document provides clear implementation requirements and the Linux implementation serves as a reference.
