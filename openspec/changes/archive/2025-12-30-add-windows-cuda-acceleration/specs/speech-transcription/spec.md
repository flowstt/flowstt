## ADDED Requirements

### Requirement: CUDA GPU Acceleration (Windows)
On Windows, the system SHALL support optional CUDA GPU acceleration for voice transcription when built with the `cuda` feature flag. When enabled, the build system downloads CUDA-enabled whisper.cpp binaries and bundles the necessary CUDA runtime DLLs with the application.

#### Scenario: CUDA-enabled build on Windows
- **WHEN** the application is built on Windows with `--features cuda`
- **THEN** the build system downloads the CUDA-enabled whisper.cpp binary (`whisper-cublas-12.4.0-bin-x64.zip`)
- **AND** all required CUDA runtime DLLs are bundled with the application
- **AND** transcription uses the NVIDIA GPU when a compatible GPU and drivers are present

#### Scenario: Default CPU-only build on Windows
- **WHEN** the application is built on Windows without the `cuda` feature flag
- **THEN** the build system downloads the CPU-only whisper.cpp binary (existing behavior)

#### Scenario: CUDA runtime bundled with application
- **WHEN** the application is built with CUDA support on Windows
- **THEN** the CUDA runtime DLLs (cublas64_12.dll, cublasLt64_12.dll, cudart64_12.dll) are included in the application bundle
- **AND** end users do not need to install the CUDA Toolkit

#### Scenario: CUDA build without GPU at runtime
- **WHEN** the application is built with CUDA support on Windows
- **AND** no compatible NVIDIA GPU is available at runtime
- **THEN** transcription fails with an error indicating GPU is unavailable

## MODIFIED Requirements

### Requirement: CUDA GPU Acceleration (Linux)
On Linux, the system SHALL support optional CUDA GPU acceleration for voice transcription when built with the `cuda` feature flag. When enabled, transcription uses NVIDIA GPU hardware for faster inference. On Windows, the same `cuda` feature flag enables CUDA support via prebuilt binaries.

#### Scenario: CUDA-enabled build on Linux
- **WHEN** the application is built on Linux with `--features cuda`
- **THEN** the whisper-rs crate is compiled with CUDA support
- **AND** transcription uses the NVIDIA GPU when a compatible GPU and drivers are present

#### Scenario: Default CPU-only build on Linux
- **WHEN** the application is built on Linux without the `cuda` feature flag
- **THEN** transcription uses CPU-only processing (existing behavior)

#### Scenario: CUDA feature on macOS
- **WHEN** the `cuda` feature flag is specified on macOS builds
- **THEN** the feature has no effect (macOS uses Metal acceleration via the prebuilt framework)

#### Scenario: CUDA build without GPU at runtime
- **WHEN** the application is built with CUDA support
- **AND** no compatible NVIDIA GPU is available at runtime
- **THEN** transcription falls back to CPU processing (Linux) or fails with an error (Windows)
