# Bulk Import Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add bulk import functionality with dynamic soundboard that auto-adds/removes rows.

**Architecture:** Frontend-driven orchestration with parallel file loading on backend. SoundboardService manages dynamic pad count, SoundboardComponent handles UI and drag & drop.

**Tech Stack:** Angular 19 (signals), Tauri 2, Rust (rodio for audio decoding)

---

## Task 1: Backend - Add load_multiple_sound_files command

**Files:**
- Modify: `/Users/didouye/Workspace/voiceboard/src-tauri/src/application/commands.rs`
- Modify: `/Users/didouye/Workspace/voiceboard/src-tauri/src/main.rs` (register command)

**Step 1: Add the new command in commands.rs**

Add after the existing `load_sound_file` function (around line 780):

```rust
/// Load multiple audio files in parallel, returning results for each
#[tauri::command]
pub async fn load_multiple_sound_files(paths: Vec<String>) -> Vec<Result<SoundFileDto, String>> {
    use std::sync::Arc;
    use tokio::task::JoinSet;

    let mut join_set = JoinSet::new();

    for path in paths {
        join_set.spawn(async move {
            // Reuse the logic from load_sound_file
            load_sound_file_internal(&path).await
        });
    }

    let mut results = Vec::new();
    while let Some(res) = join_set.join_next().await {
        match res {
            Ok(inner) => results.push(inner),
            Err(e) => results.push(Err(format!("Task failed: {}", e))),
        }
    }

    results
}

/// Internal function to load a single sound file (shared logic)
async fn load_sound_file_internal(path: &str) -> Result<SoundFileDto, String> {
    use rodio::Source;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;

    tracing::info!("[load_sound_file_internal] Loading: {}", path);

    let file_path = Path::new(path);

    if !file_path.exists() {
        return Err(format!("File not found: {}", path));
    }

    let name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);

    let decoder = rodio::Decoder::new(reader)
        .map_err(|e| format!("Failed to decode audio file: {}", e))?;

    let sample_rate = decoder.sample_rate();
    let channels = decoder.channels();
    let duration = decoder
        .total_duration()
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);

    let id = format!(
        "sound_{}",
        &uuid::Uuid::new_v4().to_string().replace("-", "")[..8]
    );

    Ok(SoundFileDto {
        id,
        name,
        path: path.to_string(),
        duration,
        sample_rate,
        channels,
    })
}
```

**Step 2: Update load_sound_file to use internal function**

Replace the existing `load_sound_file` function body:

```rust
#[tauri::command]
pub async fn load_sound_file(path: String) -> Result<SoundFileDto, String> {
    load_sound_file_internal(&path).await
}
```

**Step 3: Register the command in main.rs**

Find the `.invoke_handler(tauri::generate_handler![...])` and add `load_multiple_sound_files`:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    commands::load_multiple_sound_files,
])
```

**Step 4: Verify it compiles**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: No errors

**Step 5: Commit**

```bash
git add src-tauri/src/application/commands.rs src-tauri/src/main.rs
git commit -m "feat(backend): add load_multiple_sound_files command"
```

---

## Task 2: Frontend - Add loadMultipleSoundFiles to TauriService

**Files:**
- Modify: `/Users/didouye/Workspace/voiceboard/src/app/core/services/tauri.service.ts`

**Step 1: Add the new method**

Add after `loadSoundFile` method (around line 298):

```typescript
/**
 * Load multiple audio files in parallel
 * Returns an array of results (success with SoundFile or error string)
 */
