/// Audio processor trait for extensible audio analysis.
/// Processors must be fast and non-blocking as they run in the audio callback.
pub trait AudioProcessor: Send {
    /// Process a batch of audio samples.
    /// Samples are mono f32 values, typically in the range [-1.0, 1.0].
    fn process(&mut self, samples: &[f32]);

    /// Return the processor's name for identification.
    fn name(&self) -> &str;
}

/// Silence detector that logs state transitions to console.
/// Uses RMS (root mean square) amplitude with a configurable dB threshold.
pub struct SilenceDetector {
    /// Threshold in dB below which audio is considered silent (default: -40.0)
    threshold_db: f32,
    /// Current silence state
    is_silent: bool,
    /// Whether we've logged the initial state
    initialized: bool,
}

impl SilenceDetector {
    /// Create a new silence detector with default threshold (-40 dB)
    pub fn new() -> Self {
        Self {
            threshold_db: -40.0,
            is_silent: true,
            initialized: false,
        }
    }

    /// Calculate RMS amplitude of samples
    fn calculate_rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_squares: f32 = samples.iter().map(|s| s * s).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }

    /// Convert linear amplitude to decibels
    fn amplitude_to_db(amplitude: f32) -> f32 {
        if amplitude <= 0.0 {
            return f32::NEG_INFINITY;
        }
        20.0 * amplitude.log10()
    }
}

impl Default for SilenceDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioProcessor for SilenceDetector {
    fn process(&mut self, samples: &[f32]) {
        let rms = Self::calculate_rms(samples);
        let db = Self::amplitude_to_db(rms);
        let now_silent = db < self.threshold_db;

        // Only log on state transitions (or first detection)
        if !self.initialized {
            self.initialized = true;
            self.is_silent = now_silent;
            if now_silent {
                println!("[SilenceDetector] Silence detected (initial state)");
            } else {
                println!("[SilenceDetector] Sound detected (initial state)");
            }
        } else if now_silent != self.is_silent {
            self.is_silent = now_silent;
            if now_silent {
                println!("[SilenceDetector] Silence detected");
            } else {
                println!("[SilenceDetector] Sound detected");
            }
        }
    }

    fn name(&self) -> &str {
        "SilenceDetector"
    }
}
