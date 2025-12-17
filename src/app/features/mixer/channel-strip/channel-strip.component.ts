import { Component, Input, Output, EventEmitter } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MixerChannel } from '../../../core/models';

@Component({
  selector: 'app-channel-strip',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="channel-strip" [class.muted]="channel.muted">
      <div class="channel-header">
        <span class="channel-type">{{ getTypeIcon() }}</span>
        <span class="channel-name">{{ channel.name }}</span>
        <button class="btn-remove" (click)="remove.emit()" title="Remove channel">×</button>
      </div>

      <div class="fader-container">
        <input
          type="range"
          class="fader"
          [value]="channel.volume * 100"
          min="0"
          max="200"
          (input)="onVolumeInput($event)"
        />
        <div class="volume-display">{{ (channel.volume * 100).toFixed(0) }}%</div>
      </div>

      <div class="channel-controls">
        <button
          class="btn-mute"
          [class.active]="channel.muted"
          (click)="muteToggle.emit()"
        >
          M
        </button>
      </div>
    </div>
  `,
  styles: [`
    .channel-strip {
      background: rgba(255, 255, 255, 0.08);
      border-radius: 12px;
      padding: 15px;
      display: flex;
      flex-direction: column;
      gap: 15px;
      transition: opacity 0.2s;
    }

    .channel-strip.muted {
      opacity: 0.6;
    }

    .channel-header {
      display: flex;
      align-items: center;
      gap: 8px;
    }

    .channel-type {
      font-size: 1.2rem;
    }

    .channel-name {
      flex: 1;
      font-size: 0.9rem;
      font-weight: 500;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }

    .btn-remove {
      background: transparent;
      border: none;
      color: #666;
      font-size: 1.5rem;
      cursor: pointer;
      padding: 0;
      line-height: 1;
      transition: color 0.2s;
    }

    .btn-remove:hover {
      color: #e74c3c;
    }

    .fader-container {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 10px;
    }

    .fader {
      -webkit-appearance: none;
      width: 100%;
      height: 120px;
      writing-mode: vertical-lr;
      direction: rtl;
      background: transparent;
    }

    .fader::-webkit-slider-runnable-track {
      width: 8px;
      height: 100%;
      background: linear-gradient(to top, #00d4ff 0%, #7b2cbf 100%);
      border-radius: 4px;
    }

    .fader::-webkit-slider-thumb {
      -webkit-appearance: none;
      width: 24px;
      height: 12px;
      background: #fff;
      border-radius: 3px;
      cursor: pointer;
      margin-left: -8px;
    }

    .volume-display {
      font-size: 0.8rem;
      color: #888;
      font-family: monospace;
    }

    .channel-controls {
      display: flex;
      justify-content: center;
      gap: 8px;
    }

    .btn-mute {
      width: 36px;
      height: 36px;
      border: none;
      border-radius: 6px;
      background: #333;
      color: #888;
      font-weight: bold;
      cursor: pointer;
      transition: all 0.2s;
    }

    .btn-mute:hover {
      background: #444;
    }

    .btn-mute.active {
      background: #e74c3c;
      color: #fff;
    }
  `]
})
export class ChannelStripComponent {
  @Input({ required: true }) channel!: MixerChannel;
  @Output() volumeChange = new EventEmitter<number>();
  @Output() muteToggle = new EventEmitter<void>();
  @Output() remove = new EventEmitter<void>();

  getTypeIcon(): string {
    switch (this.channel.channelType) {
      case 'Microphone': return '🎤';
      case 'AudioFile': return '🎵';
      case 'SystemAudio': return '🔊';
      default: return '📢';
    }
  }

  onVolumeInput(event: Event): void {
    const input = event.target as HTMLInputElement;
    const volume = parseFloat(input.value) / 100;
    this.volumeChange.emit(volume);
  }
}
