# Voice Gate (VAD Auto-Mute) Design

> **Date:** 2026-01-23
> **Status:** Approved

## Overview

Auto-mute the microphone when no voice is detected, using the VAD (Voice Activity Detection) probability from nnnoiseless. This eliminates background noise between phrases.

### Design Decisions

| Aspect | Choice |
|--------|--------|
| Control | Separate "Voice Gate" toggle in Settings |
| Dependency | Grayed out when Noise Suppression is OFF |
| Threshold | Fixed at 0.7 (mute if VAD < 70%) |
| Hold-off | 200ms after voice detection |
| Default | Disabled by default |

### How It Works

```
Audio Input → NoiseFilter.process_frame() → (filtered audio, VAD probability)
                                                      ↓
                                            VAD ≥ 0.7 → voice detected → unmute
                                            VAD < 0.7 → no voice → start 200ms timer
                                                      ↓
                                            Timer expired → mute
```

The 200ms hold-off prevents rapid on/off switching between words. As long as voice is detected regularly, the mic stays open.

## Technical Architecture

### NoiseFilter Modifications

The `NoiseFilter` must:
1. Capture the VAD value returned by `process_frame()`
2. Expose this value for the audio engine to use

```rust
pub struct NoiseFilter {
    // ... existing fields ...
    last_vad: f32,  // Last VAD probability (0.0 to 1.0)
}

impl NoiseFilter {
    pub fn process_sample(&mut self, sample: f32) -> &[f32] {
        // ... existing code ...
        if self.buffer.len() >= DENOISE_FRAME_SIZE {
            // process_frame returns VAD probability
            self.last_vad = self.denoiser.process_frame(&mut self.output_buffer, &self.buffer);
            // ... scale output ...
        }
        &self.output_buffer
    }

    /// Get the last VAD probability (0.0 = no voice, 1.0 = voice)
    pub fn last_vad(&self) -> f32 {
        self.last_vad
    }
}
```

### Voice Gate in AudioEngine

New state in the input callback:

```rust
// Constants
const VAD_THRESHOLD: f32 = 0.7;
const VOICE_HOLDOFF_MS: u64 = 200;

// State for voice gate
let voice_gate_enabled = Arc::new(AtomicBool::new(vad_enabled));
let voice_detected_until = Arc::new(AtomicU64::new(0)); // timestamp in ms

// In input callback, after processing frame:
if voice_gate_enabled.load(Ordering::Relaxed) && filter.is_enabled() {
    let vad = filter.last_vad();
    let now_ms = /* current time in ms */;

    if vad >= VAD_THRESHOLD {
        // Voice detected - extend hold-off
        voice_detected_until.store(now_ms + VOICE_HOLDOFF_MS, Ordering::Relaxed);
    }

    let voice_gate_muted = now_ms > voice_detected_until.load(Ordering::Relaxed);
    // Apply mute to samples before pushing to ring buffer
}
```

## Settings & UI

### Persistence

Add field to `AudioSettings`:

```rust
// src-tauri/src/domain/settings.rs
pub struct AudioSettings {
    // ... existing fields ...
    pub noise_suppression_enabled: bool,
    pub voice_gate_enabled: bool,  // NEW - default: false
}
```

### Tauri Commands

```rust
#[tauri::command]
fn get_voice_gate(state: State<AppState>) -> bool

#[tauri::command]
fn set_voice_gate(app: AppHandle, state: State<AppState>, enabled: bool) -> Result<(), String>
```

### UI - Settings Popup

Toggle added right after "Noise Suppression":

```
┌─────────────────────────────────────┐
│  Mixer                              │
│  ...                                │
│                                     │
│  🎙️ Noise Suppression    [••━━] ON  │
│     Reduce background noise         │
│                                     │
│  🔇 Voice Gate           [━━••] OFF │  ← New
│     Auto-mute when not speaking     │
│     (requires Noise Suppression)    │  ← Hint when NS=OFF
│                                     │
└─────────────────────────────────────┘
```

### UI Behavior

- If `noiseSuppression() === false`:
  - Voice Gate toggle grayed out (`opacity-50 pointer-events-none`)
  - Hint text visible: "Requires Noise Suppression"
- If `noiseSuppression() === true`:
  - Voice Gate toggle active
  - Hint hidden

## Implementation Notes

- VAD is only available when Noise Suppression is enabled (same nnnoiseless process)
- The hold-off timer uses atomic operations for thread safety in the audio callback
- Voice gate muting is applied AFTER noise suppression, BEFORE volume adjustment
