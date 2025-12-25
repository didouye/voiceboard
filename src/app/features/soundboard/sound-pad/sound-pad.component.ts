import { Component, Input, Output, EventEmitter } from '@angular/core';
import { CommonModule } from '@angular/common';
import { SoundPad } from '../../../core/models';
import { SoundboardService } from '../../../core/services/soundboard.service';

@Component({
  selector: 'app-sound-pad',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div
      class="sound-pad"
      [class.has-sound]="pad.sound"
      [class.playing]="pad.isPlaying"
      [class.previewing]="isPreviewing"
      [class.loading]="loading"
      [style.--pad-color]="pad.color"
      (click)="onClick($event)"
      (contextmenu)="onRightClick($event)"
    >
      @if (hotkey) {
        <span class="hotkey-badge">{{ hotkey }}</span>
      }
      @if (pad.sound) {
        <div class="pad-content">
          <span class="sound-name">{{ pad.sound.name }}</span>
          <span class="sound-duration">{{ formatDuration(pad.sound.duration) }}</span>
        </div>
        @if (pad.isPlaying) {
          <div class="playing-indicator">
            <span class="bar"></span>
            <span class="bar"></span>
            <span class="bar"></span>
          </div>
        }
        <div class="action-buttons">
          <button class="action-btn preview-btn"
                  [class.active]="isPreviewing"
                  (click)="onPreview($event)"
                  [title]="isPreviewing ? 'Stop preview' : 'Preview (system output)'">
            {{ isPreviewing ? '⏹' : '▶' }}
          </button>
          <button class="action-btn remove-btn" (click)="onRemove($event)" title="Remove sound">
            ×
          </button>
        </div>
      } @else {
        <div class="empty-pad">
          <span class="plus-icon">+</span>
          <span class="import-text">Import</span>
        </div>
      }
    </div>
  `,
  styles: [`
    .sound-pad {
      --pad-color: #7b2cbf;
      aspect-ratio: 1;
      background: linear-gradient(145deg, rgba(255,255,255,0.08), rgba(255,255,255,0.02));
      border: 2px solid rgba(255, 255, 255, 0.1);
      border-radius: 12px;
      cursor: pointer;
      position: relative;
      overflow: hidden;
      transition: all 0.2s ease;
      display: flex;
      align-items: center;
      justify-content: center;
    }

    .sound-pad:hover {
      border-color: rgba(255, 255, 255, 0.25);
      transform: translateY(-2px);
      box-shadow: 0 4px 12px rgba(0,0,0,0.3);
    }

    .sound-pad:active {
      transform: translateY(0);
    }

    .sound-pad.has-sound {
      background: linear-gradient(145deg, var(--pad-color), color-mix(in srgb, var(--pad-color) 70%, black));
      border-color: var(--pad-color);
    }

    .sound-pad.has-sound:hover {
      box-shadow: 0 4px 20px color-mix(in srgb, var(--pad-color) 50%, transparent);
    }

    .sound-pad.playing {
      animation: pulse 0.5s ease-in-out infinite alternate;
      border-color: #fff;
    }

    .sound-pad.loading {
      opacity: 0.5;
      pointer-events: none;
    }

    @keyframes pulse {
      from { box-shadow: 0 0 10px var(--pad-color); }
      to { box-shadow: 0 0 25px var(--pad-color); }
    }

    .pad-content {
      text-align: center;
      padding: 10px;
      width: 100%;
    }

    .sound-name {
      display: block;
      font-size: 0.8rem;
      font-weight: 600;
      color: #fff;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      margin-bottom: 4px;
      text-shadow: 0 1px 2px rgba(0,0,0,0.3);
    }

    .sound-duration {
      display: block;
      font-size: 0.7rem;
      color: rgba(255,255,255,0.7);
    }

    .playing-indicator {
      position: absolute;
      bottom: 8px;
      left: 50%;
      transform: translateX(-50%);
      display: flex;
      gap: 3px;
      align-items: flex-end;
      height: 16px;
    }

    .bar {
      width: 4px;
      background: #fff;
      border-radius: 2px;
      animation: soundbar 0.4s ease-in-out infinite alternate;
    }

    .bar:nth-child(1) { animation-delay: 0s; height: 8px; }
    .bar:nth-child(2) { animation-delay: 0.1s; height: 14px; }
    .bar:nth-child(3) { animation-delay: 0.2s; height: 10px; }

    @keyframes soundbar {
      from { height: 4px; }
      to { height: 16px; }
    }

    .action-buttons {
      position: absolute;
      top: 4px;
      right: 4px;
      display: flex;
      gap: 4px;
      opacity: 0;
      transition: opacity 0.2s;
    }

    .sound-pad:hover .action-buttons {
      opacity: 1;
    }

    .action-btn {
      width: 20px;
      height: 20px;
      background: rgba(0,0,0,0.5);
      border: none;
      border-radius: 50%;
      color: #fff;
      font-size: 10px;
      line-height: 1;
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      transition: background 0.2s;
    }

    .preview-btn:hover {
      background: #27ae60;
    }

    .preview-btn.active {
      background: #00d4ff;
      color: #000;
    }

    .remove-btn:hover {
      background: #e74c3c;
    }

    .sound-pad.previewing {
      animation: preview-pulse 1s ease-in-out infinite;
      border-color: #00d4ff;
    }

    @keyframes preview-pulse {
      0%, 100% {
        box-shadow: 0 0 8px rgba(0, 212, 255, 0.4);
      }
      50% {
        box-shadow: 0 0 16px rgba(0, 212, 255, 0.7);
      }
    }

    .empty-pad {
      display: flex;
      flex-direction: column;
      align-items: center;
      color: rgba(255,255,255,0.3);
    }

    .plus-icon {
      font-size: 2rem;
      font-weight: 300;
    }

    .import-text {
      font-size: 0.7rem;
      text-transform: uppercase;
      letter-spacing: 0.5px;
    }

    .sound-pad:hover .empty-pad {
      color: rgba(255,255,255,0.6);
    }

    .hotkey-badge {
      position: absolute;
      top: 4px;
      left: 4px;
      background: rgba(0, 0, 0, 0.6);
      color: rgba(255, 255, 255, 0.8);
      font-size: 0.65rem;
      font-weight: 600;
      padding: 2px 5px;
      border-radius: 3px;
      text-transform: uppercase;
      font-family: monospace;
    }

    .sound-pad.has-sound .hotkey-badge {
      background: rgba(0, 0, 0, 0.4);
    }
  `]
})
export class SoundPadComponent {
  @Input({ required: true }) pad!: SoundPad;
  @Input() hotkey?: string;
  @Input() loading = false;
  @Input() isPreviewing = false;

  @Output() play = new EventEmitter<void>();
  @Output() preview = new EventEmitter<void>();
  @Output() import = new EventEmitter<void>();
  @Output() remove = new EventEmitter<void>();

  constructor(private soundboardService: SoundboardService) {}

  onClick(event: MouseEvent): void {
    if (this.pad.sound) {
      this.play.emit();
    } else {
      this.import.emit();
    }
  }

  onRightClick(event: MouseEvent): void {
    event.preventDefault();
    if (this.pad.sound) {
      // Could show context menu in future
    }
  }

  onPreview(event: MouseEvent): void {
    event.stopPropagation();
    this.preview.emit();
  }

  onRemove(event: MouseEvent): void {
    event.stopPropagation();
    this.remove.emit();
  }

  formatDuration(seconds: number): string {
    return this.soundboardService.formatDuration(seconds);
  }
}
