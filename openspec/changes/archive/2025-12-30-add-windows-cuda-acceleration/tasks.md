# Tasks

## 1. Build System Changes

- [x] 1.1 Update `build.rs` to detect `cuda` feature via `CARGO_FEATURE_CUDA` environment variable
- [x] 1.2 Add conditional binary selection for CUDA-enabled Windows builds
- [x] 1.3 Update library extraction to handle additional CUDA DLLs
- [x] 1.4 Ensure all CUDA DLLs are copied to the runtime directory

## 2. Documentation

- [x] 2.1 Update README with Windows CUDA build instructions
- [x] 2.2 Document runtime requirements (NVIDIA GPU, driver version)
- [x] 2.3 Document binary size differences (CPU vs CUDA)

## 3. Validation

- [x] 3.1 Test CPU-only build still works (no regression)
- [x] 3.2 Test CUDA build downloads correct binary
- [x] 3.3 Test CUDA build bundles all required DLLs
- [x] 3.4 Test transcription works with NVIDIA GPU
