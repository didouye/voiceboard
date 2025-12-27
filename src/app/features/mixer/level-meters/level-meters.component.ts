import { Component, OnInit, OnDestroy, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

interface AudioLevels {
  inputRms: number;
  inputPeak: number;
  outputRms: number;
  outputPeak: number;
}

@Component({
  selector: 'app-level-meters',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="level-meters">
      <div class="meter-container">
        <span class="meter-label">INPUT</span>
        <div class="meter-track">
          <div class="meter-fill" [style.width.%]="inputLevel() * 100"></div>
          <div class="meter-peak" [style.left.%]="inputPeak() * 100"></div>
        </div>
      </div>
      <div class="meter-container">
        <span class="meter-label">OUTPUT</span>
        <div class="meter-track">
          <div class="meter-fill" [style.width.%]="outputLevel() * 100"></div>
          <div class="meter-peak" [style.left.%]="outputPeak() * 100"></div>
        </div>
      </div>
    </div>
  `,
  styles: [`
    .level-meters {
      display: flex;
      gap: 20px;
      padding: 12px 20px;
      background: rgba(0, 0, 0, 0.3);
      border-top: 1px solid rgba(255, 255, 255, 0.1);
    }

    .meter-container {
      flex: 1;
      display: flex;
      align-items: center;
      gap: 12px;
    }

    .meter-label {
      font-size: 0.7rem;
      font-weight: 600;
      color: rgba(255, 255, 255, 0.6);
      text-transform: uppercase;
      letter-spacing: 0.5px;
      min-width: 50px;
    }

    .meter-track {
      flex: 1;
      height: 10px;
      background: rgba(255, 255, 255, 0.1);
      border-radius: 5px;
      position: relative;
      overflow: hidden;
    }

    .meter-fill {
      height: 100%;
      background: linear-gradient(90deg, #7b2cbf, #00d4ff);
      border-radius: 5px;
      transition: width 50ms ease-out;
      box-shadow: 0 0 10px rgba(123, 44, 191, 0.5);
    }

    .meter-peak {
      position: absolute;
      top: 0;
      width: 2px;
      height: 100%;
      background: #fff;
      border-radius: 1px;
      transform: translateX(-50%);
      transition: left 50ms ease-out;
      box-shadow: 0 0 4px rgba(255, 255, 255, 0.8);
    }
  `]
})
export class LevelMetersComponent implements OnInit, OnDestroy {
  inputLevel = signal(0);
  inputPeak = signal(0);
  outputLevel = signal(0);
  outputPeak = signal(0);

  private unlisten?: UnlistenFn;

  async ngOnInit() {
    this.unlisten = await listen<AudioLevels>('audio-levels', (event) => {
      this.inputLevel.set(Math.min(event.payload.inputRms * 3, 1)); // Scale for visibility
      this.inputPeak.set(Math.min(event.payload.inputPeak * 3, 1));
      this.outputLevel.set(Math.min(event.payload.outputRms * 3, 1));
      this.outputPeak.set(Math.min(event.payload.outputPeak * 3, 1));
    });
  }

  ngOnDestroy() {
    this.unlisten?.();
  }
}
