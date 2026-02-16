<picture>
  <source srcset="images/flowstt-landscape.svg" media="(prefers-color-scheme: dark)">
  <source srcset="images/flowstt-landscape-light.svg" media="(prefers-color-scheme: light)">
  <img src="images/flowstt-landscape.svg" alt="FlowSTT logo">
</picture>

---

FlowSTT is a free, privacy-first speech-to-text application that runs entirely on your local machine. No subscriptions, no signups, no cloud services —- your voice data never leaves your computer.

## Features

- **Local transcription** — Offline Whisper inference via whisper.cpp
- **Hardware accelerated** — CUDA on Windows/Linux, Metal on macOS
- **Real-time visualization** — Waveform, spectrogram, and speech activity graphs
- **Multi-source audio** — Microphone, system audio, or mixed mode with echo cancellation (WebRTC AEC3)
- **Cross-platform** — Windows (WASAPI), macOS (CoreAudio), Linux (PipeWire)
- **Scriptable CLI** — Full command-line interface with JSON output

## Installation

### Windows

### macOS

### Linux

## CLI Usage

```bash
flowstt list                    # List audio devices
flowstt transcribe              # Start transcription
flowstt status                  # Show service state
flowstt stop                    # Stop transcription
flowstt model                   # Show Whisper model status
flowstt model download          # Download Whisper model
flowstt gpu                     # Show GPU/CUDA status
flowstt config show             # Display configuration
flowstt config set key val      # Set config value
flowstt setup                   # Interactive first-time setup
```

Global flags: `--format json` for machine output, `-q/--quiet`, `-v/--verbose`.

## Development

### Architecture

FlowSTT runs as three cooperating binaries:

| Binary | Role |
|--------|------|
| `flowstt-service` | Background daemon for audio capture and transcription |
| `flowstt` | CLI for headless operation and scripting |
| `flowstt-app` | Tauri 2.0 desktop GUI |

Clients communicate with the service over platform-native IPC (Unix sockets on Linux/macOS, named pipes on Windows).

### Build

Prerequisites: Rust toolchain, pnpm

```bash
# Install
pnpm install

# Standard build
make build

# Debug build
make build-debug

# Lint and test
make lint
make test

# Run the Service
make run-service

# Run the UI
pnpm tauri dev
```

## Tech Stack

- **Backend**: Rust, Tauri 2.0, whisper-rs, WebRTC AEC3, rustfft
- **Frontend**: TypeScript, Vite
- **Audio**: WASAPI (Windows), CoreAudio (macOS), PipeWire (Linux)

## License

MIT
