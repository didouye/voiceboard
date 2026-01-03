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
        assert!(mixed
            .samples()
            .iter()
            .all(|s| (s.value() - 0.5).abs() < 0.001));
    }

    #[test]
    fn test_buffer_mixing_channel_mismatch() {
        let buffer1 = AudioBuffer::from_raw_f32(vec![0.5, 0.5], 1, 44100);
        let buffer2 = AudioBuffer::from_raw_f32(vec![0.5, 0.5], 2, 44100);

        let result = buffer1.mix(&buffer2);
        assert!(matches!(result, Err(BufferError::ChannelMismatch { .. })));
    }

    #[test]
    fn test_buffer_mixing_sample_rate_mismatch() {
        let buffer1 = AudioBuffer::from_raw_f32(vec![0.5, 0.5], 2, 44100);
        let buffer2 = AudioBuffer::from_raw_f32(vec![0.5, 0.5], 2, 48000);

        let result = buffer1.mix(&buffer2);
        assert!(matches!(
            result,
            Err(BufferError::SampleRateMismatch { .. })
        ));
    }

    #[test]
    fn test_buffer_to_raw_f32() {
        let buffer = AudioBuffer::from_raw_f32(vec![0.5, -0.5, 0.25], 1, 44100);
        let raw = buffer.to_raw_f32();
        assert_eq!(raw.len(), 3);
        assert_eq!(raw[0], 0.5);
        assert_eq!(raw[1], -0.5);
        assert_eq!(raw[2], 0.25);
    }

    #[test]
    fn test_buffer_from_raw_f32() {
        let buffer = AudioBuffer::from_raw_f32(vec![1.0, -1.0], 2, 48000);
        assert_eq!(buffer.channels(), 2);
        assert_eq!(buffer.sample_rate(), 48000);
        assert_eq!(buffer.samples().len(), 2);
    }

    #[test]
    fn test_buffer_samples_mut() {
        let mut buffer = AudioBuffer::from_raw_f32(vec![0.0, 0.0], 2, 44100);
        buffer.samples_mut()[0] = Sample::new(0.5);
        assert_eq!(buffer.samples()[0].value(), 0.5);
    }

    #[test]
    fn test_buffer_apply_gain() {
        let mut buffer = AudioBuffer::from_raw_f32(vec![0.5, 0.5], 2, 44100);
        buffer.apply_gain(2.0);
        assert!(buffer.samples().iter().all(|s| s.value() == 1.0)); // Clamped
    }

    #[test]
    fn test_buffer_apply_gain_zero() {
        let mut buffer = AudioBuffer::from_raw_f32(vec![0.5, 0.5], 2, 44100);
        buffer.apply_gain(0.0);
        assert!(buffer.samples().iter().all(|s| s.value() == 0.0));
    }

    #[test]
    fn test_buffer_duration_ms() {
        let buffer = AudioBuffer::silence(100, 2, 44100);
        // At 44100 Hz, 100ms = 4410 samples per channel
        assert!(buffer.duration_ms() >= 95 && buffer.duration_ms() <= 105);
    }

    #[test]
    fn test_buffer_frame_count() {
        let samples = vec![Sample::new(0.0); 200]; // 100 frames for stereo
        let buffer = AudioBuffer::new(samples, 2, 44100);
        assert_eq!(buffer.frame_count(), 100);
    }

    #[test]
    fn test_buffer_clone() {
        let buffer1 = AudioBuffer::from_raw_f32(vec![0.5, 0.5], 2, 44100);
        let buffer2 = buffer1.clone();
        assert_eq!(buffer1.samples().len(), buffer2.samples().len());
        assert_eq!(buffer1.channels(), buffer2.channels());
    }

    #[test]
    fn test_buffer_partial_eq() {
        let buffer1 = AudioBuffer::from_raw_f32(vec![0.5, 0.5], 2, 44100);
        let buffer2 = AudioBuffer::from_raw_f32(vec![0.5, 0.5], 2, 44100);
        assert_eq!(buffer1, buffer2);
    }

    #[test]
    fn test_buffer_error_channel_mismatch_display() {
        let error = BufferError::ChannelMismatch {
            expected: 2,
            got: 1,
        };
        let msg = format!("{}", error);
        assert!(msg.contains("Channel mismatch"));
    }

    #[test]
    fn test_buffer_error_sample_rate_mismatch_display() {
        let error = BufferError::SampleRateMismatch {
            expected: 44100,
            got: 48000,
        };
        let msg = format!("{}", error);
        assert!(msg.contains("Sample rate mismatch"));
    }

    #[test]
    fn test_buffer_mix_different_lengths() {
        let buffer1 = AudioBuffer::from_raw_f32(vec![0.5, 0.5, 0.5, 0.5], 2, 44100);
        let buffer2 = AudioBuffer::from_raw_f32(vec![0.5, 0.5], 2, 44100);

        let mixed = buffer1.mix(&buffer2).unwrap();
        // Should use the shorter length
        assert_eq!(mixed.samples().len(), 2);
    }
}
