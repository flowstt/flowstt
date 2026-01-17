## Context

FlowSTT is a cross-platform Tauri application with:
- Rust workspace containing: `src-tauri` (GUI), `src-common` (shared), `src-service` (background service), `src-cli` (CLI)
- TypeScript/Vite frontend
- Platform-specific audio dependencies (PipeWire on Linux, CoreAudio on macOS, WASAPI on Windows)

The omnirec project provides a reference implementation for similar CI/CD needs.

## Goals / Non-Goals

**Goals:**
- Automated linting and testing on every PR and push to master
- Automated release builds triggered by version tags
- Cross-platform support (Linux, macOS Intel/ARM, Windows)
- Caching for faster CI runs

**Non-Goals:**
- CUDA-enabled builds (requires NVIDIA toolkit, not available in GitHub-hosted runners)
- Automatic version bumping or changelog generation
- Deployment to package registries (AUR, Homebrew, etc.)

## Decisions

### 1. Workflow Triggers
- **CI**: Push to `master` branch and all pull requests targeting `master`
- **Release**: Push of tags matching `v*.*.*` pattern

**Rationale**: Standard Git flow pattern. Version tags trigger releases, matching Tauri's version management.

### 2. Platform Matrix
- Linux: `ubuntu-22.04` (matches PipeWire requirements)
- macOS: `macos-latest` (builds both aarch64 and x86_64 for releases)
- Windows: `windows-latest`

**Rationale**: Ubuntu 22.04 required for PipeWire 0.3.65+ via PPA. macOS universal binary support for Apple Silicon and Intel.

### 3. Linux Dependencies
Install via apt with PipeWire PPA:
- `libwebkit2gtk-4.1-dev` (Tauri WebView)
- `libappindicator3-dev` (system tray)
- `librsvg2-dev` (SVG rendering)
- `patchelf` (binary patching)
- `libpipewire-0.3-dev`, `libspa-0.2-dev` (audio capture)
- `libclang-dev` (FFI bindings)

**Rationale**: Matches existing project build requirements from Makefile and Cargo dependencies.

### 4. Rust Caching Strategy
Use `Swatinem/rust-cache@v2` with explicit workspace paths:
```yaml
workspaces: |
  src-tauri -> target
  src-service -> target
  src-common -> target
  src-cli -> target
```

**Rationale**: Shared target directory in workspace root, but explicit paths ensure proper cache invalidation.

### 5. Clippy Configuration
- All crates: `--all-targets --all-features -- -D warnings`
- Exception: `src-service` uses `--all-targets` only (no `--all-features`) because `cuda` feature requires NVIDIA CUDA Toolkit

**Rationale**: Treat warnings as errors for quality enforcement. CUDA feature excluded from CI as it requires hardware.

### 6. Release Build Strategy
- Build service and CLI binaries first
- Stage CLI binary into `src-tauri/binaries/` for Tauri sidecar bundling (if needed)
- Use `tauri-apps/tauri-action@v0.6.0` for app bundling
- Create draft releases for manual review before publishing

**Rationale**: Follows Tauri's recommended release pattern. Draft releases allow verification before public release.

### 7. macOS Universal Binary
Release workflow builds both targets:
- `aarch64-apple-darwin` (Apple Silicon)
- `x86_64-apple-darwin` (Intel)

**Rationale**: Ensures compatibility with both Apple Silicon and older Intel Macs.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Linux dependencies may change | Pin to ubuntu-22.04; document PPA requirement |
| Cache invalidation issues | Use workspace-aware caching; monitor build times |
| Long build times | Parallel platform builds; aggressive caching |
| CUDA builds not tested | Document that CUDA is local-build only |

## Migration Plan

1. Create `.github/workflows/` directory
2. Add `ci.yml` and `release.yml` workflows
3. Push to branch and verify CI runs
4. Create test tag to verify release workflow
5. Update README with CI badge (optional)

## Open Questions

- Should we add a workflow dispatch trigger for manual releases?
- Should CLI binary be bundled as Tauri sidecar or separate release asset?
