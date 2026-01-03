//! Audio format specifications

use serde::{Deserialize, Serialize};

/// Supported audio file formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioFileFormat {
    Mp3,
    Ogg,
    Wav,
    Flac,
}

impl AudioFileFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            AudioFileFormat::Mp3 => "mp3",
            AudioFileFormat::Ogg => "ogg",
            AudioFileFormat::Wav => "wav",
            AudioFileFormat::Flac => "flac",
        }
    }

    /// Try to determine format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "mp3" => Some(AudioFileFormat::Mp3),
            "ogg" => Some(AudioFileFormat::Ogg),
            "wav" => Some(AudioFileFormat::Wav),
            "flac" => Some(AudioFileFormat::Flac),
            _ => None,
        }
    }
}

/// Audio stream configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioFormat {
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
}

impl AudioFormat {
    /// Standard CD quality format (44.1kHz, 16-bit, stereo)
    pub const CD_QUALITY: Self = Self {
        sample_rate: 44100,
        channels: 2,
        bits_per_sample: 16,
    };

    /// Standard voice format (16kHz, 16-bit, mono)
    pub const VOICE: Self = Self {
        sample_rate: 16000,
        channels: 1,
        bits_per_sample: 16,
    };

    /// High quality format (48kHz, 24-bit, stereo)
    pub const HIGH_QUALITY: Self = Self {
        sample_rate: 48000,
        channels: 2,
        bits_per_sample: 24,
    };

    pub fn new(sample_rate: u32, channels: u16, bits_per_sample: u16) -> Self {
        Self {
            sample_rate,
            channels,
            bits_per_sample,
        }
    }

    /// Calculate bytes per second for this format
    pub fn bytes_per_second(&self) -> u32 {
        self.sample_rate * self.channels as u32 * (self.bits_per_sample / 8) as u32
    }

    /// Calculate bytes per frame
    pub fn bytes_per_frame(&self) -> u16 {
        self.channels * (self.bits_per_sample / 8)
    }
}

impl Default for AudioFormat {
    fn default() -> Self {
        Self::CD_QUALITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_from_extension() {
        assert_eq!(
            AudioFileFormat::from_extension("mp3"),
            Some(AudioFileFormat::Mp3)
        );
        assert_eq!(
            AudioFileFormat::from_extension("MP3"),
            Some(AudioFileFormat::Mp3)
        );
        assert_eq!(
            AudioFileFormat::from_extension("ogg"),
            Some(AudioFileFormat::Ogg)
        );
        assert_eq!(AudioFileFormat::from_extension("xyz"), None);
    }

    #[test]
    fn test_audio_format_calculations() {
        let format = AudioFormat::CD_QUALITY;
        assert_eq!(format.bytes_per_second(), 176400);
        assert_eq!(format.bytes_per_frame(), 4);
    }

    #[test]
    fn test_audio_file_format_extension() {
        assert_eq!(AudioFileFormat::Mp3.extension(), "mp3");
        assert_eq!(AudioFileFormat::Ogg.extension(), "ogg");
        assert_eq!(AudioFileFormat::Wav.extension(), "wav");
        assert_eq!(AudioFileFormat::Flac.extension(), "flac");
    }

    #[test]
    fn test_audio_file_format_from_extension_all_formats() {
        assert_eq!(
            AudioFileFormat::from_extension("wav"),
            Some(AudioFileFormat::Wav)
        );
        assert_eq!(
            AudioFileFormat::from_extension("flac"),
            Some(AudioFileFormat::Flac)
        );
    }

    #[test]
    fn test_audio_format_cd_quality() {
        let format = AudioFormat::CD_QUALITY;
        assert_eq!(format.sample_rate, 44100);
        assert_eq!(format.channels, 2);
        assert_eq!(format.bits_per_sample, 16);
    }

    #[test]
    fn test_audio_format_voice() {
        let format = AudioFormat::VOICE;
        assert_eq!(format.sample_rate, 16000);
        assert_eq!(format.channels, 1);
        assert_eq!(format.bits_per_sample, 16);
    }

    #[test]
    fn test_audio_format_high_quality() {
        let format = AudioFormat::HIGH_QUALITY;
        assert_eq!(format.sample_rate, 48000);
        assert_eq!(format.channels, 2);
        assert_eq!(format.bits_per_sample, 24);
    }

    #[test]
    fn test_audio_format_new() {
        let format = AudioFormat::new(96000, 4, 32);
        assert_eq!(format.sample_rate, 96000);
        assert_eq!(format.channels, 4);
        assert_eq!(format.bits_per_sample, 32);
    }

    #[test]
    fn test_audio_format_default() {
        let format = AudioFormat::default();
        assert_eq!(format, AudioFormat::CD_QUALITY);
    }

    #[test]
    fn test_audio_format_bytes_per_second_high_quality() {
        let format = AudioFormat::HIGH_QUALITY;
        // 48000 * 2 * 3 = 288000
        assert_eq!(format.bytes_per_second(), 288000);
    }

    #[test]
    fn test_audio_format_bytes_per_frame_high_quality() {
        let format = AudioFormat::HIGH_QUALITY;
        // 2 * 3 = 6
        assert_eq!(format.bytes_per_frame(), 6);
    }

    #[test]
    fn test_audio_format_clone() {
        let format1 = AudioFormat::CD_QUALITY;
        let format2 = format1.clone();
        assert_eq!(format1, format2);
    }

    #[test]
    fn test_audio_file_format_clone() {
        let format1 = AudioFileFormat::Mp3;
        let format2 = format1.clone();
        assert_eq!(format1, format2);
    }

    #[test]
    fn test_audio_file_format_debug() {
        let format = AudioFileFormat::Wav;
        let debug = format!("{:?}", format);
        assert!(debug.contains("Wav"));
    }

    #[test]
    fn test_audio_format_debug() {
        let format = AudioFormat::CD_QUALITY;
        let debug = format!("{:?}", format);
        assert!(debug.contains("AudioFormat"));
    }

    #[test]
    fn test_audio_file_format_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(AudioFileFormat::Mp3);
        set.insert(AudioFileFormat::Ogg);
        assert_eq!(set.len(), 2);
        assert!(set.contains(&AudioFileFormat::Mp3));
    }
}
