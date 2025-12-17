import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MixerService } from '../../core/services';
import { ChannelStripComponent } from './channel-strip/channel-strip.component';
import { MasterControlComponent } from './master-control/master-control.component';

@Component({
  selector: 'app-mixer',
  standalone: true,
  imports: [CommonModule, ChannelStripComponent, MasterControlComponent],
  template: `
    <div class="mixer-container">
      <header class="mixer-header">
        <h1>Voiceboard</h1>
        <p class="subtitle">Virtual Microphone Mixer</p>

        @if (mixer.virtualDriverInstalled() === false) {
          <div class="warning-banner">
            Virtual Audio Driver not installed.
            <a href="https://github.com/VirtualDrivers/Virtual-Audio-Driver" target="_blank">
              Install Driver
            </a>
          </div>
        }
      </header>

      @if (mixer.loading()) {
        <div class="loading">Loading...</div>
      } @else if (mixer.error()) {
        <div class="error-banner">
          {{ mixer.error() }}
          <button (click)="mixer.clearError()">Dismiss</button>
        </div>
      }

      <div class="mixer-controls">
        <div class="channels-section">
          <div class="section-header">
            <h2>Channels</h2>
            <div class="add-buttons">
              <button (click)="addMicrophone()" class="btn-add">+ Microphone</button>
              <button (click)="addAudioFile()" class="btn-add">+ Audio File</button>
            </div>
          </div>

          <div class="channels-grid">
            @for (channel of mixer.channels(); track channel.id) {
              <app-channel-strip
                [channel]="channel"
                (volumeChange)="onVolumeChange(channel.id, $event)"
                (muteToggle)="onMuteToggle(channel.id)"
                (remove)="onRemoveChannel(channel.id)"
              />
            } @empty {
              <div class="no-channels">
                No channels added. Click "+ Microphone" or "+ Audio File" to add one.
              </div>
            }
          </div>
        </div>

        <div class="master-section">
          <app-master-control
            [volume]="mixer.masterVolume()"
            [isRunning]="mixer.isRunning()"
            (volumeChange)="onMasterVolumeChange($event)"
            (startStop)="onStartStop()"
          />
        </div>
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

    .warning-banner {
      background: #ff6b35;
      padding: 10px 20px;
      border-radius: 8px;
      margin-top: 15px;
      display: inline-block;
    }

    .warning-banner a {
      color: #fff;
      font-weight: bold;
      margin-left: 10px;
    }

    .error-banner {
      background: #e74c3c;
      padding: 15px 20px;
      border-radius: 8px;
      margin-bottom: 20px;
      display: flex;
      justify-content: space-between;
      align-items: center;
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

    .mixer-controls {
      display: grid;
      grid-template-columns: 1fr 200px;
      gap: 30px;
      max-width: 1400px;
      margin: 0 auto;
    }

    .section-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 20px;
    }

    .section-header h2 {
      margin: 0;
      font-size: 1.2rem;
      color: #aaa;
    }

    .add-buttons {
      display: flex;
      gap: 10px;
    }

    .btn-add {
      background: #7b2cbf;
      border: none;
      color: #fff;
      padding: 8px 16px;
      border-radius: 6px;
      cursor: pointer;
      font-size: 0.9rem;
      transition: background 0.2s;
    }

    .btn-add:hover {
      background: #9b4ddb;
    }

    .channels-grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
      gap: 15px;
    }

    .no-channels {
      grid-column: 1 / -1;
      text-align: center;
      padding: 40px;
      color: #666;
      background: rgba(255,255,255,0.05);
      border-radius: 12px;
      border: 2px dashed #333;
    }
  `]
})
export class MixerComponent implements OnInit {
  constructor(public mixer: MixerService) {}

  ngOnInit(): void {
    this.mixer.initialize();
  }

  async addMicrophone(): Promise<void> {
    const name = prompt('Enter microphone channel name:', 'Microphone');
    if (name) {
      await this.mixer.addMicrophoneChannel(name);
    }
  }

  async addAudioFile(): Promise<void> {
    const name = prompt('Enter audio file channel name:', 'Audio');
    if (name) {
      await this.mixer.addAudioFileChannel(name);
    }
  }

  async onVolumeChange(channelId: string, volume: number): Promise<void> {
    await this.mixer.setChannelVolume(channelId, volume);
  }

  async onMuteToggle(channelId: string): Promise<void> {
    await this.mixer.toggleChannelMute(channelId);
  }

  async onRemoveChannel(channelId: string): Promise<void> {
    if (confirm('Remove this channel?')) {
      await this.mixer.removeChannel(channelId);
    }
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
