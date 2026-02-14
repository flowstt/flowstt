## 1. Resizable Main Window
- [x] 1.1 Update `src-tauri/tauri.conf.json`: set `resizable: true`, add `minWidth: 600`, `minHeight: 300`
- [x] 1.2 Update `src-tauri/src/tray/windows.rs` window recreation: set `.resizable(true)`, add `.min_inner_size(600.0, 300.0)`
- [x] 1.3 Verify CSS flexbox layout in `src/styles.css` handles resize correctly (transcription area `flex: 1` should work; check header and controls bar stay anchored)
- [x] 1.4 Test: resize window, confirm header/controls maintain margins, transcription area fills available space

## 2. Mini-Visualizer Visibility
- [x] 2.1 In `src/styles.css`, add default `display: none` to `.mini-waveform`
- [x] 2.2 In `src/main.ts`, on `capture-state-changed` with `capturing: true`: show mini-waveform (`display: block`), on `capturing: false`: hide it (`display: none`)
- [x] 2.3 Update init logic (`main.ts` lines 636-639): only show mini-waveform if already capturing
- [x] 2.4 Test: confirm mini-waveform hidden on idle, visible during PTT hold and auto-transcription

## 3. Persistent Transcription History (Service)
- [x] 3.1 Create `src-service/src/history.rs` module with `HistoryEntry` struct (`id: String`, `text: String`, `timestamp: String`, `wav_path: Option<String>`) and `TranscriptionHistory` struct
- [x] 3.2 Implement `TranscriptionHistory::load()` / `save()` using `directories::ProjectDirs::from("", "", "flowstt")` data_dir + `history.json`
- [x] 3.3 Implement `add_entry()`, `delete_entry(id)`, `get_entries()`, `cleanup_wav_files(max_age: Duration)` methods
- [x] 3.4 Handle corrupted JSON file: backup and start fresh
- [x] 3.5 Register history module in `src-service/src/main.rs`
- [x] 3.6 Wire history: after transcription completes, create history entry with text, timestamp, and wav_path; save to file

## 4. WAV File Location Change
- [x] 4.1 Update `src-service/src/transcription/transcribe_state.rs` `queue_segment()`: change recordings dir from `home_dir/Documents/Recordings` to `ProjectDirs::data_dir()/recordings`
- [x] 4.2 Verify `generate_recording_filename()` still works (no changes needed)
- [x] 4.3 Test: confirm WAV files appear in new location

## 5. WAV Cleanup on Startup
- [x] 5.1 In service startup (`src-service/src/main.rs`), after loading history, call `cleanup_wav_files(Duration::from_secs(86400))`
- [x] 5.2 Cleanup scans recordings dir, deletes files with mtime > 24h, nullifies corresponding `wav_path` entries in history
- [x] 5.3 Test: create old WAV files, start service, confirm they are deleted and history entries updated

## 6. IPC Protocol Extensions
- [x] 6.1 Add `GetHistory` request to `src-common/src/ipc/requests.rs`
- [x] 6.2 Add `DeleteHistoryEntry { id: String }` request to `src-common/src/ipc/requests.rs`
- [x] 6.3 Add `HistoryResponse` (list of entries) and `HistoryEntryDeleted { id: String }` to `src-common/src/ipc/responses.rs`
- [x] 6.4 Extend `TranscriptionComplete` event payload to include `id`, `timestamp`, and `wav_path` fields alongside `text`
- [x] 6.5 Handle new requests in service IPC handler
- [x] 6.6 Broadcast `HistoryEntryDeleted` event to all subscribed clients when an entry is deleted

## 7. Tauri Command Layer
- [x] 7.1 Add `get_history` Tauri command in `src-tauri/src/lib.rs` that sends `GetHistory` via IPC client
- [x] 7.2 Add `delete_history_entry` Tauri command that sends `DeleteHistoryEntry` via IPC client
- [x] 7.3 Register new commands in Tauri builder
- [x] 7.4 Forward `HistoryEntryDeleted` events from service to frontend via Tauri event system

## 8. Frontend Transcription History UI
- [x] 8.1 Update `index.html`: replace `.result-content` inner structure with a scrollable history container; remove `.result-fade` div
- [x] 8.2 Remove `.result-fade` CSS from `src/styles.css`
- [x] 8.3 Add CSS styles for history segments: `.history-segment` row with timestamp, text, and action buttons; scrollable container with `overflow-y: auto`
- [x] 8.4 Rewrite `src/main.ts` transcription display: remove `transcriptionBuffer`, `updateTranscriptionDisplay()`, `appendTranscription()`
- [x] 8.5 On app load, call `get_history` Tauri command and render all existing segments
- [x] 8.6 On `transcription-complete` event (now with enriched payload), append a new segment row
- [x] 8.7 Implement copy button: `navigator.clipboard.writeText(segmentText)`
- [x] 8.8 Implement delete button: call `delete_history_entry` command, remove segment row from DOM
- [x] 8.9 Implement play button: use `convertFileSrc()` to get asset URL, create `<audio>` element, play WAV; only show button if `wav_path` is non-null
- [x] 8.10 Listen for `history-entry-deleted` event (from other clients or cleanup) and remove segment from DOM if present

## 9. Integration Testing
- [x] 9.1 Full flow test: PTT hold -> speak -> release -> verify segment appears in UI with timestamp, copy, delete, play buttons
- [x] 9.2 Verify history persists across service restart
- [x] 9.3 Verify delete removes segment from UI, history file, and WAV file
- [x] 9.4 Verify play button works for recent segments and is hidden for segments with expired WAV
- [x] 9.5 Verify window resize behavior
- [x] 9.6 Build succeeds: `pnpm tauri build`
