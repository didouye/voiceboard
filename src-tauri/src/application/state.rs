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
        let mut mixer_config = MixerConfig::default();
        mixer_config.master_volume = settings.audio.master_volume;

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
