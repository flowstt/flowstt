# Design: Windows CUDA Acceleration

## Context

The current Windows implementation uses prebuilt whisper.cpp binaries downloaded from GitHub releases during the build process (`build.rs`). These are CPU-only binaries from `whisper-bin-x64.zip`. The whisper.cpp project also publishes CUDA-enabled binaries:
- `whisper-cublas-11.8.0-bin-x64.zip` (CUDA 11.8, ~59MB)
- `whisper-cublas-12.4.0-bin-x64.zip` (CUDA 12.4, ~457MB)

Unlike Linux (which uses `whisper-rs` and requires the CUDA Toolkit at build time), Windows can use prebuilt CUDA binaries that include all necessary runtime DLLs.

### Comparison with Linux CUDA Support

| Aspect | Linux | Windows |
|--------|-------|---------|
| Whisper integration | whisper-rs crate | FFI via libloading |
| CUDA source | Built from source at compile time | Prebuilt binaries from whisper.cpp releases |
| Build requirements | CUDA Toolkit (nvcc, cuBLAS) | None (uses prebuilt) |
| Runtime requirements | NVIDIA drivers + CUDA runtime | NVIDIA drivers (runtime bundled) |
| Binary size impact | ~20-50MB | ~60-460MB depending on CUDA version |

## Goals

- Enable opt-in CUDA acceleration for Windows users with NVIDIA GPUs
- Maintain backward compatibility (CPU-only builds remain the default)
- No additional build-time dependencies required
- Bundle CUDA runtime DLLs so end users don't need to install CUDA Toolkit

## Non-Goals

- CUDA 11.x support (only CUDA 12.4 will be supported initially for simplicity)
- User selection of CUDA version at runtime (compile-time selection only)
- AMD GPU support via ROCm (different architecture)
- Automatic GPU detection and fallback (users must build with correct feature)

## Decisions

### Decision: Use CUDA 12.4 prebuilt binaries

**What**: When the `cuda` feature is enabled on Windows, download `whisper-cublas-12.4.0-bin-x64.zip` instead of the CPU-only binary.

**Why**:
- CUDA 12.4 is the more recent version with broader driver compatibility
- Prebuilt binaries include all necessary CUDA runtime DLLs
- Simplifies build process - no CUDA Toolkit installation required
- Consistent with the approach of using prebuilt binaries on Windows/macOS

**Alternatives considered**:
- CUDA 11.8: Older, smaller binary (~59MB vs ~457MB), but less future-proof
- Both versions: Adds complexity with another feature flag; not needed initially
- Build from source: Would require CUDA Toolkit installed, inconsistent with current approach

### Decision: Reuse existing `cuda` Cargo feature

**What**: The existing `cuda` feature flag (currently Linux-only) will also affect Windows builds.

**Why**:
- Consistent developer experience across platforms
- Single feature flag for GPU acceleration regardless of platform
- The Linux implementation already documents that the feature is "Linux only" - we're extending it

**Implementation**:
```toml
[features]
cuda = ["whisper-rs/cuda"]  # Existing - affects Linux via whisper-rs
# Windows CUDA is handled in build.rs based on this feature
```

The `build.rs` will check for the `cuda` feature via `CARGO_FEATURE_CUDA` environment variable.

### Decision: Bundle CUDA runtime DLLs

**What**: When built with CUDA, include all necessary CUDA DLLs in the application bundle.

**Why**:
- End users don't need to install CUDA Toolkit
- Simpler distribution and installation
- The whisper.cpp CUDA release includes these DLLs

**DLLs to bundle** (from whisper-cublas-12.4.0-bin-x64.zip):
- `whisper.dll` - Main whisper library
- `ggml.dll`, `ggml-base.dll`, `ggml-cpu.dll`, `ggml-cuda.dll` - GGML backends
- `cublas64_12.dll`, `cublasLt64_12.dll` - cuBLAS libraries
- `cudart64_12.dll` - CUDA runtime

### Decision: No runtime GPU detection or fallback

**What**: If built with CUDA and no compatible GPU is available, the application will fail to transcribe (same as Linux behavior).

**Why**:
- Keeps implementation simple
- Users explicitly opt into CUDA at build time
- Bundling both CPU and CUDA libraries would significantly increase binary size
- Error messages from whisper.cpp are informative enough

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Large binary size (~457MB for CUDA build) | Document size difference; CUDA is opt-in |
| CUDA driver version incompatibility | Document minimum driver requirements; CUDA 12.4 has good backward compatibility |
| DLL loading failures | Clear error messages; document troubleshooting |
| User confusion about when to use CUDA | Clear documentation in README |

## Implementation Approach

### build.rs Changes

```rust
// Detect if cuda feature is enabled
let cuda_enabled = env::var("CARGO_FEATURE_CUDA").is_ok();

// Windows binary selection
let (zip_name, lib_names) = if cuda_enabled {
    (
        "whisper-cublas-12.4.0-bin-x64.zip",
        vec![
            "whisper.dll",
            "ggml.dll",
            "ggml-base.dll",
            "ggml-cpu.dll",
            "ggml-cuda.dll",
            "cublas64_12.dll",
            "cublasLt64_12.dll",
            "cudart64_12.dll",
        ],
    )
} else {
    (
        "whisper-bin-x64.zip",
        vec!["whisper.dll", "ggml.dll", "ggml-base.dll", "ggml-cpu.dll"],
    )
};
```

### Build Commands

```bash
# Default CPU-only build (no change)
cargo build --release

# CUDA-enabled build (Windows)
cargo build --release --features cuda

# Tauri build with CUDA
cargo tauri build --features cuda
```

## Open Questions

None - the implementation is straightforward since we're leveraging existing prebuilt binaries.