async loadMultipleSoundFiles(paths: string[]): Promise<Array<{ ok: SoundFile } | { err: string }>> {
  const results = await invoke<Array<{ Ok?: any; Err?: string }>>('load_multiple_sound_files', { paths });
  return results.map(r => {
    if (r.Ok) {
      return {
        ok: {
          id: r.Ok.id,
          name: r.Ok.name,
          path: r.Ok.path,
          duration: r.Ok.duration,
          sampleRate: r.Ok.sample_rate,
          channels: r.Ok.channels
        }
      };
    } else {
      return { err: r.Err || 'Unknown error' };
    }
  });
}
```

**Step 2: Verify it compiles**

Run: `npm run build`
Expected: No errors

**Step 3: Commit**

```bash
git add src/app/core/services/tauri.service.ts
git commit -m "feat(frontend): add loadMultipleSoundFiles to TauriService"
```

---

## Task 3: SoundboardService - Add dynamic pad management methods

**Files:**
- Modify: `/Users/didouye/Workspace/voiceboard/src/app/core/services/soundboard.service.ts`

**Step 1: Add ensureEmptyPadAvailable method**

Add after `addPads` method (around line 132):

```typescript
/**
 * Ensure there's at least one empty pad available.
 * If the last empty pad was just filled, add a new row of 4 pads.
 */
private ensureEmptyPadAvailable(): void {
  const pads = this._pads();
  const hasEmptyPad = pads.some(p => p.sound === null);

  if (!hasEmptyPad) {
    this.addPads(4);
  }
}
```

**Step 2: Add cleanupEmptyRows method**

Add after `ensureEmptyPadAvailable`:

```typescript
/**
 * Remove empty rows from the end, keeping:
 * - Minimum 12 pads (3 rows)
 * - At least 1 empty pad
 */
private cleanupEmptyRows(): void {
  const pads = this._pads();
  const minPads = 12;

  if (pads.length <= minPads) return;

  // Find the last pad with a sound
  let lastFilledIndex = -1;
  for (let i = pads.length - 1; i >= 0; i--) {
    if (pads[i].sound !== null) {
      lastFilledIndex = i;
      break;
    }
  }

  // Calculate how many pads to keep (round up to complete row of 4, plus ensure 1 empty)
  const padsNeeded = lastFilledIndex + 1;
  const rowsNeeded = Math.ceil((padsNeeded + 1) / 4); // +1 to ensure at least 1 empty pad
  const padsToKeep = Math.max(minPads, rowsNeeded * 4);

  if (pads.length > padsToKeep) {
    this._pads.set(pads.slice(0, padsToKeep));
  }
}
```

**Step 3: Modify importSound to call ensureEmptyPadAvailable**

In `importSound` method, add after `await this.saveState();` (around line 177):

```typescript
// Ensure there's always an empty pad available
this.ensureEmptyPadAvailable();
```

**Step 4: Modify removeSound to call cleanupEmptyRows**

Replace the `removeSound` method:

```typescript
/**
 * Remove sound from a pad
 */
removeSound(padId: string): void {
  this._pads.update(pads => pads.map(pad =>
    pad.id === padId
      ? { ...pad, sound: null, isPlaying: false }
      : pad
  ));
  this.cleanupEmptyRows();
  this.saveState();
}
```

**Step 5: Verify it compiles**

Run: `npm run build`
Expected: No errors

**Step 6: Commit**

```bash
git add src/app/core/services/soundboard.service.ts
git commit -m "feat(soundboard): add dynamic pad management (auto-add/remove rows)"
```

---

## Task 4: SoundboardService - Add importMultipleSounds method

**Files:**
- Modify: `/Users/didouye/Workspace/voiceboard/src/app/core/services/soundboard.service.ts`

**Step 1: Add import for open with multiple selection**

The import already exists at line 4, no changes needed.

**Step 2: Add importMultipleSounds method**

Add after `importSound` method:

```typescript
/**
 * Import multiple sound files at once
 * Opens a multi-file dialog and assigns sounds to empty pads
 */
async importMultipleSounds(): Promise<{ imported: number; errors: string[] }> {
  try {
    this._loading.set(true);
    this._error.set(null);

    // Open file dialog with multiple selection
    const selected = await open({
      multiple: true,
      filters: [{
        name: 'Audio Files',
        extensions: ['mp3', 'ogg', 'wav', 'flac']
      }]
    });

    if (!selected || (Array.isArray(selected) && selected.length === 0)) {
      this._loading.set(false);
      return { imported: 0, errors: [] };
    }

    const paths = Array.isArray(selected) ? selected : [selected];
    return await this.importSoundsFromPaths(paths);
  } catch (err) {
    this._error.set(err instanceof Error ? err.message : String(err));
    return { imported: 0, errors: [String(err)] };
  } finally {
    this._loading.set(false);
  }
}

