import { Component, Input, Output, EventEmitter } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-master-control',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="master-control">
      <h3>Master</h3>

      <div class="master-fader">
        <input
          type="range"
          class="fader"
          [value]="volume * 100"
          min="0"
          max="100"
          (input)="onVolumeInput($event)"
        />
        <div class="volume-display">{{ (volume * 100).toFixed(0) }}%</div>
      </div>

      <button
        class="btn-start-stop"
        [class.running]="isRunning"
        (click)="startStop.emit()"
      >
        {{ isRunning ? 'STOP' : 'START' }}
      </button>

      <div class="status-indicator" [class.active]="isRunning">
        <span class="dot"></span>
        <span class="text">{{ isRunning ? 'Mixing Active' : 'Stopped' }}</span>
      </div>
    </div>
  `,
  styles: [`
    .master-control {
      background: rgba(255, 255, 255, 0.08);
      border-radius: 12px;
      padding: 20px;
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 20px;
    }

    h3 {
      margin: 0;
      font-size: 1rem;
      color: #888;
      text-transform: uppercase;
      letter-spacing: 2px;
    }

    .master-fader {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 10px;
    }

    .fader {
      -webkit-appearance: none;
      width: 100%;
      height: 150px;
      writing-mode: vertical-lr;
      direction: rtl;
      background: transparent;
    }

    .fader::-webkit-slider-runnable-track {
      width: 12px;
      height: 100%;
      background: linear-gradient(to top, #2ecc71 0%, #f1c40f 70%, #e74c3c 100%);
      border-radius: 6px;
    }

    .fader::-webkit-slider-thumb {
      -webkit-appearance: none;
      width: 32px;
      height: 16px;
      background: #fff;
      border-radius: 4px;
      cursor: pointer;
      margin-left: -10px;
      box-shadow: 0 2px 8px rgba(0,0,0,0.3);
    }

    .volume-display {
      font-size: 1.2rem;
      font-weight: bold;
      font-family: monospace;
      color: #fff;
    }

    .btn-start-stop {
      width: 100%;
      padding: 15px;
      border: none;
      border-radius: 8px;
      font-size: 1rem;
      font-weight: bold;
      cursor: pointer;
      transition: all 0.2s;
      background: #2ecc71;
      color: #fff;
    }

    .btn-start-stop:hover {
      background: #27ae60;
      transform: scale(1.02);
    }

    .btn-start-stop.running {
      background: #e74c3c;
    }

    .btn-start-stop.running:hover {
      background: #c0392b;
    }

    .status-indicator {
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 0.8rem;
      color: #666;
    }

    .status-indicator .dot {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: #666;
      transition: all 0.2s;
    }

    .status-indicator.active .dot {
      background: #2ecc71;
      box-shadow: 0 0 10px #2ecc71;
      animation: pulse 1.5s infinite;
    }

    .status-indicator.active .text {
      color: #2ecc71;
    }

    @keyframes pulse {
      0%, 100% { opacity: 1; }
      50% { opacity: 0.5; }
    }
  `]
})
export class MasterControlComponent {
  @Input() volume = 1;
  @Input() isRunning = false;
  @Output() volumeChange = new EventEmitter<number>();
  @Output() startStop = new EventEmitter<void>();

  onVolumeInput(event: Event): void {
    const input = event.target as HTMLInputElement;
    const volume = parseFloat(input.value) / 100;
    this.volumeChange.emit(volume);
  }
}
