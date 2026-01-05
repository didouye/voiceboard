# Sound Renaming Design

## Overview

Allow users to set a custom name for each sound pad, displayed prominently with the original filename shown below in smaller text.

## Data Model

Add `customName` field to `SoundPad` interface:

```typescript
export interface SoundPad {
  id: string;
  sound: SoundFile | null;
  color: string;
  hotkey?: string;
  isPlaying: boolean;
  volume: number;
  speed: number;
  customName?: string;  // NEW: user-defined name (optional)
}
```

The `SoundFile.name` remains unchanged (contains original filename).

## Display Logic

**Pad display:**
- If `customName` is set: show `customName` (bold, white) + `sound.name` (smaller, grey) + duration
- If `customName` is empty/undefined: show only `sound.name` (current behavior) + duration

**Visual layout:**
```
┌─────────────────────┐
│  Custom Name       │  ← customName (bold, white, truncated)
│  filename.mp3      │  ← sound.name (10px, text-muted)
│      0:03          │  ← duration
└─────────────────────┘
```

## UI Changes

### Pad Settings Modal

Add "Name" input field at the top of the modal (before Volume section):

```
┌─────────────────────────────────────┐
│  filename.mp3                    ✕  │  ← header shows original filename
├─────────────────────────────────────┤
│  Name                               │
│  ┌─────────────────────────────┐    │
│  │ Custom name here...         │    │  ← text input
│  └─────────────────────────────┘    │
│  Original: filename.mp3             │  ← hint text
├─────────────────────────────────────┤
│  Volume                        100% │
│  ═══════════════════○═══════════    │
│  ...                                │
└─────────────────────────────────────┘
```

## Persistence

Update `SavedPad` interface in `soundboard.service.ts`:

```typescript
interface SavedPad {
  id: string;
  sound: SoundFile | null;
  color: string;
  hotkey?: string;
  volume?: number;
  speed?: number;
  customName?: string;  // NEW
}
```

Backwards compatible: existing saved data without `customName` will work (undefined = use filename).

## Service Methods

Add to `SoundboardService`:

```typescript
setPadCustomName(padId: string, name: string | null): void {
  this._pads.update(pads => pads.map(p =>
    p.id === padId ? { ...p, customName: name || undefined } : p
  ));
  this.saveState();
}
```

## Component Changes

### sound-pad.component.ts

1. Add `@Output() customNameChange = new EventEmitter<string | null>()`
2. Add name input field in modal template
3. Update display to show both names when customName is set

### soundboard.component.ts

1. Handle `customNameChange` event
2. Call `soundboardService.setPadCustomName()`

## Files to Modify

1. `src/app/core/models/audio-device.model.ts` - Add `customName` to SoundPad
2. `src/app/core/services/soundboard.service.ts` - Add SavedPad field + setPadCustomName method
3. `src/app/features/soundboard/sound-pad/sound-pad.component.ts` - UI changes
4. `src/app/features/soundboard/soundboard.component.ts` - Handle event
