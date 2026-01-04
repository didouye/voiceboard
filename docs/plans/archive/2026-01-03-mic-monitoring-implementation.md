# Mic Monitoring Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable users to hear their microphone and pad sounds on the monitoring output device.

**Architecture:** Add a 3rd output stream in AudioEngine that sends mic audio (when monitoring enabled) and pad sounds to the preview device. Uses same ring buffer pattern as existing virtual output.

**Tech Stack:** Rust (cpal, ringbuf), Angular (signals), Tauri IPC

---

## Task 1: Add mic_monitoring field to Settings

**Files:**
- Modify: `src-tauri/src/domain/settings.rs:6-20`

**Step 1: Add mic_monitoring field to AudioSettings struct**

In `src-tauri/src/domain/settings.rs`, add the `mic_monitoring` field:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AudioSettings {
    pub input_device_id: Option<String>,
    pub output_device_id: Option<String>,
    pub preview_device_id: Option<String>,
    pub master_volume: f32,
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub mic_monitoring: bool,  // NEW
}
```

**Step 2: Update AudioSettings::new() default**

```rust
impl AudioSettings {
    pub fn new() -> Self {
        Self {
            input_device_id: None,
            output_device_id: None,
            preview_device_id: None,
            master_volume: 1.0,
            sample_rate: 48000,
            buffer_size: 1024,
            mic_monitoring: false,  // NEW - default off
        }
    }
}
```

**Step 3: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Compilation succeeds

**Step 4: Commit**

```bash
git add src-tauri/src/domain/settings.rs
git commit -m "feat(settings): add mic_monitoring field"
```

---

## Task 2: Update Settings DTOs in commands.rs

**Files:**
- Modify: `src-tauri/src/application/commands.rs:112-146`

**Step 1: Add mic_monitoring to AudioSettingsDto**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettingsDto {
    pub input_device_id: Option<String>,
    pub output_device_id: Option<String>,
    pub preview_device_id: Option<String>,
    pub master_volume: f32,
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub mic_monitoring: bool,  // NEW
}
```

**Step 2: Update From<&AudioSettings> for AudioSettingsDto**

```rust
impl From<&AudioSettings> for AudioSettingsDto {
    fn from(settings: &AudioSettings) -> Self {
        Self {
            input_device_id: settings.input_device_id.clone(),
            output_device_id: settings.output_device_id.clone(),
            preview_device_id: settings.preview_device_id.clone(),
            master_volume: settings.master_volume,
            sample_rate: settings.sample_rate,
            buffer_size: settings.buffer_size,
            mic_monitoring: settings.mic_monitoring,  // NEW
        }
    }
}
```

**Step 3: Update From<AudioSettingsDto> for AudioSettings**

```rust
impl From<AudioSettingsDto> for AudioSettings {
    fn from(dto: AudioSettingsDto) -> Self {
        Self {
            input_device_id: dto.input_device_id,
            output_device_id: dto.output_device_id,
            preview_device_id: dto.preview_device_id,
            master_volume: dto.master_volume,
            sample_rate: dto.sample_rate,
            buffer_size: dto.buffer_size,
            mic_monitoring: dto.mic_monitoring,  // NEW
        }
    }
}
```

**Step 4: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Compilation succeeds

**Step 5: Commit**

```bash
git add src-tauri/src/application/commands.rs
git commit -m "feat(commands): add mic_monitoring to DTOs"
```

---

## Task 3: Add set_mic_monitoring Tauri command

**Files:**
- Modify: `src-tauri/src/application/commands.rs` (after line 458)
- Modify: `src-tauri/src/lib.rs` (register command)

**Step 1: Add set_mic_monitoring command**

Add after `set_preview_device` function in `commands.rs`:

```rust
/// Set mic monitoring enabled/disabled
#[tauri::command]
pub async fn set_mic_monitoring(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    tracing::info!("Setting mic monitoring to: {}", enabled);

    {
        let mut settings = state.settings.write().await;
        settings.audio.mic_monitoring = enabled;
    }

    // Send to audio engine
    let engine = state.audio_engine.lock().await;
    engine
        .send_command(AudioEngineCommand::SetMicMonitoring(enabled))
        .map_err(|e| format!("Failed to set mic monitoring: {}", e))?;

    // Auto-save settings
    let settings = state.settings.read().await;
    let dto = AppSettingsDto::from(&*settings);
    drop(settings);

    let store = app.store(SETTINGS_STORE).map_err(|e| e.to_string())?;
    let _ = store.reload();
    store.set(
        SETTINGS_KEY,
        serde_json::to_value(&dto).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| {
        tracing::error!("Failed to save settings: {}", e);
        e.to_string()
    })?;

    tracing::info!("Mic monitoring saved: {}", enabled);
    Ok(())
}
```

