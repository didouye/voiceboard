/// Device-related Tauri commands
use crate::devices::{AudioDevice, DeviceManager};
use crate::error::Result;

/// Get all input devices
#[tauri::command]
pub async fn get_input_devices() -> Result<Vec<AudioDevice>> {
    DeviceManager::list_input_devices()
}

/// Get all output devices
#[tauri::command]
pub async fn get_output_devices() -> Result<Vec<AudioDevice>> {
    DeviceManager::list_output_devices()
}

/// Get default input device
#[tauri::command]
pub async fn get_default_input_device() -> Result<Option<AudioDevice>> {
    DeviceManager::get_default_input()
}

/// Get default output device
#[tauri::command]
pub async fn get_default_output_device() -> Result<Option<AudioDevice>> {
    DeviceManager::get_default_output()
}

/// Select input device
#[tauri::command]
pub async fn select_input_device(device_id: String) -> Result<()> {
    // TODO: Implement device selection in audio engine
    tracing::info!("Selected input device: {}", device_id);
    Ok(())
}

/// Select output device (virtual cable)
#[tauri::command]
pub async fn select_output_device(device_id: String) -> Result<()> {
    // TODO: Implement device selection in audio engine
    tracing::info!("Selected output device: {}", device_id);
    Ok(())
}
