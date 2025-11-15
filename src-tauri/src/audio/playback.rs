/// Sound file playback management
use super::decoder::AudioDecoder;
use super::types::{AudioBuffer, AudioFormat, Volume};
use crate::error::{Result, VoiceboardError};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};

/// Sound playback handle
pub struct SoundPlayer {
    /// Sound ID
    id: String,
    /// Decoded audio buffer
    buffer: Arc<AudioBuffer>,
    /// Current playback position (in samples)
    position: usize,
    /// Volume for this sound
    volume: Volume,
    /// Whether this sound is looping
    looping: bool,
}

impl SoundPlayer {
    /// Create a new sound player from a file
    pub fn from_file<P: AsRef<Path>>(id: String, path: P, volume: Volume) -> Result<Self> {
        info!("Loading sound: {} from {}", id, path.as_ref().display());

        // Decode the audio file
        let buffer = AudioDecoder::decode_file(path)?;

        Ok(Self {
            id,
            buffer: Arc::new(buffer),
            position: 0,
            volume,
            looping: false,
        })
    }

    /// Create a new sound player from an audio buffer
    pub fn from_buffer(id: String, buffer: AudioBuffer, volume: Volume) -> Self {
        Self {
            id,
            buffer: Arc::new(buffer),
            position: 0,
            volume,
            looping: false,
        }
    }

    /// Get the next chunk of audio samples
    ///
    /// Returns the number of samples written to the output buffer
    pub fn read(&mut self, output: &mut [f32]) -> usize {
        let available = self.buffer.data.len() - self.position;
        if available == 0 {
            if self.looping {
                self.position = 0;
            } else {
                return 0; // Finished playing
            }
        }

        let to_read = available.min(output.len());

        // Copy samples and apply volume
        for i in 0..to_read {
            output[i] = self.buffer.data[self.position + i] * self.volume.linear();
        }

        self.position += to_read;
        to_read
    }

    /// Reset playback to the beginning
    pub fn reset(&mut self) {
        self.position = 0;
    }

    /// Check if playback is finished
    pub fn is_finished(&self) -> bool {
        !self.looping && self.position >= self.buffer.data.len()
    }

    /// Set whether this sound should loop
    pub fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }

    /// Set the volume for this sound
    pub fn set_volume(&mut self, volume: Volume) {
        self.volume = volume;
    }

    /// Get the sound ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the audio format
    pub fn format(&self) -> AudioFormat {
        self.buffer.format
    }

    /// Get the total duration in seconds
    pub fn duration(&self) -> f32 {
        self.buffer.duration()
    }

    /// Get the current playback position in seconds
    pub fn position_secs(&self) -> f32 {
        let frames = self.position / self.buffer.format.channels as usize;
        self.buffer.format.frames_to_seconds(frames)
    }
}

/// Manages multiple sound players
pub struct PlaybackManager {
    /// Active sound players
    players: Vec<SoundPlayer>,
    /// Output format
    format: AudioFormat,
}

impl PlaybackManager {
    /// Create a new playback manager
    pub fn new(format: AudioFormat) -> Self {
        Self {
            players: Vec::new(),
            format,
        }
    }

    /// Add a sound player
    pub fn add_player(&mut self, player: SoundPlayer) {
        debug!("Adding sound player: {}", player.id());
        self.players.push(player);
    }

    /// Remove a sound player by ID
    pub fn remove_player(&mut self, id: &str) -> Option<SoundPlayer> {
        if let Some(index) = self.players.iter().position(|p| p.id() == id) {
            debug!("Removing sound player: {}", id);
            Some(self.players.remove(index))
        } else {
            None
        }
    }

    /// Get a mutable reference to a player by ID
    pub fn get_player_mut(&mut self, id: &str) -> Option<&mut SoundPlayer> {
        self.players.iter_mut().find(|p| p.id() == id)
    }

    /// Remove all finished players
    pub fn cleanup_finished(&mut self) -> Vec<String> {
        let mut finished = Vec::new();

        self.players.retain(|player| {
            if player.is_finished() {
                finished.push(player.id().to_string());
                false
            } else {
                true
            }
        });

        finished
    }

    /// Get the number of active players
    pub fn active_count(&self) -> usize {
        self.players.len()
    }

    /// Clear all players
    pub fn clear(&mut self) {
        debug!("Clearing all sound players");
        self.players.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_player_basic() {
        let format = AudioFormat::standard();
        let buffer = AudioBuffer::zeros(format, 100);

        let mut player = SoundPlayer::from_buffer("test".to_string(), buffer, Volume::max());

        assert_eq!(player.id(), "test");
        assert!(!player.is_finished());

        let mut output = vec![0.0f32; 200];
        let read = player.read(&mut output);

        assert_eq!(read, 200);
        assert!(player.is_finished());
    }

    #[test]
    fn test_playback_manager() {
        let format = AudioFormat::standard();
        let mut manager = PlaybackManager::new(format);

        let buffer = AudioBuffer::zeros(format, 100);
        let player = SoundPlayer::from_buffer("test".to_string(), buffer, Volume::max());

        manager.add_player(player);
        assert_eq!(manager.active_count(), 1);

        manager.remove_player("test");
        assert_eq!(manager.active_count(), 0);
    }
}
