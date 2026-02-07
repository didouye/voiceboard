import {
  Component,
  Output,
  EventEmitter,
  signal,
  inject,
  OnInit,
  computed,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { YouTubeService, YouTubeAudioDto } from '../../../core/services/youtube.service';
import { BinaryManagerService } from '../../../core/services/binary-manager.service';
import { AudioTrimmerComponent } from './audio-trimmer.component';

type ModalState = 'installing' | 'idle' | 'downloading' | 'editing' | 'importing';

@Component({
  selector: 'app-youtube-import-modal',
  standalone: true,
  imports: [CommonModule, FormsModule, AudioTrimmerComponent],
  template: `
    <!-- Backdrop -->
    <div
      class="fixed inset-0 bg-black/70 backdrop-blur-sm z-50 flex items-center justify-center"
      (click)="onBackdropClick($event)"
    >
      <!-- Modal -->
      <div
        class="bg-surface border border-border rounded-xl shadow-2xl w-full max-w-xl mx-4 overflow-hidden"
        (click)="$event.stopPropagation()"
      >
        <!-- Header -->
        <div class="flex items-center justify-between px-6 py-4 border-b border-border">
          <h2 class="text-lg font-semibold text-text-primary">
            Import from YouTube
          </h2>
          <button
            class="text-text-muted hover:text-text-primary transition-colors"
            (click)="onClose()"
          >
            X
          </button>
        </div>

        <!-- Content -->
        <div class="p-6">
          @switch (state()) {
            @case ('installing') {
              <!-- Binary Installation -->
              <div class="text-center py-8 space-y-4">
                @if (binaryManager.error()) {
                  <div class="text-status-error mb-4">
                    <p class="font-medium">Installation failed</p>
                    <p class="text-sm mt-1">{{ binaryManager.error() }}</p>
                  </div>
                  <button
                    class="px-6 py-3 bg-accent hover:bg-accent/80 text-white rounded-lg font-medium transition-colors"
                    (click)="onRetryInstall()"
                  >
                    Retry
                  </button>
                } @else {
                  <div class="animate-spin w-12 h-12 border-4 border-accent border-t-transparent rounded-full mx-auto"></div>
                  <p class="text-text-primary">Installing required tools...</p>
                  @if (binaryManager.progress(); as progress) {
                    <div class="space-y-2">
                      <p class="text-sm text-text-muted">
                        Downloading {{ progress.binary }}...
                        @if (progress.total) {
                          {{ formatBytes(progress.downloaded) }} / {{ formatBytes(progress.total) }}
                        } @else {
                          {{ formatBytes(progress.downloaded) }}
                        }
                      </p>
                      @if (progress.total) {
                        <div class="w-full bg-surface-hover rounded-full h-2">
                          <div
                            class="bg-accent h-2 rounded-full transition-all"
                            [style.width.%]="downloadPercent()"
                          ></div>
                        </div>
                      }
                    </div>
                  }
                }
              </div>
            }

            @case ('idle') {
              <!-- URL Input -->
              <div class="space-y-4">
                @if (binaryManager.updateAvailable(); as version) {
                  <div class="flex items-center justify-between p-3 bg-accent/10 border border-accent/30 rounded-lg">
                    <span class="text-sm text-text-primary">yt-dlp update available: {{ version }}</span>
                    <button
                      class="px-3 py-1 text-sm bg-accent hover:bg-accent/80 text-white rounded font-medium transition-colors"
                      (click)="onUpdateYtdlp()"
                    >
                      Update
                    </button>
                  </div>
                }
                <div>
                  <label class="text-sm text-text-muted block mb-2">YouTube URL</label>
                  <input
                    type="url"
                    [(ngModel)]="url"
                    placeholder="https://youtube.com/watch?v=..."
                    class="w-full px-4 py-3 bg-surface-hover border border-border rounded-lg text-text-primary placeholder-text-muted focus:border-accent focus:outline-none"
                    (keydown.enter)="onDownload()"
                  />
                </div>
                @if (error()) {
                  <p class="text-sm text-status-error">{{ error() }}</p>
                }
                <button
                  class="w-full px-4 py-3 bg-accent hover:bg-accent/80 text-white rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  [disabled]="!isValidUrl()"
                  (click)="onDownload()"
                >
                  Download Audio
                </button>
              </div>
            }

            @case ('downloading') {
              <!-- Progress -->
              <div class="text-center py-8 space-y-4">
                <div class="animate-spin w-12 h-12 border-4 border-accent border-t-transparent rounded-full mx-auto"></div>
                <p class="text-text-primary">Downloading audio...</p>
                <p class="text-sm text-text-muted">This may take a moment</p>
              </div>
            }

            @case ('editing') {
              <!-- Trimmer -->
              <div class="space-y-4">
                <div>
                  <h3 class="font-medium text-text-primary">{{ audioData()?.title }}</h3>
                  <p class="text-sm text-text-muted">
                    Duration: {{ formatDuration(audioData()?.duration || 0) }}
                  </p>
                </div>

                <app-audio-trimmer
                  [audioUrl]="audioUrl()"
                  [duration]="audioData()?.duration || 0"
                  (selectionChange)="onSelectionChange($event)"
                />

                <div>
                  <label class="text-sm text-text-muted block mb-2">Sound name</label>
                  <input
                    type="text"
                    [(ngModel)]="soundName"
                    [placeholder]="audioData()?.title || 'Sound name'"
                    class="w-full px-4 py-3 bg-surface-hover border border-border rounded-lg text-text-primary placeholder-text-muted focus:border-accent focus:outline-none"
                  />
                </div>

                @if (error()) {
                  <p class="text-sm text-status-error">{{ error() }}</p>
                }

                <div class="flex gap-3">
                  <button
                    class="flex-1 px-4 py-3 bg-surface-hover hover:bg-border text-text-secondary rounded-lg font-medium transition-colors"
                    (click)="onCancel()"
                  >
                    Cancel
                  </button>
                  <button
                    class="flex-1 px-4 py-3 bg-accent hover:bg-accent/80 text-white rounded-lg font-medium transition-colors"
                    (click)="onImport()"
                  >
                    Import Sound
                  </button>
                </div>
              </div>
            }

            @case ('importing') {
              <!-- Importing progress -->
              <div class="text-center py-8 space-y-4">
                <div class="animate-spin w-12 h-12 border-4 border-accent border-t-transparent rounded-full mx-auto"></div>
                <p class="text-text-primary">Importing sound...</p>
              </div>
            }
          }
        </div>
      </div>
    </div>
  `,
})
export class YouTubeImportModalComponent implements OnInit {
  private youtube = inject(YouTubeService);
  binaryManager = inject(BinaryManagerService);

  @Output() close = new EventEmitter<void>();
  @Output() imported = new EventEmitter<{ hash: string; name: string; path: string; duration: number }>();

  state = signal<ModalState>('idle');
  error = signal<string | null>(null);
  audioData = signal<YouTubeAudioDto | null>(null);
  audioUrl = signal<string>('');

  url = '';
  soundName = '';
  selection = { start: 0, end: 0 };

  async ngOnInit(): Promise<void> {
    try {
      const status = await this.binaryManager.checkStatus();
      if (!status.all_installed) {
        this.state.set('installing');
        await this.binaryManager.install();
        this.state.set('idle');
      }
    } catch {
      // Error is already set in binaryManager.error signal
    }
  }

  isValidUrl(): boolean {
    return this.youtube.isValidUrl(this.url);
  }

  downloadPercent(): number {
    const p = this.binaryManager.progress();
    if (!p || !p.total) return 0;
    return Math.round((p.downloaded / p.total) * 100);
  }

  formatBytes(bytes: number): string {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
  }

  async onRetryInstall(): Promise<void> {
    this.binaryManager.error.set(null);
    try {
      await this.binaryManager.install();
      this.state.set('idle');
    } catch {
      // Error is already set in binaryManager.error signal
    }
  }

  async onUpdateYtdlp(): Promise<void> {
    try {
      await this.binaryManager.updateYtdlp();
    } catch (err: any) {
      this.error.set(err?.message || err || 'Update failed');
    }
  }

  async onDownload(): Promise<void> {
    if (!this.isValidUrl()) return;

    this.state.set('downloading');
    this.error.set(null);

    try {
      const data = await this.youtube.download(this.url);
      this.audioData.set(data);
      this.audioUrl.set(this.youtube.getAudioUrl(data.temp_path));
      this.soundName = data.title;
      this.selection.end = data.duration;
      this.state.set('editing');
    } catch (err: any) {
      this.error.set(err?.message || err || 'Download failed');
      this.state.set('idle');
    }
  }

  onSelectionChange(selection: { start: number; end: number }): void {
    this.selection = selection;
  }

  async onImport(): Promise<void> {
    const data = this.audioData();
    if (!data) return;

    this.state.set('importing');
    this.error.set(null);

    try {
      const result = await this.youtube.trimAndImport(
        data.temp_path,
        this.selection.start,
        this.selection.end,
        this.soundName || data.title
      );
      this.imported.emit(result);
      this.close.emit();
    } catch (err: any) {
      this.error.set(err?.message || err || 'Import failed');
      this.state.set('editing');
    }
  }

  async onCancel(): Promise<void> {
    const data = this.audioData();
    if (data) {
      await this.youtube.cancel(data.temp_path);
    }
    this.close.emit();
  }

  onClose(): void {
    this.onCancel();
  }

  onBackdropClick(event: MouseEvent): void {
    if (event.target === event.currentTarget) {
      this.onCancel();
    }
  }

  formatDuration(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }
}
