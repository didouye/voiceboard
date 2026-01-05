# Pad Images Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Allow users to add custom images to soundboard pads via upload, URL, or Pexels search, with auto-suggestion on import.

**Architecture:** Frontend handles image search via Pexels API (using user-provided API key), backend handles image storage in `~/.voiceboard/images/`. Images are downloaded locally for offline support. Auto-suggestion appears as a toast on single import or as a sequential wizard on bulk import.

**Tech Stack:** Angular 18 (standalone components, signals), Tauri 2, Rust, Pexels API

---

## Task 1: Add PadImage Model and Update SoundPad

**Files:**
- Modify: `src/app/core/models/audio-device.model.ts:67-79`

**Step 1: Add PadImage interface**

Add after line 63 (after SoundFile interface):

```typescript
/**
 * Image attached to a sound pad
 */
export interface PadImage {
  /** Relative path in ~/.voiceboard/images/ */
  localPath: string;
  /** Original URL source (for attribution) */
  originalUrl?: string;
  /** Attribution text (e.g., "Photo by X on Pexels") */
  attribution?: string;
}
```

**Step 2: Update SoundPad interface**

Add `image` field to SoundPad interface (after `customName`):

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
  /** Custom image for the pad */
  image?: PadImage;
}
```

**Step 3: Run tests**

Run: `npm test -- --include='**/soundboard.service.spec.ts'`
Expected: All existing tests pass (no breaking changes)

**Step 4: Commit**

```bash
git add src/app/core/models/audio-device.model.ts
git commit -m "feat(models): add PadImage interface to SoundPad"
```

---

## Task 2: Add Backend Image Commands (Rust)

**Files:**
- Modify: `src-tauri/src/application/commands.rs`
- Modify: `src-tauri/src/lib.rs` (register commands)

**Step 1: Add image command implementations**

Add after line 1078 (after `load_soundboard` function):

```rust
// ============================================================================
// Image Management Commands
// ============================================================================

/// Get the images directory path
#[tauri::command]
pub async fn get_images_dir(app: tauri::AppHandle) -> Result<String, String> {
    use std::path::PathBuf;

    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let images_dir = app_data_dir.join("images");

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&images_dir)
        .map_err(|e| format!("Failed to create images directory: {}", e))?;

    Ok(images_dir.to_string_lossy().to_string())
}

/// Save an image for a pad
/// Returns the relative path to the saved image
#[tauri::command]
pub async fn save_pad_image(
    app: tauri::AppHandle,
    pad_id: String,
    image_data: Vec<u8>,
    extension: String,
) -> Result<String, String> {
    use sha2::{Sha256, Digest};

    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let images_dir = app_data_dir.join("images");
    std::fs::create_dir_all(&images_dir)
        .map_err(|e| format!("Failed to create images directory: {}", e))?;

    // Generate hash of image content (first 8 chars)
    let mut hasher = Sha256::new();
    hasher.update(&image_data);
    let hash = format!("{:x}", hasher.finalize());
    let hash_short = &hash[..8];

    // Clean extension (remove leading dot if present)
    let ext = extension.trim_start_matches('.');

    // Filename: {padId}-{hash8}.{ext}
    let filename = format!("{}-{}.{}", pad_id, hash_short, ext);
    let file_path = images_dir.join(&filename);

    // Delete any existing images for this pad first
    if let Ok(entries) = std::fs::read_dir(&images_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&format!("{}-", pad_id)) && name != filename {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }

    // Write new image
    std::fs::write(&file_path, &image_data)
        .map_err(|e| format!("Failed to save image: {}", e))?;

    tracing::info!("Saved pad image: {}", filename);

    // Return relative path (just filename)
    Ok(filename)
}

/// Delete image for a pad
#[tauri::command]
pub async fn delete_pad_image(
    app: tauri::AppHandle,
    pad_id: String,
) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let images_dir = app_data_dir.join("images");

    // Delete all images for this pad
    if let Ok(entries) = std::fs::read_dir(&images_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&format!("{}-", pad_id)) {
                std::fs::remove_file(entry.path())
                    .map_err(|e| format!("Failed to delete image: {}", e))?;
                tracing::info!("Deleted pad image: {}", name);
            }
        }
    }

    Ok(())
}

/// Clean up orphaned images (not referenced by any pad)
/// Called on app startup
#[tauri::command]
pub async fn cleanup_orphaned_images(
    app: tauri::AppHandle,
) -> Result<u32, String> {
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let images_dir = app_data_dir.join("images");

    if !images_dir.exists() {
        return Ok(0);
    }

    // Load soundboard to get referenced images
    let store = app.store(SOUNDBOARD_STORE).map_err(|e| e.to_string())?;
    let pads_value = store.get(SOUNDBOARD_KEY);

    let mut referenced_images: std::collections::HashSet<String> = std::collections::HashSet::new();

    if let Some(pads) = pads_value {
        if let Some(pads_array) = pads.as_array() {
            for pad in pads_array {
                if let Some(image) = pad.get("image") {
                    if let Some(local_path) = image.get("localPath").and_then(|v| v.as_str()) {
                        referenced_images.insert(local_path.to_string());
                    }
                }
            }
        }
    }

    // Delete orphaned images
    let mut deleted_count = 0u32;
    if let Ok(entries) = std::fs::read_dir(&images_dir) {
        for entry in entries.flatten() {
            let filename = entry.file_name().to_string_lossy().to_string();
            if !referenced_images.contains(&filename) {
                if std::fs::remove_file(entry.path()).is_ok() {
                    tracing::info!("Deleted orphaned image: {}", filename);
                    deleted_count += 1;
                }
            }
        }
    }

    Ok(deleted_count)
}
```

**Step 2: Add sha2 dependency to Cargo.toml**

Add to `src-tauri/Cargo.toml` dependencies:

```toml
sha2 = "0.10"
```

**Step 3: Register commands in lib.rs**

Add the new commands to the invoke_handler in `src-tauri/src/lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    commands::get_images_dir,
    commands::save_pad_image,
    commands::delete_pad_image,
    commands::cleanup_orphaned_images,
])
```

**Step 4: Run tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: All tests pass

**Step 5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/application/commands.rs src-tauri/src/lib.rs
git commit -m "feat(backend): add image management commands"
```

---

## Task 3: Add TauriService Image Methods

**Files:**
- Modify: `src/app/core/services/tauri.service.ts`

**Step 1: Add image methods**

Add after line 490 (after `loadSoundboardState`):