**Step 2: Register command in lib.rs**

Find the `.invoke_handler(tauri::generate_handler![...])` call and add `set_mic_monitoring` to the list.

**Step 3: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Error about `AudioEngineCommand::SetMicMonitoring` not existing (expected, we add it in Task 4)

**Step 4: Commit**

```bash
git add src-tauri/src/application/commands.rs src-tauri/src/lib.rs
git commit -m "feat(commands): add set_mic_monitoring Tauri command"
```

---

## Task 4: Add SetMicMonitoring command to AudioEngine

**Files:**
- Modify: `src-tauri/src/application/audio_engine.rs:24-48`

**Step 1: Add SetMicMonitoring variant to AudioEngineCommand**

```rust
pub enum AudioEngineCommand {
    Start {
        input_device: String,
        output_device: String,
        sample_rate: u32,
        channels: u16,
    },
    Stop,
    PlaySound { id: String, samples: Vec<f32> },
    StopSound { id: String },
    SetMicVolume(f32),
    SetMasterVolume(f32),
    SetMicMuted(bool),
    SetMicMonitoring(bool),  // NEW
    SetMonitoringDevice(String),  // NEW
    Shutdown,
}
```

**Step 2: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Warning about unhandled match arms (expected, we handle in Task 5)

**Step 3: Commit**

```bash
git add src-tauri/src/application/audio_engine.rs
git commit -m "feat(audio-engine): add SetMicMonitoring command variant"
```

---

## Task 5: Implement monitoring output stream in AudioEngine

**Files:**
- Modify: `src-tauri/src/application/audio_engine.rs`

This is the core implementation. We need to:
1. Add atomic for mic_monitoring state
2. Add a second ring buffer for monitoring
3. Create a third output stream to the preview device
4. Route mic audio to monitoring when enabled
5. Route pad sounds to both outputs

**Step 1: Add monitoring state variables in run_engine_thread**

After line 211 (after `mic_muted`), add:

```rust
let mic_monitoring = Arc::new(AtomicBool::new(false));
let monitoring_device_name = Arc::new(Mutex::new(String::from("default")));
```

**Step 2: Add monitoring stream variable**

After line 198 (after `output_stream`), add:

```rust
let mut monitoring_stream: Option<cpal::Stream> = None;
```

**Step 3: Create second ring buffer for monitoring**

In the `AudioEngineCommand::Start` handler, after the first ring buffer creation (around line 393), add:

```rust
// Create second ring buffer for monitoring output
let rb_monitoring = HeapRb::<f32>::new(RING_BUFFER_SIZE);
let (producer_monitoring, consumer_monitoring) = rb_monitoring.split();
let producer_monitoring = Arc::new(Mutex::new(producer_monitoring));
let consumer_monitoring = Arc::new(Mutex::new(consumer_monitoring));
```

**Step 4: Update input callback to write to both ring buffers**

The input callback needs to also write to the monitoring ring buffer when mic_monitoring is enabled. Update the input callback:

```rust
let producer_monitoring_clone = producer_monitoring.clone();
let mic_monitoring_clone = mic_monitoring.clone();

// In the input callback, after writing to main producer:
if mic_monitoring_clone.load(Ordering::Relaxed) {
    if let Ok(mut prod) = producer_monitoring_clone.try_lock() {
        for &sample in data {
            let processed = if muted { 0.0 } else { sample * volume };
            let _ = prod.try_push(processed);
        }
    }
}
```

**Step 5: Build monitoring output stream**

After the main output stream is built and before starting streams, add the monitoring stream creation. This requires getting the preview device and building a similar output stream:

