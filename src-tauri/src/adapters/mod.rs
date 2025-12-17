//! Adapters layer - Concrete implementations of ports
//!
//! Adapters implement the port interfaces using specific technologies
//! (cpal, rodio, WASAPI, etc.)

mod cpal_input;
mod cpal_device_manager;
mod rodio_decoder;

pub use cpal_input::*;
pub use cpal_device_manager::*;
pub use rodio_decoder::*;

// Virtual device adapter will be platform-specific
#[cfg(target_os = "windows")]
mod windows_virtual_output;

#[cfg(target_os = "windows")]
pub use windows_virtual_output::*;
