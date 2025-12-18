//! Application services - Business logic orchestration

use crate::domain::{AudioBuffer, MixerChannel, MixerConfig};
use crate::ports::{
    AudioInput, AudioInputError, AudioOutput, AudioOutputError, DeviceManager,
    DeviceManagerError, FileDecoderError,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Error types for the mixer service
#[derive(Debug, thiserror::Error)]
pub enum MixerServiceError {
    #[error("Input error: {0}")]
    InputError(#[from] AudioInputError),

    #[error("Output error: {0}")]
    OutputError(#[from] AudioOutputError),

    #[error("Device error: {0}")]
    DeviceError(#[from] DeviceManagerError),

    #[error("Decoder error: {0}")]
    DecoderError(#[from] FileDecoderError),

    #[error("Mixer not started")]
    NotStarted,

    #[error("Channel not found: {0}")]
    ChannelNotFound(String),
}

/// Service for managing audio mixing operations
pub struct MixerService<I, O, D>
where
    I: AudioInput,
    O: AudioOutput,
    D: DeviceManager,
{
    input: Arc<RwLock<I>>,
    output: Arc<RwLock<O>>,
    device_manager: Arc<D>,
    config: Arc<RwLock<MixerConfig>>,
    is_running: Arc<RwLock<bool>>,
}

impl<I, O, D> MixerService<I, O, D>
where
    I: AudioInput,
    O: AudioOutput,
    D: DeviceManager,
{
    pub fn new(input: I, output: O, device_manager: D) -> Self {
        Self {
            input: Arc::new(RwLock::new(input)),
            output: Arc::new(RwLock::new(output)),
            device_manager: Arc::new(device_manager),
            config: Arc::new(RwLock::new(MixerConfig::default())),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the mixer with the current configuration
    pub async fn start(&self) -> Result<(), MixerServiceError> {
        let config = self.config.read().await;
        let format = config.output_format;
        drop(config);

        // Start would initialize input/output streams
        // This is a simplified implementation
        *self.is_running.write().await = true;

        tracing::info!("Mixer service started with format: {:?}", format);
        Ok(())
    }

    /// Stop the mixer
    pub async fn stop(&self) -> Result<(), MixerServiceError> {
        *self.is_running.write().await = false;

        self.input.write().await.stop()?;
        self.output.write().await.stop()?;

        tracing::info!("Mixer service stopped");
        Ok(())
    }

    /// Check if the mixer is currently running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Add a channel to the mixer
    pub async fn add_channel(&self, channel: MixerChannel) -> Result<(), MixerServiceError> {
        let mut config = self.config.write().await;
        config.add_channel(channel);
        Ok(())
    }

    /// Remove a channel from the mixer
    pub async fn remove_channel(&self, channel_id: &str) -> Result<(), MixerServiceError> {
        let mut config = self.config.write().await;
        config
            .remove_channel(channel_id)
            .ok_or_else(|| MixerServiceError::ChannelNotFound(channel_id.to_string()))?;
        Ok(())
    }

    /// Set volume for a specific channel
    pub async fn set_channel_volume(
        &self,
        channel_id: &str,
        volume: f32,
    ) -> Result<(), MixerServiceError> {
        let mut config = self.config.write().await;
        let channel = config
            .get_channel_mut(channel_id)
            .ok_or_else(|| MixerServiceError::ChannelNotFound(channel_id.to_string()))?;
        channel.set_volume(volume);
        Ok(())
    }

    /// Set mute state for a channel
    pub async fn set_channel_muted(
        &self,
        channel_id: &str,
        muted: bool,
    ) -> Result<(), MixerServiceError> {
        let mut config = self.config.write().await;
        let channel = config
            .get_channel_mut(channel_id)
            .ok_or_else(|| MixerServiceError::ChannelNotFound(channel_id.to_string()))?;
        channel.set_muted(muted);
        Ok(())
    }

    /// Set master volume
    pub async fn set_master_volume(&self, volume: f32) -> Result<(), MixerServiceError> {
        let mut config = self.config.write().await;
        config.master_volume = volume.clamp(0.0, 1.0);
        Ok(())
    }

    /// Get current mixer configuration
    pub async fn get_config(&self) -> MixerConfig {
        self.config.read().await.clone()
    }

    /// Mix multiple audio buffers together
    pub fn mix_buffers(buffers: &[AudioBuffer], weights: &[f32]) -> Option<AudioBuffer> {
        if buffers.is_empty() || buffers.len() != weights.len() {
            return None;
        }

        let first = &buffers[0];
        let mut result = first.clone();

        for (i, buffer) in buffers.iter().enumerate().skip(1) {
            if let Ok(mixed) = result.mix(buffer) {
                result = mixed;
            }
        }

        // Apply weights (simplified - in practice would be per-sample)
        let total_weight: f32 = weights.iter().sum();
        if total_weight > 0.0 {
            result.apply_gain(1.0 / total_weight);
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ChannelType;

    // Tests would use mock implementations of the ports
    // Example structure for future tests:

    #[test]
    fn test_mix_buffers_empty() {
        let result = MixerService::<
            crate::adapters::CpalAudioInput,
            crate::adapters::CpalAudioInput, // Placeholder
            crate::adapters::CpalDeviceManager,
        >::mix_buffers(&[], &[]);
        assert!(result.is_none());
    }
}
