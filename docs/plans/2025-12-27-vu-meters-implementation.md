# VU Meters Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add real-time horizontal level meters (input/output) in a footer panel with peak hold and decay.

**Architecture:** Calculate RMS levels in AudioEngine callbacks, emit via Tauri events at 30Hz, display with Angular component using CSS animations.

**Tech Stack:** Rust (AudioEngine), Tauri Events, Angular Signals, CSS transitions

---

## Task 1: Add Level Calculation to AudioEngine

**Files:**
- Modify: `src-tauri/src/application/audio_engine.rs`

**Step 1: Add atomic level storage and peak tracking**

Add after line 16 (after `RING_BUFFER_SIZE`):

```rust
/// Level update interval in milliseconds (~30Hz)
const LEVEL_UPDATE_INTERVAL_MS: u64 = 33;
```

Add after line 58 (inside `AudioEngineEvent` enum, replace existing `LevelUpdate`):

```rust
    /// Audio level update (for UI meters)
    LevelUpdate {
        input_rms: f32,
        input_peak: f32,
        output_rms: f32,
        output_peak: f32,
    },
```

**Step 2: Add level tracking in input callback**

In the input callback (around line 259), replace the existing callback body with:

```rust
move |data: &[f32], _: &cpal::InputCallbackInfo| {
    let muted = mic_muted_clone.load(Ordering::Relaxed);
    let volume = f32::from_bits(mic_volume_clone.load(Ordering::Relaxed));

    // Calculate RMS for input level
    let mut sum_squares = 0.0f32;

    if let Ok(mut prod) = producer_clone.try_lock() {
        for &sample in data {
            let processed = if muted { 0.0 } else { sample * volume };
            sum_squares += processed * processed;
            let _ = prod.try_push(processed);
        }
    }

    // Store RMS level (will be read by level monitoring thread)
    if !data.is_empty() {
        let rms = (sum_squares / data.len() as f32).sqrt();
        input_level_clone.store(rms.to_bits(), Ordering::Relaxed);
    }
},
```

**Step 3: Add level tracking in output callback**

In the output callback (around line 294), add level calculation after master volume:

```rust
// Calculate output RMS after master volume
let mut sum_squares = 0.0f32;
for sample in data.iter() {
    sum_squares += sample * sample;
}
if !data.is_empty() {
    let rms = (sum_squares / data.len() as f32).sqrt();
    output_level_clone.store(rms.to_bits(), Ordering::Relaxed);
}
```

**Step 4: Add atomic level variables before stream creation**

Add before input stream creation (around line 252):

```rust
// Atomic level values for lock-free reading
let input_level = Arc::new(AtomicU32::new(0));
let output_level = Arc::new(AtomicU32::new(0));
let input_level_clone = input_level.clone();
let output_level_clone = output_level.clone();
```

**Step 5: Add level emission loop**

Add after streams are started (after line 374, inside the `Started` block):

```rust
// Start level monitoring thread
let input_level_monitor = input_level.clone();
let output_level_monitor = output_level.clone();
let event_tx_monitor = event_tx.clone();
let is_running_monitor = is_running.clone();

std::thread::spawn(move || {
    let mut input_peak = 0.0f32;
    let mut output_peak = 0.0f32;
    let decay_rate = 0.05; // ~20dB/sec at 30Hz

    while is_running_monitor.load(Ordering::Relaxed) {
        let input_rms = f32::from_bits(input_level_monitor.load(Ordering::Relaxed));
        let output_rms = f32::from_bits(output_level_monitor.load(Ordering::Relaxed));

        // Update peaks
        if input_rms > input_peak {
            input_peak = input_rms;
        } else {
            input_peak = (input_peak - decay_rate).max(0.0);
        }

        if output_rms > output_peak {
            output_peak = output_rms;
        } else {
            output_peak = (output_peak - decay_rate).max(0.0);
        }

        let _ = event_tx_monitor.send(AudioEngineEvent::LevelUpdate {
            input_rms,
            input_peak,
            output_rms,
            output_peak,
        });

        std::thread::sleep(std::time::Duration::from_millis(LEVEL_UPDATE_INTERVAL_MS));
    }
});
```

**Step 6: Verify it compiles**

Run: `cd /Users/didouye/Workspace/voiceboard && cargo check -p voiceboard`

Expected: Compilation succeeds

**Step 7: Commit**

```bash
git add src-tauri/src/application/audio_engine.rs
git commit -m "feat(audio): add RMS level calculation with peak tracking"
```

---

## Task 2: Emit Level Events via Tauri

**Files:**
- Modify: `src-tauri/src/application/state.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Check current state structure**

Read `src-tauri/src/application/state.rs` to understand AppState.

**Step 2: Add Tauri event emission for levels**

In `src-tauri/src/lib.rs`, add a background task that polls AudioEngine events and emits to frontend.

Add after the app setup (in the `setup` hook):

```rust
// Start level event forwarding
let app_handle = app.handle().clone();
let engine_for_levels = app_state.audio_engine.clone();

