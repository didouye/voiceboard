//! Tauri commands - Bridge between frontend and Rust backend

use crate::adapters::CpalDeviceManager;
use crate::application::audio_engine::AudioEngineCommand;
use crate::application::AppState;
use crate::domain::{
    AppSettings, AudioDevice, AudioSettings, ChannelType, DeviceType, MixerChannel, MixerConfig,
};
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
    pub preview_device_id: Option<String>,
    pub master_volume: f32,
    pub sample_rate: u32,
    pub buffer_size: u32,
    #[serde(default)]
    pub mic_monitoring: bool,
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
    let preview_device = settings.audio.preview_device_id.clone().unwrap_or_else(|| "default".to_string());
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
    use rodio::Source;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;

    tracing::info!("[load_sound_file] Called with path: {}", path);

    let file_path = Path::new(&path);

    // Validate file exists
    if !file_path.exists() {
        tracing::error!("[load_sound_file] File not found: {}", path);
        return Err(format!("File not found: {}", path));
    }
    tracing::info!("[load_sound_file] File exists: {}", path);

    // Get file name
    let name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();
    tracing::info!("[load_sound_file] File name: {}", name);

    // Open and decode the file to get metadata
    let file = File::open(&path).map_err(|e| {
        tracing::error!("[load_sound_file] Failed to open file: {}", e);
        format!("Failed to open file: {}", e)
    })?;
    let reader = BufReader::new(file);
    tracing::info!("[load_sound_file] File opened successfully");

    let decoder = rodio::Decoder::new(reader).map_err(|e| {
        tracing::error!("[load_sound_file] Failed to decode audio: {}", e);
        format!("Failed to decode audio file: {}", e)
    })?;
    tracing::info!("[load_sound_file] Decoder created successfully");

    let sample_rate = decoder.sample_rate();
    let channels = decoder.channels();
    tracing::info!(
        "[load_sound_file] Audio info: {}Hz, {} channels",
        sample_rate,
        channels
    );

    // Get duration in seconds
    let duration = decoder
        .total_duration()
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    tracing::info!("[load_sound_file] Duration: {:.2}s", duration);

    // Generate unique ID
    let id = format!(
        "sound_{}",
        &uuid::Uuid::new_v4().to_string().replace("-", "")[..8]
    );

    tracing::info!(
        "[load_sound_file] Success: {} ({:.1}s, {}Hz, {}ch)",
        name,
        duration,
        sample_rate,
        channels
    );

    Ok(SoundFileDto {
        id,
        name,
        path,
        duration,
        sample_rate,
        channels,
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

    // Get engine sample rate from settings
    let settings = state.settings.read().await;
    let target_sample_rate = settings.audio.sample_rate;
    drop(settings);

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

    // Send to audio engine
    let engine = state.audio_engine.lock().await;
    engine
        .send_command(AudioEngineCommand::PlaySound { id, samples })
        .map_err(|e| format!("Failed to play sound: {}", e))?;

    tracing::info!(
        "Playing sound: {} ({} samples original, {} final, {}Hz -> {}Hz, {} ch)",
        path,
        original_len,
        samples_len,
        source_sample_rate,
        target_sample_rate,
        channels
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
    let engine = state.audio_engine.lock().await;
    engine
        .send_command(AudioEngineCommand::StopSound { id })
        .map_err(|e| format!("Failed to stop sound: {}", e))?;

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
