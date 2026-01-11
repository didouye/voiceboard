# Fuzzy Search Bar Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a search bar above the soundboard that filters sounds using fuzzy matching (exact, subsequence, Levenshtein) with character highlighting.

**Architecture:** Create a new `FuzzySearchService` for search logic, add `searchQuery` signal to `SoundboardService`, create a `SearchBarComponent`, and modify `SoundPadComponent` to highlight matched characters.

**Tech Stack:** Angular 17+ signals, standalone components, Jasmine tests

---

## Task 1: Create FuzzySearchService with Exact Match

**Files:**
- Create: `src/app/core/services/fuzzy-search.service.ts`
- Create: `src/app/core/services/fuzzy-search.service.spec.ts`

**Step 1: Write failing tests for exact match**

```typescript
// src/app/core/services/fuzzy-search.service.spec.ts
import { TestBed } from '@angular/core/testing';
import { FuzzySearchService, SearchResult } from './fuzzy-search.service';
import { Sound } from '../models';

describe('FuzzySearchService', () => {
  let service: FuzzySearchService;

  const createMockSound = (overrides: Partial<Sound> = {}): Sound => ({
    id: 'hash_abc123',
    name: 'test-sound',
    path: '/path/to/test-sound.mp3',
    duration: 5.0,
    volume: 1.0,
    speed: 1.0,
    folderIds: [],
    isPlaying: false,
    addedAt: Date.now(),
    ...overrides
  });

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [FuzzySearchService]
    });
    service = TestBed.inject(FuzzySearchService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });

  describe('exact match', () => {
    it('should find exact substring match', () => {
      const sounds = [createMockSound({ id: '1', name: 'barbe bleue' })];
      const results = service.search('barbe', sounds);

      expect(results.length).toBe(1);
      expect(results[0].sound.id).toBe('1');
      expect(results[0].score).toBe(100);
    });

    it('should be case insensitive', () => {
      const sounds = [createMockSound({ id: '1', name: 'Barbe Bleue' })];
      const results = service.search('BARBE', sounds);

      expect(results.length).toBe(1);
      expect(results[0].score).toBe(100);
    });

    it('should return matched indices for exact match', () => {
      const sounds = [createMockSound({ id: '1', name: 'barbe bleue' })];
      const results = service.search('barbe', sounds);

      expect(results[0].matchedIndices).toEqual([0, 1, 2, 3, 4]);
    });

    it('should use customName when available', () => {
      const sounds = [createMockSound({ id: '1', name: 'original', customName: 'barbe' })];
      const results = service.search('barbe', sounds);

      expect(results.length).toBe(1);
    });

    it('should return empty array for no matches', () => {
      const sounds = [createMockSound({ id: '1', name: 'test' })];
      const results = service.search('xyz', sounds);

      expect(results.length).toBe(0);
    });
  });
});
```

**Step 2: Run test to verify it fails**

Run: `npm test -- --include='**/fuzzy-search.service.spec.ts'`
Expected: FAIL - service not found

**Step 3: Write minimal implementation**

```typescript
// src/app/core/services/fuzzy-search.service.ts
import { Injectable } from '@angular/core';
import { Sound } from '../models';

export interface SearchResult {
  sound: Sound;
  score: number;
  matchedIndices: number[];
}

@Injectable({
  providedIn: 'root'
})
export class FuzzySearchService {

  search(query: string, sounds: Sound[]): SearchResult[] {
    const normalizedQuery = this.normalizeText(query);
    if (!normalizedQuery) return [];

    const results: SearchResult[] = [];

    for (const sound of sounds) {
      const displayName = sound.customName || sound.name;
      const normalizedName = this.normalizeText(displayName);

      const exactMatch = this.exactMatch(normalizedQuery, normalizedName, displayName);
      if (exactMatch) {
        results.push({ sound, ...exactMatch });
      }
    }

    return results.sort((a, b) => b.score - a.score);
  }

  private exactMatch(query: string, normalizedText: string, originalText: string): { score: number; matchedIndices: number[] } | null {
    const index = normalizedText.indexOf(query);
    if (index === -1) return null;

    const matchedIndices: number[] = [];
    for (let i = index; i < index + query.length; i++) {
      matchedIndices.push(i);
    }

    return { score: 100, matchedIndices };
  }

  private normalizeText(text: string): string {
    return text
      .toLowerCase()
      .normalize('NFD')
      .replace(/[\u0300-\u036f]/g, ''); // Remove accents
  }
}
```

