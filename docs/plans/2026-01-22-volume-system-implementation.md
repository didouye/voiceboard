# Volume System Redesign Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix volume slider visibility, persist all volumes, add Soundboard Volume control, and normalize sounds at import.

**Architecture:** Add `mic_volume` and `soundboard_volume` to settings with full persistence. Calculate effective sound volume in frontend before sending to backend. Normalize imported sounds to -3dB peak and store in AppData.

**Tech Stack:** Rust (Tauri backend), Angular (frontend), WAV encoding for normalized sounds.

---

### Task 1: Add volume fields to Rust settings

**Files:**
- Modify: `src-tauri/src/domain/settings.rs`

**Step 1: Add mic_volume and soundboard_volume fields**

In `AudioSettings` struct, add after `mic_monitoring`:

```rust
/// Microphone volume (0.0 to 2.0)
#[serde(default = "default_volume")]
pub mic_volume: f32,
/// Soundboard global volume (0.0 to 2.0)
#[serde(default = "default_volume")]
pub soundboard_volume: f32,
```

Add helper function before `AudioSettings`:

```rust
fn default_volume() -> f32 {
    1.0
}
```

**Step 2: Update AudioSettings::new()**

Add the new fields with default 1.0:

```rust
pub fn new() -> Self {
    Self {
        input_device_id: None,
        output_device_id: None,
        preview_device_id: None,
        master_volume: 1.0,
        sample_rate: 48000,
        buffer_size: 1024,
        mic_monitoring: false,
        global_hotkeys_enabled: true,
        mic_volume: 1.0,
        soundboard_volume: 1.0,
    }
}
```

**Step 3: Run tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml -p voiceboard-lib -- settings`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src-tauri/src/domain/settings.rs
git commit -m "feat(settings): add mic_volume and soundboard_volume fields"
```

---

### Task 2: Add volume fields to commands DTOs

**Files:**
- Modify: `src-tauri/src/application/commands.rs`

**Step 1: Update AudioSettingsDto**

Add after `global_hotkeys_enabled`:

```rust
#[serde(default = "default_volume")]
pub mic_volume: f32,
#[serde(default = "default_volume")]
pub soundboard_volume: f32,
```

Add helper function near other defaults:

```rust
fn default_volume() -> f32 {
    1.0
}
```

**Step 2: Update From<&AudioSettings> for AudioSettingsDto**

Add in the conversion:

```rust
mic_volume: settings.mic_volume,
soundboard_volume: settings.soundboard_volume,
```

**Step 3: Update From<AudioSettingsDto> for AudioSettings**

Add in the conversion:

```rust
mic_volume: dto.mic_volume,
soundboard_volume: dto.soundboard_volume,
```

**Step 4: Run cargo check**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: No errors

**Step 5: Commit**

```bash
git add src-tauri/src/application/commands.rs
git commit -m "feat(commands): add mic_volume and soundboard_volume to DTOs"
```

---

### Task 3: Fix set_master_volume persistence

**Files:**
- Modify: `src-tauri/src/application/commands.rs`

**Step 1: Add app parameter and store.save() to set_master_volume**

Replace the existing `set_master_volume` function:

```rust
/// Set master volume
#[tauri::command]
pub async fn set_master_volume(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    volume: f32,
) -> Result<(), String> {
    let clamped_volume = volume.clamp(0.0, 1.0);

    // Update mixer config
    {
        let mut config = state.mixer_config.write().await;
        config.master_volume = clamped_volume;
    }

    // Update settings
    {
        let mut settings = state.settings.write().await;
        settings.audio.master_volume = clamped_volume;
    }

    // Send to audio engine
    let engine = state.audio_engine.lock().await;
    engine
        .send_command(AudioEngineCommand::SetMasterVolume(clamped_volume))
        .map_err(|e| format!("Failed to set master volume: {}", e))?;

    // Persist to store
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
        tracing::error!("Failed to save master volume: {}", e);
        e.to_string()
    })?;

    tracing::info!("Master volume saved: {}", clamped_volume);
    Ok(())
}
```

**Step 2: Run cargo check**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: No errors

**Step 3: Commit**

```bash
git add src-tauri/src/application/commands.rs
git commit -m "fix(settings): persist master volume to store"
```

---

### Task 4: Add set_mic_volume command with persistence

**Files:**
- Modify: `src-tauri/src/application/commands.rs`

**Step 1: Replace existing set_mic_volume**

Replace the current `set_mic_volume` function with this version that persists:

```rust
/// Set microphone volume (0.0 - 2.0) with persistence
#[tauri::command]
pub async fn set_mic_volume(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    volume: f32,
) -> Result<(), String> {
    let clamped_volume = volume.clamp(0.0, 2.0);

    // Update settings
    {
        let mut settings = state.settings.write().await;
        settings.audio.mic_volume = clamped_volume;
    }

    // Send to audio engine
    let engine = state.audio_engine.lock().await;
    engine
        .send_command(AudioEngineCommand::SetMicVolume(clamped_volume))
        .map_err(|e| format!("Failed to set mic volume: {}", e))?;

    // Persist to store
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
        tracing::error!("Failed to save mic volume: {}", e);
        e.to_string()
    })?;

    tracing::info!("Mic volume saved: {}", clamped_volume);
    Ok(())
}
```

**Step 2: Run cargo check**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: No errors

**Step 3: Commit**

```bash
git add src-tauri/src/application/commands.rs
git commit -m "feat(settings): add mic volume persistence"
```

---

### Task 5: Add set_soundboard_volume command

**Files:**
- Modify: `src-tauri/src/application/commands.rs`

**Step 1: Add new command after set_mic_volume**

```rust
/// Set soundboard global volume (0.0 - 2.0) with persistence
#[tauri::command]
pub async fn set_soundboard_volume(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    volume: f32,
) -> Result<(), String> {
    let clamped_volume = volume.clamp(0.0, 2.0);

    // Update settings
    {
        let mut settings = state.settings.write().await;
        settings.audio.soundboard_volume = clamped_volume;
    }

    // Persist to store
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
        tracing::error!("Failed to save soundboard volume: {}", e);
        e.to_string()
    })?;

    tracing::info!("Soundboard volume saved: {}", clamped_volume);
    Ok(())
}
```

**Step 2: Register command in lib.rs**

In `src-tauri/src/lib.rs`, add `set_soundboard_volume` to the invoke_handler list.

**Step 3: Run cargo check**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: No errors

**Step 4: Commit**

```bash
git add src-tauri/src/application/commands.rs src-tauri/src/lib.rs
git commit -m "feat(settings): add soundboard volume command with persistence"
```

---

### Task 6: Add sound normalization command

**Files:**
- Modify: `src-tauri/src/application/commands.rs`

**Step 1: Add import_and_normalize_sound command**

Add after `import_sound_with_hash`:

```rust
/// Import and normalize a sound file to -3dB peak
/// Copies the normalized file to AppData/sounds/{hash}.wav
#[tauri::command]
pub async fn import_and_normalize_sound(
    app: tauri::AppHandle,
    path: String,
) -> Result<ImportedSoundDto, String> {
    use rodio::Source;
    use sha2::{Digest, Sha256};
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;

    // Read file and calculate hash of ORIGINAL
    let original_data = std::fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))?;
    let hash = format!("{:x}", Sha256::digest(&original_data));

    // Decode audio
    let file = File::open(&path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);
    let decoder =
        rodio::Decoder::new(reader).map_err(|e| format!("Failed to decode audio: {}", e))?;

    let sample_rate = decoder.sample_rate();
    let channels = decoder.channels();

    // Collect samples as f32
    let samples: Vec<f32> = decoder.convert_samples::<f32>().collect();

    if samples.is_empty() {
        return Err("Audio file contains no samples".to_string());
    }

    // Find peak amplitude
    let peak = samples.iter().fold(0.0f32, |max, &s| max.max(s.abs()));

    // Calculate gain for -3dB peak (0.708 = 10^(-3/20))
    let target_peak = 0.708f32;
    let gain = if peak > 0.0 { target_peak / peak } else { 1.0 };

    // Apply gain (normalize)
    let normalized: Vec<f32> = samples.iter().map(|&s| (s * gain).clamp(-1.0, 1.0)).collect();

    // Get sounds directory in AppData
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let sounds_dir = app_data_dir.join("sounds");
    std::fs::create_dir_all(&sounds_dir)
        .map_err(|e| format!("Failed to create sounds directory: {}", e))?;

    // Save as WAV
    let output_path = sounds_dir.join(format!("{}.wav", hash));

    // Write WAV file
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create(&output_path, spec)
        .map_err(|e| format!("Failed to create WAV file: {}", e))?;

    for sample in &normalized {
        writer
            .write_sample(*sample)
            .map_err(|e| format!("Failed to write sample: {}", e))?;
    }

    writer
        .finalize()
        .map_err(|e| format!("Failed to finalize WAV: {}", e))?;

    // Calculate duration
    let duration = samples.len() as f64 / (sample_rate as f64 * channels as f64);

    // Extract filename
    let name = Path::new(&path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    tracing::info!(
        "Normalized sound: {} (peak {:.3} -> {:.3}, gain {:.2}x)",
        name,
        peak,
        target_peak,
        gain
    );

    Ok(ImportedSoundDto {
        hash,
        name,
        path: output_path.to_string_lossy().to_string(),
        duration,
    })
}
```

