/// Configuration management module
pub mod manager;
pub mod types;

// Re-export
pub use manager::ConfigManager;
pub use types::AppConfig;
