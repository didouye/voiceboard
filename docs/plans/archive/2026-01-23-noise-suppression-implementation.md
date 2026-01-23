# Noise Suppression Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add real-time noise suppression on microphone input using nnnoiseless (pure Rust RNNoise port).

**Architecture:** The denoiser processes audio in the input callback after mono conversion, before pushing to the ring buffer. A 480-sample frame buffer accumulates samples for batch processing. Settings are persisted and exposed via Tauri commands.

**Tech Stack:** nnnoiseless 0.5.2, Rust, Tauri, Angular

---

## Task 1: Add nnnoiseless dependency

**Files:**
- Modify: `src-tauri/Cargo.toml:37` (after ringbuf)

**Step 1: Add the dependency**

In `src-tauri/Cargo.toml`, after line 37 (`ringbuf = "0.4"`), add:

```toml
nnnoiseless = "0.5.2"            # Real-time noise suppression (RNNoise port)
```

**Step 2: Verify compilation**

Run:
```bash
cd src-tauri && cargo check
```

Expected: Compiles successfully with nnnoiseless downloaded.

**Step 3: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "chore: add nnnoiseless dependency for noise suppression"
```

---

## Task 2: Add noise_suppression_enabled to AudioSettings

**Files:**
- Modify: `src-tauri/src/domain/settings.rs:5-7` (add default function)
- Modify: `src-tauri/src/domain/settings.rs:39-40` (add field to AudioSettings)
- Modify: `src-tauri/src/domain/settings.rs:54-55` (add to new())

**Step 1: Add default function**

After line 10 (`default_volume`), add:

```rust
fn default_noise_suppression() -> bool {
    true
}
```

**Step 2: Add field to AudioSettings struct**

After line 39 (`soundboard_volume`), add:

```rust
    /// Enable noise suppression on microphone input
    #[serde(default = "default_noise_suppression")]
    pub noise_suppression_enabled: bool,
```

**Step 3: Update AudioSettings::new()**

After line 54 (`soundboard_volume: 1.0,`), add:

```rust
            noise_suppression_enabled: true,
```

**Step 4: Run tests**

```bash
cd src-tauri && cargo test settings
```

Expected: All existing settings tests pass.

**Step 5: Add test for noise_suppression_enabled**

Add this test at the end of the `mod tests` block (before the closing `}`):

```rust
    #[test]
    fn test_audio_settings_noise_suppression_default() {
        let settings = AudioSettings::new();
        assert!(settings.noise_suppression_enabled);
    }

    #[test]
    fn test_audio_settings_noise_suppression_serialization() {
        let mut settings = AudioSettings::new();
        settings.noise_suppression_enabled = false;

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("noise_suppression_enabled"));

        let deserialized: AudioSettings = serde_json::from_str(&json).unwrap();
        assert!(!deserialized.noise_suppression_enabled);
    }

    #[test]
    fn test_audio_settings_noise_suppression_default_on_missing() {
        // Old settings without the field should default to true
        let json = r#"{"input_device_id":null,"output_device_id":null,"preview_device_id":null,"master_volume":1.0,"sample_rate":48000,"buffer_size":1024,"mic_monitoring":false}"#;
        let settings: AudioSettings = serde_json::from_str(json).unwrap();
        assert!(settings.noise_suppression_enabled);
    }
```

**Step 6: Run new tests**

```bash
cd src-tauri && cargo test noise_suppression
```

Expected: 3 new tests pass.

**Step 7: Commit**

```bash
git add src-tauri/src/domain/settings.rs
git commit -m "feat(settings): add noise_suppression_enabled field"
```

---

## Task 3: Add Tauri commands for noise suppression

**Files:**
- Modify: `src-tauri/src/application/commands.rs` (add commands at end)
- Modify: `src-tauri/src/lib.rs:252-328` (register commands)

**Step 1: Add commands to commands.rs**

At the end of `src-tauri/src/application/commands.rs`, before the `#[cfg(test)]` block, add:

```rust
// ============================================================================
// Noise Suppression
// ============================================================================

/// Get noise suppression enabled state
#[tauri::command]
pub fn get_noise_suppression(state: State<AppState>) -> bool {
    let settings = state.settings.blocking_read();
    settings.audio.noise_suppression_enabled
}

/// Set noise suppression enabled state
#[tauri::command]
pub fn set_noise_suppression(
    app: tauri::AppHandle,
    state: State<AppState>,
    enabled: bool,
) -> Result<(), String> {
    // Update in-memory state
    {
        let mut settings = state.settings.blocking_write();
        settings.audio.noise_suppression_enabled = enabled;
    }

    // Persist to store
    let store = app
        .store(SETTINGS_STORE)
        .map_err(|e| format!("Failed to open store: {}", e))?;

    let settings = state.settings.blocking_read();
    store
        .set(SETTINGS_KEY, serde_json::to_value(&*settings).unwrap());
    store.save().map_err(|e| format!("Failed to save: {}", e))?;

    // Emit event for frontend
    let _ = app.emit("noise-suppression-changed", enabled);

    tracing::info!(enabled = enabled, "Noise suppression toggled");
    Ok(())
}
```

**Step 2: Export commands in lib.rs**

In `src-tauri/src/lib.rs`, add imports after line 77 (`set_debug_mode,`):

```rust
        get_noise_suppression,
        set_noise_suppression,
```

**Step 3: Register commands in invoke_handler**

In `src-tauri/src/lib.rs`, after line 317 (`set_debug_mode,`), add:

```rust
            // Noise suppression
            get_noise_suppression,
            set_noise_suppression,
```

**Step 4: Verify compilation**

```bash
cd src-tauri && cargo check
```

Expected: Compiles successfully.

**Step 5: Commit**

```bash
git add src-tauri/src/application/commands.rs src-tauri/src/lib.rs
git commit -m "feat(commands): add get/set_noise_suppression Tauri commands"
```

---

## Task 4: Implement NoiseFilter struct

**Files:**
- Create: `src-tauri/src/application/noise_filter.rs`
- Modify: `src-tauri/src/application/mod.rs` (add module)

**Step 1: Create noise_filter.rs**

Create `src-tauri/src/application/noise_filter.rs`:

```rust
//! Noise suppression filter using nnnoiseless (RNNoise port)
//!
//! Processes audio in 480-sample frames at 48kHz.

use nnnoiseless::DenoiseState;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Frame size required by nnnoiseless (10ms at 48kHz)
pub const DENOISE_FRAME_SIZE: usize = 480;

/// Real-time noise suppression filter
pub struct NoiseFilter {
    /// The denoiser state (must persist between calls)
    denoiser: DenoiseState<'static>,
    /// Buffer to accumulate samples until we have a full frame
    buffer: Vec<f32>,
    /// Output buffer for processed samples
    output_buffer: Vec<f32>,
    /// Whether noise suppression is enabled
    enabled: Arc<AtomicBool>,
}

impl NoiseFilter {
    /// Create a new noise filter
    pub fn new(enabled: Arc<AtomicBool>) -> Self {
        Self {
            denoiser: DenoiseState::new(),
            buffer: Vec::with_capacity(DENOISE_FRAME_SIZE),
            output_buffer: Vec::new(),
            enabled,
        }
    }

    /// Process a single sample, returning processed samples when a full frame is ready
    ///
    /// Call this for each input sample. When enough samples have accumulated (480),
    /// the filter processes them and returns the denoised samples.
    /// Returns an empty slice if not enough samples yet.
    pub fn process_sample(&mut self, sample: f32) -> &[f32] {
        // Clear output buffer
        self.output_buffer.clear();

        // If disabled, pass through immediately
        if !self.enabled.load(Ordering::Relaxed) {
            self.output_buffer.push(sample);
            return &self.output_buffer;
        }

        // Accumulate sample
        self.buffer.push(sample);

        // Process when we have a full frame
        if self.buffer.len() >= DENOISE_FRAME_SIZE {
            // Process the frame in place
            self.denoiser.process_frame(&mut self.buffer);

            // Move processed samples to output
            self.output_buffer.extend(self.buffer.drain(..));
        }

        &self.output_buffer
    }

    /// Flush any remaining samples in the buffer (for shutdown)
    pub fn flush(&mut self) -> Vec<f32> {
        if self.buffer.is_empty() {
            return Vec::new();
        }

        // Pad with zeros to complete the frame
        while self.buffer.len() < DENOISE_FRAME_SIZE {
            self.buffer.push(0.0);
        }

        self.denoiser.process_frame(&mut self.buffer);
        std::mem::take(&mut self.buffer)
    }

    /// Check if filter is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Set enabled state
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_filter_creation() {
        let enabled = Arc::new(AtomicBool::new(true));
        let filter = NoiseFilter::new(enabled);
        assert!(filter.is_enabled());
    }

    #[test]
    fn test_noise_filter_disabled_passthrough() {
        let enabled = Arc::new(AtomicBool::new(false));
        let mut filter = NoiseFilter::new(enabled);

        // When disabled, samples should pass through immediately
        let output = filter.process_sample(0.5);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0], 0.5);
    }

    #[test]
    fn test_noise_filter_enabled_buffering() {
        let enabled = Arc::new(AtomicBool::new(true));
        let mut filter = NoiseFilter::new(enabled);

        // First 479 samples should buffer (return empty)
        for i in 0..479 {
            let output = filter.process_sample(0.1);
            assert!(
                output.is_empty(),
                "Sample {} should buffer, got {} samples",
                i,
                output.len()
            );
        }

        // 480th sample should trigger processing
        let output = filter.process_sample(0.1);
        assert_eq!(output.len(), DENOISE_FRAME_SIZE);
    }

    #[test]
    fn test_noise_filter_toggle() {
        let enabled = Arc::new(AtomicBool::new(true));
        let filter = NoiseFilter::new(enabled);

        assert!(filter.is_enabled());
        filter.set_enabled(false);
        assert!(!filter.is_enabled());
        filter.set_enabled(true);
        assert!(filter.is_enabled());
    }

    #[test]
    fn test_noise_filter_reduces_noise() {
        let enabled = Arc::new(AtomicBool::new(true));
        let mut filter = NoiseFilter::new(enabled);

        // Generate white noise (random-ish values)
        let noise: Vec<f32> = (0..DENOISE_FRAME_SIZE)
            .map(|i| ((i * 7919) % 1000) as f32 / 1000.0 - 0.5)
            .collect();

        // Calculate input RMS
        let input_rms: f32 = (noise.iter().map(|s| s * s).sum::<f32>() / noise.len() as f32).sqrt();

        // Process the noise
        let mut output = Vec::new();
        for sample in noise {
            output.extend(filter.process_sample(sample));
        }

        // Calculate output RMS
        let output_rms: f32 =
            (output.iter().map(|s| s * s).sum::<f32>() / output.len() as f32).sqrt();

        // Output should have lower RMS than input (noise reduced)
        assert!(
            output_rms < input_rms,
            "Noise should be reduced: input_rms={}, output_rms={}",
            input_rms,
            output_rms
        );
    }

    #[test]
    fn test_noise_filter_flush() {
        let enabled = Arc::new(AtomicBool::new(true));
        let mut filter = NoiseFilter::new(enabled);

        // Add some samples (less than a full frame)
        for _ in 0..100 {
            filter.process_sample(0.1);
        }

        // Flush should return padded frame
        let flushed = filter.flush();
        assert_eq!(flushed.len(), DENOISE_FRAME_SIZE);
    }
}
```

**Step 2: Add module to mod.rs**

In `src-tauri/src/application/mod.rs`, add after the other pub mod declarations:

```rust
pub mod noise_filter;
```

**Step 3: Run tests**

```bash
cd src-tauri && cargo test noise_filter
```

Expected: All 6 tests pass.

**Step 4: Commit**

```bash
git add src-tauri/src/application/noise_filter.rs src-tauri/src/application/mod.rs
git commit -m "feat(audio): add NoiseFilter struct with nnnoiseless"
```

---

## Task 5: Integrate NoiseFilter into AudioEngine

**Files:**
- Modify: `src-tauri/src/application/audio_engine.rs`

**Step 1: Add import**

At the top of `audio_engine.rs`, after line 14 (`use std::sync::Arc`), add:

```rust
use crate::application::noise_filter::{NoiseFilter, DENOISE_FRAME_SIZE};
```

**Step 2: Add noise_suppression_enabled to AudioEngineCommand::Start**

Modify the `Start` variant (around line 28-33) to include the new field:

```rust
    /// Start mixing
    Start {
        input_device: String,
        output_device: String,
        sample_rate: u32,
        channels: u16,
        noise_suppression_enabled: bool,
    },
```

**Step 3: Add SetNoiseSuppression command**

After `SetMonitoringDevice` command (around line 56), add:

```rust
    /// Enable/disable noise suppression
    SetNoiseSuppression(bool),
```

**Step 4: Create noise filter in Start handler**

In the `AudioEngineCommand::Start` handler (around line 246), update the destructuring:

```rust
                    AudioEngineCommand::Start {
                        input_device,
                        output_device,
                        sample_rate: _,
                        channels: _,
                        noise_suppression_enabled,
                    } => {
```

**Step 5: Create noise filter before input stream**

After the atomic variable declarations (around line 238), add:

```rust
                        // Create noise suppression filter
                        let noise_enabled = Arc::new(AtomicBool::new(noise_suppression_enabled));
                        let noise_enabled_for_input = noise_enabled.clone();
```

**Step 6: Modify input callback to use noise filter**

The input callback (starting around line 462) needs to be modified. Replace the sample processing logic inside the callback with:

```rust
                        // Create noise filter for this input stream
                        let noise_filter = Arc::new(std::sync::Mutex::new(
                            NoiseFilter::new(noise_enabled_for_input.clone())
                        ));
                        let noise_filter_clone = noise_filter.clone();
```

Then in the callback itself, modify the processing loop (around line 489-502):

```rust
                                if let Ok(mut prod) = producer_clone.try_lock() {
                                    if let Ok(mut filter) = noise_filter_clone.try_lock() {
                                        for frame in 0..num_frames {
                                            // Average all channels to produce mono sample
                                            let mut sum = 0.0f32;
                                            for ch in 0..input_ch as usize {
                                                let idx = frame * input_ch as usize + ch;
                                                sum += data.get(idx).copied().unwrap_or(0.0);
                                            }
                                            let mono_sample = sum / input_ch as f32;

                                            // Process through noise filter
                                            let filtered_samples = filter.process_sample(mono_sample);

                                            // Push filtered samples to ring buffer
                                            for &sample in filtered_samples {
                                                let processed = if muted { 0.0 } else { sample * volume };
                                                sum_squares += processed * processed;
                                                let _ = prod.try_push(processed);
                                            }
                                        }
                                    }
                                }
```

**Step 7: Store noise_enabled Arc for command handling**

After storing the streams (around line 965), store the noise_enabled Arc:

```rust
                        // Store noise suppression control
                        // (We need to store this outside the callback so SetNoiseSuppression can access it)
```

Note: For simplicity, we'll handle SetNoiseSuppression by requiring a restart. Add this to the command handler section:

```rust
                    AudioEngineCommand::SetNoiseSuppression(enabled) => {
                        tracing::info!("Noise suppression set to: {} (requires restart to take effect)", enabled);
                        // The setting is applied on next Start command
                        // Frontend should restart mixing when this changes
                    }
```

