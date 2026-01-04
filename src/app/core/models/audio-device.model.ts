/**
 * Audio device models matching the Rust backend DTOs
 */

export interface AudioDevice {
  id: string;
  name: string;
  deviceType: string;
  isDefault: boolean;
  isVirtual: boolean;
}

export interface MixerChannel {
  id: string;
  name: string;
  channelType: 'Microphone' | 'AudioFile' | 'SystemAudio';
  volume: number;
  muted: boolean;
  solo: boolean;
}

export interface MixerConfig {
  masterVolume: number;
  channels: MixerChannel[];
  sampleRate: number;
  bufferSize: number;
}

export interface AudioSettings {
  inputDeviceId: string | null;
  outputDeviceId: string | null;
  previewDeviceId: string | null;
  masterVolume: number;
  sampleRate: number;
  bufferSize: number;
  micMonitoring: boolean;
}

export interface AppSettings {
  audio: AudioSettings;
  startMinimized: boolean;
  autoStartMixing: boolean;
}

export interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

/**
 * Sound file metadata for the soundboard
 */
export interface SoundFile {
  id: string;
  name: string;
  path: string;
  duration: number;  // in seconds
  sampleRate: number;
  channels: number;
}

/**
 * Sound pad configuration (position + sound)
 */
export interface SoundPad {
  id: string;
  sound: SoundFile | null;
  color: string;
  hotkey?: string;
  isPlaying: boolean;
  /** Volume level (0.0-2.0, default 1.0 = 100%) */
  volume: number;
  /** Playback speed (0.5-2.0, default 1.0 = normal) */
  speed: number;
}
