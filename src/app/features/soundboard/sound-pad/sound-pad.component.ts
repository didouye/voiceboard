import { Component, Input, Output, EventEmitter, HostListener, HostBinding, inject, OnInit, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { SoundPad, PadImage } from '../../../core/models';
import { SoundboardService } from '../../../core/services/soundboard.service';
import { ShortcutService } from '../../../core/services/shortcut.service';
import { TauriService } from '../../../core/services/tauri.service';
import { ImageSearchService, ImageSearchResult } from '../../../core/services/image-search.service';
import { convertFileSrc } from '@tauri-apps/api/core';

@Component({
  selector: 'app-sound-pad',
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
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
          [src]="imageUrl()"
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

        <!-- Playing indicator -->
        @if (pad.isPlaying) {
          <div class="absolute bottom-2 left-1/2 -translate-x-1/2 flex gap-0.5 items-end h-4">
            <span class="w-1 bg-white rounded-sm animate-[soundbar_0.4s_ease-in-out_infinite_alternate]" style="height: 8px"></span>
            <span class="w-1 bg-white rounded-sm animate-[soundbar_0.4s_ease-in-out_infinite_alternate_0.1s]" style="height: 14px"></span>
            <span class="w-1 bg-white rounded-sm animate-[soundbar_0.4s_ease-in-out_infinite_alternate_0.2s]" style="height: 10px"></span>
          </div>
        }

        <!-- Action buttons (hidden when modal is open) -->
        <div class="absolute top-2 right-2 flex gap-1 transition-opacity"
             [class]="showSettingsPopup ? 'opacity-0 pointer-events-none' : 'opacity-0 group-hover:opacity-100'">
          <button
            class="w-6 h-6 rounded-full flex items-center justify-center text-[10px] transition-colors"
            [class]="pad.volume !== 1.0 ? 'bg-status-warning text-black' : 'bg-black/50 text-white hover:bg-accent'"
            (click)="toggleSettingsPopup($event)"
            title="Settings"
          >
            &#9881;
          </button>
          <button
            class="w-6 h-6 bg-black/50 hover:bg-status-success rounded-full flex items-center justify-center text-[10px] text-white transition-colors"
            [class.bg-status-info]="isPreviewing"
            (click)="onPreview($event)"
            [title]="isPreviewing ? 'Stop preview' : 'Preview'"
          >
            {{ isPreviewing ? '&#9632;' : '&#9654;' }}
          </button>
          <button
            class="w-6 h-6 bg-black/50 hover:bg-status-error rounded-full flex items-center justify-center text-xs text-white transition-colors"
            (click)="onRemove($event)"
            title="Remove"
          >
            &times;
          </button>
        </div>
      } @else {
        <!-- Empty pad -->
        <div class="flex flex-col items-center text-text-muted group-hover:text-text-secondary transition-colors">
          <span class="text-3xl font-light">+</span>
          <span class="text-[10px] uppercase tracking-wide">Import</span>
        </div>
      }
    </div>

    <!-- Settings modal (covers pad grid area only, leaving sidebar and status bar visible) -->
    @if (showSettingsPopup && pad.sound) {
      <div
        class="fixed top-0 right-0 bottom-14 left-48 z-50 flex items-center justify-center transition-opacity duration-150 p-6"
        [class]="modalVisible ? 'opacity-100' : 'opacity-0'"
        (click)="closePopup($event)"
      >
        <!-- Dark backdrop -->
        <div class="absolute inset-0 bg-black/60 backdrop-blur-sm rounded-lg"></div>

        <!-- Modal content - fills available space with max width -->
        <div
          class="relative bg-surface border border-border rounded-xl p-6 w-full max-w-lg max-h-full overflow-y-auto shadow-xl transition-all duration-150"
          [class]="modalVisible ? 'opacity-100 scale-100' : 'opacity-0 scale-95'"
          (click)="$event.stopPropagation()"
        >
          <!-- Header -->
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-sm font-semibold text-text-primary truncate">{{ pad.sound.name }}</h3>
            <button
              class="w-6 h-6 flex items-center justify-center rounded hover:bg-surface-hover text-text-secondary hover:text-text-primary transition-colors"
              (click)="closePopup($event)"
            >
              &#10005;
            </button>
          </div>

          <!-- Custom Name -->
          <div class="mb-4">
            <div class="flex justify-between items-center mb-2 text-xs">
              <span class="text-text-secondary">Name</span>
            </div>
            <input
              type="text"
              [ngModel]="pad.customName || ''"
              (ngModelChange)="onCustomNameChange($event)"
              [placeholder]="pad.sound.name"
              class="w-full px-3 py-2 text-sm bg-surface-hover border border-border rounded text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent"
            >
          </div>

          <!-- Image -->
          <div class="mb-4 pt-4 border-t border-border">
            <div class="flex justify-between items-center mb-2 text-xs">
              <span class="text-text-secondary">Image</span>
            </div>

            <!-- Preview -->
            <div class="flex items-center gap-3 mb-3">
              <div class="w-16 h-16 rounded-lg overflow-hidden bg-surface-hover flex items-center justify-center border border-border">
                @if (imageUrl()) {
                  <img [src]="imageUrl()" alt="" class="w-full h-full object-cover">
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
                  [disabled]="imageLoading()"
                >
                  Upload
                </button>
                <button
                  class="px-3 py-1.5 text-xs bg-surface-hover hover:bg-border rounded transition-colors text-text-secondary hover:text-text-primary"
                  (click)="showImageSearch = !showImageSearch"
                  [disabled]="imageLoading()"
                >
                  {{ showImageSearch ? 'Close search' : 'Search' }}
                </button>
                @if (pad.image) {
                  <button
                    class="px-3 py-1.5 text-xs text-status-error hover:bg-status-error/10 rounded transition-colors"
                    (click)="removeImage()"
                    [disabled]="imageLoading()"
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
                        [disabled]="imageLoading()"
                      >
                        <img [src]="result.thumbnailUrl" alt="" class="w-full h-full object-cover">
                      </button>
                    }
                  </div>
                } @else if (!imageLoading()) {
                  <p class="text-xs text-text-muted text-center py-4">
                    Search for images
                  </p>
                }
              </div>
            }
          </div>

          <!-- Volume -->
          <div class="mb-4">
            <div class="flex justify-between items-center mb-2 text-xs">
              <span class="text-text-secondary">Volume</span>
              <span class="text-text-primary font-semibold">{{ Math.round(pad.volume * 100) }}%</span>
            </div>
            <input
              type="range"
              [ngModel]="pad.volume"
              (ngModelChange)="onVolumeChange($event)"
              min="0" max="2" step="0.05"
              class="w-full h-1.5 bg-surface-hover rounded-full appearance-none cursor-pointer
                     [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3
                     [&::-webkit-slider-thumb]:bg-white [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:cursor-pointer"
            >
            <div class="flex justify-between text-[10px] text-text-muted mt-1">
              <span>0%</span>
              <span>100%</span>
              <span>200%</span>
            </div>
          </div>

          <!-- Speed -->
          <div class="mb-4 pt-4 border-t border-border">
            <div class="flex justify-between items-center mb-2 text-xs">
              <span class="text-text-secondary">Speed</span>
              <span class="text-text-primary font-semibold">{{ pad.speed }}x</span>
            </div>
            <div class="flex flex-wrap gap-1">
              @for (s of speedOptions; track s) {
                <button
                  class="flex-1 min-w-[40px] px-2 py-1.5 text-xs rounded border transition-colors"
                  [class]="pad.speed === s
                    ? 'bg-accent border-accent text-white'
                    : 'bg-surface-hover border-border text-text-secondary hover:text-text-primary'"
                  (click)="onSpeedChange(s)"
                >
                  {{ s }}x
                </button>
              }
            </div>
          </div>

          <!-- Shortcut -->
          <div class="mb-4 pt-4 border-t border-border">
            <div class="flex justify-between items-center mb-2 text-xs">
              <span class="text-text-secondary">Shortcut</span>
            </div>
            <div class="flex gap-1">
              <button
                class="flex-1 px-3 py-2 text-xs rounded border transition-colors text-left"
                [class]="isRecording
                  ? 'bg-accent border-accent text-white animate-pulse'
                  : 'bg-surface-hover border-border text-text-primary hover:border-text-muted'"
                (click)="startRecording($event)"
              >
                {{ isRecording ? 'Press keys...' : (pad.hotkey || 'Click to set') }}
              </button>
              @if (pad.hotkey) {
                <button
                  class="px-2 text-text-muted hover:text-status-error transition-colors"
                  (click)="clearShortcut($event)"
                  title="Clear shortcut"
                >&times;</button>
              }
            </div>
            <p class="text-[10px] text-text-muted mt-1">Use Ctrl, Alt, Shift or Cmd + key</p>
          </div>

          <!-- Reset button -->
          <button
            class="w-full py-2 text-xs text-text-secondary hover:text-text-primary bg-surface-hover hover:bg-border rounded transition-colors"
            (click)="resetAll()"
          >
            Reset to defaults
          </button>
        </div>
      </div>
    }
  `,
  styles: [`
    @keyframes soundbar {
      from { height: 4px; }
      to { height: 16px; }
    }
  `]
})
export class SoundPadComponent implements OnInit {
  @Input({ required: true }) pad!: SoundPad;
  @Input() hotkey?: string;
  @Input() loading = false;
  @Input() isPreviewing = false;

  @Output() play = new EventEmitter<void>();
  @Output() preview = new EventEmitter<void>();
  @Output() import = new EventEmitter<void>();
  @Output() remove = new EventEmitter<void>();
  @Output() volumeChange = new EventEmitter<number>();
  @Output() speedChange = new EventEmitter<number>();
  @Output() shortcutChange = new EventEmitter<string | null>();
  @Output() customNameChange = new EventEmitter<string | null>();
  @Output() imageChange = new EventEmitter<PadImage | null>();

  @HostBinding('class') hostClass = 'relative';
  @HostBinding('class.z-50') get isPopupOpen() { return this.showSettingsPopup; }

  private tauri = inject(TauriService);
  private _imagesDir = signal<string>('');
  imageSearchService = inject(ImageSearchService);

  readonly imageUrl = computed(() => {
    const pad = this.pad;
    if (!pad.image?.localPath) return null;
    const dir = this._imagesDir();
    if (!dir) return null;
    return convertFileSrc(`${dir}/${pad.image.localPath}`);
  });

  showSettingsPopup = false;
  modalVisible = false;
  isRecording = false;
  Math = Math;
  speedOptions = [0.5, 0.75, 1, 1.25, 1.5, 2];

  // Image management properties
  showImageSearch = false;
  imageSearchQuery = '';
  imageSearchResults = signal<ImageSearchResult[]>([]);
  selectedImageId = signal<string | null>(null);
  imageLoading = signal(false);

  constructor(
    private soundboardService: SoundboardService,
    private shortcutService: ShortcutService
  ) {}

  async ngOnInit(): Promise<void> {
    try {
      const dir = await this.tauri.getImagesDir();
      this._imagesDir.set(dir);
    } catch (e) {
      console.error('Failed to get images dir:', e);
    }
  }

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

    // Disable hover effects when modal is open
    if (!this.showSettingsPopup) {
      classes += ' hover:scale-[1.02] hover:glow-subtle';
    }

    if (this.pad.isPlaying) {
      classes += ' animate-glow-pulse border-white';
    }

    if (this.isPreviewing) {
      classes += ' border-status-info';
    }

    if (this.loading) {
      classes += ' opacity-50 pointer-events-none';
    }

    return classes;
  }

  @HostListener('window:keydown', ['$event'])
  onWindowKeydown(event: KeyboardEvent): void {
    if (!this.isRecording) return;

    event.preventDefault();
    event.stopPropagation();

    const shortcut = this.shortcutService.formatEventAsShortcut(event);
    if (!shortcut) return; // Ignore modifier-only presses

    this.isRecording = false;

    // Check for conflicts
    const conflictPadId = this.shortcutService.checkConflict(shortcut, this.pad.id);
    if (conflictPadId) {
      const pads = this.soundboardService.pads();
      const conflictPad = pads.find(p => p.id === conflictPadId);
      const conflictName = conflictPad?.sound?.name || conflictPadId;

      if (!confirm(`"${shortcut}" is already assigned to "${conflictName}". Replace?`)) {
        return;
      }
      // User confirmed replacement - the old pad will lose its shortcut
    }

    this.shortcutChange.emit(shortcut);
  }

  onClick(event: MouseEvent): void {
    if (this.pad.sound) {
      this.play.emit();
    } else {
      this.import.emit();
    }
  }

  onRightClick(event: MouseEvent): void {
    event.preventDefault();
  }

  onPreview(event: MouseEvent): void {
    event.stopPropagation();
    this.preview.emit();
  }

  onRemove(event: MouseEvent): void {
    event.stopPropagation();
    this.remove.emit();
  }

  toggleSettingsPopup(event: MouseEvent): void {
    event.stopPropagation();
    if (!this.showSettingsPopup) {
      // Opening: show structure first, then fade in after a tick
      this.showSettingsPopup = true;
      this.modalVisible = false;
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          this.modalVisible = true;
        });
      });
    } else {
      // Closing
      this.showSettingsPopup = false;
      this.modalVisible = false;
    }
  }

  closePopup(event: MouseEvent): void {
    event.stopPropagation();
    event.preventDefault();
    this.showSettingsPopup = false;
    this.modalVisible = false;
    this.isRecording = false;
  }

  onVolumeChange(volume: number): void {
    this.volumeChange.emit(volume);
  }

  onSpeedChange(speed: number): void {
    this.speedChange.emit(speed);
  }

  onCustomNameChange(name: string): void {
    this.customNameChange.emit(name.trim() || null);
  }

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
      this.selectedImageId.set(result.id);

      // Download image
      const { data, extension } = await this.imageSearchService.downloadImage(result.fullUrl);

      // Save to backend
      const localPath = await this.tauri.savePadImage(this.pad.id, data, extension);

      // Update pad
      const image: PadImage = {
        localPath,
        originalUrl: result.fullUrl,
        attribution: result.title
      };
      this.imageChange.emit(image);

      this.showImageSearch = false;
      this.selectedImageId.set(null);
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

  resetAll(): void {
    this.volumeChange.emit(1.0);
    this.speedChange.emit(1.0);
    this.customNameChange.emit(null);
    this.removeImage();
  }

  startRecording(event: MouseEvent): void {
    event.stopPropagation();
    this.isRecording = true;
  }

  clearShortcut(event: MouseEvent): void {
    event.stopPropagation();
    this.shortcutChange.emit(null);
  }

  formatDuration(seconds: number): string {
    return this.soundboardService.formatDuration(seconds);
  }
}
