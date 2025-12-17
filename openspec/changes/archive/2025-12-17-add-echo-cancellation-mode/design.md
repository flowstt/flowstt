## Context
The current audio pipeline supports capturing from two sources simultaneously (microphone + system audio) in "Mixed" mode. When echo cancellation is enabled, AEC is applied to the microphone signal using system audio as a reference, then both streams are mixed together at 0.5 gain each.

Users recording voice while listening to music want the opposite: use AEC to remove the music from the mic, then output only the cleaned microphone audio without mixing in the system audio.

## Goals / Non-Goals
- **Goals**:
  - Allow users to record voice-only audio while system audio plays
  - Reuse existing AEC infrastructure (aec3 crate)
  - Minimal UI changes - add a simple mode toggle
  
- **Non-Goals**:
  - Multiple simultaneous recording outputs (e.g., separate files for each stream)
  - Advanced AEC tuning parameters exposed to users
  - Noise reduction beyond echo cancellation

## Decisions

### Recording Mode Toggle
- **Decision**: Add a "Recording Mode" dropdown/toggle with two options: "Mixed" and "Echo Cancel"
- **Rationale**: Clearer mental model than overloading the AEC toggle with conditional behavior. "Mixed" describes output containing both streams; "Echo Cancel" describes filtering out the secondary stream.
- **Alternatives considered**:
  - Overload AEC toggle to change behavior based on context - rejected as confusing
  - Three-way source type (Input/System/Mixed/EchoCancel) - rejected as EchoCancel is a mode, not a source

### AEC Toggle Interaction
- **Decision**: AEC toggle remains independent and controls whether echo cancellation processing is applied
- **Rationale**: In "Echo Cancel" mode with AEC disabled, users get raw primary-only output (useful for debugging or when AEC isn't needed). In "Mixed" mode, AEC toggle controls whether echo is removed before mixing.
- **Behavior matrix**:
  | Mode | AEC Toggle | Output |
  |------|------------|--------|
  | Mixed | Off | mic + system (0.5 gain each) |
  | Mixed | On | aec(mic, system) + system (0.5 gain each) |
  | Echo Cancel | Off | mic only |
  | Echo Cancel | On | aec(mic, system) only |

### Implementation Location
- **Decision**: Modify `AudioMixer::try_mix_and_send()` to check recording mode and conditionally skip mixing
- **Rationale**: Mixer already has access to both streams and AEC; adding mode check is minimal change
- **Data flow**: Add `recording_mode` field to `AudioMixer`, passed from backend command through shared state

## Risks / Trade-offs
- **Risk**: Users may expect "Echo Cancel" mode to work with single source - need clear UI feedback that two sources are required
  - **Mitigation**: UI grays out/disables "Echo Cancel" mode when only one source is selected
- **Risk**: AEC quality depends on system audio being accurately captured as reference
  - **Mitigation**: Document that best results come from capturing the same audio output device the user is listening to

## Migration Plan
- No migration needed - new feature, existing behavior unchanged
- Default to "Mixed" mode to preserve current behavior for existing users

## Open Questions
- None - scope is well-defined
