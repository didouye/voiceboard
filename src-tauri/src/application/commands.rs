//! Tauri commands - Bridge between frontend and Rust backend

use crate::adapters::CpalDeviceManager;
use crate::application::audio_engine::AudioEngineCommand;
use crate::application::AppState;
use crate::domain::{
    AppSettings, AudioDevice, AudioSettings, ChannelType, DeviceType, MixerChannel, MixerConfig,
};
use crate::ports::DeviceManager;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, State};
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
    pub preview_device_id: Option<String>,
    pub master_volume: f32,
    pub sample_rate: u32,
    pub buffer_size: u32,
    #[serde(default)]
    pub mic_monitoring: bool,
    #[serde(default = "default_global_hotkeys")]
    pub global_hotkeys_enabled: bool,
}

fn default_global_hotkeys() -> bool {
    true
}

impl From<&AudioSettings> for AudioSettingsDto {
    fn from(settings: &AudioSettings) -> Self {
        Self {
            input_device_id: settings.input_device_id.clone(),
            output_device_id: settings.output_device_id.clone(),
            preview_device_id: settings.preview_device_id.clone(),
            master_volume: settings.master_volume,
            sample_rate: settings.sample_rate,
            buffer_size: settings.buffer_size,
            mic_monitoring: settings.mic_monitoring,
            global_hotkeys_enabled: settings.global_hotkeys_enabled,
        }
    }
}

