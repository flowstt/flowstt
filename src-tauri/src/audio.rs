use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, StreamConfig};
use rubato::{FftFixedIn, Resampler};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter};

use crate::processor::{AudioProcessor, SilenceDetector};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
}

/// Audio samples payload for visualization events
#[derive(Clone, Serialize)]
pub struct AudioSamplesPayload {
    pub samples: Vec<f32>,
}

/// Raw recorded audio data before processing
pub struct RawRecordedAudio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

/// Shared state for audio stream
struct AudioStreamState {
    // Recording state
    recording_samples: Vec<f32>,
    sample_rate: u32,
    channels: u16,
    is_recording: bool,
    
    // Visualization state
    visualization_samples: Vec<f32>,
    is_monitoring: bool,
    
    // Processing state
    is_processing_enabled: bool,
    processor: Option<Box<dyn AudioProcessor>>,
    
    // Stream control
    stream_active: bool,
}

/// Thread-safe audio state that can be shared with Tauri
pub struct RecordingState {
    state: Arc<Mutex<AudioStreamState>>,
    stop_signal: Arc<Mutex<bool>>,
    current_device_id: Arc<Mutex<Option<String>>>,
}

impl RecordingState {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(AudioStreamState {
                recording_samples: Vec::new(),
                sample_rate: 0,
                channels: 0,
                is_recording: false,
                visualization_samples: Vec::with_capacity(512),
                is_monitoring: false,
                is_processing_enabled: false,
                processor: Some(Box::new(SilenceDetector::new())),
                stream_active: false,
            })),
            stop_signal: Arc::new(Mutex::new(false)),
            current_device_id: Arc::new(Mutex::new(None)),
        }
    }

    pub fn is_recording(&self) -> bool {
        self.state.lock().unwrap().is_recording
    }

    pub fn is_monitoring(&self) -> bool {
        self.state.lock().unwrap().is_monitoring
    }

    pub fn is_processing_enabled(&self) -> bool {
        self.state.lock().unwrap().is_processing_enabled
    }

    pub fn set_processing_enabled(&self, enabled: bool) {
        let mut state = self.state.lock().unwrap();
        state.is_processing_enabled = enabled;
        // Reset processor state when enabling
        if enabled {
            state.processor = Some(Box::new(SilenceDetector::new()));
        }
    }
}

pub fn list_devices() -> Result<Vec<AudioDevice>, String> {
    let host = cpal::default_host();
    let devices = host
        .input_devices()
        .map_err(|e| format!("Failed to enumerate devices: {}", e))?;

    let mut result = Vec::new();
    for (index, device) in devices.enumerate() {
        let name = device
            .name()
            .unwrap_or_else(|_| format!("Unknown Device {}", index));
        result.push(AudioDevice {
            id: index.to_string(),
            name,
        });
    }

    Ok(result)
}

fn get_device_by_id(device_id: &str) -> Result<Device, String> {
    let host = cpal::default_host();
    let index: usize = device_id
        .parse()
        .map_err(|_| "Invalid device ID".to_string())?;

    let devices = host
        .input_devices()
        .map_err(|e| format!("Failed to enumerate devices: {}", e))?;

    devices
        .enumerate()
        .find(|(i, _)| *i == index)
        .map(|(_, d)| d)
        .ok_or_else(|| "Device not found".to_string())
}

