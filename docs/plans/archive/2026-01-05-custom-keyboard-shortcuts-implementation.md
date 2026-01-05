# Custom Keyboard Shortcuts - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add configurable keyboard shortcuts for soundboard pads with modifier key support and global hotkeys.

**Architecture:** Frontend ShortcutService manages registry and conflict detection. Backend uses tauri-plugin-global-shortcut to register system-wide hotkeys. Shortcuts are stored in the existing `hotkey` field of SoundPad and persisted in soundboard.json.

**Tech Stack:** Angular 19, Tauri v2, tauri-plugin-global-shortcut, Rust

---

## Task 1: Add tauri-plugin-global-shortcut dependency

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/tauri.conf.json`

**Step 1: Add dependency to Cargo.toml**

In `src-tauri/Cargo.toml`, add after line 24 (after tauri-plugin-process):

```toml
tauri-plugin-global-shortcut = "2"     # Global keyboard shortcuts
```

**Step 2: Add plugin capabilities to tauri.conf.json**

In `src-tauri/tauri.conf.json`, add after the "updater" section in plugins (around line 44):

```json
    "global-shortcut": {
      "all": true
    }
```

**Step 3: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Compilation succeeds

**Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore: add tauri-plugin-global-shortcut dependency"
```

---

## Task 2: Create shortcut model and utilities (Frontend)

**Files:**
- Create: `src/app/core/models/shortcut.model.ts`
- Modify: `src/app/core/models/index.ts`

**Step 1: Create shortcut model file**

Create `src/app/core/models/shortcut.model.ts`:

```typescript
/**
 * Represents a keyboard shortcut with optional modifier keys
 */
export interface KeyboardShortcut {
  key: string;      // "1", "A", "F1", "Space", etc.
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
  meta: boolean;    // Cmd on macOS, Win on Windows
}

/**
 * Parse a shortcut string into a KeyboardShortcut object
 * @example parseShortcut("Ctrl+Shift+1") => { key: "1", ctrl: true, shift: true, alt: false, meta: false }
 */
export function parseShortcut(str: string): KeyboardShortcut {
  const parts = str.split('+');
  const key = parts[parts.length - 1];

  return {
    key,
    ctrl: parts.some(p => p.toLowerCase() === 'ctrl'),
    alt: parts.some(p => p.toLowerCase() === 'alt'),
    shift: parts.some(p => p.toLowerCase() === 'shift'),
    meta: parts.some(p => p.toLowerCase() === 'meta' || p.toLowerCase() === 'cmd'),
  };
}

/**
 * Format a KeyboardShortcut object into a display string
 * @example formatShortcut({ key: "1", ctrl: true, shift: true, ... }) => "Ctrl+Shift+1"
 */
export function formatShortcut(shortcut: KeyboardShortcut): string {
  const parts: string[] = [];

  if (shortcut.ctrl) parts.push('Ctrl');
  if (shortcut.alt) parts.push('Alt');
  if (shortcut.shift) parts.push('Shift');
  if (shortcut.meta) parts.push('Meta');
  parts.push(shortcut.key);

  return parts.join('+');
}

/**
 * Create a KeyboardShortcut from a KeyboardEvent
 */
export function shortcutFromEvent(event: KeyboardEvent): KeyboardShortcut {
  // Normalize key names
  let key = event.key;

  // Handle special keys
  if (key === ' ') key = 'Space';
  if (key.length === 1) key = key.toUpperCase();

  return {
    key,
    ctrl: event.ctrlKey,
    alt: event.altKey,
    shift: event.shiftKey,
    meta: event.metaKey,
  };
}

/**
 * Check if a KeyboardEvent matches a shortcut string
 */
export function eventMatchesShortcut(event: KeyboardEvent, shortcutStr: string): boolean {
  const shortcut = parseShortcut(shortcutStr);

  // Normalize the event key for comparison
  let eventKey = event.key;
  if (eventKey === ' ') eventKey = 'Space';
  if (eventKey.length === 1) eventKey = eventKey.toUpperCase();

  return (
    eventKey === shortcut.key &&
    event.ctrlKey === shortcut.ctrl &&
    event.altKey === shortcut.alt &&
    event.shiftKey === shortcut.shift &&
    event.metaKey === shortcut.meta
  );
}

/**
 * Check if a key is a modifier key (not a valid shortcut by itself)
 */
export function isModifierKey(key: string): boolean {
  return ['Control', 'Alt', 'Shift', 'Meta', 'Command', 'Cmd'].includes(key);
}
```

