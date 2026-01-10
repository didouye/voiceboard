# Folder Management Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement a folder/tagging system where sounds can belong to multiple folders, with full CRUD operations and drag & drop support.

**Architecture:** Folders work as tags - each pad has a `folderIds` array. A special "All" folder shows everything. Filtering is done via computed signals. Persistence extends existing soundboard.json.

**Tech Stack:** Angular 19 signals, Tauri 2, Rust backend, Tailwind CSS

---

## Task 1: Update Data Model

**Files:**
- Modify: `src/app/core/models/audio-device.model.ts:79-93`

**Step 1: Add folderIds to SoundPad interface**

```typescript
// In audio-device.model.ts, update SoundPad interface (lines 79-93)
export interface SoundPad {
  id: string;
  sound: SoundFile | null;
  color: string;
  hotkey?: string;
  isPlaying: boolean;
  /** Volume level (0.0-2.0, default 1.0 = 100%) */
  volume: number;
  /** Playback speed (0.5-2.0, default 1.0 = normal) */
  speed: number;
  /** User-defined custom name (optional, fallback to sound.name) */
  customName?: string;
  /** Custom image for the pad */
  image?: PadImage;
  /** Folder IDs this sound belongs to (empty = only in "All") */
  folderIds: string[];
}
```

**Step 2: Verify TypeScript compilation**

Run: `npm run build 2>&1 | head -50`
Expected: Compilation errors about missing `folderIds` property in various places

---

## Task 2: Update SoundboardService - Initial Pads and Migration

**Files:**
- Modify: `src/app/core/services/soundboard.service.ts`

**Step 1: Update createInitialPads to include folderIds**

Find `createInitialPads` method (around line 97-107) and add `folderIds: []`:

```typescript
private createInitialPads(count: number): SoundPad[] {
  return Array.from({ length: count }, (_, i) => ({
    id: `pad-${i}`,
    sound: null,
    color: PAD_COLORS[i % PAD_COLORS.length],
    isPlaying: false,
    volume: 1.0,
    speed: 1.0,
    image: undefined,
    folderIds: []
  }));
}
```

**Step 2: Update loadState migration for backwards compatibility**

Find `loadState` method and update the pad restoration (around line 117-120):

```typescript
let restoredPads: SoundPad[] = saved.map(p => ({
  ...p,
  isPlaying: false,
  volume: p.volume ?? 1.0,
  speed: p.speed ?? 1.0,
  folderIds: p.folderIds ?? []  // Migration: add empty array if missing
}));
```

**Step 3: Update reorganizePads to preserve folderIds**

Find `reorganizePads` method (around line 262-309) and add `folderIds` to the data mapping:

```typescript
const filledPadData = pads
  .filter(p => p.sound !== null)
  .map(p => ({
    sound: p.sound!,
    image: p.image,
    volume: p.volume,
    speed: p.speed,
    customName: p.customName,
    hotkey: p.hotkey,
    folderIds: p.folderIds  // ADD THIS LINE
  }))
  .sort((a, b) => a.sound.name.toLowerCase().localeCompare(b.sound.name.toLowerCase()));
```

And in the pad reassignment (around line 284-293):

```typescript
return {
  ...pad,
  sound: data.sound,
  image: data.image,
  volume: data.volume,
  speed: data.speed,
  customName: data.customName,
  hotkey: data.hotkey,
  folderIds: data.folderIds,  // ADD THIS LINE
  isPlaying: false
};
```

And in the empty pad reset (around line 296-305):

```typescript
return {
  ...pad,
  sound: null,
  image: undefined,
  volume: 1.0,
  speed: 1.0,
  customName: undefined,
  hotkey: undefined,
  folderIds: [],  // ADD THIS LINE
  isPlaying: false
};
```

**Step 4: Verify build passes**

Run: `npm run build 2>&1 | tail -20`
Expected: Build succeeds

**Step 5: Commit**

