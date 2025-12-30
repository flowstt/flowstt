use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const WHISPER_VERSION: &str = "1.8.2";
const GITHUB_RELEASE_BASE: &str = "https://github.com/ggml-org/whisper.cpp/releases/download";

fn main() {
    tauri_build::build();

    // Only download binaries on Windows and macOS
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    
    // Check if CUDA feature is enabled (set by Cargo when --features cuda is used)
    let cuda_enabled = env::var("CARGO_FEATURE_CUDA").is_ok();
    
    // macOS: Link ScreenCaptureKit framework for system audio capture
    if target_os == "macos" {
        println!("cargo:rustc-link-lib=framework=ScreenCaptureKit");
        println!("cargo:rustc-link-lib=framework=CoreMedia");
        println!("cargo:rustc-link-lib=framework=AVFoundation");
    }
    
    if target_os == "linux" {
        println!("cargo:warning=Linux build: using whisper-rs crate (builds from source)");
        if cuda_enabled {
            println!("cargo:warning=CUDA feature enabled - whisper-rs will be built with CUDA support");
        }
        return;
    }

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));

    // Determine which binary to download and which libraries to extract
    // When CUDA feature is enabled on Windows x64, use CUDA-enabled binaries
    let (zip_name, lib_names): (&str, Vec<&str>) = match (target_os.as_str(), target_arch.as_str()) {
        ("windows", "x86_64") if cuda_enabled => {
            println!("cargo:warning=CUDA feature enabled - using CUDA-accelerated whisper.cpp binaries");
            (
                "whisper-cublas-12.4.0-bin-x64.zip",
                vec![
                    "whisper.dll",
                    "ggml.dll",
                    "ggml-base.dll",
                    "ggml-cpu.dll",
                    "ggml-cuda.dll",
                    // CUDA runtime libraries
                    "cublas64_12.dll",
                    "cublasLt64_12.dll",
                    "cudart64_12.dll",
                    // Additional CUDA libraries required for GPU detection
                    "nvrtc64_120_0.dll",
                    "nvrtc-builtins64_124.dll",
                    "nvblas64_12.dll",
                ],
            )
        }
        ("windows", "x86_64") => (
            "whisper-bin-x64.zip",
            vec!["whisper.dll", "ggml.dll", "ggml-base.dll", "ggml-cpu.dll"],
        ),
        ("windows", "x86") => {
            if cuda_enabled {
                println!("cargo:warning=CUDA feature is not supported on Windows x86 - using CPU-only build");
            }
            (
                "whisper-bin-Win32.zip",
                vec!["whisper.dll", "ggml.dll", "ggml-base.dll", "ggml-cpu.dll"],
            )
        }
        ("macos", _) => {
            if cuda_enabled {
                println!("cargo:warning=CUDA feature has no effect on macOS - using Metal acceleration via prebuilt framework");
            }
            (
                &format!("whisper-v{}-xcframework.zip", WHISPER_VERSION) as &str,
                vec!["libwhisper.dylib"],
            )
        }
        _ => {
            println!("cargo:warning=Unsupported platform: {}-{}", target_os, target_arch);
            return;
        }
    };
    let primary_lib = lib_names[0];

    // Cache directory for downloaded files
    let cache_dir = out_dir.join("whisper-cache");
    fs::create_dir_all(&cache_dir).expect("Failed to create cache directory");

    // Include cuda in cache path to separate CUDA and non-CUDA builds
    let cuda_suffix = if cuda_enabled { "-cuda" } else { "" };
    let zip_path = cache_dir.join(format!("whisper-{}-{}{}.zip", WHISPER_VERSION, target_arch, cuda_suffix));
    // Separate output directories for CUDA and non-CUDA to avoid mixing libraries
    let lib_output_dir = out_dir.join(format!("whisper-lib{}", cuda_suffix));
    fs::create_dir_all(&lib_output_dir).expect("Failed to create lib output directory");

    let primary_lib_path = lib_output_dir.join(primary_lib);

    // Check if we already have the primary library
    if !primary_lib_path.exists() {
        // Download if not cached
        if !zip_path.exists() {
            let url = format!("{}/v{}/{}", GITHUB_RELEASE_BASE, WHISPER_VERSION, zip_name);
            println!("cargo:warning=Downloading whisper.cpp binary from: {}", url);
            download_file(&url, &zip_path).expect("Failed to download whisper.cpp binary");
        }

        // Extract all required libraries
        println!("cargo:warning=Extracting whisper.cpp libraries...");
        extract_libraries(&zip_path, &lib_output_dir, &lib_names, &target_os, &target_arch)
            .expect("Failed to extract whisper.cpp libraries");
    }

    // Set linker flags
    println!("cargo:rustc-link-search=native={}", lib_output_dir.display());

    // For Windows, we need to tell the linker about the import library
    if target_os == "windows" {
        // The DLL doesn't have an import lib in the release, so we use runtime loading
        // Just ensure the DLL can be found at runtime
    }

    // Copy all libraries to target directory for runtime
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let target_dir = out_dir
        .ancestors()
        .find(|p| p.ends_with("target") || p.file_name().map(|n| n == "target").unwrap_or(false))
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| out_dir.join("..").join("..").join(".."));

    let runtime_lib_dir = target_dir.join(&profile);
    if runtime_lib_dir.exists() {
        for lib_name in &lib_names {
            let lib_path = lib_output_dir.join(lib_name);
            let runtime_lib_path = runtime_lib_dir.join(lib_name);
            if lib_path.exists()
                && (!runtime_lib_path.exists()
                    || fs::metadata(&lib_path).map(|m| m.len()).unwrap_or(0)
                        != fs::metadata(&runtime_lib_path).map(|m| m.len()).unwrap_or(0))
            {
                fs::copy(&lib_path, &runtime_lib_path).ok();
                println!(
                    "cargo:warning=Copied {} to {}",
                    lib_name,
                    runtime_lib_dir.display()
                );
            }
        }
    }

    // Also write the primary library path to a file for runtime discovery
    let lib_path_file = out_dir.join("whisper_lib_path.txt");
    fs::write(&lib_path_file, primary_lib_path.to_string_lossy().as_bytes())
        .expect("Failed to write library path file");

    // Rerun if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
}