```typescript
// =========================================================================
// Image Management
// =========================================================================

/**
 * Get the images directory path
 */
async getImagesDir(): Promise<string> {
  if (this.demoService.isDemoMode) {
    return '/demo/images';
  }
  return invoke<string>('get_images_dir');
}

/**
 * Save an image for a pad
 * @returns The relative path to the saved image
 */
async savePadImage(padId: string, imageData: Uint8Array, extension: string): Promise<string> {
  if (this.demoService.isDemoMode) {
    return `${padId}-demo.${extension}`;
  }
  return invoke<string>('save_pad_image', {
    padId,
    imageData: Array.from(imageData),
    extension
  });
}

/**
 * Delete image for a pad
 */
async deletePadImage(padId: string): Promise<void> {
  if (this.demoService.isDemoMode) return;
  await invoke('delete_pad_image', { padId });
}

/**
 * Clean up orphaned images
 * @returns Number of deleted images
 */
async cleanupOrphanedImages(): Promise<number> {
  if (this.demoService.isDemoMode) return 0;
  return invoke<number>('cleanup_orphaned_images');
}
```

**Step 2: Run build check**

Run: `npm run build`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/app/core/services/tauri.service.ts
git commit -m "feat(tauri): add image management methods"
```

---

## Task 4: Create ImageSearchService

**Files:**
- Create: `src/app/core/services/image-search.service.ts`
- Modify: `src/app/core/services/index.ts`

**Step 1: Create ImageSearchService**

```typescript
import { Injectable, signal, computed } from '@angular/core';

export interface ImageSearchResult {
  id: string;
  thumbnailUrl: string;
  fullUrl: string;
  attribution: string;
  photographer: string;
}

interface PexelsPhoto {
  id: number;
  photographer: string;
  src: {
    tiny: string;
    small: string;
    medium: string;
  };
}

interface PexelsResponse {
  photos: PexelsPhoto[];
  next_page?: string;
}

@Injectable({
  providedIn: 'root'
})
export class ImageSearchService {
  private _apiKey = signal<string | null>(null);
  private _loading = signal(false);
  private _error = signal<string | null>(null);

  readonly apiKey = this._apiKey.asReadonly();
  readonly loading = this._loading.asReadonly();
  readonly error = this._error.asReadonly();
  readonly hasApiKey = computed(() => !!this._apiKey());

  private readonly PEXELS_API_URL = 'https://api.pexels.com/v1';
  private readonly STORAGE_KEY = 'pexels_api_key';

  constructor() {
    this.loadApiKey();
  }

  /**
   * Load API key from localStorage
   */
  private loadApiKey(): void {
    const key = localStorage.getItem(this.STORAGE_KEY);
    if (key) {
      this._apiKey.set(key);
    }
  }

  /**
   * Set and persist the Pexels API key
   */
  setApiKey(key: string | null): void {
    if (key) {
      localStorage.setItem(this.STORAGE_KEY, key);
      this._apiKey.set(key);
    } else {
      localStorage.removeItem(this.STORAGE_KEY);
      this._apiKey.set(null);
    }
  }

  /**
   * Test if the API key is valid
   */
  async testApiKey(key: string): Promise<boolean> {
    try {
      const response = await fetch(`${this.PEXELS_API_URL}/search?query=test&per_page=1`, {
        headers: { Authorization: key }
      });
      return response.ok;
    } catch {
      return false;
    }
  }

  /**
   * Search for images
   */
  async search(query: string, page: number = 1, perPage: number = 12): Promise<ImageSearchResult[]> {
    const apiKey = this._apiKey();
    if (!apiKey) {
      throw new Error('Pexels API key not configured');
    }

    this._loading.set(true);
    this._error.set(null);

    try {
      const url = `${this.PEXELS_API_URL}/search?query=${encodeURIComponent(query)}&page=${page}&per_page=${perPage}`;
      const response = await fetch(url, {
        headers: { Authorization: apiKey }
      });

      if (!response.ok) {
        if (response.status === 401) {
          throw new Error('Invalid API key');
        }
        if (response.status === 429) {
          throw new Error('Rate limit reached, try again in 1 hour');
        }
        throw new Error(`Search failed: ${response.statusText}`);
      }

      const data: PexelsResponse = await response.json();

      return data.photos.map(photo => ({
        id: photo.id.toString(),
        thumbnailUrl: photo.src.tiny,
        fullUrl: photo.src.medium,
        attribution: `Photo by ${photo.photographer} on Pexels`,
        photographer: photo.photographer
      }));
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Search failed';
      this._error.set(message);
      throw err;
    } finally {
      this._loading.set(false);
    }
  }

  /**
   * Download an image and return as Uint8Array
   */
  async downloadImage(url: string): Promise<{ data: Uint8Array; extension: string }> {
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error('Failed to download image');
    }

    const contentType = response.headers.get('content-type') || '';
    let extension = 'jpg';
    if (contentType.includes('png')) extension = 'png';
    else if (contentType.includes('webp')) extension = 'webp';
    else if (contentType.includes('gif')) extension = 'gif';

    const buffer = await response.arrayBuffer();
    return {
      data: new Uint8Array(buffer),
      extension
    };
  }

  /**
   * Extract search query from filename
   * "funny_airhorn_sound.mp3" -> "funny airhorn sound"
   * Takes first 3 words max
   */
  extractQueryFromFilename(filename: string): string {
    // Remove extension
    const nameWithoutExt = filename.replace(/\.[^/.]+$/, '');
    // Replace separators with spaces
    const words = nameWithoutExt.replace(/[-_]/g, ' ').split(/\s+/);
    // Take first 3 words
    return words.slice(0, 3).join(' ').toLowerCase();
  }
}
```

**Step 2: Export from index.ts**

Add to `src/app/core/services/index.ts`:

```typescript
export * from './image-search.service';
```

**Step 3: Run build check**

Run: `npm run build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add src/app/core/services/image-search.service.ts src/app/core/services/index.ts
git commit -m "feat(services): add ImageSearchService for Pexels API"
```

---

## Task 5: Update SoundboardService for Image Support

**Files:**
- Modify: `src/app/core/services/soundboard.service.ts`

**Step 1: Update SavedPad interface**

Add `image` field to SavedPad interface (line ~22):

```typescript
interface SavedPad {
  id: string;
  sound: SoundFile | null;
  color: string;
  hotkey?: string;
  volume?: number;
  speed?: number;
  customName?: string;
  image?: PadImage; // Add this line
}
```

**Step 2: Add PadImage import**

Update imports at top of file:

```typescript
import { SoundFile, SoundPad, Folder, PadImage } from '../models';
```

**Step 3: Update createInitialPads to include image**

Update the return object in `createInitialPads` (line ~103):

```typescript
private createInitialPads(count: number): SoundPad[] {
  return Array.from({ length: count }, (_, i) => ({
    id: `pad-${i}`,
    sound: null,
    color: PAD_COLORS[i % PAD_COLORS.length],
    isPlaying: false,
    volume: 1.0,
    speed: 1.0,
    image: undefined
  }));
}
```

**Step 4: Update loadState to restore image**

Update the `restoredPads` mapping (line ~115):

```typescript
const restoredPads: SoundPad[] = saved.map(p => ({
  ...p,
  isPlaying: false,
  volume: p.volume ?? 1.0,
  speed: p.speed ?? 1.0,
  customName: p.customName,
  image: p.image
}));
```

**Step 5: Update saveState to include image**

Update the `padsToSave` mapping (line ~141):

```typescript
const padsToSave: SavedPad[] = this._pads().map(p => ({
  id: p.id,
  sound: p.sound,
  color: p.color,
  hotkey: p.hotkey,
  volume: p.volume,
  speed: p.speed,
  customName: p.customName,
  image: p.image
}));
```

**Step 6: Add setPadImage method**

Add after `setPadCustomName` (line ~502):

```typescript
/**
 * Set image for a pad
 */