```bash
git add src/app/core/models/audio-device.model.ts src/app/core/services/soundboard.service.ts
git commit -m "feat(folders): add folderIds field to SoundPad model

- Add folderIds: string[] to SoundPad interface
- Update createInitialPads with empty folderIds
- Add migration for backwards compatibility
- Preserve folderIds in reorganizePads

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Update Folder Initialization with "All" Folder

**Files:**
- Modify: `src/app/core/services/soundboard.service.ts:45-46`

**Step 1: Change default folder from "Default" to "All"**

Update the folder initialization (line 45-46):

```typescript
// Folder state
private _folders = signal<Folder[]>([{ id: 'all', name: 'Tous', createdAt: Date.now() }]);
private _activeFolderId = signal<string>('all');
```

**Step 2: Verify build passes**

Run: `npm run build 2>&1 | tail -10`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/app/core/services/soundboard.service.ts
git commit -m "feat(folders): initialize with 'All' folder instead of 'Default'

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Add Folder CRUD Methods to SoundboardService

**Files:**
- Modify: `src/app/core/services/soundboard.service.ts`

**Step 1: Add createFolder method**

Add after `setActiveFolder` method (around line 732):

```typescript
/**
 * Create a new folder
 */
createFolder(name: string): void {
  const trimmedName = name.trim();
  if (!trimmedName) return;

  // Check for duplicate names
  if (this._folders().some(f => f.name.toLowerCase() === trimmedName.toLowerCase())) {
    return;
  }

  const id = `folder-${Date.now()}`;
  const newFolder: Folder = { id, name: trimmedName, createdAt: Date.now() };
  this._folders.update(folders => [...folders, newFolder]);
  this.saveFolders();
}

/**
 * Rename a folder
 */
renameFolder(folderId: string, newName: string): void {
  if (folderId === 'all') return; // Protected

  const trimmedName = newName.trim();
  if (!trimmedName) return;

  // Check for duplicate names (excluding current folder)
  if (this._folders().some(f => f.id !== folderId && f.name.toLowerCase() === trimmedName.toLowerCase())) {
    return;
  }

  this._folders.update(folders =>
    folders.map(f => f.id === folderId ? { ...f, name: trimmedName } : f)
  );
  this.saveFolders();
}

/**
 * Delete a folder
 */
deleteFolder(folderId: string): void {
  if (folderId === 'all') return; // Protected

  // Remove this folder from all pads
  this._pads.update(pads => pads.map(pad => ({
    ...pad,
    folderIds: pad.folderIds.filter(id => id !== folderId)
  })));

  // Delete the folder
  this._folders.update(folders => folders.filter(f => f.id !== folderId));

  // Return to "All" if we were in this folder
  if (this._activeFolderId() === folderId) {
    this._activeFolderId.set('all');
  }

  this.saveFolders();
  this.saveState();
}
```

**Step 2: Add togglePadFolder method**

```typescript
/**
 * Toggle a folder assignment for a pad
 */
togglePadFolder(padId: string, folderId: string): void {
  if (folderId === 'all') return; // Can't toggle "All"

  this._pads.update(pads => pads.map(pad => {
    if (pad.id !== padId) return pad;

    const hasFolder = pad.folderIds.includes(folderId);
    return {
      ...pad,
      folderIds: hasFolder
        ? pad.folderIds.filter(id => id !== folderId)
        : [...pad.folderIds, folderId]
    };
  }));
  this.saveState();
}

/**
 * Add a pad to a folder (for drag & drop)
 */
addPadToFolder(padId: string, folderId: string): void {
  if (folderId === 'all') return;

  this._pads.update(pads => pads.map(pad => {
    if (pad.id !== padId || pad.folderIds.includes(folderId)) return pad;
    return { ...pad, folderIds: [...pad.folderIds, folderId] };
  }));
  this.saveState();
}
```

**Step 3: Add saveFolders placeholder method**

```typescript
/**
 * Save folders to persistent storage
 */