**Step 4: Run test to verify it passes**

Run: `npm test -- --include='**/fuzzy-search.service.spec.ts'`
Expected: PASS

**Step 5: Commit**

```bash
git add src/app/core/services/fuzzy-search.service.ts src/app/core/services/fuzzy-search.service.spec.ts
git commit -m "feat(search): add FuzzySearchService with exact match"
```

---

## Task 2: Add Subsequence Match to FuzzySearchService

**Files:**
- Modify: `src/app/core/services/fuzzy-search.service.ts`
- Modify: `src/app/core/services/fuzzy-search.service.spec.ts`

**Step 1: Write failing tests for subsequence match**

Add to spec file:

```typescript
describe('subsequence match', () => {
  it('should find subsequence match', () => {
    const sounds = [createMockSound({ id: '1', name: 'barbe bleue' })];
    const results = service.search('bbe', sounds);

    expect(results.length).toBe(1);
    expect(results[0].score).toBeGreaterThan(0);
    expect(results[0].score).toBeLessThan(100);
  });

  it('should return correct indices for subsequence', () => {
    const sounds = [createMockSound({ id: '1', name: 'barbe bleue' })];
    const results = service.search('bbe', sounds);

    // b(0), b(3 or 6), e(4 or 10) - depending on implementation
    expect(results[0].matchedIndices.length).toBe(3);
  });

  it('should score denser subsequences higher', () => {
    const sounds = [
      createMockSound({ id: '1', name: 'abc' }),      // dense: a(0)b(1)c(2)
      createMockSound({ id: '2', name: 'a---b---c' }) // sparse
    ];
    const results = service.search('abc', sounds);

    // First should have exact match (score 100), second subsequence
    const denseResult = results.find(r => r.sound.id === '1');
    const sparseResult = results.find(r => r.sound.id === '2');

    expect(denseResult!.score).toBeGreaterThan(sparseResult!.score);
  });

  it('should prefer exact match over subsequence', () => {
    const sounds = [createMockSound({ id: '1', name: 'barbe' })];
    const results = service.search('barbe', sounds);

    expect(results[0].score).toBe(100); // Exact, not subsequence
  });
});
```

**Step 2: Run test to verify it fails**

Run: `npm test -- --include='**/fuzzy-search.service.spec.ts'`
Expected: FAIL - subsequence tests fail

**Step 3: Update implementation**

Update `search` method and add `subsequenceMatch`:

```typescript
search(query: string, sounds: Sound[]): SearchResult[] {
  const normalizedQuery = this.normalizeText(query);
  if (!normalizedQuery) return [];

  const results: SearchResult[] = [];

  for (const sound of sounds) {
    const displayName = sound.customName || sound.name;
    const normalizedName = this.normalizeText(displayName);

    // Try exact match first (highest priority)
    const exactMatch = this.exactMatch(normalizedQuery, normalizedName, displayName);
    if (exactMatch) {
      results.push({ sound, ...exactMatch });
      continue;
    }

    // Try subsequence match
    const subseqMatch = this.subsequenceMatch(normalizedQuery, normalizedName);
    if (subseqMatch) {
      results.push({ sound, ...subseqMatch });
    }
  }

  return results.sort((a, b) => b.score - a.score);
}

private subsequenceMatch(query: string, text: string): { score: number; matchedIndices: number[] } | null {
  const indices: number[] = [];
  let queryIdx = 0;

  for (let i = 0; i < text.length && queryIdx < query.length; i++) {
    if (text[i] === query[queryIdx]) {
      indices.push(i);
      queryIdx++;
    }
  }

  // All characters must be found
  if (queryIdx !== query.length) return null;

  // Score based on density (how close together the matches are)
  const span = indices[indices.length - 1] - indices[0] + 1;
  const density = query.length / span;
  const score = Math.round(60 + (density * 30)); // 60-90 range

  return { score: Math.min(score, 90), matchedIndices: indices };
}
```