```rust
// Get monitoring device name
let monitoring_dev_name = {
    let name = monitoring_device_name.lock().unwrap();
    name.clone()
};

// Find monitoring device
if let Some(monitoring_dev) = find_device(&host, &monitoring_dev_name, false) {
    let consumer_monitoring_clone = consumer_monitoring.clone();
    let master_volume_monitoring = master_volume.clone();
    let audio_state_monitoring = audio_state.clone();
    let mic_monitoring_for_output = mic_monitoring.clone();

    if let Ok(monitoring_s) = monitoring_dev.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let master_vol = f32::from_bits(master_volume_monitoring.load(Ordering::Relaxed));
            let monitoring_enabled = mic_monitoring_for_output.load(Ordering::Relaxed);

            // Fill with mic input from monitoring ring buffer (if enabled)
            if monitoring_enabled {
                if let Ok(mut cons) = consumer_monitoring_clone.try_lock() {
                    for sample in data.iter_mut() {
                        *sample = cons.try_pop().unwrap_or(0.0);
                    }
                } else {
                    for sample in data.iter_mut() {
                        *sample = 0.0;
                    }
                }
            } else {
                for sample in data.iter_mut() {
                    *sample = 0.0;
                }
            }

            // Mix in playing sounds (always play on monitoring)
            if let Ok(mut state) = audio_state_monitoring.try_lock() {
                for (_id, sound) in state.playing_sounds.iter_mut() {
                    let remaining = sound.samples.len() - sound.position;
                    let to_mix = remaining.min(data.len());

                    for (i, sample) in data.iter_mut().take(to_mix).enumerate() {
                        // Note: position already advanced by main output, so we need separate tracking
                        // For simplicity, we re-read from same position (sounds play in sync)
                        let sound_pos = sound.position.saturating_sub(data.len()) + i;
                        if sound_pos < sound.samples.len() {
                            *sample = (*sample + sound.samples[sound_pos]).clamp(-1.0, 1.0);
                        }
                    }
                }
            }

            // Apply master volume
            for sample in data.iter_mut() {
                *sample = (*sample * master_vol).clamp(-1.0, 1.0);
            }
        },
        move |err| {
            tracing::error!("Monitoring stream error: {}", err);
        },
        None,
    ) {
        if let Err(e) = monitoring_s.play() {
            tracing::warn!("Failed to start monitoring stream: {}", e);
        } else {
            monitoring_stream = Some(monitoring_s);
            tracing::info!("Monitoring stream started on: {}", monitoring_dev_name);
        }
    }
}
```

**Step 6: Handle SetMicMonitoring command**

Add handler in the command match:

```rust
AudioEngineCommand::SetMicMonitoring(enabled) => {
    mic_monitoring.store(enabled, Ordering::Relaxed);
    tracing::info!("Mic monitoring set to: {}", enabled);
}

AudioEngineCommand::SetMonitoringDevice(device_name) => {
    if let Ok(mut name) = monitoring_device_name.lock() {
        *name = device_name.clone();
    }
    tracing::info!("Monitoring device set to: {}", device_name);
}
```

**Step 7: Stop monitoring stream on Stop command**

In the `AudioEngineCommand::Stop` handler, add:

```rust
if let Some(ref stream) = monitoring_stream {
    let _ = stream.pause();
}
monitoring_stream = None;
```

**Step 8: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Compilation succeeds

**Step 9: Commit**

```bash
git add src-tauri/src/application/audio_engine.rs
git commit -m "feat(audio-engine): implement monitoring output stream"
```

---

## Task 6: Pass monitoring device from settings on Start

**Files:**
- Modify: `src-tauri/src/application/commands.rs` (start_mixing function)

**Step 1: Update start_mixing to send monitoring device**

In `start_mixing`, after sending the Start command, send the monitoring device:

```rust
// Send monitoring device to audio engine
let preview_device = settings.audio.preview_device_id.clone().unwrap_or_else(|| "default".to_string());
engine
    .send_command(AudioEngineCommand::SetMonitoringDevice(preview_device))
    .ok();

// Restore mic monitoring state
if settings.audio.mic_monitoring {
    engine
        .send_command(AudioEngineCommand::SetMicMonitoring(true))
        .ok();
}
```

**Step 2: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Compilation succeeds

**Step 3: Commit**

```bash
git add src-tauri/src/application/commands.rs
git commit -m "feat(commands): pass monitoring device on mixer start"
```

---

## Task 7: Update frontend AudioSettings model

**Files:**
- Modify: `src/app/core/models/audio-device.model.ts:29-36`

**Step 1: Add micMonitoring to AudioSettings interface**

```typescript
export interface AudioSettings {
  inputDeviceId: string | null;
  outputDeviceId: string | null;
  previewDeviceId: string | null;
  masterVolume: number;
  sampleRate: number;
  bufferSize: number;
  micMonitoring: boolean;  // NEW
}
```

**Step 2: Commit**

```bash
git add src/app/core/models/audio-device.model.ts
git commit -m "feat(frontend): add micMonitoring to AudioSettings model"
```

---

## Task 8: Update TauriService settings mapping

**Files:**
- Modify: `src/app/core/services/tauri.service.ts:144-175`

**Step 1: Update mapSettings to include micMonitoring**

```typescript
private mapSettings(s: any): AppSettings {
  return {
    audio: {
      inputDeviceId: s.audio.input_device_id,
      outputDeviceId: s.audio.output_device_id,
      previewDeviceId: s.audio.preview_device_id,
      masterVolume: s.audio.master_volume,
      sampleRate: s.audio.sample_rate,
      bufferSize: s.audio.buffer_size,
      micMonitoring: s.audio.mic_monitoring ?? false  // NEW
    },
    startMinimized: s.start_minimized,
    autoStartMixing: s.auto_start_mixing
  };
}
```

**Step 2: Update unmapSettings to include micMonitoring**

