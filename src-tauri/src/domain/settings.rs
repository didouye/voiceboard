//! Application settings and preferences

use serde::{Deserialize, Serialize};

fn default_global_hotkeys() -> bool {
    true
}

/// User preferences for audio devices
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AudioSettings {
    /// Selected input device ID (microphone)
    pub input_device_id: Option<String>,
    /// Selected output device ID (virtual microphone)
    pub output_device_id: Option<String>,
    /// Selected preview output device ID (for monitoring)
    pub preview_device_id: Option<String>,
    /// Master volume (0.0 to 1.0)
    pub master_volume: f32,
    /// Sample rate to use
    pub sample_rate: u32,
    /// Buffer size in frames
    pub buffer_size: u32,
    /// Enable mic monitoring on preview output
    #[serde(default)]
    pub mic_monitoring: bool,
    /// Enable global hotkeys for keyboard shortcuts
    #[serde(default = "default_global_hotkeys")]
    pub global_hotkeys_enabled: bool,
}

impl AudioSettings {
    pub fn new() -> Self {
        Self {
            input_device_id: None,
            output_device_id: None,
            preview_device_id: None,
            master_volume: 1.0,
            sample_rate: 48000,
            buffer_size: 1024,
            mic_monitoring: false,
            global_hotkeys_enabled: true,
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
        assert_eq!(
            settings.audio.master_volume,
            deserialized.audio.master_volume
        );
    }

    #[test]
    fn test_audio_settings_new() {
        let settings = AudioSettings::new();
        assert!(settings.input_device_id.is_none());
        assert!(settings.output_device_id.is_none());
        assert!(settings.preview_device_id.is_none());
        assert_eq!(settings.master_volume, 1.0);
        assert_eq!(settings.sample_rate, 48000);
        assert_eq!(settings.buffer_size, 1024);
        assert!(!settings.mic_monitoring);
        assert!(settings.global_hotkeys_enabled);
    }

    #[test]
    fn test_audio_settings_default() {
        let settings = AudioSettings::default();
        assert_eq!(settings.master_volume, 0.0); // Default trait gives 0.0
    }

    #[test]
    fn test_app_settings_new() {
        let settings = AppSettings::new();
        assert!(!settings.start_minimized);
        assert!(!settings.auto_start_mixing);
    }

    #[test]
    fn test_app_settings_clone() {
        let settings1 = AppSettings::new();
        let settings2 = settings1.clone();
        assert_eq!(settings1.start_minimized, settings2.start_minimized);
    }

    #[test]
    fn test_audio_settings_clone() {
        let settings1 = AudioSettings::new();
        let settings2 = settings1.clone();
        assert_eq!(settings1.sample_rate, settings2.sample_rate);
    }

    #[test]
    fn test_audio_settings_with_device_ids() {
        let mut settings = AudioSettings::new();
        settings.input_device_id = Some("mic-1".to_string());
        settings.output_device_id = Some("vb-cable".to_string());
        settings.preview_device_id = Some("speakers".to_string());

        assert_eq!(settings.input_device_id.as_deref(), Some("mic-1"));
        assert_eq!(settings.output_device_id.as_deref(), Some("vb-cable"));
        assert_eq!(settings.preview_device_id.as_deref(), Some("speakers"));
    }

    #[test]
    fn test_audio_settings_mic_monitoring() {
        let mut settings = AudioSettings::new();
        assert!(!settings.mic_monitoring);
        settings.mic_monitoring = true;
        assert!(settings.mic_monitoring);
    }

    #[test]
    fn test_audio_settings_global_hotkeys_enabled() {
        let settings = AudioSettings::new();
        // Default is true
        assert!(settings.global_hotkeys_enabled);
    }

    #[test]
    fn test_audio_settings_global_hotkeys_toggle() {
        let mut settings = AudioSettings::new();
        assert!(settings.global_hotkeys_enabled);
        settings.global_hotkeys_enabled = false;
        assert!(!settings.global_hotkeys_enabled);
    }

    #[test]
    fn test_audio_settings_global_hotkeys_serialization() {
        // Test that global_hotkeys_enabled is properly serialized
        let mut settings = AudioSettings::new();
        settings.global_hotkeys_enabled = false;

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("global_hotkeys_enabled"));

        let deserialized: AudioSettings = serde_json::from_str(&json).unwrap();
        assert!(!deserialized.global_hotkeys_enabled);
    }

    #[test]
    fn test_audio_settings_global_hotkeys_default_on_missing() {
        // Test that missing field in JSON defaults to true
        let json = r#"{"input_device_id":null,"output_device_id":null,"preview_device_id":null,"master_volume":1.0,"sample_rate":48000,"buffer_size":1024,"mic_monitoring":false}"#;
        let settings: AudioSettings = serde_json::from_str(json).unwrap();
        assert!(settings.global_hotkeys_enabled); // Should default to true
    }

    #[test]
    fn test_app_settings_start_minimized() {
        let mut settings = AppSettings::new();
        settings.start_minimized = true;
        assert!(settings.start_minimized);
    }

    #[test]
    fn test_app_settings_auto_start_mixing() {
        let mut settings = AppSettings::new();
        settings.auto_start_mixing = true;
        assert!(settings.auto_start_mixing);
    }

    #[test]
    fn test_app_settings_debug() {
        let settings = AppSettings::new();
        let debug = format!("{:?}", settings);
        assert!(debug.contains("AppSettings"));
    }

    #[test]
    fn test_audio_settings_debug() {
        let settings = AudioSettings::new();
        let debug = format!("{:?}", settings);
        assert!(debug.contains("AudioSettings"));
    }

    #[test]
    fn test_audio_settings_buffer_size() {
        let mut settings = AudioSettings::new();
        settings.buffer_size = 2048;
        assert_eq!(settings.buffer_size, 2048);
    }

    #[test]
    fn test_audio_settings_sample_rate() {
        let mut settings = AudioSettings::new();
        settings.sample_rate = 96000;
        assert_eq!(settings.sample_rate, 96000);
    }
}
