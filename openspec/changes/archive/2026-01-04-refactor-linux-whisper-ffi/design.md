# Design: Linux Whisper FFI Migration

## Context

The current FlowSTT implementation uses two different approaches for Whisper integration:
- **Windows/macOS**: Prebuilt whisper.cpp binaries downloaded at build time, loaded via FFI (libloading)
- **Linux**: whisper-rs crate, which compiles whisper.cpp from source at build time

This asymmetry adds complexity:
- Different code paths in `transcribe.rs` with `#[cfg(target_os = "linux")]` conditionals
- Linux builds are significantly slower due to whisper.cpp compilation
- CUDA support on Linux requires the full CUDA Toolkit installed at build time
- Maintaining two separate integration approaches

## Constraints

**Critical**: The official whisper.cpp GitHub releases do not include prebuilt Linux binaries. Available release assets are:
- `whisper-bin-x64.zip` (Windows CPU)
- `whisper-bin-Win32.zip` (Windows CPU x86)
- `whisper-cublas-*.zip` (Windows CUDA)
- `whisper-v*-xcframework.zip` (macOS/iOS)

This means we must either:
1. Build Linux binaries ourselves and host them
2. Build Linux binaries as part of a CI/CD pipeline
3. Continue building from source but via a different mechanism

## Goals

- Unify the codebase by using FFI on all platforms
- Eliminate whisper-rs dependency entirely
- Reduce Linux build complexity and time
- Maintain CUDA support as an optional feature
- Ensure the solution is maintainable long-term

## Non-Goals

- Hosting our own binary distribution infrastructure (too complex to maintain)
- Supporting architectures beyond x86_64 initially
- Supporting non-CUDA GPU acceleration (e.g., ROCm, Vulkan)

## Decision: Build Whisper.cpp from Source with CMake

**What**: Replace whisper-rs with a build.rs script that compiles whisper.cpp from source using CMake, producing a shared library that is then loaded via FFI at runtime.

**Why**:
- No external binary hosting required - uses official whisper.cpp source
- CMake is the official build system for whisper.cpp
- Build complexity is contained in build.rs, not exposed to end users
- Same FFI code path as Windows/macOS
- Full control over build flags (CUDA, optimization, etc.)

**Trade-offs**:
- Build time remains longer than downloading prebuilt binaries
- Requires CMake and a C/C++ compiler at build time
- However, this is simpler than the current whisper-rs approach which also requires these plus Rust bindgen

**Alternatives considered**:

1. **Host our own prebuilt binaries**: Rejected - requires infrastructure, security considerations, and ongoing maintenance for each whisper.cpp release.

2. **Use whisper-rs but unify the Rust API**: Rejected - doesn't address the core issue of build complexity and slow builds.

3. **Contribute Linux builds to upstream**: Rejected - depends on upstream acceptance, timeline uncertainty.

## Decision: CMake Integration in build.rs

**What**: The build.rs script will:
1. Download whisper.cpp source tarball from GitHub releases
2. Configure via CMake with appropriate flags
3. Build the shared library (libwhisper.so)
4. Copy the library to the appropriate location for runtime loading

**Why**:
- Mirrors how the Windows/macOS builds work (download + extract)
- CMake provides robust cross-platform builds with official support
- Enables easy flag toggling for CUDA support

**Implementation approach**:
```
build.rs workflow:
1. Check for cached whisper.cpp source
2. Download source tarball if not cached
3. Run cmake configure with options:
   - BUILD_SHARED_LIBS=ON
   - CMAKE_BUILD_TYPE=Release
   - GGML_CUDA=ON (if cuda feature enabled)
4. Run cmake build
5. Copy libwhisper.so to output directory
6. Set up library search path for runtime
```

## Decision: CUDA Support via CMake Flag

**What**: The `cuda` feature flag will pass `-DGGML_CUDA=ON` to CMake.

**Why**:
- This is the official way to enable CUDA in whisper.cpp
- User still needs CUDA toolkit installed, but this is unavoidable for any CUDA build
- Same build-time requirement as the current whisper-rs/cuda approach

**Scenarios**:
- Default build: CPU-only, no CUDA toolkit required
- `--features cuda`: Requires CUDA toolkit, produces GPU-accelerated binary

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| CMake not installed | Build fails | Clear error message with install instructions |
| CUDA toolkit missing for cuda build | Build fails | Check for nvcc before attempting CUDA build |
| Longer builds than prebuilt downloads | Developer friction | Source caching, consider ccache support |
| ABI compatibility across Linux distros | Runtime crashes | Build with static linking where possible |
| whisper.cpp API changes | FFI breakage | Pin to specific version, test on updates |

## Implementation Phases

### Phase 1: Core Infrastructure
- Modify build.rs to download whisper.cpp source (not binaries)
- Add CMake build integration for Linux
- Produce libwhisper.so shared library

### Phase 2: FFI Unification
- Remove `#[cfg(target_os = "linux")]` guards from whisper_ffi.rs
- Update transcribe.rs to use FFI on all platforms
- Remove whisper-rs from Cargo.toml

### Phase 3: CUDA Support
- Add CMake CUDA flag detection
- Test CUDA build path
- Update documentation

## Build Dependencies

**CPU-only build**:
- CMake 3.14+
- C/C++ compiler (gcc, clang)
- git (for whisper.cpp submodules if needed)

**CUDA build** (additional):
- NVIDIA CUDA Toolkit 11.8+ or 12.x
- cuBLAS development libraries
- nvcc compiler

## Open Questions

1. **Source download format**: Should we download the release tarball or clone the git repo? Tarball is simpler but doesn't include submodules. Whisper.cpp's CMake fetches ggml as a dependency, so tarball should work.

2. **Build caching**: How aggressive should build caching be? Options:
   - Always rebuild (safest, slowest)
   - Cache based on version + features (reasonable default)
   - Full incremental builds (fastest, most complex)
   
   **Recommendation**: Cache based on version + features hash.

3. **Static vs dynamic linking**: Should libwhisper.so link its dependencies statically or dynamically?
   - Static: Larger binary, more portable
   - Dynamic: Smaller binary, requires runtime dependencies (OpenBLAS, etc.)
   
   **Recommendation**: Start with default CMake behavior (dynamic), document runtime dependencies.
