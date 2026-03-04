//! Build script for flowstt-engine
//!
//! Handles downloading/building whisper.cpp library for transcription.

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const WHISPER_VERSION: &str = "1.8.2";
const GITHUB_RELEASE_BASE: &str = "https://github.com/ggml-org/whisper.cpp/releases/download";

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    // Check if CUDA feature is enabled (set by Cargo when --features cuda is used)
    let cuda_enabled = env::var("CARGO_FEATURE_CUDA").is_ok();

    // macOS: Link ScreenCaptureKit framework for system audio capture
    if target_os == "macos" {
        println!("cargo:rustc-link-lib=framework=ScreenCaptureKit");
        println!("cargo:rustc-link-lib=framework=CoreMedia");
        println!("cargo:rustc-link-lib=framework=AVFoundation");
    }

    // Linux: Build whisper.cpp from source using CMake
    if target_os == "linux" {
        build_whisper_linux(cuda_enabled);
        return;
    }

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));

    // Use a stable cache directory in target/ rather than OUT_DIR (which changes per build)
    // This ensures downloads are cached across rebuilds
    let stable_cache_dir = out_dir
        .ancestors()
        .find(|p| p.file_name().map(|n| n == "target").unwrap_or(false))
        .map(|p| p.join("whisper-cache"))
        .unwrap_or_else(|| out_dir.join("whisper-cache"));

    // Windows x64: download BOTH CUDA and CPU-only prebuilt binaries.
    // They are placed in separate subdirectories (cuda/ and cpu/) so the app
    // can try the CUDA variant first and fall back to CPU at runtime.
    // The `cuda` feature flag has no effect on Windows (it is Linux-only).
    if target_os == "windows" && target_arch == "x86_64" {
        if cuda_enabled {
            println!("cargo:warning=Note: --features cuda has no effect on Windows (GPU+CPU support is always included)");
        }
        download_windows_x64_dual_binaries(&stable_cache_dir, &out_dir);
        println!("cargo:rerun-if-changed=build.rs");
        return;
    }

    // Determine which binary to download and which libraries to extract
    let (zip_name, lib_names): (&str, Vec<&str>) = match (target_os.as_str(), target_arch.as_str())
    {
        ("windows", "x86") => (
            "whisper-bin-Win32.zip",
            vec!["whisper.dll", "ggml.dll", "ggml-base.dll", "ggml-cpu.dll"],
        ),
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
            println!(
                "cargo:warning=Unsupported platform: {}-{}",
                target_os, target_arch
            );
            return;
        }
    };
    let primary_lib = lib_names[0];

    // Use stable cache directory for downloads (persists across rebuilds)
    fs::create_dir_all(&stable_cache_dir).expect("Failed to create cache directory");

    let zip_path =
        stable_cache_dir.join(format!("whisper-{}-{}.zip", WHISPER_VERSION, target_arch));
    // Extracted libraries also go in stable cache (versioned to handle updates)
    let lib_output_dir =
        stable_cache_dir.join(format!("whisper-{}-{}-lib", WHISPER_VERSION, target_arch));
    fs::create_dir_all(&lib_output_dir).expect("Failed to create lib output directory");

    let primary_lib_path = lib_output_dir.join(primary_lib);

    // Check if we already have the primary library
    if primary_lib_path.exists() {
        // Library already cached - skip download
    } else {
        // Download if not cached
        if !zip_path.exists() {
            let url = format!("{}/v{}/{}", GITHUB_RELEASE_BASE, WHISPER_VERSION, zip_name);
            println!("cargo:warning=Downloading whisper.cpp binary from: {}", url);
            download_file(&url, &zip_path).expect("Failed to download whisper.cpp binary");
        }

        // Extract all required libraries
        println!("cargo:warning=Extracting whisper.cpp libraries...");
        extract_libraries(
            &zip_path,
            &lib_output_dir,
            &lib_names,
            &target_os,
            &target_arch,
        )
        .expect("Failed to extract whisper.cpp libraries");
    }

    // Set linker flags
    println!(
        "cargo:rustc-link-search=native={}",
        lib_output_dir.display()
    );

    // Copy all libraries to target directory for runtime
    copy_libraries_to_runtime(&lib_output_dir, &lib_names, &out_dir);

    // macOS: Also copy to release directory for Tauri bundling (even in debug builds)
    // Tauri's build script validates bundle resources exist at configured paths
    if target_os == "macos" {
        copy_libraries_for_tauri_bundle(&lib_output_dir, &lib_names, &out_dir);
    }

    // Also write the primary library path to a file for runtime discovery
    let lib_path_file = out_dir.join("whisper_lib_path.txt");
    fs::write(
        &lib_path_file,
        primary_lib_path.to_string_lossy().as_bytes(),
    )
    .expect("Failed to write library path file");

    // Rerun if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
}

