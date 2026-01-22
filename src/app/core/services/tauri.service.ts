import { inject, Injectable, signal } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import {
  AudioDevice,
  MixerChannel,
  MixerConfig,
  AppSettings,
  ApiResponse,
  SoundFile,
  Folder,
} from "../models";
import { DemoService } from "./demo.service";
import {
  DEMO_INPUT_DEVICES,
  DEMO_OUTPUT_DEVICES,
  DEMO_PREVIEW_DEVICES,
  DEMO_SETTINGS,
  DEMO_MIXER_CONFIG,
  DEMO_SOUNDBOARD_PADS,
  createDemoSoundFile,
} from "./demo-data";

/**
 * Imported sound with hash
 */
export interface ImportedSound {
  hash: string;
  name: string;
  path: string;
  duration: number;
}

/**
 * Service for communicating with the Tauri/Rust backend
 * In demo mode, returns mock data instead of calling Tauri APIs
 */
@Injectable({
  providedIn: "root",
})
export class TauriService {
  private demoService = inject(DemoService);
  private _imagesDirCache = signal<string | null>(null);
  private _imagesDirPromise: Promise<string> | null = null;
  private demoMixing = false;

  // =========================================================================
  // Device Management
  // =========================================================================

  /**
   * Get all available audio devices
   */
  async getAudioDevices(): Promise<AudioDevice[]> {
    if (this.demoService.isDemoMode) {
      return [
        ...DEMO_INPUT_DEVICES,
        ...DEMO_OUTPUT_DEVICES,
        ...DEMO_PREVIEW_DEVICES,
      ];
    }
    const response =
      await invoke<ApiResponse<AudioDevice[]>>("get_audio_devices");
    if (!response.success || !response.data) {
      throw new Error(response.error || "Failed to get audio devices");
    }
    return this.mapDevices(response.data);
  }

  /**
   * Get input devices only (microphones)
   */
  async getInputDevices(): Promise<AudioDevice[]> {
    if (this.demoService.isDemoMode) {
      return DEMO_INPUT_DEVICES;
    }
    const response =
      await invoke<ApiResponse<AudioDevice[]>>("get_input_devices");
    if (!response.success || !response.data) {
      throw new Error(response.error || "Failed to get input devices");
    }
    return this.mapDevices(response.data);
  }

  /**
   * Get virtual output devices (for sending mixed audio)
   */
  async getVirtualOutputDevices(): Promise<AudioDevice[]> {
    if (this.demoService.isDemoMode) {
      return DEMO_OUTPUT_DEVICES;
    }
    const response = await invoke<ApiResponse<AudioDevice[]>>(
      "get_virtual_output_devices",
    );
    if (!response.success || !response.data) {
      throw new Error(response.error || "Failed to get virtual output devices");
    }
    return this.mapDevices(response.data);
  }

  /**
   * Get physical output devices (speakers, headphones - for preview/monitoring)
   */
  async getPhysicalOutputDevices(): Promise<AudioDevice[]> {
    if (this.demoService.isDemoMode) {
      return DEMO_PREVIEW_DEVICES;
    }
    const response = await invoke<ApiResponse<AudioDevice[]>>(
      "get_physical_output_devices",
    );
    if (!response.success || !response.data) {
      throw new Error(
        response.error || "Failed to get physical output devices",
      );
    }
    return this.mapDevices(response.data);
  }

  /**
   * Get virtual output devices sorted by priority (VB-Cable first)
   */
  async getVirtualOutputsByPriority(): Promise<AudioDevice[]> {
    if (this.demoService.isDemoMode) {
      return DEMO_OUTPUT_DEVICES;
    }
    const response = await invoke<ApiResponse<AudioDevice[]>>(
      "get_virtual_outputs_by_priority",
    );
    if (!response.success || !response.data) {
      throw new Error(
        response.error || "Failed to get virtual outputs by priority",
      );
    }
    return this.mapDevices(response.data);
  }

  /**
   * Check if virtual audio driver is installed
   */
  async checkVirtualDriver(): Promise<boolean> {
    if (this.demoService.isDemoMode) {
      return true;
    }
    const response = await invoke<ApiResponse<boolean>>("check_virtual_driver");
    return response.success && response.data === true;
  }