**Step 4: Run test to verify it passes**

Run: `npm test -- --include='**/fuzzy-search.service.spec.ts'`
Expected: PASS

**Step 5: Commit**

```bash
git add src/app/core/services/fuzzy-search.service.ts src/app/core/services/fuzzy-search.service.spec.ts
git commit -m "feat(search): add subsequence matching to FuzzySearchService"
```

---

## Task 3: Add Levenshtein Match to FuzzySearchService

**Files:**
- Modify: `src/app/core/services/fuzzy-search.service.ts`
- Modify: `src/app/core/services/fuzzy-search.service.spec.ts`

**Step 1: Write failing tests for Levenshtein match**

Add to spec file:

```typescript
describe('levenshtein match', () => {
  it('should find match with typo (substitution)', () => {
    const sounds = [createMockSound({ id: '1', name: 'barbe' })];
    const results = service.search('babre', sounds);

    expect(results.length).toBe(1);
    expect(results[0].score).toBeGreaterThan(0);
    expect(results[0].score).toBeLessThan(60); // Lower than subsequence
  });

  it('should find match with missing character', () => {
    const sounds = [createMockSound({ id: '1', name: 'barbe' })];
    const results = service.search('barb', sounds);

    expect(results.length).toBe(1);
  });

  it('should find match with extra character', () => {
    const sounds = [createMockSound({ id: '1', name: 'barbe' })];
    const results = service.search('barbee', sounds);

    expect(results.length).toBe(1);
  });

  it('should not match if too many errors', () => {
    const sounds = [createMockSound({ id: '1', name: 'barbe' })];
    const results = service.search('xxxxx', sounds);

    expect(results.length).toBe(0);
  });

  it('should not use levenshtein for queries under 3 characters', () => {
    const sounds = [createMockSound({ id: '1', name: 'ab' })];
    const results = service.search('ax', sounds);

    // 'ax' vs 'ab' is 1 edit, but query is too short
    expect(results.length).toBe(0);
  });

  it('should not return matched indices for levenshtein', () => {
    const sounds = [createMockSound({ id: '1', name: 'barbe' })];
    const results = service.search('babre', sounds);

    expect(results[0].matchedIndices).toEqual([]);
  });
});
```

**Step 2: Run test to verify it fails**

Run: `npm test -- --include='**/fuzzy-search.service.spec.ts'`
Expected: FAIL - levenshtein tests fail

**Step 3: Update implementation**

Add `levenshteinMatch` and `levenshteinDistance` methods:

```typescript
search(query: string, sounds: Sound[]): SearchResult[] {
  const normalizedQuery = this.normalizeText(query);
  if (!normalizedQuery) return [];

  const results: SearchResult[] = [];

  for (const sound of sounds) {
    const displayName = sound.customName || sound.name;
    const normalizedName = this.normalizeText(displayName);

    // Try exact match first (highest priority)
    const exactMatch = this.exactMatch(normalizedQuery, normalizedName, displayName);
    if (exactMatch) {
      results.push({ sound, ...exactMatch });
      continue;
    }

    // Try subsequence match
    const subseqMatch = this.subsequenceMatch(normalizedQuery, normalizedName);
    if (subseqMatch) {
      results.push({ sound, ...subseqMatch });
      continue;
    }

    // Try levenshtein match (only for queries >= 3 chars)
    if (normalizedQuery.length >= 3) {
      const levMatch = this.levenshteinMatch(normalizedQuery, normalizedName);
      if (levMatch) {
        results.push({ sound, ...levMatch });
      }
    }
  }

  return results.sort((a, b) => b.score - a.score);
}

private levenshteinMatch(query: string, text: string): { score: number; matchedIndices: number[] } | null {
  // Skip very long texts for performance
  if (text.length > 50) return null;

  const distance = this.levenshteinDistance(query, text);

  // Max allowed errors: roughly 1 error per 3 characters, max 3
  const maxErrors = Math.min(3, Math.floor(query.length / 3) + 1);

  if (distance > maxErrors) return null;

  // Score inversely proportional to distance
  const score = Math.max(10, 50 - (distance * 15));

  return { score, matchedIndices: [] }; // No highlighting for levenshtein
}

private levenshteinDistance(a: string, b: string): number {
  const matrix: number[][] = [];

  for (let i = 0; i <= a.length; i++) {
    matrix[i] = [i];
  }
  for (let j = 0; j <= b.length; j++) {
    matrix[0][j] = j;
  }

  for (let i = 1; i <= a.length; i++) {
    for (let j = 1; j <= b.length; j++) {
      const cost = a[i - 1] === b[j - 1] ? 0 : 1;
      matrix[i][j] = Math.min(
        matrix[i - 1][j] + 1,      // deletion
        matrix[i][j - 1] + 1,      // insertion
        matrix[i - 1][j - 1] + cost // substitution
      );
    }
  }

  return matrix[a.length][b.length];
}
```

**Step 4: Run test to verify it passes**

Run: `npm test -- --include='**/fuzzy-search.service.spec.ts'`
Expected: PASS

**Step 5: Commit**

```bash
git add src/app/core/services/fuzzy-search.service.ts src/app/core/services/fuzzy-search.service.spec.ts
git commit -m "feat(search): add Levenshtein matching to FuzzySearchService"
```

---

## Task 4: Add Accent Insensitivity Tests

**Files:**
- Modify: `src/app/core/services/fuzzy-search.service.spec.ts`

**Step 1: Write tests for accent handling**

Add to spec file:

```typescript
describe('accent insensitivity', () => {
  it('should match accented characters with non-accented query', () => {
    const sounds = [createMockSound({ id: '1', name: 'café' })];
    const results = service.search('cafe', sounds);

    expect(results.length).toBe(1);
  });

  it('should match non-accented characters with accented query', () => {
    const sounds = [createMockSound({ id: '1', name: 'cafe' })];
    const results = service.search('café', sounds);

    expect(results.length).toBe(1);
  });

  it('should handle various accents', () => {
    const sounds = [createMockSound({ id: '1', name: 'résumé' })];
    const results = service.search('resume', sounds);

    expect(results.length).toBe(1);
  });
});
```

**Step 2: Run tests**

Run: `npm test -- --include='**/fuzzy-search.service.spec.ts'`
Expected: PASS (already implemented in normalizeText)

**Step 3: Commit**

```bash
git add src/app/core/services/fuzzy-search.service.spec.ts
git commit -m "test(search): add accent insensitivity tests"
```

---

## Task 5: Export FuzzySearchService from Core Services Index

**Files:**
- Modify: `src/app/core/services/index.ts`

**Step 1: Read current index file**

Read the file to see current exports.

**Step 2: Add export**

Add to `src/app/core/services/index.ts`:

```typescript
export { FuzzySearchService, SearchResult } from './fuzzy-search.service';
```

**Step 3: Run all tests to ensure no breaking changes**

Run: `npm test`
Expected: PASS

**Step 4: Commit**

```bash
git add src/app/core/services/index.ts
git commit -m "chore: export FuzzySearchService from core services"
```

---

## Task 6: Add searchQuery Signal to SoundboardService

**Files:**
- Modify: `src/app/core/services/soundboard.service.ts`
- Modify: `src/app/core/services/soundboard.service.spec.ts`