/// Download both CUDA and CPU-only prebuilt binaries for Windows x64.
///
/// The CUDA variant includes GPU acceleration via ggml-cuda.dll + CUDA runtime,
/// but requires NVIDIA drivers (nvcuda.dll) to load. The CPU variant works on
/// any machine. Both are placed in separate subdirectories so the app can try
/// CUDA first and fall back to CPU at runtime.
///
/// Layout in target/release/ (picked up by Tauri bundling):
///   cuda/  - CUDA-enabled DLLs (ggml.dll links to ggml-cuda.dll → nvcuda.dll)
///   cpu/   - CPU-only DLLs (no CUDA dependency)
///
/// Both subdirs also get the VC++ runtime DLLs (msvcp140.dll, etc.) bundled.
fn download_windows_x64_dual_binaries(stable_cache_dir: &Path, out_dir: &Path) {
    fs::create_dir_all(stable_cache_dir).expect("Failed to create cache directory");

    let target_dir = out_dir
        .ancestors()
        .find(|p| p.file_name().map(|n| n == "target").unwrap_or(false))
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| out_dir.join("..").join("..").join(".."));

    let cuda_libs: Vec<&str> = vec![
        "whisper.dll",
        "ggml.dll",
        "ggml-base.dll",
        "ggml-cpu.dll",
        "ggml-cuda.dll",
        "cublas64_12.dll",
        "cublasLt64_12.dll",
        "cudart64_12.dll",
        "nvrtc64_120_0.dll",
        "nvrtc-builtins64_124.dll",
        "nvblas64_12.dll",
    ];

    let cpu_libs: Vec<&str> = vec!["whisper.dll", "ggml.dll", "ggml-base.dll", "ggml-cpu.dll"];

    // Download and extract CUDA variant
    let cuda_cache = stable_cache_dir.join(format!("whisper-{}-x86_64-cuda-lib", WHISPER_VERSION));
    let cuda_zip = stable_cache_dir.join(format!("whisper-{}-x86_64-cuda.zip", WHISPER_VERSION));
    fs::create_dir_all(&cuda_cache).expect("Failed to create CUDA cache directory");

    if !cuda_cache.join("whisper.dll").exists() {
        if !cuda_zip.exists() {
            let url = format!(
                "{}/v{}/whisper-cublas-12.4.0-bin-x64.zip",
                GITHUB_RELEASE_BASE, WHISPER_VERSION
            );
            println!(
                "cargo:warning=Downloading CUDA whisper.cpp binaries from: {}",
                url
            );
            download_file(&url, &cuda_zip).expect("Failed to download CUDA whisper.cpp binary");
        }
        println!("cargo:warning=Extracting CUDA whisper.cpp libraries...");
        extract_libraries(&cuda_zip, &cuda_cache, &cuda_libs, "windows", "x86_64")
            .expect("Failed to extract CUDA whisper.cpp libraries");
    }

    // Download and extract CPU variant
    let cpu_cache = stable_cache_dir.join(format!("whisper-{}-x86_64-cpu-lib", WHISPER_VERSION));
    let cpu_zip = stable_cache_dir.join(format!("whisper-{}-x86_64-cpu.zip", WHISPER_VERSION));
    fs::create_dir_all(&cpu_cache).expect("Failed to create CPU cache directory");

    if !cpu_cache.join("whisper.dll").exists() {
        if !cpu_zip.exists() {
            let url = format!(
                "{}/v{}/whisper-bin-x64.zip",
                GITHUB_RELEASE_BASE, WHISPER_VERSION
            );
            println!(
                "cargo:warning=Downloading CPU whisper.cpp binaries from: {}",
                url
            );
            download_file(&url, &cpu_zip).expect("Failed to download CPU whisper.cpp binary");
        }
        println!("cargo:warning=Extracting CPU whisper.cpp libraries...");
        extract_libraries(&cpu_zip, &cpu_cache, &cpu_libs, "windows", "x86_64")
            .expect("Failed to extract CPU whisper.cpp libraries");
    }

    // Set linker search path (CUDA variant, has all symbols)
    println!("cargo:rustc-link-search=native={}", cuda_cache.display());

    // Copy to target/release/cuda/ and target/release/cpu/ for Tauri bundling
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let runtime_dir = target_dir.join(&profile);
    let release_dir = target_dir.join("release");

    for dir in [&runtime_dir, &release_dir] {
        let cuda_dest = dir.join("cuda");
        let cpu_dest = dir.join("cpu");
        let _ = fs::create_dir_all(&cuda_dest);
        let _ = fs::create_dir_all(&cpu_dest);

        for lib in &cuda_libs {
            let src = cuda_cache.join(lib);
            let dest = cuda_dest.join(lib);
            if src.exists() {
                copy_if_changed(&src, &dest, lib);
            }
        }
        for lib in &cpu_libs {
            let src = cpu_cache.join(lib);
            let dest = cpu_dest.join(lib);
            if src.exists() {
                copy_if_changed(&src, &dest, lib);
            }
        }

        // Clean up placeholder DLLs created by src-tauri/build.rs
        for subdir in [&cuda_dest, &cpu_dest] {
            let placeholder = subdir.join(".tauri-placeholder.dll");
            if placeholder.exists() {
                let _ = fs::remove_file(&placeholder);
            }
        }
    }

    // Bundle VC++ runtime DLLs into BOTH cuda/ and cpu/ subdirs
    bundle_vcruntime_dlls_to_subdir(out_dir, "cuda");
    bundle_vcruntime_dlls_to_subdir(out_dir, "cpu");

    // Write the primary library path for runtime discovery
    let lib_path_file =
        PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set")).join("whisper_lib_path.txt");
    fs::write(
        &lib_path_file,
        cuda_cache.join("whisper.dll").to_string_lossy().as_bytes(),
    )
    .expect("Failed to write library path file");

    println!("cargo:warning=Windows x64: bundled both CUDA and CPU whisper.cpp variants");
}