  /**
   * Map backend device DTOs to frontend model (handle snake_case to camelCase)
   */
  private mapDevices(devices: any[]): AudioDevice[] {
    return devices.map((d) => ({
      id: d.id,
      name: d.name,
      deviceType: d.device_type,
      isDefault: d.is_default,
      isVirtual: d.is_virtual,
    }));
  }

  // =========================================================================
  // Settings Management
  // =========================================================================

  /**
   * Get current application settings
   */
  async getSettings(): Promise<AppSettings> {
    if (this.demoService.isDemoMode) {
      return DEMO_SETTINGS;
    }
    const settings = await invoke<any>("get_settings");
    return this.mapSettings(settings);
  }

  /**
   * Save application settings
   */
  async saveSettings(settings: AppSettings): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("save_settings", { settings: this.unmapSettings(settings) });
  }

  /**
   * Load settings from persistent storage
   */
  async loadSettings(): Promise<AppSettings> {
    if (this.demoService.isDemoMode) {
      return DEMO_SETTINGS;
    }
    const settings = await invoke<any>("load_settings");
    return this.mapSettings(settings);
  }

  /**
   * Set input device (microphone)
   */
  async setInputDevice(deviceId: string | null): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("set_input_device", { deviceId });
  }

  /**
   * Set output device (virtual microphone)
   */
  async setOutputDevice(deviceId: string | null): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("set_output_device", { deviceId });
  }

  /**
   * Map backend settings to frontend model
   */
  private mapSettings(s: any): AppSettings {
    return {
      audio: {
        inputDeviceId: s.audio.input_device_id,
        outputDeviceId: s.audio.output_device_id,
        previewDeviceId: s.audio.preview_device_id,
        masterVolume: s.audio.master_volume,
        sampleRate: s.audio.sample_rate,
        bufferSize: s.audio.buffer_size,
        micMonitoring: s.audio.mic_monitoring ?? false,
        globalHotkeysEnabled: s.audio.global_hotkeys_enabled ?? true,
        micVolume: s.audio.mic_volume ?? 1.0,
        soundboardVolume: s.audio.soundboard_volume ?? 1.0,
      },
      startMinimized: s.start_minimized,
      autoStartMixing: s.auto_start_mixing,
    };
  }

  /**
   * Unmap frontend settings to backend format
   */
  private unmapSettings(s: AppSettings): any {
    return {
      audio: {
        input_device_id: s.audio.inputDeviceId,
        output_device_id: s.audio.outputDeviceId,
        preview_device_id: s.audio.previewDeviceId,
        master_volume: s.audio.masterVolume,
        sample_rate: s.audio.sampleRate,
        buffer_size: s.audio.bufferSize,
        mic_monitoring: s.audio.micMonitoring,
        global_hotkeys_enabled: s.audio.globalHotkeysEnabled,
        mic_volume: s.audio.micVolume,
        soundboard_volume: s.audio.soundboardVolume,
      },
      start_minimized: s.startMinimized,
      auto_start_mixing: s.autoStartMixing,
    };
  }

  // =========================================================================
  // Mixer Configuration
  // =========================================================================

  /**
   * Get current mixer configuration
   */
  async getMixerConfig(): Promise<MixerConfig> {
    if (this.demoService.isDemoMode) {
      return DEMO_MIXER_CONFIG;
    }
    const config = await invoke<any>("get_mixer_config");
    return {
      masterVolume: config.master_volume,
      channels: config.channels.map((c: any) => ({
        id: c.id,
        name: c.name,
        channelType: c.channel_type,
        volume: c.volume,
        muted: c.muted,
        solo: c.solo,
      })),
      sampleRate: config.sample_rate,
      bufferSize: config.buffer_size,
    };
  }

  /**
   * Set master volume (0.0 to 1.0)
   */
  async setMasterVolume(volume: number): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("set_master_volume", {
      volume: Math.max(0, Math.min(1, volume)),
    });
  }

  // =========================================================================
  // Channel Management
  // =========================================================================

  /**
   * Add a microphone channel
   */
  async addMicrophoneChannel(id: string, name: string): Promise<MixerChannel> {
    if (this.demoService.isDemoMode) {
      return {
        id,
        name,
        channelType: "Microphone",
        volume: 1.0,
        muted: false,
        solo: false,
      };
    }
    return invoke<MixerChannel>("add_microphone_channel", { id, name });
  }

  /**
   * Add an audio file channel
   */
  async addAudioFileChannel(id: string, name: string): Promise<MixerChannel> {
    if (this.demoService.isDemoMode) {
      return {
        id,
        name,
        channelType: "AudioFile",
        volume: 1.0,
        muted: false,
        solo: false,
      };
    }
    return invoke<MixerChannel>("add_audio_file_channel", { id, name });
  }

  /**
   * Remove a channel
   */
  async removeChannel(channelId: string): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("remove_channel", { channelId });
  }

  /**
   * Set channel volume (0.0 to 2.0)
   */
  async setChannelVolume(channelId: string, volume: number): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("set_channel_volume", { channelId, volume });
  }

  /**
   * Toggle channel mute state
   */
  async toggleChannelMute(channelId: string): Promise<boolean> {
    if (this.demoService.isDemoMode) return false;
    return invoke<boolean>("toggle_channel_mute", { channelId });
  }

  // =========================================================================
  // Mixing Control
  // =========================================================================

  /**
   * Start audio mixing
   */
  async startMixing(): Promise<void> {
    if (this.demoService.isDemoMode) {
      this.demoMixing = true;
      return;
    }
    await invoke("start_mixing");
  }

  /**
   * Stop audio mixing
   */
  async stopMixing(): Promise<void> {
    if (this.demoService.isDemoMode) {
      this.demoMixing = false;
      return;
    }
    await invoke("stop_mixing");
  }

  /**
   * Check if currently mixing
   */
  async isMixing(): Promise<boolean> {
    if (this.demoService.isDemoMode) {
      return this.demoMixing;
    }
    return invoke<boolean>("is_mixing");
  }

  // =========================================================================
  // Sound Playback (Soundboard)
  // =========================================================================

  /**
   * Load and decode an audio file, returning its metadata
   */
  async loadSoundFile(path: string): Promise<SoundFile> {
    if (this.demoService.isDemoMode) {
      return createDemoSoundFile(path);
    }
    console.log("[TauriService] loadSoundFile called with path:", path);
    try {
      const result = await invoke<any>("load_sound_file", { path });
      console.log("[TauriService] loadSoundFile result:", result);
      return {
        id: result.id,
        name: result.name,
        path: result.path,
        duration: result.duration,
        sampleRate: result.sample_rate,
        channels: result.channels,
      };
    } catch (err) {
      console.error("[TauriService] loadSoundFile error:", err);
      throw err;
    }
  }

  /**
   * Load multiple audio files in parallel
   * Returns an array of results (success with SoundFile or error string)
   */
  async loadMultipleSoundFiles(
    paths: string[],
  ): Promise<Array<{ ok: SoundFile } | { err: string }>> {
    if (this.demoService.isDemoMode) {
      return paths.map((path) => ({ ok: createDemoSoundFile(path) }));
    }
    const results = await invoke<Array<{ Ok?: any; Err?: string }>>(
      "load_multiple_sound_files",
      { paths },
    );
    return results.map((r) => {
      if (r.Ok) {
        return {
          ok: {
            id: r.Ok.id,
            name: r.Ok.name,
            path: r.Ok.path,
            duration: r.Ok.duration,
            sampleRate: r.Ok.sample_rate,
            channels: r.Ok.channels,
          },
        };
      } else {
        return { err: r.Err || "Unknown error" };
      }
    });
  }

  /**
   * Play a sound file (mixed with microphone)
   * @param volume Volume level (0.0-2.0, default 1.0)
   * @param speed Playback speed (0.5-2.0, default 1.0)
   */
  async playSound(
    id: string,
    path: string,
    volume: number = 1.0,
    speed: number = 1.0,
  ): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("play_sound", { id, path, volume, speed });
  }

  /**
   * Stop a playing sound
   */
  async stopSound(id: string): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("stop_sound", { id });
  }

  /**
   * Stop all playing sounds
   */
  async stopAllSounds(): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("stop_all_sounds");
  }

  /**
   * Preview a sound on a specific output device
   */
  async previewSound(
    path: string,
    deviceName: string,
    padId: string,
  ): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("preview_sound", { path, deviceName, padId });
  }

  /**
   * Stop the currently playing preview
   */
  async stopPreview(): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("stop_preview");
  }

  /**
   * Get the currently previewing pad ID
   */
  async getPreviewState(): Promise<string | null> {
    if (this.demoService.isDemoMode) return null;
    return invoke<string | null>("get_preview_state");
  }

  /**
   * Set the preview output device
   */
  async setPreviewDevice(deviceId: string | null): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("set_preview_device", { deviceId });
  }

  /**
   * Set mic monitoring enabled/disabled
   */
  async setMicMonitoring(enabled: boolean): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("set_mic_monitoring", { enabled });
  }

  /**
   * Set microphone volume (0.0 to 2.0)
   */
  async setMicVolume(volume: number): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("set_mic_volume", {
      volume: Math.max(0, Math.min(2, volume)),
    });
  }

  /**
   * Mute or unmute microphone
   */
  async setMicMuted(muted: boolean): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("set_mic_muted", { muted });
  }

  // =========================================================================
  // Soundboard Persistence
  // =========================================================================

  /**
   * Save soundboard state to persistent storage
   */
  async saveSoundboardState(data: {
    sounds: Record<string, any>;
  }): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("save_soundboard", { pads: data });
  }

  /**
   * Load soundboard state from persistent storage
   */
  async loadSoundboardState(): Promise<any[] | null> {
    if (this.demoService.isDemoMode) {
      return DEMO_SOUNDBOARD_PADS;
    }
    return invoke<any[] | null>("load_soundboard");
  }

  // =========================================================================
  // Hash-based Sound Import
  // =========================================================================

  /**
   * Import a sound file and get its hash
   */
  async importSoundWithHash(path: string): Promise<ImportedSound> {
    if (this.demoService.isDemoMode) {
      return {
        hash: "demo_" + Math.random().toString(36).substring(7),
        name:
          path
            .split("/")
            .pop()
            ?.replace(/\.[^.]+$/, "") || "demo",
        path,
        duration: 5.0,
      };
    }
    return invoke<ImportedSound>("import_sound_with_hash", { path });
  }

  /**
   * Import multiple sound files with hashes
   */
  async importMultipleSoundsWithHash(
    paths: string[],
  ): Promise<Array<{ ok: ImportedSound } | { err: string }>> {
    if (this.demoService.isDemoMode) {
      return paths.map((path) => ({
        ok: {
          hash: "demo_" + Math.random().toString(36).substring(7),
          name:
            path
              .split("/")
              .pop()
              ?.replace(/\.[^.]+$/, "") || "demo",
          path,
          duration: 5.0,
        },
      }));
    }
    const results = await invoke<
      Array<{ Ok: ImportedSound } | { Err: string }>
    >("import_multiple_sounds_with_hash", { paths });
    return results.map((r) => ("Ok" in r ? { ok: r.Ok } : { err: r.Err }));
  }

  /**
   * Calculate SHA-256 hash of a file
   */
  async hashFile(path: string): Promise<string> {
    if (this.demoService.isDemoMode) {
      return "demo_" + Math.random().toString(36).substring(7);
    }
    return invoke<string>("hash_file", { path });
  }

  /**
   * Save folders to persistent storage
   */
  async saveFolders(folders: Folder[]): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("save_folders", { folders });
  }

  /**
   * Load folders from persistent storage
   */
  async loadFolders(): Promise<Folder[] | null> {
    if (this.demoService.isDemoMode) {
      return [{ id: "all", name: "Tous", createdAt: Date.now() }];
    }
    return invoke<Folder[] | null>("load_folders");
  }

  // =========================================================================
  // Image Management
  // =========================================================================

  /**
   * Signal for the cached images directory path
   * Returns null if not yet loaded, use in templates for reactivity
   */
  readonly imagesDir = this._imagesDirCache.asReadonly();

  /**
   * Get the images directory path (cached)
   */
  async getImagesDir(): Promise<string> {
    if (this.demoService.isDemoMode) {
      return "/demo/images";
    }

    // Return cached value if available
    const cached = this._imagesDirCache();
    if (cached) {
      return cached;
    }

    // Return existing promise if already loading
    if (this._imagesDirPromise) {
      return this._imagesDirPromise;
    }

    // Load and cache
    this._imagesDirPromise = invoke<string>("get_images_dir").then((dir) => {
      this._imagesDirCache.set(dir);
      return dir;
    });

    return this._imagesDirPromise;
  }

  /**
   * Save an image for a pad
   * @returns The relative path to the saved image
   */
  async savePadImage(
    padId: string,
    imageData: Uint8Array,
    extension: string,
  ): Promise<string> {
    if (this.demoService.isDemoMode) {
      return `${padId}-demo.${extension}`;
    }
    return invoke<string>("save_pad_image", {
      padId,
      imageData: Array.from(imageData),
      extension,
    });
  }

  /**
   * Delete image for a pad
   */
  async deletePadImage(padId: string): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("delete_pad_image", { padId });
  }

  /**
   * Clean up orphaned images
   * @returns Number of deleted images
   */
  async cleanupOrphanedImages(): Promise<number> {
    if (this.demoService.isDemoMode) return 0;
    return invoke<number>("cleanup_orphaned_images");
  }

  // =========================================================================
  // Preview Event Listeners
  // =========================================================================

  /**
   * Listen for preview started events
   */
  async listenPreviewStarted(
    callback: (padId: string) => void,
  ): Promise<() => void> {
    if (this.demoService.isDemoMode) {
      return () => {}; // No-op unlisten
    }
    const { listen } = await import("@tauri-apps/api/event");
    const unlisten = await listen<string>("preview-started", (event) => {
      callback(event.payload);
    });
    return unlisten;
  }

  /**
   * Listen for preview stopped events
   */
  async listenPreviewStopped(
    callback: (padId: string) => void,
  ): Promise<() => void> {
    if (this.demoService.isDemoMode) {
      return () => {}; // No-op unlisten
    }
    const { listen } = await import("@tauri-apps/api/event");
    const unlisten = await listen<string>("preview-stopped", (event) => {
      callback(event.payload);
    });
    return unlisten;
  }

  // =========================================================================
  // Shortcut Management
  // =========================================================================

  /**
   * Register a global shortcut for a sound
   */
  async registerGlobalShortcut(
    soundId: string,
    shortcut: string,
  ): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("register_global_shortcut", { soundId, shortcut });
  }

  /**
   * Unregister a global shortcut
   */
  async unregisterGlobalShortcut(shortcut: string): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("unregister_global_shortcut", { shortcut });
  }

  /**
   * Unregister all global shortcuts
   */
  async unregisterAllShortcuts(): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("unregister_all_shortcuts");
  }

  /**
   * Enable or disable global hotkeys
   */
  async setGlobalHotkeysEnabled(enabled: boolean): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("set_global_hotkeys_enabled", { enabled });
  }

  /**
   * Get current global hotkeys enabled state
   */
  async getGlobalHotkeysEnabled(): Promise<boolean> {
    if (this.demoService.isDemoMode) return true;
    return await invoke("get_global_hotkeys_enabled");
  }

  /**
   * Listen for global shortcut triggered events
   */
  async listenGlobalShortcut(
    callback: (data: { soundId: string; shortcut: string }) => void,
  ): Promise<() => void> {
    if (this.demoService.isDemoMode) {
      return () => {}; // No-op unlisten
    }
    const { listen } = await import("@tauri-apps/api/event");
    const unlisten = await listen<{ soundId: string; shortcut: string }>(
      "global-shortcut-triggered",
      (event) => {
        callback(event.payload);
      },
    );
    return unlisten;
  }

  // =========================================================================
  // Volume Control
  // =========================================================================

  /**
   * Set soundboard global volume (0.0 to 2.0)
   */
  async setSoundboardVolume(volume: number): Promise<void> {
    if (this.demoService.isDemoMode) return;
    await invoke("set_soundboard_volume", { volume });
  }

  // =========================================================================
  // Sound Import with Normalization
  // =========================================================================

  /**
   * Import and normalize a sound file to -3dB peak
   * @returns The imported sound with hash, name, path, and duration
   */
  async importAndNormalizeSound(
    path: string,
  ): Promise<{ hash: string; name: string; path: string; duration: number }> {
    if (this.demoService.isDemoMode) {
      return {
        hash: "demo_" + Math.random().toString(36).substring(7),
        name:
          path
            .split("/")
            .pop()
            ?.replace(/\.[^.]+$/, "") || "demo",
        path,
        duration: 5.0,
      };
    }
    return invoke("import_and_normalize_sound", { path });
  }

  /**
   * Import and normalize multiple sound files
   * @returns Array of results (success with sound data or error string)
   */
  async importMultipleAndNormalize(
    paths: string[],
  ): Promise<
    Array<
      | { ok: { hash: string; name: string; path: string; duration: number } }
      | { err: string }
    >
  > {
    const results = [];
    for (const path of paths) {
      try {
        const result = await this.importAndNormalizeSound(path);
        results.push({ ok: result });
      } catch (err) {
        results.push({ err: String(err) });
      }
    }
    return results;
  }
}
