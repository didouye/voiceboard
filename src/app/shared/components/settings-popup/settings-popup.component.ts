import {
  Component,
  Input,
  Output,
  EventEmitter,
  OnInit,
  signal,
  computed,
} from "@angular/core";
import { CommonModule } from "@angular/common";
import { FormsModule } from "@angular/forms";
import { invoke } from "@tauri-apps/api/core";
import { TauriService } from "../../../core/services/tauri.service";
import { MixerService } from "../../../core/services/mixer.service";
import { ShortcutService } from "../../../core/services/shortcut.service";
import { DebugConsoleService } from "../../../core/services/debug-console.service";
import { AudioDevice, AppSettings } from "../../../core/models";

@Component({
  selector: "app-settings-popup",
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
    <div
      class="fixed inset-0 z-50 flex items-center justify-center"
      (click)="close.emit()"
    >
      <!-- Backdrop -->
      <div class="absolute inset-0 bg-black/60 backdrop-blur-sm"></div>

      <!-- Modal -->
      <div
        class="relative bg-surface border border-border rounded-xl p-6 w-full max-w-md max-h-[90vh] overflow-y-auto animate-scale-in"
        (click)="$event.stopPropagation()"
      >
        <!-- Header -->
        <div class="flex items-center justify-between mb-6">
          <h2
            class="text-lg font-semibold text-text-primary flex items-center gap-2"
          >
            <span>&#9881;</span> Settings
          </h2>
          <button
            class="w-8 h-8 flex items-center justify-center rounded-lg hover:bg-surface-hover text-text-secondary hover:text-text-primary transition-colors"
            (click)="close.emit()"
          >
            &#10005;
          </button>
        </div>

        @if (loading()) {
          <div class="text-center py-8 text-text-secondary">Loading...</div>
        } @else {
          <!-- Audio Devices Section -->
          <div class="mb-6">
            <h3
              class="text-xs font-semibold text-text-muted uppercase tracking-wider mb-4"
            >
              Audio Devices
            </h3>

            <!-- Input Device -->
            <div class="mb-4">
              <label
                class="flex items-center gap-2 text-sm text-text-secondary mb-2"
              >
                <span>&#127908;</span> Input
              </label>
              <select
                class="w-full px-4 py-3 bg-background border border-border rounded-lg text-text-primary focus:outline-none focus:border-accent transition-colors"
                (change)="onInputChange($event)"
              >
                <option value="" [selected]="!selectedInputId()">
                  -- Select Microphone --
                </option>
                @for (device of inputDevices(); track device.id) {
                  <option
                    [value]="device.id"
                    [selected]="device.id === selectedInputId()"
                  >
                    {{ device.name }}{{ device.isDefault ? " (Default)" : "" }}
                  </option>
                }
              </select>
            </div>

            <!-- Output Device -->
            <div class="mb-4">
              <label
                class="flex items-center gap-2 text-sm text-text-secondary mb-2"
              >
                <span>&#128266;</span> Output (Virtual Mic)
              </label>
              <select
                class="w-full px-4 py-3 bg-background border border-border rounded-lg text-text-primary focus:outline-none focus:border-accent transition-colors"
                (change)="onOutputChange($event)"
              >
                <option value="" [selected]="!selectedOutputId()">
                  -- Select Virtual Output --
                </option>
                @for (device of virtualOutputDevices(); track device.id) {
                  <option
                    [value]="device.id"
                    [selected]="device.id === selectedOutputId()"
                  >
                    {{ device.name }}
                  </option>
                }
              </select>
            </div>

            <!-- Preview Device -->
            <div class="mb-4">
              <label
                class="flex items-center gap-2 text-sm text-text-secondary mb-2"
              >
                <span>&#127911;</span> Preview
              </label>
              <select
                class="w-full px-4 py-3 bg-background border border-border rounded-lg text-text-primary focus:outline-none focus:border-accent transition-colors"
                (change)="onPreviewChange($event)"
              >
                <option value="" [selected]="!selectedPreviewId()">
                  -- System Default --
                </option>
                @for (device of physicalOutputDevices(); track device.id) {
                  <option
                    [value]="device.id"
                    [selected]="device.id === selectedPreviewId()"
                  >
                    {{ device.name }}{{ device.isDefault ? " (Default)" : "" }}
                  </option>
                }
              </select>
            </div>
          </div>

          <!-- Mixer Section -->
          <div class="mb-6">
            <h3
              class="text-xs font-semibold text-text-muted uppercase tracking-wider mb-4"
            >
              Mixer
            </h3>

            <!-- Mic Volume -->
            <div class="mb-4">
              <div class="flex items-center justify-between mb-2">
                <label class="text-sm text-text-secondary">Mic Volume</label>
                <div class="flex items-center gap-2">
                  <span class="text-sm text-text-primary font-mono"
                    >{{ Math.round(micVolume() * 100) }}%</span
                  >
                  <button
                    class="w-8 h-8 flex items-center justify-center rounded-lg transition-colors"
                    [class]="
                      micMuted()
                        ? 'bg-status-error text-white'
                        : 'bg-surface-hover text-text-secondary hover:text-text-primary'
                    "
                    (click)="toggleMicMute()"
                  >
                    {{ micMuted() ? "&#128263;" : "&#128264;" }}
                  </button>
                </div>
              </div>
              <input
                type="range"
                min="0"
                max="200"
                [value]="micVolume() * 100"
                (input)="onMicVolumeChange($event)"
                class="w-full h-2 rounded-full appearance-none cursor-pointer
                       [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:h-4
                       [&::-webkit-slider-thumb]:bg-white [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:cursor-pointer
                       [&::-webkit-slider-thumb]:transition-transform [&::-webkit-slider-thumb]:hover:scale-110"
                [style.background]="
                  'linear-gradient(to right, #9d4edd 0%, #9d4edd ' +
                  micVolume() * 50 +
                  '%, #12121a ' +
                  micVolume() * 50 +
                  '%, #12121a 100%)'
                "
              />
            </div>

            <!-- Soundboard Volume -->
            <div class="mb-4">
              <div class="flex items-center justify-between mb-2">
                <label class="text-sm text-text-secondary"
                  >Soundboard Volume</label
                >
                <span class="text-sm text-text-primary font-mono"
                  >{{ Math.round(soundboardVolume() * 100) }}%</span
                >
              </div>
              <input
                type="range"
                min="0"
                max="200"
                [value]="soundboardVolume() * 100"
                (input)="onSoundboardVolumeChange($event)"
                class="w-full h-2 rounded-full appearance-none cursor-pointer
                       [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:h-4
                       [&::-webkit-slider-thumb]:bg-white [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:cursor-pointer
                       [&::-webkit-slider-thumb]:transition-transform [&::-webkit-slider-thumb]:hover:scale-110"
                [style.background]="
                  'linear-gradient(to right, #9d4edd 0%, #9d4edd ' +
                  soundboardVolume() * 50 +
                  '%, #12121a ' +
                  soundboardVolume() * 50 +
                  '%, #12121a 100%)'
                "
              />
            </div>

            <!-- Master Volume -->
            <div class="mb-4">
              <div class="flex items-center justify-between mb-2">
                <label class="text-sm text-text-secondary">Master Volume</label>
                <span class="text-sm text-text-primary font-mono"
                  >{{ Math.round(masterVolume() * 100) }}%</span
                >
              </div>
              <input
                type="range"
                min="0"
                max="100"
                [value]="masterVolume() * 100"
                (input)="onMasterVolumeChange($event)"
                class="w-full h-2 rounded-full appearance-none cursor-pointer
                       [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:h-4
                       [&::-webkit-slider-thumb]:bg-white [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:cursor-pointer
                       [&::-webkit-slider-thumb]:transition-transform [&::-webkit-slider-thumb]:hover:scale-110"
                [style.background]="
                  'linear-gradient(to right, #9d4edd 0%, #9d4edd ' +
                  masterVolume() * 100 +
                  '%, #12121a ' +
                  masterVolume() * 100 +
                  '%, #12121a 100%)'
                "
              />
            </div>

            <!-- Mic Monitoring -->
            <div
              class="flex items-center justify-between py-3 px-4 bg-background rounded-lg"
            >
              <span class="text-sm text-text-secondary">Mic Monitoring</span>
              <button
                class="w-12 h-6 rounded-full transition-colors relative"
                [class]="micMonitoring() ? 'bg-accent' : 'bg-surface'"
                (click)="toggleMicMonitoring()"
              >
                <div
                  class="absolute top-1 w-4 h-4 bg-white rounded-full transition-transform"
                  [class]="micMonitoring() ? 'left-7' : 'left-1'"
                ></div>
              </button>
            </div>

            <!-- Noise Suppression -->
            <div
              class="flex items-center justify-between py-3 px-4 bg-background rounded-lg mt-3"
            >
              <div>
                <span class="text-sm text-text-primary">Noise Suppression</span>
                <p class="text-xs text-text-muted mt-0.5">
                  Reduce background noise (fans, keyboard)
                </p>
              </div>
              <button
                class="w-12 h-6 rounded-full transition-colors relative"
                [class]="noiseSuppression() ? 'bg-accent' : 'bg-surface'"
                (click)="toggleNoiseSuppression()"
              >
                <div
                  class="absolute top-1 w-4 h-4 bg-white rounded-full transition-transform"
                  [class]="noiseSuppression() ? 'left-7' : 'left-1'"
                ></div>
              </button>
            </div>
          </div>

          <!-- Keyboard Section -->
          <div class="pt-6 border-t border-border">
            <h3
              class="text-xs font-semibold text-text-muted uppercase tracking-wider mb-4"
            >
              Keyboard
            </h3>

            <div
              class="flex items-center justify-between py-3 px-4 bg-background rounded-lg"
            >
              <div>
                <span class="text-sm text-text-primary">Global Hotkeys</span>
                <p class="text-xs text-text-muted mt-0.5">
                  Trigger sounds when app is in background
                </p>
              </div>
              <button
                class="w-12 h-6 rounded-full transition-colors relative"
                [class]="globalHotkeysEnabled() ? 'bg-accent' : 'bg-surface'"
                (click)="toggleGlobalHotkeys()"
              >
                <div
                  class="absolute top-1 w-4 h-4 bg-white rounded-full transition-transform"
                  [class]="globalHotkeysEnabled() ? 'left-7' : 'left-1'"
                ></div>
              </button>
            </div>
          </div>

          <!-- Debug Section -->
          <div class="pt-6 border-t border-border">
            <h3
              class="text-xs font-semibold text-text-muted uppercase tracking-wider mb-4"
            >
              Advanced
            </h3>

            <div
              class="flex items-center justify-between py-3 px-4 bg-background rounded-lg"
            >
              <div>
                <span class="text-sm text-text-primary">Debug Mode</span>
                <p class="text-xs text-text-muted mt-0.5">
                  Show debug console for troubleshooting
                </p>
              </div>
              <button
                class="w-12 h-6 rounded-full transition-colors relative"
                [class]="debugEnabled() ? 'bg-accent' : 'bg-surface'"
                (click)="toggleDebugMode()"
              >
                <div
                  class="absolute top-1 w-4 h-4 bg-white rounded-full transition-transform"
                  [class]="debugEnabled() ? 'left-7' : 'left-1'"
                ></div>
              </button>
            </div>
          </div>
        }
      </div>
    </div>
  `,
  styles: [],
})
export class SettingsPopupComponent implements OnInit {
  @Output() close = new EventEmitter<void>();

  // State
  private _inputDevices = signal<AudioDevice[]>([]);
  private _virtualOutputDevices = signal<AudioDevice[]>([]);
  private _physicalOutputDevices = signal<AudioDevice[]>([]);
  private _settings = signal<AppSettings | null>(null);
  private _loading = signal(true);

  // Public signals
  readonly inputDevices = this._inputDevices.asReadonly();
  readonly virtualOutputDevices = this._virtualOutputDevices.asReadonly();
  readonly physicalOutputDevices = this._physicalOutputDevices.asReadonly();
  readonly loading = this._loading.asReadonly();

  // Computed from settings
  readonly selectedInputId = computed(
    () => this._settings()?.audio.inputDeviceId ?? "",
  );
  readonly selectedOutputId = computed(
    () => this._settings()?.audio.outputDeviceId ?? "",
  );
  readonly selectedPreviewId = computed(
    () => this._settings()?.audio.previewDeviceId ?? "",
  );
  readonly micMonitoring = computed(
    () => this._settings()?.audio.micMonitoring ?? false,
  );

  // Mixer state from service
  readonly masterVolume = computed(() => this.mixer.masterVolume());

  // Local state for mic controls (not persisted)
  private _micVolume = signal(1.0);
  private _micMuted = signal(false);
  readonly micVolume = this._micVolume.asReadonly();
  readonly micMuted = this._micMuted.asReadonly();

  // Soundboard volume
  private _soundboardVolume = signal(1.0);
  readonly soundboardVolume = this._soundboardVolume.asReadonly();

  // Global hotkeys enabled state
  readonly globalHotkeysEnabled = computed(() =>
    this.shortcutService.enabled(),
  );

  // Debug mode enabled state
  readonly debugEnabled = computed(() => this.debugConsole.isEnabled());

  // Noise suppression state
  private _noiseSuppression = signal(true);
  readonly noiseSuppression = this._noiseSuppression.asReadonly();

  Math = Math;

  constructor(
    private tauri: TauriService,
    private mixer: MixerService,
    private shortcutService: ShortcutService,
    private debugConsole: DebugConsoleService,
  ) {}

  async ngOnInit(): Promise<void> {
    await this.loadData();
  }

  private async loadData(): Promise<void> {
    this._loading.set(true);
    try {
      const [inputDevices, physicalOutputs, virtualOutputs, settings] =
        await Promise.all([
          this.tauri.getInputDevices(),
          this.tauri.getPhysicalOutputDevices(),
          this.tauri.getVirtualOutputsByPriority(),
          this.tauri.loadSettings(),
        ]);

      this._inputDevices.set(inputDevices);
      this._physicalOutputDevices.set(physicalOutputs);
      this._virtualOutputDevices.set(virtualOutputs);
      this._settings.set(settings);

      // Load volume values
      this._micVolume.set(settings.audio.micVolume ?? 1.0);
      this._soundboardVolume.set(settings.audio.soundboardVolume ?? 1.0);

      // Load noise suppression state
      const noiseSuppressionEnabled = await this.tauri.getNoiseSuppression();
      this._noiseSuppression.set(noiseSuppressionEnabled);
    } catch (err) {
      console.error("Failed to load settings data:", err);
    } finally {
      this._loading.set(false);
    }
  }

  async onInputChange(event: Event): Promise<void> {
    const select = event.target as HTMLSelectElement;
    const deviceId = select.value || null;
    await this.tauri.setInputDevice(deviceId);
    await this.updateSettingsLocal("inputDeviceId", deviceId);
    await this.mixer.startOrRestartWithDevices();
  }

  async onOutputChange(event: Event): Promise<void> {
    const select = event.target as HTMLSelectElement;
    const deviceId = select.value || null;
    await this.tauri.setOutputDevice(deviceId);
    await this.updateSettingsLocal("outputDeviceId", deviceId);
    await this.mixer.startOrRestartWithDevices();
  }

  async onPreviewChange(event: Event): Promise<void> {
    const select = event.target as HTMLSelectElement;
    const deviceId = select.value || null;
    await this.tauri.setPreviewDevice(deviceId);
    await this.updateSettingsLocal("previewDeviceId", deviceId);
  }

  async onMicVolumeChange(event: Event): Promise<void> {
    const input = event.target as HTMLInputElement;
    const volume = parseFloat(input.value) / 100;
    this._micVolume.set(volume);
    await this.tauri.setMicVolume(volume);
  }

  async onMasterVolumeChange(event: Event): Promise<void> {
    const input = event.target as HTMLInputElement;
    const volume = parseFloat(input.value) / 100;
    await this.mixer.setMasterVolume(volume);
  }

  async onSoundboardVolumeChange(event: Event): Promise<void> {
    const input = event.target as HTMLInputElement;
    const volume = parseFloat(input.value) / 100;
    this._soundboardVolume.set(volume);
    await this.mixer.setSoundboardVolume(volume);
  }

  async toggleMicMute(): Promise<void> {
    const newMuted = !this._micMuted();
    this._micMuted.set(newMuted);
    await this.tauri.setMicMuted(newMuted);
  }

  async toggleMicMonitoring(): Promise<void> {
    const newValue = !this.micMonitoring();
    await this.tauri.setMicMonitoring(newValue);
    await this.updateSettingsLocal("micMonitoring", newValue);
  }

  async toggleGlobalHotkeys(): Promise<void> {
    const newValue = !this.globalHotkeysEnabled();
    try {
      await this.shortcutService.setEnabled(newValue);
    } catch (err) {
      console.error("Failed to toggle global hotkeys:", err);
    }
  }

  async toggleDebugMode(): Promise<void> {
    const newValue = !this.debugEnabled();
    try {
      await invoke("set_debug_mode", { enabled: newValue });
    } catch (err) {
      console.error("Failed to toggle debug mode:", err);
    }
  }

  async toggleNoiseSuppression(): Promise<void> {
    const newValue = !this.noiseSuppression();
    try {
      await this.tauri.setNoiseSuppression(newValue);
      this._noiseSuppression.set(newValue);
      // Restart mixing to apply the change
      await this.mixer.startOrRestartWithDevices();
    } catch (err) {
      console.error("Failed to toggle noise suppression:", err);
    }
  }

  private async updateSettingsLocal(
    key: string,
    value: unknown,
  ): Promise<void> {
    const settings = this._settings();
    if (settings) {
      this._settings.set({
        ...settings,
        audio: { ...settings.audio, [key]: value },
      });
    }
  }
}
