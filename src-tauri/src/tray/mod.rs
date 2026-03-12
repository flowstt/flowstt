//! System tray support for FlowSTT.
//!
//! Platform-specific implementations:
//! - Windows: windows.rs
//! - macOS: macos.rs

#[cfg(not(target_os = "macos"))]
use std::path::PathBuf;
#[cfg(not(target_os = "macos"))]
use tauri::image::Image;
use tauri::Manager;
use tracing::warn;

#[cfg(windows)]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

/// Menu item identifiers.
#[allow(dead_code)]
pub mod menu_ids {
    pub const SHOW: &str = "show";
    pub const ALWAYS_ON_TOP: &str = "always_on_top";
    pub const SETTINGS: &str = "settings";
    pub const LOGS: &str = "logs";
    pub const ABOUT: &str = "about";
    pub const CHECK_FOR_UPDATES: &str = "check_for_updates";
    pub const RUN_TEST: &str = "run_test";
    pub const EXIT: &str = "exit";
}

/// Menu item labels.
#[allow(dead_code)]
pub mod menu_labels {
    pub const SHOW: &str = "Show";
    pub const ALWAYS_ON_TOP: &str = "Always on Top";
    pub const SETTINGS: &str = "Settings";
    pub const LOGS: &str = "Logs";
    pub const ABOUT: &str = "About";
    pub const CHECK_FOR_UPDATES: &str = "Check for Updates";
    pub const RUN_TEST: &str = "Run Test (WAV Directory)...";
    pub const EXIT: &str = "Exit";
}

/// Platform-specific tray setup.
#[cfg(windows)]
pub use windows::setup_tray;

#[cfg(windows)]
pub use windows::show_main_window;

#[cfg(target_os = "macos")]
pub use macos::setup_tray;

#[cfg(target_os = "macos")]
pub use macos::show_main_window;

/// Linux tray - no-op for now.
#[cfg(not(any(windows, target_os = "macos")))]
pub fn setup_tray(_app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

/// Show the main window on Linux (simplified implementation).
#[cfg(not(any(windows, target_os = "macos")))]
pub fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Clean up IPC socket file on Unix before exiting.
fn shutdown_engine() {
    // The AudioEngine shuts down via Drop when the Tauri managed state is released.
    // On Unix, we clean up the socket file here for CLI clients.
    #[cfg(unix)]
    {
        let socket_path = flowstt_common::ipc::get_socket_path();
        if socket_path.exists() {
            let _ = std::fs::remove_file(&socket_path);
        }
    }
}

/// Update the tray icon to reflect the current recording state.
///
/// When `recording` is `true` the tray shows `icon-recording.png`; when
/// `false` it reverts to `icon.png` (the default tray icon).
pub fn update_tray_icon(app_handle: &tauri::AppHandle, recording: bool) {
    let icon_name = if recording {
        "icon-recording.png"
    } else {
        "icon.png"
    };

    let resource_dir = app_handle.path().resource_dir().ok();

    // On macOS, prefer the platform-specific loader which checks
    // icons/tray/macos/ first (monochrome template icons) before falling
    // back to the generic icons/tray/ directory.
    #[cfg(target_os = "macos")]
    let icon = macos::load_tray_icon_from_paths(resource_dir, icon_name).or_else(|| {
        if recording {
            None
        } else {
            let resource_dir2 = app_handle.path().resource_dir().ok();
            macos::load_tray_icon_from_paths(resource_dir2, "32x32.png")
        }
    });

    #[cfg(not(target_os = "macos"))]
    let icon = load_tray_icon_from_paths(resource_dir, icon_name).or_else(|| {
        if recording {
            // No dedicated recording icon found – keep current icon unchanged.
            None
        } else {
            // Fallback: try 32x32.png for the default icon.
            let resource_dir2 = app_handle.path().resource_dir().ok();
            load_tray_icon_from_paths(resource_dir2, "32x32.png")
        }
    });

    if let Some(icon) = icon {
        if let Some(tray) = app_handle.tray_by_id("main-tray") {
            // On macOS, set the template flag before swapping the icon to avoid
            // a race condition in the underlying tray-icon crate where a separate
            // set_icon_as_template call can arrive out of order.
            #[cfg(target_os = "macos")]
            if let Err(e) = tray.set_icon_as_template(true) {
                warn!("[Tray] Failed to set icon as template: {}", e);
            }
            if let Err(e) = tray.set_icon(Some(icon)) {
                warn!("[Tray] Failed to update tray icon: {}", e);
            }
        }
    } else if recording {
        warn!("[Tray] Recording icon not found – tray icon unchanged");
    }
}

/// Load a tray icon image by searching several candidate paths.
///
/// Checks bundled resource paths first (production), then relative and
/// absolute development paths.
#[cfg(not(target_os = "macos"))]
fn load_tray_icon_from_paths(
    resource_dir: Option<PathBuf>,
    icon_name: &str,
) -> Option<Image<'static>> {
    let resource_dir_clone = resource_dir.clone();
    let icon_paths = [
        resource_dir.map(|p| p.join(format!("icons/tray/{}", icon_name))),
        resource_dir_clone.map(|p| p.join(format!("icons/{}", icon_name))),
        Some(PathBuf::from(format!("icons/tray/{}", icon_name))),
        Some(PathBuf::from(format!("src-tauri/icons/tray/{}", icon_name))),
        Some(PathBuf::from(format!("icons/{}", icon_name))),
        Some(PathBuf::from(format!("src-tauri/icons/{}", icon_name))),
        Some(PathBuf::from(format!(
            "{}/icons/tray/{}",
            env!("CARGO_MANIFEST_DIR"),
            icon_name
        ))),
        Some(PathBuf::from(format!(
            "{}/icons/{}",
            env!("CARGO_MANIFEST_DIR"),
            icon_name
        ))),
    ];

    for path in icon_paths.iter().flatten() {
        if path.exists() {
            match Image::from_path(path) {
                Ok(img) => return Some(img.to_owned()),
                Err(e) => {
                    warn!("[Tray] Failed to load icon from {:?}: {}", path, e);
                }
            }
        }
    }
    None
}
