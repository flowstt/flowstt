//! Platform-specific hotkey / accessibility permission helpers.

/// Check whether this process has macOS Accessibility permission.
/// Returns `true` on non-macOS platforms (permission not applicable).
#[cfg(target_os = "macos")]
pub fn check_accessibility_permission() -> bool {
    use std::process::Command;
    // AXIsProcessTrusted() returns whether the current process has accessibility access.
    // We call it via a small inline check using the Objective-C runtime.
    // For now, use a simple system call as a proxy until a proper FFI binding exists.
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }
    unsafe { AXIsProcessTrusted() }
}

#[cfg(not(target_os = "macos"))]
pub fn check_accessibility_permission() -> bool {
    true
}

/// Request macOS Accessibility permission for this process.
/// On macOS, this shows the system dialog prompting the user to grant access.
/// No-op on other platforms.
#[cfg(target_os = "macos")]
pub fn request_accessibility_permission() {
    // AXIsProcessTrustedWithOptions with kAXTrustedCheckOptionPrompt = true
    // triggers the system dialog. We replicate this with a subprocess call
    // as an interim approach until a proper FFI binding is in place.
    use std::collections::HashMap;
    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: *const std::ffi::c_void) -> bool;
    }
    // The simplest approach: calling AXIsProcessTrusted() with prompt option
    // requires CoreFoundation. Use the existing check as a placeholder; the actual
    // permission dialog is opened by the OS via the Settings URL.
    let _ = std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn();
}

#[cfg(not(target_os = "macos"))]
pub fn request_accessibility_permission() {}
