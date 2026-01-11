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
      <div class="relative bg-surface border border-border rounded-xl p-6 w-full max-w-2xl animate-scale-in" (click)="$event.stopPropagation()">
        <!-- Header -->
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-lg font-semibold text-text-primary">
            Image pour "{{ currentSoundName() }}" ({{ currentIndex() + 1 }}/{{ pads.length }})
          </h2>
          <button
            class="text-text-muted hover:text-text-primary"
            (click)="skip()"
          >
            Passer
          </button>
        </div>

        <!-- Search -->
        <div class="flex gap-2 mb-4">
          <input
            type="text"
            [(ngModel)]="searchQuery"
            (keydown.enter)="search()"
            placeholder="Rechercher des images..."
            class="flex-1 px-3 py-2 text-sm bg-surface-hover border border-border rounded text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent"
          >
          <button
            class="px-4 py-2 text-sm bg-accent hover:bg-accent/80 text-white rounded transition-colors"
            (click)="search()"
          >
            Chercher
          </button>
        </div>

        <!-- Selected Image Preview -->
        <div class="mb-4">
          <div class="w-36 h-36 mx-auto rounded-xl overflow-hidden border-2 transition-all"
               [class]="selectedImage() ? 'border-accent bg-surface-hover' : 'border-dashed border-border bg-surface-hover'">
            @if (selectedImage()) {
              <img [src]="selectedImage()!.thumbnailUrl" alt="" class="w-full h-full object-cover">
            } @else {
              <div class="w-full h-full flex flex-col items-center justify-center text-text-muted">
                <span class="text-3xl mb-1">&#128247;</span>
                <span class="text-xs">Sélectionnez une image</span>
              </div>
            }
          </div>
        </div>

        <!-- Results Grid -->
        @if (searchResults().length > 0) {
          <div class="flex flex-wrap justify-center gap-2 mb-4 max-h-64 overflow-y-auto p-1">
            @for (result of searchResults(); track result.id) {
              <button
                class="w-16 h-16 rounded-lg overflow-hidden border-2 transition-all hover:scale-105 flex-shrink-0"
                [class]="selectedImage()?.id === result.id ? 'border-accent ring-2 ring-accent/50' : 'border-transparent hover:border-border'"
                (click)="selectedImage.set(result)"
              >
                <img [src]="result.thumbnailUrl" alt="" class="w-full h-full object-cover">
              </button>
            }
          </div>
        } @else {
          <div class="h-32 flex items-center justify-center text-text-muted">
            Recherchez des images
          </div>
        }

        <!-- Navigation -->
        <div class="flex items-center justify-between pt-4 border-t border-border">
          <button
            class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
            [disabled]="currentIndex() === 0"
            (click)="previous()"
          >
            &larr; Précédent
          </button>

          <div class="flex gap-2">
            @if (selectedImage()) {
              <button
                class="px-4 py-2 text-sm bg-accent hover:bg-accent/80 text-white rounded transition-colors"
                (click)="selectAndNext()"
              >
                {{ isLast() ? 'Terminer' : 'Valider & Suivant' }}
              </button>
            }
            <button
              class="px-4 py-2 text-sm bg-surface-hover hover:bg-border text-text-secondary rounded transition-colors"
              (click)="nextOrFinish()"
            >
              {{ isLast() ? 'Terminer' : 'Suivant &rarr;' }}
            </button>
          </div>
        </div>

        <!-- Finish button -->
        <button
          class="w-full mt-4 py-2 text-sm text-text-muted hover:text-text-primary transition-colors"
          (click)="finish()"
        >
          Terminer et garder les images sélectionnées
        </button>
      </div>
    </div>
  `
})
export class BulkImageWizardComponent implements OnInit {
  @Input({ required: true }) pads!: SoundPad[];
  @Output() close = new EventEmitter<void>();
  @Output() selectImage = new EventEmitter<{ soundId: string; image: ImageSearchResult }>();

  private imageSearch = inject(ImageSearchService);

  currentIndex = signal(0);
  searchQuery = '';
  searchResults = signal<ImageSearchResult[]>([]);
  selectedImage = signal<ImageSearchResult | null>(null);
  selections = new Map<string, ImageSearchResult>();

  readonly currentPad = () => this.pads[this.currentIndex()];
  readonly currentSoundName = () => this.currentPad()?.sound?.name || '';
  readonly currentSoundId = () => this.currentPad()?.sound?.id || '';
  readonly isLast = () => this.currentIndex() === this.pads.length - 1;

  async ngOnInit(): Promise<void> {
    await this.loadCurrentPad();
  }

  private async loadCurrentPad(): Promise<void> {
    const pad = this.currentPad();
    if (!pad?.sound) return;

    // Check if we already have a selection for this pad
    const existingSelection = this.selections.get(pad.sound.id);
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
    const soundId = this.currentSoundId();
    if (soundId) {
      this.selections.delete(soundId);
    }
    this.nextOrFinish();
  }

  selectAndNext(): void {
    const image = this.selectedImage();
    const pad = this.currentPad();
    if (image && pad?.sound) {
      this.selections.set(pad.sound.id, image);
      this.selectImage.emit({ soundId: pad.sound.id, image });
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
