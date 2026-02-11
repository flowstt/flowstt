# Change Proposal: Add System Tray Support

## Summary

Add system tray support to the FlowSTT Tauri application (not the service), providing a persistent presence in the system notification area with quick access to core functionality.

## Motivation

Users need a way to:
- Access FlowSTT quickly without searching for the window
- See at-a-glance status (idle, recording, processing)
- Minimize the app to the tray to reduce taskbar clutter
- Access common actions (show window, about, exit) from the tray

## Scope

### In Scope
- Windows system tray icon with context menu
- Tray icon state changes to reflect recording status
- "Show" menu item to bring window to foreground
- "About" menu item to show About popup window
- "Exit" menu item to quit the application
- Hide to tray on window close (instead of exit)
- About popup window with app info, version, and links

### Out of Scope
- macOS menu bar support (future enhancement)
- Linux system tray support (future enhancement)
- Tray-based recording controls (use main window)

## Approach

Implement system tray in the Tauri app (`src-tauri`) following the pattern established in OmniRec:

1. **Tray Module Structure**
   - Create `src-tauri/src/tray/mod.rs` with cross-platform interface
   - Create `src-tauri/src/tray/windows.rs` with Windows implementation
   - Use Tauri's built-in `TrayIconBuilder` and menu APIs

2. **Menu Items**
   - Show - Brings main window to foreground
   - About - Opens About popup window
   - Separator
   - Exit - Quits the application

3. **Icon States**
   - Normal: Standard FlowSTT icon
   - Recording: Icon with red indicator (future, when recording is added)

4. **Window Behavior**
   - Close button hides window to tray (doesn't exit)
   - Double-click tray icon shows window
   - Window may be destroyed on hide (Windows quirk with transparent windows)
   - Recreate window if destroyed when user requests show

5. **About Window**
   - Separate popup window (not a tab in main window)
   - Same visual style as main window (dark theme, rounded corners, transparent)
   - Contains: Logo, app name, version, description, links (website, GitHub)
   - Custom close button (no native title bar)
   - Does not appear in taskbar (`skipTaskbar: true`)

## Implementation Details

### Tray Setup (Windows)

```rust
pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let icon = load_tray_icon(app);
    
    let show_item = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let about_item = MenuItem::with_id(app, "about", "About", true, None::<&str>)?;
    let exit_item = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;
    
    let menu = Menu::with_items(app, &[
        &show_item,
        &about_item,
        &PredefinedMenuItem::separator(app)?,
        &exit_item,
    ])?;
    
    let tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("FlowSTT")
        .on_tray_icon_event(handle_tray_event)
        .on_menu_event(handle_menu_event)
        .build(app)?;
    
    app.manage(TrayState { tray: Mutex::new(tray) });
    Ok(())
}
```

### About Window Creation

```rust
fn show_about_window(app: &tauri::AppHandle) {
    // Check if already open
    if let Some(window) = app.get_webview_window("about") {
        let _ = window.set_focus();
        return;
    }
    
    // Create new about window
    let _ = tauri::WebviewWindowBuilder::new(
        app,
        "about",
        WebviewUrl::App("about.html".into())
    )
    .title("About FlowSTT")
    .inner_size(400.0, 300.0)
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

### Window Close Handling

```rust
// In lib.rs on_window_event
if let tauri::WindowEvent::CloseRequested { api, .. } = event {
    if window.label() == "main" {
        api.prevent_close();
        let _ = window.hide();
    }
    // About window can close normally
}
```

## Dependencies

### Cargo.toml Additions
```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }

[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
    "Win32_UI_WindowsAndMessaging"
] }
```

### New Files
- `src-tauri/src/tray/mod.rs`
- `src-tauri/src/tray/windows.rs`
- `src-tauri/icons/tray/icon.png` (32x32)
- `src/about.html`
- `src/about.css`
- `src/about.ts`

### Modified Files
- `src-tauri/src/lib.rs` - Tray setup and window event handling
- `src-tauri/Cargo.toml` - Dependencies
- `src-tauri/tauri.conf.json` - Tray icon resources

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Window destroyed on hide (Windows quirk) | Recreate window when needed, use URL params for state |
| Menu not appearing (previous issue) | Use Tauri's built-in menu API, not native TrackPopupMenu |
| Icon not loading | Fallback icon generation, multiple path checks |

## Testing

1. Right-click tray icon - menu appears immediately
2. Click "Show" - window appears and is focused
3. Click "About" - About window appears centered
4. Click "Exit" - application exits completely
5. Close main window (X button) - window hides, tray icon remains
6. Double-click tray icon - window appears
7. Open About, close it, open again - works correctly

## References

- OmniRec tray implementation: `../omnirec/omnirec/src-tauri/src/tray/`
- Tauri tray documentation: https://v2.tauri.app/learn/system-tray/
- Dev notes: `Dev/Tauri Development.md` (Obsidian)
