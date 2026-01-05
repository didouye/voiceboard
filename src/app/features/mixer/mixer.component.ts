import { Component, OnInit, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MixerService } from '../../core/services';
import { SoundboardService } from '../../core/services/soundboard.service';
import { SoundboardComponent } from '../soundboard/soundboard.component';
import { StatusBarComponent } from './status-bar/status-bar.component';
import { SettingsPopupComponent } from '../../shared/components/settings-popup/settings-popup.component';

@Component({
  selector: 'app-mixer',
  standalone: true,
  imports: [CommonModule, SoundboardComponent, StatusBarComponent, SettingsPopupComponent],
  template: `
    <div class="h-screen flex flex-col bg-background">
      <!-- Main content area -->
      <div class="flex-1 flex overflow-hidden">
        <!-- Sidebar -->
        <aside class="w-48 bg-surface border-r border-border flex flex-col">
          <!-- Folders header -->
          <div class="px-4 py-3 border-b border-border">
            <h2 class="text-xs font-semibold text-text-muted uppercase tracking-wider flex items-center gap-2">
              <span>&#128193;</span> Folders
            </h2>
          </div>

          <!-- Folder list -->
          <div class="flex-1 py-2 overflow-y-auto">
            @for (folder of soundboard.folders(); track folder.id) {
              <button
                class="w-full px-4 py-2.5 text-left text-sm transition-colors flex items-center gap-2"
                [class]="folder.id === soundboard.activeFolderId()
                  ? 'bg-surface-hover text-text-primary border-l-2 border-accent'
                  : 'text-text-secondary hover:bg-surface-hover hover:text-text-primary border-l-2 border-transparent'"
                (click)="soundboard.setActiveFolder(folder.id)"
              >
                <span>&#9654;</span>
                {{ folder.name }}
              </button>
            }

            <!-- New folder button (disabled for now) -->
            <button
              class="w-full px-4 py-2.5 text-left text-sm text-text-muted border-l-2 border-transparent opacity-50 cursor-not-allowed flex items-center gap-2"
              disabled
              title="Coming soon"
            >
              <span>+</span>
              New Folder
            </button>
          </div>

          <!-- Settings button -->
          <div class="p-3 border-t border-border">
            <button
              class="w-full px-4 py-2.5 rounded-lg bg-surface-hover text-text-secondary hover:text-text-primary transition-colors flex items-center gap-2"
              (click)="showSettings.set(true)"
            >
              <span>&#9881;</span>
              Settings
            </button>
          </div>
        </aside>

        <!-- Main content -->
        <main class="flex-1 flex flex-col overflow-hidden">
          <!-- Error banner -->
          @if (mixer.error()) {
            <div class="mx-4 mt-4 px-4 py-3 bg-status-error/20 border border-status-error rounded-lg flex items-center justify-between">
              <span class="text-status-error">{{ mixer.error() }}</span>
              <button
                class="px-3 py-1 text-sm text-status-error hover:bg-status-error/20 rounded transition-colors"
                (click)="mixer.clearError()"
              >
                Dismiss
              </button>
            </div>
          }

          <!-- Soundboard -->
          <div class="flex-1 p-4 overflow-y-auto">
            <app-soundboard />
          </div>
        </main>
      </div>

      <!-- Status bar -->
      <app-status-bar />

      <!-- Settings popup -->
      @if (showSettings()) {
        <app-settings-popup (close)="showSettings.set(false)" />
      }
    </div>
  `,
  styles: []
})
export class MixerComponent implements OnInit {
  showSettings = signal(false);

  constructor(
    public mixer: MixerService,
    public soundboard: SoundboardService
  ) {}

  ngOnInit(): void {
    this.mixer.initialize();
  }
}
