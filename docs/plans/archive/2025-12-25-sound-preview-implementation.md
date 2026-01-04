# Sound Preview System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Complete the sound preview feature with selectable output device, stop control, and visual feedback.

**Architecture:** Dedicated PreviewEngine thread with channel-based communication, emitting Tauri events for UI sync. Separate from AudioEngine to keep concerns isolated.

**Tech Stack:** Rust (cpal, rodio, crossbeam-channel), Angular (signals, Tauri events)

---

## Task 1: Create PreviewEngine Module

**Files:**
- Create: `src-tauri/src/application/preview_engine.rs`
- Modify: `src-tauri/src/application/mod.rs`

**Step 1: Create the PreviewEngine struct and command enum**

```rust
// src-tauri/src/application/preview_engine.rs
//! Preview Engine - Plays sounds on a selectable output device for monitoring

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{bounded, Receiver, Sender};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// Commands that can be sent to the preview engine
#[derive(Debug)]
pub enum PreviewCommand {
    /// Play a sound file on a specific device
    Play {
        path: String,
        device_name: String,
        pad_id: String,
    },
    /// Stop the currently playing preview
    Stop,
    /// Shutdown the engine
    Shutdown,
}

/// The preview engine that manages sound previews
pub struct PreviewEngine {
    command_tx: Sender<PreviewCommand>,
    current_pad_id: Arc<Mutex<Option<String>>>,
    thread_handle: Option<JoinHandle<()>>,
}

impl PreviewEngine {
    /// Create and start a new preview engine
    pub fn new(app_handle: AppHandle) -> Self {
        let (command_tx, command_rx) = bounded(16);
        let current_pad_id = Arc::new(Mutex::new(None::<String>));
        let current_pad_id_clone = current_pad_id.clone();

        let thread_handle = thread::spawn(move || {
            run_preview_thread(command_rx, current_pad_id_clone, app_handle);
        });

        Self {
            command_tx,
            current_pad_id,
            thread_handle: Some(thread_handle),
        }
    }

    /// Send a command to the preview engine
    pub fn send_command(&self, command: PreviewCommand) -> Result<(), String> {
        self.command_tx
            .send(command)
            .map_err(|e| format!("Failed to send preview command: {}", e))
    }

    /// Get the currently previewing pad ID
    pub fn current_pad_id(&self) -> Option<String> {
        self.current_pad_id.lock().ok()?.clone()
    }

    /// Shutdown the preview engine
    pub fn shutdown(&mut self) {
        let _ = self.command_tx.send(PreviewCommand::Shutdown);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for PreviewEngine {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Find an output device by name
fn find_output_device(name: &str) -> Option<cpal::Device> {
    let host = cpal::default_host();

    if name == "default" || name.is_empty() {
        return host.default_output_device();
    }

    if let Ok(devices) = host.output_devices() {
        for device in devices {
            if let Ok(device_name) = device.name() {
                if device_name == name {
                    return Some(device);
                }
            }
        }
    }

    // Fallback to default
    host.default_output_device()
}

/// The main preview thread
fn run_preview_thread(
    command_rx: Receiver<PreviewCommand>,
    current_pad_id: Arc<Mutex<Option<String>>>,
    app_handle: AppHandle,
) {
    // Current playback state - these must stay alive during playback
    let mut current_sink: Option<Sink> = None;
    let mut _current_stream: Option<OutputStream> = None;
    let mut _current_stream_handle: Option<OutputStreamHandle> = None;

    loop {
        // Check if current sound finished naturally
        if let Some(ref sink) = current_sink {
            if sink.empty() {
                if let Ok(mut pad_id) = current_pad_id.lock() {
                    if let Some(id) = pad_id.take() {
                        let _ = app_handle.emit("preview-stopped", &id);
                        tracing::info!("Preview finished naturally: {}", id);
                    }
                }
                current_sink = None;
                _current_stream = None;
                _current_stream_handle = None;
            }
        }

        match command_rx.recv_timeout(Duration::from_millis(50)) {
            Ok(command) => match command {
                PreviewCommand::Play { path, device_name, pad_id } => {
                    // Stop current preview if any
                    if let Some(sink) = current_sink.take() {
                        sink.stop();
                    }
                    if let Ok(mut current) = current_pad_id.lock() {
                        if let Some(old_id) = current.take() {
                            let _ = app_handle.emit("preview-stopped", &old_id);
                        }
                    }
                    _current_stream = None;
                    _current_stream_handle = None;

                    // Find the output device
                    let device = match find_output_device(&device_name) {
                        Some(d) => d,
                        None => {
                            tracing::error!("Preview device not found: {}", device_name);
                            continue;
                        }
                    };

                    // Create output stream on the specific device
                    let (stream, stream_handle) = match OutputStream::try_from_device(&device) {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::error!("Failed to create preview stream: {}", e);
                            continue;
                        }
                    };

                    // Create sink
                    let sink = match Sink::try_new(&stream_handle) {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::error!("Failed to create preview sink: {}", e);
                            continue;
                        }
                    };

                    // Open and decode the file
                    let file = match File::open(&path) {
                        Ok(f) => f,
                        Err(e) => {
                            tracing::error!("Failed to open file for preview: {}", e);
                            continue;
                        }
                    };

                    let source = match Decoder::new(BufReader::new(file)) {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::error!("Failed to decode file for preview: {}", e);
                            continue;
                        }
                    };

                    // Play the sound
                    sink.append(source);

                    // Store state
                    current_sink = Some(sink);
                    _current_stream = Some(stream);
                    _current_stream_handle = Some(stream_handle);

                    if let Ok(mut current) = current_pad_id.lock() {
                        *current = Some(pad_id.clone());
                    }

                    let _ = app_handle.emit("preview-started", &pad_id);
                    tracing::info!("Preview started: {} on {}", path, device_name);
                }

                PreviewCommand::Stop => {
                    if let Some(sink) = current_sink.take() {
                        sink.stop();
                    }
                    if let Ok(mut current) = current_pad_id.lock() {
                        if let Some(id) = current.take() {
                            let _ = app_handle.emit("preview-stopped", &id);
                            tracing::info!("Preview stopped: {}", id);
                        }
                    }
                    _current_stream = None;
                    _current_stream_handle = None;
                }

                PreviewCommand::Shutdown => {
                    if let Some(sink) = current_sink.take() {
                        sink.stop();
                    }
                    tracing::info!("Preview engine shutdown");
                    return;
                }
            },
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                // No command, continue checking sink state
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                tracing::info!("Preview command channel closed");
                return;
            }
        }
    }
}
```

