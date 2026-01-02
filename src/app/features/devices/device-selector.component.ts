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
          @if (inputDevices().length === 0) {
            <div class="no-device-warning">No input device available</div>
          } @else {
            <select (change)="onInputDeviceChange($event)" class="device-select">
              <option value="" [selected]="!selectedInputId()">-- Select Microphone --</option>
              @for (device of inputDevices(); track device.id) {
                <option [value]="device.id" [selected]="device.id === selectedInputId()">
                  {{ device.name }} @if (device.isDefault) { (Default) }
                </option>
              }
            </select>
          }
        </div>

        <!-- Preview Output Device Selection -->
        <div class="device-group">
          <label>
            <span class="label-icon">🎧</span>
            <span class="label-text">Preview Output (Monitoring)</span>
          </label>
          <select (change)="onPreviewDeviceChange($event)" class="device-select">
            <option value="" [selected]="!selectedPreviewId()">-- System Default --</option>
            @for (device of physicalOutputDevices(); track device.id) {
              <option [value]="device.id" [selected]="device.id === selectedPreviewId()">
                {{ device.name }} @if (device.isDefault) { (Default) }
              </option>
            }
          </select>
        </div>

        <!-- Virtual Output Selection (only if multiple) -->
        @if (showVirtualOutputSelector()) {
          <div class="device-group">
            <label>
              <span class="label-icon">🔊</span>
              <span class="label-text">Virtual Output</span>
            </label>
            <select (change)="onOutputDeviceChange($event)" class="device-select">
              @for (device of virtualOutputDevices(); track device.id) {
                <option [value]="device.id" [selected]="device.id === selectedOutputId()">
                  {{ device.name }}
                </option>
              }
            </select>
          </div>
        }

        <!-- Status -->
        <div class="status-section">
          <div class="status-item" [class.ready]="isConfigured()" [class.error]="inputDevices().length === 0">
            <span class="status-dot"></span>
            <span>
              @if (inputDevices().length === 0) {
                No input device
              } @else if (isConfigured()) {
                Ready to mix
              } @else {
                Select devices to start
              }
            </span>
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

    .no-device-warning {
      padding: 12px 15px;
      background: rgba(231, 76, 60, 0.1);
      border: 1px solid rgba(231, 76, 60, 0.3);
      border-radius: 8px;
      color: #e74c3c;
      font-size: 0.9rem;
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

    .status-item.error .status-dot {
      background: #e74c3c;
    }

    .status-item.error {
      color: #e74c3c;
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
  private _virtualOutputDevices = signal<AudioDevice[]>([]);
  private _physicalOutputDevices = signal<AudioDevice[]>([]);
  private _settings = signal<AppSettings | null>(null);
  private _loading = signal(true);
  private _error = signal<string | null>(null);

  // Public signals
  readonly inputDevices = this._inputDevices.asReadonly();
  readonly virtualOutputDevices = this._virtualOutputDevices.asReadonly();
  readonly physicalOutputDevices = this._physicalOutputDevices.asReadonly();
  readonly loading = this._loading.asReadonly();
  readonly error = this._error.asReadonly();

  // Computed
  readonly selectedInputId = computed(() => this._settings()?.audio.inputDeviceId ?? '');
  readonly selectedOutputId = computed(() => this._settings()?.audio.outputDeviceId ?? '');
  readonly selectedPreviewId = computed(() => this._settings()?.audio.previewDeviceId ?? '');
  readonly showVirtualOutputSelector = computed(() => this._virtualOutputDevices().length > 1);
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
      const [inputDevices, physicalOutputs, virtualOutputs, settings] = await Promise.all([
        this.tauri.getInputDevices(),
        this.tauri.getPhysicalOutputDevices(),
        this.tauri.getVirtualOutputsByPriority(),
        this.tauri.loadSettings()
      ]);

      this._inputDevices.set(inputDevices);
      this._physicalOutputDevices.set(physicalOutputs);
      this._virtualOutputDevices.set(virtualOutputs);
      this._settings.set(settings);

      console.log('[DeviceSelector] Input devices:', inputDevices.length);
      console.log('[DeviceSelector] Physical outputs:', physicalOutputs.length);
      console.log('[DeviceSelector] Virtual outputs:', virtualOutputs.length);
    } catch (err) {
      this._error.set(err instanceof Error ? err.message : 'Failed to load devices');
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

  async onPreviewDeviceChange(event: Event): Promise<void> {
    const select = event.target as HTMLSelectElement;
    const deviceId = select.value || null;

    try {
      await this.tauri.setPreviewDevice(deviceId);

      // Update local state
      const settings = this._settings();
      if (settings) {
        this._settings.set({
          ...settings,
          audio: { ...settings.audio, previewDeviceId: deviceId }
        });
      }
    } catch (err) {
      console.error('Failed to set preview device:', err);
    }
  }
}
