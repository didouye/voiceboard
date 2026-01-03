# Bulk Import Design

> **Date:** 2026-01-03
> **Status:** Validated

## Overview

Add bulk import functionality to the soundboard, allowing users to import multiple audio files at once via a button or drag & drop. The soundboard becomes dynamic, automatically adding rows when needed.

## Dynamic Soundboard Behavior

### Auto-add pads

- The soundboard must always have **at least 1 empty pad**
- When the user fills the last empty pad (single or bulk import), a new row of 4 pads is added automatically
- Minimum: 12 pads (3 rows) - the soundboard never goes below this

### Auto-remove rows

- When the user removes a sound, check if the last row is entirely empty
- If yes AND there are more than 12 pads AND at least 1 empty pad will remain after removal → remove the row
- Repeat until the last row contains at least one sound or 12 pads is reached

### Remove "Add pads" button

- The existing "Add pads" button becomes obsolete and will be removed
- Pad addition is now fully automatic

## Bulk Import

### "Import Multiple" Button

**Location:** Centered below the pad grid.

**Behavior on click:**
1. Open native file dialog with `multiple: true`
2. Filter: `extensions: ['mp3', 'ogg', 'wav', 'flac']`
3. User selects 1 or more files
4. Files are sorted alphabetically by filename
5. System calculates how many empty pads are available
6. If needed, add rows of 4 pads to accommodate all files + keep 1 empty pad
7. Assign files to empty pads in order (left→right, top→bottom)
8. Show loading state during processing
9. On partial errors: show notification listing failed files

### Drag & Drop

**Drop zone:** Entire soundboard surface (pad grid).

**Visual feedback on hover:**
- Semi-transparent overlay on the soundboard
- Centered text: "Drop to import X files" (X = number of detected files)

**Behavior on drop:** Same as button (alphabetical sort, add rows if needed, etc.)

## Technical Implementation

### Frontend (Angular)

**SoundboardService modifications:**
- New method `importMultipleSounds()`: opens multi-file dialog, orchestrates import
- New method `ensureEmptyPadAvailable()`: checks and adds a row if needed
- New method `cleanupEmptyRows()`: removes excess empty rows
- Modify existing `importSound()` to call `ensureEmptyPadAvailable()` after import

**SoundboardComponent modifications:**
- Add "Import Multiple" button below the grid
- Implement drag & drop events (`dragover`, `dragleave`, `drop`)
- Local state for drop overlay (`isDragging: boolean`)

**Tauri commands:**
- New command `load_multiple_sound_files(paths: Vec<String>)` → `Vec<Result<SoundFileDto, String>>`
- Process all files in parallel for better performance
- Return success or error message for each file

### Backend (Rust)

**New command in `commands.rs`:**
- `load_multiple_sound_files`: iterates over paths, calls existing `load_sound_file` logic, collects results
- Use `rayon` or async to parallelize audio file decoding

## UI/UX

### Loading states

**During bulk import:**
- Disable "Import Multiple" button and pad interactions
- Show progress indicator (spinner or bar)
- Text: "Importing X/Y files..."

### Error notifications

**Partial error message format:**
```
Import complete: X files imported
Failures (Y):
- filename_1.mp3: unsupported format
- filename_2.wav: corrupted file
```

**Display:** Toast/notification that stays visible for a few seconds or until manually closed.

### "Import Multiple" button

**Style:** Secondary button (not too prominent), folder icon + "Import Multiple" text

**Location:** Centered below the grid with small spacing
