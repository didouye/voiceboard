# Sound Model Refactoring Design

> **Date:** 2026-01-10
> **Status:** Approved

## Overview

Refactor the data model to separate Sound (entity) from Pad (display position). This fixes the bug where clicking a pad in a filtered folder plays the wrong sound, and prepares the architecture for cloud synchronization.

## Problem

Current model mixes two concepts:
- **Pad** = position in grid (pad-0, pad-1, etc.)
- **Sound** = content assigned to a position

When filtering by folder, `filteredPads` reassigns IDs (`pad-2` becomes `pad-0`), breaking the reference to the actual sound in `_pads`.

## Key Decisions

| Aspect | Decision |
|--------|----------|
| Identifier | SHA-256 hash of file content (64 chars hex) |
| Hash calculation | At import only |
| Structure | Sounds separated, pads generated virtually |
| Properties | All on Sound (volume, speed, hotkey, image, customName, folderIds) |
| Persistence | Map<hash, Sound> + folders |
| Migration | Automatic on first load |

## Data Model

### Sound (new entity)

```typescript
export interface Sound {
  id: string;              // SHA-256 hash of file (64 chars hex)
  name: string;            // Filename
  path: string;            // Absolute path to file
  duration: number;        // Duration in seconds

  // User properties
  volume: number;          // 0.0-2.0, default 1.0
  speed: number;           // 0.5-2.0, default 1.0
  hotkey?: string;         // Keyboard shortcut
  customName?: string;     // Custom name
  image?: PadImage;        // Pad image
  folderIds: string[];     // Folders containing this sound

  // Metadata
  isPlaying: boolean;      // Playback state (runtime, not persisted)
  addedAt: number;         // Timestamp when added
}
```

### SoundPad (virtual, for display only)

```typescript
export interface SoundPad {
  index: number;           // Position in grid (0, 1, 2, ...)
  sound: Sound | null;     // Reference to sound or null if empty
  color: string;           // Color generated from index
}
```

### Persistence (soundboard.json)

```json
{
  "sounds": {
    "a1b2c3d4...": { "name": "explosion.mp3", "path": "...", "volume": 1.0, ... },
    "e5f6g7h8...": { "name": "music.mp3", "path": "...", "volume": 0.8, ... }
  },
  "folders": [
    { "id": "all", "name": "Tous", "createdAt": 1234567890 }
  ]
}
```

## SoundboardService

### Internal State

```typescript
// Source of truth
private _sounds = signal<Map<string, Sound>>(new Map());
private _folders = signal<Folder[]>([{ id: 'all', name: 'Tous', createdAt: Date.now() }]);
private _activeFolderId = signal<string>('all');

// Public signals
readonly sounds = this._sounds.asReadonly();
readonly folders = this._folders.asReadonly();
readonly activeFolderId = this._activeFolderId.asReadonly();
```

### Virtual Grid (computed)

```typescript
readonly pads = computed(() => {
  const sounds = this._sounds();
  const activeFolderId = this._activeFolderId();

  // Filter sounds by folder
  let filteredSounds = Array.from(sounds.values());
  if (activeFolderId !== 'all') {
    filteredSounds = filteredSounds.filter(s => s.folderIds.includes(activeFolderId));
  }

  // Sort alphabetically
  filteredSounds.sort((a, b) =>
    (a.customName || a.name).toLowerCase()
      .localeCompare((b.customName || b.name).toLowerCase())
  );

  // Generate virtual grid
  const minPads = Math.max(12, Math.ceil(filteredSounds.length / 4) * 4 + 4);
  const pads: SoundPad[] = [];

  for (let i = 0; i < minPads; i++) {
    pads.push({
      index: i,
      sound: filteredSounds[i] || null,
      color: PAD_COLORS[i % PAD_COLORS.length]
    });
  }

  return pads;
});
```

### Actions use sound.id

```typescript
playSound(soundId: string): void { ... }
stopSound(soundId: string): void { ... }
setSoundVolume(soundId: string, volume: number): void { ... }
// etc.
```

## Import Flow

1. **User selects audio file**
2. **Rust backend**:
   - Reads file
   - Calculates SHA-256 of content
   - Decodes metadata (duration, etc.)
   - Returns `{ hash, name, path, duration }`
3. **Frontend**:
   - Checks if hash exists → duplicate detected
   - Creates `Sound` object with defaults
   - Adds to `_sounds`

### New Rust Command