**Step 8: Verify compilation**

```bash
cd src-tauri && cargo check
```

**Step 9: Run existing tests**

```bash
cd src-tauri && cargo test audio_engine
```

Expected: All existing tests pass.

**Step 10: Commit**

```bash
git add src-tauri/src/application/audio_engine.rs
git commit -m "feat(audio): integrate NoiseFilter into input callback"
```

---

## Task 6: Update start_mixing command

**Files:**
- Modify: `src-tauri/src/application/commands.rs` (start_mixing function)

**Step 1: Find and update start_mixing**

Locate the `start_mixing` function and update it to read `noise_suppression_enabled` from settings and pass it to the engine:

```rust
#[tauri::command]
pub fn start_mixing(app: tauri::AppHandle, state: State<AppState>) -> Result<(), String> {
    let settings = state.settings.blocking_read();

    let input_device = settings
        .audio
        .input_device_id
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let output_device = settings
        .audio
        .output_device_id
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let sample_rate = settings.audio.sample_rate;
    let noise_suppression_enabled = settings.audio.noise_suppression_enabled;

    drop(settings); // Release lock before sending command

    let engine = state.audio_engine.blocking_lock();
    engine
        .send_command(AudioEngineCommand::Start {
            input_device,
            output_device,
            sample_rate,
            channels: 2,
            noise_suppression_enabled,
        })
        .map_err(|e| e.to_string())?;

    // ... rest of the function
```

**Step 2: Verify compilation**

```bash
cd src-tauri && cargo check
```

**Step 3: Commit**

```bash
git add src-tauri/src/application/commands.rs
git commit -m "feat(commands): pass noise_suppression_enabled to audio engine"
```

---

## Task 7: Add frontend TauriService methods

**Files:**
- Modify: `src/app/core/services/tauri.service.ts`

**Step 1: Add methods to TauriService**

Add these methods to the TauriService class:

```typescript
  // =========================================================================
  // Noise Suppression
  // =========================================================================

  /**
   * Get noise suppression enabled state
   */
  async getNoiseSuppression(): Promise<boolean> {
    if (this.demoService.isDemoMode) {
      return true; // Default enabled in demo
    }
    return invoke<boolean>("get_noise_suppression");
  }

  /**
   * Set noise suppression enabled state
   */
  async setNoiseSuppression(enabled: boolean): Promise<void> {
    if (this.demoService.isDemoMode) {
      console.log("[Demo] Noise suppression set to:", enabled);
      return;
    }
    await invoke("set_noise_suppression", { enabled });
  }
```

**Step 2: Commit**

```bash
git add src/app/core/services/tauri.service.ts
git commit -m "feat(frontend): add noise suppression methods to TauriService"
```

---

## Task 8: Add toggle to Settings Popup

**Files:**
- Modify: `src/app/shared/components/settings-popup/settings-popup.component.ts`

**Step 1: Add signal for noise suppression state**

In the component class, after `debugEnabled` computed (around line 377), add:

```typescript
  // Noise suppression state
  private _noiseSuppression = signal(true);
  readonly noiseSuppression = this._noiseSuppression.asReadonly();
```

**Step 2: Load state in loadData**

In the `loadData` method, after loading settings (around line 410), add:

```typescript
      // Load noise suppression state
      const noiseSuppressionEnabled = await this.tauri.getNoiseSuppression();
      this._noiseSuppression.set(noiseSuppressionEnabled);
```

**Step 3: Add toggle method**

After `toggleDebugMode` method (around line 489), add:

```typescript
  async toggleNoiseSuppression(): Promise<void> {
    const newValue = !this.noiseSuppression();
    try {
      await this.tauri.setNoiseSuppression(newValue);
      this._noiseSuppression.set(newValue);
      // Restart mixing to apply the change
      await this.mixer.startOrRestartWithDevices();
    } catch (err) {
      console.error("Failed to toggle noise suppression:", err);
    }
  }
```