impl From<AudioSettingsDto> for AudioSettings {
    fn from(dto: AudioSettingsDto) -> Self {
        Self {
            input_device_id: dto.input_device_id,
            output_device_id: dto.output_device_id,
            preview_device_id: dto.preview_device_id,
            master_volume: dto.master_volume,
            sample_rate: dto.sample_rate,
            buffer_size: dto.buffer_size,
            mic_monitoring: dto.mic_monitoring,
            global_hotkeys_enabled: dto.global_hotkeys_enabled,
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

/// Get physical output devices (speakers, headphones - for preview/monitoring)
#[tauri::command]
pub async fn get_physical_output_devices() -> ApiResponse<Vec<AudioDeviceDto>> {
    let manager = CpalDeviceManager::new();

    match manager.find_physical_outputs() {
        Ok(devices) => {
            let dtos: Vec<AudioDeviceDto> = devices.into_iter().map(AudioDeviceDto::from).collect();
            ApiResponse::ok(dtos)
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get virtual output devices sorted by priority (VB-Cable first, then Voicemeeter, etc.)
#[tauri::command]
pub async fn get_virtual_outputs_by_priority() -> ApiResponse<Vec<AudioDeviceDto>> {
    let manager = CpalDeviceManager::new();

    match manager.find_virtual_outputs_by_priority() {
        Ok(devices) => {
            tracing::info!(
                "[get_virtual_outputs_by_priority] Found {} virtual outputs",
                devices.len()
            );
            for (i, dev) in devices.iter().enumerate() {
                tracing::info!(
                    "[get_virtual_outputs_by_priority]   {}: {}",
                    i + 1,
                    dev.name()
                );
            }
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
    store.set(
        SETTINGS_KEY,
        serde_json::to_value(&settings).map_err(|e| e.to_string())?,
    );
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
    let store = app.store(SETTINGS_STORE).map_err(|e| {
        tracing::error!("Failed to open settings store: {}", e);
        e.to_string()
    })?;

    // Explicitly reload from disk to ensure we have the latest data
    if let Err(e) = store.reload() {
        tracing::warn!(
            "Could not reload settings from disk (may be first run): {}",
            e
        );
    }

    if let Some(value) = store.get(SETTINGS_KEY) {
        tracing::info!("Found saved settings: {:?}", value);
        let settings: AppSettingsDto = serde_json::from_value(value.clone()).map_err(|e| {
            tracing::error!("Failed to parse settings: {}", e);
            e.to_string()
        })?;

        tracing::info!(
            "Loaded settings - input: {:?}, output: {:?}",
            settings.audio.input_device_id,
            settings.audio.output_device_id
        );

        // Update in-memory state
        {
            let mut current = state.settings.write().await;
            *current = AppSettings::from(settings.clone());
        }

        Ok(settings)
    } else {
        tracing::info!("No saved settings found, returning defaults");
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
    tracing::info!("Setting input device to: {:?}", device_id);

    {
        let mut settings = state.settings.write().await;
        settings.audio.input_device_id = device_id.clone();
    }

    // Auto-save settings
    let settings = state.settings.read().await;
    let dto = AppSettingsDto::from(&*settings);
    drop(settings);

    let store = app.store(SETTINGS_STORE).map_err(|e| e.to_string())?;
    // Ensure store is reloaded before updating to avoid overwriting other settings
    let _ = store.reload();
    store.set(
        SETTINGS_KEY,
        serde_json::to_value(&dto).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| {
        tracing::error!("Failed to save settings: {}", e);
        e.to_string()
    })?;

    tracing::info!("Input device saved: {:?}", device_id);
    Ok(())
}

/// Set output device (virtual microphone)
#[tauri::command]
pub async fn set_output_device(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    device_id: Option<String>,
) -> Result<(), String> {
    tracing::info!("Setting output device to: {:?}", device_id);

    {
        let mut settings = state.settings.write().await;
        settings.audio.output_device_id = device_id.clone();
    }

    // Auto-save settings
    let settings = state.settings.read().await;
    let dto = AppSettingsDto::from(&*settings);
    drop(settings);

    let store = app.store(SETTINGS_STORE).map_err(|e| e.to_string())?;
    // Ensure store is reloaded before updating to avoid overwriting other settings
    let _ = store.reload();
    store.set(
        SETTINGS_KEY,
        serde_json::to_value(&dto).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| {
        tracing::error!("Failed to save settings: {}", e);
        e.to_string()
    })?;

    tracing::info!("Output device saved: {:?}", device_id);
    Ok(())
}

/// Set preview output device (for monitoring)
#[tauri::command]
pub async fn set_preview_device(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    device_id: Option<String>,
) -> Result<(), String> {
    tracing::info!("Setting preview device to: {:?}", device_id);

    {
        let mut settings = state.settings.write().await;
        settings.audio.preview_device_id = device_id.clone();
    }

    // Auto-save settings
    let settings = state.settings.read().await;
    let dto = AppSettingsDto::from(&*settings);
    drop(settings);

    let store = app.store(SETTINGS_STORE).map_err(|e| e.to_string())?;
    // Ensure store is reloaded before updating to avoid overwriting other settings
    let _ = store.reload();
    store.set(
        SETTINGS_KEY,
        serde_json::to_value(&dto).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| {
        tracing::error!("Failed to save settings: {}", e);
        e.to_string()
    })?;

    tracing::info!("Preview device saved: {:?}", device_id);
    Ok(())
}

/// Set mic monitoring enabled/disabled
#[tauri::command]
pub async fn set_mic_monitoring(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    tracing::info!("Setting mic monitoring to: {}", enabled);

    {
        let mut settings = state.settings.write().await;
        settings.audio.mic_monitoring = enabled;
    }

    // Send to audio engine
    let engine = state.audio_engine.lock().await;
    engine
        .send_command(AudioEngineCommand::SetMicMonitoring(enabled))
        .map_err(|e| format!("Failed to set mic monitoring: {}", e))?;

    // Auto-save settings
    let settings = state.settings.read().await;
    let dto = AppSettingsDto::from(&*settings);
    drop(settings);

    let store = app.store(SETTINGS_STORE).map_err(|e| e.to_string())?;
    let _ = store.reload();
    store.set(
        SETTINGS_KEY,
        serde_json::to_value(&dto).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| {
        tracing::error!("Failed to save settings: {}", e);
        e.to_string()
    })?;

    tracing::info!("Mic monitoring saved: {}", enabled);
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
pub async fn set_master_volume(state: State<'_, AppState>, volume: f32) -> Result<(), String> {
    let clamped_volume = volume.clamp(0.0, 1.0);

    // Update mixer config
    {
        let mut config = state.mixer_config.write().await;
        config.master_volume = clamped_volume;
    }

    // Update settings
    {
        let mut settings = state.settings.write().await;
        settings.audio.master_volume = clamped_volume;
    }

    // Send to audio engine
    let engine = state.audio_engine.lock().await;
    engine
        .send_command(AudioEngineCommand::SetMasterVolume(clamped_volume))
        .map_err(|e| format!("Failed to set master volume: {}", e))?;

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
pub async fn remove_channel(state: State<'_, AppState>, channel_id: String) -> Result<(), String> {
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
    use crate::application::audio_engine::AudioEngineEvent;

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
    let preview_device = settings
        .audio
        .preview_device_id
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let mic_monitoring = settings.audio.mic_monitoring;
    drop(settings);

    // Send monitoring device BEFORE start (so it's available when stream is created)
    let engine = state.audio_engine.lock().await;
    engine
        .send_command(AudioEngineCommand::SetMonitoringDevice(preview_device))
        .ok();

    // Restore mic monitoring state BEFORE start
    if mic_monitoring {
        engine
            .send_command(AudioEngineCommand::SetMicMonitoring(true))
            .ok();
    }

    // Send start command to audio engine
    engine
        .send_command(AudioEngineCommand::Start {
            input_device,
            output_device,
            sample_rate,
            channels: 2, // Stereo
        })
        .map_err(|e| format!("Failed to start audio engine: {}", e))?;

    // Poll for events to extract the actual sample rate used by the engine
    // The engine emits "Input config: Xch, YHz | Output config: Zch, YHz" when it starts
    let mut actual_sample_rate = sample_rate;
    let start_time = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(500);

    while start_time.elapsed() < timeout {
        if let Some(event) = engine.try_recv_event() {
            match event {
                AudioEngineEvent::Info(msg) => {
                    // Parse "Input config: Xch, YHz | Output config: Zch, YHz"
                    if msg.contains("Input config:") && msg.contains("Hz") {
                        // Extract sample rate from the message
                        // Format: "Input config: 1ch, 24000Hz | Output config: 16ch, 24000Hz"
                        if let Some(hz_pos) = msg.find("Hz") {
                            // Find the number before Hz
                            let before_hz = &msg[..hz_pos];
                            if let Some(comma_pos) = before_hz.rfind(", ") {
                                let rate_str = &before_hz[comma_pos + 2..];
                                if let Ok(rate) = rate_str.parse::<u32>() {
                                    actual_sample_rate = rate;
                                    tracing::info!(
                                        "Extracted actual engine sample rate: {}Hz",
                                        actual_sample_rate
                                    );
                                }
                            }
                        }
                    }
                }
                AudioEngineEvent::Started => {
                    // Engine started, we can stop polling
                    break;
                }
                AudioEngineEvent::Error(e) => {
                    return Err(format!("Audio engine error: {}", e));
                }
                _ => {}
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    // Update the state with the actual sample rate
    state.set_engine_sample_rate(actual_sample_rate);
    tracing::info!(
        "Engine sample rate set to: {}Hz (requested: {}Hz)",
        actual_sample_rate,
        sample_rate
    );

    drop(engine);

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
    pub duration: f64, // Duration in seconds
    pub sample_rate: u32,
    pub channels: u16,
}

/// Load and decode an audio file, returning its metadata
#[tauri::command]
pub async fn load_sound_file(path: String) -> Result<SoundFileDto, String> {
    load_sound_file_internal(&path).await
}

/// Load multiple audio files in parallel, returning results for each
/// Results are returned in the same order as the input paths
#[tauri::command]
pub async fn load_multiple_sound_files(paths: Vec<String>) -> Vec<Result<SoundFileDto, String>> {
    use futures::future::join_all;

    let futures: Vec<_> = paths
        .iter()
        .map(|path| load_sound_file_internal(path))
        .collect();
    join_all(futures).await
}

/// Internal function to load a single sound file (shared logic)
async fn load_sound_file_internal(path: &str) -> Result<SoundFileDto, String> {
    use rodio::Source;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;

    tracing::info!("[load_sound_file_internal] Loading: {}", path);

    let file_path = Path::new(path);

    if !file_path.exists() {
        return Err(format!("File not found: {}", path));
    }

    let name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);

    let decoder =
        rodio::Decoder::new(reader).map_err(|e| format!("Failed to decode audio file: {}", e))?;

    let sample_rate = decoder.sample_rate();
    let channels = decoder.channels();
    let duration = decoder
        .total_duration()
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);

    let id = format!(
        "sound_{}",
        &uuid::Uuid::new_v4().to_string().replace("-", "")[..8]
    );

    Ok(SoundFileDto {
        id,
        name,
        path: path.to_string(),
        duration,
        sample_rate,
        channels,
    })
}

/// Calculate SHA-256 hash of a file
#[tauri::command]
pub async fn hash_file(path: String) -> Result<String, String> {
    use sha2::{Digest, Sha256};

    let data = std::fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))?;
    let hash = format!("{:x}", Sha256::digest(&data));
    Ok(hash)
}

/// DTO for imported sound with hash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedSoundDto {
    pub hash: String,
    pub name: String,
    pub path: String,
    pub duration: f64,
}

/// Import a sound file and return its hash and metadata
#[tauri::command]
pub async fn import_sound_with_hash(path: String) -> Result<ImportedSoundDto, String> {
    use rodio::Source;
    use sha2::{Digest, Sha256};
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;

    // Read file and calculate hash
    let data = std::fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))?;
    let hash = format!("{:x}", Sha256::digest(&data));

    // Decode audio to get duration
    let file = File::open(&path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);
    let decoder =
        rodio::Decoder::new(reader).map_err(|e| format!("Failed to decode audio: {}", e))?;

    let duration = decoder
        .total_duration()
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);

    // Extract filename
    let name = Path::new(&path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(ImportedSoundDto {
        hash,
        name,
        path,
        duration,
    })
}

/// Import multiple sound files with hashes in parallel
#[tauri::command]
pub async fn import_multiple_sounds_with_hash(
    paths: Vec<String>,
) -> Vec<Result<ImportedSoundDto, String>> {
    use futures::future::join_all;

    let futures: Vec<_> = paths.into_iter().map(import_sound_with_hash).collect();

    join_all(futures).await
}

/// Play a sound file (mix with microphone) with optional volume (0.0-2.0, default 1.0)
/// and optional speed (0.5-2.0, default 1.0)
#[tauri::command]
pub async fn play_sound(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
    path: String,
    volume: Option<f32>,
    speed: Option<f32>,
) -> Result<(), String> {
    let volume = volume.unwrap_or(1.0);
    let speed = speed.unwrap_or(1.0);
    use rodio::Source;
    use std::fs::File;
    use std::io::BufReader;

    // Capture the stop generation BEFORE we start loading/decoding
    // If this changes while we're decoding, a StopAllSounds was called and we should abort
    let start_generation = state.get_stop_generation();

    // Get the ACTUAL sample rate from the audio engine state
    // This is set when the engine starts and reflects the negotiated rate
    // between input and output devices (may differ from device default config)
    let target_sample_rate = state.get_engine_sample_rate();
    tracing::debug!(
        "Using engine sample rate for resampling: {}Hz",
        target_sample_rate
    );

    // Decode the audio file
    let file = File::open(&path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);

    let decoder =
        rodio::Decoder::new(reader).map_err(|e| format!("Failed to decode audio file: {}", e))?;

    // Get format info
    let source_sample_rate = decoder.sample_rate();
    let channels = decoder.channels();

    // Collect all samples as f32
    let mut samples: Vec<f32> = decoder.convert_samples::<f32>().collect();
    let original_len = samples.len();

    if samples.is_empty() {
        return Err("Audio file contains no samples".to_string());
    }

    // Convert stereo to mono if needed (average channels)
    if channels == 2 {
        let mono_samples: Vec<f32> = samples
            .chunks(2)
            .map(|chunk| {
                if chunk.len() == 2 {
                    (chunk[0] + chunk[1]) / 2.0
                } else {
                    chunk[0]
                }
            })
            .collect();
        samples = mono_samples;
    }

    // Resample if source sample rate differs from target
    if source_sample_rate != target_sample_rate {
        let ratio = target_sample_rate as f64 / source_sample_rate as f64;
        let new_len = (samples.len() as f64 * ratio) as usize;
        let mut resampled = Vec::with_capacity(new_len);

        for i in 0..new_len {
            let src_pos = i as f64 / ratio;
            let src_idx = src_pos.floor() as usize;
            let frac = src_pos - src_idx as f64;

            let sample = if src_idx + 1 < samples.len() {
                // Linear interpolation
                let s0 = samples[src_idx] as f64;
                let s1 = samples[src_idx + 1] as f64;
                (s0 + (s1 - s0) * frac) as f32
            } else if src_idx < samples.len() {
                samples[src_idx]
            } else {
                0.0
            };

            resampled.push(sample);
        }

        tracing::info!(
            "Resampled sound from {}Hz to {}Hz ({} -> {} samples)",
            source_sample_rate,
            target_sample_rate,
            samples.len(),
            resampled.len()
        );
        samples = resampled;
    }

    let samples_len = samples.len();

    // Debug: log sample statistics to diagnose saturation issues
    let (min_sample, max_sample) = samples
        .iter()
        .fold((0.0f32, 0.0f32), |(min, max), &s| (min.min(s), max.max(s)));

    // Send debug info via event so it shows in frontend debug console
    use tauri::Emitter;
    let _ = app_handle.emit(
        "audio-debug",
        format!(
            "Sound: {}Hz->{}Hz, {}ch, peak={:.3}, resampled={}",
            source_sample_rate,
            target_sample_rate,
            channels,
            min_sample.abs().max(max_sample.abs()),
            source_sample_rate != target_sample_rate
        ),
    );

    // Check if a StopAllSounds was called while we were decoding
    // If so, don't play - the user already requested to stop all sounds
    let current_generation = state.get_stop_generation();
    if current_generation != start_generation {
        tracing::info!(
            "Sound '{}' discarded: StopAllSounds called during decode (gen {} -> {})",
            id,
            start_generation,
            current_generation
        );
        return Ok(());
    }

    // Send to audio engine
    let engine = state.audio_engine.lock().await;
    engine
        .send_command(AudioEngineCommand::PlaySound {
            id,
            samples,
            volume,
            speed,
        })
        .map_err(|e| format!("Failed to play sound: {}", e))?;

    tracing::info!(
        "Playing sound: {} ({} samples original, {} final, {}Hz -> {}Hz, {} ch, vol: {:.0}%, speed: {:.2}x)",
        path,
        original_len,
        samples_len,
        source_sample_rate,
        target_sample_rate,
        channels,
        volume * 100.0,
        speed
    );

    Ok(())
}

/// Preview a sound file on a specific output device
#[tauri::command]
pub async fn preview_sound(
    state: State<'_, AppState>,
    path: String,
    device_name: String,
    pad_id: String,
) -> Result<(), String> {
    use crate::application::preview_engine::PreviewCommand;

    let preview = state.preview_engine.lock().await;
    if let Some(ref engine) = *preview {
        engine.send_command(PreviewCommand::Play {
            path,
            device_name,
            pad_id,
        })
    } else {
        Err("Preview engine not initialized".to_string())
    }
}

/// Stop the currently playing preview
#[tauri::command]
pub async fn stop_preview(state: State<'_, AppState>) -> Result<(), String> {
    use crate::application::preview_engine::PreviewCommand;

    let preview = state.preview_engine.lock().await;
    if let Some(ref engine) = *preview {
        engine.send_command(PreviewCommand::Stop)
    } else {
        Err("Preview engine not initialized".to_string())
    }
}

/// Get the currently previewing pad ID
#[tauri::command]
pub async fn get_preview_state(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let preview = state.preview_engine.lock().await;
    Ok(preview.as_ref().and_then(|e| e.current_pad_id()))
}

/// Stop a playing sound
#[tauri::command]
pub async fn stop_sound(state: State<'_, AppState>, id: String) -> Result<(), String> {
    // Increment generation to abort any in-flight play_sound for this specific sound
    // This uses the same mechanism as stop_all_sounds
    state.increment_stop_generation();

    let engine = state.audio_engine.lock().await;
    engine
        .send_command(AudioEngineCommand::StopSound { id })
        .map_err(|e| format!("Failed to stop sound: {}", e))?;

    Ok(())
}

/// Stop all playing sounds
#[tauri::command]
pub async fn stop_all_sounds(state: State<'_, AppState>) -> Result<(), String> {
    // Increment generation FIRST to signal any in-flight play_sound calls to abort
    // This prevents sounds that started loading before this call from being added
    let new_gen = state.increment_stop_generation();
    tracing::debug!("StopAllSounds: generation incremented to {}", new_gen);

    let engine = state.audio_engine.lock().await;
    engine
        .send_command(AudioEngineCommand::StopAllSounds)
        .map_err(|e| format!("Failed to stop all sounds: {}", e))?;

    Ok(())
}

/// Set microphone volume (0.0 - 2.0)
#[tauri::command]
pub async fn set_mic_volume(state: State<'_, AppState>, volume: f32) -> Result<(), String> {
    let engine = state.audio_engine.lock().await;
    engine
        .send_command(AudioEngineCommand::SetMicVolume(volume))
        .map_err(|e| format!("Failed to set mic volume: {}", e))?;

    Ok(())
}

/// Mute/unmute microphone
#[tauri::command]
pub async fn set_mic_muted(state: State<'_, AppState>, muted: bool) -> Result<(), String> {
    let engine = state.audio_engine.lock().await;
    engine
        .send_command(AudioEngineCommand::SetMicMuted(muted))
        .map_err(|e| format!("Failed to set mic muted: {}", e))?;

    Ok(())
}

// ============================================================================
// Soundboard Persistence Commands
// ============================================================================

const SOUNDBOARD_STORE: &str = "soundboard.json";
const SOUNDBOARD_KEY: &str = "pads";

/// Save soundboard pads to persistent storage
#[tauri::command]
pub async fn save_soundboard(app: tauri::AppHandle, pads: serde_json::Value) -> Result<(), String> {
    let store = app.store(SOUNDBOARD_STORE).map_err(|e| e.to_string())?;
    store.set(SOUNDBOARD_KEY, pads);
    store.save().map_err(|e| e.to_string())?;
    tracing::debug!("Soundboard state saved");
    Ok(())
}

/// Load soundboard pads from persistent storage
#[tauri::command]
pub async fn load_soundboard(app: tauri::AppHandle) -> Result<Option<serde_json::Value>, String> {
    let store = app.store(SOUNDBOARD_STORE).map_err(|e| e.to_string())?;
    #[allow(clippy::map_clone)]
    let pads = store.get(SOUNDBOARD_KEY).map(|v| v.clone());
    tracing::debug!("Soundboard state loaded: {:?}", pads.is_some());
    Ok(pads)
}

const FOLDERS_KEY: &str = "folders";

/// Save folders to persistent storage
#[tauri::command]
pub async fn save_folders(app: tauri::AppHandle, folders: serde_json::Value) -> Result<(), String> {
    let store = app.store(SOUNDBOARD_STORE).map_err(|e| e.to_string())?;
    store.set(FOLDERS_KEY, folders);
    store.save().map_err(|e| e.to_string())?;
    tracing::debug!("Folders saved");
    Ok(())
}

/// Load folders from persistent storage
#[tauri::command]
pub async fn load_folders(app: tauri::AppHandle) -> Result<Option<serde_json::Value>, String> {
    let store = app.store(SOUNDBOARD_STORE).map_err(|e| e.to_string())?;
    #[allow(clippy::map_clone)]
    let folders = store.get(FOLDERS_KEY).map(|v| v.clone());
    tracing::debug!("Folders loaded: {:?}", folders.is_some());
    Ok(folders)
}

// ============================================================================
// Image Management Commands
// ============================================================================

/// Get the images directory path
#[tauri::command]
pub async fn get_images_dir(app: tauri::AppHandle) -> Result<String, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let images_dir = app_data_dir.join("images");

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&images_dir)
        .map_err(|e| format!("Failed to create images directory: {}", e))?;

    Ok(images_dir.to_string_lossy().to_string())
}

/// Save an image for a pad
/// Returns the relative path to the saved image
#[tauri::command]
pub async fn save_pad_image(
    app: tauri::AppHandle,
    pad_id: String,
    image_data: Vec<u8>,
    extension: String,
) -> Result<String, String> {
    use sha2::{Digest, Sha256};

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let images_dir = app_data_dir.join("images");
    std::fs::create_dir_all(&images_dir)
        .map_err(|e| format!("Failed to create images directory: {}", e))?;

    // Generate hash of image content (first 8 chars)
    let mut hasher = Sha256::new();
    hasher.update(&image_data);
    let hash = format!("{:x}", hasher.finalize());
    let hash_short = &hash[..8];

    // Clean extension (remove leading dot if present)
    let ext = extension.trim_start_matches('.');

    // Filename: {padId}-{hash8}.{ext}
    let filename = format!("{}-{}.{}", pad_id, hash_short, ext);
    let file_path = images_dir.join(&filename);

    // Delete any existing images for this pad first
    if let Ok(entries) = std::fs::read_dir(&images_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&format!("{}-", pad_id)) && name != filename {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }

    // Write new image
    std::fs::write(&file_path, &image_data).map_err(|e| format!("Failed to save image: {}", e))?;

    tracing::info!("Saved pad image: {}", filename);

    // Return relative path (just filename)
    Ok(filename)
}

/// Delete image for a pad
#[tauri::command]
pub async fn delete_pad_image(app: tauri::AppHandle, pad_id: String) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let images_dir = app_data_dir.join("images");

    // Delete all images for this pad
    if let Ok(entries) = std::fs::read_dir(&images_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&format!("{}-", pad_id)) {
                std::fs::remove_file(entry.path())
                    .map_err(|e| format!("Failed to delete image: {}", e))?;
                tracing::info!("Deleted pad image: {}", name);
            }
        }
    }

    Ok(())
}

/// Read an image file from disk
/// Used for uploading local images
#[tauri::command]
pub async fn read_image_file(path: String) -> Result<Vec<u8>, String> {
    // Validate file extension
    let path_lower = path.to_lowercase();
    let valid_extensions = [".jpg", ".jpeg", ".png", ".webp", ".gif"];
    if !valid_extensions.iter().any(|ext| path_lower.ends_with(ext)) {
        return Err("Invalid image format. Use JPG, PNG, WebP, or GIF.".to_string());
    }

    // Read file
    let data = std::fs::read(&path).map_err(|e| format!("Failed to read image: {}", e))?;

    // Validate file size (10MB max)
    if data.len() > 10 * 1024 * 1024 {
        return Err("Image too large. Maximum size is 10MB.".to_string());
    }

    Ok(data)
}

/// Clean up orphaned images (not referenced by any pad)
/// Called on app startup
#[tauri::command]
pub async fn cleanup_orphaned_images(app: tauri::AppHandle) -> Result<u32, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let images_dir = app_data_dir.join("images");

    if !images_dir.exists() {
        return Ok(0);
    }

    // Load soundboard to get referenced images
    let store = app.store(SOUNDBOARD_STORE).map_err(|e| e.to_string())?;
    let pads_value = store.get(SOUNDBOARD_KEY);

    let mut referenced_images: std::collections::HashSet<String> = std::collections::HashSet::new();

    if let Some(pads) = pads_value {
        if let Some(pads_array) = pads.as_array() {
            for pad in pads_array {
                if let Some(image) = pad.get("image") {
                    if let Some(local_path) = image.get("localPath").and_then(|v| v.as_str()) {
                        referenced_images.insert(local_path.to_string());
                    }
                }
            }
        }
    }

    // Delete orphaned images
    let mut deleted_count = 0u32;
    if let Ok(entries) = std::fs::read_dir(&images_dir) {
        for entry in entries.flatten() {
            let filename = entry.file_name().to_string_lossy().to_string();
            if !referenced_images.contains(&filename)
                && std::fs::remove_file(entry.path()).is_ok() {
                    tracing::info!("Deleted orphaned image: {}", filename);
                    deleted_count += 1;
                }
        }
    }

    Ok(deleted_count)
}

// ============================================================================
// Update Commands
// ============================================================================

use tauri_plugin_updater::UpdaterExt;

/// Information about an available update
#[derive(Debug, Serialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub version: Option<String>,
    pub body: Option<String>,
}