**Step 2: Export the module**

In `src-tauri/src/application/mod.rs`, add:

```rust
pub mod preview_engine;
pub use preview_engine::*;
```

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`

Expected: Compilation succeeds with no errors

**Step 4: Commit**

```bash
git add src-tauri/src/application/preview_engine.rs src-tauri/src/application/mod.rs
git commit -m "feat: add PreviewEngine module for sound preview"
```

---

## Task 2: Add PreviewEngine to AppState

**Files:**
- Modify: `src-tauri/src/application/state.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Update AppState struct**

Replace `src-tauri/src/application/state.rs` with:

```rust
//! Application state management

use crate::application::audio_engine::AudioEngine;
use crate::application::preview_engine::PreviewEngine;
use crate::domain::{AppSettings, MixerConfig};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Global application state managed by Tauri
pub struct AppState {
    pub mixer_config: Arc<RwLock<MixerConfig>>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub is_mixing: Arc<RwLock<bool>>,
    pub audio_engine: Arc<Mutex<AudioEngine>>,
    pub preview_engine: Arc<Mutex<Option<PreviewEngine>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mixer_config: Arc::new(RwLock::new(MixerConfig::default())),
            settings: Arc::new(RwLock::new(AppSettings::default())),
            is_mixing: Arc::new(RwLock::new(false)),
            audio_engine: Arc::new(Mutex::new(AudioEngine::new())),
            preview_engine: Arc::new(Mutex::new(None)),
        }
    }

    /// Create state with pre-loaded settings
    pub fn with_settings(settings: AppSettings) -> Self {
        let mut mixer_config = MixerConfig::default();
        mixer_config.master_volume = settings.audio.master_volume;

        Self {
            mixer_config: Arc::new(RwLock::new(mixer_config)),
            settings: Arc::new(RwLock::new(settings)),
            is_mixing: Arc::new(RwLock::new(false)),
            audio_engine: Arc::new(Mutex::new(AudioEngine::new())),
            preview_engine: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 2: Initialize PreviewEngine in lib.rs setup**

In `src-tauri/src/lib.rs`, find the `setup` closure and add PreviewEngine initialization. Add after `app.manage(state);`:

```rust
// Initialize preview engine with app handle
let app_handle = app.handle().clone();
let state_ref = app.state::<AppState>();
let preview_engine = PreviewEngine::new(app_handle);
{
    let mut preview = state_ref.preview_engine.blocking_lock();
    *preview = Some(preview_engine);
}
```

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`

