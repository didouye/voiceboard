//! Windows virtual audio output adapter
//!
//! This adapter interfaces with the Virtual-Audio-Driver to send
//! mixed audio to the virtual microphone device.

use crate::domain::{AudioBuffer, AudioFormat, DeviceId};
use crate::ports::{AudioOutput, AudioOutputError, VirtualAudioOutput};

/// Virtual audio output adapter for Windows
///
/// Uses the Virtual-Audio-Driver (https://github.com/VirtualDrivers/Virtual-Audio-Driver)
/// to create a virtual microphone that other applications can use as input.
pub struct WindowsVirtualOutput {
    device_name: String,
    is_playing: bool,
    format: Option<AudioFormat>,
    // In a full implementation, this would hold WASAPI stream handles
}

impl WindowsVirtualOutput {
    /// Default name of the Virtual Audio Driver microphone
    pub const DEFAULT_DEVICE_NAME: &'static str = "Virtual Audio Device";

    pub fn new() -> Self {
        Self {
            device_name: Self::DEFAULT_DEVICE_NAME.to_string(),
            is_playing: false,
            format: None,
        }
    }

    pub fn with_device_name(mut self, name: impl Into<String>) -> Self {
        self.device_name = name.into();
        self
    }

    /// Check if the Virtual Audio Driver is installed
    pub fn check_driver_installed() -> bool {
        // In a full implementation, this would query the Windows audio
        // device list to check if the virtual device exists
        // For now, we'll use a placeholder implementation

        #[cfg(target_os = "windows")]
        {
            use cpal::traits::{DeviceTrait, HostTrait};

            let host = cpal::default_host();
            if let Ok(devices) = host.output_devices() {
                for device in devices {
                    if let Ok(name) = device.name() {
                        if name.contains("Virtual Audio") {
                            return true;
                        }
                    }
                }
            }
            false
        }

        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }
}

impl Default for WindowsVirtualOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioOutput for WindowsVirtualOutput {
    fn start(&mut self, _device_id: &DeviceId, format: AudioFormat) -> Result<(), AudioOutputError> {
        if !Self::check_driver_installed() {
            return Err(AudioOutputError::DeviceNotFound(
                "Virtual Audio Driver not installed. Please install from: \
                https://github.com/VirtualDrivers/Virtual-Audio-Driver"
                    .into(),
            ));
        }

        // In a full implementation:
        // 1. Open the virtual device using WASAPI
        // 2. Configure the stream with the specified format
        // 3. Start the audio stream

        self.format = Some(format);
        self.is_playing = true;

        tracing::info!("Virtual audio output started");
        Ok(())
    }

    fn stop(&mut self) -> Result<(), AudioOutputError> {
        self.is_playing = false;
        self.format = None;
        tracing::info!("Virtual audio output stopped");
        Ok(())
    }

    fn is_playing(&self) -> bool {
        self.is_playing
    }

    fn write(&mut self, buffer: &AudioBuffer) -> Result<(), AudioOutputError> {
        if !self.is_playing {
            return Err(AudioOutputError::StreamError("Stream not started".into()));
        }

        // In a full implementation:
        // 1. Convert buffer to the format expected by WASAPI
        // 2. Write to the virtual device's audio buffer
        // 3. Handle buffer underruns

        // For now, we just validate the buffer format matches
        if let Some(format) = &self.format {
            if buffer.sample_rate() != format.sample_rate {
                return Err(AudioOutputError::UnsupportedFormat(
                    "Sample rate mismatch".into(),
                ));
            }
            if buffer.channels() != format.channels {
                return Err(AudioOutputError::UnsupportedFormat(
                    "Channel count mismatch".into(),
                ));
            }
        }

        Ok(())
    }

    fn current_format(&self) -> Option<AudioFormat> {
        self.format
    }

    fn available_frames(&self) -> usize {
        // In a full implementation, this would query the WASAPI buffer
        1024
    }
}

impl VirtualAudioOutput for WindowsVirtualOutput {
    fn is_driver_installed(&self) -> bool {
        Self::check_driver_installed()
    }

    fn virtual_device_name(&self) -> &str {
        &self.device_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_output_creation() {
        let output = WindowsVirtualOutput::new();
        assert!(!output.is_playing());
        assert_eq!(
            output.virtual_device_name(),
            WindowsVirtualOutput::DEFAULT_DEVICE_NAME
        );
    }

    #[test]
    fn test_custom_device_name() {
        let output = WindowsVirtualOutput::new().with_device_name("My Virtual Mic");
        assert_eq!(output.virtual_device_name(), "My Virtual Mic");
    }
}
