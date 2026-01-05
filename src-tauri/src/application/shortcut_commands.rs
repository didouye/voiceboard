//! Tauri commands for global keyboard shortcut management

use serde_json::json;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// State to track registered shortcuts (shortcut string -> pad_id)
pub struct ShortcutRegistry {
    pub shortcuts: Mutex<HashMap<String, String>>,
    pub enabled: Mutex<bool>,
}

impl Default for ShortcutRegistry {
    fn default() -> Self {
        Self {
            shortcuts: Mutex::new(HashMap::new()),
            enabled: Mutex::new(true),
        }
    }
}

/// Register a global shortcut for a pad
#[tauri::command]
pub fn register_global_shortcut(
    app: AppHandle,
    registry: State<ShortcutRegistry>,
    pad_id: String,
    shortcut: String,
) -> Result<(), String> {
    // Check if enabled
    let enabled = registry.enabled.lock().map_err(|e| e.to_string())?;
    if !*enabled {
        // Still store in registry but don't activate
        let mut shortcuts = registry.shortcuts.lock().map_err(|e| e.to_string())?;
        shortcuts.insert(shortcut, pad_id);
        return Ok(());
    }
    drop(enabled);

    // Parse and register the shortcut
    let parsed: Shortcut = shortcut
        .parse()
        .map_err(|e| format!("Invalid shortcut '{}': {}", shortcut, e))?;

    let shortcut_clone = shortcut.clone();
    let pad_id_clone = pad_id.clone();
    let app_clone = app.clone();

    app.global_shortcut()
        .on_shortcut(parsed, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                tracing::info!(
                    "[Shortcuts] Global shortcut triggered: {} -> {}",
                    shortcut_clone,
                    pad_id_clone
                );
                let _ = app_clone.emit(
                    "global-shortcut-triggered",
                    json!({
                        "padId": pad_id_clone,
                        "shortcut": shortcut_clone
                    }),
                );
            }
        })
        .map_err(|e| format!("Failed to register shortcut: {}", e))?;

    // Store in registry
    let mut shortcuts = registry.shortcuts.lock().map_err(|e| e.to_string())?;
    shortcuts.insert(shortcut, pad_id);

    Ok(())
}

/// Unregister a global shortcut
#[tauri::command]
pub fn unregister_global_shortcut(
    app: AppHandle,
    registry: State<ShortcutRegistry>,
    shortcut: String,
) -> Result<(), String> {
    // Parse and unregister
    let parsed: Shortcut = shortcut
        .parse()
        .map_err(|e| format!("Invalid shortcut '{}': {}", shortcut, e))?;

    let _ = app.global_shortcut().unregister(parsed);

    // Remove from registry
    let mut shortcuts = registry.shortcuts.lock().map_err(|e| e.to_string())?;
    shortcuts.remove(&shortcut);

    Ok(())
}

/// Unregister all global shortcuts
#[tauri::command]
pub fn unregister_all_shortcuts(
    app: AppHandle,
    registry: State<ShortcutRegistry>,
) -> Result<(), String> {
    let _ = app.global_shortcut().unregister_all();

    let mut shortcuts = registry.shortcuts.lock().map_err(|e| e.to_string())?;
    shortcuts.clear();

    tracing::info!("[Shortcuts] All global shortcuts unregistered");
    Ok(())
}

/// Enable or disable global hotkeys
#[tauri::command]
pub fn set_global_hotkeys_enabled(
    app: AppHandle,
    registry: State<ShortcutRegistry>,
    enabled: bool,
) -> Result<(), String> {
    let mut enabled_state = registry.enabled.lock().map_err(|e| e.to_string())?;
    let was_enabled = *enabled_state;
    *enabled_state = enabled;
    drop(enabled_state);

    if enabled && !was_enabled {
        // Re-register all shortcuts from registry
        let shortcuts = registry.shortcuts.lock().map_err(|e| e.to_string())?;
        for (shortcut, pad_id) in shortcuts.iter() {
            if let Ok(parsed) = shortcut.parse::<Shortcut>() {
                let shortcut_clone = shortcut.clone();
                let pad_id_clone = pad_id.clone();
                let app_clone = app.clone();

                let _ = app
                    .global_shortcut()
                    .on_shortcut(parsed, move |_app, _shortcut, event| {
                        if event.state == ShortcutState::Pressed {
                            let _ = app_clone.emit(
                                "global-shortcut-triggered",
                                json!({
                                    "padId": pad_id_clone,
                                    "shortcut": shortcut_clone
                                }),
                            );
                        }
                    });
            }
        }
        tracing::info!("[Shortcuts] Global hotkeys enabled");
    } else if !enabled && was_enabled {
        // Unregister all but keep in registry
        let _ = app.global_shortcut().unregister_all();
        tracing::info!("[Shortcuts] Global hotkeys disabled");
    }

    Ok(())
}

/// Get current global hotkeys enabled state
#[tauri::command]
pub fn get_global_hotkeys_enabled(registry: State<ShortcutRegistry>) -> Result<bool, String> {
    let enabled = registry.enabled.lock().map_err(|e| e.to_string())?;
    Ok(*enabled)
}