/**
 * Import sounds from an array of file paths
 * Used by both button import and drag & drop
 */
async importSoundsFromPaths(paths: string[]): Promise<{ imported: number; errors: string[] }> {
  if (paths.length === 0) {
    return { imported: 0, errors: [] };
  }

  // Sort paths alphabetically by filename
  const sortedPaths = [...paths].sort((a, b) => {
    const nameA = a.split('/').pop()?.toLowerCase() || a;
    const nameB = b.split('/').pop()?.toLowerCase() || b;
    return nameA.localeCompare(nameB);
  });

  // Load all files in parallel
  const results = await this.tauri.loadMultipleSoundFiles(sortedPaths);

  // Separate successes and errors
  const successfulSounds: { sound: SoundFile; originalPath: string }[] = [];
  const errors: string[] = [];

  results.forEach((result, index) => {
    if ('ok' in result) {
      successfulSounds.push({ sound: result.ok, originalPath: sortedPaths[index] });
    } else {
      const fileName = sortedPaths[index].split('/').pop() || sortedPaths[index];
      errors.push(`${fileName}: ${result.err}`);
    }
  });

  if (successfulSounds.length === 0) {
    return { imported: 0, errors };
  }

  // Ensure we have enough empty pads
  this.ensurePadsForImport(successfulSounds.length);

  // Find empty pads and assign sounds
  const pads = this._pads();
  const emptyPadIds: string[] = [];
  for (const pad of pads) {
    if (pad.sound === null && emptyPadIds.length < successfulSounds.length) {
      emptyPadIds.push(pad.id);
    }
  }

  // Assign sounds to empty pads
  this._pads.update(currentPads => {
    const updatedPads = [...currentPads];
    successfulSounds.forEach((item, index) => {
      const padId = emptyPadIds[index];
      const padIndex = updatedPads.findIndex(p => p.id === padId);
      if (padIndex !== -1) {
        updatedPads[padIndex] = { ...updatedPads[padIndex], sound: item.sound };
      }
    });
    return updatedPads;
  });

  // Ensure there's still an empty pad available after import
  this.ensureEmptyPadAvailable();

  await this.saveState();

  return { imported: successfulSounds.length, errors };
}

/**
 * Ensure there are enough pads for the import
 * Adds rows as needed to accommodate all files plus 1 empty pad
 */
private ensurePadsForImport(fileCount: number): void {
  const pads = this._pads();
  const emptyPads = pads.filter(p => p.sound === null).length;

  // Need fileCount empty pads + 1 extra for the "always have 1 empty" rule
  const padsNeeded = fileCount + 1;

  if (emptyPads < padsNeeded) {
    const additionalPadsNeeded = padsNeeded - emptyPads;
    const rowsToAdd = Math.ceil(additionalPadsNeeded / 4);
    this.addPads(rowsToAdd * 4);
  }
}
```

**Step 3: Make ensureEmptyPadAvailable and ensurePadsForImport accessible**

Change `ensureEmptyPadAvailable` from `private` to just have no modifier (keep as private but accessible within class).

**Step 4: Verify it compiles**

Run: `npm run build`
Expected: No errors

**Step 5: Commit**

```bash
git add src/app/core/services/soundboard.service.ts
git commit -m "feat(soundboard): add importMultipleSounds with bulk import logic"
```

---

## Task 5: SoundboardComponent - Update UI (remove Add Pads, add Import Multiple)

**Files:**
- Modify: `/Users/didouye/Workspace/voiceboard/src/app/features/soundboard/soundboard.component.ts`

**Step 1: Remove the "Add Pads" button**

In the template, remove this line (around line 23):
```html
<button class="btn-add-pads" (click)="soundboard.addPads(4)">+ Add Pads</button>
```

**Step 2: Add Import Multiple button below the grid**

Add after the closing `</div>` of `pads-grid` (around line 47):

```html
<div class="soundboard-footer">
  <button
    class="btn-import-multiple"
    (click)="importMultiple()"
    [disabled]="soundboard.loading()"
  >
    <span class="icon">📁</span>
    Import Multiple
  </button>
