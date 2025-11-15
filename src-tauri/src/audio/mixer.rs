/// Audio mixer for combining multiple audio sources
use super::types::{AudioBuffer, AudioFormat, Sample, Volume};
use crate::error::{Result, VoiceboardError};
use std::collections::HashMap;
use tracing::debug;

/// Audio mixer that combines multiple audio sources
pub struct AudioMixer {
    /// Output format
    format: AudioFormat,
    /// Active sound sources
    sources: HashMap<String, MixerSource>,
    /// Microphone input buffer
    mic_buffer: Vec<Sample>,
    /// Master volume
    master_volume: Volume,
}

impl AudioMixer {
    /// Create a new audio mixer
    pub fn new(format: AudioFormat) -> Self {
        Self {
            format,
            sources: HashMap::new(),
            mic_buffer: Vec::new(),
            master_volume: Volume::max(),
        }
    }

    /// Add a sound source to the mixer
    pub fn add_source(&mut self, id: String, buffer: AudioBuffer, volume: Volume) -> Result<()> {
        // TODO: Resample if format doesn't match
        if buffer.format != self.format {
            return Err(VoiceboardError::AudioFormat(format!(
                "Source format {:?} doesn't match mixer format {:?}",
                buffer.format, self.format
            )));
        }

        debug!("Adding sound source: {}", id);
        self.sources.insert(
            id,
            MixerSource {
                buffer,
                position: 0,
                volume,
            },
        );

        Ok(())
    }

    /// Remove a sound source from the mixer
    pub fn remove_source(&mut self, id: &str) {
        debug!("Removing sound source: {}", id);
        self.sources.remove(id);
    }

    /// Set microphone input
    pub fn set_mic_input(&mut self, samples: &[Sample]) {
        self.mic_buffer.clear();
        self.mic_buffer.extend_from_slice(samples);
    }

    /// Mix all sources and produce output
    pub fn mix(&mut self, output: &mut [Sample], mic_volume: Volume, effects_volume: Volume) {
        // Clear output buffer
        for sample in output.iter_mut() {
            *sample = 0.0;
        }

        // Add microphone input
        if !self.mic_buffer.is_empty() {
            let len = output.len().min(self.mic_buffer.len());
            for i in 0..len {
                output[i] += self.mic_buffer[i] * mic_volume.linear();
            }
        }

        // Mix in all sound sources
        let mut finished_sources = Vec::new();

        for (id, source) in self.sources.iter_mut() {
            let remaining = source.buffer.data.len() - source.position;
            if remaining == 0 {
                finished_sources.push(id.clone());
                continue;
            }

            let to_mix = remaining.min(output.len());
            let volume_factor = source.volume.linear() * effects_volume.linear();

            for i in 0..to_mix {
                output[i] += source.buffer.data[source.position + i] * volume_factor;
            }

            source.position += to_mix;
        }

        // Remove finished sources
        for id in finished_sources {
            self.remove_source(&id);
        }

        // Apply master volume
        self.master_volume.apply(output);

        // Clip output to prevent distortion
        for sample in output.iter_mut() {
            *sample = sample.clamp(-1.0, 1.0);
        }
    }

    /// Set master volume
    pub fn set_master_volume(&mut self, volume: Volume) {
        self.master_volume = volume;
    }

    /// Get number of active sources
    pub fn active_sources(&self) -> usize {
        self.sources.len()
    }

    /// Clear all sources
    pub fn clear(&mut self) {
        self.sources.clear();
    }
}

/// A single audio source in the mixer
struct MixerSource {
    /// Audio buffer
    buffer: AudioBuffer,
    /// Current playback position (in samples)
    position: usize,
    /// Volume for this source
    volume: Volume,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mixer_basic() {
        let format = AudioFormat::standard();
        let mut mixer = AudioMixer::new(format);

        // Create a simple sine wave buffer
        let mut buffer = AudioBuffer::zeros(format, 480);
        for i in 0..buffer.data.len() {
            buffer.data[i] = (i as f32 * 0.1).sin() * 0.5;
        }

        mixer
            .add_source("test".to_string(), buffer, Volume::max())
            .unwrap();

        let mut output = vec![0.0f32; 480 * 2];
        mixer.mix(
            &mut output,
            Volume::default_volume(),
            Volume::default_volume(),
        );

        // Output should not be all zeros
        assert!(output.iter().any(|&s| s != 0.0));
    }

    #[test]
    fn test_mixer_multiple_sources() {
        let format = AudioFormat::standard();
        let mut mixer = AudioMixer::new(format);

        let buffer1 = AudioBuffer::zeros(format, 480);
        let buffer2 = AudioBuffer::zeros(format, 480);

        mixer
            .add_source("sound1".to_string(), buffer1, Volume::max())
            .unwrap();
        mixer
            .add_source("sound2".to_string(), buffer2, Volume::max())
            .unwrap();

        assert_eq!(mixer.active_sources(), 2);

        mixer.remove_source("sound1");
        assert_eq!(mixer.active_sources(), 1);
    }
}
