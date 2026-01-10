# Sound Model Refactoring Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Refactor data model to separate Sound (entity with SHA-256 ID) from Pad (virtual display position), fixing the bug where wrong sound plays in filtered folders.

**Architecture:** Sounds are stored in a Map keyed by SHA-256 hash. Pads are generated virtually from sounds based on active folder filter. All properties (volume, speed, hotkey, etc.) belong to Sound. Migration handles existing data.

**Tech Stack:** Angular 19 signals, Tauri 2, Rust with sha2 crate, TypeScript

---

## Task 1: Add sha2 Dependency to Rust

**Files:**
- Modify: `src-tauri/Cargo.toml`

**Step 1: Add sha2 to dependencies**

Add after the `# Audio` section (around line 37):

```toml
# Hashing
sha2 = "0.10"
```

**Step 2: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Success

**Step 3: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "build: add sha2 dependency for sound file hashing

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Add hash_file and import_sound_with_hash Commands

**Files:**
- Modify: `src-tauri/src/application/commands.rs`

**Step 1: Add hash_file command**

Add after `load_sound_file_internal` function (around line 835):

```rust
/// Calculate SHA-256 hash of a file
#[tauri::command]
pub async fn hash_file(path: String) -> Result<String, String> {
    use sha2::{Sha256, Digest};

    let data = std::fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))?;
    let hash = format!("{:x}", Sha256::digest(&data));
    Ok(hash)
}

/// DTO for imported sound with hash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedSoundDto {
    pub hash: String,
    pub name: String,
    pub path: String,
    pub duration: f64,
}

/// Import a sound file and return its hash and metadata
#[tauri::command]
pub async fn import_sound_with_hash(path: String) -> Result<ImportedSoundDto, String> {
    use sha2::{Sha256, Digest};
    use rodio::Source;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;

    // Read file and calculate hash
    let data = std::fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))?;
    let hash = format!("{:x}", Sha256::digest(&data));

    // Decode audio to get duration
    let file = File::open(&path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);
    let decoder = rodio::Decoder::new(reader)
        .map_err(|e| format!("Failed to decode audio: {}", e))?;

    let duration = decoder
        .total_duration()
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);

    // Extract filename
    let name = Path::new(&path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(ImportedSoundDto {
        hash,
        name,
        path,
        duration,
    })
}

/// Import multiple sound files with hashes in parallel
#[tauri::command]
pub async fn import_multiple_sounds_with_hash(paths: Vec<String>) -> Vec<Result<ImportedSoundDto, String>> {
    use futures::future::join_all;

    let futures: Vec<_> = paths
        .into_iter()
        .map(|p| import_sound_with_hash(p))
        .collect();

    join_all(futures).await
}
```

**Step 2: Register commands in lib.rs**

In `src-tauri/src/lib.rs`, add to the invoke_handler:

```rust
application::commands::hash_file,
application::commands::import_sound_with_hash,
application::commands::import_multiple_sounds_with_hash,
```

**Step 3: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Success

**Step 4: Run tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: All tests pass

**Step 5: Commit**

```bash
git add src-tauri/src/application/commands.rs src-tauri/src/lib.rs
git commit -m "feat: add hash_file and import_sound_with_hash commands

- hash_file: Calculate SHA-256 of any file
- import_sound_with_hash: Import sound and return hash + metadata
- import_multiple_sounds_with_hash: Parallel import

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Create New Sound Interface

**Files:**
- Modify: `src/app/core/models/audio-device.model.ts`

**Step 1: Add Sound interface**

Add after `SoundFile` interface (around line 62):

```typescript
/**
 * Sound entity identified by SHA-256 hash
 * Contains all properties previously on SoundPad
 */
export interface Sound {
  /** SHA-256 hash of file content (64 chars hex) */
  id: string;
  /** Filename without extension */
  name: string;
  /** Absolute path to audio file */
  path: string;
  /** Duration in seconds */
  duration: number;

  // User properties
  /** Volume level (0.0-2.0, default 1.0 = 100%) */
  volume: number;
  /** Playback speed (0.5-2.0, default 1.0 = normal) */
  speed: number;
  /** Keyboard shortcut */
  hotkey?: string;
  /** User-defined custom name */
  customName?: string;
  /** Custom image */
  image?: PadImage;
  /** Folder IDs this sound belongs to */
  folderIds: string[];

  // Runtime state (not persisted)
  /** Whether sound is currently playing */
  isPlaying: boolean;

