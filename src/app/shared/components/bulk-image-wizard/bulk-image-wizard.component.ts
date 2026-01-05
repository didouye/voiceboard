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
            Search for images
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