/// Copy a file only if the destination doesn't exist or has a different size.
fn copy_if_changed(src: &Path, dest: &Path, label: &str) {
    let needs_copy = if dest.exists() {
        fs::metadata(src).map(|m| m.len()).unwrap_or(0)
            != fs::metadata(dest).map(|m| m.len()).unwrap_or(0)
    } else {
        true
    };
    if needs_copy {
        if let Err(e) = fs::copy(src, dest) {
            println!("cargo:warning=Failed to copy {}: {}", label, e);
        }
    }
}

/// Build whisper.cpp from source on Linux using CMake
fn build_whisper_linux(cuda_enabled: bool) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    // Check for CMake
    if !check_cmake_available() {
        panic!(
            "\n\nCMake is required to build whisper.cpp on Linux.\n\
            Please install CMake:\n\
            - Ubuntu/Debian: sudo apt install cmake\n\
            - Fedora: sudo dnf install cmake\n\
            - Arch: sudo pacman -S cmake\n\n"
        );
    }

    // Check for CUDA toolkit if cuda feature is enabled
    if cuda_enabled && !check_cuda_available() {
        panic!(
            "\n\nCUDA feature is enabled but CUDA Toolkit is not found.\n\
            Please install NVIDIA CUDA Toolkit:\n\
            - Ubuntu/Debian: sudo apt install nvidia-cuda-toolkit\n\
            - Or download from: https://developer.nvidia.com/cuda-downloads\n\n\
            If you don't need CUDA, build without the cuda feature:\n\
            cargo build --release\n\n"
        );
    }

    // Use stable cache directory in target/ (persists across rebuilds)
    let stable_cache_dir = out_dir
        .ancestors()
        .find(|p| p.file_name().map(|n| n == "target").unwrap_or(false))
        .map(|p| p.join("whisper-cache"))
        .unwrap_or_else(|| out_dir.join("whisper-cache"));
    fs::create_dir_all(&stable_cache_dir).expect("Failed to create cache directory");

    // Include cuda in paths to separate CUDA and non-CUDA builds
    let cuda_suffix = if cuda_enabled { "-cuda" } else { "" };
    let source_tarball = stable_cache_dir.join(format!("whisper-{}.tar.gz", WHISPER_VERSION));
    let source_dir = stable_cache_dir.join(format!("whisper.cpp-{}", WHISPER_VERSION));
    let build_dir =
        stable_cache_dir.join(format!("whisper-{}-build{}", WHISPER_VERSION, cuda_suffix));
    let lib_output_dir = stable_cache_dir.join(format!(
        "whisper-{}-{}{}-lib",
        WHISPER_VERSION, target_arch, cuda_suffix
    ));

    fs::create_dir_all(&lib_output_dir).expect("Failed to create lib output directory");

    let lib_path = lib_output_dir.join("libwhisper.so");

    // Check if we already have the library built
    if lib_path.exists() {
        println!("cargo:warning=Using cached whisper.cpp library");
    } else {
        // Download source tarball if not cached
        if !source_dir.exists() {
            if !source_tarball.exists() {
                let url = format!(
                    "https://github.com/ggml-org/whisper.cpp/archive/refs/tags/v{}.tar.gz",
                    WHISPER_VERSION
                );
                println!("cargo:warning=Downloading whisper.cpp source from: {}", url);
                download_file(&url, &source_tarball)
                    .expect("Failed to download whisper.cpp source");
            }

            // Extract tarball
            println!("cargo:warning=Extracting whisper.cpp source...");
            extract_tarball(&source_tarball, &stable_cache_dir)
                .expect("Failed to extract whisper.cpp source");
        }

        // Create build directory
        fs::create_dir_all(&build_dir).expect("Failed to create build directory");

        // Configure with CMake
        println!("cargo:warning=Configuring whisper.cpp with CMake...");
        let mut cmake_args = vec![
            source_dir.to_string_lossy().to_string(),
            "-DBUILD_SHARED_LIBS=ON".to_string(),
            "-DCMAKE_BUILD_TYPE=Release".to_string(),
            "-DWHISPER_BUILD_EXAMPLES=OFF".to_string(),
            "-DWHISPER_BUILD_TESTS=OFF".to_string(),
            "-DWHISPER_BUILD_SERVER=OFF".to_string(),
        ];

        if cuda_enabled {
            println!("cargo:warning=CUDA feature enabled - configuring with GPU support");
            cmake_args.push("-DGGML_CUDA=ON".to_string());
        }

        let cmake_status = Command::new("cmake")
            .args(&cmake_args)
            .current_dir(&build_dir)
            .status()
            .expect("Failed to run cmake configure");

        if !cmake_status.success() {
            panic!("CMake configuration failed");
        }

        // Build
        println!("cargo:warning=Building whisper.cpp (this may take a few minutes)...");
        let build_status = Command::new("cmake")
            .args(["--build", ".", "--config", "Release", "-j"])
            .current_dir(&build_dir)
            .status()
            .expect("Failed to run cmake build");

        if !build_status.success() {
            panic!("CMake build failed");
        }

        // Find and copy built libraries
        println!("cargo:warning=Copying built libraries...");
        copy_built_libraries(&build_dir, &lib_output_dir).expect("Failed to copy built libraries");
    }

    // Set linker flags
    println!(
        "cargo:rustc-link-search=native={}",
        lib_output_dir.display()
    );

    // Copy libraries to runtime directory
    let lib_names: Vec<&str> = vec![
        "libwhisper.so",
        "libggml.so",
        "libggml-base.so",
        "libggml-cpu.so",
    ];
    copy_libraries_to_runtime(&lib_output_dir, &lib_names, &out_dir);

    // If CUDA is enabled, also copy CUDA-specific libraries
    if cuda_enabled {
        let cuda_libs = ["libggml-cuda.so"];
        for lib in cuda_libs {
            let src = lib_output_dir.join(lib);
            if src.exists() {
                copy_library_to_runtime(&src, lib, &out_dir);
            }
        }
    }

    // Write library path for runtime discovery
    let lib_path_file = out_dir.join("whisper_lib_path.txt");
    fs::write(&lib_path_file, lib_path.to_string_lossy().as_bytes())
        .expect("Failed to write library path file");

    println!("cargo:warning=Linux build: whisper.cpp built from source with CMake");
    if cuda_enabled {
        println!("cargo:warning=CUDA support enabled");
    }

    println!("cargo:rerun-if-changed=build.rs");
}