  // Metadata
  /** Timestamp when sound was added */
  addedAt: number;
}
```

**Step 2: Simplify SoundPad interface**

Replace the existing `SoundPad` interface:

```typescript
/**
 * Virtual pad for display (generated from sounds)
 */
export interface SoundPad {
  /** Position in grid (0, 1, 2, ...) */
  index: number;
  /** Reference to sound or null if empty */
  sound: Sound | null;
  /** Color generated from index */
  color: string;
}
```

**Step 3: Verify build fails (expected)**

Run: `npm run build 2>&1 | head -30`
Expected: Many TypeScript errors (old SoundPad fields missing)

**Step 4: Commit partial progress**

```bash
git add src/app/core/models/audio-device.model.ts
git commit -m "feat: add Sound interface, simplify SoundPad to virtual display

BREAKING: SoundPad no longer contains sound properties
Next: Update SoundboardService to use new model

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Add TauriService Methods for Hash Import

**Files:**
- Modify: `src/app/core/services/tauri.service.ts`

**Step 1: Add ImportedSound interface and methods**

Add after the existing `loadSoundboardState` method (around line 500):

```typescript
/**
 * Imported sound with hash
 */
export interface ImportedSound {
  hash: string;
  name: string;
  path: string;
  duration: number;
}

/**
 * Import a sound file and get its hash
 */
async importSoundWithHash(path: string): Promise<ImportedSound> {
  if (this.demoService.isDemoMode) {
    return {
      hash: 'demo_' + Math.random().toString(36).substring(7),
      name: path.split('/').pop()?.replace(/\.[^.]+$/, '') || 'demo',
      path,
      duration: 5.0
    };
  }
  return invoke<ImportedSound>('import_sound_with_hash', { path });
}

/**
 * Import multiple sound files with hashes
 */
async importMultipleSoundsWithHash(paths: string[]): Promise<Array<{ ok: ImportedSound } | { err: string }>> {
  if (this.demoService.isDemoMode) {
    return paths.map(path => ({
      ok: {
        hash: 'demo_' + Math.random().toString(36).substring(7),
        name: path.split('/').pop()?.replace(/\.[^.]+$/, '') || 'demo',
        path,
        duration: 5.0
      }
    }));
  }
  const results = await invoke<Array<{ Ok: ImportedSound } | { Err: string }>>('import_multiple_sounds_with_hash', { paths });
  return results.map(r => 'Ok' in r ? { ok: r.Ok } : { err: r.Err });
}

/**
 * Calculate SHA-256 hash of a file
 */
async hashFile(path: string): Promise<string> {
  if (this.demoService.isDemoMode) {
    return 'demo_' + Math.random().toString(36).substring(7);
  }
  return invoke<string>('hash_file', { path });
}
```

**Step 2: Verify build (still failing, expected)**

Run: `npm run build 2>&1 | grep -c error`
Expected: Multiple errors (SoundboardService not yet updated)

**Step 3: Commit**

```bash
git add src/app/core/services/tauri.service.ts
git commit -m "feat: add TauriService methods for hash-based sound import

- importSoundWithHash: Import single sound with SHA-256
- importMultipleSoundsWithHash: Parallel import
- hashFile: Calculate hash of existing file

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Refactor SoundboardService - Core State

**Files:**
- Modify: `src/app/core/services/soundboard.service.ts`

**Step 1: Replace state with new model**

Replace the state section (around lines 28-62) with:

```typescript
const PAD_COLORS = [
  '#e74c3c', '#e67e22', '#f1c40f', '#2ecc71',
  '#1abc9c', '#3498db', '#9b59b6', '#e91e63',
  '#00bcd4', '#8bc34a', '#ff5722', '#795548'
];

@Injectable({
  providedIn: 'root'
})
export class SoundboardService {
  // Source of truth
  private _sounds = signal<Map<string, Sound>>(new Map());
  private _folders = signal<Folder[]>([{ id: 'all', name: 'Tous', createdAt: Date.now() }]);
  private _activeFolderId = signal<string>('all');
  private _loading = signal(false);
  private _error = signal<string | null>(null);
  private _initialized = false;

  // Preview state
  private _previewingSoundId = signal<string | null>(null);
  private _previewDeviceId = signal<string | null>(null);
  readonly previewingSoundId = this._previewingSoundId.asReadonly();
  readonly previewDeviceId = this._previewDeviceId.asReadonly();

