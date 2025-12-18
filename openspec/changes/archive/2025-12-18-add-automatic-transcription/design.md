## Context
The application currently has manual recording triggered by a Record button. Speech detection already exists and emits `speech-started` and `speech-ended` events with lookback audio. The goal is to leverage this existing infrastructure to automate the recording and transcription workflow.

Key stakeholders: End users who want hands-free transcription for meetings or continuous speech capture.

## Goals / Non-Goals

### Goals
- Provide a "Transcribe" toggle that enables automatic speech-triggered recording
- Queue speech segments for transcription without blocking new recordings
- Allow transcription to run independently and asynchronously from recording
- Maintain existing WAV file saving behavior
- Preserve lookback audio in transcribed segments (speech starts from true beginning)
- Never drop audio data - continuous capture with speech-based slicing

### Non-Goals
- Real-time streaming transcription (audio is still buffered per speech segment)
- Automatic cleanup of WAV files (handled separately)
- Changing the underlying speech detection algorithm
- Modifying the Whisper transcription engine

## Decisions

### Decision: Single "Transcribe" toggle replaces "Record" button
- **Rationale**: The user requested replacing Record with a Transcribe toggle. When active, the system handles recording automatically based on speech detection.
- **Alternative considered**: Keep both Record and Transcribe buttons for manual override capability. Rejected to keep UI simple per user request.

### Decision: Continuous recording with speech-based segment extraction
- **Rationale**: Audio capture must run continuously to avoid dropping samples between speech segments. Speech detection events mark boundaries for extracting segments from the continuous stream, but capture never pauses.
- **Alternative considered**: Start/stop recording on speech events. Rejected because stopping and restarting capture could drop audio samples at segment boundaries.

### Decision: Speech segment queue with async transcription workers
- **Rationale**: Transcription is CPU-intensive and slower than real-time speech. A queue allows multiple segments to accumulate while being processed sequentially. Recording continues uninterrupted.
- **Alternative considered**: Single blocking transcription per segment. Rejected because it would pause recording during transcription.

### Decision: Use existing `speech-started` and `speech-ended` events as segment markers
- **Rationale**: The speech detection system already emits events with lookback audio. These events define segment boundaries within the continuous audio stream - they do not control whether capture is running.
- **Alternative considered**: Add new event types for transcription mode. Rejected as unnecessary complexity.

### Decision: Ring buffer for continuous capture with segment extraction
- **Rationale**: A ring buffer continuously accumulates audio samples. When speech-ended fires, the relevant portion is extracted (copied) for transcription while the buffer continues accumulating new samples without interruption.
- **Alternative considered**: Linear buffer that gets cleared. Rejected because clearing could cause race conditions with incoming samples.

### Decision: Monitor mode must be active for Transcribe mode
- **Rationale**: Speech detection requires audio monitoring. When Transcribe is enabled, monitoring is implicitly enabled. Disabling monitoring disables Transcribe.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Frontend (TypeScript)                     │
├─────────────────────────────────────────────────────────────────┤
│  Transcribe Toggle ──▶ start_transcribe_mode()                  │
│                        stop_transcribe_mode()                   │
│                                                                  │
│  Event Listeners:                                                │
│    - transcription-complete ──▶ appendTranscription()           │
│    - recording-saved ──▶ show notification                      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Backend (Rust)                            │
├─────────────────────────────────────────────────────────────────┤
│  TranscribeMode State                                            │
│    - is_transcribe_active: bool                                 │
│    - in_speech_segment: bool (between speech-started/ended)     │
│    - segment_start_index: usize (where current segment began)   │
│                                                                  │
│  Continuous Audio Capture                                        │
│    - Audio backend streams samples continuously                 │
│    - Samples always flow to ring buffer (never paused)          │
│    - Speech detector processes every sample                     │
│                                                                  │
│  Segment Extraction (on speech-ended):                          │
│    - Copy samples from segment_start_index to current position  │
│    - Include lookback samples at segment start                  │
│    - Queue extracted segment for transcription                  │
│    - Save extracted segment to WAV file                         │
│                                                                  │
│  TranscriptionQueue                                              │
│    - Bounded queue of extracted audio segments                  │
│    - Worker thread processes queue sequentially                 │
│    - Emits transcription-complete for each result               │
└─────────────────────────────────────────────────────────────────┘
```

## Continuous Recording Design

### Ring Buffer Architecture

The core principle is that audio capture **never stops** while transcribe mode is active. This ensures no samples are dropped between speech segments.

```
Audio Stream (continuous)
    │
    ▼
