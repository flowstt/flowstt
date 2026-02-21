//! System tray support for FlowSTT.
//!
//! Platform-specific implementations:
//! - Windows: windows.rs
//! - macOS: macos.rs

#[cfg(windows)]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

/// Menu item identifiers.
#[allow(dead_code)]
pub mod menu_ids {
    pub const SHOW: &str = "show";
    pub const SETTINGS: &str = "settings";
    pub const ABOUT: &str = "about";
    pub const EXIT: &str = "exit";
}

/// Menu item labels.
#[allow(dead_code)]
pub mod menu_labels {
    pub const SHOW: &str = "Show";
    pub const SETTINGS: &str = "Settings";
    pub const ABOUT: &str = "About";
    pub const EXIT: &str = "Exit";
}

/// Platform-specific tray setup.
#[cfg(windows)]
pub use windows::setup_tray;

#[cfg(target_os = "macos")]
pub use macos::setup_tray;

/// Linux tray - no-op for now.
#[cfg(not(any(windows, target_os = "macos")))]
pub fn setup_tray(_app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

/// Send a shutdown request to the service (best-effort).
/// Used by the tray Exit handler to stop the service before exiting the app.
/// Errors are silently ignored — the service may already be stopped.
fn shutdown_service() {
    use flowstt_common::ipc::{get_socket_path, read_json, write_json, Request, Response};

    let socket_path = get_socket_path();

    // Use block_on to run the async IPC call from the synchronous menu handler
    let _ = tauri::async_runtime::block_on(async {
        #[cfg(unix)]
        {
            let stream = tokio::net::UnixStream::connect(&socket_path).await?;
            let (mut reader, mut writer) = stream.into_split();
            write_json(&mut writer, &Request::Shutdown).await?;
            let _response: Response = read_json(&mut reader).await?;
            Ok::<(), flowstt_common::ipc::IpcError>(())
        }
        #[cfg(windows)]
        {
            use tokio::net::windows::named_pipe::ClientOptions;
            let pipe = ClientOptions::new()
                .open(&socket_path)
                .map_err(flowstt_common::ipc::IpcError::Io)?;
            let (mut reader, mut writer) = tokio::io::split(pipe);
            write_json(&mut writer, &Request::Shutdown).await?;
            let _response: Response = read_json(&mut reader).await?;
            Ok::<(), flowstt_common::ipc::IpcError>(())
        }
    });
}