```rust
#[derive(Serialize)]
pub struct ImportedSound {
    pub hash: String,      // SHA-256 hex
    pub name: String,
    pub path: String,
    pub duration: f64,
}

#[tauri::command]
pub async fn import_sound_file(path: String) -> Result<ImportedSound, String> {
    // Read file
    let data = std::fs::read(&path).map_err(|e| e.to_string())?;

    // Calculate SHA-256
    use sha2::{Sha256, Digest};
    let hash = format!("{:x}", Sha256::digest(&data));

    // Decode duration (existing)
    let duration = decode_audio_duration(&path)?;

    // Extract filename
    let name = Path::new(&path).file_name()...;

    Ok(ImportedSound { hash, name, path, duration })
}
```

### Duplicate Detection

```typescript
async importSound(): Promise<void> {
  const result = await this.tauri.importSoundFile(path);

  if (this._sounds().has(result.hash)) {
    console.warn('Duplicate detected, ignored');
    return;
  }

  const sound: Sound = {
    id: result.hash,
    name: result.name,
    // ... default values
  };

  this._sounds.update(sounds => {
    const updated = new Map(sounds);
    updated.set(sound.id, sound);
    return updated;
  });
}
```

## UI Components

### SoundboardComponent

```typescript
// Template - uses sound.id for all actions
@for (pad of soundboard.pads(); track pad.sound?.id ?? pad.index) {
  <app-sound-pad
    [pad]="pad"
    (play)="soundboard.playSound(pad.sound!.id)"
    (volumeChange)="soundboard.setSoundVolume(pad.sound!.id, $event)"
    (folderToggle)="soundboard.toggleSoundFolder(pad.sound!.id, $event)"
  />
}
```

### Keyboard Shortcuts (global)

```typescript
handleKeydown(event: KeyboardEvent): void {
  // Iterate over all sounds (not pads)
  for (const sound of this.soundboard.sounds().values()) {
    if (sound.hotkey && eventMatchesShortcut(event, sound.hotkey)) {
      this.soundboard.playSound(sound.id);
      return;
    }
  }
}
```

**Advantage**: Shortcuts work regardless of active folder, since we iterate over sounds, not the grid.

## Data Migration

### Strategy

On load, if `sounds` doesn't exist but `pads` exists:

```typescript
private async loadState(): Promise<void> {
  const data = await this.tauri.loadSoundboardState();

  // New format
  if (data.sounds) {
    this._sounds.set(new Map(Object.entries(data.sounds)));
    this._folders.set(data.folders);
    return;
  }

  // Migration from old format
  if (data.pads) {
    const sounds = new Map<string, Sound>();

    for (const pad of data.pads) {
      if (!pad.sound) continue;

      // Calculate hash (backend call)
      const hash = await this.tauri.hashFile(pad.sound.path);

      sounds.set(hash, {
        id: hash,
        name: pad.sound.name,
        path: pad.sound.path,
        duration: pad.sound.duration,
        volume: pad.volume ?? 1.0,
        speed: pad.speed ?? 1.0,
        hotkey: pad.hotkey,
        customName: pad.customName,
        image: pad.image,
        folderIds: pad.folderIds ?? [],
        isPlaying: false,
        addedAt: Date.now()
      });
    }

    this._sounds.set(sounds);
    await this.saveState(); // Save in new format
  }
}
```

### Rust Command for Hashing Existing File

```rust
#[tauri::command]
pub async fn hash_file(path: String) -> Result<String, String> {
  let data = std::fs::read(&path).map_err(|e| e.to_string())?;
  Ok(format!("{:x}", Sha256::digest(&data)))
}
```

## Files to Modify

### Frontend (TypeScript/Angular)

| File | Changes |
|------|---------|
| `audio-device.model.ts` | New `Sound` interface, simplify `SoundPad` |
| `soundboard.service.ts` | Complete refactor: `_sounds` Map, `pads` computed |
| `tauri.service.ts` | New methods: `importSoundFile`, `hashFile` |
| `soundboard.component.ts` | Actions via `sound.id` |
| `sound-pad.component.ts` | Adapt to new props |
| `shortcut.service.ts` | Iterate over sounds, not pads |

### Backend (Rust)

| File | Changes |
|------|---------|
| `Cargo.toml` | Add `sha2` dependency |
| `commands.rs` | `import_sound_file`, `hash_file`, refactor `save/load_soundboard` |

## Benefits

- Shortcuts work in all folders
- Automatic duplicate detection
- Ready for cloud synchronization
- Cleaner and more maintainable architecture
