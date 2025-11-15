/// Audio device manager
use super::types::{AudioDevice, DeviceType};
use crate::error::Result;
use cpal::traits::HostTrait;
use tracing::debug;

/// Manages audio device enumeration and selection
pub struct DeviceManager;

impl DeviceManager {
    /// List all available input devices
    pub fn list_input_devices() -> Result<Vec<AudioDevice>> {
        debug!("Listing input devices");

        let host = cpal::default_host();
        let default_device = host.default_input_device();
        let default_name = default_device.as_ref().and_then(|d| d.name().ok());

        let devices = host.input_devices()?;

        let mut result = Vec::new();
        for (index, device) in devices.enumerate() {
            if let Ok(name) = device.name() {
                let is_default = default_name.as_ref().map(|dn| *dn == name).unwrap_or(false);

                result.push(AudioDevice {
                    id: format!("input_{}", index),
                    name,
                    device_type: DeviceType::Input,
                    is_default,
                });
            }
        }

        debug!("Found {} input devices", result.len());
        Ok(result)
    }

    /// List all available output devices
    pub fn list_output_devices() -> Result<Vec<AudioDevice>> {
        debug!("Listing output devices");

        let host = cpal::default_host();
        let default_device = host.default_output_device();
        let default_name = default_device.as_ref().and_then(|d| d.name().ok());

        let devices = host.output_devices()?;

        let mut result = Vec::new();
        for (index, device) in devices.enumerate() {
            if let Ok(name) = device.name() {
                let is_default = default_name.as_ref().map(|dn| *dn == name).unwrap_or(false);

                result.push(AudioDevice {
                    id: format!("output_{}", index),
                    name,
                    device_type: DeviceType::Output,
                    is_default,
                });
            }
        }

        debug!("Found {} output devices", result.len());
        Ok(result)
    }

    /// Get the default input device
    pub fn get_default_input() -> Result<Option<AudioDevice>> {
        let host = cpal::default_host();

        if let Some(device) = host.default_input_device() {
            if let Ok(name) = device.name() {
                return Ok(Some(AudioDevice {
                    id: "input_default".to_string(),
                    name,
                    device_type: DeviceType::Input,
                    is_default: true,
                }));
            }
        }

        Ok(None)
    }

    /// Get the default output device
    pub fn get_default_output() -> Result<Option<AudioDevice>> {
        let host = cpal::default_host();

        if let Some(device) = host.default_output_device() {
            if let Ok(name) = device.name() {
                return Ok(Some(AudioDevice {
                    id: "output_default".to_string(),
                    name,
                    device_type: DeviceType::Output,
                    is_default: true,
                }));
            }
        }

        Ok(None)
    }
}