private async saveFolders(): Promise<void> {
  try {
    await this.tauri.saveFolders(this._folders());
  } catch (err) {
    console.error('Failed to save folders:', err);
  }
}
```

**Step 4: Verify build (will fail on tauri.saveFolders)**

Run: `npm run build 2>&1 | grep -i error`
Expected: Error about `saveFolders` not existing on TauriService

**Step 5: Commit partial progress**

```bash
git add src/app/core/services/soundboard.service.ts
git commit -m "feat(folders): add folder CRUD and pad-folder toggle methods

- createFolder, renameFolder, deleteFolder
- togglePadFolder, addPadToFolder
- saveFolders placeholder (TauriService method pending)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Add Folder Persistence to TauriService

**Files:**
- Modify: `src/app/core/services/tauri.service.ts`
- Modify: `src/app/core/models/index.ts` (if Folder not exported)

**Step 1: Add saveFolders and loadFolders methods to TauriService**

Find the soundboard section (around line 487-500) and add after `loadSoundboardState`:

```typescript
/**
 * Save folders to persistent storage
 */
async saveFolders(folders: Folder[]): Promise<void> {
  if (this.demoService.isDemoMode) return;
  await invoke('save_folders', { folders });
}

/**
 * Load folders from persistent storage
 */
async loadFolders(): Promise<Folder[] | null> {
  if (this.demoService.isDemoMode) {
    return [{ id: 'all', name: 'Tous', createdAt: Date.now() }];
  }
  return invoke<Folder[] | null>('load_folders');
}
```

**Step 2: Add Folder import if needed**

At the top of `tauri.service.ts`, ensure Folder is imported:

```typescript
import {
  AudioDevice,
  MixerChannel,
  MixerConfig,
  AppSettings,
  ApiResponse,
  SoundFile,
  Folder  // ADD THIS
} from '../models';
```

**Step 3: Verify Folder is exported from models/index.ts**

Check `src/app/core/models/index.ts` includes:
```typescript
export * from './audio-device.model';
```

**Step 4: Verify build (will fail on Rust commands)**

Run: `npm run build 2>&1 | tail -10`
Expected: Build succeeds (Rust commands checked at runtime)

**Step 5: Commit**

```bash
git add src/app/core/services/tauri.service.ts
git commit -m "feat(folders): add folder persistence methods to TauriService

- saveFolders and loadFolders methods
- Demo mode support

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Add Rust Backend Commands for Folder Persistence

**Files:**
- Modify: `src-tauri/src/application/commands.rs`
- Modify: `src-tauri/src/lib.rs` (register commands)

**Step 1: Add folder persistence commands in commands.rs**

Find the soundboard persistence section (around line 1095-1116) and add after `load_soundboard`:

```rust
const FOLDERS_KEY: &str = "folders";

/// Save folders to persistent storage
#[tauri::command]
pub async fn save_folders(app: tauri::AppHandle, folders: serde_json::Value) -> Result<(), String> {
    let store = app.store(SOUNDBOARD_STORE).map_err(|e| e.to_string())?;
    store.set(FOLDERS_KEY, folders);
    store.save().map_err(|e| e.to_string())?;
    tracing::debug!("Folders saved");
    Ok(())
}

/// Load folders from persistent storage
#[tauri::command]
pub async fn load_folders(app: tauri::AppHandle) -> Result<Option<serde_json::Value>, String> {
    let store = app.store(SOUNDBOARD_STORE).map_err(|e| e.to_string())?;
    #[allow(clippy::map_clone)]
    let folders = store.get(FOLDERS_KEY).map(|v| v.clone());
    tracing::debug!("Folders loaded: {:?}", folders.is_some());
    Ok(folders)
}
```

**Step 2: Register commands in lib.rs**

Find the `invoke_handler` in `src-tauri/src/lib.rs` and add the new commands:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    application::commands::save_folders,
    application::commands::load_folders,
])
```

