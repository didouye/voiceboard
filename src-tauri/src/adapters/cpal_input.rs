//! CPAL-based audio input adapter

use crate::domain::{AudioBuffer, AudioFormat, DeviceId, Sample};
use crate::ports::{AudioInput, AudioInputError};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

/// Audio input adapter using CPAL
pub struct CpalAudioInput {
    stream: Option<cpal::Stream>,
    sender: Option<Sender<AudioBuffer>>,
    receiver: Option<Receiver<AudioBuffer>>,
    format: Option<AudioFormat>,
    is_capturing: Arc<Mutex<bool>>,
}

impl CpalAudioInput {
    pub fn new() -> Self {
        Self {
            stream: None,
            sender: None,
            receiver: None,
            format: None,
            is_capturing: Arc::new(Mutex::new(false)),
        }
    }

    fn find_device(&self, device_id: &DeviceId) -> Result<cpal::Device, AudioInputError> {
        use cpal::traits::{DeviceTrait, HostTrait};

        let host = cpal::default_host();

        // If looking for default device
        if device_id.as_str() == "default" {
            return host
                .default_input_device()
                .ok_or_else(|| AudioInputError::DeviceNotFound("No default input device".into()));
        }

        // Search for device by name
        let devices = host
            .input_devices()
            .map_err(|e| AudioInputError::DeviceNotFound(e.to_string()))?;

        for device in devices {
            if let Ok(name) = device.name() {
                if name == device_id.as_str() {
                    return Ok(device);
                }
            }
        }

        Err(AudioInputError::DeviceNotFound(format!(
            "Device '{}' not found",
            device_id.as_str()
        )))
    }
}

impl Default for CpalAudioInput {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioInput for CpalAudioInput {
    fn start(&mut self, device_id: &DeviceId, format: AudioFormat) -> Result<(), AudioInputError> {
        use cpal::traits::{DeviceTrait, StreamTrait};

        if self.is_capturing() {
            self.stop()?;
        }

        let device = self.find_device(device_id)?;

        let config = cpal::StreamConfig {
            channels: format.channels,
            sample_rate: cpal::SampleRate(format.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let (tx, rx) = mpsc::channel();
        self.sender = Some(tx.clone());
        self.receiver = Some(rx);
        self.format = Some(format);

        let is_capturing = self.is_capturing.clone();
        let channels = format.channels;
        let sample_rate = format.sample_rate;

        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let samples: Vec<Sample> = data.iter().map(|&s| Sample::new(s)).collect();
                    let buffer = AudioBuffer::new(samples, channels, sample_rate);
                    let _ = tx.send(buffer);
                },
                move |err| {
                    tracing::error!("Audio input stream error: {}", err);
                },
                None,
            )
            .map_err(|e| AudioInputError::OpenError(e.to_string()))?;

        stream
            .play()
            .map_err(|e| AudioInputError::StreamError(e.to_string()))?;

        self.stream = Some(stream);
        *is_capturing.lock().unwrap() = true;

        Ok(())
    }

    fn stop(&mut self) -> Result<(), AudioInputError> {
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }
        self.sender = None;
        self.receiver = None;
        *self.is_capturing.lock().unwrap() = false;
        Ok(())
    }

    fn is_capturing(&self) -> bool {
        *self.is_capturing.lock().unwrap()
    }

    fn get_receiver(&self) -> Option<Receiver<AudioBuffer>> {
        // Note: This is a simplified implementation
        // In practice, you might want to use Arc<Mutex<Receiver>> or channels
        None // Receiver can't be cloned, would need different architecture
    }

    fn current_format(&self) -> Option<AudioFormat> {
        self.format
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpal_input_creation() {
        let input = CpalAudioInput::new();
        assert!(!input.is_capturing());
        assert!(input.current_format().is_none());
    }
}
