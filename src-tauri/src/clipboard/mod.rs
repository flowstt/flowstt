//! Clipboard write, foreground detection, paste simulation, and clipboard restore.
//!
//! After each transcription segment completes, this module copies the text to
//! the system clipboard and optionally simulates a paste keystroke into the
//! active foreground application. Paste simulation is suppressed when a FlowSTT
//! window is in the foreground.
//!
//! When `restore_clipboard_enabled` is true, the clipboard contents present
//! before the transcription paste are saved and restored afterwards.
//!
//! Platform-specific implementations live in submodules following the same
//! backend-trait pattern used by `crate::hotkey`.

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;

use std::time::Duration;
use tracing::{debug, info, warn};

/// Opaque snapshot of clipboard contents captured before a transcription paste.
///
/// The representation is platform-specific: each backend captures whatever is
/// needed to faithfully restore the clipboard state on that platform.
#[cfg(target_os = "windows")]
pub type ClipboardContents = windows::ClipboardContents;

#[cfg(target_os = "macos")]
pub type ClipboardContents = macos::ClipboardContents;

#[cfg(target_os = "linux")]
pub type ClipboardContents = linux::ClipboardContents;

/// Platform-agnostic clipboard and paste backend.
pub trait ClipboardPaster: Send + Sync {
    /// Write plain text to the system clipboard.
    fn write_clipboard(&self, text: &str) -> Result<(), String>;

    /// Check whether the current foreground window belongs to FlowSTT.
    fn is_flowstt_foreground(&self) -> bool;

    /// Simulate a paste keystroke (Ctrl+V / Cmd+V) into the foreground window.
    fn simulate_paste(&self) -> Result<(), String>;

    /// Read the current clipboard contents.
    ///
    /// Returns `None` if the clipboard is empty or cannot be read.
    fn read_clipboard(&self) -> Option<ClipboardContents>;

    /// Restore previously saved clipboard contents.
    ///
    /// Returns an error if the restore fails (caller should log and continue).
    fn restore_clipboard(&self, contents: &ClipboardContents) -> Result<(), String>;
}

/// Create the platform-specific backend.
fn create_backend() -> Box<dyn ClipboardPaster> {
    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsClipboardPaster)
    }

    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOSClipboardPaster)
    }

    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxClipboardPaster)
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        compile_error!("Unsupported platform for clipboard/paste");
    }
}

/// Perform the full clipboard-copy-and-paste flow for a transcription result.
///
/// 1. Skip if the text is empty or a "no speech" placeholder.
/// 2. Optionally save the current clipboard contents if `restore_clipboard_enabled`.
/// 3. Write the text to the clipboard.
/// 4. If `auto_paste` is enabled and the foreground window is not FlowSTT,
///    wait `delay` and simulate a paste keystroke.
/// 5. Optionally restore the previously saved clipboard contents, unless another
///    app modified the clipboard between our write and the restore.
pub fn copy_and_paste(
    text: &str,
    auto_paste_enabled: bool,
    delay_ms: u32,
    restore_clipboard_enabled: bool,
) {
    // Skip empty / no-speech results
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed == "(No speech detected)" {
        return;
    }

    let backend = create_backend();

    // Save clipboard contents before we overwrite them
    let saved = if restore_clipboard_enabled {
        let contents = backend.read_clipboard();
        if contents.is_some() {
            debug!("[Clipboard] Saved clipboard contents for later restore");
        }
        contents
    } else {
        None
    };

    // Always write to clipboard (preserve original text including trailing space)
    if let Err(e) = backend.write_clipboard(text) {
        warn!("[Clipboard] Failed to write clipboard: {}", e);
        return;
    }
    debug!("[Clipboard] Text copied to clipboard");

    // Paste only when enabled
    if auto_paste_enabled {
        // Suppress paste when FlowSTT is the foreground window
        if backend.is_flowstt_foreground() {
            info!("[Clipboard] FlowSTT is foreground, skipping paste");
        } else {
            // Configurable delay before simulating paste
            if delay_ms > 0 {
                std::thread::sleep(Duration::from_millis(delay_ms as u64));
            }

            if let Err(e) = backend.simulate_paste() {
                warn!("[Clipboard] Failed to simulate paste: {}", e);
            } else {
                debug!("[Clipboard] Paste simulated into foreground application");
                // Wait for the target app to read the clipboard before restoring.
                // The pre-paste delay is already applied; we need a post-paste wait too.
                let post_paste_ms = delay_ms.max(150);
                std::thread::sleep(Duration::from_millis(post_paste_ms as u64));
            }
        }
    }

    // Restore clipboard if we saved it
    if let Some(ref contents) = saved {
        if let Err(e) = backend.restore_clipboard(contents) {
            warn!("[Clipboard] Failed to restore clipboard: {}", e);
        } else {
            debug!("[Clipboard] Clipboard contents restored");
        }
    }
}
