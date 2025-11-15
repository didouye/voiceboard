// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tauri::Manager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use voiceboard::{
    audio::{AudioConfig, AudioEngine},
    commands,
    config::ConfigManager,
    soundboard::SoundboardManager,
};

fn main() {
    // Initialize tracing/logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "voiceboard=debug,tauri=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tauri::Builder::default()
        .setup(|app| {
            // Get app data directory
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            // Ensure app data directory exists
            std::fs::create_dir_all(&app_dir).expect("Failed to create app data directory");

            // Initialize configuration
            let config_path = app_dir.join("config.json");
            let config_manager = ConfigManager::new(config_path).expect("Failed to initialize config manager");

            // Initialize soundboard manager
            let db_path = app_dir.join("voiceboard.db");
            let soundboard_manager = tauri::async_runtime::block_on(async {
                SoundboardManager::new(db_path)
                    .await
                    .expect("Failed to initialize soundboard manager")
            });

            // Initialize audio engine
            let audio_config = config_manager.get().audio;
            let audio_engine = AudioEngine::new(audio_config);

            // Store managers in Tauri state
            app.manage(Arc::new(soundboard_manager));
            app.manage(Arc::new(audio_engine));
            app.manage(Arc::new(config_manager));

            tracing::info!("Voiceboard application initialized successfully");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Soundboard commands
            commands::get_sounds,
            commands::add_sound,
            commands::delete_sound,
            commands::rename_sound,
            commands::update_sound_volume,
            commands::reorder_sounds,
            commands::filter_sounds,
            commands::get_sound_count,
            // Device commands
            commands::get_input_devices,
            commands::get_output_devices,
            commands::get_default_input_device,
            commands::get_default_output_device,
            commands::select_input_device,
            commands::select_output_device,
            // Audio commands
            commands::start_audio_engine,
            commands::stop_audio_engine,
            commands::play_sound,
            commands::stop_sound,
            commands::stop_all_sounds,
            commands::set_master_volume,
            commands::set_mic_volume,
            commands::set_effects_volume,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}
