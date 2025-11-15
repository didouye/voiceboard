/// Type definitions for audio processing
use serde::{Deserialize, Serialize};

/// Audio sample rate in Hz
pub type SampleRate = u32;

/// Audio sample value (32-bit float)
pub type Sample = f32;

/// Number of audio channels
pub type Channels = u16;

/// Audio format specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioFormat {
    /// Sample rate in Hz
    pub sample_rate: SampleRate,
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: Channels,
}

impl AudioFormat {
    /// Create a new audio format
    pub fn new(sample_rate: SampleRate, channels: Channels) -> Self {
        Self {
            sample_rate,
            channels,
        }
    }

    /// Standard format for WASAPI (48kHz stereo)
    pub fn standard() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
        }
    }

    /// Convert duration in seconds to number of frames
    pub fn seconds_to_frames(&self, seconds: f32) -> usize {
        (seconds * self.sample_rate as f32) as usize
    }

    /// Convert number of frames to duration in seconds
    pub fn frames_to_seconds(&self, frames: usize) -> f32 {
        frames as f32 / self.sample_rate as f32
    }

    /// Calculate buffer size in samples for both channels
    pub fn buffer_size(&self, frames: usize) -> usize {
        frames * self.channels as usize
    }
}

impl Default for AudioFormat {
    fn default() -> Self {
        Self::standard()
    }
}

/// Audio buffer containing PCM samples
#[derive(Debug, Clone)]
pub struct AudioBuffer {
    /// Sample data (interleaved for multi-channel)
    pub data: Vec<Sample>,
    /// Audio format
    pub format: AudioFormat,
}

impl AudioBuffer {
    /// Create a new empty audio buffer
    pub fn new(format: AudioFormat) -> Self {
        Self {
            data: Vec::new(),
            format,
        }
    }

    /// Create a new audio buffer with specified capacity
    pub fn with_capacity(format: AudioFormat, frames: usize) -> Self {
        let capacity = format.buffer_size(frames);
        Self {
            data: Vec::with_capacity(capacity),
            format,
        }
    }

    /// Create a new audio buffer filled with zeros
    pub fn zeros(format: AudioFormat, frames: usize) -> Self {
        let size = format.buffer_size(frames);
        Self {
            data: vec![0.0; size],
            format,
        }
    }

    /// Get the number of frames in this buffer
    pub fn frames(&self) -> usize {
        self.data.len() / self.format.channels as usize
    }

    /// Get the duration in seconds
    pub fn duration(&self) -> f32 {
        self.format.frames_to_seconds(self.frames())
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Resize the buffer to hold a specific number of frames
    pub fn resize(&mut self, frames: usize) {
        let size = self.format.buffer_size(frames);
        self.data.resize(size, 0.0);
    }

    /// Append samples from another buffer
    pub fn append(&mut self, other: &AudioBuffer) {
        // TODO: Handle format conversion if formats don't match
        assert_eq!(self.format, other.format, "Audio formats must match");
        self.data.extend_from_slice(&other.data);
    }
}

/// Audio level information for visualization
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AudioLevel {
    /// RMS (Root Mean Square) level in dB
    pub rms_db: f32,
    /// Peak level in dB
    pub peak_db: f32,
    /// RMS level as linear value (0.0 to 1.0)
    pub rms_linear: f32,
    /// Peak level as linear value (0.0 to 1.0)
    pub peak_linear: f32,
}

impl AudioLevel {
    /// Calculate audio level from a buffer
    pub fn from_buffer(buffer: &[Sample]) -> Self {
        if buffer.is_empty() {
            return Self::silence();
        }

        // Calculate RMS (Root Mean Square)
        let sum_squares: f32 = buffer.iter().map(|&s| s * s).sum();
        let rms = (sum_squares / buffer.len() as f32).sqrt();

        // Calculate peak
        let peak = buffer.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);

        // Convert to dB (with floor at -60dB)
        let rms_db = if rms > 0.0 {
            20.0 * rms.log10().max(-60.0)
        } else {
            -60.0
        };

        let peak_db = if peak > 0.0 {
            20.0 * peak.log10().max(-60.0)
        } else {
            -60.0
        };

        Self {
            rms_db,
            peak_db,
            rms_linear: rms,
            peak_linear: peak,
        }
    }

    /// Create a silent audio level
    pub fn silence() -> Self {
        Self {
            rms_db: -60.0,
            peak_db: -60.0,
            rms_linear: 0.0,
            peak_linear: 0.0,
        }
    }
}

/// Volume control with linear and decibel representations
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Volume {
    /// Linear volume (0.0 to 1.0)
    linear: f32,
}

impl Volume {
    /// Create a new volume from linear value (0.0 to 1.0)
    pub fn from_linear(value: f32) -> Self {
        Self {
            linear: value.clamp(0.0, 1.0),
        }
    }

    /// Create a new volume from decibel value
    pub fn from_db(db: f32) -> Self {
        let linear = 10.0f32.powf(db / 20.0);
        Self::from_linear(linear)
    }

    /// Get linear volume (0.0 to 1.0)
    pub fn linear(&self) -> f32 {
        self.linear
    }

    /// Get volume in decibels
    pub fn db(&self) -> f32 {
        if self.linear > 0.0 {
            20.0 * self.linear.log10()
        } else {
            -60.0
        }
    }

    /// Apply this volume to a buffer
    pub fn apply(&self, buffer: &mut [Sample]) {
        if self.linear == 1.0 {
            return; // No change needed
        }

        for sample in buffer.iter_mut() {
            *sample *= self.linear;
        }
    }

    /// Muted volume
    pub fn mute() -> Self {
        Self { linear: 0.0 }
    }

    /// Maximum volume
    pub fn max() -> Self {
        Self { linear: 1.0 }
    }

    /// Default volume (50%)
    pub fn default_volume() -> Self {
        Self { linear: 0.5 }
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self::default_volume()
    }
}

/// Configuration for audio engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Target sample rate
    pub sample_rate: SampleRate,
    /// Number of channels
    pub channels: Channels,
    /// Buffer size in frames (affects latency)
    pub buffer_size: usize,
    /// Master volume
    pub master_volume: Volume,
    /// Microphone volume
    pub mic_volume: Volume,
    /// Effects volume
    pub effects_volume: Volume,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            buffer_size: 480, // 10ms at 48kHz
            master_volume: Volume::max(),
            mic_volume: Volume::default_volume(),
            effects_volume: Volume::default_volume(),
        }
    }
}

impl AudioConfig {
    /// Get the audio format from this config
    pub fn format(&self) -> AudioFormat {
        AudioFormat::new(self.sample_rate, self.channels)
    }

    /// Get buffer duration in milliseconds
    pub fn buffer_duration_ms(&self) -> f32 {
        (self.buffer_size as f32 / self.sample_rate as f32) * 1000.0
    }
}
