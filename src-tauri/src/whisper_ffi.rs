//! FFI bindings to whisper.cpp for Windows and macOS.
//! This module uses libloading to dynamically load the whisper shared library at runtime.

use libloading::Library;
use std::ffi::{c_char, c_float, c_int, CStr, CString};
use std::path::Path;
use std::sync::OnceLock;

/// Opaque pointer to whisper_context
type WhisperContext = *mut std::ffi::c_void;



/// whisper_full_params is a large struct, but we only need to pass it by value
/// The actual struct is ~200 bytes, we'll use the default params function
#[repr(C)]
#[derive(Clone)]
pub struct WhisperFullParams {
    // This is a simplified representation - the actual struct is larger
    // We'll allocate enough space and let whisper fill it in
    _data: [u8; 512], // Oversized to be safe
}

impl Default for WhisperFullParams {
    fn default() -> Self {
        Self { _data: [0u8; 512] }
    }
}

/// Sampling strategy enum matching whisper.cpp
#[repr(C)]
#[allow(dead_code)]
pub enum WhisperSamplingStrategy {
    Greedy = 0,
    BeamSearch = 1,
}

/// Global library handle
static WHISPER_LIB: OnceLock<Option<WhisperLibrary>> = OnceLock::new();

/// Wrapper around the loaded whisper library
pub struct WhisperLibrary {
    _lib: Library,
    // Function pointers
    init_from_file:
        unsafe extern "C" fn(path_model: *const c_char) -> WhisperContext,
    free: unsafe extern "C" fn(ctx: WhisperContext),
    full_default_params:
        unsafe extern "C" fn(strategy: c_int) -> WhisperFullParams,
    full: unsafe extern "C" fn(
        ctx: WhisperContext,
        params: WhisperFullParams,
        samples: *const c_float,
        n_samples: c_int,
    ) -> c_int,
    full_n_segments: unsafe extern "C" fn(ctx: WhisperContext) -> c_int,
    full_get_segment_text:
        unsafe extern "C" fn(ctx: WhisperContext, i_segment: c_int) -> *const c_char,
    print_system_info: unsafe extern "C" fn() -> *const c_char,
}

// SAFETY: The library handle and function pointers don't contain thread-local data
unsafe impl Send for WhisperLibrary {}
unsafe impl Sync for WhisperLibrary {}

impl WhisperLibrary {
    /// Load the whisper library from the given path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        unsafe {
            let lib = Library::new(path.as_ref())
                .map_err(|e| format!("Failed to load whisper library: {}", e))?;

            // Load all required symbols - dereference immediately to get raw fn pointers
            let init_from_file = *lib
                .get::<unsafe extern "C" fn(*const c_char) -> WhisperContext>(
                    b"whisper_init_from_file\0",
                )
                .map_err(|e| format!("Failed to load whisper_init_from_file: {}", e))?;

            let free = *lib
                .get::<unsafe extern "C" fn(WhisperContext)>(b"whisper_free\0")
                .map_err(|e| format!("Failed to load whisper_free: {}", e))?;

            let full_default_params = *lib
                .get::<unsafe extern "C" fn(c_int) -> WhisperFullParams>(
                    b"whisper_full_default_params\0",
                )
                .map_err(|e| format!("Failed to load whisper_full_default_params: {}", e))?;

            let full = *lib
                .get::<unsafe extern "C" fn(WhisperContext, WhisperFullParams, *const c_float, c_int) -> c_int>(
                    b"whisper_full\0",
                )
                .map_err(|e| format!("Failed to load whisper_full: {}", e))?;

            let full_n_segments = *lib
                .get::<unsafe extern "C" fn(WhisperContext) -> c_int>(b"whisper_full_n_segments\0")
                .map_err(|e| format!("Failed to load whisper_full_n_segments: {}", e))?;

            let full_get_segment_text = *lib
                .get::<unsafe extern "C" fn(WhisperContext, c_int) -> *const c_char>(
                    b"whisper_full_get_segment_text\0",
                )
                .map_err(|e| format!("Failed to load whisper_full_get_segment_text: {}", e))?;

            let print_system_info = *lib
                .get::<unsafe extern "C" fn() -> *const c_char>(
                    b"whisper_print_system_info\0",
                )
                .map_err(|e| format!("Failed to load whisper_print_system_info: {}", e))?;

            Ok(Self {
                _lib: lib,
                init_from_file,
                free,
                full_default_params,
                full,
                full_n_segments,
                full_get_segment_text,
                print_system_info,
            })
        }
    }
}