/// Check if CMake is available
fn check_cmake_available() -> bool {
    Command::new("cmake")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if CUDA toolkit is available (nvcc compiler)
fn check_cuda_available() -> bool {
    Command::new("nvcc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Extract a .tar.gz file
fn extract_tarball(tarball: &Path, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("tar")
        .args([
            "-xzf",
            &tarball.to_string_lossy(),
            "-C",
            &dest.to_string_lossy(),
        ])
        .status()?;

    if !status.success() {
        return Err("Failed to extract tarball".into());
    }

    Ok(())
}

/// Copy built libraries from CMake build directory to output
fn copy_built_libraries(
    build_dir: &Path,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Look for shared libraries in the build directory
    // CMake may put them in different locations depending on configuration
    let search_paths = [
        build_dir.to_path_buf(),
        build_dir.join("src"),
        build_dir.join("ggml").join("src"),
        build_dir.join("lib"),
    ];

    let lib_patterns = [
        "libwhisper.so",
        "libggml.so",
        "libggml-base.so",
        "libggml-cpu.so",
        "libggml-cuda.so",
    ];

    for pattern in &lib_patterns {
        for search_path in &search_paths {
            // Look for the library (may have version suffix like .so.1.8.2)
            if let Ok(entries) = fs::read_dir(search_path) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();

                    // Match libwhisper.so or libwhisper.so.1.8.2 etc.
                    if name_str.starts_with(pattern) || name_str == *pattern {
                        let src = entry.path();

                        // Follow symlinks to get the actual file
                        let real_src = if src.is_symlink() {
                            fs::read_link(&src).unwrap_or(src.clone())
                        } else {
                            src.clone()
                        };

                        // Resolve relative symlink paths
                        let real_src = if real_src.is_relative() {
                            search_path.join(&real_src)
                        } else {
                            real_src
                        };

                        if real_src.exists() && real_src.is_file() {
                            // Copy as the base name (libwhisper.so)
                            let dest = output_dir.join(pattern);
                            fs::copy(&real_src, &dest)?;
                            println!("cargo:warning=Copied {} from {:?}", pattern, real_src);
                            break;
                        }
                    }
                }
            }
        }
    }

    // Verify libwhisper.so was copied
    if !output_dir.join("libwhisper.so").exists() {
        return Err("libwhisper.so not found in build output".into());
    }

    Ok(())
}

/// Copy libraries to the runtime directory
fn copy_libraries_to_runtime(lib_dir: &Path, lib_names: &[&str], out_dir: &Path) {
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let target_dir = out_dir
        .ancestors()
        .find(|p| p.ends_with("target") || p.file_name().map(|n| n == "target").unwrap_or(false))
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| out_dir.join("..").join("..").join(".."));

    let runtime_lib_dir = target_dir.join(&profile);
    if runtime_lib_dir.exists() {
        for lib_name in lib_names {
            let lib_path = lib_dir.join(lib_name);
            if lib_path.exists() {
                copy_library_to_runtime(&lib_path, lib_name, out_dir);
            }
        }
    }
}

