/// Soundboard manager for high-level operations
use super::sound::Sound;
use super::storage::SoundboardStorage;
use crate::error::{Result, VoiceboardError};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};

/// Soundboard manager
pub struct SoundboardManager {
    /// SQLite storage
    storage: Arc<SoundboardStorage>,
    /// In-memory cache of sounds
    cache: Arc<RwLock<HashMap<String, Sound>>>,
}

impl SoundboardManager {
    /// Create a new soundboard manager
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let storage = Arc::new(SoundboardStorage::new(db_path).await?);

        let manager = Self {
            storage,
            cache: Arc::new(RwLock::new(HashMap::new())),
        };

        // Load all sounds into cache
        manager.reload_cache().await?;

        Ok(manager)
    }

    /// Reload the cache from storage
    pub async fn reload_cache(&self) -> Result<()> {
        debug!("Reloading soundboard cache");

        let sounds = self.storage.get_all().await?;
        let mut cache = self.cache.write();

        cache.clear();
        for sound in sounds {
            cache.insert(sound.id.clone(), sound);
        }

        info!("Loaded {} sounds into cache", cache.len());
        Ok(())
    }

    /// Get all sounds
    pub fn get_all(&self) -> Vec<Sound> {
        let cache = self.cache.read();
        let mut sounds: Vec<Sound> = cache.values().cloned().collect();

        // Sort by sort_order, then by created_at
        sounds.sort_by(|a, b| {
            a.sort_order
                .cmp(&b.sort_order)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });

        sounds
    }

    /// Get a sound by ID
    pub fn get_by_id(&self, id: &str) -> Option<Sound> {
        self.cache.read().get(id).cloned()
    }

    /// Add a new sound
    pub async fn add_sound(&self, name: String, file_path: String) -> Result<Sound> {
        info!("Adding sound: {} from {}", name, file_path);

        // Validate file exists
        if !Path::new(&file_path).exists() {
            return Err(VoiceboardError::SoundFile(format!(
                "File not found: {}",
                file_path
            )));
        }

        let sound = Sound::new(name, file_path);

        // Validate sound
        sound.validate().map_err(VoiceboardError::InvalidOperation)?;

        // Check if format is supported
        if !sound.is_supported_format() {
            return Err(VoiceboardError::SoundFile(format!(
                "Unsupported audio format: {:?}",
                sound.file_extension()
            )));
        }

        // Insert into storage
        self.storage.insert(&sound).await?;

        // Update cache
        self.cache.write().insert(sound.id.clone(), sound.clone());

        info!("Sound added successfully: {}", sound.id);
        Ok(sound)
    }

    /// Delete a sound
    pub async fn delete_sound(&self, id: &str) -> Result<()> {
        info!("Deleting sound: {}", id);

        // Delete from storage
        self.storage.delete(id).await?;

        // Remove from cache
        self.cache.write().remove(id);

        info!("Sound deleted successfully: {}", id);
        Ok(())
    }

    /// Rename a sound
    pub async fn rename_sound(&self, id: &str, new_name: String) -> Result<()> {
        info!("Renaming sound {} to {}", id, new_name);

        let mut sound = self
            .get_by_id(id)
            .ok_or_else(|| VoiceboardError::SoundNotFound(id.to_string()))?;

        sound.set_name(new_name);
        sound.validate().map_err(VoiceboardError::InvalidOperation)?;

        // Update storage
        self.storage.update(&sound).await?;

        // Update cache
        self.cache.write().insert(sound.id.clone(), sound);

        info!("Sound renamed successfully: {}", id);
        Ok(())
    }

    /// Update sound volume
    pub async fn update_volume(&self, id: &str, volume: f32) -> Result<()> {
        debug!("Updating volume for sound {} to {}", id, volume);

        let mut sound = self
            .get_by_id(id)
            .ok_or_else(|| VoiceboardError::SoundNotFound(id.to_string()))?;

        sound.set_volume(volume);
        sound.validate().map_err(VoiceboardError::InvalidOperation)?;

        // Update storage
        self.storage.update(&sound).await?;

        // Update cache
        self.cache.write().insert(sound.id.clone(), sound);

        Ok(())
    }

    /// Reorder sounds
    pub async fn reorder_sounds(&self, sound_ids: Vec<String>) -> Result<()> {
        info!("Reordering {} sounds", sound_ids.len());

        // Validate all IDs exist
        {
            let cache = self.cache.read();
            for id in &sound_ids {
                if !cache.contains_key(id) {
                    return Err(VoiceboardError::SoundNotFound(id.to_string()));
                }
            }
        }

        // Update sort orders in storage
        self.storage.update_sort_orders(&sound_ids).await?;

        // Update cache
        let mut cache = self.cache.write();
        for (index, id) in sound_ids.iter().enumerate() {
            if let Some(sound) = cache.get_mut(id) {
                sound.set_sort_order(index as i32);
            }
        }

        info!("Sounds reordered successfully");
        Ok(())
    }

    /// Filter sounds by name
    pub fn filter_by_name(&self, query: &str) -> Vec<Sound> {
        let query_lower = query.to_lowercase();
        let mut sounds: Vec<Sound> = self
            .cache
            .read()
            .values()
            .filter(|sound| sound.name.to_lowercase().contains(&query_lower))
            .cloned()
            .collect();

        sounds.sort_by(|a, b| {
            a.sort_order
                .cmp(&b.sort_order)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });

        sounds
    }

    /// Get the count of sounds
    pub fn count(&self) -> usize {
        self.cache.read().len()
    }

    /// Clear all sounds
    pub async fn clear_all(&self) -> Result<()> {
        info!("Clearing all sounds");

        self.storage.delete_all().await?;
        self.cache.write().clear();

        info!("All sounds cleared");
        Ok(())
    }

    /// Validate all sound files still exist
    pub fn validate_files(&self) -> Vec<String> {
        let cache = self.cache.read();
        cache
            .values()
            .filter(|sound| !sound.file_exists())
            .map(|sound| sound.id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;

    async fn create_test_manager() -> (SoundboardManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let manager = SoundboardManager::new(&db_path).await.unwrap();
        (manager, temp_dir)
    }

    #[tokio::test]
    async fn test_add_and_get_sound() {
        let (manager, temp_dir) = create_test_manager().await;

        // Create a temporary sound file
        let sound_path = temp_dir.path().join("test.mp3");
        File::create(&sound_path).unwrap();

        let sound = manager
            .add_sound("Test Sound".to_string(), sound_path.to_str().unwrap().to_string())
            .await
            .unwrap();

        assert_eq!(manager.count(), 1);

        let retrieved = manager.get_by_id(&sound.id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Sound");
    }

    #[tokio::test]
    async fn test_delete_sound() {
        let (manager, temp_dir) = create_test_manager().await;

        let sound_path = temp_dir.path().join("test.mp3");
        File::create(&sound_path).unwrap();

        let sound = manager
            .add_sound("Test".to_string(), sound_path.to_str().unwrap().to_string())
            .await
            .unwrap();

        manager.delete_sound(&sound.id).await.unwrap();
        assert_eq!(manager.count(), 0);
    }

    #[tokio::test]
    async fn test_rename_sound() {
        let (manager, temp_dir) = create_test_manager().await;

        let sound_path = temp_dir.path().join("test.mp3");
        File::create(&sound_path).unwrap();

        let sound = manager
            .add_sound("Test".to_string(), sound_path.to_str().unwrap().to_string())
            .await
            .unwrap();

        manager
            .rename_sound(&sound.id, "New Name".to_string())
            .await
            .unwrap();

        let retrieved = manager.get_by_id(&sound.id).unwrap();
        assert_eq!(retrieved.name, "New Name");
    }
}
