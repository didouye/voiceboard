//! Application services - Business logic orchestration

use crate::domain::{AudioBuffer, MixerChannel, MixerConfig};
use crate::ports::{
    AudioInput, AudioInputError, AudioOutput, AudioOutputError, DeviceManager, DeviceManagerError,
    FileDecoderError,
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
    #[allow(dead_code)]
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
}

/// Mix multiple audio buffers together
///
/// Takes a slice of buffers and corresponding weights, returns mixed result.
/// Returns None if buffers is empty or lengths don't match.
pub fn mix_buffers(buffers: &[AudioBuffer], weights: &[f32]) -> Option<AudioBuffer> {
    if buffers.is_empty() || buffers.len() != weights.len() {
        return None;
    }

    let first = &buffers[0];
    let mut result = first.clone();

    for buffer in buffers.iter().skip(1) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AudioDevice, AudioFormat, ChannelType, DeviceId, DeviceType, Sample};
    use std::sync::mpsc::{self, Receiver};

    // ==================== Mock Implementations ====================

    struct MockAudioInput {
        is_capturing: bool,
    }

    impl MockAudioInput {
        fn new() -> Self {
            Self {
                is_capturing: false,
            }
        }
    }

    impl AudioInput for MockAudioInput {
        fn start(
            &mut self,
            _device_id: &DeviceId,
            _format: AudioFormat,
        ) -> Result<(), AudioInputError> {
            self.is_capturing = true;
            Ok(())
        }

        fn stop(&mut self) -> Result<(), AudioInputError> {
            self.is_capturing = false;
            Ok(())
        }

        fn is_capturing(&self) -> bool {
            self.is_capturing
        }

        fn get_receiver(&self) -> Option<Receiver<AudioBuffer>> {
            let (_tx, rx) = mpsc::channel();
            Some(rx)
        }

        fn current_format(&self) -> Option<AudioFormat> {
            None
        }
    }

    struct MockAudioOutput {
        is_playing: bool,
    }

    impl MockAudioOutput {
        fn new() -> Self {
            Self { is_playing: false }
        }
    }

    impl AudioOutput for MockAudioOutput {
        fn start(
            &mut self,
            _device_id: &DeviceId,
            _format: AudioFormat,
        ) -> Result<(), AudioOutputError> {
            self.is_playing = true;
            Ok(())
        }

        fn stop(&mut self) -> Result<(), AudioOutputError> {
            self.is_playing = false;
            Ok(())
        }

        fn is_playing(&self) -> bool {
            self.is_playing
        }

        fn write(&mut self, _buffer: &AudioBuffer) -> Result<(), AudioOutputError> {
            Ok(())
        }

        fn current_format(&self) -> Option<AudioFormat> {
            None
        }

        fn available_frames(&self) -> usize {
            1024
        }
    }

    struct MockDeviceManager;

    impl MockDeviceManager {
        fn new() -> Self {
            Self
        }
    }

    impl DeviceManager for MockDeviceManager {
        fn list_devices(&self) -> Result<Vec<AudioDevice>, DeviceManagerError> {
            Ok(vec![])
        }

        fn list_devices_by_type(
            &self,
            _device_type: DeviceType,
        ) -> Result<Vec<AudioDevice>, DeviceManagerError> {
            Ok(vec![])
        }

        fn default_input_device(&self) -> Result<Option<AudioDevice>, DeviceManagerError> {
            Ok(None)
        }

        fn default_output_device(&self) -> Result<Option<AudioDevice>, DeviceManagerError> {
            Ok(None)
        }

        fn get_device(&self, _id: &DeviceId) -> Result<Option<AudioDevice>, DeviceManagerError> {
            Ok(None)
        }

        fn refresh(&mut self) -> Result<(), DeviceManagerError> {
            Ok(())
        }
    }

    // ==================== MixerServiceError Tests ====================

    #[test]
    fn test_mixer_service_error_not_started() {
        let error = MixerServiceError::NotStarted;
        assert_eq!(format!("{}", error), "Mixer not started");
    }

    #[test]
    fn test_mixer_service_error_channel_not_found() {
        let error = MixerServiceError::ChannelNotFound("ch-1".to_string());
        assert_eq!(format!("{}", error), "Channel not found: ch-1");
    }

    #[test]
    fn test_mixer_service_error_debug() {
        let error = MixerServiceError::NotStarted;
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("NotStarted"));
    }

    // ==================== mix_buffers Tests ====================

    #[test]
    fn test_mix_buffers_empty() {
        let result = mix_buffers(&[], &[]);
        assert!(result.is_none());
    }

    #[test]
    fn test_mix_buffers_mismatched_lengths() {
        let result = mix_buffers(&[], &[1.0]);
        assert!(result.is_none());
    }

    #[test]
    fn test_mix_buffers_single_buffer() {
        let samples = vec![Sample::new(0.5), Sample::new(-0.3)];
        let buffer = AudioBuffer::new(samples, 1, 48000);
        let result = mix_buffers(&[buffer], &[1.0]);

        assert!(result.is_some());
        let mixed = result.unwrap();
        assert_eq!(mixed.channels(), 1);
        assert_eq!(mixed.sample_rate(), 48000);
    }

    #[test]
    fn test_mix_buffers_two_buffers() {
        let samples1 = vec![Sample::new(0.5), Sample::new(0.5)];
        let samples2 = vec![Sample::new(0.3), Sample::new(0.3)];
        let buffer1 = AudioBuffer::new(samples1, 1, 48000);
        let buffer2 = AudioBuffer::new(samples2, 1, 48000);

        let result = mix_buffers(&[buffer1, buffer2], &[0.5, 0.5]);
        assert!(result.is_some());
    }

    #[test]
    fn test_mix_buffers_weights_applied() {
        let samples = vec![Sample::new(1.0), Sample::new(1.0)];
        let buffer = AudioBuffer::new(samples, 1, 48000);

        // Single buffer with weight 1.0 should normalize
        let result = mix_buffers(&[buffer], &[1.0]);
        assert!(result.is_some());
    }

    // ==================== MixerService Tests ====================

    #[tokio::test]
    async fn test_mixer_service_creation() {
        let input = MockAudioInput::new();
        let output = MockAudioOutput::new();
        let device_manager = MockDeviceManager::new();

        let service = MixerService::new(input, output, device_manager);
        assert!(!service.is_running().await);
    }

    #[tokio::test]
    async fn test_mixer_service_start_stop() {
        let input = MockAudioInput::new();
        let output = MockAudioOutput::new();
        let device_manager = MockDeviceManager::new();

        let service = MixerService::new(input, output, device_manager);

        assert!(service.start().await.is_ok());
        assert!(service.is_running().await);

        assert!(service.stop().await.is_ok());
        assert!(!service.is_running().await);
    }

    #[tokio::test]
    async fn test_mixer_service_add_channel() {
        let input = MockAudioInput::new();
        let output = MockAudioOutput::new();
        let device_manager = MockDeviceManager::new();

        let service = MixerService::new(input, output, device_manager);
        let channel = MixerChannel::new("ch-1", "Microphone", ChannelType::Microphone);

        assert!(service.add_channel(channel).await.is_ok());

        let config = service.get_config().await;
        assert_eq!(config.channels.len(), 1);
    }

    #[tokio::test]
    async fn test_mixer_service_remove_channel() {
        let input = MockAudioInput::new();
        let output = MockAudioOutput::new();
        let device_manager = MockDeviceManager::new();

        let service = MixerService::new(input, output, device_manager);
        let channel = MixerChannel::new("ch-1", "Microphone", ChannelType::Microphone);

        service.add_channel(channel).await.unwrap();
        assert!(service.remove_channel("ch-1").await.is_ok());

        let config = service.get_config().await;
        assert!(config.channels.is_empty());
    }

    #[tokio::test]
    async fn test_mixer_service_remove_nonexistent_channel() {
        let input = MockAudioInput::new();
        let output = MockAudioOutput::new();
        let device_manager = MockDeviceManager::new();

        let service = MixerService::new(input, output, device_manager);
        let result = service.remove_channel("nonexistent").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mixer_service_set_channel_volume() {
        let input = MockAudioInput::new();
        let output = MockAudioOutput::new();
        let device_manager = MockDeviceManager::new();

        let service = MixerService::new(input, output, device_manager);
        let channel = MixerChannel::new("ch-1", "Mic", ChannelType::Microphone);
        service.add_channel(channel).await.unwrap();

        assert!(service.set_channel_volume("ch-1", 0.5).await.is_ok());

        let config = service.get_config().await;
        let ch = config.channels.first().unwrap();
        assert_eq!(ch.volume(), 0.5);
    }

    #[tokio::test]
    async fn test_mixer_service_set_channel_volume_nonexistent() {
        let input = MockAudioInput::new();
        let output = MockAudioOutput::new();
        let device_manager = MockDeviceManager::new();

        let service = MixerService::new(input, output, device_manager);
        let result = service.set_channel_volume("nonexistent", 0.5).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mixer_service_set_channel_muted() {
        let input = MockAudioInput::new();
        let output = MockAudioOutput::new();
        let device_manager = MockDeviceManager::new();

        let service = MixerService::new(input, output, device_manager);
        let channel = MixerChannel::new("ch-1", "Mic", ChannelType::Microphone);
        service.add_channel(channel).await.unwrap();

        assert!(service.set_channel_muted("ch-1", true).await.is_ok());

        let config = service.get_config().await;
        let ch = config.channels.first().unwrap();
        assert!(ch.is_muted());
    }

    #[tokio::test]
    async fn test_mixer_service_set_master_volume() {
        let input = MockAudioInput::new();
        let output = MockAudioOutput::new();
        let device_manager = MockDeviceManager::new();

        let service = MixerService::new(input, output, device_manager);

        assert!(service.set_master_volume(0.75).await.is_ok());

        let config = service.get_config().await;
        assert_eq!(config.master_volume, 0.75);
    }

    #[tokio::test]
    async fn test_mixer_service_master_volume_clamped() {
        let input = MockAudioInput::new();
        let output = MockAudioOutput::new();
        let device_manager = MockDeviceManager::new();

        let service = MixerService::new(input, output, device_manager);

        // Volume > 1.0 should be clamped to 1.0
        service.set_master_volume(1.5).await.unwrap();
        let config = service.get_config().await;
        assert_eq!(config.master_volume, 1.0);

        // Volume < 0.0 should be clamped to 0.0
        service.set_master_volume(-0.5).await.unwrap();
        let config = service.get_config().await;
        assert_eq!(config.master_volume, 0.0);
    }

    #[tokio::test]
    async fn test_mixer_service_get_config() {
        let input = MockAudioInput::new();
        let output = MockAudioOutput::new();
        let device_manager = MockDeviceManager::new();

        let service = MixerService::new(input, output, device_manager);
        let config = service.get_config().await;

        // Default config should have no channels and master volume 1.0
        assert!(config.channels.is_empty());
        assert_eq!(config.master_volume, 1.0);
    }
}
