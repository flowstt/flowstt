# Change: Migrate Linux Whisper Integration to FFI

## Why

The current Linux implementation uses whisper-rs, which requires building whisper.cpp from source during compilation. This introduces significant build complexity:
- Long build times (whisper.cpp compilation)
- C++ toolchain dependencies (cmake, clang/gcc)
- CUDA toolkit must be installed on the build machine for GPU acceleration
- Different code paths between Linux (whisper-rs) and Windows/macOS (FFI with prebuilt binaries)

Migrating Linux to use the same FFI approach as Windows/macOS simplifies the build process, reduces dependencies, and unifies the codebase.

## What Changes

- **BREAKING**: Remove whisper-rs dependency entirely
- Add Linux support to the build.rs binary download process
- Download prebuilt whisper.cpp Linux binaries from GitHub releases
- Extend whisper_ffi.rs to be used on all platforms (Linux, Windows, macOS)
- Support CUDA via prebuilt CUDA-enabled Linux binaries (optional feature)

## Impact

- Affected specs: `speech-transcription`
- Affected code:
  - `src-tauri/Cargo.toml` - Remove whisper-rs, add libloading for Linux
  - `src-tauri/build.rs` - Add Linux binary download logic
  - `src-tauri/src/transcribe.rs` - Unify platform implementations
  - `src-tauri/src/whisper_ffi.rs` - Remove platform guards, add Linux library name