  private unlistenPreviewStarted?: () => void;
  private unlistenPreviewStopped?: () => void;

  // Public readonly signals
  readonly sounds = this._sounds.asReadonly();
  readonly folders = this._folders.asReadonly();
  readonly activeFolderId = this._activeFolderId.asReadonly();
  readonly loading = this._loading.asReadonly();
  readonly error = this._error.asReadonly();

  readonly activeFolder = computed(() =>
    this._folders().find(f => f.id === this._activeFolderId()) || this._folders()[0]
  );

  readonly activeSounds = computed(() =>
    Array.from(this._sounds().values())
  );

  readonly playingCount = computed(() =>
    Array.from(this._sounds().values()).filter(s => s.isPlaying).length
  );

  /**
   * Virtual pad grid computed from sounds and active folder
   */
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

  constructor(private tauri: TauriService) {
    this.loadState();
    this.initPreviewListeners();
    this.initSoundFinishedListener();
  }
```

**Step 2: Update imports at top of file**

```typescript
import { Injectable, signal, computed } from '@angular/core';
import { TauriService } from './tauri.service';
import { Sound, SoundPad, Folder, PadImage } from '../models';
import { open } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';
```

This is a partial refactor - the file won't compile yet. Continue to next task.

**Step 3: Commit partial progress**

```bash
git add src/app/core/services/soundboard.service.ts
git commit -m "refactor: update SoundboardService state to use Sound Map

WIP: Service methods not yet updated

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Refactor SoundboardService - Core Methods

**Files:**
- Modify: `src/app/core/services/soundboard.service.ts`

**Step 1: Update/add core sound methods**

Replace the sound manipulation methods with:

```typescript
// =========================================================================
// Sound Operations
// =========================================================================

/**
 * Get a sound by ID
 */
getSound(soundId: string): Sound | undefined {
  return this._sounds().get(soundId);
}

/**
 * Add a sound to the store
 */
private addSound(sound: Sound): void {
  this._sounds.update(sounds => {
    const updated = new Map(sounds);
    updated.set(sound.id, sound);
    return updated;
  });
}

/**
 * Update a sound in the store
 */
private updateSound(soundId: string, updates: Partial<Sound>): void {
  this._sounds.update(sounds => {
    const sound = sounds.get(soundId);
    if (!sound) return sounds;

    const updated = new Map(sounds);
    updated.set(soundId, { ...sound, ...updates });
    return updated;
  });
}

/**
 * Remove a sound from the store
 */
removeSound(soundId: string): void {
  this._sounds.update(sounds => {
    const updated = new Map(sounds);
    updated.delete(soundId);
    return updated;
  });
  this.saveState();
}

/**
 * Play a sound
 */
async playSound(soundId: string): Promise<void> {
  const sound = this._sounds().get(soundId);
  if (!sound) return;

  try {
    await this.tauri.playSound(sound.path, sound.volume, sound.speed);
    this.updateSound(soundId, { isPlaying: true });
  } catch (err) {
    console.error('Failed to play sound:', err);
    this._error.set('Failed to play sound');
  }
}

/**
 * Stop a specific sound
 */
async stopSound(soundId: string): Promise<void> {
  const sound = this._sounds().get(soundId);
  if (!sound) return;

  try {
    await this.tauri.stopSound(sound.path);
    this.updateSound(soundId, { isPlaying: false });
  } catch (err) {
    console.error('Failed to stop sound:', err);
  }
}

/**
 * Stop all playing sounds
 */
async stopAll(): Promise<void> {
  try {
    await this.tauri.stopAllSounds();
    this._sounds.update(sounds => {
      const updated = new Map(sounds);
      for (const [id, sound] of updated) {
        if (sound.isPlaying) {
          updated.set(id, { ...sound, isPlaying: false });
        }
      }
      return updated;
    });
  } catch (err) {
    console.error('Failed to stop all sounds:', err);
  }
}

/**
 * Preview a sound on the preview output device
 */
async previewSound(soundId: string): Promise<void> {
  const sound = this._sounds().get(soundId);
  if (!sound) return;

  try {
    if (this._previewingSoundId() === soundId) {
      await this.tauri.stopPreview();
      this._previewingSoundId.set(null);
    } else {
      await this.tauri.previewSound(sound.path, sound.volume, sound.speed);
      this._previewingSoundId.set(soundId);
    }
  } catch (err) {
    console.error('Failed to preview sound:', err);
  }
}

// =========================================================================
// Sound Property Setters
// =========================================================================

setSoundVolume(soundId: string, volume: number): void {
  this.updateSound(soundId, { volume });
  this.saveState();
}

setSoundSpeed(soundId: string, speed: number): void {
  this.updateSound(soundId, { speed });
  this.saveState();
}

setSoundHotkey(soundId: string, hotkey: string | null): void {
  this.updateSound(soundId, { hotkey: hotkey || undefined });
  this.saveState();
}

setSoundCustomName(soundId: string, customName: string | null): void {
  this.updateSound(soundId, { customName: customName || undefined });
  this.saveState();
}

setSoundImage(soundId: string, image: PadImage | null): void {
  this.updateSound(soundId, { image: image || undefined });
  this.saveState();
}
```

**Step 2: Commit partial progress**

```bash
git add src/app/core/services/soundboard.service.ts
git commit -m "refactor: add Sound CRUD and property setter methods

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 7: Refactor SoundboardService - Import Methods

**Files:**
- Modify: `src/app/core/services/soundboard.service.ts`

**Step 1: Update import methods**

```typescript
// =========================================================================
// Import Operations
// =========================================================================

/**
 * Import a single sound file
 */
async importSound(): Promise<void> {
  try {
    this._loading.set(true);
    this._error.set(null);

    const selected = await open({
      multiple: false,
      filters: [{ name: 'Audio', extensions: ['mp3', 'ogg', 'wav', 'flac'] }]
    });

    if (!selected) return;

    const path = selected as string;
    const imported = await this.tauri.importSoundWithHash(path);

    // Check for duplicate
    if (this._sounds().has(imported.hash)) {
      this._error.set('This sound already exists in your library');
      return;
    }

    const sound: Sound = {
      id: imported.hash,
      name: imported.name,
      path: imported.path,
      duration: imported.duration,
      volume: 1.0,
      speed: 1.0,
      folderIds: [],
      isPlaying: false,
      addedAt: Date.now()
    };

    this.addSound(sound);
    await this.saveState();
  } catch (err) {
    console.error('Failed to import sound:', err);
    this._error.set('Failed to import sound');
  } finally {
    this._loading.set(false);
  }
}

/**
 * Import multiple sound files
 */
async importMultipleSounds(): Promise<{ imported: number; skippedDuplicates: number; errors: string[] }> {
  const result = { imported: 0, skippedDuplicates: 0, errors: [] as string[] };

  try {
    this._loading.set(true);
    this._error.set(null);

    const selected = await open({
      multiple: true,
      filters: [{ name: 'Audio', extensions: ['mp3', 'ogg', 'wav', 'flac'] }]
    });

    if (!selected || (Array.isArray(selected) && selected.length === 0)) {
      return result;
    }

    const paths = Array.isArray(selected) ? selected : [selected];
    const importResults = await this.tauri.importMultipleSoundsWithHash(paths);

    for (const res of importResults) {
      if ('err' in res) {
        result.errors.push(res.err);
        continue;
      }

      const imported = res.ok;

      // Check for duplicate
      if (this._sounds().has(imported.hash)) {
        result.skippedDuplicates++;
        continue;
      }

      const sound: Sound = {
        id: imported.hash,
        name: imported.name,
        path: imported.path,
        duration: imported.duration,
        volume: 1.0,
        speed: 1.0,
        folderIds: [],
        isPlaying: false,
        addedAt: Date.now()
      };

      this.addSound(sound);
      result.imported++;
    }

    if (result.imported > 0) {
      await this.saveState();
    }
  } catch (err) {
    console.error('Failed to import sounds:', err);
    result.errors.push(String(err));
  } finally {
    this._loading.set(false);
  }

  return result;
}

/**
 * Import sounds from file paths (for drag & drop)
 */
async importSoundsFromPaths(paths: string[]): Promise<{ imported: number; skippedDuplicates: number; errors: string[] }> {
  const result = { imported: 0, skippedDuplicates: 0, errors: [] as string[] };

  try {
    this._loading.set(true);
    const importResults = await this.tauri.importMultipleSoundsWithHash(paths);

    for (const res of importResults) {
      if ('err' in res) {
        result.errors.push(res.err);
        continue;
      }

      const imported = res.ok;

      if (this._sounds().has(imported.hash)) {
        result.skippedDuplicates++;
        continue;
      }

      const sound: Sound = {
        id: imported.hash,
        name: imported.name,
        path: imported.path,
        duration: imported.duration,
        volume: 1.0,
        speed: 1.0,
        folderIds: [],
        isPlaying: false,
        addedAt: Date.now()
      };

      this.addSound(sound);
      result.imported++;
    }

    if (result.imported > 0) {
      await this.saveState();
    }
  } catch (err) {
    result.errors.push(String(err));
  } finally {
    this._loading.set(false);
  }

  return result;
}
```

**Step 2: Commit**

```bash
git add src/app/core/services/soundboard.service.ts
git commit -m "refactor: update import methods to use hash-based Sound model

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 8: Refactor SoundboardService - Folder Methods

**Files:**
- Modify: `src/app/core/services/soundboard.service.ts`

**Step 1: Update folder methods to use soundId**

```typescript
// =========================================================================
// Folder Operations
// =========================================================================

setActiveFolder(folderId: string): void {
  if (this._folders().some(f => f.id === folderId)) {
    this._activeFolderId.set(folderId);
  }
}

createFolder(name: string): void {
  const trimmedName = name.trim();
  if (!trimmedName) return;

  if (this._folders().some(f => f.name.toLowerCase() === trimmedName.toLowerCase())) {
    return;
  }

  const id = `folder-${Date.now()}`;
  const newFolder: Folder = { id, name: trimmedName, createdAt: Date.now() };
  this._folders.update(folders => [...folders, newFolder]);
  this.saveFolders();
}

renameFolder(folderId: string, newName: string): void {
  if (folderId === 'all') return;

  const trimmedName = newName.trim();
  if (!trimmedName) return;

  if (this._folders().some(f => f.id !== folderId && f.name.toLowerCase() === trimmedName.toLowerCase())) {
    return;
  }

  this._folders.update(folders =>
    folders.map(f => f.id === folderId ? { ...f, name: trimmedName } : f)
  );
  this.saveFolders();
}

deleteFolder(folderId: string): void {
  if (folderId === 'all') return;

  // Remove folder from all sounds
  this._sounds.update(sounds => {
    const updated = new Map(sounds);
    for (const [id, sound] of updated) {
      if (sound.folderIds.includes(folderId)) {
        updated.set(id, {
          ...sound,
          folderIds: sound.folderIds.filter(f => f !== folderId)
        });
      }
    }
    return updated;
  });

  this._folders.update(folders => folders.filter(f => f.id !== folderId));

  if (this._activeFolderId() === folderId) {
    this._activeFolderId.set('all');
  }

  this.saveFolders();
  this.saveState();
}

toggleSoundFolder(soundId: string, folderId: string): void {
  if (folderId === 'all') return;

  const sound = this._sounds().get(soundId);
  if (!sound) return;

  const hasFolder = sound.folderIds.includes(folderId);
  this.updateSound(soundId, {
    folderIds: hasFolder
      ? sound.folderIds.filter(f => f !== folderId)
      : [...sound.folderIds, folderId]
  });
  this.saveState();
}

addSoundToFolder(soundId: string, folderId: string): void {
  if (folderId === 'all') return;

  const sound = this._sounds().get(soundId);
  if (!sound || sound.folderIds.includes(folderId)) return;

  this.updateSound(soundId, {
    folderIds: [...sound.folderIds, folderId]
  });
  this.saveState();
}
```

**Step 2: Commit**

```bash
git add src/app/core/services/soundboard.service.ts
git commit -m "refactor: update folder methods to work with Sound model

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 9: Refactor SoundboardService - Persistence and Migration

**Files:**
- Modify: `src/app/core/services/soundboard.service.ts`

**Step 1: Update persistence methods with migration support**

```typescript
// =========================================================================
// Persistence
// =========================================================================

private async loadState(): Promise<void> {
  try {
    // Load folders
    const savedFolders = await this.tauri.loadFolders();
    if (savedFolders && savedFolders.length > 0) {
      const hasAll = savedFolders.some(f => f.id === 'all');
      if (!hasAll) {
        savedFolders.unshift({ id: 'all', name: 'Tous', createdAt: 0 });
      }
      this._folders.set(savedFolders);
    }

    // Load sounds (new format)
    const data = await this.tauri.loadSoundboardState();

    if (data && data.sounds) {
      // New format: sounds as object
      const soundsMap = new Map<string, Sound>();
      for (const [id, soundData] of Object.entries(data.sounds as Record<string, any>)) {
        soundsMap.set(id, {
          ...soundData,
          isPlaying: false // Reset runtime state
        } as Sound);
      }
      this._sounds.set(soundsMap);
    } else if (data && Array.isArray(data)) {
      // Old format: pads array - migrate
      await this.migrateFromOldFormat(data);
    }

    this._initialized = true;
  } catch (err) {
    console.error('Failed to load state:', err);
    this._initialized = true;
  }
}

private async migrateFromOldFormat(pads: any[]): Promise<void> {
  console.log('Migrating from old pad format to new sound format...');
  const sounds = new Map<string, Sound>();

  for (const pad of pads) {
    if (!pad.sound) continue;

    try {
      // Calculate hash for existing file
      const hash = await this.tauri.hashFile(pad.sound.path);

      // Skip if already migrated (duplicate)
      if (sounds.has(hash)) continue;

      const sound: Sound = {
        id: hash,
        name: pad.sound.name,
        path: pad.sound.path,
        duration: pad.sound.duration || 0,
        volume: pad.volume ?? 1.0,
        speed: pad.speed ?? 1.0,
        hotkey: pad.hotkey,
        customName: pad.customName,
        image: pad.image,
        folderIds: pad.folderIds ?? [],
        isPlaying: false,
        addedAt: Date.now()
      };

      sounds.set(hash, sound);
    } catch (err) {
      console.error(`Failed to migrate sound ${pad.sound?.name}:`, err);
    }
  }

  this._sounds.set(sounds);
  await this.saveState();
  console.log(`Migration complete: ${sounds.size} sounds migrated`);
}

private async saveState(): Promise<void> {
  try {
    // Convert Map to object for JSON serialization
    const soundsObj: Record<string, Omit<Sound, 'isPlaying'>> = {};
    for (const [id, sound] of this._sounds()) {
      const { isPlaying, ...rest } = sound;
      soundsObj[id] = rest;
    }

    await this.tauri.saveSoundboardState({ sounds: soundsObj });
  } catch (err) {
    console.error('Failed to save state:', err);
  }
}

private async saveFolders(): Promise<void> {
  try {
    await this.tauri.saveFolders(this._folders());
  } catch (err) {
    console.error('Failed to save folders:', err);
  }
}

// =========================================================================
// Listeners
// =========================================================================

private async initSoundFinishedListener(): Promise<void> {
  try {
    await listen<{ id: string }>('sound-finished', (event) => {
      // Find sound by path (backend sends path as id)
      const soundPath = event.payload.id;
      for (const [id, sound] of this._sounds()) {
        if (sound.path === soundPath) {
          this.updateSound(id, { isPlaying: false });
          break;
        }
      }
    });
  } catch (e) {
    console.error('Failed to initialize sound-finished listener:', e);
  }
}

private async initPreviewListeners(): Promise<void> {
  this.unlistenPreviewStarted = await this.tauri.listenPreviewStarted((soundPath) => {
    // Find sound by path
    for (const [id, sound] of this._sounds()) {
      if (sound.path === soundPath) {
        this._previewingSoundId.set(id);
        break;
      }
    }
  });

  this.unlistenPreviewStopped = await this.tauri.listenPreviewStopped(() => {
    this._previewingSoundId.set(null);
  });
}

// =========================================================================
// Utilities
// =========================================================================

formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

clearError(): void {
  this._error.set(null);
}
```

**Step 2: Commit**

```bash
git add src/app/core/services/soundboard.service.ts
git commit -m "refactor: update persistence with migration from old format

- New format: { sounds: { hash: Sound } }
- Auto-migrates old pad array format
- Calculates hash for existing files during migration

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 10: Update TauriService saveSoundboardState

**Files:**
- Modify: `src/app/core/services/tauri.service.ts`

**Step 1: Update saveSoundboardState to accept new format**

Find `saveSoundboardState` and update:

```typescript
/**
 * Save soundboard state to persistent storage
 */
async saveSoundboardState(data: { sounds: Record<string, any> }): Promise<void> {
  if (this.demoService.isDemoMode) return;
  await invoke('save_soundboard', { pads: data });
}
```

**Step 2: Verify no TypeScript errors in tauri.service.ts**

Run: `npx tsc --noEmit src/app/core/services/tauri.service.ts 2>&1 | head -10`

**Step 3: Commit**

```bash
git add src/app/core/services/tauri.service.ts
git commit -m "refactor: update saveSoundboardState for new format

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 11: Update SoundPadComponent

**Files:**
- Modify: `src/app/features/soundboard/sound-pad/sound-pad.component.ts`

**Step 1: Update component to use new SoundPad interface**

Update inputs and property access:

```typescript
@Input({ required: true }) pad!: SoundPad;

// Update all pad.sound?.X references since sound is now Sound | null
// pad.sound.volume -> pad.sound?.volume (already correct pattern)
// pad.hotkey -> pad.sound?.hotkey
// pad.volume -> pad.sound?.volume
// pad.speed -> pad.sound?.speed
// pad.customName -> pad.sound?.customName
// pad.image -> pad.sound?.image
// pad.isPlaying -> pad.sound?.isPlaying
// pad.id -> pad.sound?.id (for actions)
// pad.color -> pad.color (still on pad)
```

Key changes needed:
- `[isPreviewing]` check uses `pad.sound?.id`
- All outputs emit `pad.sound!.id` instead of `pad.id`
- Volume/speed/hotkey display from `pad.sound`
- `folderIds` from `pad.sound?.folderIds`

**Step 2: Update template references**

This requires reading the full component and updating systematically. The key pattern is:
- `pad.volume` → `pad.sound?.volume ?? 1.0`
- `pad.hotkey` → `pad.sound?.hotkey`
- Etc.

**Step 3: Verify build**

Run: `npm run build 2>&1 | tail -20`

**Step 4: Commit**

```bash
git add src/app/features/soundboard/sound-pad/sound-pad.component.ts
git commit -m "refactor: update SoundPadComponent for new SoundPad interface

Properties now accessed via pad.sound instead of pad directly

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 12: Update SoundboardComponent

**Files:**
- Modify: `src/app/features/soundboard/soundboard.component.ts`

**Step 1: Update template to use sound.id**

Key changes:
- `track pad.id` → `track pad.sound?.id ?? pad.index`
- `soundboard.playSound(pad.id)` → `soundboard.playSound(pad.sound!.id)`
- `soundboard.previewSound(pad.id)` → `soundboard.previewSound(pad.sound!.id)`
- `soundboard.previewingPadId() === pad.id` → `soundboard.previewingSoundId() === pad.sound?.id`
- All other pad.id references → pad.sound!.id
- `onImportSound(pad.id)` → `onImportSound()` (no pad ID needed)
- Remove `getHotkey(i)` - hotkey comes from `pad.sound?.hotkey`

**Step 2: Update keyboard shortcut handler**

```typescript
@HostListener('window:keydown', ['$event'])
handleKeydown(event: KeyboardEvent): void {
  if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
    return;
  }

  if (event.key === 'Escape') {
    this.soundboard.stopAll();
    return;
  }

  // Check all sounds for matching hotkey
  for (const sound of this.soundboard.sounds().values()) {
    if (sound.hotkey && eventMatchesShortcut(event, sound.hotkey)) {
      event.preventDefault();
      this.soundboard.playSound(sound.id);
      return;
    }
  }
}
```

**Step 3: Verify build**

Run: `npm run build 2>&1 | tail -20`

**Step 4: Commit**

```bash
git add src/app/features/soundboard/soundboard.component.ts
git commit -m "refactor: update SoundboardComponent to use sound.id

- Actions use sound.id instead of pad.id
- Keyboard shortcuts iterate over sounds
- Import no longer needs pad ID

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 13: Update MixerComponent

**Files:**
- Modify: `src/app/features/mixer/mixer.component.ts`

**Step 1: Update drag & drop to use sound.id**

Update `onFolderDrop`:
```typescript
onFolderDrop(event: DragEvent, folder: Folder): void {
  event.preventDefault();
  this.dragOverFolderId.set(null);

  if (folder.id === 'all') return;

  const soundId = event.dataTransfer?.getData('text/plain');
  if (soundId) {
    this.soundboard.addSoundToFolder(soundId, folder.id);
  }
}
```

**Step 2: Verify build**

Run: `npm run build 2>&1 | tail -10`

**Step 3: Commit**

```bash
git add src/app/features/mixer/mixer.component.ts
git commit -m "refactor: update MixerComponent drag & drop for sound.id

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 14: Update ShortcutService

**Files:**
- Modify: `src/app/core/services/shortcut.service.ts`

**Step 1: Update to iterate over sounds**

Update `syncFromSoundboard`:
```typescript
async syncFromSoundboard(): Promise<void> {
  const sounds = Array.from(this.soundboard.sounds().values());

  await this.tauri.unregisterAllShortcuts();
  this.registry.clear();

  for (const sound of sounds) {
    if (sound.hotkey) {
      try {
        await this.tauri.registerGlobalShortcut(sound.id, sound.hotkey);
        this.registry.set(sound.hotkey, sound.id);
      } catch (err) {
        console.error(`Failed to register shortcut ${sound.hotkey} for ${sound.id}:`, err);
      }
    }
  }
}
```

Update the effect to watch sounds:
```typescript
effect(() => {
  const sounds = Array.from(this.soundboard.sounds().values());
  const snapshot = sounds
    .filter(s => s.hotkey)
    .map(s => `${s.hotkey}:${s.id}`)
    .sort()
    .join('|');

  if (this.initialized && snapshot !== this.lastHotkeySnapshot) {
    this.lastHotkeySnapshot = snapshot;
    this.syncFromSoundboard();
  }
});
```

**Step 2: Verify build**

Run: `npm run build 2>&1 | tail -10`

**Step 3: Commit**

```bash
git add src/app/core/services/shortcut.service.ts
git commit -m "refactor: update ShortcutService to use Sound model

- Iterate over sounds instead of pads
- Register shortcuts with sound.id

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 15: Update Test Files

**Files:**
- Modify: `src/app/features/soundboard/sound-pad/sound-pad.component.spec.ts`
- Modify: `src/app/core/services/soundboard.service.spec.ts`

**Step 1: Update mock Sound and SoundPad**

In spec files, update mock factories:

```typescript
function createMockSound(overrides: Partial<Sound> = {}): Sound {
  return {
    id: 'hash_abc123',
    name: 'test-sound',
    path: '/path/to/sound.mp3',
    duration: 5.0,
    volume: 1.0,
    speed: 1.0,
    folderIds: [],
    isPlaying: false,
    addedAt: Date.now(),
    ...overrides
  };
}

function createMockPad(overrides: Partial<SoundPad> = {}): SoundPad {
  return {
    index: 0,
    sound: createMockSound(),
    color: '#e74c3c',
    ...overrides
  };
}
```

**Step 2: Run tests**

Run: `npm test -- --no-watch --browsers=ChromeHeadless`
Expected: All tests pass

**Step 3: Commit**

```bash
git add src/app/features/soundboard/sound-pad/sound-pad.component.spec.ts src/app/core/services/soundboard.service.spec.ts
git commit -m "test: update mocks for new Sound model

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 16: Final Verification

**Step 1: Run full test suite**

```bash
npm run build && npm test -- --no-watch --browsers=ChromeHeadless && cargo test --manifest-path src-tauri/Cargo.toml
```

**Step 2: Run formatters**

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml
```

**Step 3: Manual testing checklist**

- [ ] Import a new sound - verify hash is generated
- [ ] Import duplicate - verify it's detected and skipped
- [ ] Create folder, add sound to it
- [ ] Switch to folder, click pad - verify correct sound plays
- [ ] Assign hotkey, verify it works in all folders
- [ ] Restart app - verify data persisted correctly
- [ ] Test with old data - verify migration works

**Step 4: Archive design and update ROADMAP**

```bash
mv docs/plans/2026-01-10-sound-model-refactoring-design.md docs/plans/archive/
git add docs/plans/
git commit -m "docs: archive sound model refactoring design

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Add sha2 dependency | Cargo.toml |
| 2 | Add hash commands | commands.rs, lib.rs |
| 3 | Create Sound interface | audio-device.model.ts |
| 4 | Add TauriService methods | tauri.service.ts |
| 5 | Refactor service state | soundboard.service.ts |
| 6 | Refactor core methods | soundboard.service.ts |
| 7 | Refactor import methods | soundboard.service.ts |
| 8 | Refactor folder methods | soundboard.service.ts |
| 9 | Add persistence + migration | soundboard.service.ts |
| 10 | Update save method | tauri.service.ts |
| 11 | Update SoundPadComponent | sound-pad.component.ts |
| 12 | Update SoundboardComponent | soundboard.component.ts |
| 13 | Update MixerComponent | mixer.component.ts |
| 14 | Update ShortcutService | shortcut.service.ts |
| 15 | Update tests | *.spec.ts |
| 16 | Final verification | All |
