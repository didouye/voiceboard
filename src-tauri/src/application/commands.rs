//! Tauri commands - Bridge between frontend and Rust backend

use crate::adapters::CpalDeviceManager;
use crate::application::AppState;
use crate::domain::{AudioDevice, ChannelType, DeviceType, MixerChannel, MixerConfig};
use crate::ports::DeviceManager;
use serde::{Deserialize, Serialize};
use tauri::State;

/// Response wrapper for API calls
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(error: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.into()),
        }
    }
}

/// DTO for audio device information
#[derive(Debug, Serialize, Deserialize)]
pub struct AudioDeviceDto {
    pub id: String,
    pub name: String,
    pub device_type: String,
    pub is_default: bool,
}

impl From<AudioDevice> for AudioDeviceDto {
    fn from(device: AudioDevice) -> Self {
        Self {
            id: device.id().as_str().to_string(),
            name: device.name().to_string(),
            device_type: format!("{:?}", device.device_type()),
            is_default: device.is_default(),
        }
    }
}

/// DTO for mixer channel
#[derive(Debug, Serialize, Deserialize)]
pub struct MixerChannelDto {
    pub id: String,
    pub name: String,
    pub channel_type: String,
    pub volume: f32,
    pub muted: bool,
    pub solo: bool,
}

impl From<&MixerChannel> for MixerChannelDto {
    fn from(channel: &MixerChannel) -> Self {
        Self {
            id: channel.id().to_string(),
            name: channel.name().to_string(),
            channel_type: format!("{:?}", channel.channel_type()),
            volume: channel.volume(),
            muted: channel.is_muted(),
            solo: channel.is_solo(),
        }
    }
}

/// DTO for mixer configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct MixerConfigDto {
    pub master_volume: f32,
    pub channels: Vec<MixerChannelDto>,
    pub sample_rate: u32,
    pub buffer_size: u32,
}

impl From<&MixerConfig> for MixerConfigDto {
    fn from(config: &MixerConfig) -> Self {
        Self {
            master_volume: config.master_volume,
            channels: config.channels.iter().map(MixerChannelDto::from).collect(),
            sample_rate: config.output_format.sample_rate,
            buffer_size: config.buffer_size,
        }
    }
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Get list of available audio devices
#[tauri::command]
pub async fn get_audio_devices() -> ApiResponse<Vec<AudioDeviceDto>> {
    let manager = CpalDeviceManager::new();

    match manager.list_devices() {
        Ok(devices) => {
            let dtos: Vec<AudioDeviceDto> = devices.into_iter().map(AudioDeviceDto::from).collect();
            ApiResponse::ok(dtos)
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get input devices only
#[tauri::command]
pub async fn get_input_devices() -> ApiResponse<Vec<AudioDeviceDto>> {
    let manager = CpalDeviceManager::new();

    match manager.list_devices_by_type(DeviceType::InputPhysical) {
        Ok(devices) => {
            let dtos: Vec<AudioDeviceDto> = devices.into_iter().map(AudioDeviceDto::from).collect();
            ApiResponse::ok(dtos)
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get current mixer configuration
#[tauri::command]
pub async fn get_mixer_config(state: State<'_, AppState>) -> Result<MixerConfigDto, String> {
    let config = state.mixer_config.read().await;
    Ok(MixerConfigDto::from(&*config))
}

/// Set master volume
#[tauri::command]
pub async fn set_master_volume(
    state: State<'_, AppState>,
    volume: f32,
) -> Result<(), String> {
    let mut config = state.mixer_config.write().await;
    config.master_volume = volume.clamp(0.0, 1.0);
    Ok(())
}

/// Add a microphone channel
#[tauri::command]
pub async fn add_microphone_channel(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> Result<MixerChannelDto, String> {
    let channel = MixerChannel::new(&id, &name, ChannelType::Microphone);
    let dto = MixerChannelDto::from(&channel);

    let mut config = state.mixer_config.write().await;
    config.add_channel(channel);

    Ok(dto)
}

/// Add an audio file channel
#[tauri::command]
pub async fn add_audio_file_channel(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> Result<MixerChannelDto, String> {
    let channel = MixerChannel::new(&id, &name, ChannelType::AudioFile);
    let dto = MixerChannelDto::from(&channel);

    let mut config = state.mixer_config.write().await;
    config.add_channel(channel);

    Ok(dto)
}

/// Remove a channel
#[tauri::command]
pub async fn remove_channel(
    state: State<'_, AppState>,
    channel_id: String,
) -> Result<(), String> {
    let mut config = state.mixer_config.write().await;
    config
        .remove_channel(&channel_id)
        .ok_or_else(|| format!("Channel '{}' not found", channel_id))?;
    Ok(())
}

/// Set channel volume
#[tauri::command]
pub async fn set_channel_volume(
    state: State<'_, AppState>,
    channel_id: String,
    volume: f32,
) -> Result<(), String> {
    let mut config = state.mixer_config.write().await;
    let channel = config
        .get_channel_mut(&channel_id)
        .ok_or_else(|| format!("Channel '{}' not found", channel_id))?;
    channel.set_volume(volume);
    Ok(())
}

/// Toggle channel mute
#[tauri::command]
pub async fn toggle_channel_mute(
    state: State<'_, AppState>,
    channel_id: String,
) -> Result<bool, String> {
    let mut config = state.mixer_config.write().await;
    let channel = config
        .get_channel_mut(&channel_id)
        .ok_or_else(|| format!("Channel '{}' not found", channel_id))?;
    channel.toggle_mute();
    Ok(channel.is_muted())
}

/// Check if virtual audio driver is installed
#[tauri::command]
pub async fn check_virtual_driver() -> ApiResponse<bool> {
    #[cfg(target_os = "windows")]
    {
        use crate::adapters::WindowsVirtualOutput;
        ApiResponse::ok(WindowsVirtualOutput::check_driver_installed())
    }

    #[cfg(not(target_os = "windows"))]
    {
        ApiResponse::err("Virtual audio driver is only supported on Windows")
    }
}

/// Start mixing
#[tauri::command]
pub async fn start_mixing(state: State<'_, AppState>) -> Result<(), String> {
    let mut is_mixing = state.is_mixing.write().await;
    *is_mixing = true;
    tracing::info!("Mixing started");
    Ok(())
}

/// Stop mixing
#[tauri::command]
pub async fn stop_mixing(state: State<'_, AppState>) -> Result<(), String> {
    let mut is_mixing = state.is_mixing.write().await;
    *is_mixing = false;
    tracing::info!("Mixing stopped");
    Ok(())
}

/// Get mixing status
#[tauri::command]
pub async fn is_mixing(state: State<'_, AppState>) -> bool {
    *state.is_mixing.read().await
}
