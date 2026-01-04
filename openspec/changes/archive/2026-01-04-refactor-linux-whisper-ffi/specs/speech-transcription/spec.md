## MODIFIED Requirements

### Requirement: Local Whisper Transcription
The system SHALL transcribe recorded audio to text using a local Whisper model. On all platforms (Windows, macOS, and Linux), transcription uses the whisper.cpp shared library loaded via FFI. In transcribe mode, segments are processed asynchronously from a queue.

#### Scenario: Successful transcription
- **WHEN** recording stops and audio data is available
- **THEN** the audio is transcribed and the resulting text is displayed in the UI

#### Scenario: Transcription in progress
- **WHEN** transcription is processing
- **THEN** the UI displays a loading indicator

#### Scenario: Windows library loading
- **WHEN** transcription is requested on Windows
- **THEN** the whisper.cpp shared library (whisper.dll) is loaded from the application bundle

#### Scenario: macOS library loading
- **WHEN** transcription is requested on macOS
- **THEN** the whisper.cpp shared library (libwhisper.dylib) is loaded from the application bundle

#### Scenario: Linux library loading
- **WHEN** transcription is requested on Linux
- **THEN** the whisper.cpp shared library (libwhisper.so) is loaded from the application bundle

#### Scenario: Queue-based transcription in transcribe mode
- **WHEN** transcribe mode is active and a speech segment is queued
- **THEN** the transcription worker processes the segment from the queue and emits the result

### Requirement: Whisper Library Bundling
The system SHALL bundle the whisper.cpp shared library with the application on all platforms. On Windows and macOS, the library SHALL be downloaded from the official whisper.cpp GitHub releases during the build process. On Linux, the library SHALL be built from source during the build process using CMake.

#### Scenario: Build downloads library (Windows/macOS)
- **WHEN** the application is built on Windows or macOS
- **THEN** the build process downloads the appropriate whisper.cpp binary from GitHub releases if not already cached

#### Scenario: Build compiles library (Linux)
- **WHEN** the application is built on Linux
- **THEN** the build process downloads the whisper.cpp source and compiles libwhisper.so using CMake

#### Scenario: Library bundled with application
- **WHEN** the application is packaged for distribution
- **THEN** the whisper.dll (Windows), libwhisper.dylib (macOS), or libwhisper.so (Linux) is included in the application bundle

#### Scenario: Cached source reused (Linux)
- **WHEN** building on Linux and the whisper.cpp source for the target version already exists in the build cache
- **THEN** the cached source is used without re-downloading

#### Scenario: Cached binary reused (Windows/macOS)
- **WHEN** building on Windows or macOS and the whisper.cpp binary for the target version already exists in the build cache
- **THEN** the cached binary is used without re-downloading

#### Scenario: Download failure handling
- **WHEN** the build process cannot download the whisper.cpp source or binary (network error, GitHub unavailable)
- **THEN** the build fails with a clear error message indicating the download failure

#### Scenario: CMake not installed (Linux)
- **WHEN** building on Linux and CMake is not installed
- **THEN** the build fails with a clear error message directing the user to install CMake

### Requirement: Platform-Specific Binary Selection
The build system SHALL select the correct whisper.cpp binary or build configuration based on the target platform and architecture.

#### Scenario: Windows x64 build
- **WHEN** building for Windows x64
- **THEN** the `whisper-bin-x64.zip` binary is downloaded and whisper.dll is extracted

#### Scenario: Windows x86 build
- **WHEN** building for Windows x86
- **THEN** the `whisper-bin-Win32.zip` binary is downloaded and whisper.dll is extracted

#### Scenario: macOS build
- **WHEN** building for macOS
- **THEN** the `whisper-v{version}-xcframework.zip` is downloaded and the correct architecture dylib is extracted

#### Scenario: Linux x64 build
- **WHEN** building for Linux x86_64
- **THEN** the build system compiles whisper.cpp from source with CMake and produces libwhisper.so

### Requirement: CUDA GPU Acceleration (Linux)
On Linux, the system SHALL support optional CUDA GPU acceleration for voice transcription when built with the `cuda` feature flag. When enabled, the build system configures CMake to enable CUDA support, and transcription uses NVIDIA GPU hardware for faster inference.

#### Scenario: CUDA-enabled build on Linux
- **WHEN** the application is built on Linux with `--features cuda`
- **THEN** the build system configures CMake with `-DGGML_CUDA=ON`
- **AND** transcription uses the NVIDIA GPU when a compatible GPU and drivers are present

#### Scenario: Default CPU-only build on Linux
- **WHEN** the application is built on Linux without the `cuda` feature flag
- **THEN** the build system compiles whisper.cpp without CUDA support (CPU-only processing)

#### Scenario: CUDA feature on macOS
- **WHEN** the `cuda` feature flag is specified on macOS builds
- **THEN** the feature has no effect (macOS uses Metal acceleration via the prebuilt framework)

#### Scenario: CUDA build without GPU at runtime
- **WHEN** the application is built with CUDA support on Linux
- **AND** no compatible NVIDIA GPU is available at runtime
- **THEN** transcription falls back to CPU processing

#### Scenario: CUDA toolkit not installed
- **WHEN** building on Linux with `--features cuda`
- **AND** the NVIDIA CUDA Toolkit is not installed
- **THEN** the build fails with a clear error message directing the user to install the CUDA Toolkit
