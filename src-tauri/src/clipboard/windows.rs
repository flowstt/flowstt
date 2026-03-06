//! Windows clipboard, foreground detection, and paste simulation.
//!
//! Uses Win32 APIs:
//! - Clipboard: `OpenClipboard` / `EmptyClipboard` / `SetClipboardData` / `CloseClipboard`
//! - Clipboard read: `GetClipboardSequenceNumber` / `EnumClipboardFormats` / `GetClipboardData`
//! - Foreground: `GetForegroundWindow` / `GetWindowThreadProcessId`
//! - Paste sim: `SendInput` with `INPUT_KEYBOARD` for Ctrl+V

use super::ClipboardPaster;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use tracing::{debug, warn};
use windows::Win32::Foundation::{HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData, OpenClipboard,
    SetClipboardData,
};
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    MapVirtualKeyW, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, MAP_VIRTUAL_KEY_TYPE, VIRTUAL_KEY, VK_CONTROL, VK_V,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

/// The Win32 clipboard format for Unicode text.
const CF_UNICODETEXT: u32 = 13;

/// Clipboard formats that store GDI handles or have special ownership semantics.
///
/// These cannot be saved/restored by copying raw HGLOBAL memory bytes.
/// - CF_BITMAP (2): HBITMAP — GDI object
/// - CF_METAFILEPICT (3): HMETAFILEPICT — GDI object
/// - CF_SYLK (4), CF_DIF (5), CF_TIFF (6): special data formats
/// - CF_PALETTE (9): HPALETTE — GDI object
/// - CF_PENDATA (10): pen input data
/// - CF_RIFF (11), CF_WAVE (12): audio handle formats
/// - CF_ENHMETAFILE (14): HENHMETAFILE — GDI object
/// - CF_HDROP (15): HDROP file list handle
/// - CF_LOCALE (16): locale handle
/// - CF_OWNERDISPLAY (0x80): owner-draw format (no data)
const NON_HGLOBAL_FORMATS: &[u32] = &[2, 3, 4, 5, 6, 9, 10, 11, 12, 14, 15, 16, 0x80];

/// Returns true if this clipboard format stores a plain HGLOBAL memory block
/// that can be safely copied and restored as raw bytes.
fn is_hglobal_format(format: u32) -> bool {
    !NON_HGLOBAL_FORMATS.contains(&format)
}

/// Platform-specific clipboard snapshot for Windows.
///
/// Contains all format IDs and their raw data from the clipboard at save time.
pub struct ClipboardContents {
    /// All clipboard formats and their raw byte data
    pub formats: Vec<(u32, Vec<u8>)>,
}

pub struct WindowsClipboardPaster;

impl ClipboardPaster for WindowsClipboardPaster {
    fn write_clipboard(&self, text: &str) -> Result<(), String> {
        write_clipboard_text(text)
    }

    fn is_flowstt_foreground(&self) -> bool {
        is_flowstt_foreground_window()
    }

    fn simulate_paste(&self) -> Result<(), String> {
        simulate_ctrl_v()
    }

    fn read_clipboard(&self) -> Option<super::ClipboardContents> {
        read_clipboard_all()
    }

    fn restore_clipboard(&self, contents: &super::ClipboardContents) -> Result<(), String> {
        restore_clipboard_all(contents)
    }
}

/// Write UTF-16 text to the Windows clipboard.
fn write_clipboard_text(text: &str) -> Result<(), String> {
    unsafe {
        // Encode to UTF-16 with null terminator
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let byte_len = wide.len() * std::mem::size_of::<u16>();

        // Allocate moveable global memory
        let hmem = GlobalAlloc(GMEM_MOVEABLE, byte_len)
            .map_err(|e| format!("GlobalAlloc failed: {}", e))?;

        // Copy text into the allocated memory
        let ptr = GlobalLock(hmem);
        if ptr.is_null() {
            return Err("GlobalLock returned null".into());
        }
        std::ptr::copy_nonoverlapping(wide.as_ptr() as *const u8, ptr as *mut u8, byte_len);
        let _ = GlobalUnlock(hmem);

        // Open clipboard, empty it, set our data, close it
        OpenClipboard(None).map_err(|e| format!("OpenClipboard failed: {}", e))?;

        if let Err(e) = EmptyClipboard() {
            let _ = CloseClipboard();
            return Err(format!("EmptyClipboard failed: {}", e));
        }

        // SetClipboardData takes an Option<HANDLE>; HGLOBAL and HANDLE share the same
        // underlying representation (*mut c_void).
        let result = SetClipboardData(CF_UNICODETEXT, Some(HANDLE(hmem.0)));
        let _ = CloseClipboard();

        result.map_err(|e| format!("SetClipboardData failed: {}", e))?;
        Ok(())
    }
}

