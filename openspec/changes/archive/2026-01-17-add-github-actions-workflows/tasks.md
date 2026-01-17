## 1. Implementation

- [x] 1.1 Create `.github/workflows/` directory structure
- [x] 1.2 Implement `ci.yml` workflow with:
  - [x] 1.2.1 Trigger on push to master and pull requests
  - [x] 1.2.2 Matrix strategy for ubuntu-22.04, macos-latest, windows-latest
  - [x] 1.2.3 Install Linux dependencies (PipeWire PPA, webkit, etc.)
  - [x] 1.2.4 Setup Node.js and pnpm
  - [x] 1.2.5 Setup Rust toolchain with clippy
  - [x] 1.2.6 Cache Rust dependencies for all workspace crates
  - [x] 1.2.7 TypeScript type check step
  - [x] 1.2.8 Clippy lint for all workspace crates
  - [x] 1.2.9 Rust tests for all workspace crates
- [x] 1.3 Implement `release.yml` workflow with:
  - [x] 1.3.1 Trigger on version tags (v*.*.*)
  - [x] 1.3.2 Version validation job
  - [x] 1.3.3 Test and lint job (reuse CI logic)
  - [x] 1.3.4 Build and release job with platform matrix
  - [x] 1.3.5 Build service and CLI binaries per platform
  - [x] 1.3.6 Tauri app bundling via tauri-action
  - [x] 1.3.7 Linux tar.gz archive creation
  - [x] 1.3.8 Upload release artifacts

## 2. Validation

- [x] 2.1 Run `openspec validate add-github-actions-workflows --strict`
- [x] 2.2 Verify workflow syntax with GitHub Actions linter (optional)
