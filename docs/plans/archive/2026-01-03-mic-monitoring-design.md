# Mic Monitoring Feature Design

> **Date:** 2026-01-03
> **Status:** Approved

## Overview

Add the ability to hear microphone audio on the monitoring/preview output device. This enables users to monitor what they sound like in real-time.

## Requirements

| Source | Virtual Output (VB-Cable) | Monitoring Output |
|--------|---------------------------|-------------------|
| **Microphone** | Always | Only if "Mic Monitoring" toggle is ON |
| **Pad click** | Always | Always |
| **Pad preview** | Never | Always |

## Architecture

### Audio Flow

```
┌─────────────┐
│   Micro     │──────┬──────────────────────────▶ Output Virtuel (VB-Cable)
└─────────────┘      │
                     └─── [if mic_monitoring ON] ─▶ Output Monitoring

┌─────────────┐
│  Pad Sound  │──────┬──────────────────────────▶ Output Virtuel (VB-Cable)
└─────────────┘      └──────────────────────────▶ Output Monitoring

┌─────────────┐
│Pad Preview  │─────────────────────────────────▶ Output Monitoring (via PreviewEngine)
└─────────────┘
```

### AudioEngine Changes

- Add 3rd output stream to preview/monitoring device
- Add 2nd ring buffer for monitoring (mic samples when enabled)
- Pad sounds are mixed into BOTH outputs
- New command: `SetMicMonitoring(bool)`
- New command: `SetMonitoringDevice(String)`

### PreviewEngine

No changes - continues to handle pad preview button only.

### New Atomic States

- `mic_monitoring: AtomicBool` - toggle for mic monitoring
- `monitoring_device: Arc<Mutex<String>>` - monitoring device name

## Commands

### New AudioEngineCommand variants

```rust
pub enum AudioEngineCommand {
    // ... existing ...

    /// Enable/disable mic monitoring on preview output
    SetMicMonitoring(bool),

    /// Set the monitoring output device
    SetMonitoringDevice(String),
}
```

### Output Callback (Virtual) - Unchanged

1. Read mic samples from ring buffer
2. Mix pad sounds
3. Apply master volume

### Output Callback (Monitoring) - New

1. If `mic_monitoring` ON → Read mic samples from 2nd ring buffer
2. Mix pad sounds (same logic as virtual)
3. Apply master volume (same as virtual output)

## Settings

### Backend (domain/settings.rs)

```rust
pub struct AudioSettings {
    // ... existing ...
    pub preview_device_id: Option<String>,  // already present
    pub mic_monitoring: bool,               // NEW - default: false
}
```

### Frontend (models/audio-device.model.ts)

```typescript
interface AudioSettings {
  // ... existing ...
  previewDeviceId: string | null;
  micMonitoring: boolean;  // NEW
}
```

### Persistence

- `mic_monitoring` toggle state saved in settings
- Restored on application startup
- Monitoring uses the same `previewDeviceId` as pad preview

## User Interface

### Toggle Location

In `device-selector` component, under "Preview Output (Monitoring)" section:

```
┌─────────────────────────────────────────────────┐
│ Preview Output (Monitoring)                     │
│ ┌─────────────────────────────────────────────┐ │
│ │ [Dropdown: Speakers (Realtek Audio)    ▼]  │ │
│ └─────────────────────────────────────────────┘ │
│                                                 │
│ ┌─────────────────────────────────────────────┐ │
│ │ 🎤 Mic Monitoring          [====○    ] ON  │ │
│ └─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

### Toggle Behavior

- OFF (default): Mic is not heard in monitoring
- ON: Mic is routed to monitoring output in real-time

## Files to Modify

### Backend (Rust)

| File | Changes |
|------|---------|
| `audio_engine.rs` | 3rd output stream, 2nd ring buffer, new commands |
| `commands.rs` | New Tauri command `set_mic_monitoring` |
| `settings.rs` | Field `mic_monitoring: bool` |
| `state.rs` | Pass monitoring state to engine |

### Frontend (Angular)

| File | Changes |
|------|---------|
| `device-selector.component.ts` | Toggle UI for mic monitoring |
| `tauri.service.ts` | Method `setMicMonitoring()` |
| `soundboard.service.ts` | Signal and persistence |
| `audio-device.model.ts` | Field `micMonitoring` |

## Implementation Order

1. Backend: Settings + commands
2. Backend: AudioEngine (3rd stream + routing)
3. Frontend: Service + model
4. Frontend: UI toggle
