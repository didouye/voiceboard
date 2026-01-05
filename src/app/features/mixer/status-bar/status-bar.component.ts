import { Component, OnInit, OnDestroy, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { TauriService } from '../../../core/services/tauri.service';
import { MixerService } from '../../../core/services/mixer.service';
import { VuMeterComponent } from '../../../shared/components/vu-meter/vu-meter.component';
import { AudioDevice, AppSettings } from '../../../core/models';

interface AudioLevels {
  inputRms: number;
  inputPeak: number;
  outputRms: number;
  outputPeak: number;
}

@Component({
  selector: 'app-status-bar',
  standalone: true,
  imports: [CommonModule, VuMeterComponent],
  template: `
    <div class="h-14 bg-background border-t border-border flex">
      <!-- Input -->
      <div class="flex-1 px-4 py-2 flex flex-col justify-center border-r border-border">
        <div class="flex items-center gap-2 mb-1">
          <span class="text-sm">&#127908;</span>
          <span class="text-xs text-text-secondary truncate flex-1">{{ inputDeviceName() }}</span>
          <span class="flex items-center gap-1">
            <span
              class="w-2 h-2 rounded-full"
              [class]="mixer.isRunning() ? 'bg-status-success animate-pulse' : 'bg-text-muted'"
            ></span>
            <span class="text-[10px] text-text-muted">{{ mixer.isRunning() ? 'Recording' : 'Ready' }}</span>
          </span>
        </div>
        <app-vu-meter [level]="inputLevel()" />
      </div>

      <!-- Output -->
      <div class="flex-1 px-4 py-2 flex flex-col justify-center border-r border-border">
        <div class="flex items-center gap-2 mb-1">
          <span class="text-sm">&#128266;</span>
          <span class="text-xs text-text-secondary truncate flex-1">{{ outputDeviceName() }}</span>
          <span class="flex items-center gap-1">
            <span
              class="w-2 h-2 rounded-full"
              [class]="mixer.isRunning() ? 'bg-accent animate-pulse' : 'bg-text-muted'"
            ></span>
            <span class="text-[10px] text-text-muted">{{ mixer.isRunning() ? 'Streaming' : 'Ready' }}</span>
          </span>
        </div>
        <app-vu-meter [level]="outputLevel()" />
      </div>

      <!-- Preview -->
      <div class="flex-1 px-4 py-2 flex flex-col justify-center">
        <div class="flex items-center gap-2 mb-1">
          <span class="text-sm">&#127911;</span>
          <span class="text-xs text-text-secondary truncate flex-1">{{ previewDeviceName() }}</span>
          <span class="flex items-center gap-1">
            <span class="w-2 h-2 rounded-full bg-status-info"></span>
            <span class="text-[10px] text-text-muted">Monitor</span>
          </span>
        </div>
        <!-- Preview doesn't have VU meter - it's a local monitor output -->
        <div class="h-1.5 bg-surface rounded-full flex items-center justify-center">
          <span class="text-[8px] text-text-muted">Local preview only</span>
        </div>
      </div>
    </div>
  `,
  styles: []
})
export class StatusBarComponent implements OnInit, OnDestroy {
  // Audio levels
  inputLevel = signal(0);
  outputLevel = signal(0);

  // Device info
  private _settings = signal<AppSettings | null>(null);
  private _inputDevices = signal<AudioDevice[]>([]);
  private _outputDevices = signal<AudioDevice[]>([]);

  // Computed device names
  readonly inputDeviceName = computed(() => {
    const settings = this._settings();
    const devices = this._inputDevices();
    if (!settings?.audio.inputDeviceId) return 'Not selected';
    return devices.find(d => d.id === settings.audio.inputDeviceId)?.name || 'Unknown';
  });

  readonly outputDeviceName = computed(() => {
    const settings = this._settings();
    const devices = this._outputDevices();
    if (!settings?.audio.outputDeviceId) return 'Not selected';
    return devices.find(d => d.id === settings.audio.outputDeviceId)?.name || 'Unknown';
  });

  readonly previewDeviceName = computed(() => {
    const settings = this._settings();
    if (!settings?.audio.previewDeviceId) return 'System Default';
    return 'Custom';
  });

  private unlisten?: UnlistenFn;

  constructor(
    private tauri: TauriService,
    public mixer: MixerService
  ) {}

  async ngOnInit(): Promise<void> {
    // Load device info
    const [settings, inputDevices, outputDevices] = await Promise.all([
      this.tauri.loadSettings(),
      this.tauri.getInputDevices(),
      this.tauri.getVirtualOutputsByPriority()
    ]);

    this._settings.set(settings);
    this._inputDevices.set(inputDevices);
    this._outputDevices.set(outputDevices);

    // Listen for audio levels
    this.unlisten = await listen<AudioLevels>('audio-levels', (event) => {
      this.inputLevel.set(Math.min(event.payload.inputRms * 3, 1));
      this.outputLevel.set(Math.min(event.payload.outputRms * 3, 1));
    });
  }

  ngOnDestroy(): void {
    this.unlisten?.();
  }
}