/// Copy a single library to the runtime directory
fn copy_library_to_runtime(lib_path: &Path, lib_name: &str, out_dir: &Path) {
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let target_dir = out_dir
        .ancestors()
        .find(|p| p.ends_with("target") || p.file_name().map(|n| n == "target").unwrap_or(false))
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| out_dir.join("..").join("..").join(".."));

    let runtime_lib_dir = target_dir.join(&profile);
    if runtime_lib_dir.exists() {
        let runtime_lib_path = runtime_lib_dir.join(lib_name);
        if lib_path.exists()
            && (!runtime_lib_path.exists()
                || fs::metadata(lib_path).map(|m| m.len()).unwrap_or(0)
                    != fs::metadata(&runtime_lib_path)
                        .map(|m| m.len())
                        .unwrap_or(0))
        {
            if let Err(e) = fs::copy(lib_path, &runtime_lib_path) {
                println!("cargo:warning=Failed to copy {}: {}", lib_name, e);
            } else {
                println!(
                    "cargo:warning=Copied {} to {}",
                    lib_name,
                    runtime_lib_dir.display()
                );
            }
        }
    }
}

/// Copy libraries to release directory for Tauri bundling (macOS/Windows)
/// Tauri's build script validates bundle resources exist, even during debug builds
fn copy_libraries_for_tauri_bundle(lib_dir: &Path, lib_names: &[&str], out_dir: &Path) {
    let target_dir = out_dir
        .ancestors()
        .find(|p| p.file_name().map(|n| n == "target").unwrap_or(false))
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| out_dir.join("..").join("..").join(".."));

    let release_lib_dir = target_dir.join("release");
    if !release_lib_dir.exists() {
        if let Err(e) = fs::create_dir_all(&release_lib_dir) {
            println!(
                "cargo:warning=Failed to create release directory for Tauri bundle: {}",
                e
            );
            return;
        }
    }

    for lib_name in lib_names {
        let lib_path = lib_dir.join(lib_name);
        let release_lib_path = release_lib_dir.join(lib_name);

        if lib_path.exists()
            && (!release_lib_path.exists()
                || fs::metadata(&lib_path).map(|m| m.len()).unwrap_or(0)
                    != fs::metadata(&release_lib_path)
                        .map(|m| m.len())
                        .unwrap_or(0))
        {
            if let Err(e) = fs::copy(&lib_path, &release_lib_path) {
                println!(
                    "cargo:warning=Failed to copy {} for Tauri bundle: {}",
                    lib_name, e
                );
            } else {
                println!(
                    "cargo:warning=Copied {} to release/ for Tauri bundling",
                    lib_name
                );
            }
        }
    }

    // Clean up the placeholder DLL created by the Tauri app build script.
    // See src-tauri/build.rs for details on why this placeholder exists.
    let placeholder = release_lib_dir.join(".tauri-placeholder.dll");
    if placeholder.exists() {
        let _ = fs::remove_file(&placeholder);
    }
}

