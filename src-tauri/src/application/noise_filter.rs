//! Noise suppression filter using nnnoiseless (RNNoise port)
//!
//! Processes audio in 480-sample frames at 48kHz.
//!
//! IMPORTANT: nnnoiseless expects samples in 16-bit integer scale [-32768, 32767],
//! not normalized [-1.0, 1.0]. We scale internally before/after processing.

use nnnoiseless::DenoiseState;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Frame size required by nnnoiseless (10ms at 48kHz)
pub const DENOISE_FRAME_SIZE: usize = 480;

/// Scale factor to convert normalized audio [-1.0, 1.0] to 16-bit scale [-32768, 32767]
const SCALE_TO_16BIT: f32 = 32768.0;

/// Real-time noise suppression filter
pub struct NoiseFilter {
    /// The denoiser state (must persist between calls)
    denoiser: Box<DenoiseState<'static>>,
    /// Buffer to accumulate samples until we have a full frame
    buffer: Vec<f32>,
    /// Output buffer for processed samples
    output_buffer: Vec<f32>,
    /// Whether noise suppression is enabled
    enabled: Arc<AtomicBool>,
}

impl NoiseFilter {
    /// Create a new noise filter
    pub fn new(enabled: Arc<AtomicBool>) -> Self {
        Self {
            denoiser: DenoiseState::new(),
            buffer: Vec::with_capacity(DENOISE_FRAME_SIZE),
            output_buffer: Vec::with_capacity(DENOISE_FRAME_SIZE),
            enabled,
        }
    }

    /// Process a single sample, returning processed samples when a full frame is ready
    ///
    /// Call this for each input sample. When enough samples have accumulated (480),
    /// the filter processes them and returns the denoised samples.
    /// Returns an empty slice if not enough samples yet.
    ///
    /// Input/output are in normalized range [-1.0, 1.0]. Scaling to 16-bit is done internally.
    pub fn process_sample(&mut self, sample: f32) -> &[f32] {
        // Clear output buffer
        self.output_buffer.clear();

        // If disabled, pass through immediately
        if !self.enabled.load(Ordering::Relaxed) {
            self.output_buffer.push(sample);
            return &self.output_buffer;
        }

        // Accumulate sample (scaled to 16-bit range for nnnoiseless)
        self.buffer.push(sample * SCALE_TO_16BIT);

        // Process when we have a full frame
        if self.buffer.len() >= DENOISE_FRAME_SIZE {
            // Resize output buffer to match frame size
            self.output_buffer.resize(DENOISE_FRAME_SIZE, 0.0);

            // Process the frame (input -> output) - both in 16-bit scale
            self.denoiser
                .process_frame(&mut self.output_buffer, &self.buffer);

            // Scale output back to normalized range [-1.0, 1.0]
            for sample in &mut self.output_buffer {
                *sample /= SCALE_TO_16BIT;
            }

            // Clear input buffer
            self.buffer.clear();
        }

        &self.output_buffer
    }

    /// Flush any remaining samples in the buffer (for shutdown)
    pub fn flush(&mut self) -> Vec<f32> {
        if self.buffer.is_empty() {
            return Vec::new();
        }

        // Pad with zeros to complete the frame (buffer already has scaled samples)
        while self.buffer.len() < DENOISE_FRAME_SIZE {
            self.buffer.push(0.0);
        }

        let mut output = vec![0.0; DENOISE_FRAME_SIZE];
        self.denoiser.process_frame(&mut output, &self.buffer);
        self.buffer.clear();

        // Scale output back to normalized range
        for sample in &mut output {
            *sample /= SCALE_TO_16BIT;
        }
        output
    }

    /// Check if filter is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Set enabled state
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_filter_creation() {
        let enabled = Arc::new(AtomicBool::new(true));
        let filter = NoiseFilter::new(enabled);
        assert!(filter.is_enabled());
    }

    #[test]
    fn test_noise_filter_disabled_passthrough() {
        let enabled = Arc::new(AtomicBool::new(false));
        let mut filter = NoiseFilter::new(enabled);

        // When disabled, samples should pass through immediately
        let output = filter.process_sample(0.5);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0], 0.5);
    }

    #[test]
    fn test_noise_filter_enabled_buffering() {
        let enabled = Arc::new(AtomicBool::new(true));
        let mut filter = NoiseFilter::new(enabled);

        // First 479 samples should buffer (return empty)
        for i in 0..479 {
            let output = filter.process_sample(0.1);
            assert!(
                output.is_empty(),
                "Sample {} should buffer, got {} samples",
                i,
                output.len()
            );
        }

        // 480th sample should trigger processing
        let output = filter.process_sample(0.1);
        assert_eq!(output.len(), DENOISE_FRAME_SIZE);
    }

    #[test]
    fn test_noise_filter_toggle() {
        let enabled = Arc::new(AtomicBool::new(true));
        let filter = NoiseFilter::new(enabled);

        assert!(filter.is_enabled());
        filter.set_enabled(false);
        assert!(!filter.is_enabled());
        filter.set_enabled(true);
        assert!(filter.is_enabled());
    }

    #[test]
    fn test_noise_filter_reduces_noise() {
        let enabled = Arc::new(AtomicBool::new(true));
        let mut filter = NoiseFilter::new(enabled);

        // Generate white noise in normalized range [-1.0, 1.0]
        // (random-ish values simulating background noise)
        let noise: Vec<f32> = (0..DENOISE_FRAME_SIZE)
            .map(|i| ((i * 7919) % 1000) as f32 / 1000.0 - 0.5)
            .collect();

        // Calculate input RMS
        let input_rms: f32 = (noise.iter().map(|s| s * s).sum::<f32>() / noise.len() as f32).sqrt();

        // Process the noise (filter handles scaling internally)
        let mut output: Vec<f32> = Vec::new();
        for sample in noise {
            output.extend(filter.process_sample(sample));
        }

        // Calculate output RMS
        let output_rms: f32 =
            (output.iter().map(|s| s * s).sum::<f32>() / output.len() as f32).sqrt();

        // Output should have lower RMS than input (noise reduced)
        // Note: With proper scaling, RNNoise should significantly reduce non-voice noise
        assert!(
            output_rms < input_rms,
            "Noise should be reduced: input_rms={}, output_rms={}",
            input_rms,
            output_rms
        );
    }

    #[test]
    fn test_noise_filter_flush() {
        let enabled = Arc::new(AtomicBool::new(true));
        let mut filter = NoiseFilter::new(enabled);

        // Add some samples (less than a full frame)
        for _ in 0..100 {
            filter.process_sample(0.1);
        }

        // Flush should return padded frame
        let flushed = filter.flush();
        assert_eq!(flushed.len(), DENOISE_FRAME_SIZE);
    }
}
