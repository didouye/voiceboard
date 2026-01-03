//! Audio output port - Interface for sending audio to devices

use crate::domain::{AudioBuffer, AudioFormat, DeviceId};

/// Errors that can occur during audio output operations
#[derive(Debug, thiserror::Error)]
pub enum AudioOutputError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Failed to open device: {0}")]
    OpenError(String),

    #[error("Stream error: {0}")]
    StreamError(String),

    #[error("Buffer underrun")]
    BufferUnderrun,

    #[error("Device disconnected")]
    DeviceDisconnected,

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

/// Port for audio output operations
///
/// This trait defines the contract for sending audio to output devices
/// like virtual microphones or speakers.
///
/// Note: Send + Sync bounds removed because cpal::Stream is !Send + !Sync.
/// For async contexts, wrap implementations in Arc<Mutex<>> as needed.
#[cfg_attr(test, mockall::automock)]
pub trait AudioOutput {
    /// Start the audio output stream
    fn start(&mut self, device_id: &DeviceId, format: AudioFormat) -> Result<(), AudioOutputError>;

    /// Stop the audio output stream
    fn stop(&mut self) -> Result<(), AudioOutputError>;

    /// Check if currently playing
    fn is_playing(&self) -> bool;

    /// Write audio buffer to the output
    fn write(&mut self, buffer: &AudioBuffer) -> Result<(), AudioOutputError>;

    /// Get the current audio format
    fn current_format(&self) -> Option<AudioFormat>;

    /// Get the number of frames that can be written without blocking
    fn available_frames(&self) -> usize;
}

/// Port specifically for virtual audio device output
///
/// This extends AudioOutput with virtual device specific functionality
pub trait VirtualAudioOutput: AudioOutput {
    /// Check if the virtual device driver is installed
    fn is_driver_installed(&self) -> bool;

    /// Get the virtual device name as it appears in the system
    fn virtual_device_name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_output_error_device_not_found() {
        let error = AudioOutputError::DeviceNotFound("VB-Cable".to_string());
        assert_eq!(format!("{}", error), "Device not found: VB-Cable");
    }

    #[test]
    fn test_audio_output_error_open_error() {
        let error = AudioOutputError::OpenError("Stream init failed".to_string());
        assert_eq!(
            format!("{}", error),
            "Failed to open device: Stream init failed"
        );
    }

    #[test]
    fn test_audio_output_error_stream_error() {
        let error = AudioOutputError::StreamError("Write failed".to_string());
        assert_eq!(format!("{}", error), "Stream error: Write failed");
    }

    #[test]
    fn test_audio_output_error_buffer_underrun() {
        let error = AudioOutputError::BufferUnderrun;
        assert_eq!(format!("{}", error), "Buffer underrun");
    }

    #[test]
    fn test_audio_output_error_device_disconnected() {
        let error = AudioOutputError::DeviceDisconnected;
        assert_eq!(format!("{}", error), "Device disconnected");
    }

    #[test]
    fn test_audio_output_error_unsupported_format() {
        let error = AudioOutputError::UnsupportedFormat("32-bit float".to_string());
        assert_eq!(format!("{}", error), "Unsupported format: 32-bit float");
    }

    #[test]
    fn test_audio_output_error_debug() {
        let error = AudioOutputError::BufferUnderrun;
        let debug = format!("{:?}", error);
        assert!(debug.contains("BufferUnderrun"));
    }
}
