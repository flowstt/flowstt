# Design: System Tray Support

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Tauri App (src-tauri)                    │
├─────────────────────────────────────────────────────────────┤
│  lib.rs                                                      │
│  ├── setup() - Initialize tray                               │
│  ├── on_window_event() - Handle close → hide                 │
│  └── on_menu_event() - Handle tray menu (macOS)              │
├─────────────────────────────────────────────────────────────┤
│  tray/                                                       │
│  ├── mod.rs - Cross-platform interface, menu IDs             │
│  └── windows.rs - Windows tray setup & event handling        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     Windows System Tray                      │
├─────────────────────────────────────────────────────────────┤
│  Icon: FlowSTT logo (32x32 PNG)                              │
│  Tooltip: "FlowSTT"                                          │
│  Menu:                                                       │
│    ├── Show                                                  │
│    ├── About                                                 │
│    ├── ─────────                                             │
│    └── Exit                                                  │
└─────────────────────────────────────────────────────────────┘
```

## Component Details

### 1. Tray Module (`src-tauri/src/tray/`)

#### mod.rs
```rust
//! System tray support for FlowSTT.
//!
//! Platform-specific implementations:
//! - Windows: windows.rs

#[cfg(windows)]
pub mod windows;

/// Menu item identifiers.
pub mod menu_ids {
    pub const SHOW: &str = "show";
    pub const ABOUT: &str = "about";
    pub const EXIT: &str = "exit";
}

/// Menu item labels.
pub mod menu_labels {
    pub const SHOW: &str = "Show";
    pub const ABOUT: &str = "About";
    pub const EXIT: &str = "Exit";
}

/// Tray state managed by Tauri.
pub struct TrayState {
    pub tray: std::sync::Mutex<tauri::tray::TrayIcon>,
}

/// Platform-specific tray setup.
#[cfg(windows)]
pub use windows::setup_tray;
```

#### windows.rs
```rust
//! Windows system tray implementation.

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager, WebviewUrl, WebviewWindow,
};
use windows::Win32::UI::WindowsAndMessaging::{
    SetForegroundWindow, ShowWindow, SW_RESTORE, SW_SHOW,
};

use super::{menu_ids, menu_labels, TrayState};

/// Set up the system tray on Windows.
pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let icon = load_tray_icon(app);

    // Create menu items
    let show_item = MenuItem::with_id(
        app, menu_ids::SHOW, menu_labels::SHOW, true, None::<&str>
    )?;
    let about_item = MenuItem::with_id(
        app, menu_ids::ABOUT, menu_labels::ABOUT, true, None::<&str>
    )?;
    let exit_item = MenuItem::with_id(
        app, menu_ids::EXIT, menu_labels::EXIT, true, None::<&str>
    )?;

    // Build menu
    let menu = Menu::with_items(app, &[
        &show_item,
        &about_item,
        &PredefinedMenuItem::separator(app)?,
        &exit_item,
    ])?;

    // Build tray icon
    let tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("FlowSTT")
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::DoubleClick { .. } = event {
                show_main_window(tray.app_handle(), None);
            }
        })
        .on_menu_event(|app, event| {
            handle_menu_event(app, &event);
        })
        .build(app)?;

    // Store tray state
    app.manage(TrayState {
        tray: std::sync::Mutex::new(tray),
    });

    Ok(())
}

/// Handle menu item clicks.
fn handle_menu_event(app: &tauri::AppHandle, event: &tauri::menu::MenuEvent) {
    match event.id.as_ref() {
        id if id == menu_ids::SHOW => {
            show_main_window(app, None);
        }
        id if id == menu_ids::ABOUT => {
            show_about_window(app);
        }
        id if id == menu_ids::EXIT => {
            app.exit(0);
        }
        _ => {}
    }
}

/// Show the main window, recreating if necessary.
fn show_main_window(app: &tauri::AppHandle, initial_view: Option<&str>) {
    if let Some(window) = app.get_webview_window("main") {
        show_and_focus_window(&window);
    } else {
        // Window was destroyed, recreate it
        recreate_main_window(app, initial_view);
    }
}

