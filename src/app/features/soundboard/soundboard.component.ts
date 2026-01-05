import { Component, HostListener, signal, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { SoundboardService } from '../../core/services/soundboard.service';
import { ShortcutService } from '../../core/services/shortcut.service';
import { SoundPadComponent } from './sound-pad/sound-pad.component';
import { listen, TauriEvent } from '@tauri-apps/api/event';
import { eventMatchesShortcut } from '../../core/models';

@Component({
  selector: 'app-soundboard',
  standalone: true,
  imports: [CommonModule, SoundPadComponent],
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
          @for (pad of soundboard.pads(); track pad.id; let i = $index) {
            <app-sound-pad
              [pad]="pad"
              [hotkey]="getHotkey(i)"
              [loading]="soundboard.loading()"
              [isPreviewing]="soundboard.previewingPadId() === pad.id"
              (play)="soundboard.playSound(pad.id)"
              (preview)="soundboard.previewSound(pad.id)"
              (import)="soundboard.importSound(pad.id)"
              (remove)="soundboard.removeSound(pad.id)"
              (volumeChange)="soundboard.setPadVolume(pad.id, $event)"
              (speedChange)="soundboard.setPadSpeed(pad.id, $event)"
              (shortcutChange)="onShortcutChange(pad.id, $event)"
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
  `,
  styles: []
})
export class SoundboardComponent implements OnInit, OnDestroy {
  constructor(
    public soundboard: SoundboardService,
    private shortcutService: ShortcutService
  ) {}

  isDragging = signal(false);
  dragFileCount = signal(0);

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

        const result = await this.soundboard.importSoundsFromPaths(audioPaths);

        if (result.errors.length > 0) {
          console.warn(`Imported ${result.imported} files. Failed: ${result.errors.join(', ')}`);
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

    // Check custom shortcuts (with modifiers)
    const pads = this.soundboard.pads();
    for (const pad of pads) {
      if (pad.hotkey && pad.sound) {
        if (eventMatchesShortcut(event, pad.hotkey)) {
          event.preventDefault();
          this.soundboard.playSound(pad.id);
          return;
        }
      }
    }
  }

  /**
   * Get the display hotkey for a pad (only custom hotkeys, no defaults)
   */
  getHotkey(padIndex: number): string | undefined {
    const pads = this.soundboard.pads();
    if (padIndex < pads.length) {
      return pads[padIndex].hotkey;
    }
    return undefined;
  }

  async importMultiple(): Promise<void> {
    const result = await this.soundboard.importMultipleSounds();

    if (result.errors.length > 0) {
      console.warn(`Imported ${result.imported} files.\nFailed (${result.errors.length}):\n${result.errors.join('\n')}`);
    }
  }

  async onShortcutChange(padId: string, shortcut: string | null): Promise<void> {
    // Update pad
    this.soundboard.setPadHotkey(padId, shortcut);

    // Update shortcut registry
    if (shortcut) {
      try {
        await this.shortcutService.register(padId, shortcut);
      } catch (err) {
        console.error('Failed to register shortcut:', err);
      }
    } else {
      const oldShortcut = this.shortcutService.getShortcutForPad(padId);
      if (oldShortcut) {
        await this.shortcutService.unregister(oldShortcut);
      }
    }
  }

}
