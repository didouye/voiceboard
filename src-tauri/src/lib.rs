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
        // Binary manager
        check_binaries_status,
        // Updates
        check_for_update,
        // VB-Cable setup
        check_vb_cable_installed,
        check_virtual_driver,
        check_ytdlp_update,
        cleanup_orphaned_images,
        delete_pad_image,
        download_and_install_vb_cable,
        // Debug
        get_app_environment,
        // Device management
        get_audio_devices,
        get_debug_mode,
        // Image management
        get_images_dir,
        get_input_devices,
        get_install_id,
        // Mixer configuration
        get_mixer_config,
        get_noise_suppression,
        get_physical_output_devices,
        get_preview_state,
        get_sentry_dsn,
        // Settings
        get_settings,
        get_sounds_dir,
        get_virtual_output_devices,
        get_virtual_outputs_by_priority,
        get_voice_gate,
        hash_file,
        import_and_normalize_sound,
        import_multiple_sounds_with_hash,
        import_sound_with_hash,
        install_binaries,
        install_update,
        is_mixing,
        load_folders,
        load_multiple_sound_files,
        load_settings,
        // Sound playback
        load_sound_file,
        load_soundboard,
        migrate_sound_to_normalized,
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
        set_noise_suppression,
        set_output_device,
        set_preview_device,
        set_soundboard_volume,
        set_update_channel,
        set_voice_gate,
        // Mixing control
        start_mixing,
        stop_all_sounds,
        stop_mixing,
        stop_preview,
        stop_sound,
        toggle_channel_mute,
        update_ytdlp,
        // YouTube audio import
        youtube_cancel,
        youtube_download,
        youtube_trim_and_import,
    },
    shortcut_commands::{
        get_global_hotkeys_enabled, register_global_shortcut, set_global_hotkeys_enabled,
        unregister_all_shortcuts, unregister_global_shortcut, ShortcutRegistry,
    },
    AppState, PreviewEngine,
};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{Emitter, Manager};
use tauri_plugin_store::StoreExt;

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

            // Initialize Sentry Logs debug mode gate from persisted setting
            if let Ok(store) = app.store("debug.json") {
                if let Some(value) = store.get("debug_mode") {
                    if let Some(enabled) = value.as_bool() {
                        infrastructure::DEBUG_MODE_ENABLED
                            .store(enabled, std::sync::atomic::Ordering::Relaxed);
                    }
                }
            }

            // Tag Sentry events with a stable per-install id so a machine's events group together
            if let Some(install_id) = application::commands::get_or_create_install_id(app.handle())
            {
                infrastructure::set_install_id(install_id);
            }

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

            // Cleanup old YouTube temp files (>24h)
            if let Ok(app_data_dir) = app.path().app_data_dir() {
                let temp_dir = app_data_dir.join("temp").join("youtube");
                if temp_dir.exists() {
                    let now = std::time::SystemTime::now();
                    if let Ok(entries) = std::fs::read_dir(&temp_dir) {
                        for entry in entries.flatten() {
                            if let Ok(metadata) = entry.metadata() {
                                if let Ok(modified) = metadata.modified() {
                                    if let Ok(age) = now.duration_since(modified) {
                                        if age.as_secs() > 24 * 60 * 60 {
                                            let path = entry.path();
                                            if std::fs::remove_file(&path).is_ok() {
                                                tracing::info!(
                                                    "Cleaned up old YouTube temp file: {:?}",
                                                    path
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Non-blocking yt-dlp update check at launch (5s delay)
            // Use tauri::async_runtime::spawn (not tokio::spawn) because the
            // setup closure runs on the main thread which may not have the
            // tokio runtime context entered (causes panic on Windows).
            let update_app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                match application::binary_manager::check_ytdlp_update(&update_app_handle).await {
                    Ok(Some(new_version)) => {
                        tracing::info!(version = %new_version, "yt-dlp update available");
                        let _ = update_app_handle.emit("ytdlp-update-available", &new_version);
                    }
                    Ok(None) => {
                        tracing::debug!("yt-dlp is up to date");
                    }
                    Err(e) => {
                        tracing::debug!(error = %e, "yt-dlp update check failed (non-critical)");
                    }
                }
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
            import_and_normalize_sound,
            get_sounds_dir,
            migrate_sound_to_normalized,
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
            set_update_channel,
            // Debug
            get_debug_mode,
            set_debug_mode,
            get_sentry_dsn,
            get_app_environment,
            get_install_id,
            // Noise suppression
            get_noise_suppression,
            set_noise_suppression,
            // Voice gate (VAD auto-mute)
            get_voice_gate,
            set_voice_gate,
            // VB-Cable setup
            check_vb_cable_installed,
            download_and_install_vb_cable,
            // Shortcut management
            register_global_shortcut,
            unregister_global_shortcut,
            unregister_all_shortcuts,
            set_global_hotkeys_enabled,
            get_global_hotkeys_enabled,
            // YouTube audio import
            youtube_cancel,
            youtube_download,
            youtube_trim_and_import,
            // Binary manager
            check_binaries_status,
            install_binaries,
            check_ytdlp_update,
            update_ytdlp,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
