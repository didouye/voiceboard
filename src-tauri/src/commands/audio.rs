/// Audio control Tauri commands
use crate::audio::{AudioEngine, Volume};
use crate::error::Result;
use std::sync::Arc;
use tauri::State;

/// Start the audio engine
#[tauri::command]
pub async fn start_audio_engine(engine: State<'_, Arc<AudioEngine>>) -> Result<()> {
    engine.start().await
}

/// Stop the audio engine
#[tauri::command]
pub async fn stop_audio_engine(engine: State<'_, Arc<AudioEngine>>) -> Result<()> {
    engine.stop().await
}

/// Play a sound
#[tauri::command]
pub async fn play_sound(
    engine: State<'_, Arc<AudioEngine>>,
    id: String,
    file_path: String,
) -> Result<()> {
    engine.play_sound(id, file_path)
}

/// Stop a playing sound
#[tauri::command]
pub async fn stop_sound(engine: State<'_, Arc<AudioEngine>>, id: String) -> Result<()> {
    engine.stop_sound(id)
}

/// Stop all playing sounds
#[tauri::command]
pub async fn stop_all_sounds(engine: State<'_, Arc<AudioEngine>>) -> Result<()> {
    engine.stop_all_sounds()
}

/// Set master volume
#[tauri::command]
pub async fn set_master_volume(engine: State<'_, Arc<AudioEngine>>, volume: f32) -> Result<()> {
    engine.set_master_volume(Volume::from_linear(volume))
}

/// Set microphone volume
#[tauri::command]
pub async fn set_mic_volume(engine: State<'_, Arc<AudioEngine>>, volume: f32) -> Result<()> {
    engine.set_mic_volume(Volume::from_linear(volume))
}

/// Set effects volume
#[tauri::command]
pub async fn set_effects_volume(engine: State<'_, Arc<AudioEngine>>, volume: f32) -> Result<()> {
    engine.set_effects_volume(Volume::from_linear(volume))
}