┌───────────────────────────────────────────────────────────────┐
│                    Segment Ring Buffer                         │
│  ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐     │
│  │ ... │ ... │ SEG │ SEG │ SEG │ GAP │ GAP │ SEG │ ... │     │
│  └─────┴─────┴──▲──┴─────┴──▲──┴─────┴─────┴──▲──┴─────┘     │
│                 │           │                 │               │
│          segment_start   speech_end      write_pos           │
│                 │           │                                 │
│                 └───────────┘                                 │
│                  Extracted segment                            │
└───────────────────────────────────────────────────────────────┘
    │
    ▼ (on speech-ended)
┌─────────────────────┐
│  Extracted Segment  │ ──▶ Save to WAV
│  (copy of samples)  │ ──▶ Queue for transcription
└─────────────────────┘
```

### Sample Flow

```rust
struct TranscribeState {
    /// Ring buffer for continuous audio capture
    /// Sized to hold ~30 seconds of audio (enough for long utterances)
    ring_buffer: RingBuffer<f32>,
    
    /// Whether transcribe mode is active
    is_active: bool,
    
    /// Whether we're currently inside a speech segment
    in_speech: bool,
    
    /// Ring buffer index where current speech segment started
    /// Includes lookback samples from speech-started event
    segment_start_idx: usize,
    
    /// Sample rate for timing calculations
    sample_rate: u32,
}
```

### Timing and Synchronization

1. **Continuous Write**: Every audio callback writes samples to the ring buffer at `write_pos`, regardless of speech state.

2. **Speech Start Marker**: When `speech-started` fires:
   - Record `segment_start_idx = write_pos - lookback_samples`
   - Set `in_speech = true`
   - Lookback samples are already in the buffer (that's why we use a ring buffer)

3. **Speech End Extraction**: When `speech-ended` fires:
   - Calculate segment length: `write_pos - segment_start_idx`
   - Copy samples from `segment_start_idx` to `write_pos` into a new `Vec<f32>`
   - Set `in_speech = false`
   - Queue the extracted segment
   - **Do not clear or reset the ring buffer** - it continues accumulating

4. **Buffer Overflow Handling**: When writing new samples would overwrite `segment_start_idx` (i.e., the current speech segment is about to exceed buffer capacity):
   - Extract the current segment up to the current `write_pos`
   - Queue the partial segment for transcription (may split mid-word)
   - Save the partial segment to WAV file
   - Set new `segment_start_idx = write_pos` (continue the speech segment)
   - Remain in `in_speech = true` state
   - This ensures no audio is dropped, though transcription may have word boundary issues

5. **Buffer Wraparound Math**: Use modular arithmetic for all index calculations to handle the circular nature of the buffer correctly.

### Data Flow Timeline

```
Time ──────────────────────────────────────────────────────────▶

Audio:    ░░░░░░░▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░░░▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░
          │     │               │       │       │
          │     │               │       │       │
Events:   │  speech-started  speech-ended    speech-started
          │     │               │       │       │
          │     └───────────────┘       │       │
          │      Segment 1 extracted    │       │
          │      & queued               │       │
          │                             │       │
Buffer:   ├─────────────────────────────┼───────┼─────────────▶
          │     Always accumulating     │       │
          │     Never paused            │       │

Queue:    [Segment 1] ──▶ Transcribe ──▶ "Hello world"
                         (async)
```

### Key Invariants

1. **Audio callback always writes**: The audio processing callback writes to the ring buffer on every invocation, regardless of `in_speech` state.

2. **Speech events only mark boundaries**: `speech-started` and `speech-ended` events record indices and trigger extraction - they never pause or resume the audio stream.

3. **Extraction is a copy operation**: When extracting a segment, samples are copied out of the ring buffer. The original samples remain until overwritten by new audio.

4. **Transcription is decoupled**: The transcription queue and worker operate completely independently of the audio capture loop. Slow transcription does not affect capture.

5. **No audio is ever dropped**: If a speech segment would exceed buffer capacity, the segment is split and submitted in parts rather than losing any audio data.

### Ring Buffer Sizing

```
Buffer duration = 30 seconds
Sample rate = 48000 Hz
Channels = 2 (stereo, later converted to mono)
Samples needed = 48000 * 30 * 2 = 2,880,000 samples
Memory = 2,880,000 * 4 bytes = ~11.5 MB
```

This is acceptable for a desktop application and provides ample headroom for long utterances.

### Buffer Overflow Handling

When speech continues longer than the buffer can hold, we must prevent data loss. The strategy is to **split long segments** rather than drop audio.

```
Buffer Overflow Scenario:
─────────────────────────────────────────────────────────────────

Time ──────────────────────────────────────────────────────────▶

Speech:   ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
          │                              │                      │
          speech-started            overflow detected      speech-ended
          │                              │                      │
          └──────────────────────────────┘                      │
              Segment 1 (partial)                               │
              extracted & queued                                │
                                         │                      │
                                         └──────────────────────┘
                                             Segment 2 (partial)
                                             extracted & queued

