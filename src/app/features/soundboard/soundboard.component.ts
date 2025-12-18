import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { SoundboardService } from '../../core/services/soundboard.service';
import { SoundPadComponent } from './sound-pad/sound-pad.component';

@Component({
  selector: 'app-soundboard',
  standalone: true,
  imports: [CommonModule, SoundPadComponent],
  template: `
    <div class="soundboard-container">
      <div class="soundboard-header">
        <h2>Soundboard</h2>
        <div class="header-actions">
          @if (soundboard.playingCount() > 0) {
            <button class="btn-stop-all" (click)="soundboard.stopAll()">
              Stop All ({{ soundboard.playingCount() }})
            </button>
          }
          <button class="btn-add-pads" (click)="soundboard.addPads(4)">+ Add Pads</button>
        </div>
      </div>

      @if (soundboard.error()) {
        <div class="error-message">
          {{ soundboard.error() }}
          <button (click)="soundboard.clearError()">Dismiss</button>
        </div>
      }

      <div class="pads-grid">
        @for (pad of soundboard.pads(); track pad.id) {
          <app-sound-pad
            [pad]="pad"
            [loading]="soundboard.loading()"
            (play)="soundboard.playSound(pad.id)"
            (import)="soundboard.importSound(pad.id)"
            (remove)="soundboard.removeSound(pad.id)"
          />
        }
      </div>
    </div>
  `,
  styles: [`
    .soundboard-container {
      background: rgba(255, 255, 255, 0.03);
      border-radius: 12px;
      padding: 20px;
    }

    .soundboard-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 20px;
    }

    .soundboard-header h2 {
      margin: 0;
      font-size: 1.2rem;
      color: #aaa;
    }

    .header-actions {
      display: flex;
      gap: 10px;
    }

    .btn-stop-all {
      background: #e74c3c;
      border: none;
      color: white;
      padding: 8px 16px;
      border-radius: 6px;
      cursor: pointer;
      font-size: 0.85rem;
      transition: background 0.2s;
    }

    .btn-stop-all:hover {
      background: #c0392b;
    }

    .btn-add-pads {
      background: rgba(255, 255, 255, 0.1);
      border: 1px solid rgba(255, 255, 255, 0.2);
      color: #aaa;
      padding: 8px 16px;
      border-radius: 6px;
      cursor: pointer;
      font-size: 0.85rem;
      transition: all 0.2s;
    }

    .btn-add-pads:hover {
      background: rgba(255, 255, 255, 0.15);
      color: #fff;
    }

    .error-message {
      background: rgba(231, 76, 60, 0.2);
      border: 1px solid #e74c3c;
      color: #e74c3c;
      padding: 10px 15px;
      border-radius: 6px;
      margin-bottom: 15px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      font-size: 0.9rem;
    }

    .error-message button {
      background: transparent;
      border: 1px solid #e74c3c;
      color: #e74c3c;
      padding: 4px 10px;
      border-radius: 4px;
      cursor: pointer;
      font-size: 0.8rem;
    }

    .pads-grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
      gap: 12px;
    }

    @media (max-width: 600px) {
      .pads-grid {
        grid-template-columns: repeat(3, 1fr);
      }
    }
  `]
})
export class SoundboardComponent {
  constructor(public soundboard: SoundboardService) {}
}
