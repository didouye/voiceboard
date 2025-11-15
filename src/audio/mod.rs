mod capture;
mod renderer;

pub use capture::AudioCapture;
pub use renderer::AudioRenderer;

use windows::Win32::System::Com::{
    CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED,
};

/// Initialize COM for audio operations
pub fn initialize_com() -> windows::core::Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
    }
    Ok(())
}

/// Cleanup COM
pub fn uninitialize_com() {
    unsafe {
        CoUninitialize();
    }
}
