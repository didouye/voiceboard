//! CPAL-based device manager adapter

use crate::domain::{AudioDevice, DeviceId, DeviceType};
use crate::ports::{DeviceManager, DeviceManagerError};

/// Device manager adapter using CPAL
pub struct CpalDeviceManager {
    cached_devices: Vec<AudioDevice>,
}

impl CpalDeviceManager {
    pub fn new() -> Self {
        Self {
            cached_devices: Vec::new(),
        }
    }

    fn enumerate_devices(&self) -> Result<Vec<AudioDevice>, DeviceManagerError> {
        use cpal::traits::{DeviceTrait, HostTrait};

        let host = cpal::default_host();
        let mut devices = Vec::new();

        // Get default devices for comparison
        let default_input_name = host
            .default_input_device()
            .and_then(|d| d.name().ok());
        let default_output_name = host
            .default_output_device()
            .and_then(|d| d.name().ok());

        // Enumerate input devices
        if let Ok(input_devices) = host.input_devices() {
            for device in input_devices {
                if let Ok(name) = device.name() {
                    let is_default = default_input_name.as_ref() == Some(&name);

                    // Get supported configurations
                    let (sample_rates, channels) = Self::get_device_capabilities(&device, true);

                    devices.push(AudioDevice::new(
                        DeviceId::new(&name),
                        name,
                        DeviceType::InputPhysical,
                        is_default,
                        sample_rates,
                        channels,
                    ));
                }
            }
        }

        // Enumerate output devices
        if let Ok(output_devices) = host.output_devices() {
            for device in output_devices {
                if let Ok(name) = device.name() {
                    let is_default = default_output_name.as_ref() == Some(&name);

                    let (sample_rates, channels) = Self::get_device_capabilities(&device, false);

                    devices.push(AudioDevice::new(
                        DeviceId::new(&name),
                        name,
                        DeviceType::OutputPhysical,
                        is_default,
                        sample_rates,
                        channels,
                    ));
                }
            }
        }

        Ok(devices)
    }

    fn get_device_capabilities(device: &cpal::Device, is_input: bool) -> (Vec<u32>, Vec<u16>) {
        use cpal::traits::DeviceTrait;

        let configs = if is_input {
            device.supported_input_configs()
        } else {
            device.supported_output_configs()
        };

        let mut sample_rates = Vec::new();
        let mut channels = Vec::new();

        if let Ok(configs) = configs {
            for config in configs {
                // Common sample rates
                for rate in [8000, 16000, 22050, 44100, 48000, 96000] {
                    let sr = cpal::SampleRate(rate);
                    if sr >= config.min_sample_rate() && sr <= config.max_sample_rate() {
                        if !sample_rates.contains(&rate) {
                            sample_rates.push(rate);
                        }
                    }
                }

                let ch = config.channels();
                if !channels.contains(&ch) {
                    channels.push(ch);
                }
            }
        }

        // Default values if we couldn't get capabilities
        if sample_rates.is_empty() {
            sample_rates = vec![44100, 48000];
        }
        if channels.is_empty() {
            channels = vec![1, 2];
        }

        sample_rates.sort();
        channels.sort();

        (sample_rates, channels)
    }
}

impl Default for CpalDeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceManager for CpalDeviceManager {
    fn list_devices(&self) -> Result<Vec<AudioDevice>, DeviceManagerError> {
        self.enumerate_devices()
    }

    fn list_devices_by_type(
        &self,
        device_type: DeviceType,
    ) -> Result<Vec<AudioDevice>, DeviceManagerError> {
        let devices = self.enumerate_devices()?;
        Ok(devices
            .into_iter()
            .filter(|d| d.device_type() == device_type)
            .collect())
    }

    fn default_input_device(&self) -> Result<Option<AudioDevice>, DeviceManagerError> {
        let devices = self.list_devices_by_type(DeviceType::InputPhysical)?;
        Ok(devices.into_iter().find(|d| d.is_default()))
    }

    fn default_output_device(&self) -> Result<Option<AudioDevice>, DeviceManagerError> {
        let devices = self.list_devices_by_type(DeviceType::OutputPhysical)?;
        Ok(devices.into_iter().find(|d| d.is_default()))
    }

    fn get_device(&self, id: &DeviceId) -> Result<Option<AudioDevice>, DeviceManagerError> {
        let devices = self.enumerate_devices()?;
        Ok(devices.into_iter().find(|d| d.id() == id))
    }

    fn refresh(&mut self) -> Result<(), DeviceManagerError> {
        self.cached_devices = self.enumerate_devices()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_manager_creation() {
        let manager = CpalDeviceManager::new();
        assert!(manager.cached_devices.is_empty());
    }
}
