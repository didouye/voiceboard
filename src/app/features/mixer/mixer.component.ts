import { Component, OnInit, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MixerService } from '../../core/services';
import { SoundboardService } from '../../core/services/soundboard.service';
import { SoundboardComponent } from '../soundboard/soundboard.component';
import { StatusBarComponent } from './status-bar/status-bar.component';
import { SettingsPopupComponent } from '../../shared/components/settings-popup/settings-popup.component';
import { Folder } from '../../core/models';

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
                [class]="getFolderClasses(folder)"
                (click)="soundboard.setActiveFolder(folder.id)"
                (contextmenu)="onFolderContextMenu($event, folder)"
                (dragover)="onFolderDragOver($event, folder)"
                (dragleave)="onFolderDragLeave($event)"
                (drop)="onFolderDrop($event, folder)"
              >
                <span>{{ folder.id === soundboard.activeFolderId() ? '&#9654;' : '&#128193;' }}</span>
                {{ folder.name }}
              </button>
            }

            <!-- New folder button -->
            <button
              class="w-full px-4 py-2.5 text-left text-sm text-text-secondary hover:bg-surface-hover hover:text-text-primary border-l-2 border-transparent transition-colors flex items-center gap-2"
              (click)="showNewFolderPopup.set(true)"
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

      <!-- New Folder Popup -->
      @if (showNewFolderPopup()) {
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" (click)="showNewFolderPopup.set(false)">
          <div class="bg-surface border border-border rounded-lg p-4 w-80 shadow-xl" (click)="$event.stopPropagation()">
            <h3 class="text-sm font-semibold text-text-primary mb-3">New Folder</h3>
            <input
              type="text"
              [value]="newFolderName()"
              (input)="newFolderName.set($any($event.target).value)"
              (keydown.enter)="createFolder()"
              (keydown.escape)="showNewFolderPopup.set(false)"
              placeholder="Folder name"
              class="w-full px-3 py-2 text-sm bg-surface-hover border border-border rounded text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent"
              autofocus
            >
            <div class="flex justify-end gap-2 mt-4">
              <button
                class="px-3 py-1.5 text-sm text-text-secondary hover:text-text-primary transition-colors"
                (click)="showNewFolderPopup.set(false)"
              >
                Cancel
              </button>
              <button
                class="px-3 py-1.5 text-sm bg-accent hover:bg-accent/80 text-white rounded transition-colors"
                [disabled]="!newFolderName().trim()"
                (click)="createFolder()"
              >
                Create
              </button>
            </div>
          </div>
        </div>
      }

      <!-- Folder Context Menu -->
      @if (contextMenuFolder()) {
        <div
          class="fixed z-50"
          [style.left.px]="contextMenuPosition().x"
          [style.top.px]="contextMenuPosition().y"
        >
          <div class="bg-surface border border-border rounded-lg shadow-xl py-1 min-w-32">
            <button
              class="w-full px-4 py-2 text-left text-sm text-text-secondary hover:bg-surface-hover hover:text-text-primary transition-colors"
              (click)="startRenamingFolder()"
            >
              Rename
            </button>
            <button
              class="w-full px-4 py-2 text-left text-sm text-status-error hover:bg-surface-hover transition-colors"
              (click)="deleteFolder()"
            >
              Delete
            </button>
          </div>
        </div>
        <!-- Backdrop to close menu -->
        <div class="fixed inset-0 z-40" (click)="contextMenuFolder.set(null)"></div>
      }

      <!-- Rename Folder Popup -->
      @if (editingFolderId()) {
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" (click)="editingFolderId.set(null)">
          <div class="bg-surface border border-border rounded-lg p-4 w-80 shadow-xl" (click)="$event.stopPropagation()">
            <h3 class="text-sm font-semibold text-text-primary mb-3">Rename Folder</h3>
            <input
              type="text"
              [value]="editingFolderName()"
              (input)="editingFolderName.set($any($event.target).value)"
              (keydown.enter)="confirmRenameFolder()"
              (keydown.escape)="editingFolderId.set(null)"
              class="w-full px-3 py-2 text-sm bg-surface-hover border border-border rounded text-text-primary focus:outline-none focus:border-accent"
              autofocus
            >
            <div class="flex justify-end gap-2 mt-4">
              <button
                class="px-3 py-1.5 text-sm text-text-secondary hover:text-text-primary transition-colors"
                (click)="editingFolderId.set(null)"
              >
                Cancel
              </button>
              <button
                class="px-3 py-1.5 text-sm bg-accent hover:bg-accent/80 text-white rounded transition-colors"
                [disabled]="!editingFolderName().trim()"
                (click)="confirmRenameFolder()"
              >
                Rename
              </button>
            </div>
          </div>
        </div>
      }
    </div>
  `,
  styles: []
})
export class MixerComponent implements OnInit {
  showSettings = signal(false);

  // New folder popup state
  showNewFolderPopup = signal(false);
  newFolderName = signal('');

  // Context menu state
  contextMenuFolder = signal<Folder | null>(null);
  contextMenuPosition = signal({ x: 0, y: 0 });

  // Rename folder state
  editingFolderId = signal<string | null>(null);
  editingFolderName = signal('');

  // Drag & drop state
  dragOverFolderId = signal<string | null>(null);

  constructor(
    public mixer: MixerService,
    public soundboard: SoundboardService
  ) {}

  ngOnInit(): void {
    this.mixer.initialize();
  }

  createFolder(): void {
    const name = this.newFolderName().trim();
    if (name) {
      this.soundboard.createFolder(name);
      this.newFolderName.set('');
      this.showNewFolderPopup.set(false);
    }
  }

  onFolderContextMenu(event: MouseEvent, folder: Folder): void {
    if (folder.id === 'all') return; // Can't modify "All" folder
    event.preventDefault();
    this.contextMenuFolder.set(folder);
    this.contextMenuPosition.set({ x: event.clientX, y: event.clientY });
  }

  startRenamingFolder(): void {
    const folder = this.contextMenuFolder();
    if (folder) {
      this.editingFolderId.set(folder.id);
      this.editingFolderName.set(folder.name);
      this.contextMenuFolder.set(null);
    }
  }

  deleteFolder(): void {
    const folder = this.contextMenuFolder();
    if (folder && confirm(`Delete folder "${folder.name}"? Sounds will not be deleted.`)) {
      this.soundboard.deleteFolder(folder.id);
    }
    this.contextMenuFolder.set(null);
  }

  confirmRenameFolder(): void {
    const folderId = this.editingFolderId();
    const newName = this.editingFolderName().trim();
    if (folderId && newName) {
      this.soundboard.renameFolder(folderId, newName);
      this.editingFolderId.set(null);
    }
  }

  getFolderClasses(folder: Folder): string {
    const isActive = folder.id === this.soundboard.activeFolderId();
    const isDragOver = folder.id === this.dragOverFolderId() && folder.id !== 'all';

    let classes = '';
    if (isActive) {
      classes = 'bg-surface-hover text-text-primary border-l-2 border-accent';
    } else if (isDragOver) {
      classes = 'bg-accent/20 text-text-primary border-l-2 border-accent';
    } else {
      classes = 'text-text-secondary hover:bg-surface-hover hover:text-text-primary border-l-2 border-transparent';
    }
    return classes;
  }

  onFolderDragOver(event: DragEvent, folder: Folder): void {
    if (folder.id === 'all') return;
    event.preventDefault();
    event.dataTransfer!.dropEffect = 'copy';
    this.dragOverFolderId.set(folder.id);
  }

  onFolderDragLeave(event: DragEvent): void {
    this.dragOverFolderId.set(null);
  }

  onFolderDrop(event: DragEvent, folder: Folder): void {
    event.preventDefault();
    this.dragOverFolderId.set(null);

    if (folder.id === 'all') return;

    const soundId = event.dataTransfer?.getData('text/plain');
    if (soundId) {
      this.soundboard.addSoundToFolder(soundId, folder.id);
    }
  }
}
