# Folder Management Design

> **Date:** 2026-01-10
> **Status:** Approved

## Overview

Implement a folder/tagging system to organize sounds. Folders work as categories - a sound can belong to multiple folders simultaneously.

## Key Decisions

| Aspect | Decision |
|--------|----------|
| Model | Folders as tags (a sound can be in multiple folders) |
| Position | Automatic, alphabetical sorting |
| Shortcuts | Global, bound to sound (not folder) |
| Special folder | "Tous" (All) shows everything, cannot be deleted |
| Assignment | Checkboxes in pad settings popup + drag & drop |
| Folder management | "+" button + context menu (rename/delete) |
| Persistence | `folders` and `folderIds` in soundboard.json |

## Data Model

### SoundPad (modified)

```typescript
export interface SoundPad {
  id: string;
  sound: SoundFile | null;
  color: string;
  hotkey?: string;
  isPlaying: boolean;
  volume: number;
  speed: number;
  customName?: string;
  image?: PadImage;
  folderIds: string[];  // NEW - list of folders containing this sound
}
```

### Special "All" Folder

- ID: `'all'`
- Display name: "Tous"
- Created automatically, cannot be deleted or renamed
- A sound with `folderIds: []` appears only in "All"
- A sound with `folderIds: ['music', 'favorites']` appears in "All", "Music" AND "Favorites"

## Display Logic

### Filtering pads by folder

```typescript
readonly filteredPads = computed(() => {
  const activeFolderId = this._activeFolderId();
  const allPads = this._pads();

  if (activeFolderId === 'all') {
    return allPads;
  }

  return allPads.filter(pad =>
    pad.sound !== null && pad.folderIds.includes(activeFolderId)
  );
});
```

### Reorganization for display

When displaying a folder other than "All":
1. Filter sounds belonging to this folder
2. Sort alphabetically
3. Map to a new pad grid (pad-0, pad-1, pad-2...)

The displayed grid is different from the storage grid. `_pads` is the source of truth, `filteredPads` generates a filtered view.

### Playing sounds

Global shortcuts always use `_pads` (source of truth), not `filteredPads`. Shortcuts work regardless of active folder.

## Folder CRUD Operations

### Create folder

```typescript
createFolder(name: string): void {
  const id = `folder-${Date.now()}`;
  const newFolder: Folder = { id, name, createdAt: Date.now() };
  this._folders.update(folders => [...folders, newFolder]);
  this.saveFolders();
}
```

- "+" button in sidebar opens popup with text input
- Validation: non-empty name, no duplicates

### Rename folder

```typescript
renameFolder(folderId: string, newName: string): void {
  if (folderId === 'all') return; // Protected
  this._folders.update(folders =>
    folders.map(f => f.id === folderId ? { ...f, name: newName } : f)
  );
  this.saveFolders();
}
```

### Delete folder

```typescript
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

- Confirmation before deletion
- Sounds are not deleted, just removed from the folder

## Sound-to-Folder Assignment

### Pad settings popup

Add a "Folders" section at the bottom of the existing popup:

```
┌─────────────────────────────────┐
│  Volume      [━━━━━━━●━━] 100%  │
│  Speed       [━━━●━━━━━━] 1.0x  │
│  Shortcut    [Ctrl+1] [✕]       │
│  ─────────────────────────────  │
│  Folders                        │
│  ☑ Music                        │
│  ☐ Effects                      │
│  ☑ Favorites                    │
└─────────────────────────────────┘
```

- Checkbox list for each folder (except "All")
- Check = add to folder
- Uncheck = remove from folder
- Immediate update of `folderIds`

### Toggle method

```typescript
togglePadFolder(padId: string, folderId: string): void {
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
```

### Drag & drop to sidebar

- Drag a pad to a folder in the sidebar
- Adds the sound to that folder (without removing from others)
- Visual feedback: target folder highlights on hover

## Sidebar UI

### Structure

```
┌─────────────────────┐
│ 📁 Folders     [+]  │  ← "+" button to create
├─────────────────────┤
│ ▶ Tous              │  ← Always first, not modifiable
│   Music             │  ← Right-click → context menu
│   Effects           │
│   Favorites         │
├─────────────────────┤
│ ⚙ Settings          │
└─────────────────────┘
```

### "+" button (create folder)

- Click → simple popup with text input
- Placeholder: "Folder name"
- Buttons: "Cancel" / "Create"
- Validation: non-empty name

### Context menu on folder

Right-click on a folder (except "All"):
- **Rename** → inline input or small popup
- **Delete** → confirmation "Delete folder X? Sounds will not be deleted."

### Drag & drop target

- Drop zone on each folder
- On pad hover: accent border + slight scale up
- Visual indicator that drop is possible

## Persistence

### soundboard.json structure

```json
{
  "folders": [
    { "id": "all", "name": "Tous", "createdAt": 1704067200000 },
    { "id": "folder-1704067300000", "name": "Music", "createdAt": 1704067300000 },
    { "id": "folder-1704067400000", "name": "Effects", "createdAt": 1704067400000 }
  ],
  "pads": [
    {
      "id": "pad-0",
      "sound": { "name": "explosion.mp3", "path": "..." },
      "folderIds": ["folder-1704067400000"],
      "hotkey": "Ctrl+1",
      "volume": 1.0,
      "speed": 1.0
    }
  ]
}
```

### Data migration

On load, if a pad has no `folderIds`:
- Initialize with `folderIds: []`
- Sound will appear in "All" by default

If `folders` doesn't exist:
- Create "All" folder automatically

### Persistence methods

```typescript
private async saveFolders(): Promise<void> {
  await this.tauri.saveFolders(this._folders());
}
```

Add Rust commands `save_folders` and `load_folders` similar to existing pad commands.

## Files to Modify

### Frontend

| File | Changes |
|------|---------|
| `audio-device.model.ts` | Add `folderIds` to `SoundPad` |
| `soundboard.service.ts` | Folder CRUD, filtering, togglePadFolder |
| `tauri.service.ts` | New methods for folder persistence |
| `mixer.component.ts` | Sidebar (+ button, context menu, drop zone) |
| `pad-settings-popup.component.ts` | Folders section with checkboxes |
| `pad.component.ts` | Drag source for drag & drop |

### Backend

| File | Changes |
|------|---------|
| `commands.rs` | `save_folders`, `load_folders` commands |

## Implementation Order

1. Data model changes (`folderIds` in SoundPad, migration)
2. Folder CRUD in SoundboardService
3. Backend persistence commands
4. Filtering logic (`filteredPads`)
5. Sidebar UI (+ button, context menu)
6. Pad settings popup (folder checkboxes)
7. Drag & drop