/// Show and focus a window using Win32 APIs.
fn show_and_focus_window(window: &WebviewWindow) {
    let _ = window.show();
    let _ = window.unminimize();

    if let Ok(hwnd) = window.hwnd() {
        unsafe {
            let _ = ShowWindow(hwnd, SW_RESTORE);
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = SetForegroundWindow(hwnd);
        }
    }

    let _ = window.set_focus();
}

/// Show the About window.
fn show_about_window(app: &tauri::AppHandle) {
    // Check if already open
    if let Some(window) = app.get_webview_window("about") {
        let _ = window.set_focus();
        return;
    }

    // Create About window
    let _ = tauri::WebviewWindowBuilder::new(
        app,
        "about",
        WebviewUrl::App("about.html".into())
    )
    .title("About FlowSTT")
    .inner_size(400.0, 280.0)
    .resizable(false)
    .maximizable(false)
    .minimizable(false)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .skip_taskbar(true)
    .center()
    .build();
}
```

### 2. About Window

#### about.html
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>About FlowSTT</title>
    <link rel="stylesheet" href="about.css">
</head>
<body>
    <div class="about-container">
        <div class="about-header">
            <button class="close-btn" id="close-btn" title="Close">
                <svg><!-- X icon --></svg>
            </button>
        </div>
        <div class="about-content">
            <img src="assets/flowstt-logo.png" alt="FlowSTT" class="about-logo">
            <h1 class="about-title">FlowSTT</h1>
            <p class="about-version" id="about-version">Version loading...</p>
            <p class="about-description">
                A voice transcription agent for fluid, natural conversation.
            </p>
            <div class="about-links">
                <a href="#" id="link-website" class="about-link">Website</a>
                <a href="#" id="link-github" class="about-link">GitHub</a>
            </div>
            <div class="about-legal">
                <p class="about-copyright">Copyright 2024-2025 Keath Milligan</p>
                <p class="about-license">
                    Released under the <a href="#" id="link-license">MIT License</a>
                </p>
            </div>
        </div>
    </div>
    <script type="module" src="about.ts"></script>
</body>
</html>
```

#### about.css
```css
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

html, body {
    height: 100%;
    background: transparent;
    overflow: hidden;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

.about-container {
    height: 100%;
    border-radius: 12px;
    overflow: hidden;
    background: linear-gradient(180deg, #1a1a2e 0%, #16213e 100%);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(255, 255, 255, 0.1);
    display: flex;
    flex-direction: column;
}

.about-header {
    height: 40px;
    display: flex;
    justify-content: flex-end;
    align-items: center;
    padding: 0 12px;
    -webkit-app-region: drag;
}

.close-btn {
    width: 28px;
    height: 28px;
    border: none;
    background: transparent;
    color: #888;
    cursor: pointer;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    -webkit-app-region: no-drag;
    transition: all 0.2s;
}

.close-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
}

.about-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 0 32px 32px;
    color: #fff;
}

.about-logo {
    width: 80px;
    height: 80px;
    margin-bottom: 16px;
}

.about-title {
    font-size: 24px;
    font-weight: 600;
    margin-bottom: 4px;
}

.about-version {
    font-size: 14px;
    color: #888;
    margin-bottom: 16px;
}

.about-description {
    font-size: 14px;
    color: #aaa;
    text-align: center;
    margin-bottom: 20px;
}

.about-links {
    display: flex;
    gap: 24px;
    margin-bottom: 20px;
}

.about-link {
    color: #6c9bff;
    text-decoration: none;
    font-size: 14px;
}

.about-link:hover {
    text-decoration: underline;
}

.about-legal {
    text-align: center;
    color: #666;
    font-size: 12px;
}

.about-legal a {
    color: #6c9bff;
    text-decoration: none;
}
```

