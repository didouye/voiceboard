# VU Meters Design

## Overview

Add real-time level visualization (VU meters) to Voiceboard with horizontal bars displayed in a footer panel.

## Backend Architecture

### Level Calculation in AudioEngine

The `AudioEngine` calculates RMS (Root Mean Square) levels in existing audio callbacks:

- **Input level**: Calculated in input callback after mic volume is applied
- **Output level**: Calculated in output callback after final mix

Levels are emitted via the existing `LevelUpdate` event at ~30Hz (~33ms intervals) for smooth animation without CPU overhead.

### dB Conversion

- Raw RMS values (0.0 - 1.0) converted to dB (-60dB to 0dB)
- Values below -60dB treated as silence

### Peak Detection

- Separate peak holder tracks maximum value
- Decay rate: ~20dB/second for gradual return

### Enhanced Event Structure

```rust
LevelUpdate {
    input_rms: f32,      // 0.0 - 1.0
    input_peak: f32,     // 0.0 - 1.0
    output_rms: f32,     // 0.0 - 1.0
    output_peak: f32,    // 0.0 - 1.0
}
```

Events sent to frontend via Tauri events (`emit_to`).

## Frontend UI

### Layout

Horizontal footer panel at the bottom of the application:

```
┌────────────────────────────────────────────────────┐
│                    SOUNDBOARD                       │
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐        │
│  │ 1  │ │ 2  │ │ 3  │ │ 4  │ │ 5  │ │ 6  │        │
│  └────┘ └────┘ └────┘ └────┘ └────┘ └────┘        │
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐        │
│  │ 7  │ │ 8  │ │ 9  │ │10  │ │11  │ │12  │        │
│  └────┘ └────┘ └────┘ └────┘ └────┘ └────┘        │
├────────────────────────────────────────────────────┤
│ INPUT ██████████│░░░░░░░   OUTPUT ████│░░░░░░░░░░ │
└────────────────────────────────────────────────────┘
```

### Structure

- Two horizontal bars side by side (50% width each with gap)
- INPUT on left, OUTPUT on right
- Label integrated at left of each bar
- Peak indicator: thin vertical line on each bar
- Footer height: ~30px

### Visual Style

- Minimalist design matching dark theme
- Bars: ~10px height
- Accent color (#7b2cbf) with glow effect
- Glow intensifies with level
- Semi-transparent footer background

### Animation

- CSS transition on width with fast ease-out (~50ms)
- Peak decay managed in JS with `requestAnimationFrame`
