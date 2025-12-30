# Change: Add CUDA GPU Acceleration for Windows

## Why

Windows voice transcription currently uses CPU-only processing via prebuilt whisper.cpp binaries. The whisper.cpp project provides CUDA-enabled Windows binaries (`whisper-cublas-*.zip`) that offer 5-10x faster transcription on NVIDIA GPUs. Adding build-time CUDA support enables Windows users with NVIDIA GPUs to achieve significantly faster real-time transcription.

## What Changes

- Add `cuda` Cargo feature flag for Windows (mirrors existing Linux CUDA feature)
- Modify `build.rs` to download CUDA-enabled whisper.cpp binaries when the `cuda` feature is enabled
- Bundle additional CUDA runtime DLLs (cublas, cudart, cublasLt) with the application when built with CUDA
- Update documentation to explain Windows CUDA build requirements and usage
- **No breaking changes**: CUDA is opt-in; default builds remain CPU-only

## Impact

- Affected specs: `speech-transcription`
- Affected code: `src-tauri/build.rs`, `src-tauri/Cargo.toml`
- Build requirements: When CUDA feature is enabled, no additional build-time dependencies are required (uses prebuilt CUDA binaries from whisper.cpp releases)
- Runtime requirements: When built with CUDA, requires NVIDIA GPU with compatible drivers; CUDA runtime is bundled with the application