#### about.ts
```typescript
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-shell";
import { getVersion } from "@tauri-apps/api/app";

const WEBSITE_URL = "https://github.com/keathmilligan/flowstt";
const GITHUB_URL = "https://github.com/keathmilligan/flowstt";
const LICENSE_URL = "https://github.com/keathmilligan/flowstt/blob/main/LICENSE";

document.addEventListener("DOMContentLoaded", async () => {
    // Set version
    const version = await getVersion();
    document.getElementById("about-version")!.textContent = `Version ${version}`;
    
    // Close button
    document.getElementById("close-btn")!.addEventListener("click", async () => {
        await getCurrentWindow().close();
    });
    
    // Links
    document.getElementById("link-website")!.addEventListener("click", (e) => {
        e.preventDefault();
        open(WEBSITE_URL);
    });
    
    document.getElementById("link-github")!.addEventListener("click", (e) => {
        e.preventDefault();
        open(GITHUB_URL);
    });
    
    document.getElementById("link-license")!.addEventListener("click", (e) => {
        e.preventDefault();
        open(LICENSE_URL);
    });
});
```

### 3. Main Window Close Handling

In `lib.rs`:
```rust
.on_window_event(|window, event| {
    #[cfg(windows)]
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        if window.label() == "main" {
            // Hide to tray instead of closing
            api.prevent_close();
            let _ = window.hide();
        }
        // About window closes normally (no prevent_close)
    }
})
```

### 4. Icon Loading Strategy

```rust
fn load_tray_icon(app: &tauri::App) -> tauri::image::Image<'static> {
    let resource_dir = app.path().resource_dir().ok();
    
    // Try loading from bundled resources first
    if let Some(ref dir) = resource_dir {
        let icon_path = dir.join("icons/tray/icon.png");
        if icon_path.exists() {
            if let Ok(bytes) = std::fs::read(&icon_path) {
                if let Ok(img) = tauri::image::Image::from_bytes(&bytes) {
                    return img;
                }
            }
        }
    }
    
    // Try development paths
    let dev_paths = [
        "src-tauri/icons/tray/icon.png",
        "icons/tray/icon.png",
    ];
    
    for path in &dev_paths {
        if let Ok(bytes) = std::fs::read(path) {
            if let Ok(img) = tauri::image::Image::from_bytes(&bytes) {
                return img;
            }
        }
    }
    
    // Fallback: create a simple colored icon
    create_fallback_icon()
}

fn create_fallback_icon() -> tauri::image::Image<'static> {
    let size = 32u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    
    // Draw a blue circle
    let center = size as f32 / 2.0;
    let radius = center - 2.0;
    
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let dist = (dx * dx + dy * dy).sqrt();
            
            let idx = ((y * size + x) * 4) as usize;
            if dist <= radius {
                rgba[idx] = 0x6c;     // R
                rgba[idx + 1] = 0x9b; // G
                rgba[idx + 2] = 0xff; // B
                rgba[idx + 3] = 0xff; // A
            }
        }
    }
    
    tauri::image::Image::new_owned(rgba, size, size)
}
```

## File Structure

```
src-tauri/
├── src/
│   ├── lib.rs              # Modified: tray setup, window events
│   └── tray/
│       ├── mod.rs          # New: cross-platform interface
│       └── windows.rs      # New: Windows implementation
├── icons/
│   └── tray/
│       └── icon.png        # New: 32x32 tray icon
├── Cargo.toml              # Modified: add tray-icon feature
└── tauri.conf.json         # Modified: bundle tray icons

src/
├── about.html              # New: About window HTML
├── about.css               # New: About window styles
└── about.ts                # New: About window logic
```

## Configuration Changes

### tauri.conf.json
```json
{
  "bundle": {
    "resources": {
      "icons/tray/*": "icons/tray/"
    }
  },
  "app": {
    "windows": [
      {
        "label": "main",
        ...
      }
    ]
  }
}
```

### Cargo.toml
```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }

[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
    "Win32_UI_WindowsAndMessaging"
] }
```
