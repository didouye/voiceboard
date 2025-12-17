//! Application state management

use crate::domain::{AppSettings, MixerConfig};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Global application state managed by Tauri
pub struct AppState {
    pub mixer_config: Arc<RwLock<MixerConfig>>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub is_mixing: Arc<RwLock<bool>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mixer_config: Arc::new(RwLock::new(MixerConfig::default())),
            settings: Arc::new(RwLock::new(AppSettings::default())),
            is_mixing: Arc::new(RwLock::new(false)),
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
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