**Step 3: Verify Rust compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -20`
Expected: No errors

**Step 4: Run Rust tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10`
Expected: All tests pass

**Step 5: Commit**

```bash
git add src-tauri/src/application/commands.rs src-tauri/src/lib.rs
git commit -m "feat(folders): add Rust backend commands for folder persistence

- save_folders and load_folders commands
- Uses same soundboard.json store with 'folders' key

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 7: Load Folders on Startup

**Files:**
- Modify: `src/app/core/services/soundboard.service.ts`

**Step 1: Update loadState to also load folders**

Find `loadState` method and add folder loading at the beginning:

```typescript
private async loadState(): Promise<void> {
  try {
    // Load folders first
    const savedFolders = await this.tauri.loadFolders();
    if (savedFolders && savedFolders.length > 0) {
      // Ensure "All" folder exists and is first
      const hasAll = savedFolders.some(f => f.id === 'all');
      if (!hasAll) {
        savedFolders.unshift({ id: 'all', name: 'Tous', createdAt: 0 });
      }
      this._folders.set(savedFolders);
    }

    // Load pads (existing code)
    const saved = await this.tauri.loadSoundboardState();
    // ... rest of existing loadState code
```

**Step 2: Verify build passes**

Run: `npm run build 2>&1 | tail -10`
Expected: Build succeeds

**Step 3: Run Angular tests**

Run: `npm test -- --no-watch --browsers=ChromeHeadless 2>&1 | tail -20`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/app/core/services/soundboard.service.ts
git commit -m "feat(folders): load folders from persistent storage on startup

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 8: Add Filtered Pads Signal

**Files:**
- Modify: `src/app/core/services/soundboard.service.ts`

**Step 1: Add filteredPads computed signal**

Find the computed signals section (around line 60-62) and add:

```typescript
// Computed
readonly activePads = computed(() => this._pads().filter(p => p.sound !== null));
readonly playingCount = computed(() => this._pads().filter(p => p.isPlaying).length);

/**
 * Pads filtered by active folder.
 * Returns all pads for "All" folder, or only pads containing the active folder.
 */
readonly filteredPads = computed(() => {
  const activeFolderId = this._activeFolderId();
  const allPads = this._pads();

  if (activeFolderId === 'all') {
    return allPads;
  }

  // Get pads that belong to this folder
  const folderPads = allPads.filter(pad =>
    pad.sound !== null && pad.folderIds.includes(activeFolderId)
  );

  // Create a virtual pad grid with filtered sounds sorted alphabetically
  const sortedPads = [...folderPads].sort((a, b) =>
    (a.sound?.name || '').toLowerCase().localeCompare((b.sound?.name || '').toLowerCase())
  );

  // Map to new pad positions for display
  const minPads = Math.max(12, Math.ceil(sortedPads.length / 4) * 4 + 4);
  const result: SoundPad[] = [];

  for (let i = 0; i < minPads; i++) {
    if (i < sortedPads.length) {
      result.push({ ...sortedPads[i], id: `pad-${i}` });
    } else {
      result.push({
        id: `pad-${i}`,
        sound: null,
        color: PAD_COLORS[i % PAD_COLORS.length],
        isPlaying: false,
        volume: 1.0,
        speed: 1.0,
        folderIds: []
      });
    }
  }

  return result;
});
```

**Step 2: Verify build passes**

Run: `npm run build 2>&1 | tail -10`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/app/core/services/soundboard.service.ts
git commit -m "feat(folders): add filteredPads computed signal for folder filtering

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 9: Update Soundboard Component to Use filteredPads

**Files:**
- Modify: `src/app/features/soundboard/soundboard.component.ts`

**Step 1: Read current soundboard.component.ts**

First, examine the current implementation to understand the structure.

**Step 2: Update template to use filteredPads**

Change `soundboard.pads()` to `soundboard.filteredPads()` in the template where pads are rendered.

**Step 3: Verify build passes**

Run: `npm run build 2>&1 | tail -10`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add src/app/features/soundboard/soundboard.component.ts
git commit -m "feat(folders): use filteredPads in soundboard component

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 10: Update Sidebar UI - Enable New Folder Button

**Files:**
- Modify: `src/app/features/mixer/mixer.component.ts`

**Step 1: Add state for new folder popup**

Add to the component class:

```typescript
showNewFolderPopup = signal(false);
newFolderName = signal('');
editingFolderId = signal<string | null>(null);
editingFolderName = signal('');
```

**Step 2: Enable the "New Folder" button**

Replace the disabled button (lines 42-49) with:

```html
<!-- New folder button -->
<button
  class="w-full px-4 py-2.5 text-left text-sm text-text-secondary hover:bg-surface-hover hover:text-text-primary border-l-2 border-transparent transition-colors flex items-center gap-2"
  (click)="showNewFolderPopup.set(true)"
>
  <span>+</span>
  New Folder
</button>
```

**Step 3: Add new folder popup template**

Add after the sidebar closing tag:

```html
<!-- New Folder Popup -->
@if (showNewFolderPopup()) {
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" (click)="showNewFolderPopup.set(false)">
    <div class="bg-surface border border-border rounded-lg p-4 w-80 shadow-xl" (click)="$event.stopPropagation()">
      <h3 class="text-sm font-semibold text-text-primary mb-3">New Folder</h3>
      <input
        type="text"
        [value]="newFolderName()"
        (input)="newFolderName.set($any($event.target).value)"
        (keydown.enter)="createFolder()"
        (keydown.escape)="showNewFolderPopup.set(false)"
        placeholder="Folder name"
        class="w-full px-3 py-2 text-sm bg-surface-hover border border-border rounded text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent"
        autofocus
      >
      <div class="flex justify-end gap-2 mt-4">
        <button
          class="px-3 py-1.5 text-sm text-text-secondary hover:text-text-primary transition-colors"
          (click)="showNewFolderPopup.set(false)"
        >
          Cancel
        </button>
        <button
          class="px-3 py-1.5 text-sm bg-accent hover:bg-accent/80 text-white rounded transition-colors"
          [disabled]="!newFolderName().trim()"
          (click)="createFolder()"
        >
          Create
        </button>
      </div>
    </div>
  </div>
}
```

**Step 4: Add createFolder method**

```typescript
createFolder(): void {
  const name = this.newFolderName().trim();
  if (name) {
    this.soundboard.createFolder(name);
    this.newFolderName.set('');
    this.showNewFolderPopup.set(false);
  }
}
```

**Step 5: Verify build passes**

Run: `npm run build 2>&1 | tail -10`
Expected: Build succeeds

**Step 6: Commit**

```bash
git add src/app/features/mixer/mixer.component.ts
git commit -m "feat(folders): enable new folder creation in sidebar

- New Folder button opens popup
- Input with Enter/Escape support
- Creates folder via SoundboardService

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 11: Add Context Menu for Folder Rename/Delete

**Files:**
- Modify: `src/app/features/mixer/mixer.component.ts`

**Step 1: Add context menu state**

```typescript
contextMenuFolder = signal<Folder | null>(null);
contextMenuPosition = signal({ x: 0, y: 0 });
```

**Step 2: Update folder button to handle right-click**

```html
<button
  class="w-full px-4 py-2.5 text-left text-sm transition-colors flex items-center gap-2"
  [class]="folder.id === soundboard.activeFolderId()
    ? 'bg-surface-hover text-text-primary border-l-2 border-accent'
    : 'text-text-secondary hover:bg-surface-hover hover:text-text-primary border-l-2 border-transparent'"
  (click)="soundboard.setActiveFolder(folder.id)"
  (contextmenu)="onFolderContextMenu($event, folder)"
>
  <span>{{ folder.id === soundboard.activeFolderId() ? '&#9654;' : '&#128193;' }}</span>
  {{ folder.name }}
</button>
```

**Step 3: Add context menu template**

```html
<!-- Folder Context Menu -->
@if (contextMenuFolder()) {
  <div
    class="fixed z-50"
    [style.left.px]="contextMenuPosition().x"
    [style.top.px]="contextMenuPosition().y"
  >
    <div class="bg-surface border border-border rounded-lg shadow-xl py-1 min-w-32">
      <button
        class="w-full px-4 py-2 text-left text-sm text-text-secondary hover:bg-surface-hover hover:text-text-primary transition-colors"
        (click)="startRenamingFolder()"
      >
        Rename
      </button>
      <button
        class="w-full px-4 py-2 text-left text-sm text-status-error hover:bg-surface-hover transition-colors"
        (click)="deleteFolder()"
      >
        Delete
      </button>
    </div>
  </div>
  <!-- Backdrop to close menu -->
  <div class="fixed inset-0 z-40" (click)="contextMenuFolder.set(null)"></div>
}
```

**Step 4: Add context menu methods**

```typescript
onFolderContextMenu(event: MouseEvent, folder: Folder): void {
  if (folder.id === 'all') return; // Can't modify "All" folder
  event.preventDefault();
  this.contextMenuFolder.set(folder);
  this.contextMenuPosition.set({ x: event.clientX, y: event.clientY });
}

startRenamingFolder(): void {
  const folder = this.contextMenuFolder();
  if (folder) {
    this.editingFolderId.set(folder.id);
    this.editingFolderName.set(folder.name);
    this.contextMenuFolder.set(null);
  }
}

deleteFolder(): void {
  const folder = this.contextMenuFolder();
  if (folder && confirm(`Delete folder "${folder.name}"? Sounds will not be deleted.`)) {
    this.soundboard.deleteFolder(folder.id);
  }
  this.contextMenuFolder.set(null);
}
```

**Step 5: Add rename popup template**

```html
<!-- Rename Folder Popup -->
@if (editingFolderId()) {
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" (click)="editingFolderId.set(null)">
    <div class="bg-surface border border-border rounded-lg p-4 w-80 shadow-xl" (click)="$event.stopPropagation()">
      <h3 class="text-sm font-semibold text-text-primary mb-3">Rename Folder</h3>
      <input
        type="text"
        [value]="editingFolderName()"
        (input)="editingFolderName.set($any($event.target).value)"
        (keydown.enter)="confirmRenameFolder()"
        (keydown.escape)="editingFolderId.set(null)"
        class="w-full px-3 py-2 text-sm bg-surface-hover border border-border rounded text-text-primary focus:outline-none focus:border-accent"
        autofocus
      >
      <div class="flex justify-end gap-2 mt-4">
        <button
          class="px-3 py-1.5 text-sm text-text-secondary hover:text-text-primary transition-colors"
          (click)="editingFolderId.set(null)"
        >
          Cancel
        </button>
        <button
          class="px-3 py-1.5 text-sm bg-accent hover:bg-accent/80 text-white rounded transition-colors"
          [disabled]="!editingFolderName().trim()"
          (click)="confirmRenameFolder()"
        >
          Rename
        </button>
      </div>
    </div>
  </div>
}
```

**Step 6: Add confirmRenameFolder method**

```typescript
confirmRenameFolder(): void {
  const folderId = this.editingFolderId();
  const newName = this.editingFolderName().trim();
  if (folderId && newName) {
    this.soundboard.renameFolder(folderId, newName);
    this.editingFolderId.set(null);
  }
}
```

**Step 7: Add Folder import**

```typescript
import { Folder } from '../../core/models';
```

**Step 8: Verify build passes**

Run: `npm run build 2>&1 | tail -10`
Expected: Build succeeds

**Step 9: Commit**

```bash
git add src/app/features/mixer/mixer.component.ts
git commit -m "feat(folders): add context menu for folder rename and delete

- Right-click on folder shows context menu
- Rename popup with input field
- Delete with confirmation
- 'All' folder is protected

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 12: Add Folder Checkboxes to Pad Settings Popup

**Files:**
- Modify: `src/app/features/soundboard/sound-pad/sound-pad.component.ts`

**Step 1: Add folder toggle output**

Add to outputs:

```typescript
@Output() folderToggle = new EventEmitter<string>();
```

**Step 2: Add folders section to popup template**

Find the "Reset button" section (around line 337) and add before it:

```html
<!-- Folders -->
@if (folders().length > 1) {
  <div class="mb-4 pt-4 border-t border-border">
    <div class="flex justify-between items-center mb-2 text-xs">
      <span class="text-text-secondary">Folders</span>
    </div>
    <div class="space-y-1">
      @for (folder of folders(); track folder.id) {
        @if (folder.id !== 'all') {
          <label class="flex items-center gap-2 py-1 cursor-pointer hover:bg-surface-hover rounded px-2 -mx-2">
            <input
              type="checkbox"
              [checked]="pad.folderIds.includes(folder.id)"
              (change)="onFolderToggle(folder.id)"
              class="w-4 h-4 rounded border-border bg-surface-hover text-accent focus:ring-accent focus:ring-offset-0"
            >
            <span class="text-sm text-text-primary">{{ folder.name }}</span>
          </label>
        }
      }
    </div>
  </div>
}
```

**Step 3: Add folders getter**

```typescript
folders = inject(SoundboardService).folders;
```

**Step 4: Add onFolderToggle method**

```typescript
onFolderToggle(folderId: string): void {
  this.folderToggle.emit(folderId);
}
```

**Step 5: Verify build passes**

Run: `npm run build 2>&1 | tail -10`
Expected: Build succeeds

**Step 6: Commit**

```bash
git add src/app/features/soundboard/sound-pad/sound-pad.component.ts
git commit -m "feat(folders): add folder checkboxes to pad settings popup

- Shows all folders except 'All'
- Checkbox reflects pad.folderIds
- Emits folderToggle event

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 13: Wire Up Folder Toggle in Soundboard Component

**Files:**
- Modify: `src/app/features/soundboard/soundboard.component.ts`

**Step 1: Add folderToggle handler to app-sound-pad**

Find where `app-sound-pad` is used and add:

```html
(folderToggle)="soundboard.togglePadFolder(pad.id, $event)"
```

**Step 2: Verify build passes**

Run: `npm run build 2>&1 | tail -10`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/app/features/soundboard/soundboard.component.ts
git commit -m "feat(folders): wire up folder toggle in soundboard component

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 14: Add Drag & Drop to Sidebar Folders

**Files:**
- Modify: `src/app/features/mixer/mixer.component.ts`
- Modify: `src/app/features/soundboard/sound-pad/sound-pad.component.ts`

**Step 1: Make pad draggable**

In `sound-pad.component.ts`, add to the main div:

```html
[draggable]="pad.sound !== null"
(dragstart)="onDragStart($event)"
(dragend)="onDragEnd($event)"
```

Add methods:

```typescript
onDragStart(event: DragEvent): void {
  if (!this.pad.sound) return;
  event.dataTransfer?.setData('text/plain', this.pad.id);
  event.dataTransfer!.effectAllowed = 'copy';
}

onDragEnd(event: DragEvent): void {
  // Cleanup if needed
}
```

**Step 2: Make sidebar folders drop targets**

In `mixer.component.ts`, update folder buttons:

```html
<button
  class="w-full px-4 py-2.5 text-left text-sm transition-colors flex items-center gap-2"
  [class]="getFolderClasses(folder)"
  (click)="soundboard.setActiveFolder(folder.id)"
  (contextmenu)="onFolderContextMenu($event, folder)"
  (dragover)="onFolderDragOver($event, folder)"
  (dragleave)="onFolderDragLeave($event)"
  (drop)="onFolderDrop($event, folder)"
>
```

**Step 3: Add drag state and methods**

```typescript
dragOverFolderId = signal<string | null>(null);

getFolderClasses(folder: Folder): string {
  const isActive = folder.id === this.soundboard.activeFolderId();
  const isDragOver = folder.id === this.dragOverFolderId() && folder.id !== 'all';

  let classes = '';
  if (isActive) {
    classes = 'bg-surface-hover text-text-primary border-l-2 border-accent';
  } else if (isDragOver) {
    classes = 'bg-accent/20 text-text-primary border-l-2 border-accent';
  } else {
    classes = 'text-text-secondary hover:bg-surface-hover hover:text-text-primary border-l-2 border-transparent';
  }
  return classes;
}

onFolderDragOver(event: DragEvent, folder: Folder): void {
  if (folder.id === 'all') return;
  event.preventDefault();
  event.dataTransfer!.dropEffect = 'copy';
  this.dragOverFolderId.set(folder.id);
}

onFolderDragLeave(event: DragEvent): void {
  this.dragOverFolderId.set(null);
}

onFolderDrop(event: DragEvent, folder: Folder): void {
  event.preventDefault();
  this.dragOverFolderId.set(null);

  if (folder.id === 'all') return;

  const padId = event.dataTransfer?.getData('text/plain');
  if (padId) {
    this.soundboard.addPadToFolder(padId, folder.id);
  }
}
```

**Step 4: Verify build passes**

Run: `npm run build 2>&1 | tail -10`
Expected: Build succeeds

**Step 5: Run all tests**

Run: `npm test -- --no-watch --browsers=ChromeHeadless && cargo test --manifest-path src-tauri/Cargo.toml`
Expected: All tests pass

**Step 6: Commit**

```bash
git add src/app/features/mixer/mixer.component.ts src/app/features/soundboard/sound-pad/sound-pad.component.ts
git commit -m "feat(folders): add drag & drop support for adding pads to folders

- Pads are draggable when they have a sound
- Folders highlight on drag over
- Drop adds pad to folder
- 'All' folder ignores drops

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 15: Final Testing and Cleanup

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

- [ ] Create a new folder
- [ ] Rename the folder
- [ ] Delete the folder
- [ ] Add a sound to a folder via checkbox
- [ ] Remove a sound from a folder via checkbox
- [ ] Drag a pad to a folder
- [ ] Switch between folders and verify filtering
- [ ] Verify "All" folder shows all sounds
- [ ] Verify persistence (restart app)
- [ ] Verify shortcuts work regardless of active folder

**Step 4: Update ROADMAP.md**

Mark the task as complete:

```markdown
- [x] **Sound Organization**
  - Folders/categories to organize sounds
  - Drag & drop sounds into folders
  - Folder navigation in UI
```

**Step 5: Final commit**

```bash
git add ROADMAP.md
git commit -m "docs: mark folder management as complete in ROADMAP

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Add folderIds to SoundPad model | audio-device.model.ts |
| 2 | Update service for migration | soundboard.service.ts |
| 3 | Initialize with "All" folder | soundboard.service.ts |
| 4 | Add folder CRUD methods | soundboard.service.ts |
| 5 | Add TauriService persistence | tauri.service.ts |
| 6 | Add Rust backend commands | commands.rs, lib.rs |
| 7 | Load folders on startup | soundboard.service.ts |
| 8 | Add filteredPads signal | soundboard.service.ts |
| 9 | Use filteredPads in UI | soundboard.component.ts |
| 10 | Enable new folder button | mixer.component.ts |
| 11 | Add folder context menu | mixer.component.ts |
| 12 | Add folder checkboxes | sound-pad.component.ts |
| 13 | Wire up folder toggle | soundboard.component.ts |
| 14 | Add drag & drop | mixer + sound-pad |
| 15 | Final testing | All files |
