//! Transcription history helpers for the Tauri app.
//!
//! Uses vtx-engine's TranscriptionHistory for persistence.

use vtx_common::HistoryEntry;
use vtx_engine::TranscriptionHistory;

const MAX_HISTORY_ENTRIES: usize = 500;

/// Load all history entries from disk.
pub fn load_history() -> Result<Vec<HistoryEntry>, String> {
    let history = TranscriptionHistory::open("FlowSTT", MAX_HISTORY_ENTRIES)
        .map_err(|e| format!("Failed to open history: {}", e))?;
    Ok(history.entries().to_vec())
}

/// Delete a history entry and its associated WAV file.
pub fn delete_history_entry(id: &str) -> Result<(), String> {
    let mut history = TranscriptionHistory::open("FlowSTT", MAX_HISTORY_ENTRIES)
        .map_err(|e| format!("Failed to open history: {}", e))?;
    history.delete(id);
    Ok(())
}
