# ci-automation Specification

## Purpose
TBD - created by archiving change add-github-actions-workflows. Update Purpose after archive.
## Requirements
### Requirement: CI Workflow Trigger
The CI workflow SHALL run automatically on push to master branch and on pull requests targeting master.

#### Scenario: Push to master triggers CI
- **WHEN** a commit is pushed to the `master` branch
- **THEN** the CI workflow executes all lint and test jobs

#### Scenario: Pull request triggers CI
- **WHEN** a pull request is opened or updated targeting `master`
- **THEN** the CI workflow executes all lint and test jobs

### Requirement: Cross-Platform CI Matrix
The CI workflow SHALL run on all supported platforms using a matrix strategy.

#### Scenario: Linux CI execution
- **WHEN** the CI workflow runs
- **THEN** a job executes on `ubuntu-22.04` runner

#### Scenario: macOS CI execution
- **WHEN** the CI workflow runs
- **THEN** a job executes on `macos-latest` runner

#### Scenario: Windows CI execution
- **WHEN** the CI workflow runs
- **THEN** a job executes on `windows-latest` runner

### Requirement: Linux Dependency Installation
The CI workflow SHALL install required system dependencies on Linux runners.

#### Scenario: PipeWire PPA added
- **WHEN** the CI runs on Linux
- **THEN** the PipeWire upstream PPA is added to apt sources

#### Scenario: Build dependencies installed
- **WHEN** the CI runs on Linux
- **THEN** webkit, appindicator, rsvg, patchelf, pipewire, spa, and clang dev packages are installed

### Requirement: TypeScript Type Checking
The CI workflow SHALL validate TypeScript types on all platforms.

#### Scenario: TypeScript check passes
- **WHEN** all TypeScript files have valid types
- **THEN** the `tsc --noEmit` step succeeds

#### Scenario: TypeScript check fails
- **WHEN** any TypeScript file has type errors
- **THEN** the `tsc --noEmit` step fails and reports errors

### Requirement: Rust Linting
The CI workflow SHALL run clippy on all workspace crates with warnings as errors.

#### Scenario: Clippy on common library
- **WHEN** the CI runs
- **THEN** clippy runs on `src-common` with `--all-targets --all-features -D warnings`

#### Scenario: Clippy on service
- **WHEN** the CI runs
- **THEN** clippy runs on `src-service` with `--all-targets -D warnings` (excluding cuda feature)

#### Scenario: Clippy on main app
- **WHEN** the CI runs
- **THEN** clippy runs on `src-tauri` with `--all-targets --all-features -D warnings`

#### Scenario: Clippy on CLI
- **WHEN** the CI runs
- **THEN** clippy runs on `src-cli` with `--all-targets --all-features -D warnings`

### Requirement: Rust Testing
The CI workflow SHALL run tests for all workspace crates.

#### Scenario: Tests for common library
- **WHEN** the CI runs
- **THEN** cargo test runs on `src-common` with `--all-features`

#### Scenario: Tests for service
- **WHEN** the CI runs
- **THEN** cargo test runs on `src-service` (excluding cuda feature)

#### Scenario: Tests for main app
- **WHEN** the CI runs
- **THEN** cargo test runs on `src-tauri` with `--all-features`

#### Scenario: Tests for CLI
- **WHEN** the CI runs
- **THEN** cargo test runs on `src-cli` with `--all-features`

### Requirement: Dependency Caching
The CI workflow SHALL cache Rust dependencies to speed up builds.

#### Scenario: Rust cache configured
- **WHEN** the CI runs
- **THEN** rust-cache action caches all workspace target directories

#### Scenario: Cache hit speeds up build
- **WHEN** dependencies have not changed since last run
- **THEN** cached dependencies are restored instead of recompiled

### Requirement: Release Workflow Trigger
The release workflow SHALL run on version tag pushes.

#### Scenario: Version tag triggers release
- **WHEN** a tag matching `v*.*.*` pattern is pushed
- **THEN** the release workflow executes

#### Scenario: Non-version tag ignored
- **WHEN** a tag not matching `v*.*.*` is pushed
- **THEN** the release workflow does not execute

### Requirement: Version Validation
The release workflow SHALL validate that the new version is greater than the latest release.

#### Scenario: Valid version increment
- **WHEN** the tag version is greater than the latest release
- **THEN** the workflow proceeds with the build

#### Scenario: Invalid version increment
- **WHEN** the tag version is not greater than the latest release
- **THEN** the workflow fails with a version error

#### Scenario: First release
- **WHEN** no previous releases exist
- **THEN** version validation passes without comparison

### Requirement: Release Artifact Building
The release workflow SHALL build release artifacts for all platforms.

#### Scenario: Linux release build
- **WHEN** the release workflow runs
- **THEN** Linux binaries are built on `ubuntu-22.04`

#### Scenario: macOS ARM release build
- **WHEN** the release workflow runs
- **THEN** macOS binaries are built targeting `aarch64-apple-darwin`

#### Scenario: macOS Intel release build
- **WHEN** the release workflow runs
- **THEN** macOS binaries are built targeting `x86_64-apple-darwin`

#### Scenario: Windows release build
- **WHEN** the release workflow runs
- **THEN** Windows binaries are built on `windows-latest`

### Requirement: Tauri App Bundling
The release workflow SHALL create Tauri application bundles using the official action.

#### Scenario: Tauri action builds app
- **WHEN** the release build job runs
- **THEN** `tauri-apps/tauri-action` creates platform-specific installers

#### Scenario: Release created as draft
- **WHEN** the Tauri action completes
- **THEN** a draft GitHub release is created with build artifacts

### Requirement: Service Binary Building
The release workflow SHALL build the service binary for each platform.

#### Scenario: Linux service build
- **WHEN** the release runs on Linux
- **THEN** `flowstt-service` is built in release mode

#### Scenario: macOS service build
- **WHEN** the release runs on macOS
- **THEN** `flowstt-service` is built for both aarch64 and x86_64 targets

#### Scenario: Windows service build
- **WHEN** the release runs on Windows
- **THEN** `flowstt-service` is built in release mode

### Requirement: CLI Binary Building
The release workflow SHALL build the CLI binary for each platform.

#### Scenario: Linux CLI build
- **WHEN** the release runs on Linux
- **THEN** `flowstt` CLI is built in release mode

#### Scenario: macOS CLI build
- **WHEN** the release runs on macOS
- **THEN** `flowstt` CLI is built for both aarch64 and x86_64 targets

#### Scenario: Windows CLI build
- **WHEN** the release runs on Windows
- **THEN** `flowstt` CLI is built in release mode

### Requirement: Linux Archive Creation
The release workflow SHALL create a tar.gz archive for Linux distribution.

#### Scenario: Archive contains all binaries
- **WHEN** the Linux build completes
- **THEN** a tar.gz archive is created containing app, CLI, and service binaries

#### Scenario: Archive contains icons
- **WHEN** the Linux build completes
- **THEN** the tar.gz archive includes application icons

#### Scenario: Archive uploaded to release
- **WHEN** the archive is created
- **THEN** it is uploaded to the GitHub release