setPadImage(padId: string, image: PadImage | null): void {
  this._pads.update(pads => pads.map(p =>
    p.id === padId ? { ...p, image: image || undefined } : p
  ));
  this.saveState();
}
```

**Step 7: Update removeSound to clear image**

Update the `removeSound` method to also clear the image:

```typescript
removeSound(padId: string): void {
  this._pads.update(pads => pads.map(pad =>
    pad.id === padId
      ? { ...pad, sound: null, isPlaying: false, image: undefined }
      : pad
  ));
  // ... rest of method
}
```

**Step 8: Run tests**

Run: `npm test -- --include='**/soundboard.service.spec.ts'`
Expected: All tests pass

**Step 9: Commit**

```bash
git add src/app/core/services/soundboard.service.ts
git commit -m "feat(soundboard): add image support to SoundboardService"
```

---

## Task 6: Update Pad Display with Image Background

**Files:**
- Modify: `src/app/features/soundboard/sound-pad/sound-pad.component.ts`

**Step 1: Add TauriService and imageUrl computed signal**

Add to imports and component:

```typescript
import { TauriService } from '../../../core/services/tauri.service';

// In component class, add:
private tauri = inject(TauriService);
private _imagesDir = signal<string>('');

readonly imageUrl = computed(() => {
  const pad = this.pad;
  if (!pad.image?.localPath) return null;
  const dir = this._imagesDir();
  if (!dir) return null;
  return `${dir}/${pad.image.localPath}`;
});

// In constructor or ngOnInit:
async ngOnInit(): Promise<void> {
  try {
    const dir = await this.tauri.getImagesDir();
    this._imagesDir.set(dir);
  } catch (e) {
    console.error('Failed to get images dir:', e);
  }
}
```

**Step 2: Update pad template for image background**

Update the pad div to show image as background:

```html
<div
  class="aspect-square max-w-[140px] rounded-xl cursor-pointer relative overflow-hidden transition-all duration-150 flex items-end justify-center group"
  [class]="padClasses"
  [style.--pad-color]="pad.color"
  [title]="pad.sound?.name || ''"
  (click)="onClick($event)"
  (contextmenu)="onRightClick($event)"
>
  <!-- Image background -->
  @if (imageUrl()) {
    <img
      [src]="'asset://localhost/' + imageUrl()"
      alt=""
      class="absolute inset-0 w-full h-full object-cover"
    >
    <!-- Gradient overlay for text readability -->
    <div class="absolute inset-0 bg-gradient-to-t from-black/80 via-black/20 to-transparent"></div>
  }

  <!-- Hotkey badge -->
  @if (hotkey) {
    <span class="absolute top-2 left-2 px-1.5 py-0.5 bg-black/50 text-white/80 text-[10px] font-semibold rounded font-mono uppercase z-10">
      {{ hotkey }}
    </span>
  }

  @if (pad.sound) {
    <!-- Sound content (positioned at bottom with z-index) -->
    <div class="text-center px-2 w-full pb-2 relative z-10">
      <span class="block text-xs font-semibold text-white truncate mb-0.5 drop-shadow-[0_2px_4px_rgba(0,0,0,0.8)]">
        {{ pad.customName || pad.sound.name }}
      </span>
      @if (pad.customName) {
        <span class="block text-[9px] text-white/70 truncate drop-shadow-md">
          {{ pad.sound.name }}
        </span>
      }
      <span class="block text-[10px] text-white/70 mt-0.5 drop-shadow-md">
        {{ formatDuration(pad.sound.duration) }}
      </span>
    </div>
    <!-- ... rest of template -->
  }
</div>
```

**Step 3: Update padClasses for image mode**

Update the `padClasses` getter to handle image background:

```typescript
get padClasses(): string {
  const base = 'border-2';

  if (!this.pad.sound) {
    return `${base} border-dashed border-white/10 bg-white/5 hover:border-white/25 hover:bg-white/10 items-center`;
  }

  let classes = `${base} border-[var(--pad-color)]`;

  // Only add gradient background if no image
  if (!this.pad.image) {
    classes += ` bg-gradient-to-br from-[var(--pad-color)] to-[color-mix(in_srgb,var(--pad-color)_70%,black)]`;
  }

  // ... rest of method
}
```

**Step 4: Run app to verify**

Run: `npm run tauri dev`
Expected: App starts, existing pads display correctly

**Step 5: Commit**

```bash
git add src/app/features/soundboard/sound-pad/sound-pad.component.ts
git commit -m "feat(ui): display pad image as background"
```

---

## Task 7: Add Image Section to Pad Settings Modal

**Files:**
- Modify: `src/app/features/soundboard/sound-pad/sound-pad.component.ts`

**Step 1: Add image-related signals and methods**

```typescript
// Add to component class
showImageSearch = false;
imageSearchQuery = '';
imageSearchResults = signal<ImageSearchResult[]>([]);
selectedImageId = signal<string | null>(null);
imageLoading = signal(false);

private imageSearchService = inject(ImageSearchService);

// Add methods
async searchImages(): Promise<void> {
  if (!this.imageSearchQuery.trim()) return;

  try {
    this.imageLoading.set(true);
    const results = await this.imageSearchService.search(this.imageSearchQuery);
    this.imageSearchResults.set(results);
  } catch (err) {
    console.error('Image search failed:', err);
  } finally {
    this.imageLoading.set(false);
  }
}