Expected: Compilation succeeds

**Step 4: Commit**

```bash
git add src-tauri/src/application/state.rs src-tauri/src/lib.rs
git commit -m "feat: integrate PreviewEngine into AppState"
```

---

## Task 3: Update Tauri Commands for Preview

**Files:**
- Modify: `src-tauri/src/application/commands.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Replace preview_sound command and add stop_preview**

In `src-tauri/src/application/commands.rs`, find the `preview_sound` function and replace it with:

```rust
/// Preview a sound file on a specific output device
#[tauri::command]
pub async fn preview_sound(
    state: State<'_, AppState>,
    path: String,
    device_name: String,
    pad_id: String,
) -> Result<(), String> {
    use crate::application::preview_engine::PreviewCommand;

    let preview = state.preview_engine.lock().await;
    if let Some(ref engine) = *preview {
        engine.send_command(PreviewCommand::Play {
            path,
            device_name,
            pad_id,
        })
    } else {
        Err("Preview engine not initialized".to_string())
    }
}

/// Stop the currently playing preview
#[tauri::command]
pub async fn stop_preview(state: State<'_, AppState>) -> Result<(), String> {
    use crate::application::preview_engine::PreviewCommand;

    let preview = state.preview_engine.lock().await;
    if let Some(ref engine) = *preview {
        engine.send_command(PreviewCommand::Stop)
    } else {
        Err("Preview engine not initialized".to_string())
    }
}

/// Get the currently previewing pad ID
#[tauri::command]
pub async fn get_preview_state(state: State<'_, AppState>) -> Option<String> {
    let preview = state.preview_engine.lock().await;
    preview.as_ref().and_then(|e| e.current_pad_id())
}
```

**Step 2: Register new commands in lib.rs**

In `src-tauri/src/lib.rs`, update the imports to include `stop_preview` and `get_preview_state`:

```rust
use application::{
    commands::{
        // ... existing imports ...
        preview_sound, stop_preview, get_preview_state,
        // ...
    },
    // ...
};
```

And add them to the `invoke_handler`:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    preview_sound,
    stop_preview,
    get_preview_state,
    // ...
])
```

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`

Expected: Compilation succeeds

**Step 4: Commit**

```bash
git add src-tauri/src/application/commands.rs src-tauri/src/lib.rs
git commit -m "feat: update preview commands with device selection and stop"
```

---

## Task 4: Add Preview Device to Settings

**Files:**
- Modify: `src-tauri/src/domain/settings.rs`
- Modify: `src/app/core/models/audio-device.model.ts`

**Step 1: Add preview_device_id to AudioSettings (Rust)**

In `src-tauri/src/domain/settings.rs`, update `AudioSettings`:

```rust
/// User preferences for audio devices
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AudioSettings {
    /// Selected input device ID (microphone)
    pub input_device_id: Option<String>,
    /// Selected output device ID (virtual microphone)
    pub output_device_id: Option<String>,
    /// Selected preview output device ID (for monitoring)
    pub preview_device_id: Option<String>,
    /// Master volume (0.0 to 1.0)
    pub master_volume: f32,
    /// Sample rate to use
    pub sample_rate: u32,
    /// Buffer size in frames
    pub buffer_size: u32,
}

impl AudioSettings {
    pub fn new() -> Self {
        Self {
            input_device_id: None,
            output_device_id: None,
            preview_device_id: None,
            master_volume: 1.0,
            sample_rate: 48000,
            buffer_size: 1024,
        }
    }
}
```

**Step 2: Add previewDeviceId to AudioSettings (TypeScript)**

In `src/app/core/models/audio-device.model.ts`, update `AudioSettings`:

```typescript
export interface AudioSettings {
  inputDeviceId: string | null;
  outputDeviceId: string | null;
  previewDeviceId: string | null;
  masterVolume: number;
  sampleRate: number;
  bufferSize: number;
}
```

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`
Run: `npm run build`