/// Start the audio stream if not already running
fn ensure_stream_running(
    device_id: &str,
    state: &RecordingState,
    app_handle: AppHandle,
) -> Result<(), String> {
    let needs_start = {
        let audio_state = state.state.lock().unwrap();
        !audio_state.stream_active
    };

    if !needs_start {
        // Check if device changed
        let current_device = state.current_device_id.lock().unwrap();
        if current_device.as_deref() != Some(device_id) {
            return Err("Cannot change device while stream is active".to_string());
        }
        return Ok(());
    }

    let device = get_device_by_id(device_id)?;
    let config = device
        .default_input_config()
        .map_err(|e| format!("Failed to get default config: {}", e))?;

    let sample_rate = config.sample_rate().0;
    let channels = config.channels();
    let sample_format = config.sample_format();

    // Initialize state
    {
        let mut audio_state = state.state.lock().unwrap();
        audio_state.sample_rate = sample_rate;
        audio_state.channels = channels;
        audio_state.stream_active = true;
    }

    // Store current device
    {
        let mut current = state.current_device_id.lock().unwrap();
        *current = Some(device_id.to_string());
    }

    // Reset stop signal
    {
        let mut stop = state.stop_signal.lock().unwrap();
        *stop = false;
    }

    let state_clone = Arc::clone(&state.state);
    let stop_signal = Arc::clone(&state.stop_signal);

    // Spawn audio stream thread
    thread::spawn(move || {
        let stream_config: StreamConfig = config.into();
        let err_fn = |err| eprintln!("Audio stream error: {}", err);

        let state_for_callback = Arc::clone(&state_clone);
        let app_for_callback = app_handle.clone();
        let channels_for_callback = channels;

        let stream_result = match sample_format {
            SampleFormat::F32 => device.build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    process_audio_samples(
                        data,
                        channels_for_callback as usize,
                        &state_for_callback,
                        &app_for_callback,
                    );
                },
                err_fn,
                None,
            ),
            SampleFormat::I16 => {
                let state_for_i16 = Arc::clone(&state_clone);
                let app_for_i16 = app_handle.clone();
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        let float_samples: Vec<f32> =
                            data.iter().map(|&s| s as f32 / 32768.0).collect();
                        process_audio_samples(
                            &float_samples,
                            channels_for_callback as usize,
                            &state_for_i16,
                            &app_for_i16,
                        );
                    },
                    err_fn,
                    None,
                )
            }
            SampleFormat::U16 => {
                let state_for_u16 = Arc::clone(&state_clone);
                let app_for_u16 = app_handle.clone();
                device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        let float_samples: Vec<f32> = data
                            .iter()
                            .map(|&s| (s as f32 - 32768.0) / 32768.0)
                            .collect();
                        process_audio_samples(
                            &float_samples,
                            channels_for_callback as usize,
                            &state_for_u16,
                            &app_for_u16,
                        );
                    },
                    err_fn,
                    None,
                )
            }
            _ => {
                eprintln!("Unsupported sample format: {:?}", sample_format);
                if let Ok(mut s) = state_clone.lock() {
                    s.stream_active = false;
                }
                return;
            }
        };

        let stream = match stream_result {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to build stream: {}", e);
                if let Ok(mut s) = state_clone.lock() {
                    s.stream_active = false;
                    s.is_monitoring = false;
                    s.is_recording = false;
                }
                return;
            }
        };

        if let Err(e) = stream.play() {
            eprintln!("Failed to start stream: {}", e);
            if let Ok(mut s) = state_clone.lock() {
                s.stream_active = false;
                s.is_monitoring = false;
                s.is_recording = false;
            }
            return;
        }

        // Wait for stop signal
        loop {
            thread::sleep(std::time::Duration::from_millis(10));
            if *stop_signal.lock().unwrap() {
                break;
            }
        }

        // Mark stream as inactive
        if let Ok(mut s) = state_clone.lock() {
            s.stream_active = false;
        }

        // Stream is dropped here when thread ends
    });

    Ok(())
}

/// Stop the audio stream if neither monitoring nor recording
fn maybe_stop_stream(state: &RecordingState) {
    let should_stop = {
        let audio_state = state.state.lock().unwrap();
        audio_state.stream_active && !audio_state.is_monitoring && !audio_state.is_recording
    };

    if should_stop {
        // Signal the stream thread to stop
        {
            let mut stop = state.stop_signal.lock().unwrap();
            *stop = true;
        }

        // Clear device (don't wait - let it stop asynchronously)
        {
            let mut current = state.current_device_id.lock().unwrap();
            *current = None;
        }
    }
}

/// Process samples for both recording and visualization
fn process_audio_samples(
    samples: &[f32],
    channels: usize,
    state: &Arc<Mutex<AudioStreamState>>,
    app_handle: &AppHandle,
) {
    // Try to lock without blocking - if we can't get the lock, skip this batch
    if let Ok(mut audio_state) = state.try_lock() {
        // Record samples if recording
        if audio_state.is_recording {
            audio_state.recording_samples.extend_from_slice(samples);
        }

        // Convert to mono if needed (used for visualization and processing)
        let mono_samples: Vec<f32> = if channels > 1 {
            samples
                .chunks(channels)
                .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
                .collect()
        } else {
            samples.to_vec()
        };

        // Process for visualization if monitoring
        if audio_state.is_monitoring {
            audio_state.visualization_samples.extend(&mono_samples);

            // Emit when we have enough samples (~256 samples for low latency)
            const EMIT_THRESHOLD: usize = 256;
            if audio_state.visualization_samples.len() >= EMIT_THRESHOLD {
                let samples_to_emit: Vec<f32> = audio_state.visualization_samples.drain(..).collect();
                let payload = AudioSamplesPayload {
                    samples: samples_to_emit,
                };
                let _ = app_handle.emit("audio-samples", payload);
            }
        }

        // Run audio processor if enabled and monitoring is active
        if audio_state.is_monitoring && audio_state.is_processing_enabled {
            if let Some(ref mut processor) = audio_state.processor {
                processor.process(&mono_samples);
            }
        }
    }
}

