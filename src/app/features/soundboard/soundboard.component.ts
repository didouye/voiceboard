import { Component, HostListener, signal, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { SoundboardService } from '../../core/services/soundboard.service';
import { SoundPadComponent } from './sound-pad/sound-pad.component';
import { listen, TauriEvent } from '@tauri-apps/api/event';

// Default hotkeys for first 12 pads: 1-9, 0, -, =
const DEFAULT_HOTKEYS = ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '='];

@Component({
  selector: 'app-soundboard',
  standalone: true,
  imports: [CommonModule, SoundPadComponent],
  template: `
    <div class="soundboard-container">
      <div class="soundboard-header">
        <h2>Soundboard</h2>
        <div class="header-actions">
          @if (soundboard.playingCount() > 0) {
            <button class="btn-stop-all" (click)="soundboard.stopAll()">
              Stop All ({{ soundboard.playingCount() }})
            </button>
          }
        </div>
      </div>

      @if (soundboard.error()) {
        <div class="error-message">
          {{ soundboard.error() }}
          <button (click)="soundboard.clearError()">Dismiss</button>
        </div>
      }

      <div class="pads-container">
        <div class="pads-grid">
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
            />
          }
        </div>

        @if (isDragging()) {
          <div class="drop-overlay">
            <span>Drop to import {{ dragFileCount() }} file{{ dragFileCount() > 1 ? 's' : '' }}</span>
          </div>
        }
      </div>

      <div class="soundboard-footer">
        <button
          class="btn-import-multiple"
          (click)="importMultiple()"
          [disabled]="soundboard.loading()"
        >
          <span class="icon">📁</span>
          Import Multiple
        </button>
      </div>
    </div>
  `,
  styles: [`
    .soundboard-container {
      background: rgba(255, 255, 255, 0.03);
      border-radius: 12px;
      padding: 20px;
    }

    .soundboard-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 20px;
    }

    .soundboard-header h2 {
      margin: 0;
      font-size: 1.2rem;
      color: #aaa;
    }

    .header-actions {
      display: flex;
      gap: 10px;
    }

    .btn-stop-all {
      background: #e74c3c;
      border: none;
      color: white;
      padding: 8px 16px;
      border-radius: 6px;
      cursor: pointer;
      font-size: 0.85rem;
      transition: background 0.2s;
    }

    .btn-stop-all:hover {
      background: #c0392b;
    }

    .error-message {
      background: rgba(231, 76, 60, 0.2);
      border: 1px solid #e74c3c;
      color: #e74c3c;
      padding: 10px 15px;
      border-radius: 6px;
      margin-bottom: 15px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      font-size: 0.9rem;
    }

    .error-message button {
      background: transparent;
      border: 1px solid #e74c3c;
      color: #e74c3c;
      padding: 4px 10px;
      border-radius: 4px;
      cursor: pointer;
      font-size: 0.8rem;
    }

    .pads-container {
      position: relative;
    }

    .pads-grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
      gap: 12px;
    }

    .drop-overlay {
      position: absolute;
      inset: 0;
      background: rgba(52, 152, 219, 0.85);
      border: 3px dashed #fff;
      border-radius: 12px;
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: 10;
    }

    .drop-overlay span {
      color: #fff;
      font-size: 1.2rem;
      font-weight: 500;
    }

    .soundboard-footer {
      display: flex;
      justify-content: center;
      margin-top: 16px;
    }

    .btn-import-multiple {
      display: flex;
      align-items: center;
      gap: 8px;
      background: rgba(255, 255, 255, 0.08);
      border: 1px dashed rgba(255, 255, 255, 0.3);
      color: #aaa;
      padding: 10px 20px;
      border-radius: 8px;
      cursor: pointer;
      font-size: 0.9rem;
      transition: all 0.2s;
    }

    .btn-import-multiple:hover:not(:disabled) {
      background: rgba(255, 255, 255, 0.12);
      border-color: rgba(255, 255, 255, 0.5);
      color: #fff;
    }

    .btn-import-multiple:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }

    .btn-import-multiple .icon {
      font-size: 1.1rem;
    }

    @media (max-width: 600px) {
      .pads-grid {
        grid-template-columns: repeat(3, 1fr);
      }
    }
  `]
})
export class SoundboardComponent implements OnInit, OnDestroy {
  constructor(public soundboard: SoundboardService) {}

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

    // Find pad by hotkey
    const pads = this.soundboard.pads();
    const padIndex = pads.findIndex(p => {
      const hotkey = p.hotkey || DEFAULT_HOTKEYS[pads.indexOf(p)];
      return hotkey === event.key;
    });

    if (padIndex >= 0) {
      const pad = pads[padIndex];
      if (pad.sound) {
        event.preventDefault();
        this.soundboard.playSound(pad.id);
      }
    }
  }

  /**
   * Get the display hotkey for a pad
   */
  getHotkey(padIndex: number): string | undefined {
    const pads = this.soundboard.pads();
    if (padIndex < pads.length) {
      return pads[padIndex].hotkey || DEFAULT_HOTKEYS[padIndex];
    }
    return undefined;
  }

  /**
   * Handle Import Multiple button click
   */
  async importMultiple(): Promise<void> {
    const result = await this.soundboard.importMultipleSounds();

    if (result.errors.length > 0) {
      const errorMessage = `Imported ${result.imported} files.\nFailed (${result.errors.length}):\n${result.errors.join('\n')}`;
      console.warn(errorMessage);
      // TODO: Show toast notification instead of console
    }
  }

}
