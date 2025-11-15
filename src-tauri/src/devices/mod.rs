/// Audio device management module
pub mod manager;
pub mod types;

// Re-export
pub use manager::DeviceManager;
pub use types::{AudioDevice, DeviceType};
