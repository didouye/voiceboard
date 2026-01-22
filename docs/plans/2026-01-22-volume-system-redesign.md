# Volume System Redesign

## Overview

Fix volume slider visibility, add volume persistence, introduce Soundboard Volume control, and normalize sounds at import.

## Problems Addressed

1. **Volume sliders not visible** - Only the thumb is visible, not the track
2. **Volumes not persisted** - Master volume and mic volume reset to 100% on restart
3. **Soundboard sounds too loud** - No global control for soundboard volume, sounds can saturate
4. **Sound files not managed** - Original files can be deleted, breaking the soundboard

## Design

### Volume Hierarchy

```
Sound volume (if != 100%) ──┐
        or                  ├─→ × Master Volume ─→ Output
Soundboard Volume (else) ───┘
        +
   Mic Volume ──────────────┘
```

**Effective volume calculation for a sound:**
```
effective_volume = (sound.volume != 1.0) ? sound.volume : settings.soundboard_volume
```

### New Settings Fields

Add to `AudioSettings` (backend and frontend):

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `mic_volume` | f32 | 1.0 | Microphone volume (0.0-2.0) |
| `soundboard_volume` | f32 | 1.0 | Global soundboard volume (0.0-2.0) |

Use `#[serde(default)]` for backward compatibility.

### Volume Persistence

| Command | Change |
|---------|--------|
| `set_master_volume` | Add missing `store.save()` |
| `set_mic_volume` | New command with persistence |
| `set_soundboard_volume` | New command with persistence |

### Sound Import and Normalization

**Storage location:**
- macOS/Linux: `~/.voiceboard/sounds/`
- Windows: `%APPDATA%/voiceboard/sounds/`

**Import process:**
1. User selects audio file
2. Calculate SHA-256 hash of original file
3. Decode audio and find peak amplitude
4. Calculate gain for -3dB peak: `gain = 0.708 / peak`
5. Apply gain to all samples
6. Encode as WAV and save to `sounds/{hash}.wav`
7. Return path to normalized file

**Why WAV:**
- Lossless (no re-compression artifacts)
- Fast decoding (no MP3/OGG decompression on each play)
- Acceptable size for short soundboard clips

**New command:** `import_and_normalize_sound(path: String) -> ImportedSoundDto`

### UI Changes

**Slider visibility fix:**
Use dynamic gradient background instead of pseudo-elements:
```html
[style.background]="'linear-gradient(to right, #9d4edd ' + (value * 100) + '%, #12121a ' + (value * 100) + '%)'"
```

**Settings popup layout (Mixer section):**
1. Mic Volume (with mute button)
2. Soundboard Volume (new)
3. Master Volume

## Files to Modify

### Backend (Rust)

| File | Changes |
|------|---------|
| `domain/settings.rs` | Add `mic_volume`, `soundboard_volume` fields |
| `application/commands.rs` | Add `set_mic_volume`, `set_soundboard_volume`, fix `set_master_volume`, add `import_and_normalize_sound` |

### Frontend (Angular)

| File | Changes |
|------|---------|
| `models/audio-device.model.ts` | Add `micVolume`, `soundboardVolume` to `AudioSettings` |
| `services/tauri.service.ts` | Add methods for new commands |
| `services/soundboard.service.ts` | Calculate effective volume in `playSound()` |
| `services/mixer.service.ts` | Expose `soundboardVolume` signal |
| `settings-popup.component.ts` | Add Soundboard Volume slider, fix slider styles, persist mic volume |

## Backward Compatibility

- Existing settings without new fields will use defaults (1.0)
- Existing sounds with original file paths will continue to work
- No forced migration required
