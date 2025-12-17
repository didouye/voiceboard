//! Ports layer - Interfaces/Traits for the hexagonal architecture
//!
//! Ports define the contracts between the domain and the outside world.
//! They are implemented by adapters.

mod audio_input;
mod audio_output;
mod file_decoder;
mod device_manager;

pub use audio_input::*;
pub use audio_output::*;
pub use file_decoder::*;
pub use device_manager::*;
