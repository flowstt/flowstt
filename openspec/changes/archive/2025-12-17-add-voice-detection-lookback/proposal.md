# Change: Add Voice Detection Lookback

## Why
Speech analysis requires several milliseconds (80-150ms onset time) to confirm voice activity, which often results in the first word being partially or entirely missed. By the time speech is confirmed, the actual beginning of the utterance has already passed through the system without being captured for transcription.

## What Changes
- Add a ring buffer to retain recent audio samples before voice detection triggers
- Implement lookback analysis to determine the true speech start point retroactively
- Include lookback samples in the `speech-started` event so transcription captures complete utterances
- Add configurable lookback duration (default ~200ms, matching onset + margin)
- Delay the speech activity graph by 200ms to show lookback results at correct temporal position
- Visualize lookback-determined speech with a distinct color from confirmed speech

## Impact
- Affected specs: audio-processing, audio-visualization
- Affected code: 
  - `src-tauri/src/processor.rs` (SpeechDetector ring buffer, lookback analysis, metrics)
  - `src/main.ts` (SpeechActivityRenderer delay buffer, lookback visualization)
- Memory impact: Ring buffer adds ~38KB, delay buffer adds ~20 metrics × 200ms
- Visualization: Speech activity graph will be 200ms behind waveform/spectrogram (intentional)
- Latency: No impact on real-time detection or waveform/spectrogram display
