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
