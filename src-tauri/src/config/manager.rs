/// Configuration manager
use super::types::AppConfig;
use crate::error::Result;
use parking_lot::RwLock;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};

/// Manages application configuration
pub struct ConfigManager {
    /// Configuration file path
    config_path: std::path::PathBuf,
    /// In-memory configuration
    config: Arc<RwLock<AppConfig>>,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        let config_path = config_path.as_ref().to_path_buf();

        let manager = Self {
            config_path,
            config: Arc::new(RwLock::new(AppConfig::default())),
        };

        // Load configuration if it exists
        if manager.config_path.exists() {
            manager.load()?;
        } else {
            // Save default configuration
            manager.save()?;
        }

        Ok(manager)
    }

    /// Load configuration from file
    pub fn load(&self) -> Result<()> {
        debug!("Loading configuration from {:?}", self.config_path);

        let content = std::fs::read_to_string(&self.config_path)?;
        let config: AppConfig = serde_json::from_str(&content)?;

        *self.config.write() = config;

        info!("Configuration loaded successfully");
        Ok(())
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        debug!("Saving configuration to {:?}", self.config_path);

        // Create parent directory if needed
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let config = self.config.read().clone();
        let content = serde_json::to_string_pretty(&config)?;

        std::fs::write(&self.config_path, content)?;

        info!("Configuration saved successfully");
        Ok(())
    }

    /// Get the current configuration
    pub fn get(&self) -> AppConfig {
        self.config.read().clone()
    }

    /// Update the configuration
    pub fn update<F>(&self, update_fn: F) -> Result<()>
    where
        F: FnOnce(&mut AppConfig),
    {
        {
            let mut config = self.config.write();
            update_fn(&mut config);
        }

        self.save()
    }
}