/// Check if an update is available
#[tauri::command]
pub async fn check_for_update(app: tauri::AppHandle) -> Result<UpdateInfo, String> {
    tracing::info!("Starting update check");

    let updater = match app.updater() {
        Ok(u) => {
            tracing::debug!("Updater instance created successfully");
            u
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create updater instance");
            return Err(e.to_string());
        }
    };

    tracing::debug!("Checking for updates from remote endpoint");

    match updater.check().await {
        Ok(Some(update)) => {
            tracing::info!(
                version = %update.version,
                current_version = env!("CARGO_PKG_VERSION"),
                "Update available"
            );
            Ok(UpdateInfo {
                available: true,
                version: Some(update.version.clone()),
                body: update.body.clone(),
            })
        }
        Ok(None) => {
            tracing::info!(
                current_version = env!("CARGO_PKG_VERSION"),
                "No update available - already on latest version"
            );
            Ok(UpdateInfo {
                available: false,
                version: None,
                body: None,
            })
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                error_debug = ?e,
                current_version = env!("CARGO_PKG_VERSION"),
                "Update check failed"
            );
            // Return error instead of silently failing
            Err(format!("Update check failed: {}", e))
        }
    }
}

/// Download and install an available update, then restart
#[tauri::command]
pub async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    tracing::info!("Starting update installation");

    let updater = match app.updater() {
        Ok(u) => {
            tracing::debug!("Updater instance created for installation");
            u
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create updater instance for installation");
            return Err(format!("Failed to initialize updater: {}", e));
        }
    };

    tracing::debug!("Checking for update before installation");

    let update = match updater.check().await {
        Ok(Some(update)) => {
            tracing::info!(version = %update.version, "Update found, proceeding with download");
            update
        }
        Ok(None) => {
            tracing::warn!("No update available when trying to install");
            return Err("No update available".to_string());
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to check for update during installation");
            return Err(format!("Failed to check for update: {}", e));
        }
    };

    tracing::info!(version = %update.version, "Starting download and installation");

    let download_result = update
        .download_and_install(
            |downloaded, total| {
                if let Some(total) = total {
                    let percent = (downloaded as f64 / total as f64 * 100.0) as u32;
                    if percent.is_multiple_of(25) {
                        tracing::debug!(
                            downloaded_bytes = downloaded,
                            total_bytes = total,
                            percent = percent,
                            "Download progress"
                        );
                    }
                }
            },
            || {
                tracing::info!("Download complete, starting installation");
            },
        )
        .await;

    match download_result {
        Ok(()) => {
            tracing::info!("Update installed successfully, restarting application");
            app.restart();
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                error_debug = ?e,
                "Failed to download and install update"
            );
            Err(format!("Failed to install update: {}", e))
        }
    }
}