```typescript
private unmapSettings(s: AppSettings): any {
  return {
    audio: {
      input_device_id: s.audio.inputDeviceId,
      output_device_id: s.audio.outputDeviceId,
      preview_device_id: s.audio.previewDeviceId,
      master_volume: s.audio.masterVolume,
      sample_rate: s.audio.sampleRate,
      buffer_size: s.audio.bufferSize,
      mic_monitoring: s.audio.micMonitoring  // NEW
    },
    start_minimized: s.startMinimized,
    auto_start_mixing: s.autoStartMixing
  };
}
```

**Step 3: Add setMicMonitoring method**

Add after `setPreviewDevice` method:

```typescript
/**
 * Set mic monitoring enabled/disabled
 */
async setMicMonitoring(enabled: boolean): Promise<void> {
  await invoke('set_mic_monitoring', { enabled });
}
```

**Step 4: Commit**

```bash
git add src/app/core/services/tauri.service.ts
git commit -m "feat(frontend): add micMonitoring to TauriService"
```

---

## Task 9: Add Mic Monitoring toggle to device-selector UI

**Files:**
- Modify: `src/app/features/devices/device-selector.component.ts`

**Step 1: Add micMonitoring computed signal**

After `selectedPreviewId` computed (around line 271):

```typescript
readonly micMonitoring = computed(() => this._settings()?.audio.micMonitoring ?? false);
```

**Step 2: Add toggle HTML after preview device select**

After the preview device `</select>` (around line 54), add:

```html
<!-- Mic Monitoring Toggle -->
<div class="monitoring-toggle">
  <label class="toggle-label">
    <span class="toggle-icon">🎤</span>
    <span class="toggle-text">Mic Monitoring</span>
    <div class="toggle-switch"
         [class.active]="micMonitoring()"
         (click)="toggleMicMonitoring()">
      <div class="toggle-slider"></div>
    </div>
  </label>
</div>
```

**Step 3: Add toggle styles**

Add to the styles array:

```css
.monitoring-toggle {
  margin-top: 12px;
  padding: 12px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 8px;
}

.toggle-label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.toggle-icon {
  font-size: 1rem;
}

.toggle-text {
  color: #ccc;
  flex: 1;
}

.toggle-switch {
  width: 48px;
  height: 24px;
  background: #333;
  border-radius: 12px;
  position: relative;
  transition: background 0.2s;
}

.toggle-switch.active {
  background: #7b2cbf;
}

.toggle-slider {
  width: 20px;
  height: 20px;
  background: #fff;
  border-radius: 50%;
  position: absolute;
  top: 2px;
  left: 2px;
  transition: transform 0.2s;
}

.toggle-switch.active .toggle-slider {
  transform: translateX(24px);
}
```

**Step 4: Add toggleMicMonitoring method**

Add after `onPreviewDeviceChange` method:

```typescript
async toggleMicMonitoring(): Promise<void> {
  const currentValue = this.micMonitoring();
  const newValue = !currentValue;

  try {
    await this.tauri.setMicMonitoring(newValue);

    // Update local state
    const settings = this._settings();
    if (settings) {
      this._settings.set({
        ...settings,
        audio: { ...settings.audio, micMonitoring: newValue }
      });
    }
  } catch (err) {
    console.error('Failed to set mic monitoring:', err);
  }
}
```

**Step 5: Verify build**

Run: `npm run build`
Expected: Build succeeds

**Step 6: Commit**

```bash
git add src/app/features/devices/device-selector.component.ts
git commit -m "feat(ui): add mic monitoring toggle to device selector"
```

---

## Task 10: Test the feature end-to-end

**Step 1: Start the application**

Run: `npm run tauri dev`

**Step 2: Manual test checklist**

- [ ] Select a preview output device (headphones)
- [ ] Enable mic monitoring toggle
- [ ] Speak into microphone - verify you hear yourself in headphones
- [ ] Disable mic monitoring - verify you no longer hear yourself
- [ ] Click a pad with a sound - verify sound plays in both virtual output AND headphones
- [ ] Preview a pad - verify sound only plays in headphones
- [ ] Restart the app - verify mic monitoring setting is persisted

**Step 3: Commit final changes (if any)**

```bash
git add -A
git commit -m "feat: complete mic monitoring feature"
```

---

## Task 11: Update ROADMAP.md

**Files:**
- Modify: `ROADMAP.md`

**Step 1: Mark mic monitoring as complete**

Change:
```markdown
- [ ] Mic monitoring on preview output - Hear your own microphone in preview
```

To:
```markdown
- [x] Mic monitoring on preview output - Hear your own microphone in preview
```

**Step 2: Commit**

```bash
git add ROADMAP.md
git commit -m "docs: mark mic monitoring as complete in roadmap"
```
