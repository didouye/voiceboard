/**
 * Soundboard service - handles sound management
 */
import { Injectable } from '@angular/core';
import { BehaviorSubject, Observable } from 'rxjs';
import { TauriService } from './tauri.service';
import { Sound } from '../models/sound.model';

@Injectable({
  providedIn: 'root'
})
export class SoundboardService {
  private soundsSubject = new BehaviorSubject<Sound[]>([]);
  public sounds$: Observable<Sound[]> = this.soundsSubject.asObservable();

  constructor(private tauri: TauriService) {
    this.loadSounds();
  }

  /**
   * Load all sounds from Rust backend
   */
  async loadSounds(): Promise<void> {
    try {
      const sounds = await this.tauri.invoke<Sound[]>('get_sounds');
      this.soundsSubject.next(sounds);
    } catch (error) {
      console.error('Failed to load sounds:', error);
      throw error;
    }
  }

  /**
   * Get all sounds (synchronous from cache)
   */
  getSounds(): Sound[] {
    return this.soundsSubject.value;
  }

  /**
   * Add a new sound
   */
  async addSound(name: string, filePath: string): Promise<Sound> {
    try {
      const sound = await this.tauri.invoke<Sound>('add_sound', {
        name,
        filePath
      });

      // Update local cache
      const sounds = [...this.soundsSubject.value, sound];
      this.soundsSubject.next(sounds);

      return sound;
    } catch (error) {
      console.error('Failed to add sound:', error);
      throw error;
    }
  }

  /**
   * Delete a sound
   */
  async deleteSound(id: string): Promise<void> {
    try {
      await this.tauri.invoke('delete_sound', { id });

      // Update local cache
      const sounds = this.soundsSubject.value.filter(s => s.id !== id);
      this.soundsSubject.next(sounds);
    } catch (error) {
      console.error('Failed to delete sound:', error);
      throw error;
    }
  }

  /**
   * Rename a sound
   */
  async renameSound(id: string, name: string): Promise<void> {
    try {
      await this.tauri.invoke('rename_sound', { id, name });

      // Update local cache
      const sounds = this.soundsSubject.value.map(s =>
        s.id === id ? { ...s, name } : s
      );
      this.soundsSubject.next(sounds);
    } catch (error) {
      console.error('Failed to rename sound:', error);
      throw error;
    }
  }

  /**
   * Update sound volume
   */
  async updateVolume(id: string, volume: number): Promise<void> {
    try {
      await this.tauri.invoke('update_sound_volume', { id, volume });

      // Update local cache
      const sounds = this.soundsSubject.value.map(s =>
        s.id === id ? { ...s, volume } : s
      );
      this.soundsSubject.next(sounds);
    } catch (error) {
      console.error('Failed to update volume:', error);
      throw error;
    }
  }

  /**
   * Reorder sounds
   */
  async reorderSounds(soundIds: string[]): Promise<void> {
    try {
      await this.tauri.invoke('reorder_sounds', { ids: soundIds });

      // Update local cache with new order
      const soundMap = new Map(this.soundsSubject.value.map(s => [s.id, s]));
      const reorderedSounds = soundIds
        .map(id => soundMap.get(id))
        .filter((s): s is Sound => s !== undefined);

      this.soundsSubject.next(reorderedSounds);
    } catch (error) {
      console.error('Failed to reorder sounds:', error);
      throw error;
    }
  }

  /**
   * Filter sounds by name
   */
  async filterSounds(query: string): Promise<Sound[]> {
    try {
      return await this.tauri.invoke<Sound[]>('filter_sounds', { query });
    } catch (error) {
      console.error('Failed to filter sounds:', error);
      throw error;
    }
  }

  /**
   * Get sound count
   */
  async getSoundCount(): Promise<number> {
    try {
      return await this.tauri.invoke<number>('get_sound_count');
    } catch (error) {
      console.error('Failed to get sound count:', error);
      throw error;
    }
  }
}
