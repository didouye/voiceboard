//! Audio input port - Interface for capturing audio from devices

use crate::domain::{AudioBuffer, AudioFormat, DeviceId};
use std::sync::mpsc::Receiver;

/// Errors that can occur during audio input operations
#[derive(Debug, thiserror::Error)]
pub enum AudioInputError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Failed to open device: {0}")]
    OpenError(String),

    #[error("Stream error: {0}")]
    StreamError(String),

    #[error("Device disconnected")]
    DeviceDisconnected,

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

/// Port for audio input operations
///
/// This trait defines the contract for capturing audio from input devices
/// like microphones. Implementations should handle device-specific details.
#[cfg_attr(test, mockall::automock)]
pub trait AudioInput: Send + Sync {
    /// Start capturing audio from the specified device
    fn start(&mut self, device_id: &DeviceId, format: AudioFormat) -> Result<(), AudioInputError>;

    /// Stop capturing audio
    fn stop(&mut self) -> Result<(), AudioInputError>;

    /// Check if currently capturing
    fn is_capturing(&self) -> bool;

    /// Get a receiver for audio buffers
    /// Returns None if not capturing
    fn get_receiver(&self) -> Option<Receiver<AudioBuffer>>;

    /// Get the current audio format
    fn current_format(&self) -> Option<AudioFormat>;
}

/// Callback-based audio input for real-time processing
pub trait AudioInputCallback: Send + Sync {
    /// Called when new audio data is available
    fn on_audio_data(&mut self, buffer: &AudioBuffer);

    /// Called when an error occurs
    fn on_error(&mut self, error: AudioInputError);
}
