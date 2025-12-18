//! Tauri commands - Bridge between frontend and Rust backend

use crate::adapters::CpalDeviceManager;
use crate::application::audio_engine::AudioEngineCommand;
use crate::application::AppState;
use crate::domain::{AppSettings, AudioDevice, AudioSettings, ChannelType, DeviceType, MixerChannel, MixerConfig};
use crate::ports::DeviceManager;
use serde::{Deserialize, Serialize};
use tauri::State;
use tauri_plugin_store::StoreExt;

/// Settings store key
const SETTINGS_STORE: &str = "settings.json";
const SETTINGS_KEY: &str = "app_settings";

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeviceDto {
    pub id: String,
    pub name: String,
    pub device_type: String,
    pub is_default: bool,
    pub is_virtual: bool,
}

impl From<AudioDevice> for AudioDeviceDto {
    fn from(device: AudioDevice) -> Self {
        let is_virtual = device.device_type().is_virtual();
        Self {
            id: device.id().as_str().to_string(),
            name: device.name().to_string(),
            device_type: format!("{:?}", device.device_type()),
            is_default: device.is_default(),
            is_virtual,
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

/// DTO for audio settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettingsDto {
    pub input_device_id: Option<String>,
    pub output_device_id: Option<String>,
    pub master_volume: f32,
    pub sample_rate: u32,
    pub buffer_size: u32,
}

impl From<&AudioSettings> for AudioSettingsDto {
    fn from(settings: &AudioSettings) -> Self {
        Self {
            input_device_id: settings.input_device_id.clone(),
            output_device_id: settings.output_device_id.clone(),
            master_volume: settings.master_volume,
            sample_rate: settings.sample_rate,
            buffer_size: settings.buffer_size,
        }
    }
}

impl From<AudioSettingsDto> for AudioSettings {
    fn from(dto: AudioSettingsDto) -> Self {
        Self {
            input_device_id: dto.input_device_id,
            output_device_id: dto.output_device_id,
            master_volume: dto.master_volume,
            sample_rate: dto.sample_rate,
            buffer_size: dto.buffer_size,
        }
    }
}

/// DTO for app settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettingsDto {
    pub audio: AudioSettingsDto,
    pub start_minimized: bool,
    pub auto_start_mixing: bool,
}

impl From<&AppSettings> for AppSettingsDto {
    fn from(settings: &AppSettings) -> Self {
        Self {
            audio: AudioSettingsDto::from(&settings.audio),
            start_minimized: settings.start_minimized,
            auto_start_mixing: settings.auto_start_mixing,
        }
    }
}

impl From<AppSettingsDto> for AppSettings {
    fn from(dto: AppSettingsDto) -> Self {
        Self {
            audio: AudioSettings::from(dto.audio),
            start_minimized: dto.start_minimized,
            auto_start_mixing: dto.auto_start_mixing,
        }
    }
}

// ============================================================================
// Device Commands
// ============================================================================

/// Get list of all available audio devices
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

/// Get physical input devices (microphones)
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

/// Get virtual output devices (for sending mixed audio)
#[tauri::command]
pub async fn get_virtual_output_devices() -> ApiResponse<Vec<AudioDeviceDto>> {
    let manager = CpalDeviceManager::new();

    match manager.find_virtual_outputs() {
        Ok(devices) => {
            let dtos: Vec<AudioDeviceDto> = devices.into_iter().map(AudioDeviceDto::from).collect();
            ApiResponse::ok(dtos)
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Check if virtual audio driver is installed
#[tauri::command]
pub async fn check_virtual_driver() -> ApiResponse<bool> {
    let manager = CpalDeviceManager::new();

    match manager.find_virtual_outputs() {
        Ok(devices) => ApiResponse::ok(!devices.is_empty()),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

// ============================================================================
// Settings Commands
// ============================================================================

/// Get current application settings
#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettingsDto, String> {
    let settings = state.settings.read().await;
    Ok(AppSettingsDto::from(&*settings))
}

/// Save application settings
#[tauri::command]
pub async fn save_settings(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    settings: AppSettingsDto,
) -> Result<(), String> {
    // Update in-memory state
    {
        let mut current = state.settings.write().await;
        *current = AppSettings::from(settings.clone());
    }

    // Persist to store
    let store = app.store(SETTINGS_STORE).map_err(|e| e.to_string())?;
    store.set(SETTINGS_KEY, serde_json::to_value(&settings).map_err(|e| e.to_string())?);
    store.save().map_err(|e| e.to_string())?;

    tracing::info!("Settings saved");
    Ok(())
}

/// Load settings from persistent storage
#[tauri::command]
pub async fn load_settings(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<AppSettingsDto, String> {
    let store = app.store(SETTINGS_STORE).map_err(|e| e.to_string())?;

    if let Some(value) = store.get(SETTINGS_KEY) {
        let settings: AppSettingsDto = serde_json::from_value(value.clone())
            .map_err(|e| e.to_string())?;

        // Update in-memory state
        {
            let mut current = state.settings.write().await;
            *current = AppSettings::from(settings.clone());
        }

        Ok(settings)
    } else {
        // Return default settings
        let settings = state.settings.read().await;
        Ok(AppSettingsDto::from(&*settings))
    }
}

/// Set input device (microphone)
#[tauri::command]
pub async fn set_input_device(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    device_id: Option<String>,
) -> Result<(), String> {
    {
        let mut settings = state.settings.write().await;
        settings.audio.input_device_id = device_id;
    }

    // Auto-save settings
    let settings = state.settings.read().await;
    let dto = AppSettingsDto::from(&*settings);
    drop(settings);

    let store = app.store(SETTINGS_STORE).map_err(|e| e.to_string())?;
    store.set(SETTINGS_KEY, serde_json::to_value(&dto).map_err(|e| e.to_string())?);
    store.save().map_err(|e| e.to_string())?;

    tracing::info!("Input device updated");
    Ok(())
}

/// Set output device (virtual microphone)
#[tauri::command]
pub async fn set_output_device(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    device_id: Option<String>,
) -> Result<(), String> {
    {
        let mut settings = state.settings.write().await;
        settings.audio.output_device_id = device_id;
    }

    // Auto-save settings
    let settings = state.settings.read().await;
    let dto = AppSettingsDto::from(&*settings);
    drop(settings);

    let store = app.store(SETTINGS_STORE).map_err(|e| e.to_string())?;
    store.set(SETTINGS_KEY, serde_json::to_value(&dto).map_err(|e| e.to_string())?);
    store.save().map_err(|e| e.to_string())?;

    tracing::info!("Output device updated");
    Ok(())
}

// ============================================================================
// Mixer Configuration Commands
// ============================================================================

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

    // Also update in settings
    let mut settings = state.settings.write().await;
    settings.audio.master_volume = config.master_volume;

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

// ============================================================================
// Mixing Control Commands
// ============================================================================

/// Start mixing
#[tauri::command]
pub async fn start_mixing(state: State<'_, AppState>) -> Result<(), String> {
    // Verify we have devices selected
    let settings = state.settings.read().await;
    let input_device = settings
        .audio
        .input_device_id
        .clone()
        .ok_or_else(|| "No input device selected".to_string())?;
    let output_device = settings
        .audio
        .output_device_id
        .clone()
        .ok_or_else(|| "No output device selected".to_string())?;
    let sample_rate = settings.audio.sample_rate;
    drop(settings);

    // Send start command to audio engine
    let engine = state.audio_engine.lock().await;
    engine
        .send_command(AudioEngineCommand::Start {
            input_device,
            output_device,
            sample_rate,
            channels: 2, // Stereo
        })
        .map_err(|e| format!("Failed to start audio engine: {}", e))?;

    let mut is_mixing = state.is_mixing.write().await;
    *is_mixing = true;
    tracing::info!("Mixing started");
    Ok(())
}

/// Stop mixing
#[tauri::command]
pub async fn stop_mixing(state: State<'_, AppState>) -> Result<(), String> {
    // Send stop command to audio engine
    let engine = state.audio_engine.lock().await;
    engine
        .send_command(AudioEngineCommand::Stop)
        .map_err(|e| format!("Failed to stop audio engine: {}", e))?;

    let mut is_mixing = state.is_mixing.write().await;
    *is_mixing = false;
    tracing::info!("Mixing stopped");
    Ok(())
}

/// Get mixing status
#[tauri::command]
pub async fn is_mixing(state: State<'_, AppState>) -> Result<bool, String> {
    let engine = state.audio_engine.lock().await;
    Ok(engine.is_running())
}

// ============================================================================
// Sound Playback Commands
// ============================================================================

/// DTO for sound file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundFileDto {
    pub id: String,
    pub name: String,
    pub path: String,
    pub duration_ms: u64,
}

/// Load and decode an audio file, returning its metadata
#[tauri::command]
pub async fn load_sound_file(path: String) -> Result<SoundFileDto, String> {
    use rodio::Source;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;

    let file_path = Path::new(&path);

    // Validate file exists
    if !file_path.exists() {
        return Err(format!("File not found: {}", path));
    }

    // Get file name
    let name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    // Open and decode the file to get duration
    let file = File::open(&path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);

    let decoder = rodio::Decoder::new(reader)
        .map_err(|e| format!("Failed to decode audio file: {}", e))?;

    // Get duration
    let duration = decoder.total_duration()
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    // Generate unique ID
    let id = format!("sound_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..8].to_string());

    Ok(SoundFileDto {
        id,
        name,
        path,
        duration_ms: duration,
    })
}

/// Play a sound file (mix with microphone)
#[tauri::command]
pub async fn play_sound(
    state: State<'_, AppState>,
    id: String,
    path: String,
) -> Result<(), String> {
    use rodio::Source;
    use std::fs::File;
    use std::io::BufReader;

    // Decode the audio file
    let file = File::open(&path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);

    let decoder = rodio::Decoder::new(reader)
        .map_err(|e| format!("Failed to decode audio file: {}", e))?;

    // Get format info
    let sample_rate = decoder.sample_rate();
    let channels = decoder.channels();

    // Collect all samples as f32
    let samples: Vec<f32> = decoder.convert_samples::<f32>().collect();

    if samples.is_empty() {
        return Err("Audio file contains no samples".to_string());
    }

    // Send to audio engine
    let engine = state.audio_engine.lock().await;
    engine
        .send_command(AudioEngineCommand::PlaySound { id, samples })
        .map_err(|e| format!("Failed to play sound: {}", e))?;

    tracing::info!("Playing sound: {} ({} samples, {}Hz, {} ch)",
        path, samples.len(), sample_rate, channels);

    Ok(())
}

/// Stop a playing sound
#[tauri::command]
pub async fn stop_sound(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let engine = state.audio_engine.lock().await;
    engine
        .send_command(AudioEngineCommand::StopSound { id })
        .map_err(|e| format!("Failed to stop sound: {}", e))?;

    Ok(())
}

/// Set microphone volume (0.0 - 2.0)
#[tauri::command]
pub async fn set_mic_volume(
    state: State<'_, AppState>,
    volume: f32,
) -> Result<(), String> {
    let engine = state.audio_engine.lock().await;
    engine
        .send_command(AudioEngineCommand::SetMicVolume(volume))
        .map_err(|e| format!("Failed to set mic volume: {}", e))?;

    Ok(())
}

/// Mute/unmute microphone
#[tauri::command]
pub async fn set_mic_muted(
    state: State<'_, AppState>,
    muted: bool,
) -> Result<(), String> {
    let engine = state.audio_engine.lock().await;
    engine
        .send_command(AudioEngineCommand::SetMicMuted(muted))
        .map_err(|e| format!("Failed to set mic muted: {}", e))?;

    Ok(())
}
