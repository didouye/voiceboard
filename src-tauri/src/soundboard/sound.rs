/// Sound entity definition
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A sound in the soundboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sound {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Path to audio file
    pub file_path: String,
    /// Volume (0.0 to 1.0)
    #[serde(default = "default_volume")]
    pub volume: f32,
    /// Sort order (lower numbers appear first)
    #[serde(default)]
    pub sort_order: i32,
    /// Creation timestamp
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

fn default_volume() -> f32 {
    1.0
}

impl Sound {
    /// Create a new sound
    pub fn new(name: String, file_path: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            file_path,
            volume: 1.0,
            sort_order: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a sound with a specific ID (for database loading)
    pub fn with_id(
        id: String,
        name: String,
        file_path: String,
        volume: f32,
        sort_order: i32,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            file_path,
            volume,
            sort_order,
            created_at,
            updated_at,
        }
    }

    /// Update the name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    /// Update the volume
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        self.updated_at = Utc::now();
    }

    /// Update the sort order
    pub fn set_sort_order(&mut self, sort_order: i32) {
        self.sort_order = sort_order;
        self.updated_at = Utc::now();
    }

    /// Validate the sound
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Sound name cannot be empty".to_string());
        }

        if self.file_path.trim().is_empty() {
            return Err("Sound file path cannot be empty".to_string());
        }

        if !(0.0..=1.0).contains(&self.volume) {
            return Err("Sound volume must be between 0.0 and 1.0".to_string());
        }

        Ok(())
    }

    /// Check if the sound file exists
    pub fn file_exists(&self) -> bool {
        std::path::Path::new(&self.file_path).exists()
    }

    /// Get the file extension
    pub fn file_extension(&self) -> Option<String> {
        std::path::Path::new(&self.file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
    }

    /// Check if this sound format is supported
    pub fn is_supported_format(&self) -> bool {
        if let Some(ext) = self.file_extension() {
            matches!(ext.as_str(), "mp3" | "wav" | "ogg" | "flac" | "aac" | "m4a")
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_sound() {
        let sound = Sound::new("Test Sound".to_string(), "/path/to/sound.mp3".to_string());

        assert!(!sound.id.is_empty());
        assert_eq!(sound.name, "Test Sound");
        assert_eq!(sound.file_path, "/path/to/sound.mp3");
        assert_eq!(sound.volume, 1.0);
        assert_eq!(sound.sort_order, 0);
    }

    #[test]
    fn test_validate_sound() {
        let mut sound = Sound::new("Test".to_string(), "/path/to/sound.mp3".to_string());
        assert!(sound.validate().is_ok());

        sound.name = "".to_string();
        assert!(sound.validate().is_err());

        sound.name = "Test".to_string();
        sound.volume = 1.5;
        assert!(sound.validate().is_err());
    }

    #[test]
    fn test_file_extension() {
        let sound = Sound::new("Test".to_string(), "/path/to/sound.mp3".to_string());
        assert_eq!(sound.file_extension(), Some("mp3".to_string()));
        assert!(sound.is_supported_format());

        let sound2 = Sound::new("Test".to_string(), "/path/to/sound.xyz".to_string());
        assert_eq!(sound2.file_extension(), Some("xyz".to_string()));
        assert!(!sound2.is_supported_format());
    }

    #[test]
    fn test_update_sound() {
        let mut sound = Sound::new("Test".to_string(), "/path/to/sound.mp3".to_string());
        let original_updated_at = sound.updated_at;

        std::thread::sleep(std::time::Duration::from_millis(10));

        sound.set_name("New Name".to_string());
        assert_eq!(sound.name, "New Name");
        assert!(sound.updated_at > original_updated_at);
    }
}