**Step 1: Write failing tests**

Add to spec file:

```typescript
describe('search', () => {
  it('should have searchQuery signal initialized to empty string', () => {
    expect(service.searchQuery()).toBe('');
  });

  it('should update searchQuery when setSearchQuery is called', () => {
    service.setSearchQuery('test');
    expect(service.searchQuery()).toBe('test');
  });

  it('should clear searchQuery when setActiveFolder is called', () => {
    service.setSearchQuery('test');
    service.setActiveFolder('all');
    expect(service.searchQuery()).toBe('');
  });
});
```

**Step 2: Run test to verify it fails**

Run: `npm test -- --include='**/soundboard.service.spec.ts'`
Expected: FAIL - searchQuery not found

**Step 3: Add to soundboard.service.ts**

Add after line 22 (after `_error` signal):

```typescript
private _searchQuery = signal<string>('');
```

Add after line 39 (after `error` readonly):

```typescript
readonly searchQuery = this._searchQuery.asReadonly();
```

Add method after `setActiveFolder` (around line 448):

```typescript
setSearchQuery(query: string): void {
  this._searchQuery.set(query);
}
```

Modify `setActiveFolder` to clear search:

```typescript
setActiveFolder(folderId: string): void {
  if (this._folders().some(f => f.id === folderId)) {
    this._activeFolderId.set(folderId);
    this._searchQuery.set(''); // Clear search on folder change
  }
}
```

**Step 4: Run test to verify it passes**

Run: `npm test -- --include='**/soundboard.service.spec.ts'`
Expected: PASS

**Step 5: Commit**

```bash
git add src/app/core/services/soundboard.service.ts src/app/core/services/soundboard.service.spec.ts
git commit -m "feat(search): add searchQuery signal to SoundboardService"
```

---

## Task 7: Integrate Search into Pads Computed

**Files:**
- Modify: `src/app/core/services/soundboard.service.ts`
- Modify: `src/app/core/services/soundboard.service.spec.ts`

**Step 1: Write failing tests**

Add to spec file:

```typescript
describe('pads with search', () => {
  beforeEach(() => {
    const sounds = [
      createMockSound({ id: '1', name: 'barbe bleue' }),
      createMockSound({ id: '2', name: 'chat noir' }),
      createMockSound({ id: '3', name: 'barbare' })
    ];
    const soundsMap = new Map(sounds.map(s => [s.id, s]));
    (service as any)._sounds.set(soundsMap);
  });

  it('should filter pads when searchQuery is set', () => {
    service.setSearchQuery('barb');

    const padsWithSounds = service.pads().filter(p => p.sound !== null);
    expect(padsWithSounds.length).toBe(2); // barbe bleue, barbare
  });

  it('should sort pads by search score', () => {
    service.setSearchQuery('barb');

    const padsWithSounds = service.pads().filter(p => p.sound !== null);
    // Both should match, order depends on score
    expect(padsWithSounds.every(p => p.sound!.name.includes('barb'))).toBeTrue();
  });

  it('should show all pads when searchQuery is empty', () => {
    service.setSearchQuery('');

    const padsWithSounds = service.pads().filter(p => p.sound !== null);
    expect(padsWithSounds.length).toBe(3);
  });
});
```

**Step 2: Run test to verify it fails**

Run: `npm test -- --include='**/soundboard.service.spec.ts'`
Expected: FAIL - search not filtering

**Step 3: Update pads computed**

Import FuzzySearchService and inject it:

```typescript
import { FuzzySearchService } from './fuzzy-search.service';

// In constructor
constructor(
  private tauri: TauriService,
  private fuzzySearch: FuzzySearchService
) {
  // ...
}
```

Update `pads` computed (replace the existing one around line 56):

