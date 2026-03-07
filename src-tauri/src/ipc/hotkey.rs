//! Platform-specific hotkey / accessibility permission helpers.

/// Check whether this process has macOS Accessibility permission.
/// Returns `true` on non-macOS platforms (permission not applicable).
#[cfg(target_os = "macos")]
pub fn check_accessibility_permission() -> bool {
    // AXIsProcessTrusted() returns whether the current process has accessibility access.
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
    // Open the Accessibility section of System Settings so the user can grant access.
    let _ = std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn();
}

#[cfg(not(target_os = "macos"))]
pub fn request_accessibility_permission() {}
