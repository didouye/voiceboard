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

        let file = File::open(path)
            .map_err(|e| FileDecoderError::IoError(e.to_string()))?;

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
        let source = self.source.as_mut()
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
        Err(FileDecoderError::DecodeError("Seeking not supported".into()))
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
        Err(FileDecoderError::DecodeError("Reset requires reopening file".into()))
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
            AudioFileFormat::Mp3 | AudioFileFormat::Ogg | AudioFileFormat::Wav | AudioFileFormat::Flac
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

    #[test]
    fn test_decoder_creation() {
        let decoder = RodioFileDecoder::new();
        assert!(decoder.source.is_none());
        assert!(!decoder.is_finished());
    }

    #[test]
    fn test_factory_supported_formats() {
        let factory = RodioDecoderFactory::new();
        assert!(factory.supports_format(AudioFileFormat::Mp3));
        assert!(factory.supports_format(AudioFileFormat::Ogg));
        assert!(factory.supports_format(AudioFileFormat::Wav));
    }
}
