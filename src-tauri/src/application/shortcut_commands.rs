//! Tauri commands for global keyboard shortcut management

use serde_json::json;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// State to track registered shortcuts (shortcut string -> sound_id)
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

/// Register a global shortcut for a sound
#[tauri::command]
pub fn register_global_shortcut(
    app: AppHandle,
    registry: State<ShortcutRegistry>,
    sound_id: String,
    shortcut: String,
) -> Result<(), String> {
    // Check if enabled
    let enabled = registry.enabled.lock().map_err(|e| e.to_string())?;
    if !*enabled {
        // Still store in registry but don't activate
        let mut shortcuts = registry.shortcuts.lock().map_err(|e| e.to_string())?;
        shortcuts.insert(shortcut, sound_id);
        return Ok(());
    }
    drop(enabled);

    // Parse and register the shortcut
    let parsed: Shortcut = shortcut
        .parse()
        .map_err(|e| format!("Invalid shortcut '{}': {}", shortcut, e))?;

    let shortcut_clone = shortcut.clone();
    let sound_id_clone = sound_id.clone();
    let app_clone = app.clone();

    app.global_shortcut()
        .on_shortcut(parsed, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                tracing::info!(
                    "[Shortcuts] Global shortcut triggered: {} -> {}",
                    shortcut_clone,
                    sound_id_clone
                );
                let _ = app_clone.emit(
                    "global-shortcut-triggered",
                    json!({
                        "soundId": sound_id_clone,
                        "shortcut": shortcut_clone
                    }),
                );
            }
        })
        .map_err(|e| format!("Failed to register shortcut: {}", e))?;

    // Store in registry
    let mut shortcuts = registry.shortcuts.lock().map_err(|e| e.to_string())?;
    shortcuts.insert(shortcut, sound_id);

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
        for (shortcut, sound_id) in shortcuts.iter() {
            if let Ok(parsed) = shortcut.parse::<Shortcut>() {
                let shortcut_clone = shortcut.clone();
                let sound_id_clone = sound_id.clone();
                let app_clone = app.clone();

                let _ = app
                    .global_shortcut()
                    .on_shortcut(parsed, move |_app, _shortcut, event| {
                        if event.state == ShortcutState::Pressed {
                            let _ = app_clone.emit(
                                "global-shortcut-triggered",
                                json!({
                                    "soundId": sound_id_clone,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_registry_default() {
        let registry = ShortcutRegistry::default();

        // Default enabled state is true
        let enabled = registry.enabled.lock().unwrap();
        assert!(*enabled);
        drop(enabled);

        // Default shortcuts map is empty
        let shortcuts = registry.shortcuts.lock().unwrap();
        assert!(shortcuts.is_empty());
    }

    #[test]
    fn test_shortcut_registry_store_shortcut() {
        let registry = ShortcutRegistry::default();

        // Store a shortcut
        {
            let mut shortcuts = registry.shortcuts.lock().unwrap();
            shortcuts.insert("Ctrl+1".to_string(), "pad-0".to_string());
        }

        // Verify it's stored
        let shortcuts = registry.shortcuts.lock().unwrap();
        assert_eq!(shortcuts.get("Ctrl+1"), Some(&"pad-0".to_string()));
    }

    #[test]
    fn test_shortcut_registry_store_multiple_shortcuts() {
        let registry = ShortcutRegistry::default();

        {
            let mut shortcuts = registry.shortcuts.lock().unwrap();
            shortcuts.insert("Ctrl+1".to_string(), "pad-0".to_string());
            shortcuts.insert("Alt+Shift+A".to_string(), "pad-1".to_string());
            shortcuts.insert("F1".to_string(), "pad-2".to_string());
        }

        let shortcuts = registry.shortcuts.lock().unwrap();
        assert_eq!(shortcuts.len(), 3);
        assert_eq!(shortcuts.get("Ctrl+1"), Some(&"pad-0".to_string()));
        assert_eq!(shortcuts.get("Alt+Shift+A"), Some(&"pad-1".to_string()));
        assert_eq!(shortcuts.get("F1"), Some(&"pad-2".to_string()));
    }

    #[test]
    fn test_shortcut_registry_remove_shortcut() {
        let registry = ShortcutRegistry::default();

        // Store then remove
        {
            let mut shortcuts = registry.shortcuts.lock().unwrap();
            shortcuts.insert("Ctrl+1".to_string(), "pad-0".to_string());
            shortcuts.remove("Ctrl+1");
        }

        let shortcuts = registry.shortcuts.lock().unwrap();
        assert!(shortcuts.is_empty());
    }

    #[test]
    fn test_shortcut_registry_clear_all() {
        let registry = ShortcutRegistry::default();

        {
            let mut shortcuts = registry.shortcuts.lock().unwrap();
            shortcuts.insert("Ctrl+1".to_string(), "pad-0".to_string());
            shortcuts.insert("Ctrl+2".to_string(), "pad-1".to_string());
            shortcuts.clear();
        }

        let shortcuts = registry.shortcuts.lock().unwrap();
        assert!(shortcuts.is_empty());
    }

    #[test]
    fn test_shortcut_registry_toggle_enabled() {
        let registry = ShortcutRegistry::default();

        // Initially enabled
        assert!(*registry.enabled.lock().unwrap());

        // Disable
        {
            let mut enabled = registry.enabled.lock().unwrap();
            *enabled = false;
        }
        assert!(!*registry.enabled.lock().unwrap());

        // Re-enable
        {
            let mut enabled = registry.enabled.lock().unwrap();
            *enabled = true;
        }
        assert!(*registry.enabled.lock().unwrap());
    }

    #[test]
    fn test_shortcut_registry_overwrite_existing() {
        let registry = ShortcutRegistry::default();

        {
            let mut shortcuts = registry.shortcuts.lock().unwrap();
            shortcuts.insert("Ctrl+1".to_string(), "pad-0".to_string());
            // Overwrite with different pad
            shortcuts.insert("Ctrl+1".to_string(), "pad-5".to_string());
        }

        let shortcuts = registry.shortcuts.lock().unwrap();
        assert_eq!(shortcuts.len(), 1);
        assert_eq!(shortcuts.get("Ctrl+1"), Some(&"pad-5".to_string()));
    }

    #[test]
    fn test_shortcut_registry_shortcuts_persist_when_disabled() {
        let registry = ShortcutRegistry::default();

        // Add shortcuts
        {
            let mut shortcuts = registry.shortcuts.lock().unwrap();
            shortcuts.insert("Ctrl+1".to_string(), "pad-0".to_string());
        }

        // Disable
        {
            let mut enabled = registry.enabled.lock().unwrap();
            *enabled = false;
        }

        // Shortcuts should still be in registry
        let shortcuts = registry.shortcuts.lock().unwrap();
        assert_eq!(shortcuts.len(), 1);
        assert_eq!(shortcuts.get("Ctrl+1"), Some(&"pad-0".to_string()));
    }
}