// ============================================================================
// Debug Configuration
// ============================================================================

const DEBUG_STORE: &str = "debug.json";
const DEBUG_MODE_KEY: &str = "debug_mode";

/// Check if debug mode is enabled
/// Priority: 1) Runtime toggle (stored), 2) Runtime env var DEBUG_MODE
#[tauri::command]
pub fn get_debug_mode(app: tauri::AppHandle) -> bool {
    // First check the persistent store for runtime toggle
    if let Ok(store) = app.store(DEBUG_STORE) {
        if let Some(value) = store.get(DEBUG_MODE_KEY) {
            if let Some(enabled) = value.as_bool() {
                return enabled;
            }
        }
    }

    // Fall back to runtime environment variable
    std::env::var("DEBUG_MODE")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false)
}

/// Toggle debug mode and persist the setting
#[tauri::command]
pub fn set_debug_mode(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let store = app.store(DEBUG_STORE).map_err(|e| e.to_string())?;
    store.set(DEBUG_MODE_KEY, serde_json::json!(enabled));
    store.save().map_err(|e| e.to_string())?;

    // Emit event to update frontend UI
    let _ = app.emit("debug-mode-changed", enabled);

    tracing::info!(enabled = enabled, "Debug mode toggled");
    Ok(())
}

/// Get Sentry DSN (runtime env var)
#[tauri::command]
pub fn get_sentry_dsn() -> Option<String> {
    std::env::var("SENTRY_DSN").ok().filter(|s| !s.is_empty())
}