**Step 2: Export from models index**

In `src/app/core/models/index.ts`, add:

```typescript
export * from './shortcut.model';
```

**Step 3: Commit**

```bash
git add src/app/core/models/shortcut.model.ts src/app/core/models/index.ts
git commit -m "feat(shortcuts): add shortcut model and utility functions"
```

---

## Task 3: Add globalHotkeysEnabled to settings model

**Files:**
- Modify: `src/app/core/models/audio-device.model.ts`
- Modify: `src-tauri/src/domain/settings.rs`

**Step 1: Update frontend AudioSettings interface**

In `src/app/core/models/audio-device.model.ts`, add to `AudioSettings` interface (around line 36):

```typescript
export interface AudioSettings {
  inputDeviceId: string | null;
  outputDeviceId: string | null;
  previewDeviceId: string | null;
  masterVolume: number;
  sampleRate: number;
  bufferSize: number;
  micMonitoring: boolean;
  globalHotkeysEnabled: boolean;  // Add this line
}
```

**Step 2: Update backend AudioSettings struct**

In `src-tauri/src/domain/settings.rs`, add to `AudioSettings` struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub input_device_id: Option<String>,
    pub output_device_id: Option<String>,
    pub preview_device_id: Option<String>,
    pub master_volume: f32,
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub mic_monitoring: bool,
    #[serde(default = "default_global_hotkeys")]
    pub global_hotkeys_enabled: bool,  // Add this line
}

fn default_global_hotkeys() -> bool {
    true
}
```

Also update the `Default` impl:

```rust
impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            input_device_id: None,
            output_device_id: None,
            preview_device_id: None,
            master_volume: 1.0,
            sample_rate: 48000,
            buffer_size: 1024,
            mic_monitoring: false,
            global_hotkeys_enabled: true,  // Add this line
        }
    }
}
```

**Step 3: Run tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/app/core/models/audio-device.model.ts src-tauri/src/domain/settings.rs
git commit -m "feat(settings): add globalHotkeysEnabled setting (default: true)"
```

---

## Task 4: Initialize global shortcut plugin in Tauri

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Step 1: Add plugin initialization**

In `src-tauri/src/lib.rs`, add the plugin after tauri_plugin_process::init() (around line 99):

```rust
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
```

**Step 2: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Compilation succeeds

**Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(shortcuts): initialize global shortcut plugin"
```

---

## Task 5: Create shortcut Tauri commands

**Files:**
- Create: `src-tauri/src/application/shortcut_commands.rs`
- Modify: `src-tauri/src/application/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Create shortcut_commands.rs**

Create `src-tauri/src/application/shortcut_commands.rs`:

```rust
//! Tauri commands for global keyboard shortcut management

use serde_json::json;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
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
    let parsed: Shortcut = shortcut.parse().map_err(|e: tauri_plugin_global_shortcut::Error| {
        format!("Invalid shortcut '{}': {}", shortcut, e)
    })?;

    let shortcut_clone = shortcut.clone();
    let pad_id_clone = pad_id.clone();
    let app_clone = app.clone();

    app.global_shortcut().on_shortcut(parsed, move |_app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            tracing::info!("[Shortcuts] Global shortcut triggered: {} -> {}", shortcut_clone, pad_id_clone);
            let _ = app_clone.emit("global-shortcut-triggered", json!({
                "padId": pad_id_clone,
                "shortcut": shortcut_clone
            }));
        }
    }).map_err(|e| format!("Failed to register shortcut: {}", e))?;

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
    let parsed: Shortcut = shortcut.parse().map_err(|e: tauri_plugin_global_shortcut::Error| {
        format!("Invalid shortcut '{}': {}", shortcut, e)
    })?;

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

                let _ = app.global_shortcut().on_shortcut(parsed, move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let _ = app_clone.emit("global-shortcut-triggered", json!({
                            "padId": pad_id_clone,
                            "shortcut": shortcut_clone
                        }));
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
pub fn get_global_hotkeys_enabled(
    registry: State<ShortcutRegistry>,
) -> Result<bool, String> {
    let enabled = registry.enabled.lock().map_err(|e| e.to_string())?;
    Ok(*enabled)
}
```

**Step 2: Add module to mod.rs**

In `src-tauri/src/application/mod.rs`, add:

```rust
pub mod shortcut_commands;
```

**Step 3: Register commands and state in lib.rs**

In `src-tauri/src/lib.rs`:

1. Add imports at the top (after line 75):

```rust
use application::shortcut_commands::{
    register_global_shortcut,
    unregister_global_shortcut,
    unregister_all_shortcuts,
    set_global_hotkeys_enabled,
    get_global_hotkeys_enabled,
    ShortcutRegistry,
};
```

2. Add state management in setup (after line 102, after `app.manage(state)`):

```rust
            app.manage(ShortcutRegistry::default());
```

3. Add commands to invoke_handler (after line 257):

```rust
            // Shortcut management
            register_global_shortcut,
            unregister_global_shortcut,
            unregister_all_shortcuts,
            set_global_hotkeys_enabled,
            get_global_hotkeys_enabled,
```

**Step 4: Run tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: All tests pass

**Step 5: Commit**

```bash
git add src-tauri/src/application/shortcut_commands.rs src-tauri/src/application/mod.rs src-tauri/src/lib.rs
git commit -m "feat(shortcuts): add Tauri commands for global shortcut management"
```

---

## Task 6: Create ShortcutService (Frontend)

**Files:**
- Create: `src/app/core/services/shortcut.service.ts`
- Modify: `src/app/core/services/tauri.service.ts`

**Step 1: Add Tauri bindings for shortcuts**

In `src/app/core/services/tauri.service.ts`, add these methods:

```typescript
  // Shortcut management
  async registerGlobalShortcut(padId: string, shortcut: string): Promise<void> {
    await invoke('register_global_shortcut', { padId, shortcut });
  }

  async unregisterGlobalShortcut(shortcut: string): Promise<void> {
    await invoke('unregister_global_shortcut', { shortcut });
  }

  async unregisterAllShortcuts(): Promise<void> {
    await invoke('unregister_all_shortcuts');
  }

  async setGlobalHotkeysEnabled(enabled: boolean): Promise<void> {
    await invoke('set_global_hotkeys_enabled', { enabled });
  }

  async getGlobalHotkeysEnabled(): Promise<boolean> {
    return await invoke('get_global_hotkeys_enabled');
  }

  async listenGlobalShortcut(callback: (data: { padId: string; shortcut: string }) => void): Promise<UnlistenFn> {
    return await listen<{ padId: string; shortcut: string }>('global-shortcut-triggered', (event) => {
      callback(event.payload);
    });
  }
```

**Step 2: Create ShortcutService**

Create `src/app/core/services/shortcut.service.ts`:

```typescript
import { Injectable, signal, OnDestroy } from '@angular/core';
import { TauriService } from './tauri.service';
import { SoundboardService } from './soundboard.service';
import { formatShortcut, shortcutFromEvent, isModifierKey } from '../models';

@Injectable({
  providedIn: 'root'
})
export class ShortcutService implements OnDestroy {
  // Map shortcut string -> pad id
  private registry = new Map<string, string>();

  // Enabled state
  private _enabled = signal(true);
  readonly enabled = this._enabled.asReadonly();

  private unlistenGlobalShortcut?: () => void;

  constructor(
    private tauri: TauriService,
    private soundboard: SoundboardService
  ) {
    this.init();
  }

  private async init(): Promise<void> {
    // Load initial state
    try {
      const settings = await this.tauri.loadSettings();
      this._enabled.set(settings.audio.globalHotkeysEnabled);
    } catch (err) {
      console.error('Failed to load global hotkeys setting:', err);
    }

    // Listen for global shortcut events from backend
    this.unlistenGlobalShortcut = await this.tauri.listenGlobalShortcut((data) => {
      console.log('[ShortcutService] Global shortcut triggered:', data);
      this.soundboard.playSound(data.padId);
    });

    // Register all existing shortcuts from soundboard
    await this.syncFromSoundboard();
  }

  ngOnDestroy(): void {
    this.unlistenGlobalShortcut?.();
  }

  /**
   * Sync shortcuts from soundboard pads to backend
   */
  async syncFromSoundboard(): Promise<void> {
    const pads = this.soundboard.pads();

    // Clear existing
    await this.tauri.unregisterAllShortcuts();
    this.registry.clear();

    // Register all pad shortcuts
    for (const pad of pads) {
      if (pad.hotkey) {
        try {
          await this.tauri.registerGlobalShortcut(pad.id, pad.hotkey);
          this.registry.set(pad.hotkey, pad.id);
        } catch (err) {
          console.error(`Failed to register shortcut ${pad.hotkey} for ${pad.id}:`, err);
        }
      }
    }
  }

  /**
   * Check if a shortcut is already used by another pad
   * Returns the pad ID if conflict exists, null otherwise
   */
  checkConflict(shortcut: string, excludePadId?: string): string | null {
    const existingPadId = this.registry.get(shortcut);
    if (existingPadId && existingPadId !== excludePadId) {
      return existingPadId;
    }
    return null;
  }

  /**
   * Register a shortcut for a pad
   */
  async register(padId: string, shortcut: string): Promise<void> {
    // Unregister old shortcut for this pad if exists
    const oldShortcut = this.getShortcutForPad(padId);
    if (oldShortcut) {
      await this.unregister(oldShortcut);
    }

    // Register new shortcut
    try {
      await this.tauri.registerGlobalShortcut(padId, shortcut);
      this.registry.set(shortcut, padId);
    } catch (err) {
      console.error(`Failed to register shortcut ${shortcut}:`, err);
      throw err;
    }
  }

  /**
   * Unregister a shortcut
   */
  async unregister(shortcut: string): Promise<void> {
    try {
      await this.tauri.unregisterGlobalShortcut(shortcut);
      this.registry.delete(shortcut);
    } catch (err) {
      console.error(`Failed to unregister shortcut ${shortcut}:`, err);
    }
  }

  /**
   * Get shortcut assigned to a pad
   */
  getShortcutForPad(padId: string): string | null {
    for (const [shortcut, id] of this.registry.entries()) {
      if (id === padId) return shortcut;
    }
    return null;
  }

  /**
   * Enable or disable global hotkeys
   */
  async setEnabled(enabled: boolean): Promise<void> {
    try {
      await this.tauri.setGlobalHotkeysEnabled(enabled);
      this._enabled.set(enabled);
    } catch (err) {
      console.error('Failed to set global hotkeys enabled:', err);
      throw err;
    }
  }

  /**
   * Format a keyboard event as a shortcut string for recording
   */
  formatEventAsShortcut(event: KeyboardEvent): string | null {
    if (isModifierKey(event.key)) {
      return null; // Don't record modifier-only presses
    }
    const shortcut = shortcutFromEvent(event);
    return formatShortcut(shortcut);
  }
}
```

**Step 3: Commit**

```bash
git add src/app/core/services/shortcut.service.ts src/app/core/services/tauri.service.ts
git commit -m "feat(shortcuts): add ShortcutService for frontend shortcut management"
```