</div>
```

**Step 3: Add styles for the footer and button**

Add to the styles array:

```css
.soundboard-footer {
  display: flex;
  justify-content: center;
  margin-top: 16px;
}

.btn-import-multiple {
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(255, 255, 255, 0.08);
  border: 1px dashed rgba(255, 255, 255, 0.3);
  color: #aaa;
  padding: 10px 20px;
  border-radius: 8px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: all 0.2s;
}

.btn-import-multiple:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.12);
  border-color: rgba(255, 255, 255, 0.5);
  color: #fff;
}

.btn-import-multiple:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-import-multiple .icon {
  font-size: 1.1rem;
}
```

**Step 4: Remove btn-add-pads styles**

Remove the `.btn-add-pads` and `.btn-add-pads:hover` CSS rules (lines 90-104).

**Step 5: Add importMultiple method to component**

Add after `getHotkey` method:

```typescript
/**
 * Handle Import Multiple button click
 */
async importMultiple(): Promise<void> {
  const result = await this.soundboard.importMultipleSounds();

  if (result.errors.length > 0) {
    const errorMessage = `Imported ${result.imported} files.\nFailed (${result.errors.length}):\n${result.errors.join('\n')}`;
    console.warn(errorMessage);
    // TODO: Show toast notification instead of console
  }
}
```

**Step 6: Verify it compiles**

Run: `npm run build`
Expected: No errors

**Step 7: Commit**

```bash
git add src/app/features/soundboard/soundboard.component.ts
git commit -m "feat(ui): replace Add Pads with Import Multiple button"
```

---

## Task 6: SoundboardComponent - Add drag & drop support

**Files:**
- Modify: `/Users/didouye/Workspace/voiceboard/src/app/features/soundboard/soundboard.component.ts`

**Step 1: Add drag state signal**

Add to component class (after constructor):

```typescript
isDragging = signal(false);
dragFileCount = signal(0);
```

**Step 2: Update template with drag overlay**

Wrap the `pads-grid` in a container and add overlay. Replace the `pads-grid` div:

```html
<div
  class="pads-container"
  (dragover)="onDragOver($event)"
  (dragleave)="onDragLeave($event)"
  (drop)="onDrop($event)"
>
  <div class="pads-grid">
    @for (pad of soundboard.pads(); track pad.id; let i = $index) {
      <app-sound-pad
        [pad]="pad"
        [hotkey]="getHotkey(i)"
        [loading]="soundboard.loading()"
        [isPreviewing]="soundboard.previewingPadId() === pad.id"
        (play)="soundboard.playSound(pad.id)"
        (preview)="soundboard.previewSound(pad.id)"
        (import)="soundboard.importSound(pad.id)"
        (remove)="soundboard.removeSound(pad.id)"
      />
    }
  </div>

  @if (isDragging()) {
    <div class="drop-overlay">
      <span>Drop to import {{ dragFileCount() }} file{{ dragFileCount() > 1 ? 's' : '' }}</span>
    </div>
  }
</div>
```

**Step 3: Add styles for drag & drop**

Add to styles:

```css
.pads-container {
  position: relative;
}

.drop-overlay {
  position: absolute;
  inset: 0;
  background: rgba(52, 152, 219, 0.85);
  border: 3px dashed #fff;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10;
}

.drop-overlay span {
  color: #fff;
  font-size: 1.2rem;
  font-weight: 500;
}
```

**Step 4: Add drag & drop event handlers**

Add to component class:

```typescript
private readonly AUDIO_EXTENSIONS = ['mp3', 'ogg', 'wav', 'flac'];

