# DuckDuckGo Images Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace Pexels API with DuckDuckGo Images for pad image search.

**Architecture:** Use Tauri HTTP plugin to bypass CORS, call DuckDuckGo's internal image API with vqd token authentication.

**Tech Stack:** Angular 18, Tauri 2, tauri-plugin-http

---

## Task 1: Add Tauri HTTP Plugin

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/capabilities/default.json`

**Step 1: Add dependency to Cargo.toml**

Add to `[dependencies]` section:

```toml
tauri-plugin-http = "2"
```

**Step 2: Register plugin in lib.rs**

Add after other plugin registrations (around line 108):

```rust
.plugin(tauri_plugin_http::init())
```

**Step 3: Add HTTP permissions to capabilities**

Update `src-tauri/capabilities/default.json`, add to permissions array:

```json
{
  "identifier": "http:default",
  "allow": [
    { "url": "https://duckduckgo.com/*" }
  ]
}
```

**Step 4: Verify build**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Compiles without errors

**Step 5: Commit**

```bash
git add src-tauri/
git commit -m "feat(backend): add tauri-plugin-http for DuckDuckGo images"
```

---

## Task 2: Rewrite ImageSearchService for DuckDuckGo

**Files:**
- Modify: `src/app/core/services/image-search.service.ts`

**Step 1: Replace entire service implementation**

```typescript
import { Injectable, signal, computed } from '@angular/core';
import { fetch } from '@tauri-apps/plugin-http';

export interface ImageSearchResult {
  id: string;
  thumbnailUrl: string;
  fullUrl: string;
  title: string;
}

@Injectable({
  providedIn: 'root'
})
export class ImageSearchService {
  private _loading = signal(false);
  private _error = signal<string | null>(null);

  readonly loading = this._loading.asReadonly();
  readonly error = this._error.asReadonly();

  // Always available (no API key needed)
  readonly hasApiKey = computed(() => true);

  /**
   * Get vqd token from DuckDuckGo
   */
  private async getVqdToken(query: string): Promise<string> {
    const url = `https://duckduckgo.com/?q=${encodeURIComponent(query)}&iax=images&ia=images`;

    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36'
      }
    });

    if (!response.ok) {
      throw new Error('Failed to get search token');
    }

    const html = await response.text();

    // Extract vqd token from HTML
    const vqdMatch = html.match(/vqd=["']([^"']+)["']/);
    if (!vqdMatch) {
      throw new Error('Could not extract search token');
    }

    return vqdMatch[1];
  }

  /**
   * Search for images using DuckDuckGo
   */
  async search(query: string, page: number = 1, perPage: number = 12): Promise<ImageSearchResult[]> {
    this._loading.set(true);
    this._error.set(null);

    try {
      // Get vqd token first
      const vqd = await this.getVqdToken(query);

      // Fetch images
      const url = `https://duckduckgo.com/i.js?l=fr-fr&o=json&q=${encodeURIComponent(query)}&vqd=${vqd}&p=${page}`;

      const response = await fetch(url, {
        method: 'GET',
        headers: {
          'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36',
          'Accept': 'application/json'
        }
      });

      if (!response.ok) {
        throw new Error('Search failed');
      }

      const data = await response.json() as { results?: Array<{ image: string; thumbnail: string; title: string }> };

      if (!data.results || data.results.length === 0) {
        return [];
      }

      // Map to our interface, limit to perPage
      return data.results.slice(0, perPage).map((item, index) => ({
        id: `ddg-${index}-${Date.now()}`,
        thumbnailUrl: item.thumbnail,
        fullUrl: item.image,
        title: item.title || 'Image'
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
    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36'
      }
    });

    if (!response.ok) {
      throw new Error('Failed to download image');
    }

    // Determine extension from URL or content-type
    const contentType = response.headers.get('content-type') || '';
    let extension = 'jpg';
    if (contentType.includes('png') || url.includes('.png')) extension = 'png';
    else if (contentType.includes('webp') || url.includes('.webp')) extension = 'webp';
    else if (contentType.includes('gif') || url.includes('.gif')) extension = 'gif';

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
    const nameWithoutExt = filename.replace(/\.[^/.]+$/, '');
    const words = nameWithoutExt.replace(/[-_]/g, ' ').split(/\s+/);
    return words.slice(0, 3).join(' ').toLowerCase();
  }

  // Removed Pexels-specific methods:
  // - setApiKey()
  // - testApiKey()
  // - apiKey signal
}
```

**Step 2: Run build check**

Run: `npm run build`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/app/core/services/image-search.service.ts
git commit -m "feat(images): replace Pexels with DuckDuckGo search"
```

---

## Task 3: Remove Pexels Settings Section

**Files:**
- Modify: `src/app/shared/components/settings-popup/settings-popup.component.ts`

**Step 1: Remove Image Search section from template**

Delete the entire "Image Search Section" block (lines ~212-248):

```html
<!-- Image Search Section -->
<div class="pt-6 border-t border-border">
  ...entire section...
</div>
```

**Step 2: Remove ImageSearchService import and injection**

Remove from imports:
```typescript
import { ImageSearchService } from '../../../core/services/image-search.service';
```

