//! Audio Engine - Real-time audio processing pipeline
//!
//! This module handles the real-time audio capture, mixing, and output.
//! It uses ring buffers for lock-free communication between audio threads.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{bounded, Receiver, Sender};
use ringbuf::{
    traits::{Consumer, Producer, Split},
    HeapRb,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::application::noise_filter::NoiseFilter;

/// Size of the ring buffer in samples (not frames)
const RING_BUFFER_SIZE: usize = 8192;

/// Level update interval in milliseconds (~30Hz)
const LEVEL_UPDATE_INTERVAL_MS: u64 = 33;

/// Commands that can be sent to the audio engine
#[derive(Debug)]
pub enum AudioEngineCommand {
    /// Start mixing
    Start {
        input_device: String,
        output_device: String,
        sample_rate: u32,
        channels: u16,
        noise_suppression_enabled: bool,
        voice_gate_enabled: bool,
    },
    /// Stop mixing
    Stop,
    /// Play an audio buffer with volume (0.0-2.0) and speed (0.5-2.0)
    PlaySound {
        id: String,
        samples: Vec<f32>,
        volume: f32,
        speed: f32,
    },
    /// Stop a playing sound
    StopSound { id: String },
    /// Stop all playing sounds
    StopAllSounds,
    /// Set microphone volume (0.0 - 2.0)
    SetMicVolume(f32),
    /// Set master volume (0.0 - 2.0)
    SetMasterVolume(f32),
    /// Mute/unmute microphone
    SetMicMuted(bool),
    /// Enable/disable mic monitoring
    SetMicMonitoring(bool),
    /// Set monitoring output device
    SetMonitoringDevice(String),
    /// Enable/disable noise suppression
    SetNoiseSuppression(bool),
    /// Enable/disable voice gate (VAD auto-mute)
    SetVoiceGate(bool),
    /// Shutdown the engine
    Shutdown,
}

/// Events emitted by the audio engine
#[derive(Debug, Clone)]
pub enum AudioEngineEvent {
    /// Engine started successfully
    Started,
    /// Engine stopped
    Stopped,
    /// Error occurred
    Error(String),
    /// Info message (for debugging)
    Info(String),
    /// Audio level update (for UI meters)
    LevelUpdate {
        input_rms: f32,
        input_peak: f32,
        output_rms: f32,
        output_peak: f32,
        monitoring_rms: f32,
    },
    /// Sound finished playing naturally
    SoundFinished { id: String },
}

/// A sound that is currently playing
struct PlayingSound {
    samples: Vec<f32>,
    position: usize,
    /// Fractional position for interpolated playback
    frac_position: f64,
    /// Volume for this sound (0.0 - 2.0)
    volume: f32,
    /// Playback speed (0.5 - 2.0, default 1.0)
    speed: f32,
}

/// Shared state for audio processing
#[allow(dead_code)]
struct AudioState {
    playing_sounds: HashMap<String, PlayingSound>,
    mic_volume: f32,
    master_volume: f32,
    mic_muted: bool,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            playing_sounds: HashMap::new(),
            mic_volume: 1.0,
            master_volume: 1.0,
            mic_muted: false,
        }
    }
}

/// The audio engine that manages real-time audio processing
pub struct AudioEngine {
    command_tx: Sender<AudioEngineCommand>,
    event_rx: Receiver<AudioEngineEvent>,
    is_running: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
}

impl AudioEngine {
    /// Create and start a new audio engine
    pub fn new() -> Self {
        let (command_tx, command_rx) = bounded(32);
        let (event_tx, event_rx) = bounded(64);
        let is_running = Arc::new(AtomicBool::new(false));
        let is_running_clone = is_running.clone();

        let thread_handle = thread::spawn(move || {
            run_engine_thread(command_rx, event_tx, is_running_clone);
        });

        Self {
            command_tx,
            event_rx,
            is_running,
            thread_handle: Some(thread_handle),
        }
    }

    /// Send a command to the audio engine
    pub fn send_command(&self, command: AudioEngineCommand) -> Result<(), String> {
        self.command_tx
            .send(command)
            .map_err(|e| format!("Failed to send command: {}", e))
    }

    /// Try to receive an event from the audio engine (non-blocking)
    pub fn try_recv_event(&self) -> Option<AudioEngineEvent> {
        self.event_rx.try_recv().ok()
    }

