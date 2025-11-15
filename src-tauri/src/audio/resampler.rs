/// Audio resampling using rubato
use super::types::{AudioBuffer, AudioFormat, Sample};
use crate::error::{Result, VoiceboardError};
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use tracing::debug;

/// Audio resampler for converting between sample rates
pub struct AudioResampler {
    /// Source sample rate
    source_rate: u32,
    /// Target sample rate
    target_rate: u32,
    /// Number of channels
    channels: u16,
}

impl AudioResampler {
    /// Create a new audio resampler
    pub fn new(source_rate: u32, target_rate: u32, channels: u16) -> Self {
        Self {
            source_rate,
            target_rate,
            channels,
        }
    }

    /// Resample an audio buffer to a target sample rate
    pub fn resample(&self, input: &AudioBuffer) -> Result<AudioBuffer> {
        // If sample rates match, no resampling needed
        if self.source_rate == self.target_rate {
            return Ok(input.clone());
        }

        debug!(
            "Resampling audio: {}Hz -> {}Hz, {} channels",
            self.source_rate, self.target_rate, self.channels
        );

        let resample_ratio = self.target_rate as f64 / self.source_rate as f64;

        // Convert interleaved samples to separate channels
        let input_frames = input.frames();
        let mut channels_data: Vec<Vec<Sample>> = vec![Vec::new(); self.channels as usize];

        for frame_idx in 0..input_frames {
            for ch in 0..self.channels as usize {
                let sample_idx = frame_idx * self.channels as usize + ch;
                channels_data[ch].push(input.data[sample_idx]);
            }
        }

        // Create resampler
        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };

        let mut resampler = SincFixedIn::<Sample>::new(
            resample_ratio,
            2.0,
            params,
            input_frames,
            self.channels as usize,
        )
        .map_err(|e| VoiceboardError::Resampling(format!("Failed to create resampler: {}", e)))?;

        // Resample each channel
        let output_channels = resampler
            .process(&channels_data, None)
            .map_err(|e| VoiceboardError::Resampling(format!("Resampling failed: {}", e)))?;

        // Convert back to interleaved format
        let output_frames = output_channels[0].len();
        let mut output_data = Vec::with_capacity(output_frames * self.channels as usize);

        for frame_idx in 0..output_frames {
            for ch in 0..self.channels as usize {
                output_data.push(output_channels[ch][frame_idx]);
            }
        }

        let output_format = AudioFormat::new(self.target_rate, self.channels);
        let mut output_buffer = AudioBuffer::new(output_format);
        output_buffer.data = output_data;

        debug!(
            "Resampled {} frames to {} frames",
            input_frames, output_frames
        );

        Ok(output_buffer)
    }

    /// Resample audio buffer in-place to a target format
    pub fn resample_to_format(input: &AudioBuffer, target_format: AudioFormat) -> Result<AudioBuffer> {
        // Check if formats match
        if input.format == target_format {
            return Ok(input.clone());
        }

        // Handle channel conversion if needed
        if input.format.channels != target_format.channels {
            return Err(VoiceboardError::AudioFormat(
                "Channel conversion not yet implemented".to_string(),
            ));
        }

        // Resample to target sample rate
        let resampler = AudioResampler::new(
            input.format.sample_rate,
            target_format.sample_rate,
            input.format.channels,
        );

        resampler.resample(input)
    }
}

/// Helper function to convert mono to stereo
pub fn mono_to_stereo(input: &AudioBuffer) -> Result<AudioBuffer> {
    if input.format.channels != 1 {
        return Err(VoiceboardError::AudioFormat(
            "Input is not mono".to_string(),
        ));
    }

    let mut output = AudioBuffer::new(AudioFormat::new(input.format.sample_rate, 2));
    output.data.reserve(input.data.len() * 2);

    // Duplicate each mono sample to both channels
    for &sample in &input.data {
        output.data.push(sample);
        output.data.push(sample);
    }

    Ok(output)
}

/// Helper function to convert stereo to mono
pub fn stereo_to_mono(input: &AudioBuffer) -> Result<AudioBuffer> {
    if input.format.channels != 2 {
        return Err(VoiceboardError::AudioFormat(
            "Input is not stereo".to_string(),
        ));
    }

    let mut output = AudioBuffer::new(AudioFormat::new(input.format.sample_rate, 1));
    output.data.reserve(input.data.len() / 2);

    // Average the two channels
    for chunk in input.data.chunks_exact(2) {
        let mono_sample = (chunk[0] + chunk[1]) / 2.0;
        output.data.push(mono_sample);
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mono_to_stereo() {
        let format = AudioFormat::new(48000, 1);
        let mut input = AudioBuffer::new(format);
        input.data = vec![0.5, 0.25, 0.75];

        let output = mono_to_stereo(&input).unwrap();

        assert_eq!(output.format.channels, 2);
        assert_eq!(output.data.len(), 6);
        assert_eq!(output.data, vec![0.5, 0.5, 0.25, 0.25, 0.75, 0.75]);
    }

    #[test]
    fn test_stereo_to_mono() {
        let format = AudioFormat::new(48000, 2);
        let mut input = AudioBuffer::new(format);
        input.data = vec![0.5, 0.5, 0.25, 0.75];

        let output = stereo_to_mono(&input).unwrap();

        assert_eq!(output.format.channels, 1);
        assert_eq!(output.data.len(), 2);
        assert_eq!(output.data, vec![0.5, 0.5]);
    }
}
