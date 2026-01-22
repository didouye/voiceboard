import { Injectable, signal, computed } from '@angular/core';
import { TauriService } from './tauri.service';
import { MixerConfig, MixerChannel, AudioDevice } from '../models';

/**
 * Service for managing mixer state and operations
 * Uses Angular signals for reactive state management
 */
@Injectable({
  providedIn: 'root'
})
export class MixerService {
  // Reactive state using signals
  private _config = signal<MixerConfig | null>(null);
  private _isRunning = signal(false);
  private _devices = signal<AudioDevice[]>([]);
  private _virtualDriverInstalled = signal<boolean | null>(null);
  private _loading = signal(false);
  private _error = signal<string | null>(null);
  private _soundboardVolume = signal(1.0);

  // Public computed signals
  readonly config = this._config.asReadonly();
  readonly isRunning = this._isRunning.asReadonly();
  readonly devices = this._devices.asReadonly();
  readonly virtualDriverInstalled = this._virtualDriverInstalled.asReadonly();
  readonly loading = this._loading.asReadonly();
  readonly error = this._error.asReadonly();

  // Computed values
  readonly masterVolume = computed(() => this._config()?.masterVolume ?? 1);
  readonly channels = computed(() => this._config()?.channels ?? []);
  readonly inputDevices = computed(() =>
    this._devices().filter(d => d.deviceType === 'InputPhysical')
  );
  readonly soundboardVolume = this._soundboardVolume.asReadonly();

  constructor(private tauri: TauriService) {}

  /**
   * Initialize the mixer service and auto-start if config is valid
   */
  async initialize(): Promise<void> {
    this._loading.set(true);
    this._error.set(null);

    try {
      // Load all initial data in parallel
      const [config, devices, virtualDriver, isMixing] = await Promise.all([
        this.tauri.getMixerConfig(),
        this.tauri.getAudioDevices(),
        this.tauri.checkVirtualDriver(),
        this.tauri.isMixing()
      ]);

      this._config.set(config);
      this._devices.set(devices);
      this._virtualDriverInstalled.set(virtualDriver);
      this._isRunning.set(isMixing);

      // Load settings for auto-start and soundboard volume
      const settings = await this.tauri.loadSettings();
      this._soundboardVolume.set(settings.audio.soundboardVolume ?? 1.0);

      // Apply mic volume to audio engine on startup
      if (settings.audio.micVolume !== undefined) {
        await this.tauri.setMicVolume(settings.audio.micVolume);
      }

      // Auto-start if not already running and config is valid
      if (!isMixing) {
        const hasInput = !!settings.audio.inputDeviceId;
        const hasOutput = !!settings.audio.outputDeviceId;

        if (hasInput && hasOutput) {
          console.log('[MixerService] Auto-starting mixer...');
          await this.start();
        } else {
          console.log('[MixerService] Cannot auto-start: missing input or output device');
        }
      }
    } catch (err) {
      this._error.set(err instanceof Error ? err.message : 'Initialization failed');
      console.error('Failed to initialize mixer:', err);
    } finally {
      this._loading.set(false);
    }
  }

  /**
   * Refresh the device list
   */
  async refreshDevices(): Promise<void> {
    try {
      const devices = await this.tauri.getAudioDevices();
      this._devices.set(devices);
    } catch (err) {
      this._error.set(err instanceof Error ? err.message : 'Failed to refresh devices');
    }
  }

  /**
   * Set the master volume
   */
  async setMasterVolume(volume: number): Promise<void> {
    try {
      await this.tauri.setMasterVolume(volume);
      const config = this._config();
      if (config) {
        this._config.set({ ...config, masterVolume: volume });
      }
    } catch (err) {
      this._error.set(err instanceof Error ? err.message : 'Failed to set master volume');
    }
  }

  /**
   * Set the soundboard volume
   */
  async setSoundboardVolume(volume: number): Promise<void> {
    try {
      await this.tauri.setSoundboardVolume(volume);
      this._soundboardVolume.set(volume);
    } catch (err) {
      this._error.set(err instanceof Error ? err.message : 'Failed to set soundboard volume');
    }
  }