/// Read all clipboard formats and their data.
///
/// Returns `None` if the clipboard is empty or cannot be opened.
fn read_clipboard_all() -> Option<ClipboardContents> {
    unsafe {
        // Retry opening the clipboard in case it's briefly held by another app.
        let mut opened = false;
        for attempt in 0..5 {
            if attempt > 0 {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            if OpenClipboard(None).is_ok() {
                opened = true;
                break;
            }
        }
        if !opened {
            warn!("[Clipboard] Failed to open clipboard for read");
            return None;
        }

        let mut formats: Vec<(u32, Vec<u8>)> = Vec::new();
        let mut format = 0u32;

        loop {
            format = EnumClipboardFormats(format);
            if format == 0 {
                break;
            }

            // Skip GDI handle formats — they are not HGLOBAL memory blobs
            // and attempting GlobalSize/GlobalLock on them causes heap corruption.
            if !is_hglobal_format(format) {
                continue;
            }

            // Get the clipboard data handle for this format
            let hdata = match GetClipboardData(format) {
                Ok(h) => h,
                Err(_) => continue, // Some formats may not be readable; skip
            };

            if hdata.is_invalid() {
                continue;
            }

            // Lock the global memory to get a pointer
            // SAFETY: hdata is a valid HGLOBAL returned by GetClipboardData.
            // HANDLE and HGLOBAL have the same underlying representation (*mut c_void).
            let hglobal = HGLOBAL(hdata.0 as *mut _);
            let size = GlobalSize(hglobal);
            if size == 0 {
                continue;
            }

            let ptr = GlobalLock(hglobal);
            if ptr.is_null() {
                continue;
            }

            let data = std::slice::from_raw_parts(ptr as *const u8, size).to_vec();
            let _ = GlobalUnlock(hglobal);

            formats.push((format, data));
        }

        let _ = CloseClipboard();

        if formats.is_empty() {
            return None;
        }

        debug!("[Clipboard] Saved {} clipboard format(s)", formats.len());
        Some(ClipboardContents { formats })
    }
}

/// Restore previously saved clipboard contents.
fn restore_clipboard_all(contents: &ClipboardContents) -> Result<(), String> {
    unsafe {
        // The target app may still have the clipboard open immediately after paste.
        // Retry a few times with short sleeps — clipboard contention is transient.
        let mut open_err = String::new();
        let mut opened = false;
        for attempt in 0..10 {
            if attempt > 0 {
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            if OpenClipboard(None).is_ok() {
                opened = true;
                break;
            } else {
                open_err = format!("OpenClipboard failed on attempt {}", attempt + 1);
            }
        }
        if !opened {
            return Err(format!("OpenClipboard failed after retries: {}", open_err));
        }

        if let Err(e) = EmptyClipboard() {
            let _ = CloseClipboard();
            return Err(format!("EmptyClipboard failed during restore: {}", e));
        }

        for (format, data) in &contents.formats {
            let hmem = match GlobalAlloc(GMEM_MOVEABLE, data.len()) {
                Ok(h) => h,
                Err(e) => {
                    warn!(
                        "[Clipboard] GlobalAlloc failed for format {}: {}",
                        format, e
                    );
                    continue;
                }
            };

            let ptr = GlobalLock(hmem);
            if ptr.is_null() {
                warn!("[Clipboard] GlobalLock failed for format {}", format);
                continue;
            }

            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut u8, data.len());
            let _ = GlobalUnlock(hmem);

            if let Err(e) = SetClipboardData(*format, Some(HANDLE(hmem.0))) {
                warn!(
                    "[Clipboard] SetClipboardData failed for format {}: {}",
                    format, e
                );
            }
        }

        let _ = CloseClipboard();
        debug!(
            "[Clipboard] Restored {} clipboard format(s)",
            contents.formats.len()
        );
        Ok(())
    }
}

/// Check if the foreground window belongs to `flowstt-app.exe`.
fn is_flowstt_foreground_window() -> bool {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return false;
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return false;
        }

        // Open the process to query its executable name
        let handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => h,
            Err(_) => return false,
        };

        let mut buf = vec![0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut len,
        );

        let _ = windows::Win32::Foundation::CloseHandle(handle);

        if ok.is_err() || len == 0 {
            return false;
        }

        let exe_path = OsString::from_wide(&buf[..len as usize]);
        let exe_path_str = exe_path.to_string_lossy();

        // Extract the filename component
        let filename = exe_path_str
            .rsplit('\\')
            .next()
            .unwrap_or("")
            .to_lowercase();

        debug!("[Clipboard] Foreground exe: {}", filename);

        filename == "flowstt-app.exe"
    }
}

/// Simulate Ctrl+V by sending four keyboard events via `SendInput`.
fn simulate_ctrl_v() -> Result<(), String> {
    let inputs = [
        // Ctrl down
        make_key_input(VK_CONTROL, false),
        // V down
        make_key_input(VK_V, false),
        // V up
        make_key_input(VK_V, true),
        // Ctrl up
        make_key_input(VK_CONTROL, true),
    ];

    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent != inputs.len() as u32 {
        return Err(format!(
            "SendInput sent {} of {} events",
            sent,
            inputs.len()
        ));
    }
    Ok(())
}

/// Translate a virtual-key code to its hardware scan code.
///
/// Some applications (notably Chrome) ignore `SendInput` events that carry
/// only a virtual-key code (`wVk`) with `wScan == 0`. Populating the scan
/// code makes the synthetic input indistinguishable from a real keystroke.
const MAPVK_VK_TO_VSC: MAP_VIRTUAL_KEY_TYPE = MAP_VIRTUAL_KEY_TYPE(0);

fn vk_to_scan(vk: VIRTUAL_KEY) -> u16 {
    unsafe { MapVirtualKeyW(vk.0 as u32, MAPVK_VK_TO_VSC) as u16 }
}

/// Build an `INPUT` struct for a single keyboard event.
fn make_key_input(vk: VIRTUAL_KEY, key_up: bool) -> INPUT {
    let flags = if key_up {
        KEYEVENTF_KEYUP
    } else {
        KEYBD_EVENT_FLAGS(0)
    };

    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: vk_to_scan(vk),
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}
