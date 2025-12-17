//! Device manager port - Interface for managing audio devices

use crate::domain::{AudioDevice, DeviceId, DeviceType};

/// Errors that can occur during device management
#[derive(Debug, thiserror::Error)]
pub enum DeviceManagerError {
    #[error("Failed to enumerate devices: {0}")]
    EnumerationError(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("System error: {0}")]
    SystemError(String),
}

/// Port for managing audio devices
///
/// This trait defines the contract for discovering and managing
/// audio devices in the system.
#[cfg_attr(test, mockall::automock)]
pub trait DeviceManager: Send + Sync {
    /// Get all available audio devices
    fn list_devices(&self) -> Result<Vec<AudioDevice>, DeviceManagerError>;

    /// Get devices filtered by type
    fn list_devices_by_type(
        &self,
        device_type: DeviceType,
    ) -> Result<Vec<AudioDevice>, DeviceManagerError>;

    /// Get the default input device
    fn default_input_device(&self) -> Result<Option<AudioDevice>, DeviceManagerError>;

    /// Get the default output device
    fn default_output_device(&self) -> Result<Option<AudioDevice>, DeviceManagerError>;

    /// Get a specific device by ID
    fn get_device(&self, id: &DeviceId) -> Result<Option<AudioDevice>, DeviceManagerError>;

    /// Refresh the device list (re-enumerate)
    fn refresh(&mut self) -> Result<(), DeviceManagerError>;
}

/// Callback for device change notifications
pub trait DeviceChangeCallback: Send + Sync {
    /// Called when a device is added
    fn on_device_added(&mut self, device: &AudioDevice);

    /// Called when a device is removed
    fn on_device_removed(&mut self, device_id: &DeviceId);

    /// Called when the default device changes
    fn on_default_device_changed(&mut self, device_type: DeviceType, device: &AudioDevice);
}

/// Port for monitoring device changes
#[cfg_attr(test, mockall::automock)]
pub trait DeviceMonitor: Send + Sync {
    /// Start monitoring for device changes
    fn start_monitoring(&mut self) -> Result<(), DeviceManagerError>;

    /// Stop monitoring
    fn stop_monitoring(&mut self) -> Result<(), DeviceManagerError>;

    /// Check if monitoring is active
    fn is_monitoring(&self) -> bool;

    /// Register a callback for device changes
    fn register_callback(&mut self, callback: Box<dyn DeviceChangeCallback>);
}