async selectImage(result: ImageSearchResult): Promise<void> {
  try {
    this.imageLoading.set(true);

    // Download image
    const { data, extension } = await this.imageSearchService.downloadImage(result.fullUrl);

    // Save to backend
    const localPath = await this.tauri.savePadImage(this.pad.id, data, extension);

    // Update pad
    const image: PadImage = {
      localPath,
      originalUrl: result.fullUrl,
      attribution: result.attribution
    };
    this.imageChange.emit(image);

    this.showImageSearch = false;
  } catch (err) {
    console.error('Failed to save image:', err);
  } finally {
    this.imageLoading.set(false);
  }
}

async uploadImage(event: Event): Promise<void> {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;

  // Validate file type
  const validTypes = ['image/jpeg', 'image/png', 'image/webp', 'image/gif'];
  if (!validTypes.includes(file.type)) {
    alert('Unsupported image format. Use JPG, PNG, WebP, or GIF.');
    return;
  }

  // Validate file size (10MB max)
  if (file.size > 10 * 1024 * 1024) {
    alert('Image too large. Maximum size is 10MB.');
    return;
  }

  try {
    this.imageLoading.set(true);

    const data = new Uint8Array(await file.arrayBuffer());
    const extension = file.name.split('.').pop() || 'jpg';

    const localPath = await this.tauri.savePadImage(this.pad.id, data, extension);

    const image: PadImage = { localPath };
    this.imageChange.emit(image);
  } catch (err) {
    console.error('Failed to upload image:', err);
  } finally {
    this.imageLoading.set(false);
    input.value = ''; // Reset input
  }
}

async removeImage(): Promise<void> {
  try {
    await this.tauri.deletePadImage(this.pad.id);
    this.imageChange.emit(null);
  } catch (err) {
    console.error('Failed to remove image:', err);
  }
}

// Add Output
@Output() imageChange = new EventEmitter<PadImage | null>();
```

**Step 2: Add image section to modal template**

Add after the Name section in the modal:

```html
<!-- Image -->
<div class="mb-4 pt-4 border-t border-border">
  <div class="flex justify-between items-center mb-2 text-xs">
    <span class="text-text-secondary">Image</span>
  </div>

  <!-- Preview -->
  <div class="flex items-center gap-3 mb-3">
    <div class="w-16 h-16 rounded-lg overflow-hidden bg-surface-hover flex items-center justify-center border border-border">
      @if (imageUrl()) {
        <img [src]="'asset://localhost/' + imageUrl()" alt="" class="w-full h-full object-cover">
      } @else {
        <span class="text-2xl text-text-muted">&#128247;</span>
      }
    </div>
    <div class="flex flex-col gap-1">
      <input
        #imageUpload
        type="file"
        accept="image/jpeg,image/png,image/webp,image/gif"
        class="hidden"
        (change)="uploadImage($event)"
      >
      <button
        class="px-3 py-1.5 text-xs bg-surface-hover hover:bg-border rounded transition-colors text-text-secondary hover:text-text-primary"
        (click)="imageUpload.click()"
      >
        Upload
      </button>
      @if (imageSearchService.hasApiKey()) {
        <button
          class="px-3 py-1.5 text-xs bg-surface-hover hover:bg-border rounded transition-colors text-text-secondary hover:text-text-primary"
          (click)="showImageSearch = !showImageSearch"
        >
          {{ showImageSearch ? 'Close search' : 'Search' }}
        </button>
      }
      @if (pad.image) {
        <button
          class="px-3 py-1.5 text-xs text-status-error hover:bg-status-error/10 rounded transition-colors"
          (click)="removeImage()"
        >
          Remove
        </button>
      }
    </div>
  </div>

  <!-- Search Section (expandable) -->
  @if (showImageSearch) {
    <div class="p-3 bg-background rounded-lg border border-border">
      <div class="flex gap-2 mb-3">
        <input
          type="text"
          [(ngModel)]="imageSearchQuery"
          (keydown.enter)="searchImages()"
          placeholder="Search images..."
          class="flex-1 px-3 py-2 text-sm bg-surface-hover border border-border rounded text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent"
        >
        <button
          class="px-4 py-2 text-sm bg-accent hover:bg-accent/80 text-white rounded transition-colors"
          [disabled]="imageLoading()"
          (click)="searchImages()"
        >
          {{ imageLoading() ? '...' : 'Search' }}
        </button>
      </div>

      <!-- Results Grid -->
      @if (imageSearchResults().length > 0) {
        <div class="grid grid-cols-3 gap-2">
          @for (result of imageSearchResults(); track result.id) {
            <button
              class="aspect-square rounded overflow-hidden border-2 transition-all hover:scale-105"
              [class]="selectedImageId() === result.id ? 'border-accent' : 'border-transparent'"
              (click)="selectImage(result)"
            >
              <img [src]="result.thumbnailUrl" alt="" class="w-full h-full object-cover">
            </button>
          }
        </div>
      } @else if (!imageLoading()) {
        <p class="text-xs text-text-muted text-center py-4">
          Search for images on Pexels
        </p>
      }
    </div>
  }

  @if (!imageSearchService.hasApiKey()) {
    <p class="text-[10px] text-text-muted mt-2">
      Configure Pexels API key in Settings to search images
    </p>
  }
</div>
```

**Step 3: Add import for ImageSearchService and PadImage**

```typescript
import { ImageSearchService, ImageSearchResult } from '../../../core/services/image-search.service';
import { PadImage } from '../../../core/models';
```

**Step 4: Update resetAll to clear image**

```typescript
resetAll(): void {
  this.volumeChange.emit(1.0);
  this.speedChange.emit(1.0);
  this.customNameChange.emit(null);
  this.removeImage(); // Add this line
}
```

**Step 5: Run app to verify**

Run: `npm run tauri dev`
Expected: Image section appears in pad settings modal

**Step 6: Commit**

```bash
git add src/app/features/soundboard/sound-pad/sound-pad.component.ts
git commit -m "feat(ui): add image section to pad settings modal"
```

---

## Task 8: Add Pexels API Key Setting

**Files:**
- Modify: `src/app/shared/components/settings-popup/settings-popup.component.ts`

**Step 1: Add ImageSearchService injection and state**

```typescript
import { ImageSearchService } from '../../../core/services/image-search.service';

// In component class
private imageSearch = inject(ImageSearchService);

