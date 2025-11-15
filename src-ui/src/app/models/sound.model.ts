/**
 * Sound model matching the Rust Sound struct
 */
export interface Sound {
  id: string;
  name: string;
  file_path: string;
  volume: number;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

/**
 * Request to add a new sound
 */
export interface AddSoundRequest {
  name: string;
  filePath: string;
}

/**
 * Request to rename a sound
 */
export interface RenameSoundRequest {
  id: string;
  name: string;
}

/**
 * Request to update sound volume
 */
export interface UpdateSoundVolumeRequest {
  id: string;
  volume: number;
}
