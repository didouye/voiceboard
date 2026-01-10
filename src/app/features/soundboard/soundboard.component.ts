import { Component, HostListener, signal, OnInit, OnDestroy, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { SoundboardService } from '../../core/services/soundboard.service';
import { ShortcutService } from '../../core/services/shortcut.service';
import { TauriService } from '../../core/services/tauri.service';
import { SoundPadComponent } from './sound-pad/sound-pad.component';
import { listen, TauriEvent } from '@tauri-apps/api/event';
import { eventMatchesShortcut, PadImage, SoundPad } from '../../core/models';
import { ImageSuggestionToastComponent } from '../../shared/components/image-suggestion-toast/image-suggestion-toast.component';
import { BulkImageWizardComponent } from '../../shared/components/bulk-image-wizard/bulk-image-wizard.component';
import { ImageSearchService, ImageSearchResult } from '../../core/services/image-search.service';

@Component({
  selector: 'app-soundboard',
  standalone: true,
  imports: [CommonModule, SoundPadComponent, ImageSuggestionToastComponent, BulkImageWizardComponent],
  template: `
    <div class="h-full flex flex-col">
      <!-- Header -->
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-lg font-semibold text-text-primary">
          {{ soundboard.activeFolder().name || 'Soundboard' }}
        </h2>
        <div class="flex items-center gap-2">
          @if (soundboard.playingCount() > 0) {
            <button
              class="px-4 py-2 bg-status-error hover:bg-status-error/80 text-white rounded-lg text-sm font-medium transition-colors"
              (click)="soundboard.stopAll()"
            >
              &#9632; Stop All ({{ soundboard.playingCount() }})
            </button>
          }
        </div>
      </div>

      <!-- Error message -->
      @if (soundboard.error()) {
        <div class="mb-4 px-4 py-3 bg-status-error/20 border border-status-error/50 rounded-lg flex items-center justify-between">
          <span class="text-status-error text-sm">{{ soundboard.error() }}</span>
          <button
            class="px-3 py-1 text-xs text-status-error border border-status-error/50 rounded hover:bg-status-error/20 transition-colors"
            (click)="soundboard.clearError()"
          >
            Dismiss
          </button>
        </div>
      }

      <!-- Pads grid -->
      <div class="flex-1 relative">
        <div class="grid gap-3" style="grid-template-columns: repeat(auto-fill, minmax(100px, 140px));">
          @for (pad of soundboard.pads(); track pad.sound?.id ?? pad.index; let i = $index) {
            <app-sound-pad
              [pad]="pad"
              [loading]="soundboard.loading()"
              [isPreviewing]="soundboard.previewingSoundId() === pad.sound?.id"
              (play)="soundboard.playSound(pad.sound!.id)"
              (preview)="soundboard.previewSound(pad.sound!.id)"
              (import)="onImportSound()"
              (volumeChange)="soundboard.setSoundVolume(pad.sound!.id, $event)"
              (speedChange)="soundboard.setSoundSpeed(pad.sound!.id, $event)"
              (shortcutChange)="onShortcutChange(pad.sound!.id, $event)"
              (customNameChange)="soundboard.setSoundCustomName(pad.sound!.id, $event)"
              (imageChange)="onImageChange(pad.sound!.id, $event)"
              (folderToggle)="soundboard.toggleSoundFolder(pad.sound!.id, $event)"
            />
          }
        </div>

        <!-- Drop overlay -->
        @if (isDragging()) {
          <div class="absolute inset-0 bg-accent/80 border-2 border-dashed border-white rounded-xl flex items-center justify-center z-10">
            <span class="text-white text-lg font-medium">
              Drop to import {{ dragFileCount() }} file{{ dragFileCount() > 1 ? 's' : '' }}
            </span>
          </div>
        }
      </div>

      <!-- Footer -->
      <div class="mt-4 flex justify-center">
        <button
          class="px-6 py-3 bg-surface-hover border border-dashed border-border hover:border-accent text-text-secondary hover:text-text-primary rounded-lg text-sm transition-all flex items-center gap-2"
          [class.opacity-50]="soundboard.loading()"
          [class.cursor-not-allowed]="soundboard.loading()"
          [disabled]="soundboard.loading()"
          (click)="importMultiple()"
        >
          <span>&#128193;</span>
          Import Multiple
        </button>
      </div>
    </div>

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
  `,
  styles: []
})
export class SoundboardComponent implements OnInit, OnDestroy {
  public soundboard = inject(SoundboardService);
  private shortcutService = inject(ShortcutService);
  private tauri = inject(TauriService);
  private imageSearch = inject(ImageSearchService);

  isDragging = signal(false);
  dragFileCount = signal(0);

  // State for auto-suggestion
  showImageSuggestion = signal(false);
  suggestionSoundName = '';
  suggestionFilename = '';
  suggestionSoundId = '';

  showBulkWizard = signal(false);
  bulkWizardPads = signal<SoundPad[]>([]);
  pendingBulkPads = signal<SoundPad[]>([]);
  showBulkPrompt = signal(false);

  private readonly AUDIO_EXTENSIONS = ['mp3', 'ogg', 'wav', 'flac'];
  private unlistenDragEnter?: () => void;
  private unlistenDragOver?: () => void;
  private unlistenDragLeave?: () => void;
  private unlistenDragDrop?: () => void;

  async ngOnInit(): Promise<void> {
    await this.initDragDropListeners();
  }

  ngOnDestroy(): void {
    this.unlistenDragEnter?.();
    this.unlistenDragOver?.();
    this.unlistenDragLeave?.();
    this.unlistenDragDrop?.();
  }

  private async initDragDropListeners(): Promise<void> {
    // Listen to Tauri drag events
    this.unlistenDragEnter = await listen<{ paths: string[]; position: { x: number; y: number } }>(
      TauriEvent.DRAG_ENTER,
      (event) => {
        const audioPaths = event.payload.paths.filter(path => {
          const ext = path.split('.').pop()?.toLowerCase();
          return ext && this.AUDIO_EXTENSIONS.includes(ext);
        });
        if (audioPaths.length > 0) {
          this.dragFileCount.set(audioPaths.length);
          this.isDragging.set(true);
        }
      }
    );

    this.unlistenDragOver = await listen(TauriEvent.DRAG_OVER, () => {
      // Keep overlay visible during drag over
    });

    this.unlistenDragLeave = await listen(TauriEvent.DRAG_LEAVE, () => {
      this.isDragging.set(false);
      this.dragFileCount.set(0);
    });

    this.unlistenDragDrop = await listen<{ paths: string[]; position: { x: number; y: number } }>(
      TauriEvent.DRAG_DROP,
      async (event) => {
        this.isDragging.set(false);
        this.dragFileCount.set(0);

        const audioPaths = event.payload.paths.filter(path => {
          const ext = path.split('.').pop()?.toLowerCase();
          return ext && this.AUDIO_EXTENSIONS.includes(ext);
        });

        if (audioPaths.length === 0) return;

        // Take snapshot of existing sound paths before import
        // We track paths because pads get reorganized after import (sorted alphabetically)
        const existingPaths = new Set(
          this.soundboard.pads()
            .filter(p => p.sound)
            .map(p => p.sound!.path)
        );

        const result = await this.soundboard.importSoundsFromPaths(audioPaths);

        if (result.errors.length > 0 || result.skippedDuplicates > 0) {
          console.warn(`Imported ${result.imported} files. Skipped ${result.skippedDuplicates} duplicates. Failed: ${result.errors.join(', ')}`);
        }

        // If successful imports, find pads with newly imported sounds
        // A "new" sound is one whose path didn't exist before import
        if (result.imported > 0) {
          const padsAfter = this.soundboard.pads();
          const newPads = padsAfter.filter(pad =>
            pad.sound && !existingPaths.has(pad.sound.path)
          );

          if (newPads.length === 1) {
            // Single import: show toast
            this.triggerSingleImportSuggestion(newPads[0]);
          } else if (newPads.length > 1) {
            // Bulk import: show prompt
            this.triggerBulkImportSuggestion(newPads);
          }
        }
      }
    );
  }

  @HostListener('window:keydown', ['$event'])
  handleKeydown(event: KeyboardEvent): void {
    // Ignore if user is typing in an input field
    if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
      return;
    }

    // Escape stops all sounds
    if (event.key === 'Escape') {
      this.soundboard.stopAll();
      return;
    }

    // Check all sounds for matching hotkey
    for (const sound of this.soundboard.sounds().values()) {
      if (sound.hotkey && eventMatchesShortcut(event, sound.hotkey)) {
        event.preventDefault();
        this.soundboard.playSound(sound.id);
        return;
      }
    }
  }

  /**
   * Handle import from clicking an empty pad (with image suggestion)
   */
  async onImportSound(): Promise<void> {
    // Take snapshot of existing sound paths before import
    const existingPaths = new Set(
      this.soundboard.pads()
        .filter(p => p.sound)
        .map(p => p.sound!.path)
    );

    // Do the import
    await this.soundboard.importSound();

    // Find newly imported pad (if any)
    const padsAfter = this.soundboard.pads();
    const newPads = padsAfter.filter(pad =>
      pad.sound && !existingPaths.has(pad.sound.path)
    );

    // Trigger suggestion if a new sound was imported
    if (newPads.length === 1) {
      this.triggerSingleImportSuggestion(newPads[0]);
    }
  }

  async importMultiple(): Promise<void> {
    // Take snapshot of existing sound paths before import
    // We track paths because pads get reorganized after import (sorted alphabetically)
    const existingPaths = new Set(
      this.soundboard.pads()
        .filter(p => p.sound)
        .map(p => p.sound!.path)
    );

    const result = await this.soundboard.importMultipleSounds();

    if (result.errors.length > 0 || result.skippedDuplicates > 0) {
      console.warn(`Imported ${result.imported} files. Skipped ${result.skippedDuplicates} duplicates.\nFailed (${result.errors.length}):\n${result.errors.join('\n')}`);
    }

    // If successful imports, find pads with newly imported sounds
    // A "new" sound is one whose path didn't exist before import
    if (result.imported > 0) {
      const padsAfter = this.soundboard.pads();
      const newPads = padsAfter.filter(pad =>
        pad.sound && !existingPaths.has(pad.sound.path)
      );

      if (newPads.length === 1) {
        // Single import: show toast
        this.triggerSingleImportSuggestion(newPads[0]);
      } else if (newPads.length > 1) {
        // Bulk import: show prompt
        this.triggerBulkImportSuggestion(newPads);
      }
    }
  }

  async onShortcutChange(soundId: string, shortcut: string | null): Promise<void> {
    // Update sound
    this.soundboard.setSoundHotkey(soundId, shortcut);

    // Update shortcut registry
    if (shortcut) {
      try {
        await this.shortcutService.register(soundId, shortcut);
      } catch (err) {
        console.error('Failed to register shortcut:', err);
      }
    } else {
      const oldShortcut = this.shortcutService.getShortcutForSound(soundId);
      if (oldShortcut) {
        await this.shortcutService.unregister(oldShortcut);
      }
    }
  }

  onImageChange(soundId: string, image: PadImage | null): void {
    this.soundboard.setSoundImage(soundId, image);
  }

  /**
   * Handle accepting a suggested image from the toast
   */
  async onAcceptSuggestion(result: ImageSearchResult): Promise<void> {
    try {
      const { data, extension } = await this.imageSearch.downloadImage(result.fullUrl);
      const localPath = await this.tauri.savePadImage(this.suggestionSoundId, data, extension);

      const image: PadImage = {
        localPath,
        originalUrl: result.fullUrl,
        attribution: result.title
      };
      this.soundboard.setSoundImage(this.suggestionSoundId, image);
    } catch (err) {
      console.error('Failed to save suggested image:', err);
    } finally {
      this.showImageSuggestion.set(false);
    }
  }

  /**
   * Handle ignoring the image suggestion
   */
  onIgnoreSuggestion(): void {
    this.showImageSuggestion.set(false);
  }

  /**
   * Handle starting the bulk wizard
   */
  onStartBulkWizard(): void {
    this.showBulkPrompt.set(false);
    this.bulkWizardPads.set(this.pendingBulkPads());
    this.showBulkWizard.set(true);
  }

  /**
   * Handle skipping the bulk wizard
   */
  onSkipBulkWizard(): void {
    this.showBulkPrompt.set(false);
    this.pendingBulkPads.set([]);
  }

  /**
   * Handle selecting an image in the bulk wizard
   */
  async onBulkSelectImage(event: { soundId: string; image: ImageSearchResult }): Promise<void> {
    try {
      const { data, extension } = await this.imageSearch.downloadImage(event.image.fullUrl);
      const localPath = await this.tauri.savePadImage(event.soundId, data, extension);

      const image: PadImage = {
        localPath,
        originalUrl: event.image.fullUrl,
        attribution: event.image.title
      };
      this.soundboard.setSoundImage(event.soundId, image);
    } catch (err) {
      console.error('Failed to save image:', err);
    }
  }

  /**
   * Handle closing the bulk wizard
   */
  onBulkWizardClose(): void {
    this.showBulkWizard.set(false);
    this.bulkWizardPads.set([]);
    this.pendingBulkPads.set([]);
  }

  /**
   * Trigger single import suggestion after a sound is imported
   */
  private triggerSingleImportSuggestion(pad: SoundPad): void {
    if (!this.imageSearch.hasApiKey() || !pad.sound) return;

    this.suggestionSoundId = pad.sound.id;
    this.suggestionSoundName = pad.sound.name;
    this.suggestionFilename = pad.sound.path.split('/').pop() || pad.sound.name;
    this.showImageSuggestion.set(true);
  }

  /**
   * Trigger bulk import suggestion after multiple sounds are imported
   */
  private triggerBulkImportSuggestion(importedPads: SoundPad[]): void {
    if (!this.imageSearch.hasApiKey() || importedPads.length === 0) return;

    this.pendingBulkPads.set(importedPads);
    this.showBulkPrompt.set(true);
  }

}
