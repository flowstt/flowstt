//! Whisper transcription wrapper.
//!
//! This module provides a high-level API for transcribing audio using whisper.cpp.

use std::path::PathBuf;

use super::whisper_ffi::{self, Context, WhisperSamplingStrategy};

const MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin";

/// Wrapper around whisper.cpp for transcription.
pub struct Transcriber {
    ctx: Option<Context>,
    model_path: PathBuf,
    library_initialized: bool,
}

impl Transcriber {
    /// Create a new transcriber with the default model path.
    pub fn new() -> Self {
        let model_path = get_default_model_path();
        Self {
            ctx: None,
            model_path,
            library_initialized: false,
        }
    }

    /// Get the path to the model file.
    pub fn get_model_path(&self) -> &PathBuf {
        &self.model_path
    }

    /// Check if the model file exists.
    pub fn is_model_available(&self) -> bool {
        self.model_path.exists()
    }

    /// Ensure the whisper library is loaded.
    fn ensure_library(&mut self) -> Result<(), String> {
        if !self.library_initialized {
            whisper_ffi::init_library()?;
            self.library_initialized = true;
        }
        Ok(())
    }

    /// Load the whisper model. This is called automatically by transcribe() if needed.
    pub fn load_model(&mut self) -> Result<(), String> {
        if self.ctx.is_some() {
            return Ok(());
        }

        self.ensure_library()?;

        if !self.model_path.exists() {
            return Err(format!(
                "Whisper model not found at: {}\n\n\
                Please download a model file:\n\
                1. Visit: https://huggingface.co/ggerganov/whisper.cpp/tree/main\n\
                2. Download 'ggml-base.en.bin' (or another model)\n\
                3. Place it at: {}",
                self.model_path.display(),
                self.model_path.display()
            ));
        }

        tracing::info!("Loading whisper model from: {}", self.model_path.display());
        let ctx = Context::new(&self.model_path)?;
        self.ctx = Some(ctx);
        tracing::info!("Whisper model loaded successfully");
        Ok(())
    }

    /// Transcribe audio samples (mono, 16kHz).
    ///
    /// The audio should already be converted to mono 16kHz format.
    pub fn transcribe(&mut self, audio_data: &[f32]) -> Result<String, String> {
        self.load_model()?;

        let ctx = self.ctx.as_ref().unwrap();

        // Get default params with greedy strategy
        let params = whisper_ffi::full_default_params(WhisperSamplingStrategy::Greedy)?;

        // Run transcription
        ctx.full(&params, audio_data)?;

        let num_segments = ctx.full_n_segments()?;

        if num_segments == 0 {
            return Ok("(No speech detected)".to_string());
        }

        let mut result = String::new();
        for i in 0..num_segments {
            if let Ok(segment) = ctx.full_get_segment_text(i) {
                result.push_str(&segment);
                result.push(' ');
            }
        }

        let trimmed = result.trim().to_string();
        if trimmed.is_empty() {
            Ok("(No speech detected)".to_string())
        } else {
            Ok(trimmed)
        }
    }

    /// Transcribe audio with duration hint for optimization.
    ///
    /// The duration_ms parameter helps optimize whisper parameters for short audio.
    pub fn transcribe_with_duration(
        &mut self,
        audio_data: &[f32],
        duration_ms: i32,
    ) -> Result<String, String> {
        self.load_model()?;

        let ctx = self.ctx.as_ref().unwrap();

        // Get default params with greedy strategy
        let mut params = whisper_ffi::full_default_params(WhisperSamplingStrategy::Greedy)?;

        // Optimize for short audio if duration is known
        if duration_ms > 0 && duration_ms < 10000 {
            params.configure_for_short_audio(audio_data.len(), duration_ms);
        }

        // Run transcription
        ctx.full(&params, audio_data)?;

        let num_segments = ctx.full_n_segments()?;

        if num_segments == 0 {
            return Ok("(No speech detected)".to_string());
        }

        let mut result = String::new();
        for i in 0..num_segments {
            if let Ok(segment) = ctx.full_get_segment_text(i) {
                result.push_str(&segment);
                result.push(' ');
            }
        }

        let trimmed = result.trim().to_string();
        if trimmed.is_empty() {
            Ok("(No speech detected)".to_string())
        } else {
            Ok(trimmed)
        }
    }
}

impl Default for Transcriber {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the default model path.
fn get_default_model_path() -> PathBuf {
    let cache_dir = directories::BaseDirs::new()
        .map(|d| d.cache_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    cache_dir.join("whisper").join("ggml-base.en.bin")
}

/// Download the Whisper model to the specified path.
pub fn download_model(model_path: &PathBuf) -> Result<(), String> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = model_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    tracing::info!("Downloading whisper model to: {}", model_path.display());

    // Download the model
    let response = reqwest::blocking::get(MODEL_URL)
        .map_err(|e| format!("Failed to download model: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to download model: HTTP {}",
            response.status()
        ));
    }

    let bytes = response
        .bytes()
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Write to file
    std::fs::write(model_path, &bytes).map_err(|e| format!("Failed to write file: {}", e))?;

    tracing::info!("Model downloaded successfully ({} bytes)", bytes.len());

    Ok(())
}
