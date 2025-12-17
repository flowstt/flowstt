# Design: Voice Detection Lookback

## Context
The current speech detector (`SpeechDetector` in `processor.rs`) uses onset time accumulation (80-150ms) to confirm speech activity. This prevents false triggers but means the actual start of speech has already passed by the time `speech-started` is emitted. For transcription, this results in partial or missing first words.

The audio pipeline processes samples in real-time through `process_samples()` in `audio.rs`, which feeds mono audio to the `SpeechDetector`. Recording samples are accumulated separately in `AudioStreamState.recording_samples`.

## Goals / Non-Goals
**Goals:**
- Capture the true start of speech by retaining recent audio in a ring buffer
- Analyze buffered audio retroactively when speech is confirmed
- Provide lookback audio data to downstream consumers (transcription)
- Visualize lookback speech detection distinctly from confirmed speech detection
- Maintain real-time performance in the audio callback

**Non-Goals:**
- Changing the fundamental speech detection algorithm
- Adding new event types (will enhance existing `speech-started` event)
- Delaying waveform or spectrogram visualizations (they remain real-time)

## Decisions

### Decision 1: Ring Buffer in SpeechDetector
Store a rolling window of recent audio samples within the `SpeechDetector` struct.

**Rationale:** 
- Keeps lookback logic co-located with detection logic
- Single responsibility: detector manages both detection and speech boundary determination
- No changes needed to the audio pipeline architecture

**Alternatives considered:**
- Buffer in `AudioStreamState`: Would require coordination between recorder and detector, more complex
- Separate lookback processor: Over-engineered for this use case

### Decision 2: Lookback Duration = 200ms
Default lookback buffer holds 200ms of audio (9,600 samples at 48kHz).

**Rationale:**
- Covers the maximum onset time (150ms for whisper) plus 50ms margin
- Speech typically starts with plosives/fricatives that have clear amplitude signatures within this window
- Memory overhead is minimal (~38KB for f32 samples)

### Decision 3: Energy-Based Lookback Analysis
When speech is confirmed, scan backward through the ring buffer to find where amplitude first exceeded a lower threshold (e.g., -55dB, below normal detection threshold).

**Rationale:**
- Simple and computationally cheap
- Catches the initial burst of speech energy
- Lower threshold than normal detection avoids missing soft speech starts
- More sophisticated analysis (feature-based) is overkill for finding the approximate start

**Alternatives considered:**
- ZCR + centroid lookback: More accurate but computationally expensive, diminishing returns
- Fixed offset from confirmation: Simpler but wastes samples or misses starts

### Decision 4: Emit Lookback Audio with Event
Extend `SpeechEventPayload` to include lookback audio samples when speech starts.

**Rationale:**
- Transcription module receives complete audio from the true start
- Backward compatible: `lookback_samples` can be optional (None for speech-ended)

### Decision 5: Delayed Speech Activity Visualization
The speech activity graph is delayed by the max lookback duration (200ms) so that lookback analysis results can be displayed at the correct temporal position.

**Rationale:**
- When speech is confirmed, we know where it actually started (via lookback)
- By delaying the speech activity graph, we can show the lookback-determined start at the correct position
- Waveform and spectrogram remain real-time - the temporal offset is intentional and informative
- Users can see exactly how far back the lookback determined speech actually began

**Implementation:**
- `SpeechActivityRenderer` buffers incoming metrics for 200ms before rendering
- When lookback start is determined, emit a separate metric indicating "lookback speech" state
- Use distinct colors: lookback region vs confirmed speech region

### Decision 6: Two-State Speech Visualization
Display lookback-determined speech and confirmed speech as distinct visual states in the speech activity graph.

**Rationale:**
- Shows the difference between when speech actually started vs when it was confirmed
- Provides insight into detection algorithm behavior
- Helps users understand the lookback feature

**Visual Design:**
- **Lookback speech region**: Different color (e.g., lighter/different hue) indicating "speech started here retroactively"
- **Confirmed speech region**: Current color indicating "speech confirmed at this point"
- Both regions visible in the delayed speech activity graph

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Ring buffer adds memory | 38KB is negligible; configurable if needed |
| Lookback analysis is imprecise | Acceptable for transcription; starting a few ms early is better than missing words |
| Audio callback slows down | Lookback scan only runs on speech confirmation (~1/second), not every callback |
| Speech graph out of sync with waveform | Intentional - the delay shows where lookback placed the true start |
| User confusion about delayed graph | Label the graph or document the intentional delay |

## Open Questions
1. Should lookback audio be included in the event payload, or should transcription pull from the recording buffer?
   - **Proposed answer:** Include in payload for simplicity; transcription already receives audio via events
2. Should lookback duration be user-configurable?
   - **Proposed answer:** Not initially; 200ms is a reasonable default. Can add configuration later if needed.
3. What colors for lookback vs confirmed speech?
   - **Proposed answer:** Lookback region in a lighter/different shade, confirmed in current green. Exact colors TBD during implementation.