Ring Buffer State at Overflow:
┌─────────────────────────────────────────────────────────────┐
│  write_pos about to overwrite segment_start_idx             │
│  ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐   │
│  │ NEW │ NEW │ OLD │ OLD │ OLD │ OLD │ OLD │ OLD │ OLD │   │
│  └──▲──┴─────┴──▲──┴─────┴─────┴─────┴─────┴─────┴─────┘   │
│     │           │                                           │
│  write_pos   segment_start_idx                              │
│     │           │                                           │
│     └───────────┴── Distance approaching buffer capacity    │
└─────────────────────────────────────────────────────────────┘

Action: Extract segment from segment_start_idx to write_pos,
        then set segment_start_idx = write_pos and continue.
```

#### Overflow Detection Algorithm

```rust
fn check_overflow_and_extract(&mut self) -> Option<Vec<f32>> {
    if !self.in_speech {
        return None;
    }
    
    // Calculate current segment length (handling wraparound)
    let segment_len = self.segment_length();
    
    // If segment is approaching buffer capacity (e.g., 90% full),
    // extract it now to avoid data loss
    let threshold = (self.ring_buffer.capacity() * 9) / 10;
    
    if segment_len >= threshold {
        // Extract current segment
        let segment = self.extract_segment();
        
        // Start new segment at current write position
        self.segment_start_idx = self.write_pos;
        
        // Remain in speech state - we're continuing the same utterance
        // in_speech stays true
        
        return Some(segment);
    }
    
    None
}
```

#### Overflow Handling on Each Audio Callback

```rust
fn on_audio_samples(&mut self, samples: &[f32]) {
    // 1. Check if we need to extract due to overflow BEFORE writing
    if let Some(partial_segment) = self.check_overflow_and_extract() {
        // Queue partial segment for transcription
        self.queue_segment(partial_segment, is_partial: true);
    }
    
    // 2. Write new samples to ring buffer (always happens)
    self.ring_buffer.write(samples);
    self.write_pos = (self.write_pos + samples.len()) % self.capacity;
}
```

#### Trade-offs of Segment Splitting

| Aspect | Impact |
|--------|--------|
| Audio continuity | Preserved - no samples dropped |
| Transcription accuracy | May have word boundary issues at split points |
| WAV files | Multiple files for one long utterance |
| User experience | Transcription appears incrementally (may be desirable) |

The 30-second buffer is sized to make overflow rare in typical use. Most natural speech includes pauses that trigger `speech-ended` before overflow occurs.

## Transcription Queue Design

```rust
struct TranscriptionQueue {
    queue: Arc<Mutex<VecDeque<QueuedSegment>>>,
    worker_active: Arc<AtomicBool>,
}

struct QueuedSegment {
    samples: Vec<f32>,       // Extracted audio (owned copy)
    sample_rate: u32,
    channels: u16,
    wav_path: Option<PathBuf>,  // Already saved WAV file path
}
```

- Queue is bounded (e.g., 10 segments max) to prevent unbounded memory growth
- Single worker thread processes segments FIFO
- Worker emits events for each transcription result
- Queue can be drained when transcribe mode is disabled

## Risks / Trade-offs

### Risk: Very long utterances exceed ring buffer capacity
- **Mitigation**: When a segment approaches buffer capacity, automatically extract and submit the current segment, then continue recording with a new segment. This may split words at segment boundaries, potentially causing transcription artifacts, but ensures no audio is dropped. The 30-second buffer size minimizes how often this occurs in practice.

### Risk: Transcription backlog during continuous speech
- **Mitigation**: Show queue length in UI, allow user to pause speaking if needed. Not implementing hard limits on queue for MVP.

### Risk: Memory usage from queued audio segments
- **Mitigation**: Bound queue to 10 segments. Each segment is typically a few seconds of speech (~48KB per second at 16kHz mono), so max ~5MB for long segments.

### Risk: Speech detection false positives trigger unnecessary recordings
- **Mitigation**: Existing speech detection has transient rejection and onset time requirements. Short false triggers produce empty transcriptions which display as "(No speech detected)".

### Risk: Race condition between segment extraction and buffer writes
- **Mitigation**: Extraction copies samples atomically while holding the buffer lock. The audio callback acquires the same lock when writing. Lock contention is minimal since extraction is fast (memcpy).

## Migration Plan
- No migration needed - this is additive functionality
- Existing Monitor and Record buttons behavior is removed (Record button replaced with Transcribe toggle)
- Users who want single-shot recording can use Transcribe mode and speak once

## Open Questions
- Should there be a minimum segment duration to filter very short triggers? (Decision: No, let Whisper handle it - it returns "(No speech detected)" for non-speech)
- Should transcribe mode auto-stop after idle timeout? (Decision: No, keep it simple - user controls when to stop)
