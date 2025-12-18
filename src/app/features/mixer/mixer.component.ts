import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MixerService } from '../../core/services';
import { MasterControlComponent } from './master-control/master-control.component';
import { DeviceSelectorComponent } from '../devices/device-selector.component';
import { SoundboardComponent } from '../soundboard/soundboard.component';

@Component({
  selector: 'app-mixer',
  standalone: true,
  imports: [CommonModule, MasterControlComponent, DeviceSelectorComponent, SoundboardComponent],
  template: `
    <div class="mixer-container">
      <header class="mixer-header">
        <h1>Voiceboard</h1>
        <p class="subtitle">Virtual Microphone Mixer</p>
      </header>

      @if (mixer.loading()) {
        <div class="loading">Loading...</div>
      } @else if (mixer.error()) {
        <div class="error-banner">
          {{ mixer.error() }}
          <button (click)="mixer.clearError()">Dismiss</button>
        </div>
      }

      <div class="mixer-layout">
        <!-- Left Sidebar - Device Selection -->
        <aside class="sidebar">
          <app-device-selector />
        </aside>

        <!-- Main Content -->
        <main class="main-content">
          <!-- Soundboard Section -->
          <app-soundboard />
        </main>

        <!-- Right Sidebar - Master Control -->
        <aside class="master-section">
          <app-master-control
            [volume]="mixer.masterVolume()"
            [isRunning]="mixer.isRunning()"
            (volumeChange)="onMasterVolumeChange($event)"
            (startStop)="onStartStop()"
          />
        </aside>
      </div>
    </div>
  `,
  styles: [`
    .mixer-container {
      min-height: 100vh;
      background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
      color: #fff;
      padding: 20px;
    }

    .mixer-header {
      text-align: center;
      margin-bottom: 30px;
    }

    .mixer-header h1 {
      font-size: 2.5rem;
      margin: 0;
      background: linear-gradient(90deg, #00d4ff, #7b2cbf);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
    }

    .subtitle {
      color: #888;
      margin: 5px 0 0;
    }

    .error-banner {
      background: #e74c3c;
      padding: 15px 20px;
      border-radius: 8px;
      margin-bottom: 20px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      max-width: 1400px;
      margin-left: auto;
      margin-right: auto;
    }

    .error-banner button {
      background: rgba(255,255,255,0.2);
      border: none;
      color: #fff;
      padding: 5px 15px;
      border-radius: 4px;
      cursor: pointer;
    }

    .loading {
      text-align: center;
      padding: 40px;
      color: #888;
    }

    .mixer-layout {
      display: grid;
      grid-template-columns: 300px 1fr 200px;
      gap: 25px;
      max-width: 1600px;
      margin: 0 auto;
    }

    .sidebar {
      position: sticky;
      top: 20px;
      align-self: start;
    }

    .main-content {
      min-width: 0;
      display: flex;
      flex-direction: column;
      gap: 25px;
    }

    .master-section {
      position: sticky;
      top: 20px;
      align-self: start;
    }

    /* Responsive */
    @media (max-width: 1200px) {
      .mixer-layout {
        grid-template-columns: 250px 1fr 180px;
      }
    }

    @media (max-width: 900px) {
      .mixer-layout {
        grid-template-columns: 1fr;
        gap: 20px;
      }

      .sidebar, .master-section {
        position: static;
      }

      .master-section {
        order: -1;
      }
    }
  `]
})
export class MixerComponent implements OnInit {
  constructor(public mixer: MixerService) {}

  ngOnInit(): void {
    this.mixer.initialize();
  }

  async onMasterVolumeChange(volume: number): Promise<void> {
    await this.mixer.setMasterVolume(volume);
  }

  async onStartStop(): Promise<void> {
    if (this.mixer.isRunning()) {
      await this.mixer.stop();
    } else {
      await this.mixer.start();
    }
  }
}
