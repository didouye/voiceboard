import {
  Component,
  Input,
  Output,
  EventEmitter,
  OnInit,
  OnDestroy,
  ElementRef,
  ViewChild,
  signal,
  computed,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import WaveSurfer from 'wavesurfer.js';
import RegionsPlugin from 'wavesurfer.js/dist/plugins/regions.js';

@Component({
  selector: 'app-audio-trimmer',
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
    <div class="space-y-4">
      <!-- Waveform container -->
      <div
        #waveformContainer
        class="w-full h-32 bg-surface-hover rounded-lg overflow-hidden"
      ></div>

      <!-- Time inputs -->
      <div class="flex items-center gap-4">
        <div class="flex-1">
          <label class="text-xs text-text-muted block mb-1">Start (s)</label>
          <input
            type="number"
            [ngModel]="startTime()"
            (ngModelChange)="onStartTimeChange($event)"
            min="0"
            [max]="endTime() - 0.5"
            step="0.1"
            class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary"
          />
        </div>
        <div class="flex-1">
          <label class="text-xs text-text-muted block mb-1">End (s)</label>
          <input
            type="number"
            [ngModel]="endTime()"
            (ngModelChange)="onEndTimeChange($event)"
            [min]="startTime() + 0.5"
            [max]="duration"
            step="0.1"
            class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary"
          />
        </div>
        <div class="flex-1">
          <label class="text-xs text-text-muted block mb-1">Duration</label>
          <div class="px-3 py-2 bg-surface-hover border border-border rounded text-sm text-text-primary">
            {{ formatDuration(selectedDuration()) }}
          </div>
        </div>
      </div>

      <!-- Playback controls -->
      <div class="flex items-center justify-center gap-2">
        <button
          class="px-4 py-2 bg-accent hover:bg-accent/80 text-white rounded text-sm transition-colors"
          (click)="playSelection()"
        >
          {{ isPlaying() ? 'Pause' : 'Play Selection' }}
        </button>
        <button
          class="px-4 py-2 bg-surface-hover hover:bg-border text-text-secondary rounded text-sm transition-colors"
          (click)="resetSelection()"
        >
          Reset
        </button>
      </div>
    </div>
  `,
})
export class AudioTrimmerComponent implements OnInit, OnDestroy {
  @ViewChild('waveformContainer', { static: true }) waveformContainer!: ElementRef;

  @Input() audioUrl!: string;
  @Input() duration!: number;

  @Output() selectionChange = new EventEmitter<{ start: number; end: number }>();

  startTime = signal(0);
  endTime = signal(0);
  isPlaying = signal(false);

  selectedDuration = computed(() => this.endTime() - this.startTime());

  private wavesurfer: WaveSurfer | null = null;
  private regionsPlugin: RegionsPlugin | null = null;
  private activeRegion: any = null;

  ngOnInit(): void {
    this.endTime.set(Math.min(this.duration, 30)); // Default 30s or full duration
    this.initWavesurfer();
  }

  ngOnDestroy(): void {
    this.wavesurfer?.destroy();
  }

  private initWavesurfer(): void {
    this.regionsPlugin = RegionsPlugin.create();

    this.wavesurfer = WaveSurfer.create({
      container: this.waveformContainer.nativeElement,
      waveColor: '#6366f1',
      progressColor: '#818cf8',
      cursorColor: '#c084fc',
      height: 128,
      normalize: true,
      plugins: [this.regionsPlugin],
    });

    this.wavesurfer.load(this.audioUrl);

    this.wavesurfer.on('ready', () => {
      this.createRegion();
    });

    this.wavesurfer.on('play', () => this.isPlaying.set(true));
    this.wavesurfer.on('pause', () => this.isPlaying.set(false));
    this.wavesurfer.on('finish', () => this.isPlaying.set(false));

    this.regionsPlugin.on('region-updated', (region: any) => {
      this.startTime.set(Math.round(region.start * 10) / 10);
      this.endTime.set(Math.round(region.end * 10) / 10);
      this.emitSelection();
    });
  }

  private createRegion(): void {
    if (!this.regionsPlugin) return;

    this.activeRegion = this.regionsPlugin.addRegion({
      start: this.startTime(),
      end: this.endTime(),
      color: 'rgba(139, 92, 246, 0.3)',
      drag: true,
      resize: true,
    });
  }

  onStartTimeChange(value: number): void {
    const newStart = Math.max(0, Math.min(value, this.endTime() - 0.5));
    this.startTime.set(newStart);
    this.updateRegion();
    this.emitSelection();
  }

  onEndTimeChange(value: number): void {
    const newEnd = Math.max(this.startTime() + 0.5, Math.min(value, this.duration));
    this.endTime.set(newEnd);
    this.updateRegion();
    this.emitSelection();
  }

  private updateRegion(): void {
    if (this.activeRegion) {
      this.activeRegion.setOptions({
        start: this.startTime(),
        end: this.endTime(),
      });
    }
  }

  private emitSelection(): void {
    this.selectionChange.emit({
      start: this.startTime(),
      end: this.endTime(),
    });
  }

  playSelection(): void {
    if (!this.wavesurfer) return;

    if (this.isPlaying()) {
      this.wavesurfer.pause();
    } else {
      this.wavesurfer.setTime(this.startTime());
      this.wavesurfer.play();

      // Stop at end time
      const checkEnd = setInterval(() => {
        if (this.wavesurfer && this.wavesurfer.getCurrentTime() >= this.endTime()) {
          this.wavesurfer.pause();
          clearInterval(checkEnd);
        }
      }, 50);
    }
  }

  resetSelection(): void {
    this.startTime.set(0);
    this.endTime.set(Math.min(this.duration, 30));
    this.updateRegion();
    this.emitSelection();
  }

  formatDuration(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    const ms = Math.floor((seconds % 1) * 10);
    return `${mins}:${secs.toString().padStart(2, '0')}.${ms}`;
  }
}
