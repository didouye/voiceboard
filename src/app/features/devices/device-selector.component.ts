import { Component, OnInit, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { TauriService } from '../../core/services/tauri.service';
import { AudioDevice, AppSettings } from '../../core/models';

@Component({
  selector: 'app-device-selector',
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
    <div class="device-selector">
      <h2>Audio Devices</h2>

      @if (loading()) {
        <div class="loading">Loading devices...</div>
      } @else if (error()) {
        <div class="error">{{ error() }}</div>
      } @else {
        <!-- Input Device Selection -->
        <div class="device-group">
          <label>
            <span class="label-icon">🎤</span>
            <span class="label-text">Input Device (Microphone)</span>
          </label>
          <select
            [value]="selectedInputId()"
            (change)="onInputDeviceChange($event)"
            class="device-select"
          >
            <option value="">-- Select Microphone --</option>
            @for (device of inputDevices(); track device.id) {
              <option [value]="device.id">
                {{ device.name }}
                @if (device.isDefault) { (Default) }
              </option>
            }
          </select>
          @if (inputDevices().length === 0) {
            <p class="no-devices">No input devices found</p>
          }
        </div>

        <!-- Output Device Selection -->
        <div class="device-group">
          <label>
            <span class="label-icon">🔊</span>
            <span class="label-text">Output Device (Virtual Microphone)</span>
          </label>
          <select
            [value]="selectedOutputId()"
            (change)="onOutputDeviceChange($event)"
            class="device-select"
          >
            <option value="">-- Select Virtual Device --</option>
            @for (device of outputDevices(); track device.id) {
              <option [value]="device.id">
                {{ device.name }}
                @if (device.isDefault) { (Default) }
              </option>
            }
          </select>
          @if (outputDevices().length === 0) {
            <div class="warning">
              <span class="warning-icon">⚠️</span>
              <span>No virtual audio device found.</span>
              <a href="https://github.com/VirtualDrivers/Virtual-Audio-Driver" target="_blank">
                Install Virtual Audio Driver
              </a>
            </div>
          }
        </div>

        <!-- Status -->
        <div class="status-section">
          <div class="status-item" [class.ready]="isConfigured()">
            <span class="status-dot"></span>
            <span>{{ isConfigured() ? 'Ready to mix' : 'Select both devices to start' }}</span>
          </div>
        </div>

        <!-- Refresh Button -->
        <button class="btn-refresh" (click)="refreshDevices()">
          🔄 Refresh Devices
        </button>
      }
    </div>
  `,
  styles: [`
    .device-selector {
      background: rgba(255, 255, 255, 0.05);
      border-radius: 12px;
      padding: 20px;
    }

    h2 {
      margin: 0 0 20px;
      font-size: 1.1rem;
      color: #888;
      text-transform: uppercase;
      letter-spacing: 1px;
    }

    .loading, .error {
      text-align: center;
      padding: 20px;
      color: #888;
    }

    .error {
      color: #e74c3c;
    }

    .device-group {
      margin-bottom: 20px;
    }

    label {
      display: flex;
      align-items: center;
      gap: 8px;
      margin-bottom: 8px;
      font-size: 0.9rem;
    }

    .label-icon {
      font-size: 1.2rem;
    }

    .label-text {
      color: #ccc;
    }

    .device-select {
      width: 100%;
      padding: 12px 15px;
      border-radius: 8px;
      border: 1px solid #333;
      background: #1a1a2e;
      color: #fff;
      font-size: 0.95rem;
      cursor: pointer;
      transition: border-color 0.2s;
    }

    .device-select:hover {
      border-color: #555;
    }

    .device-select:focus {
      outline: none;
      border-color: #7b2cbf;
    }

    .no-devices {
      color: #666;
      font-size: 0.85rem;
      margin: 8px 0 0;
    }

    .warning {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 12px;
      background: rgba(255, 107, 53, 0.1);
      border: 1px solid rgba(255, 107, 53, 0.3);
      border-radius: 8px;
      margin-top: 10px;
      font-size: 0.85rem;
    }

    .warning a {
      color: #00d4ff;
      margin-left: auto;
    }

    .status-section {
      margin: 20px 0;
      padding: 15px;
      background: rgba(0, 0, 0, 0.2);
      border-radius: 8px;
    }

    .status-item {
      display: flex;
      align-items: center;
      gap: 10px;
      color: #888;
    }

    .status-dot {
      width: 10px;
      height: 10px;
      border-radius: 50%;
      background: #666;
    }

    .status-item.ready .status-dot {
      background: #2ecc71;
      box-shadow: 0 0 10px #2ecc71;
    }

    .status-item.ready {
      color: #2ecc71;
    }

    .btn-refresh {
      width: 100%;
      padding: 12px;
      border: 1px solid #333;
      border-radius: 8px;
      background: transparent;
      color: #888;
      font-size: 0.9rem;
      cursor: pointer;
      transition: all 0.2s;
    }

    .btn-refresh:hover {
      border-color: #555;
      color: #fff;
      background: rgba(255, 255, 255, 0.05);
    }
  `]
})
export class DeviceSelectorComponent implements OnInit {
  // State
  private _inputDevices = signal<AudioDevice[]>([]);
  private _outputDevices = signal<AudioDevice[]>([]);
  private _settings = signal<AppSettings | null>(null);
  private _loading = signal(true);
  private _error = signal<string | null>(null);

  // Public signals
  readonly inputDevices = this._inputDevices.asReadonly();
  readonly outputDevices = this._outputDevices.asReadonly();
  readonly loading = this._loading.asReadonly();
  readonly error = this._error.asReadonly();

  // Computed
  readonly selectedInputId = computed(() => this._settings()?.audio.inputDeviceId ?? '');
  readonly selectedOutputId = computed(() => this._settings()?.audio.outputDeviceId ?? '');
  readonly isConfigured = computed(() => {
    const settings = this._settings();
    return !!(settings?.audio.inputDeviceId && settings?.audio.outputDeviceId);
  });

  constructor(private tauri: TauriService) {}

  ngOnInit(): void {
    this.loadData();
  }

  async loadData(): Promise<void> {
    this._loading.set(true);
    this._error.set(null);

    try {
      const [inputDevices, allDevices, settings] = await Promise.all([
        this.tauri.getInputDevices(),
        this.tauri.getAudioDevices(),
        this.tauri.loadSettings()
      ]);

      this._inputDevices.set(inputDevices);

      // Filter output devices (we want physical outputs + virtual outputs)
      // For sending to virtual mic, we need output devices
      const outputDevices = allDevices.filter(d =>
        d.deviceType === 'OutputVirtual' || d.deviceType === 'OutputPhysical'
      );
      this._outputDevices.set(outputDevices);
      this._settings.set(settings);
    } catch (err) {
      this._error.set(err instanceof Error ? err.message : 'Failed to load devices');
      console.error('Failed to load devices:', err);
    } finally {
      this._loading.set(false);
    }
  }

  async refreshDevices(): Promise<void> {
    await this.loadData();
  }

  async onInputDeviceChange(event: Event): Promise<void> {
    const select = event.target as HTMLSelectElement;
    const deviceId = select.value || null;

    try {
      await this.tauri.setInputDevice(deviceId);

      // Update local state
      const settings = this._settings();
      if (settings) {
        this._settings.set({
          ...settings,
          audio: { ...settings.audio, inputDeviceId: deviceId }
        });
      }
    } catch (err) {
      console.error('Failed to set input device:', err);
    }
  }

  async onOutputDeviceChange(event: Event): Promise<void> {
    const select = event.target as HTMLSelectElement;
    const deviceId = select.value || null;

    try {
      await this.tauri.setOutputDevice(deviceId);

      // Update local state
      const settings = this._settings();
      if (settings) {
        this._settings.set({
          ...settings,
          audio: { ...settings.audio, outputDeviceId: deviceId }
        });
      }
    } catch (err) {
      console.error('Failed to set output device:', err);
    }
  }
}
