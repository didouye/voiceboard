# Noise Suppression Design

> **Date:** 2026-01-23
> **Status:** Approved

## Overview

Add real-time noise suppression on microphone input, targeting streamers/gamers who want to eliminate fan noise, mechanical keyboards, and ambient sounds.

### Technical Choices

| Aspect | Decision |
|--------|----------|
| Library | `nnnoiseless` 0.5.2 (pure Rust port of RNNoise) |
| UX | Simple On/Off toggle in settings, enabled by default |
| Scope | Microphone input only (not soundboard sounds) |
| VAD | Not used in V1 |

### Why nnnoiseless?

| Criteria | nnnoiseless |
|----------|-------------|
| Language | Pure Rust (no FFI) |
| Format | Mono 48kHz, 480-sample frames |
| Latency | ~10ms (480 samples @ 48kHz) |
| CPU | Very light (~60x real-time) |
| License | BSD-3 |
| Maintenance | Active, stable crate |

## Architecture

### Audio Pipeline Modification

Processing happens in the **input callback** (`audio_engine.rs`), after mono conversion and before pushing to the ring buffer:

```
Mic Input → Mono conversion → [nnnoiseless] → Ring Buffer → Mix → Output
                              ^^^^^^^^^^^^
                              New block
```

### Implementation

```rust
// Pseudo-code of the new flow
for frame in 0..num_frames {
    // 1. Multi-channel → mono conversion (existing)
    let mono_sample = average_channels(data, frame, input_ch);

    // 2. Accumulate in a 480-sample buffer
    frame_buffer.push(mono_sample);

    // 3. When we have 480 samples → nnnoiseless processing
    if frame_buffer.len() == 480 {
        denoiser.process_frame(&mut frame_buffer);

        // 4. Push filtered samples to ring buffer
        for sample in frame_buffer.drain(..) {
            let processed = if muted { 0.0 } else { sample * volume };
            ring_buffer.push(processed);
        }
    }
}
```

### Denoiser State

`nnnoiseless::DenoiseState` must persist between callbacks. It will be stored in a shared structure:

```rust
struct NoiseFilterState {
    denoiser: DenoiseState,
    buffer: Vec<f32>,      // Accumulation up to 480 samples
    enabled: AtomicBool,   // On/off toggle
}
```

### Added Latency

- **nnnoiseless** processes in 480-sample frames
- At 48kHz: 480 / 48000 = **10ms added latency**
- Acceptable for streaming (Discord/OBS tolerate ~50-100ms)

## Settings & UI

### Persistence

Add a field to `AppSettings` (already persisted in Tauri store):

```rust
// src-tauri/src/domain/settings.rs
pub struct AppSettings {
    // ... existing fields ...
    pub noise_suppression_enabled: bool,  // default: true
}
```

### Tauri Commands

Two new commands for the frontend:

```rust
#[tauri::command]
fn set_noise_suppression(enabled: bool) -> Result<(), String>

#[tauri::command]
fn get_noise_suppression() -> Result<bool, String>
```

### UI - Settings Popup

Add a toggle in the "Audio" section of the settings popup:

```
┌─────────────────────────────────────┐
│  Audio Settings                     │
├─────────────────────────────────────┤
│  Input Device     [Microphone ▼]    │
│  Output Device    [VB-Cable ▼]      │
│  Preview Device   [Speakers ▼]      │
│                                     │
│  ─────────────────────────────────  │
│                                     │
│  🎙️ Noise Suppression    [••━━] ON  │
│     Reduces background noise        │
│                                     │
└─────────────────────────────────────┘
```

### Behavior

- **Enabled by default** on first use
- **Real-time change**: toggle activates/deactivates instantly
- **Persisted**: state is saved and restored on restart

## Cargo Dependency

```toml
# src-tauri/Cargo.toml
[dependencies]
nnnoiseless = "0.5.2"
```

## Error Handling

The `nnnoiseless` denoiser cannot fail once initialized (no I/O, no dynamic allocation during processing). The only error cases:

| Case | Handling |
|------|----------|
| Initialization fails | Log error, disable filter, continue without |
| Sample rate ≠ 48kHz | Automatically disable filter (nnnoiseless only supports 48kHz) |

## Unit Tests

```rust
#[cfg(test)]
mod tests {
    // Test 1: Denoiser reduces white noise amplitude
    fn test_noise_reduction_reduces_noise()

    // Test 2: Denoiser preserves vocal signal (sine wave)
    fn test_noise_reduction_preserves_voice()

    // Test 3: On/off toggle works
    fn test_noise_suppression_toggle()

    // Test 4: Buffer accumulates correctly up to 480 samples
    fn test_frame_buffer_accumulation()
}
```

## Performance Impact

- **CPU**: negligible (~1% on modern CPU)
- **Memory**: ~100KB for RNN model
- **Latency**: +10ms (acceptable)

## Future Improvements (Not in V1)

- VAD indicator in UI
- Intensity slider (mix between original and filtered signal)
- Auto-mute when no voice detected
- DeepFilterNet integration for higher quality
