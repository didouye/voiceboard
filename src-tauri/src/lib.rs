/// Voiceboard library
///
/// This library contains all the core functionality for the Voiceboard application.

// Module declarations
pub mod audio;
pub mod soundboard;
pub mod devices;
pub mod config;
pub mod commands;
pub mod error;

// Re-export commonly used types
pub use error::{Result, VoiceboardError};
pub use audio::{AudioConfig, AudioEngine};
pub use soundboard::{Sound, SoundboardManager};
pub use devices::{AudioDevice, DeviceManager};
pub use config::{AppConfig, ConfigManager};
