# Tasks: Add System Tray Support

## Phase 1: Setup and Infrastructure

- [x] Add `tray-icon` feature to Tauri in `Cargo.toml`
- [x] Add Windows crate dependency for Win32 APIs
- [x] Create `src-tauri/src/tray/mod.rs` with menu IDs and types
- [x] Create `src-tauri/src/tray/windows.rs` with tray setup
- [x] Create tray icon (`src-tauri/icons/tray/icon.png`)
- [x] Update `tauri.conf.json` to bundle tray icons

## Phase 2: Tray Implementation

- [x] Implement `setup_tray()` function
- [x] Implement `load_tray_icon()` with fallback
- [x] Implement `handle_menu_event()` for menu clicks
- [x] Implement `show_main_window()` with recreation logic
- [x] Implement `show_and_focus_window()` with Win32 APIs
- [x] Integrate tray setup in `lib.rs` setup hook

## Phase 3: Window Behavior

- [x] Add close event handler to hide main window to tray
- [x] Handle window recreation when destroyed (Windows quirk)
- [ ] Test close/show cycle works correctly

## Phase 4: About Window

- [x] Create `src/about.html` with layout
- [x] Create `src/about.css` with dark theme styling
- [x] Create `src/about.ts` with close and link handlers
- [x] Implement `show_about_window()` in tray module
- [x] Use 128x128.png from existing icons as About logo
- [ ] Test About window opens, displays correctly, and closes

## Phase 5: Testing

- [ ] Test tray icon appears on app start
- [ ] Test right-click menu appears immediately
- [ ] Test "Show" menu item brings window to foreground
- [ ] Test "About" menu item opens About window
- [ ] Test "Exit" menu item closes application
- [ ] Test double-click tray icon shows window
- [ ] Test close button hides window to tray
- [ ] Test About window styling matches main window
- [ ] Test About window closes properly
- [ ] Test multiple show/hide cycles

## Phase 6: Polish

- [ ] Verify icon looks correct at all DPI scales
- [x] Add tooltip text to tray icon
- [ ] Ensure no memory leaks with window recreation
- [ ] Update dev journal with implementation notes

## Implementation Notes

### Additional Changes Made

1. Added `image-png` feature to Tauri for `Image::from_path` support
2. Updated Windows crate to version 0.61 to match Tauri's dependency
3. Added about.html to Vite build configuration (`vite.config.ts`)
4. Used existing 128x128.png as About window logo instead of creating new SVG
5. Used `window.open()` for external links instead of shell plugin

### Files Created/Modified

**New files:**
- `src-tauri/src/tray/mod.rs` - Cross-platform tray module interface
- `src-tauri/src/tray/windows.rs` - Windows tray implementation
- `src-tauri/icons/tray/icon.png` - Tray icon (copied from 32x32.png)
- `about.html` - About window HTML
- `src/about.css` - About window styles
- `src/about.ts` - About window TypeScript

**Modified files:**
- `src-tauri/Cargo.toml` - Added tray-icon, image-png features; windows 0.61
- `src-tauri/tauri.conf.json` - Added resources bundle for tray icons
- `src-tauri/src/lib.rs` - Added tray module, setup hook, close handler
- `vite.config.ts` - Added about.html to build inputs
