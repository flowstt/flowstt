## Context
The application currently streams audio samples during monitoring for waveform visualization. We need to add the ability to process these samples for analysis (starting with silence detection) while keeping the system extensible for future processors.

## Goals / Non-Goals
- Goals:
  - Add UI toggle for voice processing (independent of monitor toggle)
  - Create extensible processor architecture using Rust traits
  - Implement silence detection that logs to console
  - Processing only active when both toggles are enabled
- Non-Goals:
  - Frontend display of processing results (future work)
  - Multiple concurrent processors (future work)
  - Processor configuration UI (future work)

## Decisions
- **Trait-based processor pattern**: Use a `AudioProcessor` trait that processors implement. This allows easy addition of new processor types without modifying the core audio pipeline.
- **Processing in audio callback**: Run processors in the existing `process_audio_samples` function to avoid additional threading complexity. Processors must be fast and non-blocking.
- **Silence detection threshold**: Use RMS (root mean square) amplitude with a configurable threshold. Default to -40dB as silence threshold.
- **State toggle approach**: Voice processing toggle is independent - it can be enabled/disabled at any time, but only processes when monitoring is active.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Frontend (TypeScript)                │
├─────────────────────────────────────────────────────────┤
│  [Monitor Toggle]  [Voice Processing Toggle]            │
│         │                    │                          │
│         ▼                    ▼                          │
│   start_monitor()    set_processing_enabled()           │
└─────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────┐
│                    Backend (Rust)                       │
├─────────────────────────────────────────────────────────┤
│  AudioStreamState                                       │
│  ├── is_monitoring: bool                                │
│  ├── is_processing_enabled: bool                        │
│  └── processor: Option<Box<dyn AudioProcessor>>         │
│                                                         │
│  process_audio_samples()                                │
│  └── if is_monitoring && is_processing_enabled:         │
│      └── processor.process(samples)                     │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│                 AudioProcessor Trait                    │
├─────────────────────────────────────────────────────────┤
│  trait AudioProcessor: Send {                           │
│      fn process(&mut self, samples: &[f32]);            │
│      fn name(&self) -> &str;                            │
│  }                                                      │
│                                                         │
│  SilenceDetector implements AudioProcessor              │
│  ├── threshold_db: f32 (default -40.0)                  │
│  ├── is_silent: bool                                    │
│  └── logs state transitions to console                  │
└─────────────────────────────────────────────────────────┘
```

## Risks / Trade-offs
- **Processing in audio callback**: Keeps latency low but processors must complete quickly. Mitigation: Document that processors should be O(n) on sample count and avoid allocations.
- **Single processor for now**: Simplifies implementation. Multiple processors can be added later with a Vec<Box<dyn AudioProcessor>>.

## Open Questions
- None currently
