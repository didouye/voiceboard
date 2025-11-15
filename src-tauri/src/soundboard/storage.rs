/// SQLite storage for soundboard data
use super::sound::Sound;
use crate::error::{Result, VoiceboardError};
use chrono::{DateTime, Utc};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use std::path::Path;
use std::str::FromStr;
use tracing::{debug, info};

/// Soundboard storage using SQLite
pub struct SoundboardStorage {
    pool: SqlitePool,
}

impl SoundboardStorage {
    /// Create a new soundboard storage
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db_path = db_path.as_ref();
        info!("Opening soundboard database: {}", db_path.display());

        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let connection_string = format!("sqlite:{}", db_path.display());

        let options = SqliteConnectOptions::from_str(&connection_string)
            .map_err(|e| VoiceboardError::Database(e.to_string()))?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        let storage = Self { pool };

        // Initialize database schema
        storage.init_schema().await?;

        Ok(storage)
    }

    /// Initialize database schema
    async fn init_schema(&self) -> Result<()> {
        debug!("Initializing database schema");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sounds (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                file_path TEXT NOT NULL,
                volume REAL NOT NULL DEFAULT 1.0,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_sounds_sort_order
            ON sounds(sort_order)
            "#,
        )
        .execute(&self.pool)
        .await?;

        debug!("Database schema initialized successfully");
        Ok(())
    }

    /// Get all sounds
    pub async fn get_all(&self) -> Result<Vec<Sound>> {
        debug!("Fetching all sounds from database");

        let rows = sqlx::query(
            r#"
            SELECT id, name, file_path, volume, sort_order, created_at, updated_at
            FROM sounds
            ORDER BY sort_order ASC, created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let sounds: Vec<Sound> = rows
            .iter()
            .map(|row| {
                let created_timestamp: i64 = row.get("created_at");
                let updated_timestamp: i64 = row.get("updated_at");

                Sound::with_id(
                    row.get("id"),
                    row.get("name"),
                    row.get("file_path"),
                    row.get("volume"),
                    row.get("sort_order"),
                    DateTime::from_timestamp(created_timestamp, 0).unwrap_or_else(Utc::now),
                    DateTime::from_timestamp(updated_timestamp, 0).unwrap_or_else(Utc::now),
                )
            })
            .collect();

        debug!("Fetched {} sounds", sounds.len());
        Ok(sounds)
    }

    /// Get a sound by ID
    pub async fn get_by_id(&self, id: &str) -> Result<Option<Sound>> {
        debug!("Fetching sound by ID: {}", id);

        let row = sqlx::query(
            r#"
            SELECT id, name, file_path, volume, sort_order, created_at, updated_at
            FROM sounds
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let created_timestamp: i64 = row.get("created_at");
            let updated_timestamp: i64 = row.get("updated_at");

            Ok(Some(Sound::with_id(
                row.get("id"),
                row.get("name"),
                row.get("file_path"),
                row.get("volume"),
                row.get("sort_order"),
                DateTime::from_timestamp(created_timestamp, 0).unwrap_or_else(Utc::now),
                DateTime::from_timestamp(updated_timestamp, 0).unwrap_or_else(Utc::now),
            )))
        } else {
            Ok(None)
        }
    }

    /// Insert a new sound
    pub async fn insert(&self, sound: &Sound) -> Result<()> {
        debug!("Inserting sound: {} ({})", sound.name, sound.id);

        sqlx::query(
            r#"
            INSERT INTO sounds (id, name, file_path, volume, sort_order, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&sound.id)
        .bind(&sound.name)
        .bind(&sound.file_path)
        .bind(sound.volume)
        .bind(sound.sort_order)
        .bind(sound.created_at.timestamp())
        .bind(sound.updated_at.timestamp())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update an existing sound
    pub async fn update(&self, sound: &Sound) -> Result<()> {
        debug!("Updating sound: {} ({})", sound.name, sound.id);

        let result = sqlx::query(
            r#"
            UPDATE sounds
            SET name = ?, file_path = ?, volume = ?, sort_order = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&sound.name)
        .bind(&sound.file_path)
        .bind(sound.volume)
        .bind(sound.sort_order)
        .bind(sound.updated_at.timestamp())
        .bind(&sound.id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(VoiceboardError::SoundNotFound(sound.id.clone()));
        }

        Ok(())
    }

    /// Delete a sound by ID
    pub async fn delete(&self, id: &str) -> Result<()> {
        debug!("Deleting sound: {}", id);

        let result = sqlx::query("DELETE FROM sounds WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(VoiceboardError::SoundNotFound(id.to_string()));
        }

        Ok(())
    }

    /// Update sort orders for multiple sounds
    pub async fn update_sort_orders(&self, sound_ids: &[String]) -> Result<()> {
        debug!("Updating sort orders for {} sounds", sound_ids.len());

        let mut tx = self.pool.begin().await?;

        for (index, sound_id) in sound_ids.iter().enumerate() {
            sqlx::query("UPDATE sounds SET sort_order = ?, updated_at = ? WHERE id = ?")
                .bind(index as i32)
                .bind(Utc::now().timestamp())
                .bind(sound_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    /// Delete all sounds
    pub async fn delete_all(&self) -> Result<()> {
        debug!("Deleting all sounds");

        sqlx::query("DELETE FROM sounds")
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_storage() -> (SoundboardStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = SoundboardStorage::new(&db_path).await.unwrap();
        (storage, temp_dir)
    }

    #[tokio::test]
    async fn test_insert_and_get() {
        let (storage, _temp_dir) = create_test_storage().await;

        let sound = Sound::new("Test Sound".to_string(), "/path/to/sound.mp3".to_string());
        storage.insert(&sound).await.unwrap();

        let retrieved = storage.get_by_id(&sound.id).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, sound.id);
        assert_eq!(retrieved.name, sound.name);
    }

    #[tokio::test]
    async fn test_update() {
        let (storage, _temp_dir) = create_test_storage().await;

        let mut sound = Sound::new("Test Sound".to_string(), "/path/to/sound.mp3".to_string());
        storage.insert(&sound).await.unwrap();

        sound.set_name("Updated Name".to_string());
        storage.update(&sound).await.unwrap();

        let retrieved = storage.get_by_id(&sound.id).await.unwrap().unwrap();
        assert_eq!(retrieved.name, "Updated Name");
    }

    #[tokio::test]
    async fn test_delete() {
        let (storage, _temp_dir) = create_test_storage().await;

        let sound = Sound::new("Test Sound".to_string(), "/path/to/sound.mp3".to_string());
        storage.insert(&sound).await.unwrap();

        storage.delete(&sound.id).await.unwrap();

        let retrieved = storage.get_by_id(&sound.id).await.unwrap();
        assert!(retrieved.is_none());
    }
}
