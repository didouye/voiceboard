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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_decoder_error_file_not_found() {
        let error = FileDecoderError::FileNotFound("/path/to/file.mp3".to_string());
        assert_eq!(format!("{}", error), "File not found: /path/to/file.mp3");
    }

    #[test]
    fn test_file_decoder_error_unsupported_format() {
        let error = FileDecoderError::UnsupportedFormat("aac".to_string());
        assert_eq!(format!("{}", error), "Unsupported format: aac");
    }

    #[test]
    fn test_file_decoder_error_decode_error() {
        let error = FileDecoderError::DecodeError("Invalid header".to_string());
        assert_eq!(format!("{}", error), "Decode error: Invalid header");
    }

    #[test]
    fn test_file_decoder_error_io_error() {
        let error = FileDecoderError::IoError("Permission denied".to_string());
        assert_eq!(format!("{}", error), "IO error: Permission denied");
    }

    #[test]
    fn test_file_decoder_error_invalid_file() {
        let error = FileDecoderError::InvalidFile("Corrupted data".to_string());
        assert_eq!(format!("{}", error), "Invalid file: Corrupted data");
    }

    #[test]
    fn test_audio_file_metadata_creation() {
        let metadata = AudioFileMetadata {
            format: AudioFileFormat::Mp3,
            duration: Duration::from_secs(180),
            audio_format: AudioFormat::default(),
            title: Some("Test Song".to_string()),
            artist: Some("Test Artist".to_string()),
        };

        assert_eq!(metadata.format, AudioFileFormat::Mp3);
        assert_eq!(metadata.duration, Duration::from_secs(180));
        assert_eq!(metadata.title, Some("Test Song".to_string()));
        assert_eq!(metadata.artist, Some("Test Artist".to_string()));
    }

    #[test]
    fn test_audio_file_metadata_without_tags() {
        let metadata = AudioFileMetadata {
            format: AudioFileFormat::Wav,
            duration: Duration::from_secs(60),
            audio_format: AudioFormat::default(),
            title: None,
            artist: None,
        };

        assert!(metadata.title.is_none());
        assert!(metadata.artist.is_none());
    }

    #[test]
    fn test_audio_file_metadata_clone() {
        let metadata = AudioFileMetadata {
            format: AudioFileFormat::Flac,
            duration: Duration::from_secs(300),
            audio_format: AudioFormat::default(),
            title: Some("Title".to_string()),
            artist: None,
        };

        let cloned = metadata.clone();
        assert_eq!(cloned.format, metadata.format);
        assert_eq!(cloned.duration, metadata.duration);
    }
}
