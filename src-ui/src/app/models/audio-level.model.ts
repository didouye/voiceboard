/**
 * Audio level information for visualization
 */
export interface AudioLevel {
  rms_db: number;
  peak_db: number;
  rms_linear: number;
  peak_linear: number;
}

/**
 * Audio level event from Rust
 */
export interface AudioLevelEvent {
  mic_level: AudioLevel;
  output_level: AudioLevel;
}
