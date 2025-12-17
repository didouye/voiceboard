//! Application layer - Use cases and orchestration
//!
//! This layer coordinates the domain logic and adapters to implement
//! the application's use cases.

pub mod audio_engine;
pub mod commands;
mod services;
mod state;

pub use audio_engine::*;
pub use commands::*;
pub use services::*;
pub use state::*;
