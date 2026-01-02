//! Ports layer - Interfaces/Traits for the hexagonal architecture
//!
//! Ports define the contracts between the domain and the outside world.
//! They are implemented by adapters.

mod audio_input;
mod audio_output;
mod device_manager;
mod file_decoder;

pub use audio_input::*;
pub use audio_output::*;
pub use device_manager::*;
pub use file_decoder::*;