readonly pexelsApiKey = computed(() => this.imageSearch.apiKey() || '');
pexelsKeyInput = '';
testingKey = signal(false);
keyTestResult = signal<'success' | 'error' | null>(null);
```

**Step 2: Add API Key section to template**

Add after the Debug Section:

```html
<!-- Image Search Section -->
<div class="pt-6 border-t border-border">
  <h3 class="text-xs font-semibold text-text-muted uppercase tracking-wider mb-4">Image Search</h3>

  <div class="mb-4">
    <label class="text-sm text-text-secondary mb-2 block">Pexels API Key</label>
    <div class="flex gap-2">
      <input
        type="password"
        [value]="pexelsApiKey()"
        (input)="pexelsKeyInput = $any($event.target).value"
        placeholder="Enter your API key"
        class="flex-1 px-3 py-2 text-sm bg-background border border-border rounded text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent"
      >
      <button
        class="px-3 py-2 text-sm bg-surface-hover hover:bg-border rounded transition-colors text-text-secondary hover:text-text-primary"
        [disabled]="testingKey()"
        (click)="testAndSaveApiKey()"
      >
        {{ testingKey() ? 'Testing...' : 'Save' }}
      </button>
    </div>
    @if (keyTestResult() === 'success') {
      <p class="text-xs text-status-success mt-1">API key is valid</p>
    } @else if (keyTestResult() === 'error') {
      <p class="text-xs text-status-error mt-1">Invalid API key</p>
    }
    <p class="text-[10px] text-text-muted mt-2">
      Get a free API key at <a href="https://www.pexels.com/api/" target="_blank" class="text-accent hover:underline">pexels.com/api</a> (200 requests/hour)
    </p>
  </div>
</div>
```

**Step 3: Add test and save method**

```typescript
async testAndSaveApiKey(): Promise<void> {
  const key = this.pexelsKeyInput || this.pexelsApiKey();
  if (!key) return;

  this.testingKey.set(true);
  this.keyTestResult.set(null);

  try {
    const valid = await this.imageSearch.testApiKey(key);
    if (valid) {
      this.imageSearch.setApiKey(key);
      this.keyTestResult.set('success');
    } else {
      this.keyTestResult.set('error');
    }
  } catch {
    this.keyTestResult.set('error');
  } finally {
    this.testingKey.set(false);
  }
}
```

**Step 4: Run app to verify**

Run: `npm run tauri dev`
Expected: Pexels API Key section appears in settings

**Step 5: Commit**

```bash
git add src/app/shared/components/settings-popup/settings-popup.component.ts
git commit -m "feat(settings): add Pexels API key configuration"
```

---

## Task 9: Connect Image Events in Soundboard Component

**Files:**
- Modify: `src/app/features/soundboard/soundboard.component.ts`

**Step 1: Add imageChange handler**

Add to the `app-sound-pad` template bindings:

```html
(imageChange)="onImageChange(pad.id, $event)"
```

**Step 2: Add handler method**

```typescript
onImageChange(padId: string, image: PadImage | null): void {
  this.soundboard.setPadImage(padId, image);
}
```

**Step 3: Add PadImage import**

```typescript
import { PadImage } from '../../core/models';
```

**Step 4: Run full test**

Run: `npm run tauri dev`
Expected: Full image workflow works (upload, search, remove, persist)

**Step 5: Commit**

```bash
git add src/app/features/soundboard/soundboard.component.ts
git commit -m "feat(soundboard): connect image change events"
```

---

## Task 10: Add Auto-Suggestion Toast on Single Import

**Files:**
- Modify: `src/app/core/services/soundboard.service.ts`
- Create: `src/app/shared/components/image-suggestion-toast/image-suggestion-toast.component.ts`

**Step 1: Create ImageSuggestionToast component**

```typescript
import { Component, Input, Output, EventEmitter, signal, inject, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ImageSearchService, ImageSearchResult } from '../../../core/services/image-search.service';

