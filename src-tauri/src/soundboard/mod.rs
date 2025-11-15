/// Soundboard management module
///
/// This module handles:
/// - CRUD operations for sounds
/// - Sound persistence to database
/// - Sound organization (sorting, filtering)

pub mod manager;
pub mod sound;
pub mod storage;

// Re-export commonly used types
pub use manager::SoundboardManager;
pub use sound::Sound;
pub use storage::SoundboardStorage;