fn download_file(url: &str, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::blocking::Client::builder()
        .user_agent("flowstt-build")
        .build()?
        .get(url)
        .send()?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {} for URL: {}", response.status(), url).into());
    }

    let bytes = response.bytes()?;
    let mut file = fs::File::create(dest)?;
    file.write_all(&bytes)?;

    Ok(())
}

fn extract_libraries(
    zip_path: &Path,
    output_dir: &Path,
    lib_names: &[&str],
    target_os: &str,
    _target_arch: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    if target_os == "macos" {
        // xcframework structure: the macOS binary is in macos-arm64_x86_64 (universal binary)
        // The framework contains the binary at whisper.framework/Versions/A/whisper
        let lib_name = lib_names[0]; // macOS only has one library (libwhisper.dylib)

        // First try: look for the macos universal binary framework
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            // Look for the framework binary in the macos folder
            // Path: build-apple/whisper.xcframework/macos-arm64_x86_64/whisper.framework/Versions/A/whisper
            if name.contains("macos-arm64_x86_64")
                && name.contains("whisper.framework/Versions/A/whisper")
                && !name.ends_with("/")
            {
                let output_path = output_dir.join(lib_name);
                let mut output_file = fs::File::create(&output_path)?;
                io::copy(&mut file, &mut output_file)?;
                println!(
                    "cargo:warning=Extracted {} from {} (framework binary)",
                    lib_name, name
                );
                return Ok(());
            }
        }

        // Fallback: look for any dylib
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            if name.ends_with(".dylib") && !name.contains("ios") {
                let output_path = output_dir.join(lib_name);
                let mut output_file = fs::File::create(&output_path)?;
                io::copy(&mut file, &mut output_file)?;
                println!(
                    "cargo:warning=Extracted {} from {} (fallback dylib)",
                    lib_name, name
                );
                return Ok(());
            }
        }

        // Second fallback: look for any macos whisper binary (not a directory)
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            if name.contains("macos") && name.ends_with("/whisper") && file.size() > 0 {
                let output_path = output_dir.join(lib_name);
                let mut output_file = fs::File::create(&output_path)?;
                io::copy(&mut file, &mut output_file)?;
                println!(
                    "cargo:warning=Extracted {} from {} (second fallback)",
                    lib_name, name
                );
                return Ok(());
            }
        }

        return Err("Could not find whisper binary in xcframework".into());
    } else {
        // Windows: find all required DLLs in the archive
        let mut found = vec![false; lib_names.len()];

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            for (idx, lib_name) in lib_names.iter().enumerate() {
                if !found[idx] && name.ends_with(lib_name) {
                    let output_path = output_dir.join(lib_name);
                    let mut output_file = fs::File::create(&output_path)?;
                    io::copy(&mut file, &mut output_file)?;
                    println!("cargo:warning=Extracted {}", lib_name);
                    found[idx] = true;
                    break;
                }
            }
        }

        // Check that all required libraries were found
        for (idx, lib_name) in lib_names.iter().enumerate() {
            if !found[idx] {
                return Err(format!("Could not find {} in archive", lib_name).into());
            }
        }

        Ok(())
    }
}
