# Change: Add GitHub Actions CI and Release Workflows

## Why

The project lacks automated CI/CD pipelines. Adding GitHub Actions workflows will:
- Automatically validate code quality on every push and PR
- Automate cross-platform release artifact generation
- Reduce manual testing burden and catch issues early

## What Changes

- Add `.github/workflows/ci.yml` for continuous integration (lint, test, type-check)
- Add `.github/workflows/release.yml` for automated release builds on version tags
- Support for all three platforms: Linux (ubuntu-22.04), macOS (latest), Windows (latest)
- Cross-platform Rust workspace builds (src-tauri, src-common, src-service, src-cli)
- TypeScript type checking for frontend
- Tauri application bundling for releases
- Version validation to ensure releases increment properly

## Impact

- Affected specs: `build-tooling` (new CI/CD automation capability)
- Affected code: No existing code changes; adds new `.github/workflows/` directory
- New capability: `ci-automation` spec for workflow requirements
