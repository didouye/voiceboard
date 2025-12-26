# Sound Preview System Design

## Overview

Add a complete sound preview system that allows users to listen to sounds on a selected output device before sending them to the virtual microphone.

## Requirements

- Preview sounds on a selectable output device (not the virtual mic)
- Stop button to cancel preview
- Visual indicator showing which pad is previewing
- One preview at a time (new preview stops the current one)
- Persist preview device selection

## Architecture

### PreviewEngine (Backend)

Dedicated thread with channel-based communication:

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri Commands                        │
│  preview_sound(path, device_id, pad_id) | stop_preview() │
└──────────────────────┬──────────────────────────────────┘
                       │ channel
                       ▼
┌─────────────────────────────────────────────────────────┐
│                   PreviewEngine                          │
│  - Dedicated thread with Receiver<PreviewCommand> loop  │
│  - Manages single active Sink + OutputStream            │
│  - Emits events: Started(pad_id), Stopped(pad_id)       │
└─────────────────────────────────────────────────────────┘
```

**PreviewCommand enum:**
```rust
enum PreviewCommand {
    Play { path: String, device_id: String, pad_id: String },
    Stop,
    Shutdown,
}
```

**Behavior:**
1. On `Play`: stop current sound (if any), open requested device, start playback
2. Thread monitors when sound finishes naturally (sink.empty())
3. On state change, emit event to frontend via Tauri

**Shared state:**
- `Arc<Mutex<Option<String>>>` for current previewing pad_id

### Tauri Commands

```rust
#[tauri::command]
async fn preview_sound(
    state: State<'_, AppState>,
    path: String,
    device_id: String,
    pad_id: String,
) -> Result<(), String>

#[tauri::command]
async fn stop_preview(
    state: State<'_, AppState>,
) -> Result<(), String>

#[tauri::command]
fn get_preview_state(
    state: State<'_, AppState>,
) -> Option<String>  // Returns previewing pad_id
```

**AppState additions:**
```rust
pub struct AppState {
    // ... existing ...
    preview_tx: Sender<PreviewCommand>,
    preview_playing_pad: Arc<Mutex<Option<String>>>,
}
```

**Tauri events emitted:**
- `preview-started` → `{ padId: string }`
- `preview-stopped` → `{ padId: string }`

### Frontend Services

**TauriService:**
```typescript
async previewSound(path: string, deviceId: string, padId: string): Promise<void>
async stopPreview(): Promise<void>
listenPreviewStarted(callback: (padId: string) => void): Promise<UnlistenFn>
listenPreviewStopped(callback: (padId: string) => void): Promise<UnlistenFn>
```

**SoundboardService:**
```typescript
private _previewingPadId = signal<string | null>(null);
readonly previewingPadId = this._previewingPadId.asReadonly();

async previewSound(padId: string): Promise<void>
async stopPreview(): Promise<void>
```

**MixerService:**
```typescript
private _previewDeviceId = signal<string | null>(null);
readonly previewDeviceId = this._previewDeviceId.asReadonly();

async setPreviewDevice(deviceId: string): Promise<void>
```

### UI Components

**SoundPadComponent:**
- New input: `@Input() isPreviewing = false`
- Button toggles between ▶ (play) and ⏹ (stop)
- Animated border when previewing

```css
.sound-pad.previewing {
  animation: preview-pulse 1s ease-in-out infinite;
  border-color: #00d4ff;
  box-shadow: 0 0 12px rgba(0, 212, 255, 0.5);
}

@keyframes preview-pulse {
  0%, 100% { box-shadow: 0 0 8px rgba(0, 212, 255, 0.4); }
  50% { box-shadow: 0 0 16px rgba(0, 212, 255, 0.7); }
}
```

**DeviceSelectorComponent:**
- New dropdown "Preview Output" below existing device selectors
- Shows all output devices
- Selection persisted in store

## Files to Modify

### Backend (Rust)
- `src-tauri/src/application/mod.rs` - Add preview_engine module
- `src-tauri/src/application/preview_engine.rs` - New file
- `src-tauri/src/application/commands.rs` - Update preview_sound, add stop_preview
- `src-tauri/src/application/state.rs` - Add preview state
- `src-tauri/src/lib.rs` - Register new commands

### Frontend (Angular)
- `src/app/core/services/tauri.service.ts` - Add preview methods
- `src/app/core/services/soundboard.service.ts` - Add preview state/methods
- `src/app/core/services/mixer.service.ts` - Add preview device setting
- `src/app/features/soundboard/sound-pad/sound-pad.component.ts` - UI changes
- `src/app/features/devices/device-selector.component.ts` - Add preview dropdown

## Testing

- Preview plays on selected device
- Stop button stops playback
- New preview cancels current one
- Visual indicator appears/disappears correctly
- Preview device selection persists across restarts
- Preview works independently of main audio engine
