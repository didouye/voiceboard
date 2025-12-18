import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import {
  AudioDevice,
  MixerChannel,
  MixerConfig,
  AppSettings,
  ApiResponse,
  SoundFile
} from '../models';

/**
 * Service for communicating with the Tauri/Rust backend
 */
@Injectable({
  providedIn: 'root'
})
export class TauriService {

  // =========================================================================
  // Device Management
  // =========================================================================

  /**
   * Get all available audio devices
   */
  async getAudioDevices(): Promise<AudioDevice[]> {
    const response = await invoke<ApiResponse<AudioDevice[]>>('get_audio_devices');
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to get audio devices');
    }
    return this.mapDevices(response.data);
  }

  /**
   * Get input devices only (microphones)
   */
  async getInputDevices(): Promise<AudioDevice[]> {
    const response = await invoke<ApiResponse<AudioDevice[]>>('get_input_devices');
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to get input devices');
    }
    return this.mapDevices(response.data);
  }

  /**
   * Get virtual output devices (for sending mixed audio)
   */
  async getVirtualOutputDevices(): Promise<AudioDevice[]> {
    const response = await invoke<ApiResponse<AudioDevice[]>>('get_virtual_output_devices');
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to get virtual output devices');
    }
    return this.mapDevices(response.data);
  }

  /**
   * Check if virtual audio driver is installed
   */
  async checkVirtualDriver(): Promise<boolean> {
    const response = await invoke<ApiResponse<boolean>>('check_virtual_driver');
    return response.success && response.data === true;
  }

  /**
   * Map backend device DTOs to frontend model (handle snake_case to camelCase)
   */
  private mapDevices(devices: any[]): AudioDevice[] {
    return devices.map(d => ({
      id: d.id,
      name: d.name,
      deviceType: d.device_type,
      isDefault: d.is_default,
      isVirtual: d.is_virtual
    }));
  }

  // =========================================================================
  // Settings Management
  // =========================================================================

  /**
   * Get current application settings
   */
  async getSettings(): Promise<AppSettings> {
    const settings = await invoke<any>('get_settings');
    return this.mapSettings(settings);
  }

  /**
   * Save application settings
   */
  async saveSettings(settings: AppSettings): Promise<void> {
    await invoke('save_settings', { settings: this.unmapSettings(settings) });
  }

  /**
   * Load settings from persistent storage
   */
  async loadSettings(): Promise<AppSettings> {
    const settings = await invoke<any>('load_settings');
    return this.mapSettings(settings);
  }

  /**
   * Set input device (microphone)
   */
  async setInputDevice(deviceId: string | null): Promise<void> {
    await invoke('set_input_device', { deviceId });
  }

  /**
   * Set output device (virtual microphone)
   */
  async setOutputDevice(deviceId: string | null): Promise<void> {
    await invoke('set_output_device', { deviceId });
  }

  /**
   * Map backend settings to frontend model
   */
  private mapSettings(s: any): AppSettings {
    return {
      audio: {
        inputDeviceId: s.audio.input_device_id,
        outputDeviceId: s.audio.output_device_id,
        masterVolume: s.audio.master_volume,
        sampleRate: s.audio.sample_rate,
        bufferSize: s.audio.buffer_size
      },
      startMinimized: s.start_minimized,
      autoStartMixing: s.auto_start_mixing
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
        master_volume: s.audio.masterVolume,
        sample_rate: s.audio.sampleRate,
        buffer_size: s.audio.bufferSize
      },
      start_minimized: s.startMinimized,
      auto_start_mixing: s.autoStartMixing
    };
  }

  // =========================================================================
  // Mixer Configuration
  // =========================================================================

  /**
   * Get current mixer configuration
   */
  async getMixerConfig(): Promise<MixerConfig> {
    const config = await invoke<any>('get_mixer_config');
    return {
      masterVolume: config.master_volume,
      channels: config.channels.map((c: any) => ({
        id: c.id,
        name: c.name,
        channelType: c.channel_type,
        volume: c.volume,
        muted: c.muted,
        solo: c.solo
      })),
      sampleRate: config.sample_rate,
      bufferSize: config.buffer_size
    };
  }

  /**
   * Set master volume (0.0 to 1.0)
   */
  async setMasterVolume(volume: number): Promise<void> {
    await invoke('set_master_volume', { volume: Math.max(0, Math.min(1, volume)) });
  }

  // =========================================================================
  // Channel Management
  // =========================================================================

  /**
   * Add a microphone channel
   */
  async addMicrophoneChannel(id: string, name: string): Promise<MixerChannel> {
    return invoke<MixerChannel>('add_microphone_channel', { id, name });
  }

  /**
   * Add an audio file channel
   */
  async addAudioFileChannel(id: string, name: string): Promise<MixerChannel> {
    return invoke<MixerChannel>('add_audio_file_channel', { id, name });
  }

  /**
   * Remove a channel
   */
  async removeChannel(channelId: string): Promise<void> {
    await invoke('remove_channel', { channelId });
  }

  /**
   * Set channel volume (0.0 to 2.0)
   */
  async setChannelVolume(channelId: string, volume: number): Promise<void> {
    await invoke('set_channel_volume', { channelId, volume });
  }

  /**
   * Toggle channel mute state
   */
  async toggleChannelMute(channelId: string): Promise<boolean> {
    return invoke<boolean>('toggle_channel_mute', { channelId });
  }

  // =========================================================================
  // Mixing Control
  // =========================================================================

  /**
   * Start audio mixing
   */
  async startMixing(): Promise<void> {
    await invoke('start_mixing');
  }

  /**
   * Stop audio mixing
   */
  async stopMixing(): Promise<void> {
    await invoke('stop_mixing');
  }

  /**
   * Check if currently mixing
   */
  async isMixing(): Promise<boolean> {
    return invoke<boolean>('is_mixing');
  }

  // =========================================================================
  // Sound Playback (Soundboard)
  // =========================================================================

  /**
   * Load and decode an audio file, returning its metadata
   */
  async loadSoundFile(path: string): Promise<SoundFile> {
    const result = await invoke<any>('load_sound_file', { path });
    return {
      id: result.id,
      name: result.name,
      path: result.path,
      duration: result.duration,
      sampleRate: result.sample_rate,
      channels: result.channels
    };
  }

  /**
   * Play a sound file (mixed with microphone)
   */
  async playSound(id: string, path: string): Promise<void> {
    await invoke('play_sound', { id, path });
  }

  /**
   * Stop a playing sound
   */
  async stopSound(id: string): Promise<void> {
    await invoke('stop_sound', { id });
  }

  /**
   * Set microphone volume (0.0 to 2.0)
   */
  async setMicVolume(volume: number): Promise<void> {
    await invoke('set_mic_volume', { volume: Math.max(0, Math.min(2, volume)) });
  }

  /**
   * Mute or unmute microphone
   */
  async setMicMuted(muted: boolean): Promise<void> {
    await invoke('set_mic_muted', { muted });
  }

  // =========================================================================
  // Soundboard Persistence
  // =========================================================================

  /**
   * Save soundboard state to persistent storage
   */
  async saveSoundboardState(pads: any[]): Promise<void> {
    await invoke('save_soundboard', { pads });
  }

  /**
   * Load soundboard state from persistent storage
   */
  async loadSoundboardState(): Promise<any[] | null> {
    return invoke<any[] | null>('load_soundboard');
  }
}