@Component({
  selector: 'app-image-suggestion-toast',
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
    <div class="fixed bottom-4 right-4 z-50 animate-slide-in-up">
      <div class="bg-surface border border-border rounded-xl shadow-xl p-4 w-80">
        @if (!expanded()) {
          <!-- Compact view -->
          <div class="flex items-center gap-3">
            @if (suggestedImage()) {
              <img [src]="suggestedImage()!.thumbnailUrl" alt="" class="w-12 h-12 rounded object-cover">
            } @else {
              <div class="w-12 h-12 rounded bg-surface-hover flex items-center justify-center">
                <span class="text-text-muted">&#128247;</span>
              </div>
            }
            <div class="flex-1 min-w-0">
              <p class="text-sm text-text-primary truncate">Image for "{{ soundName }}"</p>
              <p class="text-xs text-text-muted">{{ suggestedImage() ? 'Suggestion found' : 'No results' }}</p>
            </div>
          </div>
          <div class="flex gap-2 mt-3">
            @if (suggestedImage()) {
              <button
                class="flex-1 px-3 py-2 text-xs bg-accent hover:bg-accent/80 text-white rounded transition-colors"
                (click)="accept.emit(suggestedImage()!)"
              >
                Accept
              </button>
              <button
                class="flex-1 px-3 py-2 text-xs bg-surface-hover hover:bg-border text-text-secondary rounded transition-colors"
                (click)="expanded.set(true)"
              >
                Other choices
              </button>
            }
            <button
              class="px-3 py-2 text-xs text-text-muted hover:text-text-primary transition-colors"
              (click)="ignore.emit()"
            >
              Ignore
            </button>
          </div>
        } @else {
          <!-- Expanded view -->
          <div class="mb-3">
            <p class="text-sm text-text-primary mb-2">Image for "{{ soundName }}"</p>
            <div class="flex gap-2">
              <input
                type="text"
                [(ngModel)]="searchQuery"
                (keydown.enter)="search()"
                placeholder="Search..."
                class="flex-1 px-3 py-2 text-sm bg-surface-hover border border-border rounded text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent"
              >
              <button
                class="px-3 py-2 text-sm bg-accent text-white rounded"
                (click)="search()"
              >
                Search
              </button>
            </div>
          </div>

          @if (searchResults().length > 0) {
            <div class="grid grid-cols-4 gap-2 mb-3">
              @for (result of searchResults(); track result.id) {
                <button
                  class="aspect-square rounded overflow-hidden border-2 transition-all hover:scale-105"
                  [class]="selectedResult()?.id === result.id ? 'border-accent' : 'border-transparent'"
                  (click)="selectedResult.set(result)"
                >
                  <img [src]="result.thumbnailUrl" alt="" class="w-full h-full object-cover">
                </button>
              }
            </div>
          }

          <div class="flex gap-2">
            @if (selectedResult()) {
              <button
                class="flex-1 px-3 py-2 text-xs bg-accent hover:bg-accent/80 text-white rounded transition-colors"
                (click)="accept.emit(selectedResult()!)"
              >
                Select
              </button>
            }
            <button
              class="px-3 py-2 text-xs text-text-muted hover:text-text-primary transition-colors"
              (click)="ignore.emit()"
            >
              Ignore
            </button>
          </div>
        }
      </div>
    </div>
  `
})
export class ImageSuggestionToastComponent implements OnInit {
  @Input({ required: true }) soundName!: string;
  @Input({ required: true }) filename!: string;
  @Output() accept = new EventEmitter<ImageSearchResult>();
  @Output() ignore = new EventEmitter<void>();

  private imageSearch = inject(ImageSearchService);

  expanded = signal(false);
  searchQuery = '';
  suggestedImage = signal<ImageSearchResult | null>(null);
  searchResults = signal<ImageSearchResult[]>([]);
  selectedResult = signal<ImageSearchResult | null>(null);

  async ngOnInit(): Promise<void> {
    // Auto-search based on filename
    this.searchQuery = this.imageSearch.extractQueryFromFilename(this.filename);
    if (this.searchQuery && this.imageSearch.hasApiKey()) {
      await this.search();
      if (this.searchResults().length > 0) {
        this.suggestedImage.set(this.searchResults()[0]);
      }
    }
  }

  async search(): Promise<void> {
    if (!this.searchQuery.trim()) return;
    try {
      const results = await this.imageSearch.search(this.searchQuery, 1, 8);
      this.searchResults.set(results);
      if (!this.expanded() && results.length > 0) {
        this.suggestedImage.set(results[0]);
      }
    } catch (err) {
      console.error('Search failed:', err);
    }
  }
}
```

**Step 2: Create component file**

Save to: `src/app/shared/components/image-suggestion-toast/image-suggestion-toast.component.ts`

**Step 3: Run build check**

Run: `npm run build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add src/app/shared/components/image-suggestion-toast/
git commit -m "feat(ui): add image suggestion toast component"
```

---

## Task 11: Add Bulk Import Image Wizard

**Files:**
- Create: `src/app/shared/components/bulk-image-wizard/bulk-image-wizard.component.ts`

**Step 1: Create BulkImageWizard component**

```typescript
import { Component, Input, Output, EventEmitter, signal, inject, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ImageSearchService, ImageSearchResult } from '../../../core/services/image-search.service';
import { SoundPad, PadImage } from '../../../core/models';

interface PadImageSelection {
  pad: SoundPad;
  selectedImage: ImageSearchResult | null;
}

@Component({
  selector: 'app-bulk-image-wizard',
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
    <div class="fixed inset-0 z-50 flex items-center justify-center">
      <!-- Backdrop -->
      <div class="absolute inset-0 bg-black/60 backdrop-blur-sm" (click)="close.emit()"></div>

      <!-- Modal -->
      <div class="relative bg-surface border border-border rounded-xl p-6 w-full max-w-lg animate-scale-in" (click)="$event.stopPropagation()">
        <!-- Header -->
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-lg font-semibold text-text-primary">
            Image for "{{ currentPad()?.sound?.name }}" ({{ currentIndex() + 1 }}/{{ pads.length }})
          </h2>
          <button
            class="text-text-muted hover:text-text-primary"
            (click)="skip()"
          >
            Skip
          </button>
        </div>

        <!-- Search -->
        <div class="flex gap-2 mb-4">
          <input
            type="text"
            [(ngModel)]="searchQuery"
            (keydown.enter)="search()"
            placeholder="Search images..."
            class="flex-1 px-3 py-2 text-sm bg-surface-hover border border-border rounded text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent"
          >
          <button
            class="px-4 py-2 text-sm bg-accent hover:bg-accent/80 text-white rounded transition-colors"
            (click)="search()"
          >
            Search
          </button>
        </div>

        <!-- Results Grid -->
        @if (searchResults().length > 0) {
          <div class="grid grid-cols-4 gap-2 mb-4 max-h-48 overflow-y-auto">
            @for (result of searchResults(); track result.id) {
              <button
                class="aspect-square rounded overflow-hidden border-2 transition-all hover:scale-105"
                [class]="selectedImage()?.id === result.id ? 'border-accent' : 'border-transparent'"
                (click)="selectedImage.set(result)"
              >
                <img [src]="result.thumbnailUrl" alt="" class="w-full h-full object-cover">
              </button>
            }
          </div>
        } @else {
          <div class="h-48 flex items-center justify-center text-text-muted">
            Search for images on Pexels
          </div>
        }

        <!-- Navigation -->
        <div class="flex items-center justify-between pt-4 border-t border-border">
          <button
            class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
            [disabled]="currentIndex() === 0"
            (click)="previous()"
          >
            &larr; Previous
          </button>

          <div class="flex gap-2">
            @if (selectedImage()) {
              <button
                class="px-4 py-2 text-sm bg-accent hover:bg-accent/80 text-white rounded transition-colors"
                (click)="selectAndNext()"
              >
                {{ isLast() ? 'Finish' : 'Select & Next' }}
              </button>
            }
            <button
              class="px-4 py-2 text-sm bg-surface-hover hover:bg-border text-text-secondary rounded transition-colors"
              (click)="nextOrFinish()"
            >
              {{ isLast() ? 'Finish' : 'Next &rarr;' }}
            </button>
          </div>
        </div>

        <!-- Finish button -->
        <button
          class="w-full mt-4 py-2 text-sm text-text-muted hover:text-text-primary transition-colors"
          (click)="finish()"
        >
          Finish and keep selected images
        </button>
      </div>
    </div>
  `
})
export class BulkImageWizardComponent implements OnInit {
  @Input({ required: true }) pads!: SoundPad[];
  @Output() close = new EventEmitter<void>();
  @Output() selectImage = new EventEmitter<{ padId: string; image: ImageSearchResult }>();

  private imageSearch = inject(ImageSearchService);

  currentIndex = signal(0);
  searchQuery = '';
  searchResults = signal<ImageSearchResult[]>([]);
  selectedImage = signal<ImageSearchResult | null>(null);
  selections = new Map<string, ImageSearchResult>();

  readonly currentPad = () => this.pads[this.currentIndex()];
  readonly isLast = () => this.currentIndex() === this.pads.length - 1;

  async ngOnInit(): Promise<void> {
    await this.loadCurrentPad();
  }

