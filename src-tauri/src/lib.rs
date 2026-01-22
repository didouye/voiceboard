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

pub mod adapters;
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod ports;

use crate::application::audio_engine::AudioEngineEvent;
use application::{
    commands::{
        add_audio_file_channel,
        // Channel management
        add_microphone_channel,
        // Updates
        check_for_update,
        // VB-Cable setup
        check_vb_cable_installed,
        check_virtual_driver,
        cleanup_orphaned_images,
        delete_pad_image,
        download_and_install_vb_cable,
        // Device management
        get_audio_devices,
        // Debug
        get_debug_mode,
        // Image management
        get_images_dir,
        get_input_devices,
        // Mixer configuration
        get_mixer_config,
        get_physical_output_devices,
        get_preview_state,
        get_sentry_dsn,
        // Settings
        get_settings,
        get_virtual_output_devices,
        get_virtual_outputs_by_priority,
        hash_file,
        import_multiple_sounds_with_hash,
        import_sound_with_hash,
        install_update,
        is_mixing,
        load_folders,
        load_multiple_sound_files,
        load_settings,
        // Sound playback
        load_sound_file,
        load_soundboard,
        play_sound,
        preview_sound,
        read_image_file,
        remove_channel,
        // Folder persistence
        save_folders,
        save_pad_image,
        save_settings,
        // Soundboard persistence
        save_soundboard,
        set_channel_volume,
        set_debug_mode,
        set_input_device,
        set_master_volume,
        set_mic_monitoring,
        set_mic_muted,
        set_mic_volume,
        set_output_device,
        set_preview_device,
        set_soundboard_volume,
        // Mixing control
        start_mixing,
        stop_all_sounds,
        stop_mixing,
        stop_preview,
        stop_sound,
        toggle_channel_mute,
    },
    shortcut_commands::{
        get_global_hotkeys_enabled, register_global_shortcut, set_global_hotkeys_enabled,
        unregister_all_shortcuts, unregister_global_shortcut, ShortcutRegistry,
    },
    AppState, PreviewEngine,
};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{Emitter, Manager};

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
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            let state = AppState::new();
            app.manage(state);
            app.manage(ShortcutRegistry::default());

            // Create application menu with Debug toggle
            let toggle_debug =
                MenuItem::with_id(app, "toggle_debug", "Toggle Debug Mode", true, None::<&str>)?;
            let app_submenu = Submenu::with_items(app, "Voiceboard", true, &[&toggle_debug])?;

            // Create Edit menu with standard items (required for Cmd+V on macOS)
            let edit_submenu = Submenu::with_items(
                app,
                "Edit",
                true,
                &[
                    &PredefinedMenuItem::undo(app, None)?,
                    &PredefinedMenuItem::redo(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::cut(app, None)?,
                    &PredefinedMenuItem::copy(app, None)?,
                    &PredefinedMenuItem::paste(app, None)?,
                    &PredefinedMenuItem::select_all(app, None)?,
                ],
            )?;

            let menu = Menu::with_items(app, &[&app_submenu, &edit_submenu])?;
            app.set_menu(menu)?;

            // Initialize preview engine with app handle
            let app_handle = app.handle().clone();
            let state_ref = app.state::<AppState>();
            let preview_engine = PreviewEngine::new(app_handle.clone());
            {
                let mut preview = state_ref.preview_engine.blocking_lock();
                *preview = Some(preview_engine);
            }

            // Start audio engine event forwarding
            let engine_for_events = state_ref.audio_engine.clone();
            std::thread::spawn(move || loop {
                if let Ok(engine) = engine_for_events.try_lock() {
                    while let Some(event) = engine.try_recv_event() {
                        match event {
                            AudioEngineEvent::LevelUpdate {
                                input_rms,
                                input_peak,
                                output_rms,
                                output_peak,
                                monitoring_rms,
                            } => {
                                let _ = app_handle.emit(
                                    "audio-levels",
                                    serde_json::json!({
                                        "inputRms": input_rms,
                                        "inputPeak": input_peak,
                                        "outputRms": output_rms,
                                        "outputPeak": output_peak,
                                        "monitoringRms": monitoring_rms,
                                    }),
                                );
                            }
                            AudioEngineEvent::Started => {
                                tracing::info!("[AudioEngine] Engine started successfully");
                                let _ = app_handle.emit(
                                    "audio-engine-log",
                                    serde_json::json!({
                                        "level": "info",
                                        "message": "Audio engine started successfully"
                                    }),
                                );
                            }
                            AudioEngineEvent::Stopped => {
                                tracing::info!("[AudioEngine] Engine stopped");
                                let _ = app_handle.emit(
                                    "audio-engine-log",
                                    serde_json::json!({
                                        "level": "info",
                                        "message": "Audio engine stopped"
                                    }),
                                );
                            }
                            AudioEngineEvent::Error(msg) => {
                                tracing::error!("[AudioEngine] Error: {}", msg);
                                let _ = app_handle.emit(
                                    "audio-engine-log",
                                    serde_json::json!({
                                        "level": "error",
                                        "message": format!("Audio engine error: {}", msg)
                                    }),
                                );
                            }
                            AudioEngineEvent::Info(msg) => {
                                tracing::info!("[AudioEngine] {}", msg);
                                let _ = app_handle.emit(
                                    "audio-engine-log",
                                    serde_json::json!({
                                        "level": "info",
                                        "message": msg
                                    }),
                                );
                            }
                            AudioEngineEvent::SoundFinished { id } => {
                                tracing::debug!("[AudioEngine] Sound finished: {}", id);
                                let _ = app_handle
                                    .emit("sound-finished", serde_json::json!({ "id": id }));
                            }
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(16));
            });

            Ok(())
        })
        .on_menu_event(|app, event| {
            if event.id() == "toggle_debug" {
                // Toggle debug mode
                let current = get_debug_mode(app.clone());
                let new_value = !current;
                if let Err(e) = set_debug_mode(app.clone(), new_value) {
                    tracing::error!(error = %e, "Failed to toggle debug mode");
                } else {
                    // Emit event to frontend to update UI
                    let _ = app.emit("debug-mode-changed", new_value);
                    tracing::info!(enabled = new_value, "Debug mode toggled via menu");
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Device management
            get_audio_devices,
            get_input_devices,
            get_virtual_output_devices,
            get_virtual_outputs_by_priority,
            get_physical_output_devices,
            check_virtual_driver,
            // Settings
            get_settings,
            save_settings,
            load_settings,
            set_input_device,
            set_output_device,
            set_preview_device,
            set_mic_monitoring,
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
            load_multiple_sound_files,
            // Sound hashing
            hash_file,
            import_sound_with_hash,
            import_multiple_sounds_with_hash,
            play_sound,
            stop_sound,
            stop_all_sounds,
            preview_sound,
            stop_preview,
            get_preview_state,
            set_mic_volume,
            set_mic_muted,
            set_soundboard_volume,
            // Soundboard persistence
            save_soundboard,
            load_soundboard,
            // Folder persistence
            save_folders,
            load_folders,
            // Image management
            get_images_dir,
            save_pad_image,
            read_image_file,
            delete_pad_image,
            cleanup_orphaned_images,
            // Updates
            check_for_update,
            install_update,
            // Debug
            get_debug_mode,
            set_debug_mode,
            get_sentry_dsn,
            // VB-Cable setup
            check_vb_cable_installed,
            download_and_install_vb_cable,
            // Shortcut management
            register_global_shortcut,
            unregister_global_shortcut,
            unregister_all_shortcuts,
            set_global_hotkeys_enabled,
            get_global_hotkeys_enabled,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