Expected: Both succeed

**Step 4: Commit**

```bash
git add src-tauri/src/domain/settings.rs src/app/core/models/audio-device.model.ts
git commit -m "feat: add preview device setting to AudioSettings"
```

---

## Task 5: Add Set Preview Device Command

**Files:**
- Modify: `src-tauri/src/application/commands.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Add set_preview_device command**

In `src-tauri/src/application/commands.rs`, add:

```rust
/// Set the preview output device
#[tauri::command]
pub async fn set_preview_device(
    state: State<'_, AppState>,
    app: AppHandle,
    device_id: Option<String>,
) -> Result<(), String> {
    // Update settings
    {
        let mut settings = state.settings.write().await;
        settings.audio.preview_device_id = device_id;
    }

    // Persist settings
    save_settings_to_store(&app, &*state.settings.read().await)
        .await
        .map_err(|e| e.to_string())?;

    tracing::info!("Preview device set");
    Ok(())
}
```

**Step 2: Register command in lib.rs**

Add `set_preview_device` to imports and invoke_handler.

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`

Expected: Compilation succeeds

**Step 4: Commit**

```bash
git add src-tauri/src/application/commands.rs src-tauri/src/lib.rs
git commit -m "feat: add set_preview_device command"
```

---

## Task 6: Update TauriService

**Files:**
- Modify: `src/app/core/services/tauri.service.ts`

**Step 1: Update previewSound method signature**

Find the `previewSound` method and update it:

```typescript
/**
 * Preview a sound on a specific output device
 */
async previewSound(path: string, deviceName: string, padId: string): Promise<void> {
  await invoke('preview_sound', { path, deviceName, padId });
}

/**
 * Stop the currently playing preview
 */
async stopPreview(): Promise<void> {
  await invoke('stop_preview');
}

/**
 * Get the currently previewing pad ID
 */
async getPreviewState(): Promise<string | null> {
  return invoke<string | null>('get_preview_state');
}

/**
 * Set the preview output device
 */
async setPreviewDevice(deviceId: string | null): Promise<void> {
  await invoke('set_preview_device', { deviceId });
}
```

**Step 2: Update mapSettings and unmapSettings**

Update the `mapSettings` method:

```typescript
private mapSettings(s: any): AppSettings {
  return {
    audio: {
      inputDeviceId: s.audio.input_device_id,
      outputDeviceId: s.audio.output_device_id,
      previewDeviceId: s.audio.preview_device_id,
      masterVolume: s.audio.master_volume,
      sampleRate: s.audio.sample_rate,
      bufferSize: s.audio.buffer_size
    },
    startMinimized: s.start_minimized,
    autoStartMixing: s.auto_start_mixing
  };
}
```

Update the `unmapSettings` method:

```typescript
private unmapSettings(s: AppSettings): any {
  return {
    audio: {
      input_device_id: s.audio.inputDeviceId,
      output_device_id: s.audio.outputDeviceId,
      preview_device_id: s.audio.previewDeviceId,
      master_volume: s.audio.masterVolume,
      sample_rate: s.audio.sampleRate,
      buffer_size: s.audio.bufferSize
    },
    start_minimized: s.startMinimized,
    auto_start_mixing: s.autoStartMixing
  };
}
```

**Step 3: Add Tauri event listener methods**

Add at the end of the class:

```typescript
/**
 * Listen for preview started events
 */
async listenPreviewStarted(callback: (padId: string) => void): Promise<() => void> {
  const { listen } = await import('@tauri-apps/api/event');
  const unlisten = await listen<string>('preview-started', (event) => {
    callback(event.payload);
  });
  return unlisten;
}

/**
 * Listen for preview stopped events
 */
async listenPreviewStopped(callback: (padId: string) => void): Promise<() => void> {
  const { listen } = await import('@tauri-apps/api/event');
  const unlisten = await listen<string>('preview-stopped', (event) => {
    callback(event.payload);
  });
  return unlisten;
}
```

**Step 4: Verify build**

Run: `npm run build`

Expected: Build succeeds

**Step 5: Commit**

```bash
git add src/app/core/services/tauri.service.ts
git commit -m "feat: update TauriService with preview device methods"
```

---

## Task 7: Update SoundboardService

**Files:**
- Modify: `src/app/core/services/soundboard.service.ts`

