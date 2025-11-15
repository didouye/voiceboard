/// Audio file decoding using Symphonia
use super::types::{AudioBuffer, AudioFormat, Sample};
use crate::error::{Result, VoiceboardError};
use std::path::Path;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use tracing::{debug, info};

/// Audio file decoder
pub struct AudioDecoder;

impl AudioDecoder {
    /// Decode an audio file to an audio buffer
    ///
    /// This function loads and decodes an entire audio file into memory.
    /// Supported formats: MP3, WAV, OGG, FLAC, AAC
    pub fn decode_file<P: AsRef<Path>>(path: P) -> Result<AudioBuffer> {
        let path = path.as_ref();
        info!("Decoding audio file: {}", path.display());

        // Open the media source
        let file = std::fs::File::open(path)
            .map_err(|e| VoiceboardError::SoundFile(format!("Failed to open file: {}", e)))?;

        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        // Create a hint to help the format registry guess the format
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        // Probe the media source for a format
        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();
        let decoder_opts = DecoderOptions::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .map_err(|e| VoiceboardError::Decoding(format!("Failed to probe format: {}", e)))?;

        let mut format = probed.format;

        // Get the default track
        let track = format
            .default_track()
            .ok_or_else(|| VoiceboardError::Decoding("No default track found".to_string()))?;

        // Create a decoder for the track
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decoder_opts)
            .map_err(|e| VoiceboardError::Decoding(format!("Failed to create decoder: {}", e)))?;

        // Get audio format information
        let codec_params = &track.codec_params;
        let sample_rate = codec_params
            .sample_rate
            .ok_or_else(|| VoiceboardError::AudioFormat("No sample rate".to_string()))?;
        let channels = codec_params
            .channels
            .ok_or_else(|| VoiceboardError::AudioFormat("No channel info".to_string()))?
            .count() as u16;

        let format = AudioFormat::new(sample_rate, channels);
        debug!(
            "Audio format: {}Hz, {} channels",
            format.sample_rate, format.channels
        );

        // Decode all packets
        let mut audio_buffer = AudioBuffer::new(format);
        let mut sample_buf = None;

        loop {
            // Get the next packet from the format reader
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    break;
                }
                Err(e) => {
                    return Err(VoiceboardError::Decoding(format!(
                        "Failed to read packet: {}",
                        e
                    )))
                }
            };

            // Decode the packet
            let decoded = decoder
                .decode(&packet)
                .map_err(|e| VoiceboardError::Decoding(format!("Failed to decode packet: {}", e)))?;

            // Convert decoded audio to f32 samples
            if sample_buf.is_none() {
                let spec = *decoded.spec();
                let duration = decoded.capacity() as u64;
                sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
            }

            if let Some(ref mut buf) = sample_buf {
                buf.copy_interleaved_ref(decoded);
                audio_buffer.data.extend_from_slice(buf.samples());
            }
        }

        info!(
            "Decoded {} frames ({:.2}s) from {}",
            audio_buffer.frames(),
            audio_buffer.duration(),
            path.display()
        );

        Ok(audio_buffer)
    }

    /// Get audio file metadata without decoding
    pub fn get_metadata<P: AsRef<Path>>(path: P) -> Result<AudioFileInfo> {
        let path = path.as_ref();
        let file = std::fs::File::open(path)?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .map_err(|e| VoiceboardError::Decoding(format!("Failed to probe format: {}", e)))?;

        let format = probed.format;
        let track = format
            .default_track()
            .ok_or_else(|| VoiceboardError::Decoding("No default track".to_string()))?;

        let codec_params = &track.codec_params;
        let sample_rate = codec_params.sample_rate.unwrap_or(0);
        let channels = codec_params.channels.map(|c| c.count() as u16).unwrap_or(0);
        let duration_secs = if let (Some(n_frames), Some(sr)) =
            (codec_params.n_frames, codec_params.sample_rate)
        {
            n_frames as f32 / sr as f32
        } else {
            0.0
        };

        Ok(AudioFileInfo {
            sample_rate,
            channels,
            duration_secs,
        })
    }
}

/// Audio file information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioFileInfo {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
    /// Duration in seconds
    pub duration_secs: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires a test audio file
    fn test_decode_file() {
        let buffer = AudioDecoder::decode_file("test.mp3").unwrap();
        assert!(buffer.frames() > 0);
        assert_eq!(buffer.format.channels, 2);
    }
}