```typescript
readonly pads = computed(() => {
  const sounds = this._sounds();
  const activeFolderId = this._activeFolderId();
  const searchQuery = this._searchQuery();

  // Filter sounds by folder
  let filteredSounds = Array.from(sounds.values());
  if (activeFolderId !== 'all') {
    filteredSounds = filteredSounds.filter(s => s.folderIds.includes(activeFolderId));
  }

  // Apply search filter
  let sortedSounds: Array<{ sound: Sound; matchedIndices: number[] }>;

  if (searchQuery.trim()) {
    const searchResults = this.fuzzySearch.search(searchQuery, filteredSounds);
    sortedSounds = searchResults.map(r => ({
      sound: r.sound,
      matchedIndices: r.matchedIndices
    }));
  } else {
    // No search: sort alphabetically
    filteredSounds.sort((a, b) =>
      (a.customName || a.name).toLowerCase()
        .localeCompare((b.customName || b.name).toLowerCase())
    );
    sortedSounds = filteredSounds.map(s => ({ sound: s, matchedIndices: [] }));
  }

  // Generate virtual grid
  const minPads = Math.max(12, Math.ceil(sortedSounds.length / 4) * 4 + 4);
  const pads: SoundPad[] = [];

  for (let i = 0; i < minPads; i++) {
    const item = sortedSounds[i];
    pads.push({
      index: i,
      sound: item?.sound || null,
      color: PAD_COLORS[i % PAD_COLORS.length],
      matchedIndices: item?.matchedIndices || []
    });
  }

  return pads;
});
```

**Step 4: Run test to verify it passes**

Run: `npm test -- --include='**/soundboard.service.spec.ts'`
Expected: PASS

**Step 5: Commit**

```bash
git add src/app/core/services/soundboard.service.ts src/app/core/services/soundboard.service.spec.ts
git commit -m "feat(search): integrate fuzzy search into pads computed"
```

---

## Task 8: Update SoundPad Model to Include matchedIndices

**Files:**
- Modify: `src/app/core/models/audio-device.model.ts`

**Step 1: Update SoundPad interface**

Add `matchedIndices` to `SoundPad` interface:

```typescript
export interface SoundPad {
  /** Position in grid (0, 1, 2, ...) */
  index: number;
  /** Reference to sound or null if empty */
  sound: Sound | null;
  /** Color generated from index */
  color: string;
  /** Indices of characters to highlight in search results */
  matchedIndices: number[];
}
```

**Step 2: Run all tests to check for breaking changes**

Run: `npm test`
Expected: Some tests may fail due to missing matchedIndices in mocks

**Step 3: Update any failing test mocks**

If tests fail, add `matchedIndices: []` to SoundPad mocks.

**Step 4: Commit**

```bash
git add src/app/core/models/audio-device.model.ts
git commit -m "feat(search): add matchedIndices to SoundPad model"
```

---

## Task 9: Create SearchBarComponent

**Files:**
- Create: `src/app/features/soundboard/search-bar/search-bar.component.ts`

**Step 1: Create the component**

```typescript
// src/app/features/soundboard/search-bar/search-bar.component.ts
import { Component, ElementRef, ViewChild, inject, Output, EventEmitter } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { SoundboardService } from '../../../core/services/soundboard.service';

@Component({
  selector: 'app-search-bar',
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
    <div class="relative">
      <div class="absolute left-3 top-1/2 -translate-y-1/2 text-text-muted pointer-events-none">
        &#128269;
      </div>
      <input
        #searchInput
        type="text"
        [ngModel]="soundboard.searchQuery()"
        (ngModelChange)="onQueryChange($event)"
        (keydown.escape)="onEscape($event)"
        placeholder="Search sounds..."
        class="w-full pl-10 pr-10 py-2 text-sm bg-surface-hover border border-border rounded-lg
               text-text-primary placeholder:text-text-muted
               focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/50
               transition-colors"
      />
      @if (soundboard.searchQuery()) {
        <button
          class="absolute right-2 top-1/2 -translate-y-1/2 w-6 h-6 flex items-center justify-center
                 text-text-muted hover:text-text-primary rounded-full hover:bg-surface-hover
                 transition-colors"
          (click)="clear()"
          title="Clear search"
        >
          &#10005;
        </button>
      }
    </div>
  `
})
export class SearchBarComponent {
  @ViewChild('searchInput') searchInput!: ElementRef<HTMLInputElement>;
  @Output() cleared = new EventEmitter<void>();