**Step 1: Add preview state signal and methods**

Add these properties after the existing signals:

```typescript
private _previewingPadId = signal<string | null>(null);
readonly previewingPadId = this._previewingPadId.asReadonly();

private _previewDeviceId = signal<string | null>(null);
readonly previewDeviceId = this._previewDeviceId.asReadonly();

private unlistenPreviewStarted?: () => void;
private unlistenPreviewStopped?: () => void;
```

**Step 2: Add init method for event listeners**

Add this method and call it from constructor:

```typescript
constructor(private tauri: TauriService) {
  this.loadState();
  this.initPreviewListeners();
}

private async initPreviewListeners(): Promise<void> {
  this.unlistenPreviewStarted = await this.tauri.listenPreviewStarted((padId) => {
    this._previewingPadId.set(padId);
  });

  this.unlistenPreviewStopped = await this.tauri.listenPreviewStopped((padId) => {
    if (this._previewingPadId() === padId) {
      this._previewingPadId.set(null);
    }
  });
}
```

**Step 3: Update previewSound method**

Replace the existing `previewSound` method:

```typescript
/**
 * Preview a sound on the selected preview output device
 */
async previewSound(padId: string): Promise<void> {
  const pad = this._pads().find(p => p.id === padId);
  if (!pad?.sound) return;

  try {
    // If same pad is previewing, stop it
    if (this._previewingPadId() === padId) {
      await this.stopPreview();
      return;
    }

    const previewDeviceId = this._previewDeviceId() || 'default';
    await this.tauri.previewSound(pad.sound.path, previewDeviceId, padId);
  } catch (err) {
    this._error.set(err instanceof Error ? err.message : 'Failed to preview sound');
  }
}

/**
 * Stop the currently playing preview
 */
async stopPreview(): Promise<void> {
  try {
    await this.tauri.stopPreview();
  } catch (err) {
    this._error.set(err instanceof Error ? err.message : 'Failed to stop preview');
  }
}

/**
 * Set the preview output device
 */
async setPreviewDevice(deviceId: string | null): Promise<void> {
  this._previewDeviceId.set(deviceId);
  try {
    await this.tauri.setPreviewDevice(deviceId);
  } catch (err) {
    this._error.set(err instanceof Error ? err.message : 'Failed to set preview device');
  }
}

/**
 * Load preview device from settings
 */
async loadPreviewDevice(): Promise<void> {
  try {
    const settings = await this.tauri.loadSettings();
    this._previewDeviceId.set(settings.audio.previewDeviceId);
  } catch (err) {
    console.error('Failed to load preview device:', err);
  }
}
```

**Step 4: Update loadState to also load preview device**

Add at the end of `loadState`:

```typescript
// Also load preview device setting
this.loadPreviewDevice();
```

**Step 5: Verify build**

Run: `npm run build`

Expected: Build succeeds

**Step 6: Commit**

```bash
git add src/app/core/services/soundboard.service.ts
git commit -m "feat: add preview state management to SoundboardService"
```

---

## Task 8: Update SoundPadComponent UI

**Files:**
- Modify: `src/app/features/soundboard/sound-pad/sound-pad.component.ts`

**Step 1: Add isPreviewing input**

Add after the existing `@Input()` declarations:

```typescript
@Input() isPreviewing = false;
```

**Step 2: Update template - add previewing class and toggle button icon**

Update the template's sound-pad div:

```html
<div
  class="sound-pad"
  [class.has-sound]="pad.sound"
  [class.playing]="pad.isPlaying"
  [class.previewing]="isPreviewing"
  [class.loading]="loading"
  [style.--pad-color]="pad.color"
  (click)="onClick($event)"
  (contextmenu)="onRightClick($event)"
>
```

Update the preview button:

```html
<button class="action-btn preview-btn"
        [class.active]="isPreviewing"
        (click)="onPreview($event)"
        [title]="isPreviewing ? 'Stop preview' : 'Preview (system output)'">
  {{ isPreviewing ? '⏹' : '▶' }}
</button>
```

**Step 3: Add previewing styles**

Add these styles to the styles array:

```css
.sound-pad.previewing {
  animation: preview-pulse 1s ease-in-out infinite;
  border-color: #00d4ff;
}

@keyframes preview-pulse {
  0%, 100% {
    box-shadow: 0 0 8px rgba(0, 212, 255, 0.4);
  }
  50% {
    box-shadow: 0 0 16px rgba(0, 212, 255, 0.7);
  }
}

.preview-btn.active {
  background: #00d4ff;
  color: #000;
}
```