  private async loadCurrentPad(): Promise<void> {
    const pad = this.currentPad();
    if (!pad?.sound) return;

    // Check if we already have a selection for this pad
    const existingSelection = this.selections.get(pad.id);
    if (existingSelection) {
      this.selectedImage.set(existingSelection);
    } else {
      this.selectedImage.set(null);
    }

    // Auto-search based on sound name
    this.searchQuery = this.imageSearch.extractQueryFromFilename(pad.sound.name);
    if (this.searchQuery && this.imageSearch.hasApiKey()) {
      await this.search();
    }
  }

  async search(): Promise<void> {
    if (!this.searchQuery.trim()) return;
    try {
      const results = await this.imageSearch.search(this.searchQuery, 1, 12);
      this.searchResults.set(results);
    } catch (err) {
      console.error('Search failed:', err);
    }
  }

  previous(): void {
    if (this.currentIndex() > 0) {
      this.currentIndex.update(i => i - 1);
      this.loadCurrentPad();
    }
  }

  skip(): void {
    this.selectedImage.set(null);
    this.selections.delete(this.currentPad()?.id || '');
    this.nextOrFinish();
  }

  selectAndNext(): void {
    const image = this.selectedImage();
    const pad = this.currentPad();
    if (image && pad) {
      this.selections.set(pad.id, image);
      this.selectImage.emit({ padId: pad.id, image });
    }
    this.nextOrFinish();
  }

  nextOrFinish(): void {
    if (this.isLast()) {
      this.finish();
    } else {
      this.currentIndex.update(i => i + 1);
      this.loadCurrentPad();
    }
  }

  finish(): void {
    this.close.emit();
  }
}
```

**Step 2: Save component file**

Save to: `src/app/shared/components/bulk-image-wizard/bulk-image-wizard.component.ts`

**Step 3: Run build check**

Run: `npm run build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add src/app/shared/components/bulk-image-wizard/
git commit -m "feat(ui): add bulk image wizard component"
```

---

## Task 12: Integrate Auto-Suggestion in Import Flow

**Files:**
- Modify: `src/app/features/soundboard/soundboard.component.ts`

**Step 1: Add component imports and state**

```typescript
import { ImageSuggestionToastComponent } from '../../shared/components/image-suggestion-toast/image-suggestion-toast.component';
import { BulkImageWizardComponent } from '../../shared/components/bulk-image-wizard/bulk-image-wizard.component';
import { ImageSearchService, ImageSearchResult } from '../../core/services/image-search.service';

// In component imports array
ImageSuggestionToastComponent,
BulkImageWizardComponent,

// In component class
private imageSearch = inject(ImageSearchService);

// State for auto-suggestion
showImageSuggestion = signal(false);
suggestionSoundName = '';
suggestionFilename = '';
suggestionPadId = '';

showBulkWizard = signal(false);
bulkWizardPads = signal<SoundPad[]>([]);
pendingBulkPads = signal<SoundPad[]>([]);
showBulkPrompt = signal(false);
```

**Step 2: Add methods for handling suggestions**

```typescript
async onAcceptSuggestion(result: ImageSearchResult): Promise<void> {
  try {
    const { data, extension } = await this.imageSearch.downloadImage(result.fullUrl);
    const localPath = await this.tauri.savePadImage(this.suggestionPadId, data, extension);

    const image: PadImage = {
      localPath,
      originalUrl: result.fullUrl,
      attribution: result.attribution
    };
    this.soundboard.setPadImage(this.suggestionPadId, image);
  } catch (err) {
    console.error('Failed to save suggested image:', err);
  } finally {
    this.showImageSuggestion.set(false);
  }
}

onIgnoreSuggestion(): void {
  this.showImageSuggestion.set(false);
}

// Bulk import handlers
onStartBulkWizard(): void {
  this.showBulkPrompt.set(false);
  this.bulkWizardPads.set(this.pendingBulkPads());
  this.showBulkWizard.set(true);
}

onSkipBulkWizard(): void {
  this.showBulkPrompt.set(false);
  this.pendingBulkPads.set([]);
}

async onBulkSelectImage(event: { padId: string; image: ImageSearchResult }): Promise<void> {
  try {
    const { data, extension } = await this.imageSearch.downloadImage(event.image.fullUrl);
    const localPath = await this.tauri.savePadImage(event.padId, data, extension);

    const image: PadImage = {
      localPath,
      originalUrl: event.image.fullUrl,
      attribution: event.image.attribution
    };
    this.soundboard.setPadImage(event.padId, image);
  } catch (err) {
    console.error('Failed to save image:', err);
  }
}

onBulkWizardClose(): void {
  this.showBulkWizard.set(false);
  this.bulkWizardPads.set([]);
  this.pendingBulkPads.set([]);
}
```

**Step 3: Update import flow to trigger suggestions**

Modify the import methods to trigger auto-suggestion:

```typescript
// After successful single import
private triggerSingleImportSuggestion(pad: SoundPad): void {
  if (!this.imageSearch.hasApiKey() || !pad.sound) return;

  this.suggestionPadId = pad.id;
  this.suggestionSoundName = pad.sound.name;
  this.suggestionFilename = pad.sound.path.split('/').pop() || pad.sound.name;
  this.showImageSuggestion.set(true);
}

// After successful bulk import
private triggerBulkImportSuggestion(importedPads: SoundPad[]): void {
  if (!this.imageSearch.hasApiKey() || importedPads.length === 0) return;

  this.pendingBulkPads.set(importedPads);
  this.showBulkPrompt.set(true);
}
```

**Step 4: Add template for toasts and wizard**

Add to component template:

```html
<!-- Image Suggestion Toast (single import) -->
@if (showImageSuggestion()) {
  <app-image-suggestion-toast
    [soundName]="suggestionSoundName"
    [filename]="suggestionFilename"
    (accept)="onAcceptSuggestion($event)"
    (ignore)="onIgnoreSuggestion()"
  />
}

<!-- Bulk Import Prompt -->
@if (showBulkPrompt()) {
  <div class="fixed bottom-4 right-4 z-50 animate-slide-in-up">
    <div class="bg-surface border border-border rounded-xl shadow-xl p-4 w-80">
      <p class="text-sm text-text-primary mb-3">
        Assign images to {{ pendingBulkPads().length }} imported sounds?
      </p>
      <div class="flex gap-2">
        <button
          class="flex-1 px-3 py-2 text-sm bg-accent hover:bg-accent/80 text-white rounded transition-colors"
          (click)="onStartBulkWizard()"
        >
          Yes
        </button>
        <button
          class="flex-1 px-3 py-2 text-sm bg-surface-hover hover:bg-border text-text-secondary rounded transition-colors"
          (click)="onSkipBulkWizard()"
        >
          No thanks
        </button>
      </div>
    </div>
  </div>
}