  soundboard = inject(SoundboardService);

  private debounceTimer: ReturnType<typeof setTimeout> | null = null;

  focus(): void {
    this.searchInput?.nativeElement.focus();
  }

  onQueryChange(value: string): void {
    // Debounce input
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }
    this.debounceTimer = setTimeout(() => {
      this.soundboard.setSearchQuery(value);
    }, 150);
  }

  onEscape(event: Event): void {
    event.stopPropagation();
    this.clear();
    this.searchInput?.nativeElement.blur();
  }

  clear(): void {
    this.soundboard.setSearchQuery('');
    this.cleared.emit();
  }
}
```

**Step 2: Run lint/build to verify no errors**

Run: `npm run build`
Expected: PASS

**Step 3: Commit**

```bash
git add src/app/features/soundboard/search-bar/search-bar.component.ts
git commit -m "feat(search): create SearchBarComponent"
```

---

## Task 10: Integrate SearchBar into SoundboardComponent

**Files:**
- Modify: `src/app/features/soundboard/soundboard.component.ts`

**Step 1: Import and add SearchBarComponent**

Add import:

```typescript
import { SearchBarComponent } from './search-bar/search-bar.component';
```

Add to imports array:

```typescript
imports: [CommonModule, SoundPadComponent, ImageSuggestionToastComponent, BulkImageWizardComponent, SearchBarComponent],
```

Add ViewChild:

```typescript
@ViewChild(SearchBarComponent) searchBar!: SearchBarComponent;
```

**Step 2: Add search bar to template**

After the header div (around line 34), add:

```html
<!-- Search bar -->
<div class="mb-4">
  <app-search-bar />
</div>
```

**Step 3: Add Ctrl+F handler**

Add to `handleKeydown` method (or create new HostListener):

```typescript
@HostListener('window:keydown.control.f', ['$event'])
handleCtrlF(event: KeyboardEvent): void {
  event.preventDefault();
  this.searchBar?.focus();
}
```

**Step 4: Run build and manual test**

Run: `npm run build && npm start`
Expected: Search bar appears, Ctrl+F focuses it

**Step 5: Commit**

```bash
git add src/app/features/soundboard/soundboard.component.ts
git commit -m "feat(search): integrate SearchBar into SoundboardComponent"
```

---

## Task 11: Add No Results Message

**Files:**
- Modify: `src/app/features/soundboard/soundboard.component.ts`

**Step 1: Add computed for no results state**

Add to component class:

```typescript
hasNoResults = computed(() => {
  const query = this.soundboard.searchQuery();
  const padsWithSounds = this.soundboard.pads().filter(p => p.sound !== null);
  return query.trim().length > 0 && padsWithSounds.length === 0;
});
```

**Step 2: Add message to template**

After the pads grid, add:

```html
<!-- No results message -->
@if (hasNoResults()) {
  <div class="flex flex-col items-center justify-center py-12 text-text-muted">
    <span class="text-4xl mb-2">&#128269;</span>
    <p class="text-sm">No sounds found for "{{ soundboard.searchQuery() }}"</p>
    <button
      class="mt-3 text-xs text-accent hover:underline"
      (click)="soundboard.setSearchQuery('')"
    >
      Clear search
    </button>
  </div>
}
```

**Step 3: Run build and manual test**

Run: `npm run build && npm start`
Expected: "No results" message appears when search yields no matches

**Step 4: Commit**

```bash
git add src/app/features/soundboard/soundboard.component.ts
git commit -m "feat(search): add no results message"
```

---

## Task 12: Add Character Highlighting to SoundPadComponent

**Files:**
- Modify: `src/app/features/soundboard/sound-pad/sound-pad.component.ts`

**Step 1: Add highlighting logic**

Add method to component:

```typescript
getHighlightedName(): { text: string; highlighted: boolean }[] {
  const sound = this.pad.sound;
  if (!sound) return [];

  const displayName = sound.customName || sound.name;
  const indices = new Set(this.pad.matchedIndices || []);

  if (indices.size === 0) {
    return [{ text: displayName, highlighted: false }];
  }

  const segments: { text: string; highlighted: boolean }[] = [];
  let currentSegment = '';
  let currentHighlighted = indices.has(0);

  for (let i = 0; i < displayName.length; i++) {
    const isHighlighted = indices.has(i);
    if (isHighlighted !== currentHighlighted) {
      if (currentSegment) {
        segments.push({ text: currentSegment, highlighted: currentHighlighted });
      }
      currentSegment = displayName[i];
      currentHighlighted = isHighlighted;
    } else {
      currentSegment += displayName[i];
    }
  }

  if (currentSegment) {
    segments.push({ text: currentSegment, highlighted: currentHighlighted });
  }

  return segments;
}
```

**Step 2: Update template**

Replace the name display (around line 49-51):

```html
<span class="block text-xs font-semibold text-white truncate mb-0.5 drop-shadow-[0_2px_4px_rgba(0,0,0,0.8)]">
  @for (segment of getHighlightedName(); track $index) {
    @if (segment.highlighted) {
      <span class="text-accent font-bold">{{ segment.text }}</span>
    } @else {
      <span>{{ segment.text }}</span>
    }
  }