onDragOver(event: DragEvent): void {
  event.preventDefault();
  event.stopPropagation();

  if (!event.dataTransfer) return;

  // Check if dragging files
  if (event.dataTransfer.types.includes('Files')) {
    event.dataTransfer.dropEffect = 'copy';

    // Count audio files
    const items = Array.from(event.dataTransfer.items);
    const audioCount = items.filter(item => {
      if (item.kind !== 'file') return false;
      const type = item.type.toLowerCase();
      return type.startsWith('audio/') ||
             this.AUDIO_EXTENSIONS.some(ext => type.includes(ext));
    }).length;

    // If we can't determine from MIME, use total count
    const count = audioCount > 0 ? audioCount : items.filter(i => i.kind === 'file').length;

    this.dragFileCount.set(count);
    this.isDragging.set(true);
  }
}

onDragLeave(event: DragEvent): void {
  event.preventDefault();
  event.stopPropagation();

  // Only hide overlay if leaving the container (not entering a child)
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
  const x = event.clientX;
  const y = event.clientY;

  if (x < rect.left || x > rect.right || y < rect.top || y > rect.bottom) {
    this.isDragging.set(false);
    this.dragFileCount.set(0);
  }
}

async onDrop(event: DragEvent): Promise<void> {
  event.preventDefault();
  event.stopPropagation();

  this.isDragging.set(false);
  this.dragFileCount.set(0);

  if (!event.dataTransfer) return;

  const files = Array.from(event.dataTransfer.files);
  const audioPaths = files
    .filter(file => {
      const ext = file.name.split('.').pop()?.toLowerCase();
      return ext && this.AUDIO_EXTENSIONS.includes(ext);
    })
    .map(file => file.path);

  if (audioPaths.length === 0) return;

  const result = await this.soundboard.importSoundsFromPaths(audioPaths);

  if (result.errors.length > 0) {
    console.warn(`Imported ${result.imported} files. Failed: ${result.errors.join(', ')}`);
  }
}
```

**Step 5: Add signal import**

Update the import at line 1:

```typescript
import { Component, HostListener, signal } from '@angular/core';
```

**Step 6: Verify it compiles**

Run: `npm run build`
Expected: No errors

**Step 7: Commit**

```bash
git add src/app/features/soundboard/soundboard.component.ts
git commit -m "feat(ui): add drag & drop support for bulk import"
```

---

## Task 7: Final verification and ROADMAP update

**Files:**
- Modify: `/Users/didouye/Workspace/voiceboard/ROADMAP.md`

**Step 1: Run the full build**

Run: `npm run build && cargo build --manifest-path src-tauri/Cargo.toml`
Expected: No errors

**Step 2: Run linting**

Run: `cargo clippy --manifest-path src-tauri/Cargo.toml && npm run lint`
Expected: No errors (or only pre-existing warnings)

**Step 3: Test manually**

1. Start the app: `npm run tauri dev`
2. Test Import Multiple button:
   - Click "Import Multiple"
   - Select multiple audio files
   - Verify they appear in pads sorted alphabetically
3. Test drag & drop:
   - Drag audio files onto the soundboard
   - Verify overlay appears with file count
   - Drop and verify import
4. Test dynamic rows:
   - Fill all pads → verify new row appears
   - Remove sounds → verify empty rows are removed
5. Verify minimum of 12 pads is maintained

**Step 4: Update ROADMAP.md**

Mark the Bulk Import task as done:

```markdown
- [x] Bulk import - Import multiple audio files at once
```

**Step 5: Commit**

```bash
git add ROADMAP.md
git commit -m "docs: mark bulk import as complete in roadmap"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Backend load_multiple_sound_files | commands.rs, main.rs |
| 2 | TauriService loadMultipleSoundFiles | tauri.service.ts |
| 3 | Dynamic pad management | soundboard.service.ts |
| 4 | importMultipleSounds method | soundboard.service.ts |
| 5 | UI: Import Multiple button | soundboard.component.ts |
| 6 | Drag & drop support | soundboard.component.ts |
| 7 | Final verification | ROADMAP.md |