// ============================================================================
// VB-Cable Setup Commands
// ============================================================================

/// Status of VB-Cable installation
#[derive(Debug, Serialize)]
pub struct VbCableStatus {
    pub installed: bool,
    pub device_name: Option<String>,
}

/// Check if VB-Cable is specifically installed (not just any virtual device)
#[tauri::command]
pub async fn check_vb_cable_installed() -> Result<VbCableStatus, String> {
    tracing::info!("[check_vb_cable] Starting VB-Cable detection");
    let manager = CpalDeviceManager::new();

    match manager.list_devices() {
        Ok(all_devices) => {
            // Log all output devices for debugging
            let output_devices: Vec<_> = all_devices
                .iter()
                .filter(|d| {
                    matches!(
                        d.device_type(),
                        crate::domain::DeviceType::OutputPhysical
                            | crate::domain::DeviceType::OutputVirtual
                    )
                })
                .collect();
            tracing::info!(
                "[check_vb_cable] Found {} output devices:",
                output_devices.len()
            );
            for dev in &output_devices {
                let is_vb_cable = CpalDeviceManager::is_vb_cable_device(dev.name());
                tracing::info!(
                    "[check_vb_cable]   - {} (virtual: {}, vb-cable: {})",
                    dev.name(),
                    dev.device_type().is_virtual(),
                    is_vb_cable
                );
            }

            // Find specifically VB-Cable device (not just any virtual device)
            let vb_cable_device = all_devices
                .iter()
                .find(|d| CpalDeviceManager::is_vb_cable_device(d.name()));

            if let Some(device) = vb_cable_device {
                tracing::info!("[check_vb_cable] VB-Cable found: {}", device.name());
                Ok(VbCableStatus {
                    installed: true,
                    device_name: Some(device.name().to_string()),
                })
            } else {
                tracing::info!(
                    "[check_vb_cable] VB-Cable NOT found (other virtual devices may exist)"
                );
                Ok(VbCableStatus {
                    installed: false,
                    device_name: None,
                })
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "[check_vb_cable] Failed to list devices");
            Ok(VbCableStatus {
                installed: false,
                device_name: None,
            })
        }
    }
}