</span>
```

**Step 3: Run build and manual test**

Run: `npm run build && npm start`
Expected: Matched characters are highlighted in accent color

**Step 4: Commit**

```bash
git add src/app/features/soundboard/sound-pad/sound-pad.component.ts
git commit -m "feat(search): add character highlighting to SoundPadComponent"
```

---

## Task 13: Run Full Test Suite

**Files:** None (verification only)

**Step 1: Run all tests**

Run: `npm test`
Expected: All tests pass

**Step 2: Run lint**

Run: `npm run lint` (if available) or check for errors in build

**Step 3: Fix any issues**

If tests fail, investigate and fix.

**Step 4: Final commit if fixes were needed**

```bash
git add -A
git commit -m "fix: address test failures"
```

---

## Task 14: Manual Testing Checklist

**Verification steps:**

1. [ ] Search bar is visible above the soundboard grid
2. [ ] Typing filters sounds in real-time (with slight debounce)
3. [ ] Ctrl+F focuses the search bar
4. [ ] Escape clears search and removes focus
5. [ ] X button clears search
6. [ ] Changing folder clears search
7. [ ] Exact matches appear first, with characters highlighted
8. [ ] Subsequence matches work (e.g., "bb" finds "barbe bleue")
9. [ ] Typo tolerance works (e.g., "babre" finds "barbe")
10. [ ] "No results" message appears when appropriate
11. [ ] Case insensitive search works
12. [ ] Accent insensitive search works (e.g., "cafe" finds "café")

---

## Summary

| Task | Description | Est. Steps |
|------|-------------|------------|
| 1 | FuzzySearchService with exact match | 5 |
| 2 | Add subsequence match | 5 |
| 3 | Add Levenshtein match | 5 |
| 4 | Accent insensitivity tests | 3 |
| 5 | Export service from index | 4 |
| 6 | Add searchQuery signal | 5 |
| 7 | Integrate search into pads | 5 |
| 8 | Update SoundPad model | 4 |
| 9 | Create SearchBarComponent | 3 |
| 10 | Integrate SearchBar | 5 |
| 11 | Add no results message | 4 |
| 12 | Add character highlighting | 4 |
| 13 | Run full test suite | 3 |
| 14 | Manual testing | 1 |

Total: 14 tasks, ~56 steps
