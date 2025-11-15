/**
 * Audio engine service - handles audio playback and control
 */
import { Injectable } from '@angular/core';
import { BehaviorSubject, Observable } from 'rxjs';
import { TauriService } from './tauri.service';
import { AudioLevelEvent } from '../models/audio-level.model';
import { UnlistenFn } from '@tauri-apps/api/event';

@Injectable({
  providedIn: 'root'
})
export class AudioEngineService {
  private isRunningSubject = new BehaviorSubject<boolean>(false);
  private masterVolumeSubject = new BehaviorSubject<number>(1.0);
  private micVolumeSubject = new BehaviorSubject<number>(0.5);
  private effectsVolumeSubject = new BehaviorSubject<number>(0.5);
  private audioLevelSubject = new BehaviorSubject<AudioLevelEvent | null>(null);

  public isRunning$: Observable<boolean> = this.isRunningSubject.asObservable();
  public masterVolume$: Observable<number> = this.masterVolumeSubject.asObservable();
  public micVolume$: Observable<number> = this.micVolumeSubject.asObservable();
  public effectsVolume$: Observable<number> = this.effectsVolumeSubject.asObservable();
  public audioLevel$: Observable<AudioLevelEvent | null> = this.audioLevelSubject.asObservable();

  private audioLevelUnlisten?: UnlistenFn;

  constructor(private tauri: TauriService) {
    this.setupEventListeners();
  }

  /**
   * Setup event listeners for audio events
   */
  private async setupEventListeners(): Promise<void> {
    // Listen for audio level events
    this.audioLevelUnlisten = await this.tauri.listen<AudioLevelEvent>(
      'audio-level',
      (level) => {
        this.audioLevelSubject.next(level);
      }
    );
  }

  /**
   * Start the audio engine
   */
  async start(): Promise<void> {
    try {
      await this.tauri.invoke('start_audio_engine');
      this.isRunningSubject.next(true);
    } catch (error) {
      console.error('Failed to start audio engine:', error);
      throw error;
    }
  }

  /**
   * Stop the audio engine
   */
  async stop(): Promise<void> {
    try {
      await this.tauri.invoke('stop_audio_engine');
      this.isRunningSubject.next(false);
    } catch (error) {
      console.error('Failed to stop audio engine:', error);
      throw error;
    }
  }

  /**
   * Play a sound
   */
  async playSound(id: string, filePath: string): Promise<void> {
    try {
      await this.tauri.invoke('play_sound', { id, filePath });
    } catch (error) {
      console.error('Failed to play sound:', error);
      throw error;
    }
  }

  /**
   * Stop a playing sound
   */
  async stopSound(id: string): Promise<void> {
    try {
      await this.tauri.invoke('stop_sound', { id });
    } catch (error) {
      console.error('Failed to stop sound:', error);
      throw error;
    }
  }

  /**
   * Stop all playing sounds
   */
  async stopAllSounds(): Promise<void> {
    try {
      await this.tauri.invoke('stop_all_sounds');
    } catch (error) {
      console.error('Failed to stop all sounds:', error);
      throw error;
    }
  }

  /**
   * Set master volume
   */
  async setMasterVolume(volume: number): Promise<void> {
    try {
      await this.tauri.invoke('set_master_volume', { volume });
      this.masterVolumeSubject.next(volume);
    } catch (error) {
      console.error('Failed to set master volume:', error);
      throw error;
    }
  }

  /**
   * Set microphone volume
   */
  async setMicVolume(volume: number): Promise<void> {
    try {
      await this.tauri.invoke('set_mic_volume', { volume });
      this.micVolumeSubject.next(volume);
    } catch (error) {
      console.error('Failed to set mic volume:', error);
      throw error;
    }
  }

  /**
   * Set effects volume
   */
  async setEffectsVolume(volume: number): Promise<void> {
    try {
      await this.tauri.invoke('set_effects_volume', { volume });
      this.effectsVolumeSubject.next(volume);
    } catch (error) {
      console.error('Failed to set effects volume:', error);
      throw error;
    }
  }

  /**
   * Cleanup
   */
  ngOnDestroy(): void {
    if (this.audioLevelUnlisten) {
      this.audioLevelUnlisten();
    }
  }
}