/// Initialize the global whisper library
pub fn init_library() -> Result<(), String> {
    WHISPER_LIB.get_or_init(|| {
        // Try to find the library in various locations
        let lib_name = if cfg!(windows) {
            "whisper.dll"
        } else {
            "libwhisper.dylib"
        };

        // Search paths in order of preference:
        // 1. Next to the executable
        // 2. In the current directory
        // 3. System library paths (handled by libloading)
        let search_paths = [
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.join(lib_name))),
            Some(std::env::current_dir().unwrap_or_default().join(lib_name)),
            Some(std::path::PathBuf::from(lib_name)),
        ];

        for path in search_paths.iter().flatten() {
            if path.exists() {
                match WhisperLibrary::load(path) {
                    Ok(lib) => {
                        eprintln!("Loaded whisper library from: {}", path.display());
                        return Some(lib);
                    }
                    Err(e) => {
                        eprintln!("Failed to load whisper library from {}: {}", path.display(), e);
                    }
                }
            }
        }

        // Try loading from system path
        match WhisperLibrary::load(lib_name) {
            Ok(lib) => {
                eprintln!("Loaded whisper library from system path");
                Some(lib)
            }
            Err(e) => {
                eprintln!("Failed to load whisper library: {}", e);
                None
            }
        }
    });

    if WHISPER_LIB.get().and_then(|l| l.as_ref()).is_some() {
        Ok(())
    } else {
        Err("Whisper library not available".to_string())
    }
}

/// Get the loaded library or return an error
fn get_lib() -> Result<&'static WhisperLibrary, String> {
    WHISPER_LIB
        .get()
        .and_then(|l| l.as_ref())
        .ok_or_else(|| "Whisper library not loaded".to_string())
}

/// Safe wrapper around whisper context
pub struct Context {
    ptr: WhisperContext,
}

// SAFETY: WhisperContext is thread-safe according to whisper.cpp documentation
unsafe impl Send for Context {}

impl Context {
    /// Create a new context from a model file
    pub fn new<P: AsRef<Path>>(model_path: P) -> Result<Self, String> {
        let lib = get_lib()?;

        let path_str = model_path
            .as_ref()
            .to_str()
            .ok_or("Invalid model path")?;
        let c_path = CString::new(path_str).map_err(|e| format!("Invalid path: {}", e))?;

        let ptr = unsafe { (lib.init_from_file)(c_path.as_ptr()) };

        if ptr.is_null() {
            return Err(format!(
                "Failed to initialize whisper context from: {}",
                path_str
            ));
        }

        Ok(Self { ptr })
    }

    /// Run full transcription on audio samples
    pub fn full(&self, params: &WhisperFullParams, samples: &[f32]) -> Result<(), String> {
        let lib = get_lib()?;

        let result = unsafe {
            (lib.full)(
                self.ptr,
                params.clone(),
                samples.as_ptr(),
                samples.len() as c_int,
            )
        };

        if result != 0 {
            return Err(format!("Transcription failed with code: {}", result));
        }

        Ok(())
    }

    /// Get the number of segments in the transcription result
    pub fn full_n_segments(&self) -> Result<i32, String> {
        let lib = get_lib()?;
        Ok(unsafe { (lib.full_n_segments)(self.ptr) })
    }

    /// Get the text of a specific segment
    pub fn full_get_segment_text(&self, i_segment: i32) -> Result<String, String> {
        let lib = get_lib()?;

        let ptr = unsafe { (lib.full_get_segment_text)(self.ptr, i_segment) };

        if ptr.is_null() {
            return Err(format!("Failed to get segment {} text", i_segment));
        }

        let c_str = unsafe { CStr::from_ptr(ptr) };
        c_str
            .to_str()
            .map(|s| s.to_string())
            .map_err(|e| format!("Invalid UTF-8 in segment: {}", e))
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        if let Ok(lib) = get_lib() {
            unsafe { (lib.free)(self.ptr) };
        }
    }
}

/// Get default parameters for the given sampling strategy
pub fn full_default_params(strategy: WhisperSamplingStrategy) -> Result<WhisperFullParams, String> {
    let lib = get_lib()?;
    Ok(unsafe { (lib.full_default_params)(strategy as c_int) })
}

/// Get whisper.cpp system info string
/// This includes information about available backends (CPU, CUDA, Metal, etc.)
pub fn get_system_info() -> Result<String, String> {
    let lib = get_lib()?;
    let ptr = unsafe { (lib.print_system_info)() };
    if ptr.is_null() {
        return Err("Failed to get system info".to_string());
    }
    let c_str = unsafe { CStr::from_ptr(ptr) };
    c_str
        .to_str()
        .map(|s| s.to_string())
        .map_err(|e| format!("Invalid UTF-8 in system info: {}", e))
}
