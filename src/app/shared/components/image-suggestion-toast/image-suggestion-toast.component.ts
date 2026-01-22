import {
  Component,
  Input,
  Output,
  EventEmitter,
  signal,
  inject,
  OnInit,
} from "@angular/core";
import { CommonModule } from "@angular/common";
import { FormsModule } from "@angular/forms";
import {
  ImageSearchService,
  ImageSearchResult,
} from "../../../core/services/image-search.service";

@Component({
  selector: "app-image-suggestion-toast",
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
    <div class="fixed bottom-4 right-4 z-50 animate-slide-in-up">
      <div
        class="bg-surface border border-border rounded-xl shadow-xl p-4 w-80"
      >
        @if (!expanded()) {
          <!-- Compact view -->
          <div class="flex items-center gap-3">
            @if (suggestedImage()) {
              <img
                [src]="suggestedImage()!.thumbnailUrl"
                alt=""
                class="w-12 h-12 rounded object-cover"
              />
            } @else {
              <div
                class="w-12 h-12 rounded bg-surface-hover flex items-center justify-center"
              >
                <span class="text-text-muted">&#128247;</span>
              </div>
            }
            <div class="flex-1 min-w-0">
              <p class="text-sm text-text-primary truncate">
                Image for "{{ soundName }}"
              </p>
              <p class="text-xs text-text-muted">
                {{ suggestedImage() ? "Suggestion found" : "No results" }}
              </p>
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
            <p class="text-sm text-text-primary mb-2">
              Image for "{{ soundName }}"
            </p>
            <div class="flex gap-2">
              <input
                type="text"
                [(ngModel)]="searchQuery"
                (keydown.enter)="search()"
                placeholder="Search..."
                class="flex-1 px-3 py-2 text-sm bg-surface-hover border border-border rounded text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent"
              />
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
                  [class]="
                    selectedResult()?.id === result.id
                      ? 'border-accent'
                      : 'border-transparent'
                  "
                  (click)="selectedResult.set(result)"
                >
                  <img
                    [src]="result.thumbnailUrl"
                    alt=""
                    class="w-full h-full object-cover"
                  />
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
  `,
})
export class ImageSuggestionToastComponent implements OnInit {
  @Input({ required: true }) soundName!: string;
  @Input({ required: true }) filename!: string;
  @Output() accept = new EventEmitter<ImageSearchResult>();
  @Output() ignore = new EventEmitter<void>();

  private imageSearch = inject(ImageSearchService);

  expanded = signal(false);
  searchQuery = "";
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
      console.error("Search failed:", err);
    }
  }
}
