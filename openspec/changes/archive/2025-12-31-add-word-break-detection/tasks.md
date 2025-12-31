## 1. Backend: Word Break Detection

- [x] 1.1 Add word break state tracking fields to `SpeechDetector` struct
  - `word_break_threshold_ratio: f32` (0.5 = 50% of recent average)
  - `min_word_break_samples: u32` (15ms worth of samples)
  - `max_word_break_samples: u32` (200ms worth of samples)
  - `recent_speech_window_samples: u32` (100ms worth of samples)
  - `recent_speech_amplitude_sum: f32` (running sum for average)
  - `recent_speech_amplitude_count: u32` (sample count for average)
  - `in_word_break: bool` (currently in a gap)
  - `word_break_sample_count: u32` (how long current gap has lasted)
  - `word_break_start_speech_samples: u64` (for offset calculation)
  - `last_is_word_break: bool` (for metrics)

- [x] 1.2 Add `WordBreakPayload` struct for events
  - `offset_ms: u32`
  - `gap_duration_ms: u32`

- [x] 1.3 Add `is_word_break: bool` field to `SpeechMetrics` struct

- [x] 1.4 Implement word break detection logic in `SpeechDetector::process()`
  - Track running average of speech amplitude during confirmed speech
  - Detect when amplitude drops below threshold
  - Track gap duration and validate against min/max bounds
  - Emit `word-break` event at gap midpoint (when gap ends)
  - Set `last_is_word_break` for metrics emission

- [x] 1.5 Update `SpeechDetector::get_metrics()` to include `is_word_break`

## 2. Frontend: Word Break Visualization

- [x] 2.1 Update `SpeechMetrics` interface in `renderers.ts`
  - Add `is_word_break: boolean` field

- [x] 2.2 Add word break buffer to `SpeechActivityRenderer`
  - Add `wordBreakBuffer: Uint8Array` ring buffer
  - Add `isWordBreak` field to `BufferedMetric` interface
  - Update `pushMetrics()` to capture word break state
  - Update `transferToRingBuffer()` to transfer word break state
  - Update `clear()` to reset word break buffer

- [x] 2.3 Implement word break bar rendering in `SpeechActivityRenderer.draw()`
  - Add `drawWordBreakBars()` method
  - Get word break samples in chronological order
  - Draw vertical bars at positions where `is_word_break` is true
  - Use semi-transparent white color (rgba(255, 255, 255, 0.7))
  - Draw bars spanning the speech bar height region
  - Call after `drawSpeechBar()` so bars overlay speech regions

## 3. Validation

- [x] 3.1 Build and test backend changes compile without errors
- [x] 3.2 Verify word-break events are emitted in console during speech with pauses
- [x] 3.3 Verify word break bars appear on speech activity graph during natural speech
- [x] 3.4 Verify no word break markers appear during silence or single sustained sounds