---

## Task 7: Add key recorder UI to SoundPadComponent

**Files:**
- Modify: `src/app/features/soundboard/sound-pad/sound-pad.component.ts`

**Step 1: Update component**

In `src/app/features/soundboard/sound-pad/sound-pad.component.ts`:

1. Add imports at the top:

```typescript
import { ShortcutService } from '../../../core/services/shortcut.service';
```

2. Add new Output and state (after line 158):

```typescript
  @Output() shortcutChange = new EventEmitter<string | null>();

  isRecording = false;
```

3. Add inject in constructor:

```typescript
  constructor(
    private soundboardService: SoundboardService,
    private shortcutService: ShortcutService
  ) {}
```

4. Add the shortcut section in the template (after the Speed section, around line 119):

```html
            <!-- Shortcut -->
            <div class="pt-3 border-t border-border">
              <div class="flex justify-between items-center mb-2 text-xs">
                <span class="text-text-secondary">Shortcut</span>
              </div>
              <div class="flex gap-1">
                <button
                  class="flex-1 px-3 py-1.5 text-xs rounded border transition-colors text-left"
                  [class]="isRecording
                    ? 'bg-accent border-accent text-white animate-pulse'
                    : 'bg-surface-hover border-border text-text-primary hover:border-text-muted'"
                  (click)="startRecording($event)"
                  (keydown)="onRecordKeydown($event)"
                >
                  {{ isRecording ? 'Press keys...' : (pad.hotkey || 'Click to set') }}
                </button>
                @if (pad.hotkey) {
                  <button
                    class="px-2 text-text-muted hover:text-status-error transition-colors"
                    (click)="clearShortcut($event)"
                    title="Clear shortcut"
                  >&times;</button>
                }
              </div>
            </div>
```

5. Add the methods (after resetAll method):

```typescript
  startRecording(event: MouseEvent): void {
    event.stopPropagation();
    this.isRecording = true;
  }

  onRecordKeydown(event: KeyboardEvent): void {
    if (!this.isRecording) return;

    event.preventDefault();
    event.stopPropagation();

    const shortcut = this.shortcutService.formatEventAsShortcut(event);
    if (!shortcut) return; // Ignore modifier-only presses

    this.isRecording = false;

    // Check for conflicts
    const conflictPadId = this.shortcutService.checkConflict(shortcut, this.pad.id);
    if (conflictPadId) {
      const pads = this.soundboardService.pads();
      const conflictPad = pads.find(p => p.id === conflictPadId);
      const conflictName = conflictPad?.sound?.name || conflictPadId;

      if (!confirm(`"${shortcut}" is already assigned to "${conflictName}". Replace?`)) {
        return;
      }
      // User confirmed replacement - the old pad will lose its shortcut
    }

    this.shortcutChange.emit(shortcut);
  }

  clearShortcut(event: MouseEvent): void {
    event.stopPropagation();
    this.shortcutChange.emit(null);
  }
```

**Step 2: Commit**

```bash
git add src/app/features/soundboard/sound-pad/sound-pad.component.ts
git commit -m "feat(shortcuts): add key recorder UI to pad settings popup"
```

---

## Task 8: Handle shortcut changes in SoundboardComponent

**Files:**
- Modify: `src/app/features/soundboard/soundboard.component.ts`
- Modify: `src/app/core/services/soundboard.service.ts`

**Step 1: Add shortcutChange handler to SoundboardComponent**

In `src/app/features/soundboard/soundboard.component.ts`:

1. Add import:

```typescript
import { ShortcutService } from '../../core/services/shortcut.service';
```

2. Inject service in constructor:

```typescript
  constructor(
    public soundboard: SoundboardService,
    private shortcutService: ShortcutService
  ) {}
```

3. Add output binding in template (around line 59):

```html
              (shortcutChange)="onShortcutChange(pad.id, $event)"
```

4. Add handler method:

```typescript
  async onShortcutChange(padId: string, shortcut: string | null): Promise<void> {
    // Update pad
    this.soundboard.setPadHotkey(padId, shortcut);

    // Update shortcut registry
    if (shortcut) {
      try {
        await this.shortcutService.register(padId, shortcut);
      } catch (err) {
        console.error('Failed to register shortcut:', err);
      }
    } else {
      const oldShortcut = this.shortcutService.getShortcutForPad(padId);
      if (oldShortcut) {
        await this.shortcutService.unregister(oldShortcut);
      }
    }
  }
```

**Step 2: Add setPadHotkey method to SoundboardService**

In `src/app/core/services/soundboard.service.ts`, add method (after setPadSpeed):

```typescript
  /**
   * Set hotkey for a pad
   */
  setPadHotkey(padId: string, hotkey: string | null): void {
    // If setting a new hotkey, clear it from any other pad first
    if (hotkey) {
      this._pads.update(pads => pads.map(p =>
        p.hotkey === hotkey && p.id !== padId
          ? { ...p, hotkey: undefined }
          : p
      ));
    }

    this._pads.update(pads => pads.map(p =>
      p.id === padId ? { ...p, hotkey: hotkey || undefined } : p
    ));
    this.saveState();
  }
```

**Step 3: Commit**

```bash
git add src/app/features/soundboard/soundboard.component.ts src/app/core/services/soundboard.service.ts
git commit -m "feat(shortcuts): handle shortcut changes in soundboard"
```

---

## Task 9: Add Global Hotkeys toggle to Settings popup

**Files:**
- Modify: `src/app/shared/components/settings-popup/settings-popup.component.ts`

**Step 1: Update SettingsPopupComponent**

In `src/app/shared/components/settings-popup/settings-popup.component.ts`:

1. Add import:

```typescript
import { ShortcutService } from '../../../core/services/shortcut.service';
```

2. Inject service in constructor:

```typescript
  constructor(
    private tauri: TauriService,
    private mixer: MixerService,
    private shortcutService: ShortcutService
  ) {}
```

3. Add computed for enabled state (after micMuted):

```typescript
  readonly globalHotkeysEnabled = computed(() => this.shortcutService.enabled());
```

4. Add template section after Mic Monitoring (around line 161):

```html
          <!-- Keyboard Section -->
          <div class="pt-6 border-t border-border">
            <h3 class="text-xs font-semibold text-text-muted uppercase tracking-wider mb-4">Keyboard</h3>

            <div class="flex items-center justify-between py-3 px-4 bg-background rounded-lg">
              <div>
                <span class="text-sm text-text-primary">Global Hotkeys</span>
                <p class="text-xs text-text-muted mt-0.5">Trigger sounds when app is in background</p>
              </div>
              <button
                class="w-12 h-6 rounded-full transition-colors relative"
                [class]="globalHotkeysEnabled() ? 'bg-accent' : 'bg-surface'"
                (click)="toggleGlobalHotkeys()"
              >
                <div
                  class="absolute top-1 w-4 h-4 bg-white rounded-full transition-transform"
                  [class]="globalHotkeysEnabled() ? 'left-7' : 'left-1'"
                ></div>
              </button>
            </div>
          </div>
```

5. Add toggle method:

```typescript
  async toggleGlobalHotkeys(): Promise<void> {
    const newValue = !this.globalHotkeysEnabled();
    try {
      await this.shortcutService.setEnabled(newValue);
    } catch (err) {
      console.error('Failed to toggle global hotkeys:', err);
    }
  }
```

**Step 2: Commit**

```bash
git add src/app/shared/components/settings-popup/settings-popup.component.ts
git commit -m "feat(shortcuts): add Global Hotkeys toggle to settings"
```

---

## Task 10: Update handleKeydown for modifier support

**Files:**
- Modify: `src/app/features/soundboard/soundboard.component.ts`

**Step 1: Update handleKeydown method**

