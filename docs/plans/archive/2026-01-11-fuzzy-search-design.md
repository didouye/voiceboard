# Fuzzy Search Bar for Soundboard

## Overview

Add a search bar above the soundboard grid that filters sounds using fuzzy matching. The search tolerates typos (e.g., "babre" matches "barbe") and provides visual feedback by highlighting matched characters.

## User Experience

### Search Bar Placement
- Always visible above the soundboard grid, below the folder header
- Text input with search icon (left) and clear button X (right, visible when text present)
- Placeholder: "Search sounds..." or localized equivalent

### Interaction
- **Ctrl+F**: Focus the search bar from anywhere in the soundboard
- **Typing**: Instant filtering with debounce (150-200ms)
- **Escape**: Clear search and remove focus
- **X button**: Clear search
- **Folder change**: Auto-clear the search field

### Search Behavior
- Searches on **displayed name only** (`customName` if set, otherwise `name`)
- **Case-insensitive**: "BARBE" matches "barbe"
- **Accent-insensitive**: "cafe" matches "café"

### Results Display
- Non-matching sounds are hidden from the grid
- Matching sounds are sorted by relevance score (best matches first)
- Matched characters are highlighted in the sound name
- Empty state: "No results found" message when no sounds match

## Fuzzy Matching Algorithm

Three strategies applied in priority order. Each sound receives the best score among all strategies.

### 1. Exact Match (Score: 100)
- Search term appears as-is within the name
- Example: "bar" in "Barbe bleue" → score 100

### 2. Subsequence Match (Score: 60-90)
- Characters appear in order but not necessarily consecutive
- Score based on character density (closer = higher score)
- Example: "bbe" matches "**B**ar**b**e bl**e**ue" → score ~75

### 3. Levenshtein Distance (Score: 10-50)
- Tolerates substitutions, insertions, deletions
- Score inversely proportional to edit distance
- Maximum threshold: 2-3 errors depending on word length
- Disabled for queries < 3 characters (too many false positives)
- Example: "babre" matches "barbe" (1 transposition) → score ~40

### Scoring & Sorting
- Sounds with score 0 are excluded from results
- Results sorted by score descending, then alphabetically for equal scores

## Highlighting

- **Exact match**: Highlight the entire matched substring
- **Subsequence**: Highlight each matched character individually
- **Levenshtein**: No highlighting (characters don't directly correspond)

Visual example:
```
Search: "bar"
Results:
  [BAR]be bleue     (exact - "bar" highlighted)
  [B]l[A]ck [R]ose  (subsequence - b, a, r highlighted)
  Sabre             (levenshtein - no highlight)
```

## Technical Implementation

### New Service: FuzzySearchService

```typescript
interface SearchResult {
  sound: Sound;
  score: number;
  matchedIndices: number[]; // character indices to highlight
}

class FuzzySearchService {
  search(query: string, sounds: Sound[]): SearchResult[];
  getDisplayName(sound: Sound): string;

  private exactMatch(query: string, text: string): { score: number; indices: number[] } | null;
  private subsequenceMatch(query: string, text: string): { score: number; indices: number[] } | null;
  private levenshteinMatch(query: string, text: string): { score: number } | null;
  private normalizeText(text: string): string; // lowercase + remove accents
}
```

### SoundboardService Changes

```typescript
// New signal
searchQuery = signal<string>('');

// New method
setSearchQuery(query: string): void {
  this.searchQuery.set(query);
}

// Modified pads computed - integrate search filtering
pads = computed(() => {
  let sounds = this.getFilteredSoundsByFolder();

  const query = this.searchQuery();
  if (query.trim()) {
    const results = this.fuzzySearch.search(query, sounds);
    // Returns sounds sorted by score with matchedIndices
    return this.generatePadsFromSearchResults(results);
  }

  return this.generatePads(sounds);
});
```

### SoundPadComponent Changes

```typescript
// New input for highlighting
@Input() matchedIndices?: number[];

// Template uses matchedIndices to render highlighted name
```

### SearchBarComponent (New)

```typescript
@Component({
  selector: 'app-search-bar',
  template: `
    <div class="search-bar">
      <mat-icon>search</mat-icon>
      <input
        #searchInput
        type="text"
        [placeholder]="'Search sounds...'"
        [value]="query()"
        (input)="onInput($event)"
        (keydown.escape)="onEscape()"
      />
      <button *ngIf="query()" (click)="clear()">
        <mat-icon>close</mat-icon>
      </button>
    </div>
  `
})
export class SearchBarComponent {
  @ViewChild('searchInput') searchInput: ElementRef;

  query = inject(SoundboardService).searchQuery;

  focus(): void {
    this.searchInput.nativeElement.focus();
  }

  onEscape(): void {
    this.clear();
    this.searchInput.nativeElement.blur();
  }

  clear(): void {
    inject(SoundboardService).setSearchQuery('');
  }
}
```

### Keyboard Shortcut (Ctrl+F)

In `SoundboardComponent`:
```typescript
@HostListener('document:keydown.control.f', ['$event'])
onCtrlF(event: KeyboardEvent): void {
  event.preventDefault(); // Prevent browser find
  this.searchBar.focus();
}
```

### Auto-clear on Folder Change

In `SoundboardService.setActiveFolder()`:
```typescript
setActiveFolder(folderId: string): void {
  this._activeFolderId.set(folderId);
  this.searchQuery.set(''); // Clear search
}
```

## Edge Cases

| Case | Behavior |
|------|----------|
| Empty/whitespace query | Show all sounds (no filtering) |
| Special characters | Treated literally, no regex |
| Query 1-2 chars | Exact + subsequence only (no Levenshtein) |
| Very long names (>50 chars) | Skip Levenshtein for performance |
| No results | Display "No results found" message |

## Performance Considerations

- **Debounce**: 150-200ms on input to avoid excessive recalculation
- **Levenshtein limit**: Only for names < 50 characters
- **Expected scale**: Hundreds of sounds, filtering remains instant

## Testing

### Unit Tests (FuzzySearchService)
- Exact match detection and scoring
- Subsequence match detection, scoring, and index calculation
- Levenshtein match detection and scoring
- Case insensitivity
- Accent insensitivity
- Special characters handling
- Combined scoring and sorting

### Integration Tests
- Ctrl+F focuses search bar
- Typing filters sounds in real-time
- Escape clears search and removes focus
- X button clears search
- Folder change clears search
- Matched characters are highlighted correctly
- "No results" message displays when appropriate

## Files to Create/Modify

### Create
- `src/app/services/fuzzy-search.service.ts`
- `src/app/services/fuzzy-search.service.spec.ts`
- `src/app/components/search-bar/search-bar.component.ts`
- `src/app/components/search-bar/search-bar.component.spec.ts`

### Modify
- `src/app/services/soundboard.service.ts` - Add searchQuery signal
- `src/app/components/soundboard/soundboard.component.ts` - Add Ctrl+F handler, integrate search bar
- `src/app/components/sound-pad/sound-pad.component.ts` - Add matchedIndices input and highlighting
