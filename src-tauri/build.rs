fn main() {
    // Ensure the release DLL directories satisfy the Tauri resource globs on Windows.
    //
    // `tauri.windows.conf.json` declares resource globs for `../target/release/cuda/*.dll`
    // and `../target/release/cpu/*.dll` for bundling whisper.cpp libraries. `tauri_build::build()`
    // validates that these globs resolve to at least one file and aborts if they don't.
    //
    // The whisper DLLs are downloaded and placed there by the `flowstt-engine` build
    // script, which also runs during debug builds specifically for this purpose. However,
    // Cargo may execute *this* build script before the engine's build script when both
    // crates need to be built from scratch (e.g. after `cargo clean`). In that case the
    // glob validation fails before the DLLs exist.
    //
    // Work around the race by creating temporary placeholder DLLs when the directories
    // are empty. The engine build script overwrites them with the real libraries once it finishes.
    #[cfg(target_os = "windows")]
    ensure_release_dll_placeholders();

    tauri_build::build();
}

/// Create placeholder files in `target/release/cuda/` and `target/release/cpu/`
/// so the Tauri resource globs match at least one entry each.
#[cfg(target_os = "windows")]
fn ensure_release_dll_placeholders() {
    use std::path::PathBuf;

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap_or_default());
    let target_dir = out_dir
        .ancestors()
        .find(|p| p.file_name().map(|n| n == "target").unwrap_or(false))
        .map(|p| p.to_path_buf());

    let Some(target_dir) = target_dir else {
        return;
    };

    let release_dir = target_dir.join("release");
    for subdir in &["cuda", "cpu"] {
        let dir = release_dir.join(subdir);
        let _ = std::fs::create_dir_all(&dir);

        // If there are already real DLLs, nothing to do for this subdir.
        let has_dlls = std::fs::read_dir(&dir)
            .map(|entries| {
                entries.flatten().any(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "dll")
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

        if !has_dlls {
            // Create a zero-byte placeholder; the engine build script will replace it.
            let placeholder = dir.join(".tauri-placeholder.dll");
            let _ = std::fs::File::create(&placeholder);
        }
    }
}