/// Bundle Visual C++ runtime DLLs for Windows into a specific subdirectory
/// under target/release/ (and target/{profile}/).
///
/// The whisper.cpp prebuilt DLLs link against MSVCP140, VCRUNTIME140,
/// VCRUNTIME140_1, and VCOMP140 (OpenMP). These are not part of Windows
/// and must be present alongside the DLLs that need them.
///
/// We copy them from the build machine's System32 into the specified subdir
/// so Tauri bundles them alongside the whisper DLLs.
fn bundle_vcruntime_dlls_to_subdir(out_dir: &Path, subdir: &str) {
    let target_dir = out_dir
        .ancestors()
        .find(|p| p.file_name().map(|n| n == "target").unwrap_or(false))
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| out_dir.join("..").join("..").join(".."));

    let system32 = PathBuf::from(env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into()))
        .join("System32");

    let vcrt_dlls = [
        "msvcp140.dll",
        "vcruntime140.dll",
        "vcruntime140_1.dll",
        "vcomp140.dll",
    ];

    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let dirs = [
        target_dir.join(&profile).join(subdir),
        target_dir.join("release").join(subdir),
    ];

    for dest_dir in &dirs {
        let _ = fs::create_dir_all(dest_dir);
        for dll in &vcrt_dlls {
            let src = system32.join(dll);
            let dest = dest_dir.join(dll);
            if src.exists() {
                copy_if_changed(&src, &dest, dll);
            } else {
                println!(
                    "cargo:warning=VC++ runtime not found on build machine: {} (target machines may fail to load whisper DLLs)",
                    src.display()
                );
            }
        }
    }
    println!("cargo:warning=Bundled VC++ runtime DLLs into {}/", subdir);
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

        Err("Could not find whisper binary in xcframework".into())
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
