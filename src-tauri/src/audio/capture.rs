/// Microphone audio capture using WASAPI
use super::types::{AudioBuffer, AudioFormat, Sample};
use crate::error::{Result, VoiceboardError};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, StreamConfig};
use ringbuf::{traits::*, HeapRb};
use std::sync::Arc;
use tracing::{debug, error, info};

/// Audio capture configuration
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    /// Target sample rate
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
    /// Buffer size in frames
    pub buffer_size: usize,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            buffer_size: 4800, // 100ms at 48kHz
        }
    }
}

/// Microphone capture handle
pub struct MicrophoneCapture {
    /// CPAL device
    device: Device,
    /// CPAL stream
    stream: Option<Stream>,
    /// Audio format
    format: AudioFormat,
    /// Ring buffer for captured audio
    ring_buffer: Arc<HeapRb<Sample>>,
}

impl MicrophoneCapture {
    /// Create a new microphone capture with the default input device
    pub fn new(config: CaptureConfig) -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| VoiceboardError::AudioDevice("No input device found".to_string()))?;

        Self::with_device(device, config)
    }

    /// Create a new microphone capture with a specific device
    pub fn with_device(device: Device, config: CaptureConfig) -> Result<Self> {
        let device_name = device.name()?;
        info!("Creating microphone capture with device: {}", device_name);

        let format = AudioFormat::new(config.sample_rate, config.channels);

        // Create ring buffer (make it large enough for smooth operation)
        let ring_buffer = HeapRb::<Sample>::new(config.buffer_size * config.channels as usize * 4);
        let ring_buffer = Arc::new(ring_buffer);

        Ok(Self {
            device,
            stream: None,
            format,
            ring_buffer,
        })
    }

    /// Start capturing audio
    pub fn start(&mut self) -> Result<()> {
        if self.stream.is_some() {
            return Err(VoiceboardError::InvalidOperation(
                "Capture already started".to_string(),
            ));
        }

        info!("Starting microphone capture");

        let stream_config = StreamConfig {
            channels: self.format.channels,
            sample_rate: cpal::SampleRate(self.format.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let ring_buffer = self.ring_buffer.clone();

        // Build input stream
        let stream = self
            .device
            .build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // Write captured samples to ring buffer
                    let mut producer = ring_buffer.producer();
                    if let Err(remaining) = producer.push_slice(data) {
                        // Buffer is full, some samples were dropped
                        if !remaining.is_empty() {
                            debug!("Dropped {} input samples (buffer full)", remaining.len());
                        }
                    }
                },
                move |err| {
                    error!("Audio capture stream error: {}", err);
                },
                None,
            )?;

        stream.play()?;
        self.stream = Some(stream);

        info!("Microphone capture started successfully");
        Ok(())
    }

    /// Stop capturing audio
    pub fn stop(&mut self) -> Result<()> {
        if let Some(stream) = self.stream.take() {
            info!("Stopping microphone capture");
            stream.pause()?;
            drop(stream);
        }
        Ok(())
    }

    /// Read captured audio into a buffer
    pub fn read(&self, output: &mut [Sample]) -> usize {
        let mut consumer = self.ring_buffer.consumer();
        consumer.pop_slice(output)
    }

    /// Check if capture is active
    pub fn is_active(&self) -> bool {
        self.stream.is_some()
    }

    /// Get the audio format
    pub fn format(&self) -> AudioFormat {
        self.format
    }

    /// Get the number of samples available to read
    pub fn available(&self) -> usize {
        self.ring_buffer.consumer().occupied_len()
    }
}

impl Drop for MicrophoneCapture {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Helper function to list all available input devices
pub fn list_input_devices() -> Result<Vec<(String, Device)>> {
    let host = cpal::default_host();
    let devices = host.input_devices()?;

    let mut result = Vec::new();
    for device in devices {
        if let Ok(name) = device.name() {
            result.push((name, device));
        }
    }

    Ok(result)
}

/// Helper function to get default input device
pub fn get_default_input_device() -> Result<Device> {
    let host = cpal::default_host();
    host.default_input_device()
        .ok_or_else(|| VoiceboardError::AudioDevice("No default input device".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_input_devices() {
        let devices = list_input_devices().unwrap();
        assert!(!devices.is_empty(), "Should have at least one input device");

        for (name, _) in &devices {
            println!("Input device: {}", name);
        }
    }
}