Remove from constructor:
```typescript
private imageSearch: ImageSearchService
```

**Step 3: Remove image search state and methods**

Delete:
```typescript
// Image search state
readonly pexelsApiKey = computed(() => this.imageSearch.apiKey() || '');
pexelsKeyInput = '';
testingKey = signal(false);
keyTestResult = signal<'success' | 'error' | null>(null);
```

Delete `ngOnInit` line:
```typescript
this.pexelsKeyInput = this.pexelsApiKey();
```

Delete method:
```typescript
async testAndSaveApiKey(): Promise<void> { ... }
```

**Step 4: Run build check**

Run: `npm run build`
Expected: Build succeeds

**Step 5: Commit**

```bash
git add src/app/shared/components/settings-popup/settings-popup.component.ts
git commit -m "refactor(settings): remove Pexels API key configuration"
```

---

## Task 4: Update Sound-Pad Image Section

**Files:**
- Modify: `src/app/features/soundboard/sound-pad/sound-pad.component.ts`

**Step 1: Remove hasApiKey check for Search button**

The Search button currently checks `imageSearchService.hasApiKey()`. Since DDG always works, simplify:

Change:
```html
@if (imageSearchService.hasApiKey()) {
  <button ...>Search</button>
}
```

To:
```html
<button
  class="px-3 py-1.5 text-xs bg-surface-hover hover:bg-border rounded transition-colors text-text-secondary hover:text-text-primary"
  (click)="showImageSearch = !showImageSearch"
  [disabled]="imageLoading()"
>
  {{ showImageSearch ? 'Close search' : 'Search' }}
</button>
```

**Step 2: Remove the "Configure API key" hint**

Delete:
```html
@if (!imageSearchService.hasApiKey()) {
  <p class="text-[10px] text-text-muted mt-2">
    Configure Pexels API key in Settings to search images
  </p>
}
```

**Step 3: Run build check**

Run: `npm run build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add src/app/features/soundboard/sound-pad/sound-pad.component.ts
git commit -m "refactor(ui): remove Pexels API key checks from image search"
```

---

## Task 5: Update Tests

**Files:**
- Modify: `src/app/core/services/image-search.service.spec.ts`

**Step 1: Update tests for new implementation**

```typescript
import { TestBed } from '@angular/core/testing';
import { ImageSearchService } from './image-search.service';

describe('ImageSearchService', () => {
  let service: ImageSearchService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(ImageSearchService);
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

  describe('hasApiKey', () => {
    it('should always return true (DuckDuckGo needs no key)', () => {
      expect(service.hasApiKey()).toBeTrue();
    });
  });

  describe('initial state', () => {
    it('should not be loading initially', () => {
      expect(service.loading()).toBeFalse();
    });

    it('should have no error initially', () => {
      expect(service.error()).toBeNull();
    });
  });
});
```

**Step 2: Run tests**

Run: `npm test -- --no-watch --browsers=ChromeHeadless`
Expected: All tests pass

**Step 3: Commit**

```bash
git add src/app/core/services/image-search.service.spec.ts
git commit -m "test: update image search tests for DuckDuckGo"
```

---

## Task 6: Clean Up and Final Verification

**Files:**
- Various

**Step 1: Remove any remaining Pexels references**

Search for "pexels" (case-insensitive) in the codebase:
```bash
grep -ri "pexels" src/
```

If any found, remove them.

**Step 2: Run full test suite**

Run: `npm test -- --no-watch --browsers=ChromeHeadless`
Expected: All tests pass

**Step 3: Run Rust tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --all`
Expected: All tests pass

**Step 4: Test manually**

Run: `npm run tauri dev`
- Open pad settings
- Click "Search"
- Search for "airhorn"
- Verify images load from DuckDuckGo

**Step 5: Commit any cleanup**

```bash
git add -A
git commit -m "chore: clean up remaining Pexels references"
```

---

## Task 7: Update Documentation

**Files:**
- Modify: `ROADMAP.md`
- Move: design/implementation files to archive

**Step 1: Update ROADMAP.md**

In Phase 3 "Pad Images" entry, update description:
```markdown
- [x] **Pad Images** *(see docs/plans/archive/2026-01-05-pad-images-design.md)*
  - Upload, URL, and DuckDuckGo search
  - Local image storage
  - Auto-suggestion on import
```

**Step 2: Archive plan files**

```bash
mv docs/plans/2026-01-05-duckduckgo-images-design.md docs/plans/archive/
mv docs/plans/2026-01-05-duckduckgo-images-implementation.md docs/plans/archive/
```

**Step 3: Commit**

```bash
git add ROADMAP.md docs/plans/
git commit -m "docs: update pad images to use DuckDuckGo, archive plans"
```

---

## Summary

**Total Tasks:** 7

**Key Changes:**
1. Add tauri-plugin-http for CORS-free HTTP requests
2. Rewrite ImageSearchService for DuckDuckGo API
3. Remove Pexels settings section
4. Simplify UI (no API key checks)
5. Update tests
6. Clean up and verify
7. Update documentation

**Removed:**
- Pexels API key configuration
- API key validation
- Attribution field

**Benefits:**
- No configuration required
- Works out of the box
- More relevant image results
