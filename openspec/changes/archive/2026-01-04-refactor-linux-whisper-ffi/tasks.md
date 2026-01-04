## 1. Build System Changes

- [x] 1.1 Update build.rs to download whisper.cpp source tarball for Linux
- [x] 1.2 Add CMake configuration and build logic in build.rs for Linux
- [x] 1.3 Implement source caching based on version hash
- [x] 1.4 Add CUDA feature detection and CMake flag (-DGGML_CUDA=ON)
- [x] 1.5 Copy libwhisper.so to runtime directory
- [x] 1.6 Add error handling for missing CMake
- [x] 1.7 Add error handling for missing CUDA toolkit when cuda feature is enabled

## 2. Cargo.toml Updates

- [x] 2.1 Remove whisper-rs dependency for Linux
- [x] 2.2 Add libloading dependency for Linux (unify with Windows/macOS)
- [x] 2.3 Update cuda feature to not depend on whisper-rs/cuda
- [x] 2.4 ~~Add cmake crate as build-dependency for CMake integration~~ (Not needed - using Command to invoke cmake directly)

## 3. FFI Module Updates

- [x] 3.1 Remove `#[cfg(not(target_os = "linux"))]` guards from whisper_ffi.rs
- [x] 3.2 Add "libwhisper.so" to library name search for Linux
- [x] 3.3 Ensure whisper_ffi.rs is compiled on all platforms

## 4. Transcription Module Updates

- [x] 4.1 Remove Linux-specific platform module using whisper-rs in transcribe.rs
- [x] 4.2 Update transcribe.rs to use FFI implementation on all platforms
- [x] 4.3 Remove conditional compilation for platform-specific TranscriberImpl

## 5. Testing and Validation

- [x] 5.1 Test Linux CPU-only build
- [ ] 5.2 Test Linux CUDA build (with CUDA toolkit installed)
- [ ] 5.3 Test build error messages when CMake is missing
- [ ] 5.4 Test build error messages when CUDA toolkit is missing (cuda feature enabled)
- [ ] 5.5 Verify transcription works correctly on Linux with the new FFI approach
- [ ] 5.6 Test Windows and macOS builds still work (regression testing)

## 6. Documentation

- [ ] 6.1 Update README with new Linux build requirements (CMake)
- [ ] 6.2 Document CUDA build requirements for Linux
