//! Mixer configuration

use super::MixerChannel;
use crate::domain::audio::AudioFormat;
use serde::{Deserialize, Serialize};

/// Configuration for the audio mixer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerConfig {
    /// Output audio format
    pub output_format: AudioFormat,
    /// Buffer size in frames
    pub buffer_size: u32,
    /// Master volume (0.0 to 1.0)
    pub master_volume: f32,
    /// Channels in the mixer
    pub channels: Vec<MixerChannel>,
}

impl MixerConfig {
    pub fn new(output_format: AudioFormat, buffer_size: u32) -> Self {
        Self {
            output_format,
            buffer_size,
            master_volume: 1.0,
            channels: Vec::new(),
        }
    }

    pub fn with_master_volume(mut self, volume: f32) -> Self {
        self.master_volume = volume.clamp(0.0, 1.0);
        self
    }

    pub fn add_channel(&mut self, channel: MixerChannel) {
        self.channels.push(channel);
    }

    pub fn remove_channel(&mut self, channel_id: &str) -> Option<MixerChannel> {
        if let Some(pos) = self.channels.iter().position(|c| c.id() == channel_id) {
            Some(self.channels.remove(pos))
        } else {
            None
        }
    }

    pub fn get_channel(&self, channel_id: &str) -> Option<&MixerChannel> {
        self.channels.iter().find(|c| c.id() == channel_id)
    }

    pub fn get_channel_mut(&mut self, channel_id: &str) -> Option<&mut MixerChannel> {
        self.channels.iter_mut().find(|c| c.id() == channel_id)
    }
}

impl Default for MixerConfig {
    fn default() -> Self {
        Self::new(AudioFormat::CD_QUALITY, 1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::mixer::ChannelType;

    #[test]
    fn test_mixer_config_creation() {
        let config = MixerConfig::default();
        assert_eq!(config.buffer_size, 1024);
        assert_eq!(config.master_volume, 1.0);
        assert!(config.channels.is_empty());
    }

    #[test]
    fn test_add_remove_channel() {
        let mut config = MixerConfig::default();
        let channel = MixerChannel::new("mic1", "Microphone", ChannelType::Microphone);

        config.add_channel(channel);
        assert_eq!(config.channels.len(), 1);

        let removed = config.remove_channel("mic1");
        assert!(removed.is_some());
        assert!(config.channels.is_empty());
    }

    #[test]
    fn test_master_volume_clamping() {
        let config = MixerConfig::default().with_master_volume(1.5);
        assert_eq!(config.master_volume, 1.0);

        let config = MixerConfig::default().with_master_volume(-0.5);
        assert_eq!(config.master_volume, 0.0);
    }
}
