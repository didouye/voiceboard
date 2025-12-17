//! Application state management

use crate::domain::MixerConfig;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Global application state managed by Tauri
pub struct AppState {
    pub mixer_config: Arc<RwLock<MixerConfig>>,
    pub is_mixing: Arc<RwLock<bool>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mixer_config: Arc::new(RwLock::new(MixerConfig::default())),
            is_mixing: Arc::new(RwLock::new(false)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