<!-- Bulk Image Wizard -->
@if (showBulkWizard()) {
  <app-bulk-image-wizard
    [pads]="bulkWizardPads()"
    (selectImage)="onBulkSelectImage($event)"
    (close)="onBulkWizardClose()"
  />
}
```

**Step 5: Run full test**

Run: `npm run tauri dev`
Expected: Auto-suggestion appears after import

**Step 6: Commit**

```bash
git add src/app/features/soundboard/soundboard.component.ts
git commit -m "feat(soundboard): integrate auto-suggestion for imported sounds"
```

---

## Task 13: Add Garbage Collection on Startup

**Files:**
- Modify: `src/app/app.component.ts`

**Step 1: Call cleanup on app init**

Add to AppComponent:

```typescript
import { TauriService } from './core/services/tauri.service';

// In constructor or ngOnInit
private async cleanupOrphanedImages(): Promise<void> {
  try {
    const count = await this.tauri.cleanupOrphanedImages();
    if (count > 0) {
      console.log(`Cleaned up ${count} orphaned images`);
    }
  } catch (err) {
    console.error('Failed to cleanup orphaned images:', err);
  }
}
```

**Step 2: Call on init**

```typescript
ngOnInit(): void {
  this.cleanupOrphanedImages();
}
```

**Step 3: Run app to verify**

Run: `npm run tauri dev`
Expected: App starts without errors

**Step 4: Commit**

```bash
git add src/app/app.component.ts
git commit -m "feat(app): add image garbage collection on startup"
```

---

## Task 14: Add Unit Tests for Image Features

**Files:**
- Create: `src/app/core/services/image-search.service.spec.ts`
- Modify: `src/app/core/services/soundboard.service.spec.ts`

**Step 1: Create ImageSearchService tests**

```typescript
import { TestBed } from '@angular/core/testing';
import { ImageSearchService } from './image-search.service';

describe('ImageSearchService', () => {
  let service: ImageSearchService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(ImageSearchService);
    localStorage.clear();
  });

  afterEach(() => {
    localStorage.clear();
  });

  describe('extractQueryFromFilename', () => {
    it('should extract words from filename', () => {
      expect(service.extractQueryFromFilename('airhorn.mp3')).toBe('airhorn');
    });

    it('should replace underscores with spaces', () => {
      expect(service.extractQueryFromFilename('funny_airhorn_sound.mp3')).toBe('funny airhorn sound');
    });

    it('should replace hyphens with spaces', () => {
      expect(service.extractQueryFromFilename('funny-airhorn-sound.mp3')).toBe('funny airhorn sound');
    });

    it('should limit to 3 words', () => {
      expect(service.extractQueryFromFilename('one_two_three_four_five.mp3')).toBe('one two three');
    });

    it('should convert to lowercase', () => {
      expect(service.extractQueryFromFilename('LOUD_AIRHORN.mp3')).toBe('loud airhorn');
    });
  });

  describe('API key management', () => {
    it('should start with no API key', () => {
      expect(service.hasApiKey()).toBeFalse();
    });

    it('should save API key to localStorage', () => {
      service.setApiKey('test-key');
      expect(localStorage.getItem('pexels_api_key')).toBe('test-key');
      expect(service.hasApiKey()).toBeTrue();
    });

    it('should remove API key when set to null', () => {
      service.setApiKey('test-key');
      service.setApiKey(null);
      expect(localStorage.getItem('pexels_api_key')).toBeNull();
      expect(service.hasApiKey()).toBeFalse();
    });

    it('should load API key from localStorage on construction', () => {
      localStorage.setItem('pexels_api_key', 'stored-key');
      const newService = new ImageSearchService();
      expect(newService.apiKey()).toBe('stored-key');
    });
  });
});
```

**Step 2: Add SoundboardService image tests**

Add to existing spec file:

```typescript
describe('setPadImage', () => {
  it('should set image on pad', () => {
    const image: PadImage = { localPath: 'pad-0-abc123.jpg' };
    service.setPadImage('pad-0', image);
    expect(service.pads()[0].image).toEqual(image);
  });

  it('should clear image when set to null', () => {
    const image: PadImage = { localPath: 'pad-0-abc123.jpg' };
    service.setPadImage('pad-0', image);
    service.setPadImage('pad-0', null);
    expect(service.pads()[0].image).toBeUndefined();
  });
});
```

**Step 3: Run tests**

Run: `npm test`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/app/core/services/image-search.service.spec.ts src/app/core/services/soundboard.service.spec.ts
git commit -m "test: add unit tests for image features"
```

---

## Task 15: Update Roadmap and Archive Plan

**Files:**
- Modify: `ROADMAP.md`
- Move: `docs/plans/2026-01-05-pad-images-design.md` -> `docs/plans/archive/`
- Move: `docs/plans/2026-01-05-pad-images-implementation.md` -> `docs/plans/archive/`

**Step 1: Update ROADMAP.md**

Mark Pad Images as complete in Phase 3:

```markdown
### Done
- [x] **Pad Images** *(see docs/plans/archive/2026-01-05-pad-images-design.md)*
  - Upload, URL, and Pexels search
  - Local image storage
  - Auto-suggestion on import
```

Update Phase 3 progress to 70% or calculate new percentage.

**Step 2: Archive plan files**

```bash
mkdir -p docs/plans/archive
mv docs/plans/2026-01-05-pad-images-design.md docs/plans/archive/
mv docs/plans/2026-01-05-pad-images-implementation.md docs/plans/archive/
```

**Step 3: Commit**

```bash
git add ROADMAP.md docs/plans/
git commit -m "docs: mark pad images complete, archive plans"
```

---

## Summary

**Total Tasks:** 15
**Estimated Time:** 4-6 hours

**Key Deliverables:**
1. Backend image storage commands (Rust)
2. Frontend ImageSearchService (Pexels API)
3. Pad display with image background
4. Image section in pad settings modal
5. Pexels API key configuration in settings
6. Auto-suggestion toast on single import
7. Bulk image wizard for multiple imports
8. Image garbage collection on startup
9. Unit tests for image features

**Dependencies:**
- sha2 crate for Rust (hashing)
- Pexels API key (user-provided)

**Future Work (Phase 4):**
- Image Search Proxy via Django backend
- Remove need for user-provided API key
