//! CPAL-based audio output adapter

use crate::domain::{AudioBuffer, AudioFormat, DeviceId};
use crate::ports::{AudioOutput, AudioOutputError};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{HeapRb, traits::{Consumer, Observer, Producer, Split}};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Size of the ring buffer in frames
const RING_BUFFER_SIZE: usize = 4096;

/// Audio output adapter using CPAL
pub struct CpalAudioOutput {
    stream: Option<cpal::Stream>,
    producer: Option<ringbuf::HeapProd<f32>>,
    format: Option<AudioFormat>,
    is_playing: Arc<AtomicBool>,
}

impl CpalAudioOutput {
    pub fn new() -> Self {
        Self {
            stream: None,
            producer: None,
            format: None,
            is_playing: Arc::new(AtomicBool::new(false)),
        }
    }

    fn find_device(&self, device_id: &DeviceId) -> Result<cpal::Device, AudioOutputError> {
        let host = cpal::default_host();

        // If looking for default device
        if device_id.as_str() == "default" {
            return host
                .default_output_device()
                .ok_or_else(|| AudioOutputError::DeviceNotFound("No default output device".into()));
        }

        // Search for device by name
        let devices = host
            .output_devices()
            .map_err(|e| AudioOutputError::DeviceNotFound(e.to_string()))?;

        for device in devices {
            if let Ok(name) = device.name() {
                if name == device_id.as_str() {
                    return Ok(device);
                }
            }
        }

        Err(AudioOutputError::DeviceNotFound(format!(
            "Device '{}' not found",
            device_id.as_str()
        )))
    }
}

impl Default for CpalAudioOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioOutput for CpalAudioOutput {
    fn start(&mut self, device_id: &DeviceId, format: AudioFormat) -> Result<(), AudioOutputError> {
        if self.is_playing() {
            self.stop()?;
        }

        let device = self.find_device(device_id)?;

        let config = cpal::StreamConfig {
            channels: format.channels,
            sample_rate: cpal::SampleRate(format.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        // Create ring buffer for audio data
        let ring_buffer = HeapRb::<f32>::new(RING_BUFFER_SIZE * format.channels as usize);
        let (producer, mut consumer) = ring_buffer.split();

        self.producer = Some(producer);
        self.format = Some(format);

        let is_playing = self.is_playing.clone();

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // Read from ring buffer
                    let read = consumer.pop_slice(data);

                    // Fill remaining with silence if buffer underrun
                    if read < data.len() {
                        for sample in &mut data[read..] {
                            *sample = 0.0;
                        }
                    }
                },
                move |err| {
                    tracing::error!("Audio output stream error: {}", err);
                },
                None,
            )
            .map_err(|e| AudioOutputError::OpenError(e.to_string()))?;

        stream
            .play()
            .map_err(|e| AudioOutputError::StreamError(e.to_string()))?;

        self.stream = Some(stream);
        is_playing.store(true, Ordering::SeqCst);

        tracing::info!("Audio output started on device: {}", device_id.as_str());
        Ok(())
    }

    fn stop(&mut self) -> Result<(), AudioOutputError> {
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }
        self.producer = None;
        self.is_playing.store(false, Ordering::SeqCst);
        tracing::info!("Audio output stopped");
        Ok(())
    }

    fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::SeqCst)
    }

    fn write(&mut self, buffer: &AudioBuffer) -> Result<(), AudioOutputError> {
        let producer = self
            .producer
            .as_mut()
            .ok_or_else(|| AudioOutputError::StreamError("Output not started".into()))?;

        let samples: Vec<f32> = buffer.to_raw_f32();

        // Try to push samples to ring buffer
        let pushed = producer.push_slice(&samples);

        if pushed < samples.len() {
            // Buffer full - we're producing faster than consuming
            // This is normal, just drop the excess samples
            tracing::trace!("Ring buffer full, dropped {} samples", samples.len() - pushed);
        }

        Ok(())
    }

    fn current_format(&self) -> Option<AudioFormat> {
        self.format
    }

    fn available_frames(&self) -> usize {
        self.producer
            .as_ref()
            .map(|p| p.vacant_len())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_creation() {
        let output = CpalAudioOutput::new();
        assert!(!output.is_playing());
        assert!(output.current_format().is_none());
    }
}
