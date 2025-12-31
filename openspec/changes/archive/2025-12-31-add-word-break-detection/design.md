# Design: Word Break Detection

## Context
The speech activity graph already displays amplitude, ZCR, and spectral centroid metrics during speech. Brief energy dips between words are visible but not programmatically detected. This feature adds detection of these intra-speech gaps to identify word boundaries.

## Goals
- Detect brief energy dips within continuous speech that indicate word boundaries
- Emit events for future transcription/segmentation use
- Visualize detected word breaks on the speech activity graph

## Non-Goals
- Actual word segmentation or alignment with transcription text
- Detecting syllable boundaries or phoneme transitions
- Affecting current speech start/end detection behavior

## Decisions

### Detection Algorithm
**Decision:** Use amplitude-based detection with short-term energy analysis.

Word breaks are characterized by:
1. Brief duration (typically 15-150ms for fast speakers, up to 200ms for slower speech)
2. Significant amplitude drop relative to surrounding speech
3. Occur only within confirmed speech regions (not during silence or onset)

Algorithm approach:
- Track a short-term running average of amplitude during speech
- Detect when amplitude drops below a threshold relative to the recent average
- Require minimum duration to filter consonant stops (15ms minimum gap)
- Require maximum duration to avoid detecting speech end (200ms maximum)
- Emit word-break event at the midpoint of the detected gap

**Alternatives considered:**
- ZCR-based detection: Rejected because ZCR varies significantly between voiced sounds and doesn't reliably indicate word gaps
- Spectral flux: More complex, better suited for onset detection than gap detection
- ML-based: Overkill for this use case, adds dependency

### Threshold Configuration
**Decision:** Use relative thresholds based on recent speech energy.

- Word break threshold: amplitude drops below 50% of recent average
- Minimum gap duration: 15ms (accommodates fast speakers while filtering most consonant stops)
- Maximum gap duration: 200ms (longer gaps are speech pauses, not word breaks)
- Recent average window: 100ms of confirmed speech

Note: Fast speakers (podcasters, news reporters) can have word gaps as short as 20-30ms. The 15ms minimum accommodates this while still filtering most plosive consonant stops (typically 5-15ms). The 50% threshold (rather than 60%) requires a more significant energy drop, which helps reduce false positives from voiced consonants that don't fully close the vocal tract.

### Event Structure
**Decision:** Emit lightweight events with position information.

```rust
struct WordBreakPayload {
    /// Timestamp offset in milliseconds from speech start
    offset_ms: u32,
    /// Duration of the detected gap in milliseconds
    gap_duration_ms: u32,
}
```

### Visualization
**Decision:** Vertical bars overlaying the speech state bar region.

- Draw vertical lines at word break positions within the speech bar area
- Use a distinct color (white or light gray with transparency) to contrast with green/blue speech bars
- Bars span the full height of the speech state bar region
- Width: 1-2 pixels

## Risks / Trade-offs

- **False positives from consonants:** Some consonants (stops like 'p', 't', 'k') have brief energy dips (5-15ms). Mitigation: 15ms minimum duration threshold filters most of these while still catching fast-speaker word breaks.
- **Sensitivity to speaking style:** Fast speech has shorter gaps (20-30ms), slow speech has longer ones (100-200ms). Mitigation: Relative thresholds adapt to speaking patterns; wide duration range (15-200ms) accommodates both styles.
- **Latency:** Detection requires seeing the full gap, adding ~15-30ms latency to word break events. Acceptable for visualization; may need adjustment for real-time transcription use.

## Open Questions
- Should word break sensitivity be user-configurable? (Defer to future enhancement if needed)
