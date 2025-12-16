## Context

FlowSTT supports mixed audio capture combining microphone input (source1) and system audio (source2). The current implementation uses simple additive mixing with 0.5 gain per source. When system audio plays through speakers, the microphone picks up that audio, causing it to appear twice in the mixed output - once from the actual system audio stream and once from the microphone's acoustic pickup. This degrades speech recognition quality.

The `aec3` crate provides a pure Rust port of WebRTC's AEC3 algorithm, which is state-of-the-art echo cancellation designed for modern sample rates.

Additionally, the current "Processing" toggle controls whether speech detection runs. This is unnecessary complexity - speech detection and visualization should always run when audio is being captured. The toggle will be repurposed to control echo cancellation instead.

## Goals / Non-Goals

Goals:
- Remove system audio echo from microphone signal before mixing
- Maintain real-time performance in the audio callback
- Preserve audio quality for speech transcription
- Simplify UI by making audio processing always-on during capture
- Provide user control over echo cancellation (toggle on/off)

Non-Goals:
- Supporting AEC for non-mixed capture modes (single source doesn't need it)
- UI configuration of AEC parameters (delay hints, config tuning)
- Independent noise suppression control

## Decisions

**Decision: Use aec3 crate (WebRTC AEC3 port)**

Initially attempted `aec-rs` (Speex-based AEC) but encountered severe quality issues at 48kHz:
- Clicking and distortion artifacts
- Very low microphone levels
- Speex AEC designed for 8-16kHz telephony, not 48kHz

The `aec3` crate provides:
- Pure Rust implementation (no C++ dependencies)
- Native 48kHz support
- WebRTC AEC3 algorithm (state-of-the-art, used in Chrome)
- Simple VoipAec3 wrapper API

**Decision: Apply AEC in the mixer before combining streams**

The `AudioMixer` struct already buffers samples from both streams. AEC will be applied when mixing: the microphone samples are processed with the system audio samples as the reference, then the cleaned microphone signal is mixed with the system audio.

**Decision: Use VoipAec3 wrapper with interleaved f32 samples**

```rust
VoipAec3::builder(48000, channels, channels)
    .enable_high_pass(true)
    .build()
```

The wrapper handles:
- Interleaved f32 sample format (native to our pipeline)
- 10ms frame alignment (480 samples per channel)
- Proper render/capture ordering

**Decision: Configure AEC for 48kHz processing**

PipeWire delivers audio at native rate (48kHz). AEC3 processes natively at this rate:
- `sample_rate`: 48000 Hz
- `frame_size`: 480 samples per channel (10ms frames)
- High-pass filter enabled (removes DC offset)

**Decision: Replace Processing toggle with Echo Cancel toggle**

The current "Processing" toggle controls speech detection. This will be changed:

1. Remove `is_processing_enabled` state from `AudioStreamState`
2. Remove `set_processing_enabled` and `is_processing_enabled` Tauri commands
3. Audio processing (speech detection, visualization) runs automatically when monitoring or recording
4. Add new `aec_enabled` state to control echo cancellation
5. Add `set_aec_enabled` and `is_aec_enabled` Tauri commands
6. Rename UI toggle from "Processing" to "Echo Cancel"
7. AEC toggle only affects mixed-mode capture (no effect on single-source capture)

**Decision: Speech events always emit during capture**

Speech detection events (`speech-started`, `speech-ended`) will always be emitted when monitoring or recording is active. The frontend will always set up listeners. This simplifies the state machine and ensures cadence analysis data is always available.

## Risks / Trade-offs

- **Latency**: AEC adds processing latency. At 10ms frame size, latency is minimal and acceptable for transcription use case.
- **CPU usage**: AEC3 is optimized for real-time use. Adds some CPU load but acceptable.
- **Convergence time**: AEC needs a few seconds of reference audio to converge on the echo path. Initial echo suppression may be partial.
- **Always-on processing**: Enabling processing by default may slightly increase CPU usage, but the processors are already optimized and the benefit of always having speech events outweighs the cost.

## Implementation Notes

AEC3 processing flow in `try_mix_and_send()`:
1. Wait for 960 samples (480 × 2 channels) in both buffers
2. Call `aec.process(&mic, Some(&system), false, &mut out)`
3. Mix processed mic with system audio at 0.5 gain each
4. Send to processing pipeline

## Migration Plan

No migration needed - this changes UI behavior but not persisted state.

## Open Questions

- Should AEC state persist across app restarts? (Deferred - can add settings later)
