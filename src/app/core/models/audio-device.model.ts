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
  channelType: "Microphone" | "AudioFile" | "SystemAudio";
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
  globalHotkeysEnabled: boolean;
  micVolume: number;
  soundboardVolume: number;
  noiseSuppressionEnabled: boolean;
  voiceGateEnabled: boolean;
}

export interface AppSettings {
  audio: AudioSettings;
  startMinimized: boolean;
  autoStartMixing: boolean;
  updateChannel: 'stable' | 'beta';
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
  duration: number; // in seconds
  sampleRate: number;
  channels: number;
}

/**
 * Image attached to a sound pad
 */
export interface PadImage {
  /** Relative path in ~/.voiceboard/images/ */
  localPath: string;
  /** Original URL source (for attribution) */
  originalUrl?: string;
  /** Attribution text for the image source */
  attribution?: string;
}

/**
 * Sound entity identified by SHA-256 hash
 * Contains all properties previously on SoundPad
 */
export interface Sound {
  /** SHA-256 hash of file content (64 chars hex) */
  id: string;
  /** Filename without extension */
  name: string;
  /** Absolute path to audio file */
  path: string;
  /** Duration in seconds */
  duration: number;

  // User properties
  /** Volume level (0.0-2.0, default 1.0 = 100%) */
  volume: number;
  /** Playback speed (0.5-2.0, default 1.0 = normal) */
  speed: number;
  /** Keyboard shortcut */
  hotkey?: string;
  /** User-defined custom name */
  customName?: string;
  /** Custom image */
  image?: PadImage;
  /** Folder IDs this sound belongs to */
  folderIds: string[];

  // Runtime state (not persisted)
  /** Whether sound is currently playing */
  isPlaying: boolean;

  // Metadata
  /** Timestamp when sound was added */
  addedAt: number;
}

/**
 * Virtual pad for display (generated from sounds)
 */
export interface SoundPad {
  /** Position in grid (0, 1, 2, ...) */
  index: number;
  /** Reference to sound or null if empty */
  sound: Sound | null;
  /** Color generated from index */
  color: string;
  /** Indices of characters to highlight in search results */
  matchedIndices: number[];
}

/**
 * Folder for organizing sounds
 */
export interface Folder {
  id: string;
  name: string;
  createdAt: number; // timestamp
}
