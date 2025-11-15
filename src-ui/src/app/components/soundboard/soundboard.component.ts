/**
 * Soundboard component - main container for sound management
 */
import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { SoundboardService } from '../../services/soundboard.service';
import { AudioEngineService } from '../../services/audio-engine.service';
import { Sound } from '../../models/sound.model';
import { Observable } from 'rxjs';

@Component({
  selector: 'app-soundboard',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './soundboard.component.html',
  styleUrls: ['./soundboard.component.scss']
})
export class SoundboardComponent implements OnInit {
  sounds$: Observable<Sound[]>;

  constructor(
    private soundboardService: SoundboardService,
    private audioEngineService: AudioEngineService
  ) {
    this.sounds$ = this.soundboardService.sounds$;
  }

  ngOnInit(): void {
    this.soundboardService.loadSounds();
  }

  /**
   * Handle add sound button click
   */
  async onAddSound(): Promise<void> {
    // TODO: Open file dialog using Tauri dialog plugin
    // For now, this is a placeholder
    console.log('Add sound clicked');
  }

  /**
   * Handle sound click - play the sound
   */
  async onPlaySound(sound: Sound): Promise<void> {
    try {
      await this.audioEngineService.playSound(sound.id, sound.file_path);
    } catch (error) {
      console.error('Failed to play sound:', error);
    }
  }

  /**
   * Handle delete sound
   */
  async onDeleteSound(id: string): Promise<void> {
    if (confirm('Are you sure you want to delete this sound?')) {
      try {
        await this.soundboardService.deleteSound(id);
      } catch (error) {
        console.error('Failed to delete sound:', error);
      }
    }
  }

  /**
   * Handle rename sound
   */
  async onRenameSound(sound: Sound): Promise<void> {
    const newName = prompt('Enter new name:', sound.name);
    if (newName && newName !== sound.name) {
      try {
        await this.soundboardService.renameSound(sound.id, newName);
      } catch (error) {
        console.error('Failed to rename sound:', error);
      }
    }
  }

  /**
   * Handle volume change
   */
  async onVolumeChange(sound: Sound, volume: number): Promise<void> {
    try {
      await this.soundboardService.updateVolume(sound.id, volume);
    } catch (error) {
      console.error('Failed to update volume:', error);
    }
  }
}
