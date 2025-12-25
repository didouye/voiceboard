//! Preview Engine - Plays sounds on a selectable output device for monitoring

use cpal::traits::{DeviceTrait, HostTrait};
use crossbeam_channel::{bounded, Receiver, Sender};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// Commands that can be sent to the preview engine
#[derive(Debug)]
pub enum PreviewCommand {
    /// Play a sound file on a specific device
    Play {
        path: String,
        device_name: String,
        pad_id: String,
    },
    /// Stop the currently playing preview
    Stop,
    /// Shutdown the engine
    Shutdown,
}

/// The preview engine that manages sound previews
pub struct PreviewEngine {
    command_tx: Sender<PreviewCommand>,
    current_pad_id: Arc<Mutex<Option<String>>>,
    thread_handle: Option<JoinHandle<()>>,
}

impl PreviewEngine {
    /// Create and start a new preview engine
    pub fn new(app_handle: AppHandle) -> Self {
        let (command_tx, command_rx) = bounded(16);
        let current_pad_id = Arc::new(Mutex::new(None::<String>));
        let current_pad_id_clone = current_pad_id.clone();

        let thread_handle = thread::spawn(move || {
            run_preview_thread(command_rx, current_pad_id_clone, app_handle);
        });

        Self {
            command_tx,
            current_pad_id,
            thread_handle: Some(thread_handle),
        }
    }

    /// Send a command to the preview engine
    pub fn send_command(&self, command: PreviewCommand) -> Result<(), String> {
        self.command_tx
            .send(command)
            .map_err(|e| format!("Failed to send preview command: {}", e))
    }

    /// Get the currently previewing pad ID
    pub fn current_pad_id(&self) -> Option<String> {
        self.current_pad_id.lock().ok()?.clone()
    }

    /// Shutdown the preview engine
    pub fn shutdown(&mut self) {
        let _ = self.command_tx.send(PreviewCommand::Shutdown);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for PreviewEngine {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Find an output device by name
fn find_output_device(name: &str) -> Option<cpal::Device> {
    let host = cpal::default_host();

    if name == "default" || name.is_empty() {
        return host.default_output_device();
    }

    if let Ok(devices) = host.output_devices() {
        for device in devices {
            if let Ok(device_name) = device.name() {
                if device_name == name {
                    return Some(device);
                }
            }
        }
    }

    // Fallback to default
    host.default_output_device()
}

/// The main preview thread
fn run_preview_thread(
    command_rx: Receiver<PreviewCommand>,
    current_pad_id: Arc<Mutex<Option<String>>>,
    app_handle: AppHandle,
) {
    // Current playback state - these must stay alive during playback
    let mut current_sink: Option<Sink> = None;
    let mut _current_stream: Option<OutputStream> = None;
    let mut _current_stream_handle: Option<OutputStreamHandle> = None;

    loop {
        // Check if current sound finished naturally
        if let Some(ref sink) = current_sink {
            if sink.empty() {
                if let Ok(mut pad_id) = current_pad_id.lock() {
                    if let Some(id) = pad_id.take() {
                        let _ = app_handle.emit("preview-stopped", &id);
                        tracing::info!("Preview finished naturally: {}", id);
                    }
                }
                current_sink = None;
                _current_stream = None;
                _current_stream_handle = None;
            }
        }

        match command_rx.recv_timeout(Duration::from_millis(50)) {
            Ok(command) => match command {
                PreviewCommand::Play { path, device_name, pad_id } => {
                    // Stop current preview if any
                    if let Some(sink) = current_sink.take() {
                        sink.stop();
                    }
                    if let Ok(mut current) = current_pad_id.lock() {
                        if let Some(old_id) = current.take() {
                            let _ = app_handle.emit("preview-stopped", &old_id);
                        }
                    }
                    _current_stream = None;
                    _current_stream_handle = None;

                    // Find the output device
                    let device = match find_output_device(&device_name) {
                        Some(d) => d,
                        None => {
                            tracing::error!("Preview device not found: {}", device_name);
                            continue;
                        }
                    };

                    // Create output stream on the specific device
                    let (stream, stream_handle) = match OutputStream::try_from_device(&device) {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::error!("Failed to create preview stream: {}", e);
                            continue;
                        }
                    };

                    // Create sink
                    let sink = match Sink::try_new(&stream_handle) {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::error!("Failed to create preview sink: {}", e);
                            continue;
                        }
                    };

                    // Open and decode the file
                    let file = match File::open(&path) {
                        Ok(f) => f,
                        Err(e) => {
                            tracing::error!("Failed to open file for preview: {}", e);
                            continue;
                        }
                    };

                    let source = match Decoder::new(BufReader::new(file)) {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::error!("Failed to decode file for preview: {}", e);
                            continue;
                        }
                    };

                    // Play the sound
                    sink.append(source);

                    // Store state
                    current_sink = Some(sink);
                    _current_stream = Some(stream);
                    _current_stream_handle = Some(stream_handle);

                    if let Ok(mut current) = current_pad_id.lock() {
                        *current = Some(pad_id.clone());
                    }

                    let _ = app_handle.emit("preview-started", &pad_id);
                    tracing::info!("Preview started: {} on {}", path, device_name);
                }

                PreviewCommand::Stop => {
                    if let Some(sink) = current_sink.take() {
                        sink.stop();
                    }
                    if let Ok(mut current) = current_pad_id.lock() {
                        if let Some(id) = current.take() {
                            let _ = app_handle.emit("preview-stopped", &id);
                            tracing::info!("Preview stopped: {}", id);
                        }
                    }
                    _current_stream = None;
                    _current_stream_handle = None;
                }

                PreviewCommand::Shutdown => {
                    if let Some(sink) = current_sink.take() {
                        sink.stop();
                    }
                    tracing::info!("Preview engine shutdown");
                    return;
                }
            },
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                // No command, continue checking sink state
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                tracing::info!("Preview command channel closed");
                return;
            }
        }
    }
}
