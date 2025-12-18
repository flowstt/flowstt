# Change: Add Windows Audio Feature Parity

## Why

FlowSTT was originally developed on Linux using PipeWire for audio capture. The Windows port using WASAPI currently only supports basic single-source microphone capture, while Linux supports system audio capture, multi-source mixing, and echo cancellation. This gap prevents Windows users from accessing key features like transcribing system audio, recording both microphone and desktop audio simultaneously, and using echo cancellation.

## What Changes

- **ADDED** Windows system audio capture (loopback) using WASAPI's loopback mode
- **ADDED** Windows multi-source capture infrastructure with thread-per-stream architecture
- **ADDED** Windows audio mixer port from Linux implementation
- **ADDED** Echo cancellation integration for Windows using the cross-platform `aec3` crate
- **MODIFIED** Windows Audio Backend (Stub) requirement to become Windows Audio Backend (Full) with complete feature support
- **MODIFIED** System Audio Device Enumeration requirement to include Windows support

## Impact

- Affected specs: `audio-recording`
- Affected code:
  - `src-tauri/src/platform/windows/wasapi.rs` (primary implementation file)
  - `src-tauri/Cargo.toml` (dependency changes for cross-platform `aec3`)
- Dependencies: `aec3` crate must be made available for Windows target
- Breaking changes: None (additive changes only)
