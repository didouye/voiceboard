/// Error types for the Voiceboard application
use thiserror::Error;

/// Result type alias for Voiceboard operations
pub type Result<T> = std::result::Result<T, VoiceboardError>;

/// Main error type for the application
#[derive(Error, Debug)]
pub enum VoiceboardError {
    /// Audio device errors
    #[error("Audio device error: {0}")]
    AudioDevice(String),

    /// Audio format errors
    #[error("Audio format error: {0}")]
    AudioFormat(String),

    /// Audio playback/capture errors
    #[error("Audio operation error: {0}")]
    AudioOperation(String),

    /// Sound file errors
    #[error("Sound file error: {0}")]
    SoundFile(String),

    /// Decoding errors
    #[error("Audio decoding error: {0}")]
    Decoding(String),

    /// Resampling errors
    #[error("Resampling error: {0}")]
    Resampling(String),

    /// Database errors
    #[error("Database error: {0}")]
    Database(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// File I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Sound not found
    #[error("Sound not found: {0}")]
    SoundNotFound(String),

    /// Device not found
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Thread error
    #[error("Thread error: {0}")]
    Thread(String),

    /// General errors
    #[error("{0}")]
    General(String),
}

// Implement From traits for common error conversions

impl From<cpal::DevicesError> for VoiceboardError {
    fn from(err: cpal::DevicesError) -> Self {
        VoiceboardError::AudioDevice(err.to_string())
    }
}

impl From<cpal::DeviceNameError> for VoiceboardError {
    fn from(err: cpal::DeviceNameError) -> Self {
        VoiceboardError::AudioDevice(err.to_string())
    }
}

impl From<cpal::BuildStreamError> for VoiceboardError {
    fn from(err: cpal::BuildStreamError) -> Self {
        VoiceboardError::AudioOperation(err.to_string())
    }
}

impl From<cpal::PlayStreamError> for VoiceboardError {
    fn from(err: cpal::PlayStreamError) -> Self {
        VoiceboardError::AudioOperation(err.to_string())
    }
}

impl From<cpal::PauseStreamError> for VoiceboardError {
    fn from(err: cpal::PauseStreamError) -> Self {
        VoiceboardError::AudioOperation(err.to_string())
    }
}

impl From<cpal::DefaultStreamConfigError> for VoiceboardError {
    fn from(err: cpal::DefaultStreamConfigError) -> Self {
        VoiceboardError::AudioDevice(err.to_string())
    }
}

impl From<cpal::SupportedStreamConfigsError> for VoiceboardError {
    fn from(err: cpal::SupportedStreamConfigsError) -> Self {
        VoiceboardError::AudioDevice(err.to_string())
    }
}

impl From<sqlx::Error> for VoiceboardError {
    fn from(err: sqlx::Error) -> Self {
        VoiceboardError::Database(err.to_string())
    }
}

// Implement Serialize for Tauri error responses
impl serde::Serialize for VoiceboardError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
