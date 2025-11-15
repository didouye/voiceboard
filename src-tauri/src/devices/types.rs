/// Device-related type definitions
use serde::{Deserialize, Serialize};

/// Audio device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    /// Unique device identifier
    pub id: String,
    /// Device name
    pub name: String,
    /// Device type
    pub device_type: DeviceType,
    /// Whether this is the default device
    pub is_default: bool,
}

/// Audio device type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    /// Input device (microphone)
    Input,
    /// Output device (speaker/virtual cable)
    Output,
}