/// Start monitoring audio (visualization only)
pub fn start_monitor(
    device_id: &str,
    state: &RecordingState,
    app_handle: AppHandle,
) -> Result<(), String> {
    {
        let audio_state = state.state.lock().unwrap();
        if audio_state.is_monitoring {
            return Err("Already monitoring".to_string());
        }
    }

    // Ensure stream is running
    ensure_stream_running(device_id, state, app_handle)?;

    // Enable monitoring
    {
        let mut audio_state = state.state.lock().unwrap();
        audio_state.visualization_samples.clear();
        audio_state.is_monitoring = true;
    }

    Ok(())
}

/// Stop monitoring
pub fn stop_monitor(state: &RecordingState) -> Result<(), String> {
    {
        let mut audio_state = state.state.lock().unwrap();
        audio_state.is_monitoring = false;
        audio_state.visualization_samples.clear();
    }

    // Stop stream if nothing else needs it
    maybe_stop_stream(state);

    Ok(())
}

/// Start recording (also enables visualization if not already monitoring)
pub fn start_recording(
    device_id: &str,
    state: &RecordingState,
    app_handle: AppHandle,
) -> Result<(), String> {
    {
        let audio_state = state.state.lock().unwrap();
        if audio_state.is_recording {
            return Err("Already recording".to_string());
        }
    }

    // Ensure stream is running
    ensure_stream_running(device_id, state, app_handle)?;

    // Enable recording (and monitoring if not already)
    {
        let mut audio_state = state.state.lock().unwrap();
        audio_state.recording_samples.clear();
        audio_state.is_recording = true;
        // Also enable monitoring for visualization during recording
        if !audio_state.is_monitoring {
            audio_state.visualization_samples.clear();
            audio_state.is_monitoring = true;
        }
    }

    Ok(())
}

/// Stop recording and extract raw audio samples (fast, non-blocking)
/// Returns the raw samples that need to be processed separately
pub fn stop_recording(state: &RecordingState, keep_monitoring: bool) -> Result<RawRecordedAudio, String> {
    // Extract recorded audio and stop recording - this is fast
    let (samples, sample_rate, channels) = {
        let mut audio_state = state.state.lock().unwrap();
        audio_state.is_recording = false;
        
        // If not keeping monitoring, stop it now
        if !keep_monitoring {
            audio_state.is_monitoring = false;
            audio_state.visualization_samples.clear();
        }
        
        let samples = std::mem::take(&mut audio_state.recording_samples);
        (samples, audio_state.sample_rate, audio_state.channels)
    };

    // Stop stream if nothing else needs it (non-blocking)
    maybe_stop_stream(state);

    if samples.is_empty() {
        return Err("No audio recorded".to_string());
    }

    Ok(RawRecordedAudio {
        samples,
        sample_rate,
        channels,
    })
}

/// Process raw recorded audio into format suitable for transcription
/// This is CPU-intensive and should be called in a separate thread/task
pub fn process_recorded_audio(raw: RawRecordedAudio) -> Result<Vec<f32>, String> {
    // Convert to mono if stereo
    let mono_samples = if raw.channels > 1 {
        convert_to_mono(&raw.samples, raw.channels as usize)
    } else {
        raw.samples
    };

    // Resample to 16kHz for Whisper
    resample_to_16khz(&mono_samples, raw.sample_rate)
}

fn convert_to_mono(samples: &[f32], channels: usize) -> Vec<f32> {
    samples
        .chunks(channels)
        .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
        .collect()
}

fn resample_to_16khz(samples: &[f32], source_rate: u32) -> Result<Vec<f32>, String> {
    const TARGET_RATE: u32 = 16000;

    if source_rate == TARGET_RATE {
        return Ok(samples.to_vec());
    }

    let chunk_size = 1024;
    let mut resampler = FftFixedIn::<f32>::new(
        source_rate as usize,
        TARGET_RATE as usize,
        chunk_size,
        2,
        1, // mono
    )
    .map_err(|e| format!("Failed to create resampler: {}", e))?;

    let mut output = Vec::new();

    for chunk in samples.chunks(chunk_size) {
        let mut padded_chunk = chunk.to_vec();
        // Pad last chunk if needed
        if padded_chunk.len() < chunk_size {
            padded_chunk.resize(chunk_size, 0.0);
        }

        let input = vec![padded_chunk];
        let result = resampler
            .process(&input, None)
            .map_err(|e| format!("Resample error: {}", e))?;

        if !result.is_empty() {
            output.extend(&result[0]);
        }
    }

    Ok(output)
}