**Step 4: Add UI toggle in template**

In the template, after the "Mic Monitoring" toggle (around line 260), add:

```html
            <!-- Noise Suppression -->
            <div
              class="flex items-center justify-between py-3 px-4 bg-background rounded-lg mt-3"
            >
              <div>
                <span class="text-sm text-text-primary">Noise Suppression</span>
                <p class="text-xs text-text-muted mt-0.5">
                  Reduce background noise (fans, keyboard)
                </p>
              </div>
              <button
                class="w-12 h-6 rounded-full transition-colors relative"
                [class]="noiseSuppression() ? 'bg-accent' : 'bg-surface'"
                (click)="toggleNoiseSuppression()"
              >
                <div
                  class="absolute top-1 w-4 h-4 bg-white rounded-full transition-transform"
                  [class]="noiseSuppression() ? 'left-7' : 'left-1'"
                ></div>
              </button>
            </div>
```

**Step 5: Run Angular build**

```bash
npm run build
```

Expected: Build succeeds.

**Step 6: Commit**

```bash
git add src/app/shared/components/settings-popup/settings-popup.component.ts
git commit -m "feat(ui): add noise suppression toggle to settings popup"
```

---

## Task 9: Update AppSettings model in frontend

**Files:**
- Modify: `src/app/core/models/index.ts` (or wherever AppSettings is defined)

**Step 1: Find AppSettings interface**

Search for the AppSettings interface and add the new field to AudioSettings:

```typescript
export interface AudioSettings {
  inputDeviceId: string | null;
  outputDeviceId: string | null;
  previewDeviceId: string | null;
  masterVolume: number;
  sampleRate: number;
  bufferSize: number;
  micMonitoring: boolean;
  globalHotkeysEnabled: boolean;
  micVolume: number;
  soundboardVolume: number;
  noiseSuppressionEnabled: boolean;  // Add this line
}
```

**Step 2: Commit**

```bash
git add src/app/core/models/index.ts
git commit -m "feat(models): add noiseSuppressionEnabled to AudioSettings"
```

---

## Task 10: Final integration test and cleanup

**Files:**
- Run manual tests
- Format and lint

**Step 1: Run all Rust tests**

```bash
cd src-tauri && cargo test
```

Expected: All tests pass.

**Step 2: Run Clippy**

```bash
cd src-tauri && cargo clippy
```

Expected: No warnings.

**Step 3: Format Rust code**

```bash
cd src-tauri && cargo fmt
```

**Step 4: Build full application**

```bash
npm run tauri dev
```

Expected: Application launches with noise suppression toggle in settings.

**Step 5: Manual test**

1. Open Settings popup
2. Verify "Noise Suppression" toggle is visible and ON by default
3. Toggle it OFF, verify setting persists after app restart
4. Toggle it ON, speak into microphone, verify audio passes through
5. Make background noise (keyboard, fan), verify it's reduced

**Step 6: Final commit**

```bash
git add -A
git commit -m "feat: complete noise suppression implementation

- nnnoiseless 0.5.2 for real-time noise reduction
- On/Off toggle in settings (enabled by default)
- Processes mic input in 480-sample frames
- ~10ms added latency (acceptable for streaming)

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Summary

| Task | Description | Estimated Steps |
|------|-------------|-----------------|
| 1 | Add nnnoiseless dependency | 3 |
| 2 | Add noise_suppression_enabled to settings | 7 |
| 3 | Add Tauri commands | 5 |
| 4 | Implement NoiseFilter struct | 4 |
| 5 | Integrate into AudioEngine | 10 |
| 6 | Update start_mixing command | 3 |
| 7 | Add frontend TauriService methods | 2 |
| 8 | Add toggle to Settings Popup | 6 |
| 9 | Update frontend models | 2 |
| 10 | Final testing and cleanup | 6 |

**Total: 10 tasks, ~48 steps**