**Step 2: Add hound dependency to Cargo.toml**

In `src-tauri/Cargo.toml`, add:

```toml
hound = "3.5"
```

**Step 3: Register command in lib.rs**

Add `import_and_normalize_sound` to invoke_handler.

**Step 4: Run cargo check**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: No errors

**Step 5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/application/commands.rs src-tauri/src/lib.rs
git commit -m "feat(audio): add sound normalization at import (-3dB peak)"
```

---

### Task 7: Update frontend AudioSettings model

**Files:**
- Modify: `src/app/core/models/audio-device.model.ts`

**Step 1: Add new fields to AudioSettings interface**

Add after `globalHotkeysEnabled`:

```typescript
micVolume: number;
soundboardVolume: number;
```

**Step 2: Commit**

```bash
git add src/app/core/models/audio-device.model.ts
git commit -m "feat(models): add micVolume and soundboardVolume to AudioSettings"
```

---

### Task 8: Update TauriService with new methods

**Files:**
- Modify: `src/app/core/services/tauri.service.ts`

**Step 1: Add setMicVolume method**

```typescript
async setMicVolume(volume: number): Promise<void> {
  await invoke('set_mic_volume', { volume });
}
```

**Step 2: Add setSoundboardVolume method**

```typescript
async setSoundboardVolume(volume: number): Promise<void> {
  await invoke('set_soundboard_volume', { volume });
}
```

**Step 3: Add importAndNormalizeSound method**

```typescript
async importAndNormalizeSound(path: string): Promise<{ hash: string; name: string; path: string; duration: number }> {
  return invoke('import_and_normalize_sound', { path });
}
```

**Step 4: Add importMultipleAndNormalize method**

```typescript
async importMultipleAndNormalize(paths: string[]): Promise<Array<{ ok: { hash: string; name: string; path: string; duration: number } } | { err: string }>> {
  const results = [];
  for (const path of paths) {
    try {
      const result = await this.importAndNormalizeSound(path);
      results.push({ ok: result });
    } catch (err) {
      results.push({ err: String(err) });
    }
  }
  return results;
}
```

**Step 5: Commit**

```bash
git add src/app/core/services/tauri.service.ts
git commit -m "feat(tauri): add volume and normalize sound methods"
```

---

### Task 9: Update MixerService with soundboardVolume

**Files:**
- Modify: `src/app/core/services/mixer.service.ts`

**Step 1: Add soundboardVolume signal**

After `masterVolume` computed:

```typescript
readonly soundboardVolume = computed(() => {
  // Load from settings on init, default to 1.0
  return 1.0; // Will be set from settings
});
```

**Step 2: Add _soundboardVolume private signal**

```typescript
private _soundboardVolume = signal(1.0);
readonly soundboardVolume = this._soundboardVolume.asReadonly();
```

**Step 3: Add setSoundboardVolume method**

```typescript
async setSoundboardVolume(volume: number): Promise<void> {
  try {
    await this.tauri.setSoundboardVolume(volume);
    this._soundboardVolume.set(volume);
  } catch (err) {
    this._error.set(err instanceof Error ? err.message : 'Failed to set soundboard volume');
  }
}
```

**Step 4: Load soundboard volume in initialize()**

In the initialize method, after loading settings:

```typescript
// Load soundboard volume from settings
const settings = await this.tauri.loadSettings();
this._soundboardVolume.set(settings.audio.soundboardVolume ?? 1.0);
```

**Step 5: Commit**

```bash
git add src/app/core/services/mixer.service.ts
git commit -m "feat(mixer): add soundboardVolume signal and setter"
```

---

### Task 10: Update SoundboardService to use effective volume

**Files:**
- Modify: `src/app/core/services/soundboard.service.ts`

**Step 1: Inject MixerService**

Add to constructor:

```typescript
constructor(
  private tauri: TauriService,
  private fuzzySearch: FuzzySearchService,
  private mixer: MixerService  // Add this
) {
```

**Step 2: Update playSound to calculate effective volume**

Replace the `playSound` method:

```typescript
async playSound(soundId: string): Promise<void> {
  const sound = this._sounds().get(soundId);
  if (!sound) return;

  try {
    if (sound.isPlaying) {
      await this.stopSound(soundId);
      return;
    }

    // Calculate effective volume:
    // If sound volume was modified (≠ 1.0), use it
    // Otherwise, use global soundboard volume
    const effectiveVolume = sound.volume !== 1.0
      ? sound.volume
      : this.mixer.soundboardVolume();

    await this.tauri.playSound(soundId, sound.path, effectiveVolume, sound.speed);
    this.updateSound(soundId, { isPlaying: true });
  } catch (err) {
    console.error('Failed to play sound:', err);
    this._error.set('Failed to play sound');
  }
}
```

**Step 3: Update import methods to use normalization**

Replace `importSoundWithHash` calls with `importAndNormalizeSound`:

In `importSound()`:
```typescript
const imported = await this.tauri.importAndNormalizeSound(path);
```

In `importMultipleSounds()`:
```typescript
const importResults = await this.tauri.importMultipleAndNormalize(paths);
```

In `importSoundsFromPaths()`:
```typescript
const importResults = await this.tauri.importMultipleAndNormalize(paths);
```

**Step 4: Commit**

```bash
git add src/app/core/services/soundboard.service.ts
git commit -m "feat(soundboard): use effective volume and normalize imports"
```

---

### Task 11: Fix volume slider CSS and add Soundboard Volume slider

**Files:**
- Modify: `src/app/shared/components/settings-popup/settings-popup.component.ts`

**Step 1: Add soundboardVolume signal**

```typescript
private _soundboardVolume = signal(1.0);
readonly soundboardVolume = this._soundboardVolume.asReadonly();
```

**Step 2: Load volumes in loadData()**

After loading settings:

```typescript
// Load volume values
this._micVolume.set(settings.audio.micVolume ?? 1.0);
this._soundboardVolume.set(settings.audio.soundboardVolume ?? 1.0);
```

**Step 3: Add onSoundboardVolumeChange handler**

```typescript
async onSoundboardVolumeChange(event: Event): Promise<void> {
  const input = event.target as HTMLInputElement;
  const volume = parseFloat(input.value) / 100;
  this._soundboardVolume.set(volume);
  await this.mixer.setSoundboardVolume(volume);
}
```

**Step 4: Update mic volume to persist**

```typescript
async onMicVolumeChange(event: Event): Promise<void> {
  const input = event.target as HTMLInputElement;
  const volume = parseFloat(input.value) / 100;
  this._micVolume.set(volume);
  await this.tauri.setMicVolume(volume);
}
```

**Step 5: Update template with gradient sliders**

Replace the Mixer section with:

```html
<!-- Mixer Section -->
<div class="mb-6">
  <h3 class="text-xs font-semibold text-text-muted uppercase tracking-wider mb-4">Mixer</h3>

  <!-- Mic Volume -->
  <div class="mb-4">
    <div class="flex items-center justify-between mb-2">
      <label class="text-sm text-text-secondary">Mic Volume</label>
      <div class="flex items-center gap-2">
        <span class="text-sm text-text-primary font-mono">{{ Math.round(micVolume() * 100) }}%</span>
        <button
          class="w-8 h-8 flex items-center justify-center rounded-lg transition-colors"
          [class]="micMuted() ? 'bg-status-error text-white' : 'bg-surface-hover text-text-secondary hover:text-text-primary'"
          (click)="toggleMicMute()"
        >
          {{ micMuted() ? '&#128263;' : '&#128264;' }}
        </button>
      </div>
    </div>
    <input
      type="range"
      min="0"
      max="200"
      [value]="micVolume() * 100"
      (input)="onMicVolumeChange($event)"
      class="w-full h-2 rounded-full appearance-none cursor-pointer
             [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:h-4
             [&::-webkit-slider-thumb]:bg-white [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:cursor-pointer
             [&::-webkit-slider-thumb]:transition-transform [&::-webkit-slider-thumb]:hover:scale-110"
      [style.background]="'linear-gradient(to right, #9d4edd 0%, #9d4edd ' + (micVolume() * 50) + '%, #12121a ' + (micVolume() * 50) + '%, #12121a 100%)'"
    >
  </div>

  <!-- Soundboard Volume -->
  <div class="mb-4">
    <div class="flex items-center justify-between mb-2">
      <label class="text-sm text-text-secondary">Soundboard Volume</label>
      <span class="text-sm text-text-primary font-mono">{{ Math.round(soundboardVolume() * 100) }}%</span>
    </div>
    <input
      type="range"
      min="0"
      max="200"
      [value]="soundboardVolume() * 100"
      (input)="onSoundboardVolumeChange($event)"
      class="w-full h-2 rounded-full appearance-none cursor-pointer
             [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:h-4
             [&::-webkit-slider-thumb]:bg-white [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:cursor-pointer
             [&::-webkit-slider-thumb]:transition-transform [&::-webkit-slider-thumb]:hover:scale-110"
      [style.background]="'linear-gradient(to right, #9d4edd 0%, #9d4edd ' + (soundboardVolume() * 50) + '%, #12121a ' + (soundboardVolume() * 50) + '%, #12121a 100%)'"
    >
  </div>

  <!-- Master Volume -->
  <div class="mb-4">
    <div class="flex items-center justify-between mb-2">
      <label class="text-sm text-text-secondary">Master Volume</label>
      <span class="text-sm text-text-primary font-mono">{{ Math.round(masterVolume() * 100) }}%</span>
    </div>
    <input
      type="range"
      min="0"
      max="100"
      [value]="masterVolume() * 100"
      (input)="onMasterVolumeChange($event)"
      class="w-full h-2 rounded-full appearance-none cursor-pointer
             [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:h-4
             [&::-webkit-slider-thumb]:bg-white [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:cursor-pointer
             [&::-webkit-slider-thumb]:transition-transform [&::-webkit-slider-thumb]:hover:scale-110"
      [style.background]="'linear-gradient(to right, #9d4edd 0%, #9d4edd ' + (masterVolume() * 100) + '%, #12121a ' + (masterVolume() * 100) + '%, #12121a 100%)'"
    >
  </div>

  <!-- Mic Monitoring -->
  <div class="flex items-center justify-between py-3 px-4 bg-background rounded-lg">
    <span class="text-sm text-text-secondary">Mic Monitoring</span>
    <button
      class="w-12 h-6 rounded-full transition-colors relative"
      [class]="micMonitoring() ? 'bg-accent' : 'bg-surface'"
      (click)="toggleMicMonitoring()"
    >
      <div
        class="absolute top-1 w-4 h-4 bg-white rounded-full transition-transform"
        [class]="micMonitoring() ? 'left-7' : 'left-1'"
      ></div>
    </button>
  </div>
</div>
```

**Step 6: Commit**

```bash
git add src/app/shared/components/settings-popup/settings-popup.component.ts
git commit -m "feat(ui): add soundboard volume slider and fix slider visibility"
```

---

### Task 12: Apply mic volume on startup

**Files:**
- Modify: `src/app/core/services/mixer.service.ts`

**Step 1: Apply mic volume in initialize()**

After loading settings and before auto-start:

```typescript
// Apply mic volume to audio engine
if (settings.audio.micVolume !== undefined) {
  await this.tauri.setMicVolume(settings.audio.micVolume);
}
```

**Step 2: Commit**

```bash
git add src/app/core/services/mixer.service.ts
git commit -m "fix(mixer): apply mic volume on startup"
```

---

### Task 13: Test and verify

**Step 1: Build and run**

```bash
npm run tauri dev
```

**Step 2: Manual verification checklist**

- [ ] Volume sliders show colored progress bar
- [ ] Mic Volume persists after restart
- [ ] Soundboard Volume persists after restart
- [ ] Master Volume persists after restart
- [ ] Importing a sound creates normalized WAV in AppData/sounds/
- [ ] Sound with custom volume (≠100%) uses that volume
- [ ] Sound with default volume (100%) uses Soundboard Volume

**Step 3: Final commit**

```bash
git add -A
git commit -m "feat: complete volume system redesign

- Fix volume slider visibility with CSS gradients
- Persist mic, soundboard, and master volumes
- Add Soundboard Volume control
- Normalize sounds at import to -3dB peak
- Store normalized sounds in AppData/sounds/"
```

---

### Task 14: Archive the plan

**Step 1: Move plan to archive**

```bash
mv docs/plans/2026-01-22-volume-system-redesign.md docs/plans/archive/
mv docs/plans/2026-01-22-volume-system-implementation.md docs/plans/archive/
git add docs/plans/
git commit -m "docs: archive volume system plans"
```