**Step 4: Verify build**

Run: `npm run build`

Expected: Build succeeds

**Step 5: Commit**

```bash
git add src/app/features/soundboard/sound-pad/sound-pad.component.ts
git commit -m "feat: add preview indicator and toggle to SoundPadComponent"
```

---

## Task 9: Connect SoundboardComponent

**Files:**
- Modify: `src/app/features/soundboard/soundboard.component.ts`

**Step 1: Update the sound-pad template binding**

Find the `<app-sound-pad>` element and add the isPreviewing binding:

```html
<app-sound-pad
  [pad]="pad"
  [hotkey]="getHotkey(i)"
  [loading]="soundboard.loading()"
  [isPreviewing]="soundboard.previewingPadId() === pad.id"
  (play)="soundboard.playSound(pad.id)"
  (preview)="soundboard.previewSound(pad.id)"
  (import)="soundboard.importSound(pad.id)"
  (remove)="soundboard.removeSound(pad.id)"
/>
```

**Step 2: Verify build**

Run: `npm run build`

Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/app/features/soundboard/soundboard.component.ts
git commit -m "feat: connect preview state to SoundboardComponent"
```

---

## Task 10: Add Preview Device Dropdown to DeviceSelector

**Files:**
- Modify: `src/app/features/devices/device-selector.component.ts`

**Step 1: Add preview device state**

Add to the component's signal declarations:

```typescript
readonly selectedPreviewId = computed(() => this._settings()?.audio.previewDeviceId ?? '');
```

**Step 2: Add preview device dropdown to template**

After the Output Device section and before the Status section, add:

```html
<!-- Preview Output Device Selection -->
<div class="device-group">
  <label>
    <span class="label-icon">🎧</span>
    <span class="label-text">Preview Output (Monitoring)</span>
  </label>
  <select
    (change)="onPreviewDeviceChange($event)"
    class="device-select"
  >
    <option value="" [selected]="!selectedPreviewId()">-- System Default --</option>
    @for (device of outputDevices(); track device.id) {
      <option [value]="device.id" [selected]="device.id === selectedPreviewId()">
        {{ device.name }}
        @if (device.isDefault) { (Default) }
      </option>
    }
  </select>
</div>
```

**Step 3: Add change handler**

Add the method:

```typescript
async onPreviewDeviceChange(event: Event): Promise<void> {
  const select = event.target as HTMLSelectElement;
  const deviceId = select.value || null;

  try {
    await this.tauri.setPreviewDevice(deviceId);

    // Update local state
    const settings = this._settings();
    if (settings) {
      this._settings.set({
        ...settings,
        audio: { ...settings.audio, previewDeviceId: deviceId }
      });
    }
  } catch (err) {
    console.error('Failed to set preview device:', err);
  }
}
```

**Step 4: Verify build**

Run: `npm run build`

Expected: Build succeeds

**Step 5: Commit**

```bash
git add src/app/features/devices/device-selector.component.ts
git commit -m "feat: add preview device selector to DeviceSelectorComponent"
```

---

## Task 11: Update ROADMAP

**Files:**
- Modify: `ROADMAP.md`

**Step 1: Mark preview feature as done**

Change:
```markdown
### In Progress
- [ ] Sound preview on system output (listen before sending to virtual mic)
```

To:
```markdown
### Done
...
- [x] Sound preview on system output with device selection
```

**Step 2: Commit**

```bash
git add ROADMAP.md
git commit -m "docs: mark sound preview feature as complete"
```

---

## Task 12: Manual Testing

**Steps:**

1. Run `npm run tauri dev`
2. Import a sound to a pad
3. Verify preview button (▶) appears on hover
4. Click preview - verify:
   - Sound plays on selected preview device
   - Button changes to ⏹
   - Pad has animated blue border
5. Click again - verify sound stops and indicator disappears
6. Start preview, then click different pad - verify first stops
7. Change preview device in dropdown - verify next preview uses new device
8. Close and reopen app - verify preview device selection persisted

---

Plan complete and saved to `docs/plans/2025-12-25-sound-preview-implementation.md`.

**Two execution options:**

1. **Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

2. **Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

**Which approach?**