/// Download and install VB-Cable
/// Returns Ok(()) on success, Err with message on failure
#[tauri::command]
pub async fn download_and_install_vb_cable(_app: tauri::AppHandle) -> Result<(), String> {
    const VB_CABLE_URL: &str =
        "https://download.vb-audio.com/Download_CABLE/VBCABLE_Driver_Pack45.zip";

    tracing::info!("Starting VB-Cable download");

    // Get temp directory
    let temp_dir = std::env::temp_dir().join("voiceboard").join("vbcable");
    std::fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let zip_path = temp_dir.join("VBCABLE_Driver_Pack.zip");

    // Download ZIP file
    tracing::info!(url = VB_CABLE_URL, "Downloading VB-Cable");
    let response = reqwest::get(VB_CABLE_URL)
        .await
        .map_err(|e| format!("Download failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Write ZIP to disk
    std::fs::write(&zip_path, &bytes).map_err(|e| format!("Failed to save ZIP: {}", e))?;
    tracing::info!(path = ?zip_path, "ZIP downloaded");

    // Extract ZIP
    let zip_file =
        std::fs::File::open(&zip_path).map_err(|e| format!("Failed to open ZIP: {}", e))?;
    let mut archive = zip::ZipArchive::new(zip_file).map_err(|e| format!("Invalid ZIP: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("ZIP error: {}", e))?;
        let outpath = temp_dir.join(file.name());

        if file.is_dir() {
            std::fs::create_dir_all(&outpath).ok();
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| format!("Failed to create file: {}", e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to extract file: {}", e))?;
        }
    }
    tracing::info!("ZIP extracted");

    // Find and run installer
    let installer_path = temp_dir.join("VBCABLE_Setup_x64.exe");
    if !installer_path.exists() {
        // Try alternative name
        let alt_path = temp_dir.join("VBCABLE_Setup.exe");
        if alt_path.exists() {
            run_installer(&alt_path)?;
        } else {
            return Err("Installer not found in ZIP".to_string());
        }
    } else {
        run_installer(&installer_path)?;
    }

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);

    tracing::info!("VB-Cable installation completed");
    Ok(())
}

fn run_installer(path: &std::path::Path) -> Result<(), String> {
    tracing::info!(path = ?path, "Running VB-Cable installer with elevation");

    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use windows::core::PCWSTR;
        use windows::Win32::Foundation::HWND;
        use windows::Win32::System::Threading::{WaitForSingleObject, INFINITE};
        use windows::Win32::UI::Shell::{
            ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
        };

        // Convert path to wide string
        let path_wide: Vec<u16> = OsStr::new(path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        // "runas" verb for UAC elevation
        let verb: Vec<u16> = OsStr::new("runas")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut info = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: SEE_MASK_NOCLOSEPROCESS,
            hwnd: HWND::default(),
            lpVerb: PCWSTR(verb.as_ptr()),
            lpFile: PCWSTR(path_wide.as_ptr()),
            lpParameters: PCWSTR::null(),
            lpDirectory: PCWSTR::null(),
            nShow: 1, // SW_SHOWNORMAL
            hInstApp: Default::default(),
            lpIDList: std::ptr::null_mut(),
            lpClass: PCWSTR::null(),
            hkeyClass: Default::default(),
            dwHotKey: 0,
            Anonymous: Default::default(),
            hProcess: Default::default(),
        };

        let result = unsafe { ShellExecuteExW(&mut info) };

        if result.is_err() {
            return Err("Failed to launch installer (UAC may have been cancelled)".to_string());
        }

        // Wait for the installer to complete
        if !info.hProcess.is_invalid() {
            unsafe {
                WaitForSingleObject(info.hProcess, INFINITE);
            }
        }

        tracing::info!("VB-Cable installer completed");
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("VB-Cable installation is only supported on Windows".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AudioDevice, AudioFormat, AudioSettings, DeviceId, DeviceType};

    // ==================== ApiResponse Tests ====================

    #[test]
    fn test_api_response_ok() {
        let response: ApiResponse<String> = ApiResponse::ok("test data".to_string());

        assert!(response.success);
        assert_eq!(response.data, Some("test data".to_string()));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_ok_with_vec() {
        let data = vec![1, 2, 3];
        let response: ApiResponse<Vec<i32>> = ApiResponse::ok(data.clone());

        assert!(response.success);
        assert_eq!(response.data, Some(data));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_err() {
        let response: ApiResponse<String> = ApiResponse::err("Something went wrong");

        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("Something went wrong".to_string()));
    }

    #[test]
    fn test_api_response_err_from_string() {
        let error_msg = String::from("Error message");
        let response: ApiResponse<i32> = ApiResponse::err(error_msg);

        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("Error message".to_string()));
    }

    // ==================== AudioDeviceDto Tests ====================

    #[test]
    fn test_audio_device_dto_from_physical_input() {
        let device = AudioDevice::new(
            DeviceId::new("mic-1"),
            "Built-in Microphone".to_string(),
            DeviceType::InputPhysical,
            true,
            vec![48000],
            vec![1, 2],
        );

        let dto = AudioDeviceDto::from(device);

        assert_eq!(dto.id, "mic-1");
        assert_eq!(dto.name, "Built-in Microphone");
        assert_eq!(dto.device_type, "InputPhysical");
        assert!(dto.is_default);
        assert!(!dto.is_virtual);
    }

    #[test]
    fn test_audio_device_dto_from_virtual_output() {
        let device = AudioDevice::new(
            DeviceId::new("vb-cable"),
            "VB-Cable Input".to_string(),
            DeviceType::OutputVirtual,
            false,
            vec![44100, 48000],
            vec![2],
        );

        let dto = AudioDeviceDto::from(device);

        assert_eq!(dto.id, "vb-cable");
        assert_eq!(dto.name, "VB-Cable Input");
        assert_eq!(dto.device_type, "OutputVirtual");
        assert!(!dto.is_default);
        assert!(dto.is_virtual);
    }

    // ==================== MixerChannelDto Tests ====================

    #[test]
    fn test_mixer_channel_dto_from_microphone() {
        let channel = MixerChannel::new("ch-1", "Main Mic", ChannelType::Microphone);
        let dto = MixerChannelDto::from(&channel);

        assert_eq!(dto.id, "ch-1");
        assert_eq!(dto.name, "Main Mic");
        assert_eq!(dto.channel_type, "Microphone");
        assert_eq!(dto.volume, 1.0);
        assert!(!dto.muted);
        assert!(!dto.solo);
    }

    #[test]
    fn test_mixer_channel_dto_from_audio_file() {
        let mut channel = MixerChannel::new("ch-2", "Sound Effect", ChannelType::AudioFile);
        channel.set_volume(0.75);
        channel.set_muted(true);

        let dto = MixerChannelDto::from(&channel);

        assert_eq!(dto.id, "ch-2");
        assert_eq!(dto.name, "Sound Effect");
        assert_eq!(dto.channel_type, "AudioFile");
        assert_eq!(dto.volume, 0.75);
        assert!(dto.muted);
        assert!(!dto.solo);
    }

    // ==================== MixerConfigDto Tests ====================

    #[test]
    fn test_mixer_config_dto_from() {
        let mut config = MixerConfig::new(AudioFormat::default(), 512);
        config.master_volume = 0.8;
        config.add_channel(MixerChannel::new("ch-1", "Mic", ChannelType::Microphone));

        let dto = MixerConfigDto::from(&config);

        assert_eq!(dto.master_volume, 0.8);
        assert_eq!(dto.channels.len(), 1);
        assert_eq!(dto.channels[0].id, "ch-1");
        assert_eq!(dto.buffer_size, 512);
    }

    // ==================== AudioSettingsDto Tests ====================

    #[test]
    fn test_audio_settings_dto_from() {
        let settings = AudioSettings {
            input_device_id: Some("mic-1".to_string()),
            output_device_id: Some("vb-cable".to_string()),
            preview_device_id: Some("speakers".to_string()),
            master_volume: 0.9,
            sample_rate: 48000,
            buffer_size: 512,
            mic_monitoring: true,
            global_hotkeys_enabled: true,
        };

        let dto = AudioSettingsDto::from(&settings);

        assert_eq!(dto.input_device_id, Some("mic-1".to_string()));
        assert_eq!(dto.output_device_id, Some("vb-cable".to_string()));
        assert_eq!(dto.preview_device_id, Some("speakers".to_string()));
        assert_eq!(dto.master_volume, 0.9);
        assert_eq!(dto.sample_rate, 48000);
        assert_eq!(dto.buffer_size, 512);
        assert!(dto.mic_monitoring);
    }

    #[test]
    fn test_audio_settings_from_dto() {
        let dto = AudioSettingsDto {
            input_device_id: Some("mic-2".to_string()),
            output_device_id: None,
            preview_device_id: None,
            master_volume: 0.5,
            sample_rate: 44100,
            buffer_size: 256,
            mic_monitoring: false,
            global_hotkeys_enabled: true,
        };

        let settings = AudioSettings::from(dto);

        assert_eq!(settings.input_device_id, Some("mic-2".to_string()));
        assert!(settings.output_device_id.is_none());
        assert!(settings.preview_device_id.is_none());
        assert_eq!(settings.master_volume, 0.5);
        assert_eq!(settings.sample_rate, 44100);
        assert_eq!(settings.buffer_size, 256);
        assert!(!settings.mic_monitoring);
    }

    #[test]
    fn test_audio_settings_roundtrip() {
        let original = AudioSettings {
            input_device_id: Some("test-mic".to_string()),
            output_device_id: Some("test-output".to_string()),
            preview_device_id: Some("test-preview".to_string()),
            master_volume: 0.75,
            sample_rate: 48000,
            buffer_size: 1024,
            mic_monitoring: true,
            global_hotkeys_enabled: true,
        };

        let dto = AudioSettingsDto::from(&original);
        let converted = AudioSettings::from(dto);

        assert_eq!(original.input_device_id, converted.input_device_id);
        assert_eq!(original.output_device_id, converted.output_device_id);
        assert_eq!(original.preview_device_id, converted.preview_device_id);
        assert_eq!(original.master_volume, converted.master_volume);
        assert_eq!(original.sample_rate, converted.sample_rate);
        assert_eq!(original.buffer_size, converted.buffer_size);
        assert_eq!(original.mic_monitoring, converted.mic_monitoring);
    }

    // ==================== AppSettingsDto Tests ====================

    #[test]
    fn test_app_settings_dto_from() {
        let settings = AppSettings {
            audio: AudioSettings::default(),
            start_minimized: true,
            auto_start_mixing: false,
        };

        let dto = AppSettingsDto::from(&settings);

        assert!(dto.start_minimized);
        assert!(!dto.auto_start_mixing);
    }

    #[test]
    fn test_app_settings_from_dto() {
        let dto = AppSettingsDto {
            audio: AudioSettingsDto {
                input_device_id: None,
                output_device_id: None,
                preview_device_id: None,
                master_volume: 1.0,
                sample_rate: 48000,
                buffer_size: 512,
                mic_monitoring: false,
                global_hotkeys_enabled: true,
            },
            start_minimized: false,
            auto_start_mixing: true,
        };

        let settings = AppSettings::from(dto);

        assert!(!settings.start_minimized);
        assert!(settings.auto_start_mixing);
    }

    #[test]
    fn test_app_settings_roundtrip() {
        let original = AppSettings {
            audio: AudioSettings {
                input_device_id: Some("mic".to_string()),
                output_device_id: Some("output".to_string()),
                preview_device_id: None,
                master_volume: 0.8,
                sample_rate: 44100,
                buffer_size: 256,
                mic_monitoring: true,
                global_hotkeys_enabled: true,
            },
            start_minimized: true,
            auto_start_mixing: true,
        };

        let dto = AppSettingsDto::from(&original);
        let converted = AppSettings::from(dto);

        assert_eq!(original.start_minimized, converted.start_minimized);
        assert_eq!(original.auto_start_mixing, converted.auto_start_mixing);
        assert_eq!(
            original.audio.input_device_id,
            converted.audio.input_device_id
        );
        assert_eq!(
            original.audio.mic_monitoring,
            converted.audio.mic_monitoring
        );
    }

    // ==================== UpdateInfo Tests ====================

    #[test]
    fn test_update_info_available() {
        let info = UpdateInfo {
            available: true,
            version: Some("1.2.0".to_string()),
            body: Some("Bug fixes and improvements".to_string()),
        };

        assert!(info.available);
        assert_eq!(info.version, Some("1.2.0".to_string()));
        assert!(info.body.is_some());
    }

    #[test]
    fn test_update_info_not_available() {
        let info = UpdateInfo {
            available: false,
            version: None,
            body: None,
        };

        assert!(!info.available);
        assert!(info.version.is_none());
        assert!(info.body.is_none());
    }

    // ==================== VbCableStatus Tests ====================

    #[test]
    fn test_vb_cable_status_installed() {
        let status = VbCableStatus {
            installed: true,
            device_name: Some("CABLE Input (VB-Audio Virtual Cable)".to_string()),
        };

        assert!(status.installed);
        assert!(status.device_name.is_some());
        assert!(status.device_name.unwrap().contains("VB-Audio"));
    }

    #[test]
    fn test_vb_cable_status_not_installed() {
        let status = VbCableStatus {
            installed: false,
            device_name: None,
        };

        assert!(!status.installed);
        assert!(status.device_name.is_none());
    }

    // ==================== SoundFileDto Tests ====================

    #[test]
    fn test_sound_file_dto_creation() {
        let dto = SoundFileDto {
            id: "sound_abc123".to_string(),
            name: "airhorn".to_string(),
            path: "/sounds/airhorn.mp3".to_string(),
            duration: 2.5,
            sample_rate: 44100,
            channels: 2,
        };

        assert_eq!(dto.id, "sound_abc123");
        assert_eq!(dto.name, "airhorn");
        assert_eq!(dto.duration, 2.5);
        assert_eq!(dto.sample_rate, 44100);
        assert_eq!(dto.channels, 2);
    }

    // ==================== Serialization Tests ====================

    #[test]
    fn test_api_response_serialization() {
        let response: ApiResponse<String> = ApiResponse::ok("test".to_string());
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"data\":\"test\""));
    }

    #[test]
    fn test_audio_device_dto_serialization() {
        let dto = AudioDeviceDto {
            id: "mic-1".to_string(),
            name: "Microphone".to_string(),
            device_type: "InputPhysical".to_string(),
            is_default: true,
            is_virtual: false,
        };

        let json = serde_json::to_string(&dto).unwrap();
        let deserialized: AudioDeviceDto = serde_json::from_str(&json).unwrap();

        assert_eq!(dto.id, deserialized.id);
        assert_eq!(dto.name, deserialized.name);
        assert_eq!(dto.is_default, deserialized.is_default);
    }

    #[test]
    fn test_audio_settings_dto_serialization() {
        let dto = AudioSettingsDto {
            input_device_id: Some("mic".to_string()),
            output_device_id: None,
            preview_device_id: None,
            master_volume: 0.8,
            sample_rate: 48000,
            buffer_size: 512,
            mic_monitoring: true,
            global_hotkeys_enabled: true,
        };

        let json = serde_json::to_string(&dto).unwrap();
        let deserialized: AudioSettingsDto = serde_json::from_str(&json).unwrap();

        assert_eq!(dto.input_device_id, deserialized.input_device_id);
        assert_eq!(dto.master_volume, deserialized.master_volume);
        assert_eq!(dto.mic_monitoring, deserialized.mic_monitoring);
    }
}
