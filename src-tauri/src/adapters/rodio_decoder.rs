//! Rodio-based file decoder adapter

use crate::domain::{AudioBuffer, AudioFileFormat, AudioFormat, Sample};
use crate::ports::{AudioFileMetadata, FileDecoder, FileDecoderError, FileDecoderFactory};
use rodio::Source;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

/// File decoder adapter using Rodio
pub struct RodioFileDecoder {
    source: Option<rodio::Decoder<BufReader<File>>>,
    metadata: Option<AudioFileMetadata>,
    position: Duration,
    finished: bool,
    buffer_size: usize,
}

impl RodioFileDecoder {
    pub fn new() -> Self {
        Self {
            source: None,
            metadata: None,
            position: Duration::ZERO,
            finished: false,
            buffer_size: 4096,
        }
    }

    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    fn detect_format(path: &Path) -> Result<AudioFileFormat, FileDecoderError> {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| FileDecoderError::InvalidFile("No file extension".into()))?;

        AudioFileFormat::from_extension(extension)
            .ok_or_else(|| FileDecoderError::UnsupportedFormat(extension.to_string()))
    }
}

impl Default for RodioFileDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl FileDecoder for RodioFileDecoder {
    fn open(&mut self, path: &Path) -> Result<AudioFileMetadata, FileDecoderError> {
        let format = Self::detect_format(path)?;

        let file = File::open(path).map_err(|e| FileDecoderError::IoError(e.to_string()))?;

        let reader = BufReader::new(file);

        let decoder = rodio::Decoder::new(reader)
            .map_err(|e| FileDecoderError::DecodeError(e.to_string()))?;

        // Get format info from decoder
        let sample_rate = decoder.sample_rate();
        let channels = decoder.channels();

        // Duration estimation (rodio doesn't always provide this)
        let duration = decoder.total_duration().unwrap_or(Duration::ZERO);

        let audio_format = AudioFormat::new(sample_rate, channels, 16);

        let metadata = AudioFileMetadata {
            format,
            duration,
            audio_format,
            title: None,
            artist: None,
        };

        self.source = Some(decoder);
        self.metadata = Some(metadata.clone());
        self.position = Duration::ZERO;
        self.finished = false;

        Ok(metadata)
    }

    fn read_next(&mut self) -> Result<Option<AudioBuffer>, FileDecoderError> {
        let source = self
            .source
            .as_mut()
            .ok_or_else(|| FileDecoderError::DecodeError("No file opened".into()))?;

        let metadata = self.metadata.as_ref().unwrap();
        let mut samples = Vec::with_capacity(self.buffer_size);

        for sample in source.by_ref().take(self.buffer_size) {
            samples.push(Sample::from(sample));
        }

        if samples.is_empty() {
            self.finished = true;
            return Ok(None);
        }

        // Update position estimate
        let frames = samples.len() / metadata.audio_format.channels as usize;
        let duration_secs = frames as f64 / metadata.audio_format.sample_rate as f64;
        self.position += Duration::from_secs_f64(duration_secs);

        Ok(Some(AudioBuffer::new(
            samples,
            metadata.audio_format.channels,
            metadata.audio_format.sample_rate,
        )))
    }

    fn seek(&mut self, _position: Duration) -> Result<(), FileDecoderError> {
        // Rodio doesn't support seeking in all formats
        Err(FileDecoderError::DecodeError(
            "Seeking not supported".into(),
        ))
    }

    fn position(&self) -> Duration {
        self.position
    }

    fn duration(&self) -> Option<Duration> {
        self.metadata.as_ref().map(|m| m.duration)
    }

    fn is_finished(&self) -> bool {
        self.finished
    }

    fn reset(&mut self) -> Result<(), FileDecoderError> {
        // Would need to reopen the file
        Err(FileDecoderError::DecodeError(
            "Reset requires reopening file".into(),
        ))
    }

    fn close(&mut self) {
        self.source = None;
        self.metadata = None;
        self.position = Duration::ZERO;
        self.finished = true;
    }
}

/// Factory for creating Rodio-based decoders
pub struct RodioDecoderFactory;

impl RodioDecoderFactory {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RodioDecoderFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl FileDecoderFactory for RodioDecoderFactory {
    fn create_decoder(&self, path: &Path) -> Result<Box<dyn FileDecoder>, FileDecoderError> {
        let mut decoder = RodioFileDecoder::new();
        decoder.open(path)?;
        Ok(Box::new(decoder))
    }

    fn supports_format(&self, format: AudioFileFormat) -> bool {
        matches!(
            format,
            AudioFileFormat::Mp3
                | AudioFileFormat::Ogg
                | AudioFileFormat::Wav
                | AudioFileFormat::Flac
        )
    }

