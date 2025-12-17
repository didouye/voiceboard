import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import {
  AudioDevice,
  MixerChannel,
  MixerConfig,
  ApiResponse
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
    return response.data;
  }

  /**
   * Get input devices only (microphones)
   */
  async getInputDevices(): Promise<AudioDevice[]> {
    const response = await invoke<ApiResponse<AudioDevice[]>>('get_input_devices');
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to get input devices');
    }
    return response.data;
  }

  /**
   * Check if virtual audio driver is installed
   */
  async checkVirtualDriver(): Promise<boolean> {
    const response = await invoke<ApiResponse<boolean>>('check_virtual_driver');
    return response.success && response.data === true;
  }

  // =========================================================================
  // Mixer Configuration
  // =========================================================================

  /**
   * Get current mixer configuration
   */
  async getMixerConfig(): Promise<MixerConfig> {
    return invoke<MixerConfig>('get_mixer_config');
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
}
