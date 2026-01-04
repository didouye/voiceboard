# Device Auto-Selection and Auto-Start Design

> **Date:** 2026-01-02
> **Status:** Approved

## Overview

Simplify device management by auto-selecting the virtual output device and auto-starting the mixer when a functional configuration is detected.

## Goals

1. Remove manual virtual output selection (auto-detect VB-Cable, BlackHole, etc.)
2. Auto-start mixer when input + virtual output are available
3. Silent restart when user changes devices
4. Fix bug: physical devices appearing in virtual device list

## Device Classification

```
Audio Devices
├── Input
│   ├── InputPhysical  → Microphones (user selectable)
│   └── InputVirtual   → Virtual inputs (not used)
└── Output
    ├── OutputPhysical → Speakers, headphones (for Preview/Monitoring)
    └── OutputVirtual  → VB-Cable, Voicemeeter, BlackHole (auto-selected)
```

## Virtual Output Priority Order

When multiple virtual outputs are detected, select by priority:

1. **VB-Cable** - `cable output (vb-audio`, `cable input (vb-audio`
2. **Voicemeeter** - `voicemeeter`
3. **BlackHole** - `blackhole`
4. **Other** - `virtual audio`, `loopback`

## Startup Sequence

```
App Launch
    │
    ▼
1. Load saved settings
    │
    ▼
2. Enumerate available devices
    │
    ▼
3. Virtual output available? ──No──► Setup Wizard (VB-Cable)
    │
    │ Yes
    ▼
4. Select virtual output
   - If 1 device: auto-select
   - If multiple: use priority (or saved preference)
    │
    ▼
5. Input available? ──No──► "No input device", mixer disabled
    │
    │ Yes
    ▼
6. Select input
   - Use saved preference if exists
   - Otherwise system default
    │
    ▼
7. AUTO-START MIXER
```

## UI Changes

### Device Selector (Simplified)

```
┌─────────────────────────────────────┐
│         Audio Devices               │
├─────────────────────────────────────┤
│                                     │
│ 🎤 Input Device (Microphone)        │
│ [Select dropdown - InputPhysical]   │
│                                     │
│ 🎧 Preview Output (Monitoring)      │
│ [Select dropdown - OutputPhysical]  │
│                                     │
│ ─────────────────────────────────── │
│                                     │
│ 🔊 Virtual Output  [only if ≥2]     │
│ [Select dropdown - OutputVirtual]   │
│                                     │
│ ─────────────────────────────────── │
│                                     │
│ ● Ready to mix / No input device    │
│                                     │
│ 🔄 Refresh Devices                  │
└─────────────────────────────────────┘
```

### Display Rules

- **Virtual Output selector**: visible only if ≥2 virtual devices detected
- **Status "Ready to mix"**: green when input + virtual output OK
- **Status "No input device"**: red when no microphone available

## Silent Restart on Device Change

When user changes Input or Preview while mixer is running:

```
User changes device
    │
    ▼
Mixer running? ──No──► Save setting only
    │
    │ Yes
    ▼
Stop mixer
    │
    ▼
Save new setting
    │
    ▼
Start mixer (new config)
```

- Total time: ~100ms (imperceptible)
- No UI feedback (silent)
- VU meter may have micro-pause (acceptable)
- If restart fails, show error

## Backend Changes (Rust)

### New Functions

```rust
// Get virtual outputs sorted by priority
fn get_virtual_outputs_by_priority() -> Vec<AudioDevice>

// Get physical outputs only (for preview)
fn get_physical_outputs() -> Vec<AudioDevice>

// Get physical inputs only (microphones)
fn get_physical_inputs() -> Vec<AudioDevice>
```

### Priority Detection

```rust
const VIRTUAL_OUTPUT_PRIORITY: &[&str] = &[
    "cable output (vb-audio",
    "cable input (vb-audio",
    "vb-audio virtual cable",
    "voicemeeter",
    "blackhole",
    "virtual audio",
    "loopback",
];
```

## Frontend Changes (Angular)

### DeviceSelectorComponent

- Filter Input dropdown: `InputPhysical` only
- Filter Preview dropdown: `OutputPhysical` only
- Virtual Output dropdown: `OutputVirtual` only, hidden if single device
- Call `restartIfRunning()` on device change

### MixerService

```typescript
// Auto-start in initialize() if functional config
async initialize() {
  // ... load devices
  if (hasInput && hasVirtualOutput) {
    await this.start();
  }
}

// Silent restart
async restartIfRunning() {
  if (this.isRunning()) {
    await this.stop();
    await this.start();
  }
}
```

### AppComponent

- Auto-select virtual output on startup (by priority or saved)
- Auto-select input (saved or system default) on first launch

## Settings Schema

```typescript
interface AudioSettings {
  inputDeviceId: string | null;      // Saved microphone choice
  outputDeviceId: string | null;     // Saved virtual output choice (renamed for clarity)
  previewDeviceId: string | null;    // Saved preview/monitoring choice
}
```

## Migration

No migration needed. Existing `outputDeviceId` will be used for virtual output. If the saved device is not virtual, it will be ignored and auto-selection will apply.