    fn supported_formats(&self) -> Vec<AudioFileFormat> {
        vec![
            AudioFileFormat::Mp3,
            AudioFileFormat::Ogg,
            AudioFileFormat::Wav,
            AudioFileFormat::Flac,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== RodioFileDecoder Tests ====================

    #[test]
    fn test_decoder_creation() {
        let decoder = RodioFileDecoder::new();
        assert!(decoder.source.is_none());
        assert!(!decoder.is_finished());
    }

    #[test]
    fn test_decoder_default() {
        let decoder = RodioFileDecoder::default();
        assert!(decoder.source.is_none());
        assert!(decoder.metadata.is_none());
        assert_eq!(decoder.position, Duration::ZERO);
        assert!(!decoder.finished);
        assert_eq!(decoder.buffer_size, 4096);
    }

    #[test]
    fn test_decoder_with_buffer_size() {
        let decoder = RodioFileDecoder::new().with_buffer_size(8192);
        assert_eq!(decoder.buffer_size, 8192);
    }

    #[test]
    fn test_decoder_with_custom_buffer_size() {
        let decoder = RodioFileDecoder::new().with_buffer_size(1024);
        assert_eq!(decoder.buffer_size, 1024);
    }

    #[test]
    fn test_decoder_initial_position() {
        let decoder = RodioFileDecoder::new();
        assert_eq!(decoder.position(), Duration::ZERO);
    }

    #[test]
    fn test_decoder_initial_duration() {
        let decoder = RodioFileDecoder::new();
        assert!(decoder.duration().is_none());
    }

    #[test]
    fn test_decoder_read_without_open_fails() {
        let mut decoder = RodioFileDecoder::new();
        let result = decoder.read_next();
        assert!(result.is_err());
    }

    #[test]
    fn test_decoder_seek_not_supported() {
        let mut decoder = RodioFileDecoder::new();
        let result = decoder.seek(Duration::from_secs(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_decoder_reset_without_open_fails() {
        let mut decoder = RodioFileDecoder::new();
        let result = decoder.reset();
        assert!(result.is_err());
    }

    #[test]
    fn test_decoder_close_without_open() {
        let mut decoder = RodioFileDecoder::new();
        decoder.close(); // Should not panic
        assert!(decoder.is_finished());
        assert!(decoder.source.is_none());
        assert!(decoder.metadata.is_none());
    }

    #[test]
    fn test_decoder_open_nonexistent_file() {
        let mut decoder = RodioFileDecoder::new();
        let result = decoder.open(Path::new("/nonexistent/path/audio.mp3"));
        assert!(result.is_err());
    }

    #[test]
    fn test_decoder_open_no_extension() {
        let mut decoder = RodioFileDecoder::new();
        let result = decoder.open(Path::new("/some/path/audiofile"));
        assert!(result.is_err());
    }

    #[test]
    fn test_decoder_open_unsupported_format() {
        let mut decoder = RodioFileDecoder::new();
        let result = decoder.open(Path::new("/some/path/audio.xyz"));
        assert!(result.is_err());
    }

    // ==================== RodioDecoderFactory Tests ====================

    #[test]
    fn test_factory_creation() {
        let factory = RodioDecoderFactory::new();
        // Factory has no state, just verify it creates
        let _ = factory;
    }

    #[test]
    fn test_factory_default() {
        let factory = RodioDecoderFactory::default();
        // Verify it supports expected formats
        assert!(factory.supports_format(AudioFileFormat::Mp3));
    }

    #[test]
    fn test_factory_supported_formats() {
        let factory = RodioDecoderFactory::new();
        assert!(factory.supports_format(AudioFileFormat::Mp3));
        assert!(factory.supports_format(AudioFileFormat::Ogg));
        assert!(factory.supports_format(AudioFileFormat::Wav));
    }

    #[test]
    fn test_factory_supports_flac() {
        let factory = RodioDecoderFactory::new();
        assert!(factory.supports_format(AudioFileFormat::Flac));
    }

    #[test]
    fn test_factory_supported_formats_list() {
        let factory = RodioDecoderFactory::new();
        let formats = factory.supported_formats();

        assert_eq!(formats.len(), 4);
        assert!(formats.contains(&AudioFileFormat::Mp3));
        assert!(formats.contains(&AudioFileFormat::Ogg));
        assert!(formats.contains(&AudioFileFormat::Wav));
        assert!(formats.contains(&AudioFileFormat::Flac));
    }

    #[test]
    fn test_factory_create_decoder_nonexistent_file() {
        let factory = RodioDecoderFactory::new();
        let result = factory.create_decoder(Path::new("/nonexistent/audio.mp3"));
        assert!(result.is_err());
    }

    // ==================== Format Detection Tests ====================

    #[test]
    fn test_detect_format_mp3() {
        let result = RodioFileDecoder::detect_format(Path::new("test.mp3"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), AudioFileFormat::Mp3);
    }

    #[test]
    fn test_detect_format_ogg() {
        let result = RodioFileDecoder::detect_format(Path::new("test.ogg"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), AudioFileFormat::Ogg);
    }

    #[test]
    fn test_detect_format_wav() {
        let result = RodioFileDecoder::detect_format(Path::new("test.wav"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), AudioFileFormat::Wav);
    }

    #[test]
    fn test_detect_format_flac() {
        let result = RodioFileDecoder::detect_format(Path::new("test.flac"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), AudioFileFormat::Flac);
    }

    #[test]
    fn test_detect_format_case_insensitive() {
        assert!(RodioFileDecoder::detect_format(Path::new("test.MP3")).is_ok());
        assert!(RodioFileDecoder::detect_format(Path::new("test.Mp3")).is_ok());
        assert!(RodioFileDecoder::detect_format(Path::new("test.WAV")).is_ok());
        assert!(RodioFileDecoder::detect_format(Path::new("test.FLAC")).is_ok());
    }

    #[test]
    fn test_detect_format_no_extension() {
        let result = RodioFileDecoder::detect_format(Path::new("audiofile"));
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_format_unsupported() {
        let result = RodioFileDecoder::detect_format(Path::new("test.aac"));
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_format_unknown_extension() {
        let result = RodioFileDecoder::detect_format(Path::new("test.xyz"));
        assert!(result.is_err());
    }
}
