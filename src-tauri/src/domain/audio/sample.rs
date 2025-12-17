//! Audio sample value object

use serde::{Deserialize, Serialize};

/// Represents a single audio sample
/// Using f32 as the canonical internal format for processing
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Sample(f32);

impl Sample {
    /// Creates a new sample, clamping the value to [-1.0, 1.0]
    pub fn new(value: f32) -> Self {
        Self(value.clamp(-1.0, 1.0))
    }

    /// Creates a silent sample (zero amplitude)
    pub fn silence() -> Self {
        Self(0.0)
    }

    /// Returns the raw f32 value
    pub fn value(&self) -> f32 {
        self.0
    }

    /// Mix two samples together with equal weight
    pub fn mix(&self, other: &Sample) -> Sample {
        Sample::new((self.0 + other.0) / 2.0)
    }

    /// Mix two samples with custom weights (weights should sum to 1.0)
    pub fn mix_weighted(&self, other: &Sample, self_weight: f32, other_weight: f32) -> Sample {
        Sample::new(self.0 * self_weight + other.0 * other_weight)
    }

    /// Apply gain to the sample
    pub fn apply_gain(&self, gain: f32) -> Sample {
        Sample::new(self.0 * gain)
    }
}

impl Default for Sample {
    fn default() -> Self {
        Self::silence()
    }
}

impl From<f32> for Sample {
    fn from(value: f32) -> Self {
        Sample::new(value)
    }
}

impl From<Sample> for f32 {
    fn from(sample: Sample) -> Self {
        sample.0
    }
}

impl From<i16> for Sample {
    fn from(value: i16) -> Self {
        Sample::new(value as f32 / i16::MAX as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_clamping() {
        let sample = Sample::new(1.5);
        assert_eq!(sample.value(), 1.0);

        let sample = Sample::new(-1.5);
        assert_eq!(sample.value(), -1.0);
    }

    #[test]
    fn test_sample_mixing() {
        let a = Sample::new(0.5);
        let b = Sample::new(0.5);
        let mixed = a.mix(&b);
        assert_eq!(mixed.value(), 0.5);
    }

    #[test]
    fn test_weighted_mixing() {
        let a = Sample::new(1.0);
        let b = Sample::new(0.0);
        let mixed = a.mix_weighted(&b, 0.7, 0.3);
        assert!((mixed.value() - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_gain_application() {
        let sample = Sample::new(0.5);
        let amplified = sample.apply_gain(2.0);
        assert_eq!(amplified.value(), 1.0); // Clamped
    }
}
