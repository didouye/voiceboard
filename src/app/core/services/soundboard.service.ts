import { Injectable, signal, computed } from '@angular/core';
import { TauriService } from './tauri.service';
import { SoundFile, SoundPad } from '../models';
import { open } from '@tauri-apps/plugin-dialog';

const PAD_COLORS = [
  '#e74c3c', '#e67e22', '#f1c40f', '#2ecc71',
  '#1abc9c', '#3498db', '#9b59b6', '#e91e63',
  '#00bcd4', '#8bc34a', '#ff5722', '#795548'
];

/** Serializable pad data for persistence */
interface SavedPad {
  id: string;
  sound: SoundFile | null;
  color: string;
  hotkey?: string;
}

@Injectable({
  providedIn: 'root'
})
export class SoundboardService {
  // State signals
  private _pads = signal<SoundPad[]>(this.createInitialPads(12));
  private _loading = signal(false);
  private _error = signal<string | null>(null);
  private _initialized = false;

  // Public readonly signals
  readonly pads = this._pads.asReadonly();
  readonly loading = this._loading.asReadonly();
  readonly error = this._error.asReadonly();

  // Computed
  readonly activePads = computed(() => this._pads().filter(p => p.sound !== null));
  readonly playingCount = computed(() => this._pads().filter(p => p.isPlaying).length);

  constructor(private tauri: TauriService) {
    // Load saved state on construction
    this.loadState();
  }

  private createInitialPads(count: number): SoundPad[] {
    return Array.from({ length: count }, (_, i) => ({
      id: `pad-${i}`,
      sound: null,
      color: PAD_COLORS[i % PAD_COLORS.length],
      isPlaying: false
    }));
  }

  /**
   * Load soundboard state from persistent storage
   */
  private async loadState(): Promise<void> {
    try {
      const saved = await this.tauri.loadSoundboardState();
      if (saved && saved.length > 0) {
        // Restore pads, ensuring isPlaying is false
        const restoredPads: SoundPad[] = saved.map(p => ({
          ...p,
          isPlaying: false
        }));
        this._pads.set(restoredPads);
        console.log(`Loaded ${saved.filter(p => p.sound).length} sounds from storage`);
      }
    } catch (err) {
      console.error('Failed to load soundboard state:', err);
    }
    this._initialized = true;
  }

  /**
   * Save soundboard state to persistent storage
   */
  private async saveState(): Promise<void> {
    if (!this._initialized) return;

    try {
      const padsToSave: SavedPad[] = this._pads().map(p => ({
        id: p.id,
        sound: p.sound,
        color: p.color,
        hotkey: p.hotkey
      }));
      await this.tauri.saveSoundboardState(padsToSave);
    } catch (err) {
      console.error('Failed to save soundboard state:', err);
    }
  }

  /**
   * Add more pads to the soundboard
   */
  addPads(count: number = 4): void {
    const current = this._pads();
    const startIndex = current.length;
    const newPads = Array.from({ length: count }, (_, i) => ({
      id: `pad-${startIndex + i}`,
      sound: null,
      color: PAD_COLORS[(startIndex + i) % PAD_COLORS.length],
      isPlaying: false
    }));
    this._pads.set([...current, ...newPads]);
    this.saveState();
  }

  /**
   * Import a sound file to a specific pad
   */
  async importSound(padId: string): Promise<void> {
    try {
      this._loading.set(true);
      this._error.set(null);

      // Open file dialog
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Audio Files',
          extensions: ['mp3', 'ogg', 'wav', 'flac']
        }]
      });

      if (!selected) {
        this._loading.set(false);
        return; // User cancelled
      }

      const path = selected as string;

      // Load and decode the file
      const soundFile = await this.tauri.loadSoundFile(path);

      // Update the pad with the sound
      this._pads.update(pads => pads.map(pad =>
        pad.id === padId
          ? { ...pad, sound: soundFile }
          : pad
      ));

      // Persist the change
      await this.saveState();
    } catch (err) {
      this._error.set(err instanceof Error ? err.message : 'Failed to import sound');
      console.error('Import sound error:', err);
    } finally {
      this._loading.set(false);
    }
  }

  /**
   * Play a sound from a pad
   */
  async playSound(padId: string): Promise<void> {
    const pad = this._pads().find(p => p.id === padId);
    if (!pad?.sound) return;

    try {
      // If already playing, stop it first
      if (pad.isPlaying) {
        await this.stopSound(padId);
        return;
      }

      // Mark as playing
      this._pads.update(pads => pads.map(p =>
        p.id === padId ? { ...p, isPlaying: true } : p
      ));

      // Play the sound
      await this.tauri.playSound(pad.sound.id, pad.sound.path);

      // Auto-stop after duration (with small buffer)
      setTimeout(() => {
        this._pads.update(pads => pads.map(p =>
          p.id === padId ? { ...p, isPlaying: false } : p
        ));
      }, (pad.sound.duration + 0.5) * 1000);

    } catch (err) {
      this._error.set(err instanceof Error ? err.message : 'Failed to play sound');
      this._pads.update(pads => pads.map(p =>
        p.id === padId ? { ...p, isPlaying: false } : p
      ));
    }
  }

  /**
   * Stop a playing sound
   */
  async stopSound(padId: string): Promise<void> {
    const pad = this._pads().find(p => p.id === padId);
    if (!pad?.sound) return;

    try {
      await this.tauri.stopSound(pad.sound.id);
      this._pads.update(pads => pads.map(p =>
        p.id === padId ? { ...p, isPlaying: false } : p
      ));
    } catch (err) {
      this._error.set(err instanceof Error ? err.message : 'Failed to stop sound');
    }
  }

  /**
   * Stop all playing sounds
   */
  async stopAll(): Promise<void> {
    const playingPads = this._pads().filter(p => p.isPlaying);
    await Promise.all(playingPads.map(p => this.stopSound(p.id)));
  }

  /**
   * Remove sound from a pad
   */
  removeSound(padId: string): void {
    this._pads.update(pads => pads.map(pad =>
      pad.id === padId
        ? { ...pad, sound: null, isPlaying: false }
        : pad
    ));
    this.saveState();
  }

  /**
   * Change pad color
   */
  setPadColor(padId: string, color: string): void {
    this._pads.update(pads => pads.map(pad =>
      pad.id === padId ? { ...pad, color } : pad
    ));
    this.saveState();
  }

  /**
   * Clear any error
   */
  clearError(): void {
    this._error.set(null);
  }

  /**
   * Format duration as mm:ss
   */
  formatDuration(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }
}
