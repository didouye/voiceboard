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
///
/// Note: Send + Sync bounds removed because cpal::Stream is !Send + !Sync.
/// For async contexts, wrap implementations in Arc<Mutex<>> as needed.
#[cfg_attr(test, mockall::automock)]
pub trait AudioInput {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_input_error_device_not_found() {
        let error = AudioInputError::DeviceNotFound("Microphone".to_string());
        assert_eq!(format!("{}", error), "Device not found: Microphone");
    }

    #[test]
    fn test_audio_input_error_open_error() {
        let error = AudioInputError::OpenError("Failed to open stream".to_string());
        assert_eq!(
            format!("{}", error),
            "Failed to open device: Failed to open stream"
        );
    }

    #[test]
    fn test_audio_input_error_stream_error() {
        let error = AudioInputError::StreamError("Buffer overflow".to_string());
        assert_eq!(format!("{}", error), "Stream error: Buffer overflow");
    }

    #[test]
    fn test_audio_input_error_device_disconnected() {
        let error = AudioInputError::DeviceDisconnected;
        assert_eq!(format!("{}", error), "Device disconnected");
    }

    #[test]
    fn test_audio_input_error_unsupported_format() {
        let error = AudioInputError::UnsupportedFormat("96kHz not supported".to_string());
        assert_eq!(
            format!("{}", error),
            "Unsupported format: 96kHz not supported"
        );
    }

    #[test]
    fn test_audio_input_error_debug() {
        let error = AudioInputError::DeviceDisconnected;
        let debug = format!("{:?}", error);
        assert!(debug.contains("DeviceDisconnected"));
    }
}
