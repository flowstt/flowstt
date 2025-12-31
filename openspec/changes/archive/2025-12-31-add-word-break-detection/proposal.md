# Change: Add Word Break Detection

## Why
Users can visually observe brief energy dips between words on the speech activity graph, but the system doesn't programmatically detect these boundaries. Detecting word breaks enables future features like improved transcription segmentation and provides immediate visual feedback showing word-level speech structure.

## What Changes
- Add word break detection algorithm to the speech detector that identifies brief energy dips within confirmed speech regions
- Emit `word-break` events when word boundaries are detected (for future use)
- Include word break state in speech metrics sent to frontend
- Display vertical bar markers on the speech activity graph at detected word break positions

## Impact
- Affected specs: `audio-processing`, `audio-visualization`
- Affected code: `src-tauri/src/processor.rs` (SpeechDetector, SpeechMetrics), `src/renderers.ts` (SpeechActivityRenderer)