  /**
   * Add a microphone channel
   */
  async addMicrophoneChannel(name: string): Promise<MixerChannel | null> {
    try {
      const id = `mic_${Date.now()}`;
      const channel = await this.tauri.addMicrophoneChannel(id, name);
      await this.refreshConfig();
      return channel;
    } catch (err) {
      this._error.set(err instanceof Error ? err.message : 'Failed to add microphone channel');
      return null;
    }
  }

  /**
   * Add an audio file channel
   */
  async addAudioFileChannel(name: string): Promise<MixerChannel | null> {
    try {
      const id = `audio_${Date.now()}`;
      const channel = await this.tauri.addAudioFileChannel(id, name);
      await this.refreshConfig();
      return channel;
    } catch (err) {
      this._error.set(err instanceof Error ? err.message : 'Failed to add audio file channel');
      return null;
    }
  }

  /**
   * Remove a channel
   */
  async removeChannel(channelId: string): Promise<void> {
    try {
      await this.tauri.removeChannel(channelId);
      await this.refreshConfig();
    } catch (err) {
      this._error.set(err instanceof Error ? err.message : 'Failed to remove channel');
    }
  }

  /**
   * Set channel volume
   */
  async setChannelVolume(channelId: string, volume: number): Promise<void> {
    try {
      await this.tauri.setChannelVolume(channelId, volume);
      await this.refreshConfig();
    } catch (err) {
      this._error.set(err instanceof Error ? err.message : 'Failed to set channel volume');
    }
  }

  /**
   * Toggle channel mute
   */
  async toggleChannelMute(channelId: string): Promise<void> {
    try {
      await this.tauri.toggleChannelMute(channelId);
      await this.refreshConfig();
    } catch (err) {
      this._error.set(err instanceof Error ? err.message : 'Failed to toggle mute');
    }
  }

  /**
   * Start mixing
   */
  async start(): Promise<void> {
    try {
      await this.tauri.startMixing();
      this._isRunning.set(true);
    } catch (err) {
      this._error.set(err instanceof Error ? err.message : 'Failed to start mixing');
    }
  }

  /**
   * Stop mixing
   */
  async stop(): Promise<void> {
    try {
      await this.tauri.stopMixing();
      this._isRunning.set(false);
    } catch (err) {
      this._error.set(err instanceof Error ? err.message : 'Failed to stop mixing');
    }
  }

  /**
   * Restart mixing silently if currently running
   * Used when device configuration changes
   */
  async restartIfRunning(): Promise<void> {
    if (this._isRunning()) {
      console.log('[MixerService] Restarting mixer silently...');
      try {
        await this.tauri.stopMixing();
        await this.tauri.startMixing();
        console.log('[MixerService] Mixer restarted successfully');
      } catch (err) {
        this._error.set(err instanceof Error ? err.message : 'Failed to restart mixer');
        this._isRunning.set(false);
      }
    }
  }

  /**
   * Start or restart mixing when devices are configured
   * Called after device selection changes - automatically starts if both devices are set
   */
  async startOrRestartWithDevices(): Promise<void> {
    try {
      const settings = await this.tauri.loadSettings();
      const hasInput = !!settings.audio.inputDeviceId;
      const hasOutput = !!settings.audio.outputDeviceId;

      if (!hasInput || !hasOutput) {
        console.log('[MixerService] Cannot start: missing input or output device');
        // Stop if running with incomplete config
        if (this._isRunning()) {
          await this.stop();
        }
        return;
      }

      // Both devices configured - start or restart
      if (this._isRunning()) {
        console.log('[MixerService] Restarting mixer with new devices...');
        await this.tauri.stopMixing();
      }

      console.log('[MixerService] Starting mixer...');
      await this.tauri.startMixing();
      this._isRunning.set(true);
      console.log('[MixerService] Mixer started successfully');
    } catch (err) {
      this._error.set(err instanceof Error ? err.message : 'Failed to start/restart mixer');
      this._isRunning.set(false);
    }
  }

  /**
   * Clear the current error
   */
  clearError(): void {
    this._error.set(null);
  }

  /**
   * Refresh the mixer configuration from backend
   */
  private async refreshConfig(): Promise<void> {
    try {
      const config = await this.tauri.getMixerConfig();
      this._config.set(config);
    } catch (err) {
      console.error('Failed to refresh config:', err);
    }
  }
}
