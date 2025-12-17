//! Application settings and preferences

use serde::{Deserialize, Serialize};

/// User preferences for audio devices
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AudioSettings {
    /// Selected input device ID (microphone)
    pub input_device_id: Option<String>,
    /// Selected output device ID (virtual microphone)
    pub output_device_id: Option<String>,
    /// Master volume (0.0 to 1.0)
    pub master_volume: f32,
    /// Sample rate to use
    pub sample_rate: u32,
    /// Buffer size in frames
    pub buffer_size: u32,
}

impl AudioSettings {
    pub fn new() -> Self {
        Self {
            input_device_id: None,
            output_device_id: None,
            master_volume: 1.0,
            sample_rate: 48000,
            buffer_size: 1024,
        }
    }
}

/// Application-wide settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Audio-related settings
    pub audio: AudioSettings,
    /// Whether to start minimized to system tray
    pub start_minimized: bool,
    /// Auto-start mixing when app launches
    pub auto_start_mixing: bool,
}

impl AppSettings {
    pub fn new() -> Self {
        Self {
            audio: AudioSettings::new(),
            start_minimized: false,
            auto_start_mixing: false,
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();
        assert_eq!(settings.audio.master_volume, 1.0);
        assert_eq!(settings.audio.sample_rate, 48000);
        assert!(settings.audio.input_device_id.is_none());
    }

    #[test]
    fn test_settings_serialization() {
        let settings = AppSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings.audio.master_volume, deserialized.audio.master_volume);
    }
}
