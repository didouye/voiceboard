import {
  Component,
  Input,
  Output,
  EventEmitter,
  HostListener,
  HostBinding,
  inject,
  OnInit,
  OnChanges,
  SimpleChanges,
  signal,
  computed,
} from "@angular/core";
import { CommonModule } from "@angular/common";
import { FormsModule } from "@angular/forms";
import { SoundPad, PadImage } from "../../../core/models";
import { SoundboardService } from "../../../core/services/soundboard.service";
import { ShortcutService } from "../../../core/services/shortcut.service";
import { TauriService } from "../../../core/services/tauri.service";
import {
  ImageSearchService,
  ImageSearchResult,
} from "../../../core/services/image-search.service";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

@Component({
  selector: "app-sound-pad",
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
    <div
      class="aspect-square max-w-[140px] rounded-xl cursor-pointer relative overflow-hidden transition-all duration-150 flex items-end justify-center group"
      [class]="padClasses"
      [style.--pad-color]="pad.color"
      [title]="pad.sound?.name || ''"
      [draggable]="pad.sound !== null"
      (click)="onClick($event)"
      (contextmenu)="onRightClick($event)"
      (dragstart)="onDragStart($event)"
      (dragend)="onDragEnd($event)"
    >
      <!-- Image background -->
      @if (imageUrl()) {
        <img
          [src]="imageUrl()"
          alt=""
          class="absolute inset-0 w-full h-full object-cover"
        />
        <!-- Gradient overlay for text readability -->
        <div
          class="absolute inset-0 bg-gradient-to-t from-black/80 via-black/20 to-transparent"
        ></div>
      }

      <!-- Hotkey badge -->
      @if (pad.sound?.hotkey; as hotkey) {
        <span
          class="absolute top-2 left-2 px-1.5 py-0.5 bg-black/50 text-white/80 text-[10px] font-semibold rounded font-mono uppercase z-10"
        >
          {{ hotkey }}
        </span>
      }

      @if (pad.sound; as sound) {
        <!-- Sound content (positioned at bottom with z-index) -->
        <div class="text-center px-2 w-full pb-2 relative z-10">
          <span
            class="block text-xs font-semibold text-white truncate mb-0.5 drop-shadow-[0_2px_4px_rgba(0,0,0,0.8)]"
          >
            @for (segment of getHighlightedName(); track $index) {
              @if (segment.highlighted) {
                <span class="text-accent font-bold">{{ segment.text }}</span>
              } @else {
                <span>{{ segment.text }}</span>
              }
            }
          </span>
          @if (sound.customName) {
            <span
              class="block text-[9px] text-white/70 truncate drop-shadow-md"
            >
              {{ sound.name }}
            </span>
          }
          <span class="block text-[10px] text-white/70 mt-0.5 drop-shadow-md">
            {{ formatDuration(sound.duration) }}
          </span>
        </div>

        <!-- Playing indicator -->
        @if (sound.isPlaying) {
          <div
            class="absolute bottom-2 left-1/2 -translate-x-1/2 flex gap-0.5 items-end h-4"
          >
            <span
              class="w-1 bg-white rounded-sm animate-[soundbar_0.4s_ease-in-out_infinite_alternate]"
              style="height: 8px"
            ></span>
            <span
              class="w-1 bg-white rounded-sm animate-[soundbar_0.4s_ease-in-out_infinite_alternate_0.1s]"
              style="height: 14px"
            ></span>
            <span
              class="w-1 bg-white rounded-sm animate-[soundbar_0.4s_ease-in-out_infinite_alternate_0.2s]"
              style="height: 10px"
            ></span>
          </div>
        }

        <!-- Action buttons (hidden when modal is open) -->
        <div
          class="absolute top-2 right-2 flex gap-1 transition-opacity"
          [class]="
            showSettingsPopup
              ? 'opacity-0 pointer-events-none'
              : 'opacity-0 group-hover:opacity-100'
          "
        >
          <button
            class="w-6 h-6 rounded-full flex items-center justify-center text-[10px] transition-colors"
            [class]="
              sound.volume !== 1.0
                ? 'bg-status-warning text-black'
                : 'bg-black/50 text-white hover:bg-accent'
            "
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
            {{ isPreviewing ? "&#9632;" : "&#9654;" }}
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
        <div
          class="flex flex-col items-center text-text-muted group-hover:text-text-secondary transition-colors"
        >
          <span class="text-3xl font-light">+</span>
          <span class="text-[10px] uppercase tracking-wide">Import</span>
        </div>
      }
    </div>

    <!-- Settings modal (covers pad grid area only, leaving sidebar and status bar visible) -->
    @if (showSettingsPopup && pad.sound; as sound) {
      <div
        class="fixed top-0 right-0 bottom-14 left-48 z-50 flex items-center justify-center transition-opacity duration-150 p-6"
        [class]="modalVisible ? 'opacity-100' : 'opacity-0'"
        (click)="closePopup($event)"
      >
        <!-- Dark backdrop -->
        <div
          class="absolute inset-0 bg-black/60 backdrop-blur-sm rounded-lg"
        ></div>

        <!-- Modal content - fills available space with max width -->
        <div
          class="relative bg-surface border border-border rounded-xl p-6 w-full max-w-lg max-h-full overflow-y-auto shadow-xl transition-all duration-150"
          [class]="
            modalVisible ? 'opacity-100 scale-100' : 'opacity-0 scale-95'
          "
          (click)="$event.stopPropagation()"
        >
          <!-- Header -->
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-sm font-semibold text-text-primary truncate">
              {{ sound.name }}
            </h3>
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
              [ngModel]="sound.customName || ''"
              (ngModelChange)="onCustomNameChange($event)"
              [placeholder]="sound.name"
              class="w-full px-3 py-2 text-sm bg-surface-hover border border-border rounded text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent"
            />
          </div>

          <!-- Image -->
          <div class="mb-4 pt-4 border-t border-border">
            <div class="flex justify-between items-center mb-2 text-xs">
              <span class="text-text-secondary">Image</span>
            </div>

            <!-- Image Preview (clickable) -->
            <div class="relative">
              <button
                class="relative w-full aspect-video rounded-lg overflow-hidden bg-surface-hover border border-border transition-all hover:border-accent group/img"
                [disabled]="imageLoading()"
                (click)="toggleImageOptions()"
              >
                @if (imageUrl()) {
                  <img
                    [src]="imageUrl()"
                    alt=""
                    class="w-full h-full object-cover"
                  />
                } @else {
                  <!-- Placeholder -->
                  <div
                    class="flex flex-col items-center justify-center h-full text-text-muted"
                  >
                    <span class="text-3xl mb-1">&#128247;</span>
                    <span class="text-xs">Cliquez pour ajouter</span>
                  </div>
                }
                <!-- Hover overlay -->
                <div
                  class="absolute inset-0 bg-black/50 flex items-center justify-center opacity-0 group-hover/img:opacity-100 transition-opacity"
                >
                  <span class="text-white text-sm font-medium">Modifier</span>
                </div>
              </button>

              <!-- Remove button (top-right, only when image exists) -->
              @if (sound.image && !imageLoading()) {
                <button
                  class="absolute top-2 right-2 w-6 h-6 bg-status-error/90 hover:bg-status-error rounded-full flex items-center justify-center text-white text-xs shadow-lg transition-colors"
                  (click)="removeImage(); $event.stopPropagation()"
                  title="Supprimer l'image"
                >
                  &#128465;
                </button>
              }

              <!-- Loading overlay -->
              @if (imageLoading()) {
                <div
                  class="absolute inset-0 bg-black/50 flex items-center justify-center rounded-lg"
                >
                  <span class="text-white text-sm">Chargement...</span>
                </div>
              }
            </div>

            <!-- Image Options Modal -->
            @if (showImageOptions) {
              <div
                class="mt-2 p-3 bg-background rounded-lg border border-border animate-fade-in"
              >
                <div class="flex gap-2 mb-3">
                  <button
                    class="flex-1 px-3 py-2 text-sm bg-surface-hover hover:bg-border rounded transition-colors text-text-secondary hover:text-text-primary flex items-center justify-center gap-2"
                    (click)="uploadImage()"
                    [disabled]="imageLoading()"
                  >
                    <span>&#128193;</span>
                    <span>Fichier local</span>
                  </button>
                  <button
                    class="flex-1 px-3 py-2 text-sm bg-surface-hover hover:bg-border rounded transition-colors text-text-secondary hover:text-text-primary flex items-center justify-center gap-2"
                    (click)="
                      showImageSearch = !showImageSearch;
                      showImageOptions = false
                    "
                    [disabled]="imageLoading()"
                  >
                    <span>&#128269;</span>
                    <span>Recherche en ligne</span>
                  </button>
                </div>
              </div>
            }

            <!-- Search Section (expandable) -->
            @if (showImageSearch) {
              <div
                class="mt-2 p-3 bg-background rounded-lg border border-border animate-fade-in"
              >
                <div class="flex gap-2 mb-3">
                  <input
                    type="text"
                    [(ngModel)]="imageSearchQuery"
                    (keydown.enter)="searchImages()"
                    placeholder="Rechercher des images..."
                    class="flex-1 px-3 py-2 text-sm bg-surface-hover border border-border rounded text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent"
                  />
                  <button
                    class="px-4 py-2 text-sm bg-accent hover:bg-accent/80 text-white rounded transition-colors"
                    [disabled]="imageLoading()"
                    (click)="searchImages()"
                  >
                    {{ imageLoading() ? "..." : "Chercher" }}
                  </button>
                </div>

                <!-- Results Grid -->
                @if (imageSearchResults().length > 0) {
                  <div class="grid grid-cols-3 gap-2">
                    @for (result of imageSearchResults(); track result.id) {
                      <button
                        class="aspect-square rounded overflow-hidden border-2 transition-all hover:scale-105"
                        [class]="
                          selectedImageId() === result.id
                            ? 'border-accent'
                            : 'border-transparent'
                        "
                        (click)="selectImage(result)"
                        [disabled]="imageLoading()"
                      >
                        <img
                          [src]="result.thumbnailUrl"
                          alt=""
                          class="w-full h-full object-cover"
                        />
                      </button>
                    }
                  </div>
                } @else if (!imageLoading()) {
                  <p class="text-xs text-text-muted text-center py-4">
                    Recherchez des images
                  </p>
                }

                <!-- Close button -->
                <button
                  class="w-full mt-3 py-1.5 text-xs text-text-muted hover:text-text-primary transition-colors"
                  (click)="showImageSearch = false"
                >
                  Fermer
                </button>
              </div>
            }
          </div>

          <!-- Volume -->
          <div class="mb-4">
            <div class="flex justify-between items-center mb-2 text-xs">
              <span class="text-text-secondary">Volume</span>
              <span class="text-text-primary font-semibold"
                >{{ Math.round(sound.volume * 100) }}%</span
              >
            </div>
            <input
              type="range"
              [ngModel]="sound.volume"
              (ngModelChange)="onVolumeChange($event)"
              min="0"
              max="2"
              step="0.05"
              class="w-full h-1.5 bg-surface-hover rounded-full appearance-none cursor-pointer
                     [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3
                     [&::-webkit-slider-thumb]:bg-white [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:cursor-pointer"
            />
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
              <span class="text-text-primary font-semibold"
                >{{ sound.speed }}x</span
              >
            </div>
            <div class="flex flex-wrap gap-1">
              @for (s of speedOptions; track s) {
                <button
                  class="flex-1 min-w-[40px] px-2 py-1.5 text-xs rounded border transition-colors"
                  [class]="
                    sound.speed === s
                      ? 'bg-accent border-accent text-white'
                      : 'bg-surface-hover border-border text-text-secondary hover:text-text-primary'
                  "
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
                [class]="
                  isRecording
                    ? 'bg-accent border-accent text-white animate-pulse'
                    : 'bg-surface-hover border-border text-text-primary hover:border-text-muted'
                "
                (click)="startRecording($event)"
              >
                {{
                  isRecording ? "Press keys..." : sound.hotkey || "Click to set"
                }}
              </button>
              @if (sound.hotkey) {
                <button
                  class="px-2 text-text-muted hover:text-status-error transition-colors"
                  (click)="clearShortcut($event)"
                  title="Clear shortcut"
                >
                  &times;
                </button>
              }
            </div>
            <p class="text-[10px] text-text-muted mt-1">
              Use Ctrl, Alt, Shift or Cmd + key
            </p>
          </div>

          <!-- Folders -->
          @if (folders().length > 1) {
            <div class="mb-4 pt-4 border-t border-border">
              <div class="flex justify-between items-center mb-2 text-xs">
                <span class="text-text-secondary">Folders</span>
              </div>
              <div class="space-y-1">
                @for (folder of folders(); track folder.id) {
                  @if (folder.id !== "all") {
                    <label
                      class="flex items-center gap-2 py-1 cursor-pointer hover:bg-surface-hover rounded px-2 -mx-2"
                    >
                      <input
                        type="checkbox"
                        [checked]="sound.folderIds.includes(folder.id)"
                        (change)="onFolderToggle(folder.id)"
                        class="w-4 h-4 rounded border-border bg-surface-hover text-accent focus:ring-accent focus:ring-offset-0"
                      />
                      <span class="text-sm text-text-primary">{{
                        folder.name
                      }}</span>
                    </label>
                  }
                }
              </div>
            </div>
          }

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
  styles: [
    `
      @keyframes soundbar {
        from {
          height: 4px;
        }
        to {
          height: 16px;
        }
      }
    `,
  ],
})
export class SoundPadComponent implements OnInit, OnChanges {
  @Input({ required: true }) pad!: SoundPad;
  @Input() loading = false;
  @Input() isPreviewing = false;

  @Output() play = new EventEmitter<void>();
  @Output() preview = new EventEmitter<void>();
  @Output() import = new EventEmitter<void>();
  @Output() volumeChange = new EventEmitter<number>();
  @Output() speedChange = new EventEmitter<number>();
  @Output() shortcutChange = new EventEmitter<string | null>();
  @Output() customNameChange = new EventEmitter<string | null>();
  @Output() imageChange = new EventEmitter<PadImage | null>();
  @Output() folderToggle = new EventEmitter<string>();

  @HostBinding("class") hostClass = "relative";
  @HostBinding("class.z-50") get isPopupOpen() {
    return this.showSettingsPopup;
  }

  private tauri = inject(TauriService);
  imageSearchService = inject(ImageSearchService);
  folders = inject(SoundboardService).folders;

  // Track pad image changes with a signal
  private _padImage = signal<PadImage | undefined>(undefined);

  // Computed image URL that reacts to both imagesDir and pad image changes
  readonly imageUrl = computed(() => {
    const image = this._padImage();
    const dir = this.tauri.imagesDir();
    if (!image?.localPath || !dir) {
      return null;
    }
    return convertFileSrc(`${dir}/${image.localPath}`);
  });

  showSettingsPopup = false;
  modalVisible = false;
  isRecording = false;
  Math = Math;
  speedOptions = [0.5, 0.75, 1, 1.25, 1.5, 2];

  // Image management properties
  showImageOptions = false;
  showImageSearch = false;
  imageSearchQuery = "";
  imageSearchResults = signal<ImageSearchResult[]>([]);
  selectedImageId = signal<string | null>(null);
  imageLoading = signal(false);

  constructor(
    private soundboardService: SoundboardService,
    private shortcutService: ShortcutService,
  ) {}

  ngOnChanges(changes: SimpleChanges): void {
    // Sync pad.sound?.image to signal when input changes
    if (changes["pad"]) {
      this._padImage.set(this.pad.sound?.image);
    }
  }

  async ngOnInit(): Promise<void> {
    // Initialize image signal
    this._padImage.set(this.pad.sound?.image);

    // Pre-load images dir into cache
    try {
      await this.tauri.getImagesDir();
    } catch (e) {
      console.error("Failed to get images dir:", e);
    }
  }

  get padClasses(): string {
    const base = "border-2";
    const sound = this.pad.sound;

    if (!sound) {
      return `${base} border-dashed border-white/10 bg-white/5 hover:border-white/25 hover:bg-white/10 items-center`;
    }

    let classes = `${base} border-[var(--pad-color)]`;

    // Only add gradient background if no image
    if (!sound.image) {
      classes += ` bg-gradient-to-br from-[var(--pad-color)] to-[color-mix(in_srgb,var(--pad-color)_70%,black)]`;
    }

    // Disable hover effects when modal is open
    if (!this.showSettingsPopup) {
      classes += " hover:scale-[1.02] hover:glow-subtle";
    }

    if (sound.isPlaying) {
      classes += " animate-glow-pulse border-white";
    }

    if (this.isPreviewing) {
      classes += " border-status-info";
    }

    if (this.loading) {
      classes += " opacity-50 pointer-events-none";
    }

    return classes;
  }

  @HostListener("window:keydown", ["$event"])
  onWindowKeydown(event: KeyboardEvent): void {
    if (!this.isRecording) return;

    event.preventDefault();
    event.stopPropagation();

    const shortcut = this.shortcutService.formatEventAsShortcut(event);
    if (!shortcut) return; // Ignore modifier-only presses

    this.isRecording = false;

    // Check for conflicts
    const conflictSoundId = this.shortcutService.checkConflict(
      shortcut,
      this.pad.sound!.id,
    );
    if (conflictSoundId) {
      const sounds = this.soundboardService.sounds();
      const conflictSound = sounds.get(conflictSoundId);
      const conflictName = conflictSound?.name || conflictSoundId;

      if (
        !confirm(
          `"${shortcut}" is already assigned to "${conflictName}". Replace?`,
        )
      ) {
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

  async onRemove(event: MouseEvent): Promise<void> {
    event.stopPropagation();
    event.preventDefault();

    // Store sound info before any async operation
    const soundId = this.pad.sound!.id;
    const soundName =
      this.pad.sound?.customName || this.pad.sound?.name || "this sound";

    // Tauri's window.confirm returns a Promise, not a boolean
    const confirmed = await window.confirm(`Supprimer "${soundName}" ?`);

    if (confirmed) {
      this.soundboardService.removeSound(soundId);
    }
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

  toggleImageOptions(): void {
    this.showImageOptions = !this.showImageOptions;
    // Close search when toggling options
    if (this.showImageOptions) {
      this.showImageSearch = false;
    }
  }

  async searchImages(): Promise<void> {
    if (!this.imageSearchQuery.trim()) return;

    try {
      this.imageLoading.set(true);
      const results = await this.imageSearchService.search(
        this.imageSearchQuery,
      );
      this.imageSearchResults.set(results);
    } catch (err) {
      console.error("Image search failed:", err);
    } finally {
      this.imageLoading.set(false);
    }
  }

  async selectImage(result: ImageSearchResult): Promise<void> {
    try {
      this.imageLoading.set(true);
      this.selectedImageId.set(result.id);

      // Try full URL first, fall back to thumbnail if it fails
      let data: Uint8Array;
      let extension: string;
      let usedUrl = result.fullUrl;

      try {
        const downloaded = await this.imageSearchService.downloadImage(
          result.fullUrl,
        );
        data = downloaded.data;
        extension = downloaded.extension;
      } catch (fullUrlErr) {
        console.warn("Full URL failed, trying thumbnail:", fullUrlErr);
        // Fall back to thumbnail
        const downloaded = await this.imageSearchService.downloadImage(
          result.thumbnailUrl,
        );
        data = downloaded.data;
        extension = downloaded.extension;
        usedUrl = result.thumbnailUrl;
      }

      // Save to backend
      const localPath = await this.tauri.savePadImage(
        this.pad.sound!.id,
        data,
        extension,
      );

      // Update pad
      const image: PadImage = {
        localPath,
        originalUrl: usedUrl,
        attribution: result.title,
      };
      this.imageChange.emit(image);

      this.showImageSearch = false;
      this.selectedImageId.set(null);
    } catch (err) {
      console.error("Failed to save image:", err);
      this.selectedImageId.set(null);
      alert("Failed to download image. Try another one.");
    } finally {
      this.imageLoading.set(false);
    }
  }

  async uploadImage(): Promise<void> {
    // Close options modal immediately
    this.showImageOptions = false;

    try {
      // Open native file picker with image filter
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "Images",
            extensions: ["jpg", "jpeg", "png", "webp", "gif"],
          },
        ],
      });

      if (!selected) return; // User cancelled

      const filePath = selected as string;
      this.imageLoading.set(true);

      // Read file via backend (validates format and size)
      const data = await invoke<number[]>("read_image_file", {
        path: filePath,
      });

      // Get extension from path
      const extension = filePath.split(".").pop()?.toLowerCase() || "jpg";

      // Save image
      const localPath = await this.tauri.savePadImage(
        this.pad.sound!.id,
        new Uint8Array(data),
        extension,
      );

      const image: PadImage = { localPath };
      this.imageChange.emit(image);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      console.error("Failed to upload image:", err);
      alert(message);
    } finally {
      this.imageLoading.set(false);
    }
  }

  async removeImage(): Promise<void> {
    try {
      await this.tauri.deletePadImage(this.pad.sound!.id);
      this.imageChange.emit(null);
    } catch (err) {
      console.error("Failed to remove image:", err);
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

  onFolderToggle(folderId: string): void {
    this.folderToggle.emit(folderId);
  }

  onDragStart(event: DragEvent): void {
    if (!this.pad.sound) return;
    event.dataTransfer?.setData("text/plain", this.pad.sound.id);
    event.dataTransfer!.effectAllowed = "copy";
  }

  onDragEnd(event: DragEvent): void {
    // Cleanup if needed
  }

  getHighlightedName(): { text: string; highlighted: boolean }[] {
    const sound = this.pad.sound;
    if (!sound) return [];

    const displayName = sound.customName || sound.name;
    const indices = new Set(this.pad.matchedIndices || []);

    if (indices.size === 0) {
      return [{ text: displayName, highlighted: false }];
    }

    const segments: { text: string; highlighted: boolean }[] = [];
    let currentSegment = "";
    let currentHighlighted = indices.has(0);

    for (let i = 0; i < displayName.length; i++) {
      const isHighlighted = indices.has(i);
      if (isHighlighted !== currentHighlighted) {
        if (currentSegment) {
          segments.push({
            text: currentSegment,
            highlighted: currentHighlighted,
          });
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
}
