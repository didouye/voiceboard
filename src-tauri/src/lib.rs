//! Voiceboard - Virtual Microphone Mixer
//!
//! A Tauri application that creates a virtual microphone output,
//! mixing audio from a real microphone with audio files (MP3, OGG).
//!
//! # Architecture
//!
//! This application follows the Hexagonal Architecture (Ports & Adapters):
//!
//! - **Domain**: Pure business logic (audio processing, mixing)
//! - **Ports**: Interfaces defining contracts (traits)
//! - **Adapters**: Concrete implementations (cpal, rodio, WASAPI)
//! - **Application**: Use cases and orchestration
//! - **Infrastructure**: Cross-cutting concerns (logging, config)

pub mod domain;
pub mod ports;
pub mod adapters;
pub mod application;
pub mod infrastructure;

use application::{
    commands::{
        // Device management
        get_audio_devices, get_input_devices, get_virtual_output_devices, check_virtual_driver,
        // Settings
        get_settings, save_settings, load_settings, set_input_device, set_output_device,
        // Mixer configuration
        get_mixer_config, set_master_volume,
        // Channel management
        add_microphone_channel, add_audio_file_channel, remove_channel,
        set_channel_volume, toggle_channel_mute,
        // Mixing control
        start_mixing, stop_mixing, is_mixing,
    },
    AppState,
};

/// Run the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    infrastructure::init_logging();

    tracing::info!("Starting Voiceboard application");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            // Device management
            get_audio_devices,
            get_input_devices,
            get_virtual_output_devices,
            check_virtual_driver,
            // Settings
            get_settings,
            save_settings,
            load_settings,
            set_input_device,
            set_output_device,
            // Mixer configuration
            get_mixer_config,
            set_master_volume,
            // Channel management
            add_microphone_channel,
            add_audio_file_channel,
            remove_channel,
            set_channel_volume,
            toggle_channel_mute,
            // Mixing control
            start_mixing,
            stop_mixing,
            is_mixing,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
