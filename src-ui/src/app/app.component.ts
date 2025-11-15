import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterOutlet } from '@angular/router';
import { SoundboardComponent } from './components/soundboard/soundboard.component';
import { AudioEngineService } from './services/audio-engine.service';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [CommonModule, RouterOutlet, SoundboardComponent],
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss']
})
export class AppComponent implements OnInit {
  title = 'Voiceboard';

  constructor(private audioEngine: AudioEngineService) {}

  ngOnInit(): void {
    // Initialize audio engine on app start
    this.initializeAudioEngine();
  }

  private async initializeAudioEngine(): Promise<void> {
    try {
      await this.audioEngine.start();
      console.log('Audio engine started successfully');
    } catch (error) {
      console.error('Failed to start audio engine:', error);
    }
  }
}
