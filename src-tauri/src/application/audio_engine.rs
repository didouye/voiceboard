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
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

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
    },
    /// Stop mixing
    Stop,
    /// Play an audio buffer (from a sound file)
    PlaySound { id: String, samples: Vec<f32> },
    /// Stop a playing sound
    StopSound { id: String },
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
    },
}

/// A sound that is currently playing
struct PlayingSound {
    samples: Vec<f32>,
    position: usize,
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
                    } => {
                        // Stop any existing streams
                        input_stream = None;
                        output_stream = None;

                        // Find devices
                        let input_dev = match find_device(&host, &input_device, true) {
                            Some(d) => d,
                            None => {
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

                        let _output_default = match output_dev.default_output_config() {
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

                        // Find a common configuration supported by both devices
                        // Try to find a sample rate that works for both
                        let common_sample_rates = [48000u32, 44100, 96000, 22050, 16000];

                        let mut found_config: Option<cpal::StreamConfig> = None;

                        // Get supported configs for input
                        let input_configs: Vec<_> = input_dev
                            .supported_input_configs()
                            .map(|c| c.collect())
                            .unwrap_or_default();

                        // Get supported configs for output
                        let output_configs: Vec<_> = output_dev
                            .supported_output_configs()
                            .map(|c| c.collect())
                            .unwrap_or_default();

                        // Log supported configs
                        let _ = event_tx.send(AudioEngineEvent::Info(format!(
                            "Input supported: {} configs",
                            input_configs.len()
                        )));
                        for cfg in &input_configs {
                            let _ = event_tx.send(AudioEngineEvent::Info(format!(
                                "  Input: {}ch, {}Hz-{}Hz, {:?}",
                                cfg.channels(),
                                cfg.min_sample_rate().0,
                                cfg.max_sample_rate().0,
                                cfg.sample_format()
                            )));
                        }

                        let _ = event_tx.send(AudioEngineEvent::Info(format!(
                            "Output supported: {} configs",
                            output_configs.len()
                        )));
                        for cfg in &output_configs {
                            let _ = event_tx.send(AudioEngineEvent::Info(format!(
                                "  Output: {}ch, {}Hz-{}Hz, {:?}",
                                cfg.channels(),
                                cfg.min_sample_rate().0,
                                cfg.max_sample_rate().0,
                                cfg.sample_format()
                            )));
                        }

                        // Find common sample rate
                        'outer: for &rate in &common_sample_rates {
                            let sr = cpal::SampleRate(rate);

                            // Check if input supports this rate with 2 channels
                            let input_supports = input_configs.iter().any(|c| {
                                c.channels() >= 1
                                    && sr >= c.min_sample_rate()
                                    && sr <= c.max_sample_rate()
                            });

                            // Check if output supports this rate
                            let output_supports = output_configs.iter().any(|c| {
                                c.channels() >= 1
                                    && sr >= c.min_sample_rate()
                                    && sr <= c.max_sample_rate()
                            });

                            if input_supports && output_supports {
                                // Find the right number of channels (prefer stereo)
                                let channels = if input_configs.iter().any(|c| c.channels() == 2)
                                    && output_configs.iter().any(|c| c.channels() >= 2)
                                {
                                    2
                                } else {
                                    1
                                };

                                found_config = Some(cpal::StreamConfig {
                                    channels,
                                    sample_rate: sr,
                                    buffer_size: cpal::BufferSize::Default,
                                });

                                let _ = event_tx.send(AudioEngineEvent::Info(format!(
                                    "Found common config: {}ch, {}Hz",
                                    channels, rate
                                )));
                                break 'outer;
                            }
                        }

                        let config = match found_config {
                            Some(c) => c,
                            None => {
                                // Fallback: try input's default config
                                let _ = event_tx.send(AudioEngineEvent::Info(
                                    "No common config found, trying input default".to_string(),
                                ));
                                input_default.into()
                            }
                        };

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
                        let input_level_clone = input_level.clone();

                        // Clone references for callbacks
                        let producer_clone = producer.clone();
                        let mic_volume_clone = mic_volume.clone();
                        let mic_muted_clone = mic_muted.clone();

                        // Debug counter for input callback
                        let input_callback_count = Arc::new(AtomicU32::new(0));
                        let input_callback_count_clone = input_callback_count.clone();

                        // Build input stream
                        let input_result = input_dev.build_input_stream(
                            &config,
                            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                                // Log first few callbacks to verify stream is working
                                let count = input_callback_count_clone.fetch_add(1, Ordering::Relaxed);
                                if count < 5 || count.is_multiple_of(1000) {
                                    let max_sample = data.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
                                    tracing::info!(
                                        "[AudioEngine] Input callback #{}: {} samples, max amplitude: {:.6}",
                                        count,
                                        data.len(),
                                        max_sample
                                    );
                                }

                                let muted = mic_muted_clone.load(Ordering::Relaxed);
                                let volume =
                                    f32::from_bits(mic_volume_clone.load(Ordering::Relaxed));

                                // Calculate RMS for input level
                                let mut sum_squares = 0.0f32;

                                if let Ok(mut prod) = producer_clone.try_lock() {
                                    for &sample in data {
                                        let processed = if muted { 0.0 } else { sample * volume };
                                        sum_squares += processed * processed;
                                        let _ = prod.try_push(processed);
                                    }
                                }

                                // Store RMS level (will be read by level monitoring thread)
                                if !data.is_empty() {
                                    let rms = (sum_squares / data.len() as f32).sqrt();
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
                        let engine_channels = config.channels;

                        // Debug counter for output callback
                        let output_callback_count = Arc::new(AtomicU32::new(0));
                        let output_callback_count_clone = output_callback_count.clone();

                        // Build output stream
                        let output_result = output_dev.build_output_stream(
                            &config,
                            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                                // Log first few callbacks to verify stream is working
                                let count = output_callback_count_clone.fetch_add(1, Ordering::Relaxed);

                                let master_vol =
                                    f32::from_bits(master_volume_clone.load(Ordering::Relaxed));
                                let mic_mon_enabled = mic_monitoring_for_output.load(Ordering::Relaxed);

                                // Read mic samples and store temporarily
                                let data_len = data.len();
                                let mut mic_samples = vec![0.0f32; data_len];
                                if let Ok(mut cons) = consumer_clone.try_lock() {
                                    for sample in mic_samples.iter_mut() {
                                        *sample = cons.try_pop().unwrap_or(0.0);
                                    }
                                }

                                // Start with zeros, then mix sounds
                                // This way we can control mic separately for monitoring
                                for sample in data.iter_mut() {
                                    *sample = 0.0;
                                }

                                // Mix in playing sounds (this goes to BOTH outputs)
                                // Sounds are stored as MONO, output may be stereo
                                // We need to mix one mono sample per FRAME, not per sample
                                if let Ok(mut state) = audio_state_clone.try_lock() {
                                    let mut finished = Vec::new();
                                    let num_frames = data_len / engine_channels as usize;

                                    for (id, sound) in state.playing_sounds.iter_mut() {
                                        let remaining = sound.samples.len() - sound.position;
                                        let frames_to_mix = remaining.min(num_frames);

                                        for frame in 0..frames_to_mix {
                                            let mono_sample = sound.samples[sound.position + frame];

                                            // Duplicate mono sample to all output channels
                                            for ch in 0..engine_channels as usize {
                                                let idx = frame * engine_channels as usize + ch;
                                                if idx < data_len {
                                                    data[idx] = (data[idx] + mono_sample).clamp(-1.0, 1.0);
                                                }
                                            }
                                        }

                                        sound.position += frames_to_mix;
                                        if sound.position >= sound.samples.len() {
                                            finished.push(id.clone());
                                        }
                                    }

                                    for id in finished {
                                        state.playing_sounds.remove(&id);
                                    }
                                }

                                // Write to monitoring buffer BEFORE adding mic to main output
                                // Monitoring gets: sounds + (mic if mic_monitoring enabled)
                                // Push MONO samples only (one per frame, not per channel)
                                if let Ok(mut prod_mon) = producer_monitoring_for_output.try_lock() {
                                    let num_frames = data_len / engine_channels as usize;

                                    for frame in 0..num_frames {
                                        // Get sound sample for this frame (average L+R if stereo)
                                        let sound_sample = if engine_channels == 2 {
                                            let l = data[frame * 2];
                                            let r = data[frame * 2 + 1];
                                            (l + r) * 0.5
                                        } else {
                                            data[frame]
                                        };

                                        // Get mic sample for this frame (average L+R if stereo)
                                        let mic_sample = if engine_channels == 2 {
                                            let l = mic_samples.get(frame * 2).copied().unwrap_or(0.0);
                                            let r = mic_samples.get(frame * 2 + 1).copied().unwrap_or(0.0);
                                            (l + r) * 0.5
                                        } else {
                                            mic_samples.get(frame).copied().unwrap_or(0.0)
                                        };

                                        let monitoring_sample = if mic_mon_enabled {
                                            (sound_sample + mic_sample) * master_vol
                                        } else {
                                            sound_sample * master_vol
                                        };
                                        let _ = prod_mon.try_push(monitoring_sample.clamp(-1.0, 1.0));
                                    }
                                }

                                // Add mic to main output (always)
                                for (i, sample) in data.iter_mut().enumerate() {
                                    let mic_sample = mic_samples.get(i).copied().unwrap_or(0.0);
                                    *sample = (*sample + mic_sample).clamp(-1.0, 1.0);
                                }

                                // Apply master volume to main output
                                for sample in data.iter_mut() {
                                    *sample = (*sample * master_vol).clamp(-1.0, 1.0);
                                }

                                // Calculate output RMS after master volume
                                let mut sum_squares = 0.0f32;
                                for sample in data.iter() {
                                    sum_squares += sample * sample;
                                }
                                if !data.is_empty() {
                                    let rms = (sum_squares / data_len as f32).sqrt();
                                    output_level_for_callback
                                        .store(rms.to_bits(), Ordering::Relaxed);

                                    // Log first few callbacks
                                    if count < 5 || count.is_multiple_of(1000) {
                                        let max_sample = data.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
                                        tracing::info!(
                                            "[AudioEngine] Output callback #{}: {} samples, max amplitude: {:.6}, rms: {:.6}",
                                            count,
                                            data_len,
                                            max_sample,
                                            rms
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
                            let engine_sample_rate = config.sample_rate.0 as f64;
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
                                config.sample_rate.0,
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
                                    } else {
                                        // Couldn't acquire locks, output silence
                                        for sample in data.iter_mut() {
                                            *sample = 0.0;
                                        }
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
                            let _ = event_tx.send(AudioEngineEvent::Error(format!(
                                "Failed to start input: {}",
                                e
                            )));
                            continue;
                        }

                        if let Err(e) = output_s.play() {
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

                    AudioEngineCommand::PlaySound { id, samples } => {
                        if let Ok(mut state) = audio_state.lock() {
                            state.playing_sounds.insert(
                                id,
                                PlayingSound {
                                    samples,
                                    position: 0,
                                },
                            );
                        }
                    }

                    AudioEngineCommand::StopSound { id } => {
                        if let Ok(mut state) = audio_state.lock() {
                            state.playing_sounds.remove(&id);
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = AudioEngine::new();
        assert!(!engine.is_running());
    }
}