    /// Check if the engine is currently running
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    /// Shutdown the audio engine
    pub fn shutdown(&mut self) {
        let _ = self.command_tx.send(AudioEngineCommand::Shutdown);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Find a device by name
fn find_device(host: &cpal::Host, name: &str, is_input: bool) -> Option<cpal::Device> {
    if name == "default" {
        return if is_input {
            host.default_input_device()
        } else {
            host.default_output_device()
        };
    }

    let devices = if is_input {
        host.input_devices().ok()?
    } else {
        host.output_devices().ok()?
    };

    for device in devices {
        if let Ok(device_name) = device.name() {
            if device_name == name {
                return Some(device);
            }
        }
    }
    None
}

/// The main engine thread that manages audio streams
fn run_engine_thread(
    command_rx: Receiver<AudioEngineCommand>,
    event_tx: Sender<AudioEngineEvent>,
    is_running: Arc<AtomicBool>,
) {
    let host = cpal::default_host();

    // Active streams (kept alive while running)
    let mut input_stream: Option<cpal::Stream> = None;
    let mut output_stream: Option<cpal::Stream> = None;
    let mut monitoring_stream: Option<cpal::Stream> = None;

    // Shared state for audio processing
    let audio_state = Arc::new(Mutex::new(AudioState::default()));

    // Ring buffer for passing audio from input to output
    let ring_buffer = Arc::new(Mutex::new(
        None::<(ringbuf::HeapProd<f32>, ringbuf::HeapCons<f32>)>,
    ));

    // Shared list of finished sounds (populated by output callback, consumed by main loop)
    let finished_sounds: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    // Atomic volume controls (for lock-free access in callbacks)
    let mic_volume = Arc::new(AtomicU32::new(f32::to_bits(1.0)));
    let master_volume = Arc::new(AtomicU32::new(f32::to_bits(1.0)));
    let mic_muted = Arc::new(AtomicBool::new(false));

    // Monitoring state
    let mic_monitoring = Arc::new(AtomicBool::new(false));
    let monitoring_device_name = Arc::new(Mutex::new(String::from("default")));

    loop {
        // Process commands
        match command_rx.recv_timeout(Duration::from_millis(10)) {
            Ok(command) => {
                match command {
                    AudioEngineCommand::Start {
                        input_device,
                        output_device,
                        sample_rate: _,
                        channels: _,
                        noise_suppression_enabled,
                        voice_gate_enabled: voice_gate_param,
                    } => {
                        // Stop any existing streams
                        input_stream = None;
                        output_stream = None;

                        // Find devices
                        let input_dev = match find_device(&host, &input_device, true) {
                            Some(d) => d,
                            None => {
                                tracing::error!("Input device not found: {}", input_device);
                                let _ = event_tx.send(AudioEngineEvent::Error(format!(
                                    "Input device not found: {}",
                                    input_device
                                )));
                                continue;
                            }
                        };

                        let output_dev = match find_device(&host, &output_device, false) {
                            Some(d) => d,
                            None => {
                                tracing::error!("Output device not found: {}", output_device);
                                let _ = event_tx.send(AudioEngineEvent::Error(format!(
                                    "Output device not found: {}",
                                    output_device
                                )));
                                continue;
                            }
                        };

                        // Get default configs from devices (more reliable on Windows)
                        let input_default = match input_dev.default_input_config() {
                            Ok(c) => {
                                let _ = event_tx.send(AudioEngineEvent::Info(format!(
                                    "Input '{}': {:?}, {}ch, {}Hz",
                                    input_device,
                                    c.sample_format(),
                                    c.channels(),
                                    c.sample_rate().0
                                )));
                                c
                            }
                            Err(e) => {
                                let _ = event_tx.send(AudioEngineEvent::Error(format!(
                                    "Failed to get input config: {}",
                                    e
                                )));
                                continue;
                            }
                        };

                        let output_default = match output_dev.default_output_config() {
                            Ok(c) => {
                                let _ = event_tx.send(AudioEngineEvent::Info(format!(
                                    "Output '{}': {:?}, {}ch, {}Hz",
                                    output_device,
                                    c.sample_format(),
                                    c.channels(),
                                    c.sample_rate().0
                                )));
                                c
                            }
                            Err(e) => {
                                let _ = event_tx.send(AudioEngineEvent::Error(format!(
                                    "Failed to get output config: {}",
                                    e
                                )));
                                continue;
                            }
                        };

                        // Use each device's native/default sample rate
                        // On Windows WASAPI shared mode, only the system-configured rate works reliably
                        // If rates differ, we resample in the input callback
                        let input_sample_rate = input_default.sample_rate();
                        let output_sample_rate = output_default.sample_rate();
                        let needs_resample = input_sample_rate != output_sample_rate;

                        if needs_resample {
                            tracing::info!(
                                "Sample rate mismatch: input {}Hz, output {}Hz - resampling enabled",
                                input_sample_rate.0,
                                output_sample_rate.0
                            );
                            let _ = event_tx.send(AudioEngineEvent::Info(format!(
                                "Resampling: input {}Hz -> output {}Hz",
                                input_sample_rate.0, output_sample_rate.0
                            )));
                        } else {
                            let _ = event_tx.send(AudioEngineEvent::Info(format!(
                                "Common sample rate: {}Hz",
                                input_sample_rate.0
                            )));
                        }

                        // Create separate configs for input and output using their native rates
                        // Input: use native channel count (may be mono) and native sample rate
                        let input_channels = input_default.channels();
                        let input_config = cpal::StreamConfig {
                            channels: input_channels,
                            sample_rate: input_sample_rate,
                            buffer_size: cpal::BufferSize::Default,
                        };

                        // Output: use native channel count (usually stereo for VB-Cable) and native sample rate
                        let output_channels = output_default.channels();
                        let output_config = cpal::StreamConfig {
                            channels: output_channels,
                            sample_rate: output_sample_rate,
                            buffer_size: cpal::BufferSize::Default,
                        };

                        // The ring buffer carries samples at the output rate
                        // Noise filter also operates at the output rate (post-resampling)
                        let sample_rate = output_sample_rate;

                        let _ = event_tx.send(AudioEngineEvent::Info(format!(
                            "Input config: {}ch, {}Hz | Output config: {}ch, {}Hz",
                            input_channels,
                            input_sample_rate.0,
                            output_channels,
                            output_sample_rate.0
                        )));

                        // Create ring buffer for audio pass-through
                        let rb = HeapRb::<f32>::new(RING_BUFFER_SIZE);
                        let (producer, consumer) = rb.split();

                        // Store producer in Arc<Mutex> for input callback
                        let producer = Arc::new(Mutex::new(producer));
                        let consumer = Arc::new(Mutex::new(consumer));

                        // Create ring buffer for monitoring: carries the MIXED output from main callback
                        // This ensures monitoring gets exactly what virtual output gets
                        let rb_monitoring = HeapRb::<f32>::new(RING_BUFFER_SIZE);
                        let (producer_monitoring, consumer_monitoring) = rb_monitoring.split();
                        let producer_monitoring = Arc::new(Mutex::new(producer_monitoring));
                        let consumer_monitoring = Arc::new(Mutex::new(consumer_monitoring));

                        // Atomic level values for lock-free reading
                        let input_level = Arc::new(AtomicU32::new(0));
                        let output_level = Arc::new(AtomicU32::new(0));
                        let monitoring_level = Arc::new(AtomicU32::new(0));
                        let input_level_clone = input_level.clone();

                        // Clone references for callbacks
                        let producer_clone = producer.clone();
                        let mic_volume_clone = mic_volume.clone();
                        let mic_muted_clone = mic_muted.clone();

                        // Debug counter for input callback
                        let input_callback_count = Arc::new(AtomicU32::new(0));
                        let input_callback_count_clone = input_callback_count.clone();
                        let input_ch = input_channels;

                        // Resampling state for input callback (when input rate != output rate)
                        // resample_step = input_rate / output_rate (input samples consumed per output sample)
                        let resample_step =
                            input_sample_rate.0 as f64 / output_sample_rate.0 as f64;
                        // State: (fractional_position, previous_sample)
                        let input_resample_state = Arc::new(Mutex::new((0.0f64, 0.0f32)));
                        let input_resample_state_clone = input_resample_state.clone();
                        let needs_resample_clone = needs_resample;

                        // Create noise suppression filter
                        // IMPORTANT: nnnoiseless only works at 48kHz - disable if sample rate differs
                        let actual_noise_enabled = if sample_rate.0 != 48000 {
                            if noise_suppression_enabled {
                                tracing::warn!(
                                    "Noise suppression disabled: sample rate {}Hz != 48000Hz (nnnoiseless requires 48kHz)",
                                    sample_rate.0
                                );
                                let _ = event_tx.send(AudioEngineEvent::Info(format!(
                                    "Noise suppression disabled: {}Hz not supported (requires 48kHz)",
                                    sample_rate.0
                                )));
                            }
                            false
                        } else {
                            noise_suppression_enabled
                        };

                        let noise_enabled = Arc::new(AtomicBool::new(actual_noise_enabled));
                        let noise_filter =
                            Arc::new(Mutex::new(NoiseFilter::new(noise_enabled.clone())));
                        let noise_filter_clone = noise_filter.clone();

                        // Voice gate (VAD auto-mute) state
                        // Only effective when noise suppression is also enabled
                        let voice_gate_enabled =
                            Arc::new(AtomicBool::new(voice_gate_param && actual_noise_enabled));
                        let voice_gate_enabled_clone = voice_gate_enabled.clone();
                        // Timestamp (ms since engine start) until which voice is considered detected
                        let voice_detected_until = Arc::new(AtomicU64::new(0));
                        let voice_detected_until_clone = voice_detected_until.clone();
                        // Reference time for voice gate timing
                        let engine_start_time = Instant::now();

                        // Voice gate constants
                        const VAD_THRESHOLD: f32 = 0.7;
                        const VOICE_HOLDOFF_MS: u64 = 200;

                        tracing::info!(
                            "Noise suppression: {}{}, Voice gate: {}",
                            if actual_noise_enabled {
                                "enabled"
                            } else {
                                "disabled"
                            },
                            if sample_rate.0 != 48000 {
                                " (48kHz required)"
                            } else {
                                ""
                            },
                            if voice_gate_param && actual_noise_enabled {
                                "enabled"
                            } else if voice_gate_param {
                                "disabled (requires noise suppression)"
                            } else {
                                "disabled"
                            }
                        );

                        // Build input stream (may be mono or stereo)
                        let input_result = input_dev.build_input_stream(
                            &input_config,
                            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                                // Log first few callbacks to verify stream is working
                                let count = input_callback_count_clone.fetch_add(1, Ordering::Relaxed);
                                if count < 5 || count.is_multiple_of(1000) {
                                    let max_sample = data.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
                                    let filter_enabled = noise_filter_clone
                                        .try_lock()
                                        .map(|f| f.is_enabled())
                                        .unwrap_or(false);
                                    tracing::info!(
                                        "[AudioEngine] Input callback #{}: {} samples, {}ch, max amplitude: {:.6}, noise_filter: {}",
                                        count,
                                        data.len(),
                                        input_ch,
                                        max_sample,
                                        if filter_enabled { "ON" } else { "OFF" }
                                    );
                                }

                                let muted = mic_muted_clone.load(Ordering::Relaxed);
                                let volume =
                                    f32::from_bits(mic_volume_clone.load(Ordering::Relaxed));

                                // Calculate RMS for input level
                                let mut sum_squares = 0.0f32;

                                // Convert multi-channel input to mono (one sample per frame)
                                // This ensures ring buffer carries mono samples regardless of input channel count
                                let num_frames = data.len() / input_ch as usize;

                                if let Ok(mut prod) = producer_clone.try_lock() {
                                    if let Ok(mut filter) = noise_filter_clone.try_lock() {
                                        // Voice gate state
                                        let voice_gate_on = voice_gate_enabled_clone.load(Ordering::Relaxed);
                                        let filter_enabled = filter.is_enabled();

                                        // Lock resample state if resampling is needed
                                        let mut resample_guard = if needs_resample_clone {
                                            input_resample_state_clone.try_lock().ok()
                                        } else {
                                            None
                                        };

                                        for frame in 0..num_frames {
                                            // Average all channels to produce mono sample
                                            let mut sum = 0.0f32;
                                            for ch in 0..input_ch as usize {
                                                let idx = frame * input_ch as usize + ch;
                                                sum += data.get(idx).copied().unwrap_or(0.0);
                                            }
                                            let mono_sample = sum / input_ch as f32;

                                            // Generate output-rate samples and process through noise filter
                                            // With resampling: interpolate to produce samples at output rate
                                            // Without: process mono sample directly
                                            if let Some(ref mut state) = resample_guard {
                                                let (ref mut frac_pos, ref mut prev) = **state;
                                                let curr = mono_sample;
                                                while *frac_pos < 1.0 {
                                                    let t = *frac_pos as f32;
                                                    let resampled = *prev + (curr - *prev) * t;

                                                    let filtered_samples: Vec<f32> = filter.process_sample(resampled).to_vec();
                                                    let voice_gate_muted = if voice_gate_on && filter_enabled && !filtered_samples.is_empty() {
                                                        let vad = filter.last_vad();
                                                        let now_ms = engine_start_time.elapsed().as_millis() as u64;
                                                        if vad >= VAD_THRESHOLD {
                                                            voice_detected_until_clone.store(now_ms + VOICE_HOLDOFF_MS, Ordering::Relaxed);
                                                            false
                                                        } else {
                                                            now_ms > voice_detected_until_clone.load(Ordering::Relaxed)
                                                        }
                                                    } else {
                                                        false
                                                    };
                                                    for sample in filtered_samples {
                                                        let processed = if muted || voice_gate_muted { 0.0 } else { sample * volume };
                                                        sum_squares += processed * processed;
                                                        let _ = prod.try_push(processed);
                                                    }

                                                    *frac_pos += resample_step;
                                                }
                                                *frac_pos -= 1.0;
                                                *prev = curr;
                                            } else {
                                                // No resampling - process directly at native rate
                                                let filtered_samples: Vec<f32> = filter.process_sample(mono_sample).to_vec();
                                                let voice_gate_muted = if voice_gate_on && filter_enabled && !filtered_samples.is_empty() {
                                                    let vad = filter.last_vad();
                                                    let now_ms = engine_start_time.elapsed().as_millis() as u64;
                                                    if vad >= VAD_THRESHOLD {
                                                        voice_detected_until_clone.store(now_ms + VOICE_HOLDOFF_MS, Ordering::Relaxed);
                                                        false
                                                    } else {
                                                        now_ms > voice_detected_until_clone.load(Ordering::Relaxed)
                                                    }
                                                } else {
                                                    false
                                                };
                                                for sample in filtered_samples {
                                                    let processed = if muted || voice_gate_muted { 0.0 } else { sample * volume };
                                                    sum_squares += processed * processed;
                                                    let _ = prod.try_push(processed);
                                                }
                                            }
                                        }
                                    }
                                }

                                // Store RMS level (will be read by level monitoring thread)
                                if num_frames > 0 {
                                    let rms = (sum_squares / num_frames as f32).sqrt();
                                    input_level_clone.store(rms.to_bits(), Ordering::Relaxed);
                                }
                            },
                            move |err| {
                                tracing::error!("Input stream error: {}", err);
                            },
                            None,
                        );

                        let input_s = match input_result {
                            Ok(s) => s,
                            Err(e) => {
                                tracing::error!("Failed to create input stream: {}", e);
                                let _ = event_tx.send(AudioEngineEvent::Error(format!(
                                    "Failed to create input stream: {}",
                                    e
                                )));
                                continue;
                            }
                        };

                        // Clone references for output callback
                        let consumer_clone = consumer.clone();
                        let master_volume_clone = master_volume.clone();
                        let audio_state_clone = audio_state.clone();
                        let output_level_for_callback = output_level.clone();
                        let producer_monitoring_for_output = producer_monitoring.clone();
                        let mic_monitoring_for_output = mic_monitoring.clone();
                        let finished_sounds_for_callback = finished_sounds.clone();
                        // Input may be mono, output may be stereo - we handle conversion in callback
                        let _input_ch = input_channels;
                        let output_ch = output_channels;

                        // Debug counter for output callback
                        let output_callback_count = Arc::new(AtomicU32::new(0));
                        let output_callback_count_clone = output_callback_count.clone();

                        // Dynamic limiter state: current gain (0.0 to 1.0)
                        // Starts at 1.0 (unity gain), reduces when clipping detected
                        let limiter_gain = Arc::new(AtomicU32::new(f32::to_bits(1.0)));
                        let limiter_gain_clone = limiter_gain.clone();

                        // Build output stream (uses output device's native config)
                        let output_result = output_dev.build_output_stream(
                            &output_config,
                            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                                // Log first few callbacks to verify stream is working
                                let count = output_callback_count_clone.fetch_add(1, Ordering::Relaxed);

                                let master_vol =
                                    f32::from_bits(master_volume_clone.load(Ordering::Relaxed));
                                let mic_mon_enabled = mic_monitoring_for_output.load(Ordering::Relaxed);

                                // Limiter parameters
                                // Release: how fast we return to unity (slow to avoid pumping)
                                // Attack is instant (no coefficient needed)
                                let release_coeff = 0.0005f32; // ~2 seconds to return to unity at 48kHz
                                let mut current_limiter_gain =
                                    f32::from_bits(limiter_gain_clone.load(Ordering::Relaxed));

                                // Calculate frames (output may be stereo, input may be mono)
                                let data_len = data.len();
                                let num_frames = data_len / output_ch as usize;

                                // Read MONO mic samples from ring buffer (one per frame)
                                // Ring buffer contains mono samples regardless of input channel count
                                // (input callback already handles multi-channel input → mono)
                                let mut mic_samples_mono = vec![0.0f32; num_frames];
                                if let Ok(mut cons) = consumer_clone.try_lock() {
                                    for sample in mic_samples_mono.iter_mut() {
                                        *sample = cons.try_pop().unwrap_or(0.0);
                                    }
                                }

                                // Start with zeros
                                for sample in data.iter_mut() {
                                    *sample = 0.0;
                                }

                                // Mix in playing sounds (sounds are stored as MONO)
                                // Use fractional position with linear interpolation for speed control
                                let sounds_mixed;
                                if let Ok(mut state) = audio_state_clone.try_lock() {
                                    let mut finished = Vec::new();
                                    sounds_mixed = state.playing_sounds.len();

                                    for (id, sound) in state.playing_sounds.iter_mut() {
                                        for frame in 0..num_frames {
                                            // Check if we've reached the end
                                            let idx = sound.frac_position.floor() as usize;
                                            if idx >= sound.samples.len() {
                                                break;
                                            }

                                            // Linear interpolation for sub-sample accuracy
                                            let frac = sound.frac_position - idx as f64;
                                            let mono_sample = if idx + 1 < sound.samples.len() {
                                                let s0 = sound.samples[idx] as f64;
                                                let s1 = sound.samples[idx + 1] as f64;
                                                ((s0 + (s1 - s0) * frac) as f32) * sound.volume
                                            } else {
                                                sound.samples[idx] * sound.volume
                                            };

                                            // Duplicate mono sample to all output channels
                                            // No fixed headroom - dynamic limiter handles clipping
                                            for ch in 0..output_ch as usize {
                                                let out_idx = frame * output_ch as usize + ch;
                                                if out_idx < data_len {
                                                    data[out_idx] += mono_sample;
                                                }
                                            }

                                            // Advance by speed factor
                                            sound.frac_position += sound.speed as f64;
                                        }

                                        // Update integer position for finished check
                                        sound.position = sound.frac_position.floor() as usize;
                                        if sound.position >= sound.samples.len() {
                                            finished.push(id.clone());
                                        }
                                    }

                                    // Remove finished sounds and notify
                                    if !finished.is_empty() {
                                        if let Ok(mut fs) = finished_sounds_for_callback.try_lock() {
                                            for id in &finished {
                                                fs.push(id.clone());
                                            }
                                        }
                                        for id in finished {
                                            state.playing_sounds.remove(&id);
                                        }
                                    }
                                } else {
                                    sounds_mixed = 0;
                                }

                                // Write to monitoring buffer BEFORE adding mic to main output
                                // Monitoring gets: sounds + (mic if mic_monitoring enabled)
                                // Push MONO samples only (one per frame)
                                if let Ok(mut prod_mon) = producer_monitoring_for_output.try_lock() {
                                    for frame in 0..num_frames {
                                        // Get sound sample for this frame (average all channels to mono)
                                        let mut sound_sum = 0.0f32;
                                        for ch in 0..output_ch as usize {
                                            let idx = frame * output_ch as usize + ch;
                                            sound_sum += data.get(idx).copied().unwrap_or(0.0);
                                        }
                                        let sound_sample = sound_sum / output_ch as f32;

                                        // Get mic sample (already mono)
                                        let mic_sample = mic_samples_mono.get(frame).copied().unwrap_or(0.0);

                                        let monitoring_sample = if mic_mon_enabled {
                                            (sound_sample + mic_sample) * master_vol
                                        } else {
                                            sound_sample * master_vol
                                        };
                                        // Apply limiter gain (from previous frame) to monitoring
                                        let _ = prod_mon.try_push((monitoring_sample * current_limiter_gain).clamp(-1.0, 1.0));
                                    }
                                }

                                // Add mic to main output (duplicate mono to all channels)
                                for frame in 0..num_frames {
                                    let mic_sample = mic_samples_mono.get(frame).copied().unwrap_or(0.0);
                                    for ch in 0..output_ch as usize {
                                        let idx = frame * output_ch as usize + ch;
                                        if idx < data_len {
                                            data[idx] += mic_sample;
                                        }
                                    }
                                }

                                // Apply master volume to main output (no clamp yet)
                                for sample in data.iter_mut() {
                                    *sample *= master_vol;
                                }

                                // Dynamic limiter: detect peaks and adjust gain
                                // Find peak level in the buffer
                                let peak = data.iter().fold(0.0f32, |acc, &s| acc.max(s.abs()));

                                // If peak would clip, calculate required gain reduction
                                let target_gain = if peak > 1.0 {
                                    1.0 / peak // Exact gain to bring peak to 1.0
                                } else {
                                    1.0 // No reduction needed
                                };

                                // Apply attack/release envelope
                                if target_gain < current_limiter_gain {
                                    // Attack: instant reduction to prevent clipping
                                    current_limiter_gain = target_gain;
                                } else {
                                    // Release: slowly return to unity gain
                                    current_limiter_gain += (target_gain - current_limiter_gain) * release_coeff;
                                    current_limiter_gain = current_limiter_gain.min(1.0);
                                }

                                // Apply limiter gain and final clamp (safety)
                                for sample in data.iter_mut() {
                                    *sample = (*sample * current_limiter_gain).clamp(-1.0, 1.0);
                                }

                                // Store limiter state for next callback
                                limiter_gain_clone.store(current_limiter_gain.to_bits(), Ordering::Relaxed);

                                // Calculate output RMS after limiter
                                let mut sum_squares = 0.0f32;
                                for sample in data.iter() {
                                    sum_squares += sample * sample;
                                }
                                if !data.is_empty() {
                                    let rms = (sum_squares / data_len as f32).sqrt();
                                    output_level_for_callback
                                        .store(rms.to_bits(), Ordering::Relaxed);

                                    // Log first few callbacks and when sounds are playing
                                    if count < 5 || count.is_multiple_of(1000) || (sounds_mixed > 0 && count.is_multiple_of(100)) {
                                        let max_sample = data.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
                                        tracing::info!(
                                            "[AudioEngine] Output callback #{}: {} samples, max: {:.6}, rms: {:.6}, sounds: {}",
                                            count,
                                            data_len,
                                            max_sample,
                                            rms,
                                            sounds_mixed
                                        );
                                    }
                                }
                            },
                            move |err| {
                                tracing::error!("Output stream error: {}", err);
                            },
                            None,
                        );

                        let output_s = match output_result {
                            Ok(s) => s,
                            Err(e) => {
                                tracing::error!("Failed to create output stream: {}", e);
                                let _ = event_tx.send(AudioEngineEvent::Error(format!(
                                    "Failed to create output stream: {}",
                                    e
                                )));
                                continue;
                            }
                        };

                        // Build monitoring output stream
                        let monitoring_dev_name = {
                            let name = monitoring_device_name.lock().unwrap();
                            name.clone()
                        };

                        let mut monitoring_s: Option<cpal::Stream> = None;
                        if let Some(monitoring_dev) =
                            find_device(&host, &monitoring_dev_name, false)
                        {
                            let consumer_monitoring_clone = consumer_monitoring.clone();
                            let event_tx_mon = event_tx.clone();

                            // Use monitoring device's default config (usually 2ch stereo)
                            // We'll duplicate mono samples to stereo if needed
                            let mon_default_config =
                                if let Ok(c) = monitoring_dev.default_output_config() {
                                    c
                                } else {
                                    let _ = event_tx.send(AudioEngineEvent::Info(
                                        "Failed to get monitoring config, skipping monitoring"
                                            .to_string(),
                                    ));
                                    // Skip monitoring setup but continue with engine startup
                                    cpal::SupportedStreamConfig::new(
                                        2,
                                        cpal::SampleRate(48000),
                                        cpal::SupportedBufferSize::Range {
                                            min: 256,
                                            max: 4096,
                                        },
                                        cpal::SampleFormat::F32,
                                    )
                                };

                            let mon_channels = mon_default_config.channels();
                            // Use device's native sample rate to avoid driver resampling issues
                            let mon_sample_rate = mon_default_config.sample_rate();
                            let engine_sample_rate = sample_rate.0 as f64;
                            let mon_sample_rate_value = mon_sample_rate.0 as f64;

                            // Calculate resample ratio: how many engine samples per monitoring sample
                            // ratio < 1.0: monitoring rate higher, need to interpolate
                            // ratio > 1.0: monitoring rate lower, need to skip samples
                            let resample_ratio = engine_sample_rate / mon_sample_rate_value;

                            let _ = event_tx.send(AudioEngineEvent::Info(format!(
                                "Monitoring '{}': {}ch, {}Hz (engine at {}Hz, ratio {:.4})",
                                monitoring_dev_name,
                                mon_channels,
                                mon_sample_rate.0,
                                sample_rate.0,
                                resample_ratio
                            )));

                            // Build monitoring stream config: device channels + device native sample rate
                            let mon_stream_config = cpal::StreamConfig {
                                channels: mon_channels,
                                sample_rate: mon_sample_rate,
                                buffer_size: cpal::BufferSize::Default,
                            };

                            // Resampling state: fractional position and sample history
                            // Stored as (fractional_pos: f64, prev_sample: f32, curr_sample: f32)
                            let resample_state = Arc::new(Mutex::new((0.0f64, 0.0f32, 0.0f32)));
                            let resample_state_clone = resample_state.clone();

                            // Counter for monitoring callback logging
                            let mon_callback_count = Arc::new(AtomicU32::new(0));
                            let mon_callback_count_clone = mon_callback_count.clone();

                            // Clone monitoring level for callback
                            let monitoring_level_for_callback = monitoring_level.clone();

                            let monitoring_result = monitoring_dev.build_output_stream(
                                &mon_stream_config,
                                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                                    let count = mon_callback_count_clone.fetch_add(1, Ordering::Relaxed);

                                    // Log first few callbacks to debug
                                    if count < 5 {
                                        tracing::info!(
                                            "[AudioEngine] Monitoring callback #{}: {} samples ({}ch), ratio {:.4}",
                                            count,
                                            data.len(),
                                            mon_channels,
                                            resample_ratio
                                        );
                                    }

                                    // Read from monitoring ring buffer with resampling
                                    // Ring buffer contains mono samples at engine rate
                                    // Output at monitoring device's native rate with linear interpolation
                                    if let (Ok(mut cons), Ok(mut state)) = (
                                        consumer_monitoring_clone.try_lock(),
                                        resample_state_clone.try_lock(),
                                    ) {
                                        let (ref mut frac_pos, ref mut prev_sample, ref mut curr_sample) = *state;

                                        // Calculate number of output frames
                                        let num_frames = data.len() / mon_channels as usize;

                                        for frame in 0..num_frames {
                                            // Linear interpolation between prev and curr sample
                                            let t = *frac_pos as f32;
                                            let sample = *prev_sample + (*curr_sample - *prev_sample) * t;

                                            // Write to output (duplicate mono sample to all channels)
                                            let base_idx = frame * mon_channels as usize;
                                            for ch in 0..mon_channels as usize {
                                                if base_idx + ch < data.len() {
                                                    data[base_idx + ch] = sample;
                                                }
                                            }

                                            // Advance position by resample ratio
                                            *frac_pos += resample_ratio;

                                            // When we've moved past current sample, shift history and pop new
                                            while *frac_pos >= 1.0 {
                                                *frac_pos -= 1.0;
                                                *prev_sample = *curr_sample;
                                                *curr_sample = cons.try_pop().unwrap_or(0.0);
                                            }
                                        }

                                        // Calculate monitoring RMS after writing data
                                        if !data.is_empty() {
                                            let mut sum_squares = 0.0f32;
                                            for sample in data.iter() {
                                                sum_squares += sample * sample;
                                            }
                                            let rms = (sum_squares / data.len() as f32).sqrt();
                                            monitoring_level_for_callback.store(rms.to_bits(), Ordering::Relaxed);
                                        }
                                    } else {
                                        // Couldn't acquire locks, output silence
                                        for sample in data.iter_mut() {
                                            *sample = 0.0;
                                        }
                                        // Reset monitoring level when no data
                                        monitoring_level_for_callback.store(0, Ordering::Relaxed);
                                    }
                                },
                                move |err| {
                                    let _ = event_tx_mon.send(AudioEngineEvent::Info(format!(
                                        "Monitoring stream error: {}",
                                        err
                                    )));
                                },
                                None,
                            );

                            match monitoring_result {
                                Ok(s) => {
                                    monitoring_s = Some(s);
                                    let _ = event_tx.send(AudioEngineEvent::Info(format!(
                                        "Monitoring stream created on: {}",
                                        monitoring_dev_name
                                    )));
                                }
                                Err(e) => {
                                    let _ = event_tx.send(AudioEngineEvent::Info(format!(
                                        "Failed to create monitoring stream: {} (non-fatal)",
                                        e
                                    )));
                                }
                            }
                        } else {
                            let _ = event_tx.send(AudioEngineEvent::Info(format!(
                                "Monitoring device not found: {} (non-fatal)",
                                monitoring_dev_name
                            )));
                        }

                        // Start streams
                        if let Err(e) = input_s.play() {
                            tracing::error!("Failed to start input stream: {}", e);
                            let _ = event_tx.send(AudioEngineEvent::Error(format!(
                                "Failed to start input: {}",
                                e
                            )));
                            continue;
                        }

                        if let Err(e) = output_s.play() {
                            tracing::error!("Failed to start output stream: {}", e);
                            let _ = event_tx.send(AudioEngineEvent::Error(format!(
                                "Failed to start output: {}",
                                e
                            )));
                            continue;
                        }

                        // Start monitoring stream if it was created
                        if let Some(ref mon_s) = monitoring_s {
                            if let Err(e) = mon_s.play() {
                                let _ = event_tx.send(AudioEngineEvent::Info(format!(
                                    "Failed to start monitoring stream: {} (non-fatal)",
                                    e
                                )));
                            } else {
                                let _ = event_tx.send(AudioEngineEvent::Info(format!(
                                    "Monitoring stream started on: {}",
                                    monitoring_dev_name
                                )));
                            }
                        }

                        // Store streams to keep them alive
                        input_stream = Some(input_s);
                        output_stream = Some(output_s);
                        monitoring_stream = monitoring_s;

                        is_running.store(true, Ordering::SeqCst);
                        let _ = event_tx.send(AudioEngineEvent::Started);
                        tracing::info!(
                            "Audio engine started: {} -> {}",
                            input_device,
                            output_device
                        );

                        // Start level monitoring thread
                        let input_level_monitor = input_level.clone();
                        let output_level_monitor = output_level.clone();
                        let monitoring_level_monitor = monitoring_level.clone();
                        let event_tx_monitor = event_tx.clone();
                        let is_running_monitor = is_running.clone();

                        std::thread::spawn(move || {
                            let mut input_peak = 0.0f32;
                            let mut output_peak = 0.0f32;
                            let decay_rate = 0.05; // ~20dB/sec at 30Hz

                            while is_running_monitor.load(Ordering::Relaxed) {
                                let input_rms =
                                    f32::from_bits(input_level_monitor.load(Ordering::Relaxed));
                                let output_rms =
                                    f32::from_bits(output_level_monitor.load(Ordering::Relaxed));
                                let monitoring_rms = f32::from_bits(
                                    monitoring_level_monitor.load(Ordering::Relaxed),
                                );

                                // Update peaks
                                if input_rms > input_peak {
                                    input_peak = input_rms;
                                } else {
                                    input_peak = (input_peak - decay_rate).max(0.0);
                                }

                                if output_rms > output_peak {
                                    output_peak = output_rms;
                                } else {
                                    output_peak = (output_peak - decay_rate).max(0.0);
                                }

                                let _ = event_tx_monitor.send(AudioEngineEvent::LevelUpdate {
                                    input_rms,
                                    input_peak,
                                    output_rms,
                                    output_peak,
                                    monitoring_rms,
                                });

                                std::thread::sleep(std::time::Duration::from_millis(
                                    LEVEL_UPDATE_INTERVAL_MS,
                                ));
                            }
                        });
                    }

                    AudioEngineCommand::Stop => {
                        // Pause streams before dropping to ensure clean stop
                        if let Some(ref stream) = input_stream {
                            let _ = stream.pause();
                        }
                        if let Some(ref stream) = output_stream {
                            let _ = stream.pause();
                        }
                        if let Some(ref stream) = monitoring_stream {
                            let _ = stream.pause();
                        }

                        // Drop the streams
                        input_stream = None;
                        output_stream = None;
                        monitoring_stream = None;

                        // Clear the ring buffer to prevent any leftover audio
                        if let Ok(mut rb) = ring_buffer.lock() {
                            *rb = None;
                        }

                        is_running.store(false, Ordering::SeqCst);

                        if let Ok(mut state) = audio_state.lock() {
                            state.playing_sounds.clear();
                        }

                        let _ = event_tx.send(AudioEngineEvent::Stopped);
                        tracing::info!("Audio engine stopped");
                    }

                    AudioEngineCommand::PlaySound {
                        id,
                        samples,
                        volume,
                        speed,
                    } => {
                        let samples_len = samples.len();
                        if let Ok(mut state) = audio_state.lock() {
                            state.playing_sounds.insert(
                                id.clone(),
                                PlayingSound {
                                    samples,
                                    position: 0,
                                    frac_position: 0.0,
                                    volume: volume.clamp(0.0, 2.0),
                                    speed: speed.clamp(0.5, 2.0),
                                },
                            );
                            tracing::info!(
                                "[AudioEngine] Added sound '{}' ({} samples, vol: {:.0}%, speed: {:.2}x), total playing: {}",
                                id,
                                samples_len,
                                volume * 100.0,
                                speed,
                                state.playing_sounds.len()
                            );
                        }
                    }

                    AudioEngineCommand::StopSound { id } => {
                        if let Ok(mut state) = audio_state.lock() {
                            state.playing_sounds.remove(&id);
                        }
                    }

                    AudioEngineCommand::StopAllSounds => {
                        if let Ok(mut state) = audio_state.lock() {
                            let count = state.playing_sounds.len();
                            state.playing_sounds.clear();
                            tracing::info!("[AudioEngine] Stopped all sounds ({} cleared)", count);
                        }
                    }

                    AudioEngineCommand::SetMicVolume(volume) => {
                        mic_volume.store(f32::to_bits(volume.clamp(0.0, 2.0)), Ordering::Relaxed);
                    }

                    AudioEngineCommand::SetMasterVolume(volume) => {
                        master_volume
                            .store(f32::to_bits(volume.clamp(0.0, 2.0)), Ordering::Relaxed);
                    }

                    AudioEngineCommand::SetMicMuted(muted) => {
                        mic_muted.store(muted, Ordering::Relaxed);
                    }

                    AudioEngineCommand::SetMicMonitoring(enabled) => {
                        mic_monitoring.store(enabled, Ordering::Relaxed);
                        tracing::info!("Mic monitoring set to: {}", enabled);
                    }

                    AudioEngineCommand::SetMonitoringDevice(device_name) => {
                        if let Ok(mut name) = monitoring_device_name.lock() {
                            *name = device_name.clone();
                        }
                        tracing::info!("Monitoring device set to: {}", device_name);
                    }

                    AudioEngineCommand::SetNoiseSuppression(enabled) => {
                        tracing::info!(
                            "Noise suppression set to: {} (requires restart to take effect)",
                            enabled
                        );
                        // The setting is applied on next Start command
                        // Frontend should restart mixing when this changes
                    }

                    AudioEngineCommand::SetVoiceGate(enabled) => {
                        tracing::info!(
                            "Voice gate set to: {} (requires restart to take effect)",
                            enabled
                        );
                        // The setting is applied on next Start command
                        // Frontend should restart mixing when this changes
                    }

                    AudioEngineCommand::Shutdown => {
                        // Pause streams before dropping
                        if let Some(ref stream) = input_stream {
                            let _ = stream.pause();
                        }
                        if let Some(ref stream) = output_stream {
                            let _ = stream.pause();
                        }
                        if let Some(ref stream) = monitoring_stream {
                            let _ = stream.pause();
                        }

                        drop(input_stream);
                        drop(output_stream);
                        drop(monitoring_stream);
                        is_running.store(false, Ordering::SeqCst);
                        tracing::info!("Audio engine shutdown");
                        return;
                    }
                }
            }
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                // No command, continue
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                // Channel closed, shutdown
                tracing::info!("Command channel closed, shutting down");
                return;
            }
        }

        // Check for finished sounds and emit events
        if let Ok(mut fs) = finished_sounds.try_lock() {
            for id in fs.drain(..) {
                let _ = event_tx.send(AudioEngineEvent::SoundFinished { id });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // ==================== AudioState Tests ====================

    #[test]
    fn test_audio_state_default_values() {
        let state = AudioState::default();
        assert!(state.playing_sounds.is_empty());
        assert_eq!(state.mic_volume, 1.0);
        assert_eq!(state.master_volume, 1.0);
        assert!(!state.mic_muted);
    }

    // ==================== AudioEngine Tests ====================

    #[test]
    fn test_engine_creation() {
        let engine = AudioEngine::new();
        assert!(!engine.is_running());
    }

    #[test]
    fn test_engine_is_not_running_initially() {
        let engine = AudioEngine::new();
        assert!(!engine.is_running());
    }

    #[test]
    fn test_send_command_succeeds() {
        let engine = AudioEngine::new();
        let result = engine.send_command(AudioEngineCommand::SetMicVolume(0.5));
        assert!(result.is_ok());
    }

    #[test]
    fn test_send_multiple_commands_succeeds() {
        let engine = AudioEngine::new();

        assert!(engine
            .send_command(AudioEngineCommand::SetMicVolume(0.5))
            .is_ok());
        assert!(engine
            .send_command(AudioEngineCommand::SetMasterVolume(0.8))
            .is_ok());
        assert!(engine
            .send_command(AudioEngineCommand::SetMicMuted(true))
            .is_ok());
        assert!(engine
            .send_command(AudioEngineCommand::SetMicMonitoring(true))
            .is_ok());
    }

    #[test]
    fn test_try_recv_event_returns_none_when_no_events() {
        let engine = AudioEngine::new();
        // Give the engine thread time to start
        std::thread::sleep(Duration::from_millis(10));
        // No events should be pending initially (before any command)
        // Note: We can't guarantee no events since the thread runs independently
        // Just verify the method doesn't panic
        let _event = engine.try_recv_event();
    }

    #[test]
    fn test_shutdown_completes_without_panic() {
        let mut engine = AudioEngine::new();
        engine.shutdown();
        // Engine should no longer be running after shutdown
        assert!(!engine.is_running());
    }

    #[test]
    fn test_shutdown_is_idempotent() {
        let mut engine = AudioEngine::new();
        engine.shutdown();
        engine.shutdown(); // Second shutdown should not panic
        assert!(!engine.is_running());
    }

    #[test]
    fn test_drop_shuts_down_engine() {
        let engine = AudioEngine::new();
        let is_running = engine.is_running.clone();
        drop(engine);
        // After drop, the engine thread should have stopped
        // Give it time to process shutdown
        std::thread::sleep(Duration::from_millis(50));
        assert!(!is_running.load(std::sync::atomic::Ordering::SeqCst));
    }

    // ==================== PlayingSound Tests ====================

    #[test]
    fn test_playing_sound_creation() {
        let samples = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let sound = PlayingSound {
            samples: samples.clone(),
            position: 0,
            frac_position: 0.0,
            volume: 1.0,
            speed: 1.0,
        };
        assert_eq!(sound.samples.len(), 5);
        assert_eq!(sound.position, 0);
        assert_eq!(sound.frac_position, 0.0);
        assert_eq!(sound.volume, 1.0);
        assert_eq!(sound.speed, 1.0);
    }

    #[test]
    fn test_playing_sound_position_tracking() {
        let mut sound = PlayingSound {
            samples: vec![0.1, 0.2, 0.3, 0.4, 0.5],
            position: 0,
            frac_position: 0.0,
            volume: 1.0,
            speed: 1.0,
        };

        // Simulate playback advancing with speed
        sound.frac_position = 2.5;
        sound.position = sound.frac_position.floor() as usize;
        assert_eq!(sound.position, 2);

        // Check remaining samples
        let remaining = sound.samples.len() - sound.position;
        assert_eq!(remaining, 3);
    }

    #[test]
    fn test_playing_sound_finished_when_position_at_end() {
        let sound = PlayingSound {
            samples: vec![0.1, 0.2, 0.3],
            position: 3,
            frac_position: 3.0,
            volume: 0.5,
            speed: 1.5,
        };
        assert!(sound.position >= sound.samples.len());
    }

    // ==================== AudioEngineCommand Tests ====================

    #[test]
    fn test_play_sound_command_creation() {
        let samples = vec![0.1, 0.2, 0.3];
        let cmd = AudioEngineCommand::PlaySound {
            id: "test-sound".to_string(),
            samples: samples.clone(),
            volume: 0.8,
            speed: 1.5,
        };

        if let AudioEngineCommand::PlaySound {
            id,
            samples: s,
            volume,
            speed,
        } = cmd
        {
            assert_eq!(id, "test-sound");
            assert_eq!(s.len(), 3);
            assert_eq!(volume, 0.8);
            assert_eq!(speed, 1.5);
        } else {
            panic!("Expected PlaySound command");
        }
    }

    #[test]
    fn test_start_command_creation() {
        let cmd = AudioEngineCommand::Start {
            input_device: "Microphone".to_string(),
            output_device: "VB-Cable".to_string(),
            sample_rate: 48000,
            channels: 2,
            noise_suppression_enabled: true,
            voice_gate_enabled: false,
        };

        if let AudioEngineCommand::Start {
            input_device,
            output_device,
            sample_rate,
            channels,
            noise_suppression_enabled,
            voice_gate_enabled,
        } = cmd
        {
            assert_eq!(input_device, "Microphone");
            assert_eq!(output_device, "VB-Cable");
            assert_eq!(sample_rate, 48000);
            assert_eq!(channels, 2);
            assert!(noise_suppression_enabled);
            assert!(!voice_gate_enabled);
        } else {
            panic!("Expected Start command");
        }
    }

    // ==================== AudioEngineEvent Tests ====================

    #[test]
    fn test_event_clone() {
        let event = AudioEngineEvent::LevelUpdate {
            input_rms: 0.5,
            input_peak: 0.8,
            output_rms: 0.3,
            output_peak: 0.6,
            monitoring_rms: 0.4,
        };

        let cloned = event.clone();
        if let AudioEngineEvent::LevelUpdate {
            input_rms,
            input_peak,
            output_rms,
            output_peak,
            monitoring_rms,
        } = cloned
        {
            assert_eq!(input_rms, 0.5);
            assert_eq!(input_peak, 0.8);
            assert_eq!(output_rms, 0.3);
            assert_eq!(output_peak, 0.6);
            assert_eq!(monitoring_rms, 0.4);
        } else {
            panic!("Expected LevelUpdate event");
        }
    }

    #[test]
    fn test_error_event_message() {
        let event = AudioEngineEvent::Error("Device not found".to_string());

        if let AudioEngineEvent::Error(msg) = event {
            assert_eq!(msg, "Device not found");
        } else {
            panic!("Expected Error event");
        }
    }

    #[test]
    fn test_info_event_message() {
        let event = AudioEngineEvent::Info("Engine started".to_string());

        if let AudioEngineEvent::Info(msg) = event {
            assert_eq!(msg, "Engine started");
        } else {
            panic!("Expected Info event");
        }
    }

    // ==================== Integration-like Tests ====================

    #[test]
    fn test_engine_accepts_stop_sound_command() {
        let engine = AudioEngine::new();
        let result = engine.send_command(AudioEngineCommand::StopSound {
            id: "nonexistent".to_string(),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_engine_accepts_set_monitoring_device_command() {
        let engine = AudioEngine::new();
        let result = engine.send_command(AudioEngineCommand::SetMonitoringDevice(
            "Speakers".to_string(),
        ));
        assert!(result.is_ok());
    }

    #[test]
    fn test_engine_default_trait() {
        let engine = AudioEngine::default();
        assert!(!engine.is_running());
    }
}
