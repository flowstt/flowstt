## Context
The current FlowSTT main window has a fixed 900x340 size, displays transcription as a single continuous text buffer (in-memory only, capped at 2000 chars), saves WAV files to `~/Documents/Recordings/`, and shows the mini-visualizer at all times. This change introduces persistent transcription history with segment-level management, WAV file caching in OS-standard directories, and UI improvements.

## Goals / Non-Goals
- Goals:
  - Make the main window resizable with responsive content layout
  - Add persistent transcription history with per-segment metadata
  - Store WAV files in OS-appropriate app data directory
  - Auto-cleanup old WAV files (>24h)
  - Provide per-segment UI actions (copy, delete, play)
  - Hide mini-visualizer when not recording
  - Remove the top fade gradient overlay
- Non-Goals:
  - Search/filter within transcription history
  - Export history to external formats
  - Cloud sync of history
  - Infinite history retention (WAV files cleaned after 24h; text history retained)

## Decisions

### History Storage Location
- Decision: Use `directories::ProjectDirs::data_dir()` for both history JSON and cached WAV files
  - Windows: `%APPDATA%/flowstt/`
  - Linux: `~/.local/share/flowstt/`
  - macOS: `~/Library/Application Support/flowstt/`
- Alternatives considered:
  - `BaseDirs::home_dir()/Documents/Recordings` (current) -- user-visible, clutters Documents
  - Separate cache dir for WAVs -- unnecessary split; data_dir is appropriate for app-managed files
- Rationale: Standard app data directory keeps files organized and hidden from casual browsing. The `directories` crate is already a dependency.

### History File Format
- Decision: Single JSON file (`history.json`) with an array of history entries
- Each entry: `{ id: string, text: string, timestamp: string (ISO 8601), wav_path: string | null }`
- Alternatives considered:
  - SQLite database -- overkill for the expected data volume
  - One file per segment -- more filesystem overhead
- Rationale: JSON matches existing config persistence pattern (`config.json`). Simple to read/write with serde_json (already a dependency).

### WAV File Organization
- Decision: WAV files stored in `<data_dir>/recordings/` subdirectory with existing timestamped naming (`flowstt-YYYYMMDD-HHMMSS.wav`)
- Decision: `wav_path` in history entries stores the full absolute path
- Rationale: Keeps recordings co-located with history. Absolute paths avoid ambiguity if data dir changes.

### History Management Architecture
- Decision: History persistence lives in the service (`src-service`), not the Tauri frontend
- New IPC requests: `GetHistory`, `DeleteHistoryEntry { id }`, `GetHistoryWavPath { id }`
- Service manages history file reads/writes and WAV cleanup
- Frontend requests history on startup and receives updates via events
- Alternatives considered:
  - Frontend-only persistence via Tauri filesystem APIs -- would duplicate state, harder to keep in sync with WAV saving
- Rationale: The service already saves WAV files and emits transcription results. Adding history tracking there keeps it authoritative.

### Transcription Complete Event Enhancement
- Decision: Extend the `transcription-complete` event payload to include `{ id, text, timestamp, wav_path }` instead of just the text string
- Frontend uses this to append new history entries to the display
- Rationale: Frontend needs segment metadata to render the history UI without a separate round-trip.

### WAV Cleanup Strategy
- Decision: On service startup, scan `<data_dir>/recordings/` and delete WAV files with mtime > 24 hours. Also remove corresponding `wav_path` references in history entries (set to null).
- Decision: Text entries in history are NOT deleted by cleanup -- only WAV files are removed.
- Alternatives considered:
  - Background timer during runtime -- adds complexity; startup-only is sufficient for daily cleanup
  - Delete history entries entirely when WAV expires -- user may want to keep the text
- Rationale: Simple startup-time scan is adequate. Users who run the app daily get cleanup on each launch.

### Resizable Window Approach
- Decision: Set `resizable: true` in Tauri config and add `minWidth`/`minHeight` constraints. Use CSS flexbox (already in use) to handle responsive sizing.
- Decision: Minimum size 600x300 to prevent layout breakage. Default remains 900x340.
- Rationale: The existing CSS layout already uses `flex: 1` for the transcription area, so it will expand naturally. Only minor adjustments needed for margins and the header.

### Mini-Visualizer Visibility
- Decision: The mini-visualizer canvas is hidden by default (CSS `display: none`). It becomes visible when a recording/capture session is active (PTT key held or speech detected in auto mode). It hides again when capture stops.
- Alternatives considered:
  - Show idle state (flat line) -- current behavior, but user wants it hidden
- Rationale: Directly implements the user's request. Reduces visual clutter when idle.

### Audio Playback
- Decision: Use the Tauri shell/audio API or a simple HTML5 `<audio>` element to play WAV files. The frontend requests the file path via IPC and plays it using a dynamically created audio element with `convertFileSrc()`.
- Rationale: Simplest approach; WAV is natively supported by all browsers/webviews. No additional dependencies needed.

## Risks / Trade-offs
- History JSON file could grow large over time if text entries are never cleaned -- mitigated by the fact that individual text entries are small (< 1KB each). Could add a configurable max-entries limit later if needed.
- Moving WAV save location is a **breaking change** for users who expect files in `~/Documents/Recordings/` -- acceptable since there's no documented contract and the old directory can remain as-is (existing files are not migrated).
- Startup cleanup only runs when service starts -- if service runs for >24h, old WAVs persist until next restart. Acceptable trade-off for simplicity.

## Open Questions
- None at this time.
