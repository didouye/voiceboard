//! Audio Engine - Real-time audio processing pipeline
//!
//! This module handles the real-time audio capture, mixing, and output.
//! It uses ring buffers for lock-free communication between audio threads.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{bounded, Receiver, Sender};
use ringbuf::{HeapRb, traits::{Consumer, Producer, Split}};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Size of the ring buffer in samples (not frames)
const RING_BUFFER_SIZE: usize = 8192;

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
    PlaySound {
        id: String,
        samples: Vec<f32>,
    },
    /// Stop a playing sound
    StopSound { id: String },
    /// Set microphone volume (0.0 - 2.0)
    SetMicVolume(f32),
    /// Set master volume (0.0 - 2.0)
    SetMasterVolume(f32),
    /// Mute/unmute microphone
    SetMicMuted(bool),
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
    /// Audio level update (for UI meters)
    LevelUpdate { input_level: f32, output_level: f32 },
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

    // Shared state for audio processing
    let audio_state = Arc::new(Mutex::new(AudioState::default()));

    // Ring buffer for passing audio from input to output
    let ring_buffer = Arc::new(Mutex::new(None::<(ringbuf::HeapProd<f32>, ringbuf::HeapCons<f32>)>));

    // Atomic volume controls (for lock-free access in callbacks)
    let mic_volume = Arc::new(AtomicU32::new(f32::to_bits(1.0)));
    let master_volume = Arc::new(AtomicU32::new(f32::to_bits(1.0)));
    let mic_muted = Arc::new(AtomicBool::new(false));

    loop {
        // Process commands
        match command_rx.recv_timeout(Duration::from_millis(10)) {
            Ok(command) => {
                match command {
                    AudioEngineCommand::Start {
                        input_device,
                        output_device,
                        sample_rate,
                        channels,
                    } => {
                        // Stop any existing streams
                        input_stream = None;
                        output_stream = None;

                        // Find devices
                        let input_dev = match find_device(&host, &input_device, true) {
                            Some(d) => d,
                            None => {
                                let _ = event_tx.send(AudioEngineEvent::Error(
                                    format!("Input device not found: {}", input_device)
                                ));
                                continue;
                            }
                        };

                        let output_dev = match find_device(&host, &output_device, false) {
                            Some(d) => d,
                            None => {
                                let _ = event_tx.send(AudioEngineEvent::Error(
                                    format!("Output device not found: {}", output_device)
                                ));
                                continue;
                            }
                        };

                        // Create ring buffer for audio pass-through
                        let rb = HeapRb::<f32>::new(RING_BUFFER_SIZE);
                        let (producer, consumer) = rb.split();

                        // Store producer in Arc<Mutex> for input callback
                        let producer = Arc::new(Mutex::new(producer));
                        let consumer = Arc::new(Mutex::new(consumer));

                        let config = cpal::StreamConfig {
                            channels,
                            sample_rate: cpal::SampleRate(sample_rate),
                            buffer_size: cpal::BufferSize::Default,
                        };

                        // Clone references for callbacks
                        let producer_clone = producer.clone();
                        let mic_volume_clone = mic_volume.clone();
                        let mic_muted_clone = mic_muted.clone();

                        // Build input stream
                        let input_result = input_dev.build_input_stream(
                            &config,
                            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                                let muted = mic_muted_clone.load(Ordering::Relaxed);
                                let volume = f32::from_bits(mic_volume_clone.load(Ordering::Relaxed));

                                if let Ok(mut prod) = producer_clone.try_lock() {
                                    for &sample in data {
                                        let processed = if muted { 0.0 } else { sample * volume };
                                        // If buffer is full, drop samples (acceptable for real-time audio)
                                        let _ = prod.try_push(processed);
                                    }
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
                                let _ = event_tx.send(AudioEngineEvent::Error(
                                    format!("Failed to create input stream: {}", e)
                                ));
                                continue;
                            }
                        };

                        // Clone references for output callback
                        let consumer_clone = consumer.clone();
                        let master_volume_clone = master_volume.clone();
                        let audio_state_clone = audio_state.clone();

                        // Build output stream
                        let output_result = output_dev.build_output_stream(
                            &config,
                            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                                let master_vol = f32::from_bits(master_volume_clone.load(Ordering::Relaxed));

                                // First, fill with mic input from ring buffer
                                if let Ok(mut cons) = consumer_clone.try_lock() {
                                    for sample in data.iter_mut() {
                                        *sample = cons.try_pop().unwrap_or(0.0);
                                    }
                                } else {
                                    // If we can't get the lock, output silence
                                    for sample in data.iter_mut() {
                                        *sample = 0.0;
                                    }
                                }

                                // Mix in playing sounds
                                if let Ok(mut state) = audio_state_clone.try_lock() {
                                    let mut finished = Vec::new();

                                    for (id, sound) in state.playing_sounds.iter_mut() {
                                        let remaining = sound.samples.len() - sound.position;
                                        let to_mix = remaining.min(data.len());

                                        for (i, sample) in data.iter_mut().take(to_mix).enumerate() {
                                            *sample = (*sample + sound.samples[sound.position + i]).clamp(-1.0, 1.0);
                                        }

                                        sound.position += to_mix;
                                        if sound.position >= sound.samples.len() {
                                            finished.push(id.clone());
                                        }
                                    }

                                    for id in finished {
                                        state.playing_sounds.remove(&id);
                                    }
                                }

                                // Apply master volume
                                for sample in data.iter_mut() {
                                    *sample = (*sample * master_vol).clamp(-1.0, 1.0);
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
                                let _ = event_tx.send(AudioEngineEvent::Error(
                                    format!("Failed to create output stream: {}", e)
                                ));
                                continue;
                            }
                        };

                        // Start streams
                        if let Err(e) = input_s.play() {
                            let _ = event_tx.send(AudioEngineEvent::Error(
                                format!("Failed to start input: {}", e)
                            ));
                            continue;
                        }

                        if let Err(e) = output_s.play() {
                            let _ = event_tx.send(AudioEngineEvent::Error(
                                format!("Failed to start output: {}", e)
                            ));
                            continue;
                        }

                        // Store streams to keep them alive
                        input_stream = Some(input_s);
                        output_stream = Some(output_s);

                        is_running.store(true, Ordering::SeqCst);
                        let _ = event_tx.send(AudioEngineEvent::Started);
                        tracing::info!("Audio engine started: {} -> {}", input_device, output_device);
                    }

                    AudioEngineCommand::Stop => {
                        // Pause streams before dropping to ensure clean stop
                        if let Some(ref stream) = input_stream {
                            let _ = stream.pause();
                        }
                        if let Some(ref stream) = output_stream {
                            let _ = stream.pause();
                        }

                        // Drop the streams
                        input_stream = None;
                        output_stream = None;

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
                            state.playing_sounds.insert(id, PlayingSound {
                                samples,
                                position: 0,
                            });
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
                        master_volume.store(f32::to_bits(volume.clamp(0.0, 2.0)), Ordering::Relaxed);
                    }

                    AudioEngineCommand::SetMicMuted(muted) => {
                        mic_muted.store(muted, Ordering::Relaxed);
                    }

                    AudioEngineCommand::Shutdown => {
                        // Pause streams before dropping
                        if let Some(ref stream) = input_stream {
                            let _ = stream.pause();
                        }
                        if let Some(ref stream) = output_stream {
                            let _ = stream.pause();
                        }

                        drop(input_stream);
                        drop(output_stream);
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
