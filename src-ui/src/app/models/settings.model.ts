/**
 * Application settings
 */
export interface AppSettings {
  selectedInputDevice?: string;
  selectedOutputDevice?: string;
  masterVolume: number;
  micVolume: number;
  effectsVolume: number;
}

/**
 * Audio configuration
 */
export interface AudioConfig {
  sample_rate: number;
  channels: number;
  buffer_size: number;
  master_volume: number;
  mic_volume: number;
  effects_volume: number;
}
