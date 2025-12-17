//! File decoder port - Interface for decoding audio files

use crate::domain::{AudioBuffer, AudioFileFormat, AudioFormat};
use std::path::Path;
use std::time::Duration;

/// Errors that can occur during audio file decoding
#[derive(Debug, thiserror::Error)]
pub enum FileDecoderError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Decode error: {0}")]
    DecodeError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Invalid file: {0}")]
    InvalidFile(String),
}

/// Metadata for an audio file
#[derive(Debug, Clone)]
pub struct AudioFileMetadata {
    pub format: AudioFileFormat,
    pub duration: Duration,
    pub audio_format: AudioFormat,
    pub title: Option<String>,
    pub artist: Option<String>,
}

/// Port for decoding audio files
///
/// This trait defines the contract for reading and decoding
/// audio files (MP3, OGG, WAV, etc.)
#[cfg_attr(test, mockall::automock)]
pub trait FileDecoder: Send + Sync {
    /// Open an audio file for decoding
    fn open(&mut self, path: &Path) -> Result<AudioFileMetadata, FileDecoderError>;

    /// Read the next chunk of audio data
    /// Returns None when the file is finished
    fn read_next(&mut self) -> Result<Option<AudioBuffer>, FileDecoderError>;

    /// Seek to a specific position in the file
    fn seek(&mut self, position: Duration) -> Result<(), FileDecoderError>;

    /// Get the current position in the file
    fn position(&self) -> Duration;

    /// Get the total duration of the file
    fn duration(&self) -> Option<Duration>;

    /// Check if the decoder has reached the end of the file
    fn is_finished(&self) -> bool;

    /// Reset the decoder to the beginning
    fn reset(&mut self) -> Result<(), FileDecoderError>;

    /// Close the file and release resources
    fn close(&mut self);
}

/// Factory for creating file decoders
#[cfg_attr(test, mockall::automock)]
pub trait FileDecoderFactory: Send + Sync {
    /// Create a decoder for the specified file
    fn create_decoder(&self, path: &Path) -> Result<Box<dyn FileDecoder>, FileDecoderError>;

    /// Check if a file format is supported
    fn supports_format(&self, format: AudioFileFormat) -> bool;

    /// Get list of supported formats
    fn supported_formats(&self) -> Vec<AudioFileFormat>;
}
