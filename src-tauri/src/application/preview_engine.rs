//! Preview Engine - Plays sounds on a selectable output device for monitoring

use cpal::traits::{DeviceTrait, HostTrait};
use crossbeam_channel::{bounded, Receiver, Sender};
use rodio::source::Source;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// A Source wrapper that tracks audio levels
struct LevelTrackingSource<S> {
    inner: S,
    level: Arc<AtomicU32>,
    sample_count: usize,
    sum_squares: f32,
    update_interval: usize, // Update level every N samples
}

impl<S> LevelTrackingSource<S>
where
    S: Source<Item = f32>,
{
    fn new(inner: S, level: Arc<AtomicU32>) -> Self {
        Self {
            inner,
            level,
            sample_count: 0,
            sum_squares: 0.0,
            update_interval: 4800, // ~100ms at 48kHz
        }
    }
}

impl<S> Iterator for LevelTrackingSource<S>
where
    S: Source<Item = f32>,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.inner.next()?;

        // Accumulate for RMS calculation
        self.sum_squares += sample * sample;
        self.sample_count += 1;

        // Update level periodically
        if self.sample_count >= self.update_interval {
            let rms = (self.sum_squares / self.sample_count as f32).sqrt();
            self.level.store(rms.to_bits(), Ordering::Relaxed);
            self.sample_count = 0;
            self.sum_squares = 0.0;
        }

        Some(sample)
    }
}

impl<S> Source for LevelTrackingSource<S>
where
    S: Source<Item = f32>,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.inner.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}

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
    preview_level: Arc<AtomicU32>,
    thread_handle: Option<JoinHandle<()>>,
}

impl PreviewEngine {
    /// Create and start a new preview engine
    pub fn new(app_handle: AppHandle) -> Self {
        let (command_tx, command_rx) = bounded(16);
        let current_pad_id = Arc::new(Mutex::new(None::<String>));
        let current_pad_id_clone = current_pad_id.clone();
        let preview_level = Arc::new(AtomicU32::new(0));
        let preview_level_clone = preview_level.clone();

        let thread_handle = thread::spawn(move || {
            run_preview_thread(
                command_rx,
                current_pad_id_clone,
                preview_level_clone,
                app_handle,
            );
        });

        Self {
            command_tx,
            current_pad_id,
            preview_level,
            thread_handle: Some(thread_handle),
        }
    }

    /// Get the current preview audio level (RMS)
    pub fn get_level(&self) -> f32 {
        f32::from_bits(self.preview_level.load(Ordering::Relaxed))
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

    if is_default_device(name) {
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

/// Check if a device name refers to the default device
fn is_default_device(name: &str) -> bool {
    name == "default" || name.is_empty()
}

/// The main preview thread
fn run_preview_thread(
    command_rx: Receiver<PreviewCommand>,
    current_pad_id: Arc<Mutex<Option<String>>>,
    preview_level: Arc<AtomicU32>,
    app_handle: AppHandle,
) {
    // Current playback state - these must stay alive during playback
    let mut current_sink: Option<Sink> = None;
    let mut _current_stream: Option<OutputStream> = None;
    let mut _current_stream_handle: Option<OutputStreamHandle> = None;
    let mut is_playing = false;

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
                is_playing = false;
                // Reset level when stopped and emit final 0
                preview_level.store(0, Ordering::Relaxed);
                let _ = app_handle.emit("preview-level", 0.0f32);
            }
        }

        // Emit preview level event when playing
        if is_playing {
            let level = f32::from_bits(preview_level.load(Ordering::Relaxed));
            let _ = app_handle.emit("preview-level", level);
        }

        match command_rx.recv_timeout(Duration::from_millis(33)) {
            // ~30 FPS for level updates
            Ok(command) => match command {
                PreviewCommand::Play {
                    path,
                    device_name,
                    pad_id,
                } => {
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
                    is_playing = false;
                    preview_level.store(0, Ordering::Relaxed);

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

                    // Wrap source with level tracking and convert to f32
                    use rodio::Source;
                    let source_f32 = source.convert_samples::<f32>();
                    let level_tracking_source =
                        LevelTrackingSource::new(source_f32, preview_level.clone());

                    // Play the sound with level tracking
                    sink.append(level_tracking_source);

                    // Store state
                    current_sink = Some(sink);
                    _current_stream = Some(stream);
                    _current_stream_handle = Some(stream_handle);
                    is_playing = true;

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
                    is_playing = false;
                    preview_level.store(0, Ordering::Relaxed);
                    // Emit final level 0 to reset the VU meter
                    let _ = app_handle.emit("preview-level", 0.0f32);
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

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== PreviewCommand Tests ====================

    #[test]
    fn test_preview_command_play_creation() {
        let cmd = PreviewCommand::Play {
            path: "/sounds/test.mp3".to_string(),
            device_name: "Speakers".to_string(),
            pad_id: "pad-1".to_string(),
        };

        if let PreviewCommand::Play {
            path,
            device_name,
            pad_id,
        } = cmd
        {
            assert_eq!(path, "/sounds/test.mp3");
            assert_eq!(device_name, "Speakers");
            assert_eq!(pad_id, "pad-1");
        } else {
            panic!("Expected Play command");
        }
    }

    #[test]
    fn test_preview_command_stop() {
        let cmd = PreviewCommand::Stop;
        assert!(matches!(cmd, PreviewCommand::Stop));
    }

    #[test]
    fn test_preview_command_shutdown() {
        let cmd = PreviewCommand::Shutdown;
        assert!(matches!(cmd, PreviewCommand::Shutdown));
    }

    #[test]
    fn test_preview_command_debug_format() {
        let cmd = PreviewCommand::Stop;
        let debug_str = format!("{:?}", cmd);
        assert_eq!(debug_str, "Stop");
    }

    #[test]
    fn test_preview_command_play_debug_format() {
        let cmd = PreviewCommand::Play {
            path: "test.mp3".to_string(),
            device_name: "default".to_string(),
            pad_id: "p1".to_string(),
        };
        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("Play"));
        assert!(debug_str.contains("test.mp3"));
    }

    // ==================== Helper Function Tests ====================

    #[test]
    fn test_is_default_device_with_default() {
        assert!(is_default_device("default"));
    }

    #[test]
    fn test_is_default_device_with_empty() {
        assert!(is_default_device(""));
    }

    #[test]
    fn test_is_default_device_with_specific_name() {
        assert!(!is_default_device("Speakers"));
        assert!(!is_default_device("MacBook Pro Speakers"));
        assert!(!is_default_device("VB-Cable"));
    }

    #[test]
    fn test_is_default_device_case_sensitive() {
        // "default" is the exact match, "Default" is not
        assert!(is_default_device("default"));
        assert!(!is_default_device("Default"));
        assert!(!is_default_device("DEFAULT"));
    }

    // ==================== Integration Test Notes ====================
    //
    // The following functionality requires Tauri AppHandle and cannot be
    // unit tested without mocking:
    //
    // - PreviewEngine::new() - requires AppHandle
    // - PreviewEngine::send_command() - requires running engine
    // - PreviewEngine::current_pad_id() - requires running engine
    // - PreviewEngine::shutdown() - requires running engine
    // - find_output_device() - requires cpal audio system
    // - run_preview_thread() - requires AppHandle and cpal
    //
    // These should be tested via integration tests with a real Tauri app
    // or by implementing a mock AppHandle trait.
}