In `src/app/features/soundboard/soundboard.component.ts`, replace the handleKeydown method:

```typescript
  @HostListener('window:keydown', ['$event'])
  handleKeydown(event: KeyboardEvent): void {
    // Ignore if user is typing in an input field
    if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
      return;
    }

    // Escape stops all sounds
    if (event.key === 'Escape') {
      this.soundboard.stopAll();
      return;
    }

    // Check custom shortcuts (with modifiers)
    const pads = this.soundboard.pads();
    for (const pad of pads) {
      if (pad.hotkey && pad.sound) {
        if (eventMatchesShortcut(event, pad.hotkey)) {
          event.preventDefault();
          this.soundboard.playSound(pad.id);
          return;
        }
      }
    }

    // Fallback to default hotkeys (no modifiers, for backwards compatibility)
    if (!event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey) {
      const padIndex = pads.findIndex(p => {
        const defaultHotkey = DEFAULT_HOTKEYS[pads.indexOf(p)];
        return !p.hotkey && defaultHotkey === event.key;
      });

      if (padIndex >= 0) {
        const pad = pads[padIndex];
        if (pad.sound) {
          event.preventDefault();
          this.soundboard.playSound(pad.id);
        }
      }
    }
  }
```

Also add the import at the top:

```typescript
import { eventMatchesShortcut } from '../../core/models';
```

**Step 2: Commit**

```bash
git add src/app/features/soundboard/soundboard.component.ts
git commit -m "feat(shortcuts): update keydown handler for modifier support"
```

---

## Task 11: Test and verify

**Files:** None (manual testing)

**Step 1: Build and run**

Run: `npm run tauri dev`

**Step 2: Test scenarios**

1. Open app, click on a pad's gear icon
2. Click "Click to set" button in the Shortcut section
3. Press a key combo (e.g., Ctrl+1)
4. Verify the shortcut is displayed
5. Press Ctrl+1 → sound should play
6. Minimize app, press Ctrl+1 → sound should still play (global hotkey)
7. Go to Settings, toggle "Global Hotkeys" off
8. Minimize app, press Ctrl+1 → sound should NOT play
9. Focus app, press Ctrl+1 → sound SHOULD play (local hotkey)
10. Try setting a conflicting shortcut → confirm dialog should appear

**Step 3: Run all tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml && npm test`
Expected: All tests pass

**Step 4: Final commit**

```bash
git add -A
git commit -m "feat(shortcuts): complete custom keyboard shortcuts feature

- Global hotkeys enabled by default
- Modifier key support (Ctrl, Alt, Shift, Meta)
- Key recorder UI in pad settings popup
- Conflict detection with user confirmation
- Toggle in Settings to enable/disable global hotkeys

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 12: Update roadmap

**Files:**
- Modify: `ROADMAP.md`

**Step 1: Update roadmap**

In `ROADMAP.md`, move "Custom Keyboard Shortcuts" from "To Do" to "Done" in Phase 3:

```markdown
### Done
- [x] **Tailwind CSS Migration** ...
- [x] **New Layout Structure** ...
- [x] **Basic Folder System** ...
- [x] **Pad Settings Popup** ...
- [x] **Custom Keyboard Shortcuts**
  - Configurable shortcut per pad (in Pad Settings popup)
  - Support for modifier key combinations (Ctrl+1, Alt+Shift+0, etc.)
  - Key combination recorder (press keys to set shortcut)
  - Conflict detection (warn if shortcut already used)
  - Global hotkeys (work even when app is not focused)
```

Update progress: `## Phase 3 - UI/UX Redesign - 55% Complete`

**Step 2: Archive design document**

```bash
mv docs/plans/2026-01-05-custom-keyboard-shortcuts-design.md docs/plans/archive/
```

**Step 3: Commit**

```bash
git add ROADMAP.md docs/plans/archive/2026-01-05-custom-keyboard-shortcuts-design.md
git commit -m "docs: mark custom keyboard shortcuts complete, archive plan"
```
