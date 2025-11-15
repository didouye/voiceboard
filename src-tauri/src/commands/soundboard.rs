/// Soundboard-related Tauri commands
use crate::error::Result;
use crate::soundboard::{Sound, SoundboardManager};
use std::sync::Arc;
use tauri::State;

/// Get all sounds
#[tauri::command]
pub async fn get_sounds(manager: State<'_, Arc<SoundboardManager>>) -> Result<Vec<Sound>> {
    Ok(manager.get_all())
}

/// Add a new sound
#[tauri::command]
pub async fn add_sound(
    manager: State<'_, Arc<SoundboardManager>>,
    name: String,
    file_path: String,
) -> Result<Sound> {
    manager.add_sound(name, file_path).await
}

/// Delete a sound
#[tauri::command]
pub async fn delete_sound(manager: State<'_, Arc<SoundboardManager>>, id: String) -> Result<()> {
    manager.delete_sound(&id).await
}

/// Rename a sound
#[tauri::command]
pub async fn rename_sound(
    manager: State<'_, Arc<SoundboardManager>>,
    id: String,
    name: String,
) -> Result<()> {
    manager.rename_sound(&id, name).await
}

/// Update sound volume
#[tauri::command]
pub async fn update_sound_volume(
    manager: State<'_, Arc<SoundboardManager>>,
    id: String,
    volume: f32,
) -> Result<()> {
    manager.update_volume(&id, volume).await
}

/// Reorder sounds
#[tauri::command]
pub async fn reorder_sounds(
    manager: State<'_, Arc<SoundboardManager>>,
    ids: Vec<String>,
) -> Result<()> {
    manager.reorder_sounds(ids).await
}

/// Filter sounds by name
#[tauri::command]
pub async fn filter_sounds(
    manager: State<'_, Arc<SoundboardManager>>,
    query: String,
) -> Result<Vec<Sound>> {
    Ok(manager.filter_by_name(&query))
}

/// Get sound count
#[tauri::command]
pub async fn get_sound_count(manager: State<'_, Arc<SoundboardManager>>) -> Result<usize> {
    Ok(manager.count())
}
