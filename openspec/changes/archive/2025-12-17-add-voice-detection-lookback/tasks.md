# Tasks: Add Voice Detection Lookback

## 1. Ring Buffer Implementation
- [x] 1.1 Add ring buffer fields to `SpeechDetector` struct (buffer vec, write index, capacity)
- [x] 1.2 Initialize ring buffer with 200ms capacity based on sample rate in `with_defaults()`
- [x] 1.3 Add sample accumulation to `process()` method, implementing circular write
- [x] 1.4 Add helper method `get_buffer_contents()` to extract samples in chronological order

## 2. Lookback Analysis
- [x] 2.1 Add lookback threshold constant (-55dB) to `SpeechDetector`
- [x] 2.2 Implement `find_lookback_start()` method that scans buffer backward for amplitude threshold crossing
- [x] 2.3 Integrate lookback analysis into speech confirmation path (when `is_speaking` becomes true)
- [x] 2.4 Calculate and store lookback offset in milliseconds

## 3. Event Payload Enhancement
- [x] 3.1 Add `lookback_samples: Option<Vec<f32>>` field to `SpeechEventPayload`
- [x] 3.2 Add `lookback_offset_ms: Option<u32>` field to `SpeechEventPayload`
- [x] 3.3 Modify `speech-started` emission to include lookback samples and offset
- [x] 3.4 Ensure `speech-ended` event continues to have `None` for lookback fields

## 4. Speech Metrics Enhancement
- [x] 4.1 Add `is_lookback_speech: bool` field to `SpeechMetrics`
- [x] 4.2 Add `lookback_offset_ms: Option<u32>` field to `SpeechMetrics`
- [x] 4.3 Emit lookback state in metrics when speech is confirmed with lookback

## 5. Frontend Delay Buffer
- [x] 5.1 Add delay buffer to `SpeechActivityRenderer` (200ms worth of metrics)
- [x] 5.2 Buffer incoming metrics before rendering instead of rendering immediately
- [x] 5.3 Implement retroactive insertion of lookback speech state into buffer
- [x] 5.4 Render from delay buffer maintaining correct temporal ordering

## 6. Lookback Visualization
- [x] 6.1 Define distinct color for lookback speech region (lighter/different hue from confirmed)
- [x] 6.2 Modify speech state bar rendering to use lookback color when `is_lookback_speech` is true
- [x] 6.3 Ensure both lookback and confirmed regions render correctly as graph scrolls

## 7. Testing and Validation
- [x] 7.1 Test ring buffer wrapping behavior with samples exceeding capacity
- [x] 7.2 Verify lookback finds earlier start point with synthetic test audio
- [x] 7.3 Test edge case: speech detected immediately (minimal lookback needed)
- [x] 7.4 Verify delay buffer correctly holds and releases metrics after 200ms
- [x] 7.5 Verify lookback region appears in correct temporal position in delayed graph
- [x] 7.6 Build and run application to verify no regressions in speech detection
- [x] 7.7 Verify event payload is correctly serialized and received by frontend
