/// Audio processing module
///
/// This module contains all audio-related functionality including:
/// - Audio device management
/// - Microphone capture
/// - Sound file playback
/// - Audio mixing
/// - Virtual microphone output
/// - Audio decoding and resampling

pub mod types;
pub mod engine;
pub mod capture;
pub mod playback;
pub mod mixer;
pub mod decoder;
pub mod resampler;
pub mod virtual_device;

// Re-export commonly used types
pub use types::{
    AudioBuffer, AudioConfig, AudioFormat, AudioLevel, Channels, Sample, SampleRate, Volume,
};
pub use engine::AudioEngine;
