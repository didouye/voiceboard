/// Configuration type definitions
use crate::audio::AudioConfig;
use serde::{Deserialize, Serialize};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Selected input device ID
    pub selected_input_device: Option<String>,
    /// Selected output device ID (virtual cable)
    pub selected_output_device: Option<String>,
    /// Audio engine configuration
    pub audio: AudioConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            selected_input_device: None,
            selected_output_device: None,
            audio: AudioConfig::default(),
        }
    }
}
