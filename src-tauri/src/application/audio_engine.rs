//! Audio Engine - Real-time audio processing pipeline
//!
//! This module handles the real-time audio capture, mixing, and output.
//! It runs in a dedicated thread to ensure low-latency processing.

use crate::adapters::{CpalAudioInput, CpalAudioOutput};
use crate::domain::{AudioBuffer, AudioFormat, DeviceId};
use crate::ports::{AudioInput, AudioOutput};
use crossbeam_channel::{bounded, Receiver, Sender};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

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
        sample_rate: u32,
        channels: u16,
    },
    /// Stop a playing sound
    StopSound { id: String },
    /// Set microphone volume
    SetMicVolume(f32),
    /// Set master volume
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
    sample_rate: u32,
    channels: u16,
}

/// The audio engine that runs in a dedicated thread
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
            run_audio_loop(command_rx, event_tx, is_running_clone);
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

/// The main audio processing loop
fn run_audio_loop(
    command_rx: Receiver<AudioEngineCommand>,
    event_tx: Sender<AudioEngineEvent>,
    is_running: Arc<AtomicBool>,
) {
    let mut input = CpalAudioInput::new();
    let mut output = CpalAudioOutput::new();

    let mut playing_sounds: HashMap<String, PlayingSound> = HashMap::new();
    let mut mic_volume: f32 = 1.0;
    let mut master_volume: f32 = 1.0;
    let mut mic_muted = false;
    let mut current_format: Option<AudioFormat> = None;

    // Buffer for mixing
    const BUFFER_SIZE: usize = 256; // ~5.8ms at 44100Hz

    loop {
        // Process commands (non-blocking)
        while let Ok(command) = command_rx.try_recv() {
            match command {
                AudioEngineCommand::Start {
                    input_device,
                    output_device,
                    sample_rate,
                    channels,
                } => {
                    let format = AudioFormat::new(sample_rate, channels, 32);
                    current_format = Some(format);

                    // Start input
                    let input_id = DeviceId::new(&input_device);
                    if let Err(e) = input.start(&input_id, format) {
                        let _ = event_tx.send(AudioEngineEvent::Error(format!(
                            "Failed to start input: {}",
                            e
                        )));
                        continue;
                    }

                    // Start output
                    let output_id = DeviceId::new(&output_device);
                    if let Err(e) = output.start(&output_id, format) {
                        let _ = input.stop();
                        let _ = event_tx.send(AudioEngineEvent::Error(format!(
                            "Failed to start output: {}",
                            e
                        )));
                        continue;
                    }

                    is_running.store(true, Ordering::SeqCst);
                    let _ = event_tx.send(AudioEngineEvent::Started);
                    tracing::info!(
                        "Audio engine started: {} -> {}",
                        input_device,
                        output_device
                    );
                }

                AudioEngineCommand::Stop => {
                    let _ = input.stop();
                    let _ = output.stop();
                    is_running.store(false, Ordering::SeqCst);
                    playing_sounds.clear();
                    let _ = event_tx.send(AudioEngineEvent::Stopped);
                    tracing::info!("Audio engine stopped");
                }

                AudioEngineCommand::PlaySound {
                    id,
                    samples,
                    sample_rate,
                    channels,
                } => {
                    playing_sounds.insert(
                        id,
                        PlayingSound {
                            samples,
                            position: 0,
                            sample_rate,
                            channels,
                        },
                    );
                }

                AudioEngineCommand::StopSound { id } => {
                    playing_sounds.remove(&id);
                }

                AudioEngineCommand::SetMicVolume(volume) => {
                    mic_volume = volume.clamp(0.0, 2.0);
                }

                AudioEngineCommand::SetMasterVolume(volume) => {
                    master_volume = volume.clamp(0.0, 2.0);
                }

                AudioEngineCommand::SetMicMuted(muted) => {
                    mic_muted = muted;
                }

                AudioEngineCommand::Shutdown => {
                    let _ = input.stop();
                    let _ = output.stop();
                    is_running.store(false, Ordering::SeqCst);
                    tracing::info!("Audio engine shutdown");
                    return;
                }
            }
        }

        // If not running, sleep a bit and continue
        if !is_running.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(10));
            continue;
        }

        let format = match current_format {
            Some(f) => f,
            None => {
                thread::sleep(Duration::from_millis(10));
                continue;
            }
        };

        // Create output buffer
        let buffer_samples = BUFFER_SIZE * format.channels as usize;
        let mut output_buffer = vec![0.0f32; buffer_samples];

        // Mix microphone input
        // Note: In a real implementation, we'd read from the input stream
        // For now, we're setting up the structure - the actual reading
        // would happen via callbacks in the CpalAudioInput

        // Mix playing sounds
        let mut sounds_to_remove = Vec::new();

        for (id, sound) in playing_sounds.iter_mut() {
            let remaining = sound.samples.len() - sound.position;
            let to_copy = remaining.min(buffer_samples);

            for i in 0..to_copy {
                let sample_idx = sound.position + i;
                output_buffer[i] += sound.samples[sample_idx];
            }

            sound.position += to_copy;

            if sound.position >= sound.samples.len() {
                sounds_to_remove.push(id.clone());
            }
        }

        // Remove finished sounds
        for id in sounds_to_remove {
            playing_sounds.remove(&id);
        }

        // Apply master volume and clamp
        for sample in &mut output_buffer {
            *sample = (*sample * master_volume).clamp(-1.0, 1.0);
        }

        // Write to output
        let audio_buffer = AudioBuffer::from_raw_f32(output_buffer, format.channels, format.sample_rate);
        if let Err(e) = output.write(&audio_buffer) {
            tracing::warn!("Failed to write audio: {}", e);
        }

        // Small sleep to prevent busy-waiting
        // In a real implementation, this would be driven by audio callbacks
        thread::sleep(Duration::from_micros(100));
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
