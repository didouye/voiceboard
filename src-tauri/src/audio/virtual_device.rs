/// Virtual microphone output using WASAPI
///
/// This module handles sending mixed audio to a virtual audio device.
/// On Windows, this works with virtual audio cables like VB-Audio Cable or VoiceMeeter.

use super::types::{AudioFormat, Sample};
use crate::error::{Result, VoiceboardError};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use ringbuf::{traits::*, HeapRb};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Virtual microphone output configuration
#[derive(Debug, Clone)]
pub struct VirtualDeviceConfig {
    /// Target sample rate
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
    /// Buffer size in frames
    pub buffer_size: usize,
}

impl Default for VirtualDeviceConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            buffer_size: 4800, // 100ms at 48kHz
        }
    }
}

/// Virtual microphone output
pub struct VirtualMicrophone {
    /// CPAL device
    device: Device,
    /// CPAL stream
    stream: Option<Stream>,
    /// Audio format
    format: AudioFormat,
    /// Ring buffer for output audio
    ring_buffer: Arc<HeapRb<Sample>>,
}

impl VirtualMicrophone {
    /// Create a new virtual microphone with the default output device
    ///
    /// Note: On Windows, you should select a virtual audio cable device
    /// (like "CABLE Input" from VB-Audio) as the output device.
    pub fn new(config: VirtualDeviceConfig) -> Result<Self> {
        let host = cpal::default_host();

        // Try to find a virtual audio cable device
        let device = Self::find_virtual_cable(&host)
            .or_else(|| host.default_output_device())
            .ok_or_else(|| {
                VoiceboardError::AudioDevice("No output device found".to_string())
            })?;

        Self::with_device(device, config)
    }

    /// Create a new virtual microphone with a specific device
    pub fn with_device(device: Device, config: VirtualDeviceConfig) -> Result<Self> {
        let device_name = device.name()?;
        info!("Creating virtual microphone with device: {}", device_name);

        let format = AudioFormat::new(config.sample_rate, config.channels);

        // Create ring buffer
        let ring_buffer = HeapRb::<Sample>::new(config.buffer_size * config.channels as usize * 4);
        let ring_buffer = Arc::new(ring_buffer);

        Ok(Self {
            device,
            stream: None,
            format,
            ring_buffer,
        })
    }

    /// Try to find a virtual audio cable device
    ///
    /// Looks for common virtual cable devices:
    /// - VB-Audio Cable (CABLE Input)
    /// - VoiceMeeter (VoiceMeeter Input)
    fn find_virtual_cable(host: &cpal::Host) -> Option<Device> {
        let devices = host.output_devices().ok()?;

        for device in devices {
            if let Ok(name) = device.name() {
                let name_lower = name.to_lowercase();
                if name_lower.contains("cable")
                    || name_lower.contains("voicemeeter")
                    || name_lower.contains("virtual")
                {
                    info!("Found virtual audio cable: {}", name);
                    return Some(device);
                }
            }
        }

        warn!("No virtual audio cable found. Install VB-Audio Cable or VoiceMeeter.");
        None
    }

    /// Start outputting audio
    pub fn start(&mut self) -> Result<()> {
        if self.stream.is_some() {
            return Err(VoiceboardError::InvalidOperation(
                "Virtual microphone already started".to_string(),
            ));
        }

        info!("Starting virtual microphone output");

        let stream_config = StreamConfig {
            channels: self.format.channels,
            sample_rate: cpal::SampleRate(self.format.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let ring_buffer = self.ring_buffer.clone();

        // Build output stream
        let stream = self
            .device
            .build_output_stream(
                &stream_config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // Read samples from ring buffer
                    let mut consumer = ring_buffer.consumer();
                    let read = consumer.pop_slice(data);

                    // Fill remaining with silence if not enough data
                    if read < data.len() {
                        for sample in &mut data[read..] {
                            *sample = 0.0;
                        }
                    }
                },
                move |err| {
                    error!("Virtual microphone output stream error: {}", err);
                },
                None,
            )?;

        stream.play()?;
        self.stream = Some(stream);

        info!("Virtual microphone output started successfully");
        Ok(())
    }

    /// Stop outputting audio
    pub fn stop(&mut self) -> Result<()> {
        if let Some(stream) = self.stream.take() {
            info!("Stopping virtual microphone output");
            stream.pause()?;
            drop(stream);
        }
        Ok(())
    }

    /// Write audio samples to the virtual microphone
    pub fn write(&self, samples: &[Sample]) -> usize {
        let mut producer = self.ring_buffer.producer();
        match producer.push_slice(samples) {
            Ok(_) => samples.len(),
            Err(remaining) => {
                let written = samples.len() - remaining.len();
                if !remaining.is_empty() {
                    debug!("Dropped {} output samples (buffer full)", remaining.len());
                }
                written
            }
        }
    }

    /// Check if output is active
    pub fn is_active(&self) -> bool {
        self.stream.is_some()
    }

    /// Get the audio format
    pub fn format(&self) -> AudioFormat {
        self.format
    }

    /// Get the amount of free space in the buffer
    pub fn free_space(&self) -> usize {
        self.ring_buffer.producer().vacant_len()
    }
}

impl Drop for VirtualMicrophone {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Helper function to list all available output devices
pub fn list_output_devices() -> Result<Vec<(String, Device)>> {
    let host = cpal::default_host();
    let devices = host.output_devices()?;

    let mut result = Vec::new();
    for device in devices {
        if let Ok(name) = device.name() {
            result.push((name, device));
        }
    }

    Ok(result)
}

/// Helper function to find virtual audio cable devices
pub fn find_virtual_cables() -> Result<Vec<(String, Device)>> {
    let all_devices = list_output_devices()?;

    let virtual_devices: Vec<_> = all_devices
        .into_iter()
        .filter(|(name, _)| {
            let name_lower = name.to_lowercase();
            name_lower.contains("cable")
                || name_lower.contains("voicemeeter")
                || name_lower.contains("virtual")
        })
        .collect();

    Ok(virtual_devices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_output_devices() {
        let devices = list_output_devices().unwrap();
        assert!(!devices.is_empty(), "Should have at least one output device");

        for (name, _) in &devices {
            println!("Output device: {}", name);
        }
    }

    #[test]
    fn test_find_virtual_cables() {
        let cables = find_virtual_cables().unwrap();
        for (name, _) in &cables {
            println!("Virtual cable: {}", name);
        }
    }
}
