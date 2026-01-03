//! Audio device entity

use serde::{Deserialize, Serialize};

/// Unique identifier for an audio device
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(String);

impl DeviceId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for DeviceId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for DeviceId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Type of audio device
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    /// Physical input device (microphone)
    InputPhysical,
    /// Physical output device (speakers/headphones)
    OutputPhysical,
    /// Virtual input device (virtual microphone)
    InputVirtual,
    /// Virtual output device (virtual speaker)
    OutputVirtual,
}

impl DeviceType {
    pub fn is_input(&self) -> bool {
        matches!(self, DeviceType::InputPhysical | DeviceType::InputVirtual)
    }

    pub fn is_output(&self) -> bool {
        matches!(self, DeviceType::OutputPhysical | DeviceType::OutputVirtual)
    }

    pub fn is_virtual(&self) -> bool {
        matches!(self, DeviceType::InputVirtual | DeviceType::OutputVirtual)
    }
}

/// Represents an audio device in the system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioDevice {
    id: DeviceId,
    name: String,
    device_type: DeviceType,
    is_default: bool,
    sample_rates: Vec<u32>,
    channels: Vec<u16>,
}

impl AudioDevice {
    pub fn new(
        id: DeviceId,
        name: String,
        device_type: DeviceType,
        is_default: bool,
        sample_rates: Vec<u32>,
        channels: Vec<u16>,
    ) -> Self {
        Self {
            id,
            name,
            device_type,
            is_default,
            sample_rates,
            channels,
        }
    }

    pub fn id(&self) -> &DeviceId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn device_type(&self) -> DeviceType {
        self.device_type
    }

    pub fn is_default(&self) -> bool {
        self.is_default
    }

    pub fn sample_rates(&self) -> &[u32] {
        &self.sample_rates
    }

    pub fn channels(&self) -> &[u16] {
        &self.channels
    }

    pub fn supports_sample_rate(&self, rate: u32) -> bool {
        self.sample_rates.contains(&rate)
    }

    pub fn supports_channels(&self, ch: u16) -> bool {
        self.channels.contains(&ch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_type_queries() {
        assert!(DeviceType::InputPhysical.is_input());
        assert!(DeviceType::InputVirtual.is_input());
        assert!(!DeviceType::OutputPhysical.is_input());

        assert!(DeviceType::InputVirtual.is_virtual());
        assert!(!DeviceType::InputPhysical.is_virtual());
    }

    #[test]
    fn test_device_creation() {
        let device = AudioDevice::new(
            DeviceId::new("test-device"),
            "Test Microphone".to_string(),
            DeviceType::InputPhysical,
            true,
            vec![44100, 48000],
            vec![1, 2],
        );

        assert_eq!(device.name(), "Test Microphone");
        assert!(device.is_default());
        assert!(device.supports_sample_rate(44100));
        assert!(!device.supports_sample_rate(96000));
    }

    #[test]
    fn test_device_id_as_str() {
        let id = DeviceId::new("my-device-id");
        assert_eq!(id.as_str(), "my-device-id");
    }

    #[test]
    fn test_device_id_from_string() {
        let id: DeviceId = "test-id".to_string().into();
        assert_eq!(id.as_str(), "test-id");
    }

    #[test]
    fn test_device_id_from_str() {
        let id: DeviceId = "test-id".into();
        assert_eq!(id.as_str(), "test-id");
    }

    #[test]
    fn test_device_id_clone() {
        let id1 = DeviceId::new("my-id");
        let id2 = id1.clone();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_device_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(DeviceId::new("id1"));
        set.insert(DeviceId::new("id2"));
        assert_eq!(set.len(), 2);
        assert!(set.contains(&DeviceId::new("id1")));
    }

    #[test]
    fn test_device_type_is_output() {
        assert!(DeviceType::OutputPhysical.is_output());
        assert!(DeviceType::OutputVirtual.is_output());
        assert!(!DeviceType::InputPhysical.is_output());
        assert!(!DeviceType::InputVirtual.is_output());
    }

    #[test]
    fn test_device_type_is_virtual() {
        assert!(DeviceType::InputVirtual.is_virtual());
        assert!(DeviceType::OutputVirtual.is_virtual());
        assert!(!DeviceType::InputPhysical.is_virtual());
        assert!(!DeviceType::OutputPhysical.is_virtual());
    }

    #[test]
    fn test_device_id_method() {
        let device = AudioDevice::new(
            DeviceId::new("test-device"),
            "Test".to_string(),
            DeviceType::InputPhysical,
            false,
            vec![44100],
            vec![2],
        );
        assert_eq!(device.id().as_str(), "test-device");
    }

    #[test]
    fn test_device_device_type_method() {
        let device = AudioDevice::new(
            DeviceId::new("test"),
            "Test".to_string(),
            DeviceType::OutputVirtual,
            false,
            vec![],
            vec![],
        );
        assert_eq!(device.device_type(), DeviceType::OutputVirtual);
    }

    #[test]
    fn test_device_sample_rates_method() {
        let device = AudioDevice::new(
            DeviceId::new("test"),
            "Test".to_string(),
            DeviceType::InputPhysical,
            false,
            vec![44100, 48000, 96000],
            vec![2],
        );
        assert_eq!(device.sample_rates(), &[44100, 48000, 96000]);
    }

    #[test]
    fn test_device_channels_method() {
        let device = AudioDevice::new(
            DeviceId::new("test"),
            "Test".to_string(),
            DeviceType::InputPhysical,
            false,
            vec![44100],
            vec![1, 2, 4],
        );
        assert_eq!(device.channels(), &[1, 2, 4]);
    }

    #[test]
    fn test_device_supports_channels() {
        let device = AudioDevice::new(
            DeviceId::new("test"),
            "Test".to_string(),
            DeviceType::InputPhysical,
            false,
            vec![44100],
            vec![1, 2],
        );
        assert!(device.supports_channels(1));
        assert!(device.supports_channels(2));
        assert!(!device.supports_channels(4));
    }

    #[test]
    fn test_device_clone() {
        let device1 = AudioDevice::new(
            DeviceId::new("test"),
            "Test".to_string(),
            DeviceType::InputPhysical,
            true,
            vec![44100],
            vec![2],
        );
        let device2 = device1.clone();
        assert_eq!(device1, device2);
    }

    #[test]
    fn test_device_partial_eq() {
        let device1 = AudioDevice::new(
            DeviceId::new("test"),
            "Test".to_string(),
            DeviceType::InputPhysical,
            true,
            vec![44100],
            vec![2],
        );
        let device2 = AudioDevice::new(
            DeviceId::new("test"),
            "Test".to_string(),
            DeviceType::InputPhysical,
            true,
            vec![44100],
            vec![2],
        );
        assert_eq!(device1, device2);
    }

    #[test]
    fn test_device_type_debug() {
        let dt = DeviceType::InputPhysical;
        let debug = format!("{:?}", dt);
        assert!(debug.contains("InputPhysical"));
    }

    #[test]
    fn test_device_id_debug() {
        let id = DeviceId::new("my-id");
        let debug = format!("{:?}", id);
        assert!(debug.contains("DeviceId"));
    }
}
