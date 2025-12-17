//! Audio buffer - A collection of samples

use super::Sample;
use serde::{Deserialize, Serialize};

/// A buffer containing audio samples for processing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioBuffer {
    samples: Vec<Sample>,
    channels: u16,
    sample_rate: u32,
}

impl AudioBuffer {
    /// Creates a new audio buffer
    pub fn new(samples: Vec<Sample>, channels: u16, sample_rate: u32) -> Self {
        Self {
            samples,
            channels,
            sample_rate,
        }
    }

    /// Creates a silent buffer with the specified duration
    pub fn silence(duration_ms: u32, channels: u16, sample_rate: u32) -> Self {
        let num_samples = (sample_rate * duration_ms / 1000) as usize * channels as usize;
        Self {
            samples: vec![Sample::silence(); num_samples],
            channels,
            sample_rate,
        }
    }

    /// Returns the samples in the buffer
    pub fn samples(&self) -> &[Sample] {
        &self.samples
    }

    /// Returns mutable access to samples
    pub fn samples_mut(&mut self) -> &mut [Sample] {
        &mut self.samples
    }

    /// Returns the number of channels
    pub fn channels(&self) -> u16 {
        self.channels
    }

    /// Returns the sample rate in Hz
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Returns the number of frames (samples per channel)
    pub fn frame_count(&self) -> usize {
        self.samples.len() / self.channels as usize
    }

    /// Returns the duration in milliseconds
    pub fn duration_ms(&self) -> u32 {
        (self.frame_count() as u32 * 1000) / self.sample_rate
    }

    /// Converts the buffer to raw f32 samples
    pub fn to_raw_f32(&self) -> Vec<f32> {
        self.samples.iter().map(|s| s.value()).collect()
    }

    /// Creates a buffer from raw f32 samples
    pub fn from_raw_f32(samples: Vec<f32>, channels: u16, sample_rate: u32) -> Self {
        Self {
            samples: samples.into_iter().map(Sample::new).collect(),
            channels,
            sample_rate,
        }
    }

    /// Mix this buffer with another buffer of the same format
    pub fn mix(&self, other: &AudioBuffer) -> Result<AudioBuffer, BufferError> {
        if self.channels != other.channels {
            return Err(BufferError::ChannelMismatch {
                expected: self.channels,
                got: other.channels,
            });
        }
        if self.sample_rate != other.sample_rate {
            return Err(BufferError::SampleRateMismatch {
                expected: self.sample_rate,
                got: other.sample_rate,
            });
        }

        let min_len = self.samples.len().min(other.samples.len());
        let mixed: Vec<Sample> = self.samples[..min_len]
            .iter()
            .zip(other.samples[..min_len].iter())
            .map(|(a, b)| a.mix(b))
            .collect();

        Ok(AudioBuffer::new(mixed, self.channels, self.sample_rate))
    }

    /// Apply gain to all samples in the buffer
    pub fn apply_gain(&mut self, gain: f32) {
        for sample in &mut self.samples {
            *sample = sample.apply_gain(gain);
        }
    }
}

/// Errors that can occur when working with audio buffers
#[derive(Debug, Clone, thiserror::Error)]
pub enum BufferError {
    #[error("Channel mismatch: expected {expected}, got {got}")]
    ChannelMismatch { expected: u16, got: u16 },

    #[error("Sample rate mismatch: expected {expected}Hz, got {got}Hz")]
    SampleRateMismatch { expected: u32, got: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_creation() {
        let samples = vec![Sample::new(0.5), Sample::new(-0.5)];
        let buffer = AudioBuffer::new(samples, 2, 44100);

        assert_eq!(buffer.channels(), 2);
        assert_eq!(buffer.sample_rate(), 44100);
        assert_eq!(buffer.frame_count(), 1);
    }

    #[test]
    fn test_silence_buffer() {
        let buffer = AudioBuffer::silence(100, 2, 44100);

        assert_eq!(buffer.channels(), 2);
        assert_eq!(buffer.sample_rate(), 44100);
        assert!(buffer.samples().iter().all(|s| s.value() == 0.0));
    }

    #[test]
    fn test_buffer_mixing() {
        let buffer1 = AudioBuffer::from_raw_f32(vec![0.5, 0.5], 2, 44100);
        let buffer2 = AudioBuffer::from_raw_f32(vec![0.5, 0.5], 2, 44100);

        let mixed = buffer1.mix(&buffer2).unwrap();
        assert!(mixed.samples().iter().all(|s| (s.value() - 0.5).abs() < 0.001));
    }

    #[test]
    fn test_buffer_mixing_channel_mismatch() {
        let buffer1 = AudioBuffer::from_raw_f32(vec![0.5, 0.5], 1, 44100);
        let buffer2 = AudioBuffer::from_raw_f32(vec![0.5, 0.5], 2, 44100);

        let result = buffer1.mix(&buffer2);
        assert!(matches!(result, Err(BufferError::ChannelMismatch { .. })));
    }
}
