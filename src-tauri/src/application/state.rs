//! Application state management

use crate::application::audio_engine::AudioEngine;
use crate::application::preview_engine::PreviewEngine;
use crate::domain::{AppSettings, MixerConfig};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Global application state managed by Tauri
pub struct AppState {
    pub mixer_config: Arc<RwLock<MixerConfig>>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub is_mixing: Arc<RwLock<bool>>,
    pub audio_engine: Arc<Mutex<AudioEngine>>,
    pub preview_engine: Arc<Mutex<Option<PreviewEngine>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mixer_config: Arc::new(RwLock::new(MixerConfig::default())),
            settings: Arc::new(RwLock::new(AppSettings::default())),
            is_mixing: Arc::new(RwLock::new(false)),
            audio_engine: Arc::new(Mutex::new(AudioEngine::new())),
            preview_engine: Arc::new(Mutex::new(None)),
        }
    }

    /// Create state with pre-loaded settings
    pub fn with_settings(settings: AppSettings) -> Self {
        let mixer_config = MixerConfig {
            master_volume: settings.audio.master_volume,
            ..Default::default()
        };

        Self {
            mixer_config: Arc::new(RwLock::new(mixer_config)),
            settings: Arc::new(RwLock::new(settings)),
            is_mixing: Arc::new(RwLock::new(false)),
            audio_engine: Arc::new(Mutex::new(AudioEngine::new())),
            preview_engine: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::AudioSettings;

    #[tokio::test]
    async fn test_app_state_new() {
        let state = AppState::new();

        // Verify initial state
        assert!(!*state.is_mixing.read().await);
        assert!(state.preview_engine.lock().await.is_none());
    }

    #[tokio::test]
    async fn test_app_state_default() {
        let state = AppState::default();

        assert!(!*state.is_mixing.read().await);
    }

    #[tokio::test]
    async fn test_app_state_with_settings() {
        let settings = AppSettings {
            audio: AudioSettings {
                master_volume: 0.75,
                ..Default::default()
            },
            start_minimized: true,
            auto_start_mixing: false,
        };

        let state = AppState::with_settings(settings);

        // Verify settings were applied
        let config = state.mixer_config.read().await;
        assert_eq!(config.master_volume, 0.75);

        let loaded_settings = state.settings.read().await;
        assert!(loaded_settings.start_minimized);
    }

    #[tokio::test]
    async fn test_app_state_mixer_config_default() {
        let state = AppState::new();
        let config = state.mixer_config.read().await;

        assert_eq!(config.master_volume, 1.0);
        assert!(config.channels.is_empty());
    }

    #[tokio::test]
    async fn test_app_state_settings_default() {
        let state = AppState::new();
        let settings = state.settings.read().await;

        assert!(!settings.start_minimized);
        assert!(!settings.auto_start_mixing);
    }

    #[tokio::test]
    async fn test_app_state_audio_engine_not_running() {
        let state = AppState::new();
        let engine = state.audio_engine.lock().await;

        assert!(!engine.is_running());
    }
}