std::thread::spawn(move || {
    loop {
        if let Ok(engine) = engine_for_levels.try_lock() {
            while let Some(event) = engine.try_recv_event() {
                match event {
                    AudioEngineEvent::LevelUpdate { input_rms, input_peak, output_rms, output_peak } => {
                        let _ = app_handle.emit("audio-levels", serde_json::json!({
                            "inputRms": input_rms,
                            "inputPeak": input_peak,
                            "outputRms": output_rms,
                            "outputPeak": output_peak,
                        }));
                    }
                    _ => {}
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
});
```

**Step 3: Add required import**

Add to imports in `lib.rs`:

```rust
use crate::application::audio_engine::AudioEngineEvent;
```

**Step 4: Verify it compiles**

Run: `cd /Users/didouye/Workspace/voiceboard && cargo check -p voiceboard`

Expected: Compilation succeeds

**Step 5: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(tauri): emit audio level events to frontend"
```

---

## Task 3: Create Level Meter Angular Component

**Files:**
- Create: `src/app/features/mixer/level-meters/level-meters.component.ts`

**Step 1: Create the component file**

```typescript
import { Component, OnInit, OnDestroy, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

interface AudioLevels {
  inputRms: number;
  inputPeak: number;
  outputRms: number;
  outputPeak: number;
}

@Component({
  selector: 'app-level-meters',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="level-meters">
      <div class="meter-container">
        <span class="meter-label">INPUT</span>
        <div class="meter-track">
          <div class="meter-fill" [style.width.%]="inputLevel() * 100"></div>
          <div class="meter-peak" [style.left.%]="inputPeak() * 100"></div>
        </div>
      </div>
      <div class="meter-container">
        <span class="meter-label">OUTPUT</span>
        <div class="meter-track">
          <div class="meter-fill" [style.width.%]="outputLevel() * 100"></div>
          <div class="meter-peak" [style.left.%]="outputPeak() * 100"></div>
        </div>
      </div>
    </div>
  `,
  styles: [`
    .level-meters {
      display: flex;
      gap: 20px;
      padding: 12px 20px;
      background: rgba(0, 0, 0, 0.3);
      border-top: 1px solid rgba(255, 255, 255, 0.1);
    }

    .meter-container {
      flex: 1;
      display: flex;
      align-items: center;
      gap: 12px;
    }

    .meter-label {
      font-size: 0.7rem;
      font-weight: 600;
      color: rgba(255, 255, 255, 0.6);
      text-transform: uppercase;
      letter-spacing: 0.5px;
      min-width: 50px;
    }

    .meter-track {
      flex: 1;
      height: 10px;
      background: rgba(255, 255, 255, 0.1);
      border-radius: 5px;
      position: relative;
      overflow: hidden;
    }

    .meter-fill {
      height: 100%;
      background: linear-gradient(90deg, #7b2cbf, #00d4ff);
      border-radius: 5px;
      transition: width 50ms ease-out;
      box-shadow: 0 0 10px rgba(123, 44, 191, 0.5);
    }

    .meter-peak {
      position: absolute;
      top: 0;
      width: 2px;
      height: 100%;
      background: #fff;
      border-radius: 1px;
      transform: translateX(-50%);
      transition: left 50ms ease-out;
      box-shadow: 0 0 4px rgba(255, 255, 255, 0.8);
    }
  `]
})
export class LevelMetersComponent implements OnInit, OnDestroy {
  inputLevel = signal(0);
  inputPeak = signal(0);
  outputLevel = signal(0);
  outputPeak = signal(0);

  private unlisten?: UnlistenFn;

  async ngOnInit() {
    this.unlisten = await listen<AudioLevels>('audio-levels', (event) => {
      this.inputLevel.set(Math.min(event.payload.inputRms * 3, 1)); // Scale for visibility
      this.inputPeak.set(Math.min(event.payload.inputPeak * 3, 1));
      this.outputLevel.set(Math.min(event.payload.outputRms * 3, 1));
      this.outputPeak.set(Math.min(event.payload.outputPeak * 3, 1));
    });
  }

  ngOnDestroy() {
    this.unlisten?.();
  }
}
```

**Step 2: Verify Angular compiles**

Run: `cd /Users/didouye/Workspace/voiceboard && npm run build`

Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/app/features/mixer/level-meters/level-meters.component.ts
git commit -m "feat(ui): add level meters component with peak indicators"
```

---

## Task 4: Integrate Level Meters into Mixer Layout

**Files:**
- Modify: `src/app/features/mixer/mixer.component.ts`

**Step 1: Import the component**

Add to imports array (line 6):

```typescript
import { LevelMetersComponent } from './level-meters/level-meters.component';
```

Update imports in decorator (line 11):

```typescript
imports: [CommonModule, MasterControlComponent, DeviceSelectorComponent, SoundboardComponent, LevelMetersComponent],
```

**Step 2: Add to template**

Replace the template with footer added (wrap content in a flex container):

```typescript
template: `
  <div class="mixer-container">
    <header class="mixer-header">
      <h1>Voiceboard</h1>
      <p class="subtitle">Virtual Microphone Mixer</p>
    </header>

    @if (mixer.loading()) {
      <div class="loading">Loading...</div>
    } @else if (mixer.error()) {
      <div class="error-banner">
        {{ mixer.error() }}
        <button (click)="mixer.clearError()">Dismiss</button>
      </div>
    }

    <div class="mixer-content">
      <div class="mixer-layout">
        <!-- Left Sidebar - Device Selection -->
        <aside class="sidebar">
          <app-device-selector />
        </aside>

        <!-- Main Content -->
        <main class="main-content">
          <!-- Soundboard Section -->
          <app-soundboard />
        </main>

        <!-- Right Sidebar - Master Control -->
        <aside class="master-section">
          <app-master-control
            [volume]="mixer.masterVolume()"
            [isRunning]="mixer.isRunning()"
            (volumeChange)="onMasterVolumeChange($event)"
            (startStop)="onStartStop()"
          />
        </aside>
      </div>
    </div>

    <footer class="mixer-footer">
      <app-level-meters />
    </footer>
  </div>
`,
```

**Step 3: Update styles**

Add/update styles:

```typescript
styles: [`
  .mixer-container {
    min-height: 100vh;
    background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
    color: #fff;
    display: flex;
    flex-direction: column;
  }

  .mixer-header {
    text-align: center;
    padding: 20px;
  }

  .mixer-header h1 {
    font-size: 2.5rem;
    margin: 0;
    background: linear-gradient(90deg, #00d4ff, #7b2cbf);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .subtitle {
    color: #888;
    margin: 5px 0 0;
  }

  .error-banner {
    background: #e74c3c;
    padding: 15px 20px;
    border-radius: 8px;
    margin: 0 20px 20px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    max-width: 1400px;
    margin-left: auto;
    margin-right: auto;
  }

  .error-banner button {
    background: rgba(255,255,255,0.2);
    border: none;
    color: #fff;
    padding: 5px 15px;
    border-radius: 4px;
    cursor: pointer;
  }

  .loading {
    text-align: center;
    padding: 40px;
    color: #888;
  }

  .mixer-content {
    flex: 1;
    padding: 0 20px 20px;
    overflow-y: auto;
  }

  .mixer-layout {
    display: grid;
    grid-template-columns: 300px 1fr 200px;
    gap: 25px;
    max-width: 1600px;
    margin: 0 auto;
  }

  .sidebar {
    position: sticky;
    top: 20px;
    align-self: start;
  }

  .main-content {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 25px;
  }

  .master-section {
    position: sticky;
    top: 20px;
    align-self: start;
  }

  .mixer-footer {
    position: sticky;
    bottom: 0;
    z-index: 100;
  }

  /* Responsive */
  @media (max-width: 1200px) {
    .mixer-layout {
      grid-template-columns: 250px 1fr 180px;
    }
  }

  @media (max-width: 900px) {
    .mixer-layout {
      grid-template-columns: 1fr;
      gap: 20px;
    }

    .sidebar, .master-section {
      position: static;
    }

    .master-section {
      order: -1;
    }
  }
`]
```

**Step 4: Verify Angular compiles**

Run: `cd /Users/didouye/Workspace/voiceboard && npm run build`

Expected: Build succeeds

**Step 5: Commit**

```bash
git add src/app/features/mixer/mixer.component.ts
git commit -m "feat(ui): integrate level meters in footer"
```

---

## Task 5: Test Manually and Polish

**Step 1: Run the application**

Run: `cd /Users/didouye/Workspace/voiceboard && npm run tauri dev`

**Step 2: Verify functionality**

- [ ] Start mixing with selected devices
- [ ] Speak into microphone - input meter should respond
- [ ] Play a sound - output meter should respond
- [ ] Peak indicators should show and decay slowly
- [ ] Meters should stop when mixing stops

**Step 3: Adjust scaling if needed**

If levels are too low/high, adjust the `* 3` multiplier in `level-meters.component.ts`.

**Step 4: Final commit**

```bash
git add -A
git commit -m "feat: add VU level meters with peak hold

- Calculate RMS levels in AudioEngine at 30Hz
- Emit levels via Tauri events
- Display horizontal bars in footer (INPUT left, OUTPUT right)
- Peak indicators with slow decay
- Minimalist design with glow effect"
```

---

## Task 6: Update ROADMAP

**Files:**
- Modify: `ROADMAP.md`

**Step 1: Mark task as done**

Change line 20 from:
```markdown
- [ ] Level visualization (VU meters) - AudioEngine emits `LevelUpdate`, connect to UI
```

To:
```markdown
- [x] Level visualization (VU meters) - AudioEngine emits `LevelUpdate`, connect to UI
```

**Step 2: Commit**

```bash
git add ROADMAP.md
git commit -m "docs: mark VU meters as completed in roadmap"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Add RMS level calculation | `audio_engine.rs` |
| 2 | Emit Tauri events | `lib.rs` |
| 3 | Create Angular component | `level-meters.component.ts` |
| 4 | Integrate in layout | `mixer.component.ts` |
| 5 | Manual testing | - |
| 6 | Update roadmap | `ROADMAP.md` |
