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

use tauri::{Manager, Emitter};
use crate::application::audio_engine::AudioEngineEvent;
use application::{
    commands::{
        // Device management
        get_audio_devices, get_input_devices, get_virtual_output_devices, check_virtual_driver,
        // Settings
        get_settings, save_settings, load_settings, set_input_device, set_output_device, set_preview_device,
        // Mixer configuration
        get_mixer_config, set_master_volume,
        // Channel management
        add_microphone_channel, add_audio_file_channel, remove_channel,
        set_channel_volume, toggle_channel_mute,
        // Mixing control
        start_mixing, stop_mixing, is_mixing,
        // Sound playback
        load_sound_file, play_sound, stop_sound, preview_sound, stop_preview, get_preview_state,
        set_mic_volume, set_mic_muted,
        // Soundboard persistence
        save_soundboard, load_soundboard,
        // Updates
        check_for_update, install_update,
    },
    AppState, PreviewEngine,
};

/// Run the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize Sentry first (returns guard that must be kept alive)
    let _sentry_guard = infrastructure::init_sentry();

    // Initialize logging (with Sentry integration if enabled)
    infrastructure::init_logging();

    tracing::info!("Starting Voiceboard application");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let state = AppState::new();
            app.manage(state);

            // Initialize preview engine with app handle
            let app_handle = app.handle().clone();
            let state_ref = app.state::<AppState>();
            let preview_engine = PreviewEngine::new(app_handle.clone());
            {
                let mut preview = state_ref.preview_engine.blocking_lock();
                *preview = Some(preview_engine);
            }

            // Start level event forwarding
            let engine_for_levels = state_ref.audio_engine.clone();
            std::thread::spawn(move || {
                loop {
                    if let Ok(engine) = engine_for_levels.try_lock() {
                        while let Some(event) = engine.try_recv_event() {
                            match event {
                                AudioEngineEvent::LevelUpdate { input_rms, input_peak, output_rms, output_peak } => {
                                    let _ = app_handle.emit("audio-levels", serde_json::json!({
                                        "inputRms": input_rms,
                                        "inputPeak": input_peak,
                                        "outputRms": output_rms,
                                        "outputPeak": output_peak,
                                    }));
                                }
                                _ => {}
                            }
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(16));
                }
            });

            Ok(())
        })
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
            set_preview_device,
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
            // Sound playback
            load_sound_file,
            play_sound,
            stop_sound,
            preview_sound,
            stop_preview,
            get_preview_state,
            set_mic_volume,
            set_mic_muted,
            // Soundboard persistence
            save_soundboard,
            load_soundboard,
            // Updates
            check_for_update,
            install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
