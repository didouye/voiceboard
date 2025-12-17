//! Mixer channel entity

use serde::{Deserialize, Serialize};

/// Type of mixer channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelType {
    /// Physical microphone input
    Microphone,
    /// Audio file playback (MP3, OGG, etc.)
    AudioFile,
    /// System audio loopback
    SystemAudio,
}

/// Represents a channel in the mixer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerChannel {
    id: String,
    name: String,
    channel_type: ChannelType,
    volume: f32,
    muted: bool,
    solo: bool,
}

impl MixerChannel {
    pub fn new(id: impl Into<String>, name: impl Into<String>, channel_type: ChannelType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            channel_type,
            volume: 1.0,
            muted: false,
            solo: false,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn channel_type(&self) -> ChannelType {
        self.channel_type
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 2.0); // Allow up to 2x gain
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }

    pub fn is_solo(&self) -> bool {
        self.solo
    }

    pub fn set_solo(&mut self, solo: bool) {
        self.solo = solo;
    }

    /// Calculate effective volume considering mute state
    pub fn effective_volume(&self) -> f32 {
        if self.muted {
            0.0
        } else {
            self.volume
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_creation() {
        let channel = MixerChannel::new("ch1", "Main Mic", ChannelType::Microphone);

        assert_eq!(channel.id(), "ch1");
        assert_eq!(channel.name(), "Main Mic");
        assert_eq!(channel.channel_type(), ChannelType::Microphone);
        assert_eq!(channel.volume(), 1.0);
        assert!(!channel.is_muted());
    }

    #[test]
    fn test_volume_clamping() {
        let mut channel = MixerChannel::new("ch1", "Test", ChannelType::AudioFile);

        channel.set_volume(3.0);
        assert_eq!(channel.volume(), 2.0);

        channel.set_volume(-1.0);
        assert_eq!(channel.volume(), 0.0);
    }

    #[test]
    fn test_effective_volume() {
        let mut channel = MixerChannel::new("ch1", "Test", ChannelType::AudioFile);
        channel.set_volume(0.8);

        assert_eq!(channel.effective_volume(), 0.8);

        channel.set_muted(true);
        assert_eq!(channel.effective_volume(), 0.0);
    }

    #[test]
    fn test_toggle_mute() {
        let mut channel = MixerChannel::new("ch1", "Test", ChannelType::Microphone);

        assert!(!channel.is_muted());
        channel.toggle_mute();
        assert!(channel.is_muted());
        channel.toggle_mute();
        assert!(!channel.is_muted());
    }
}
