import { Injectable, signal } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

export interface BinaryStatus {
  ytdlp_installed: boolean;
  ffmpeg_installed: boolean;
  all_installed: boolean;
}

export interface BinaryDownloadProgress {
  binary: string;
  downloaded: number;
  total: number | null;
  done: boolean;
}

@Injectable({ providedIn: 'root' })
export class BinaryManagerService {
  status = signal<BinaryStatus | null>(null);
  installing = signal(false);
  progress = signal<BinaryDownloadProgress | null>(null);
  updateAvailable = signal<string | null>(null);
  error = signal<string | null>(null);

  private progressUnlisten: UnlistenFn | null = null;

  async checkStatus(): Promise<BinaryStatus> {
    const status = await invoke<BinaryStatus>('check_binaries_status');
    this.status.set(status);
    return status;
  }

  async install(): Promise<void> {
    this.installing.set(true);
    this.error.set(null);
    this.progress.set(null);

    // Listen for progress events
    this.progressUnlisten = await listen<BinaryDownloadProgress>(
      'binary-download-progress',
      (event) => {
        this.progress.set(event.payload);
      }
    );

    try {
      await invoke('install_binaries');
      await this.checkStatus();
    } catch (err: any) {
      this.error.set(err?.message || err || 'Installation failed');
      throw err;
    } finally {
      this.installing.set(false);
      if (this.progressUnlisten) {
        this.progressUnlisten();
        this.progressUnlisten = null;
      }
    }
  }

  async checkForUpdate(): Promise<string | null> {
    const version = await invoke<string | null>('check_ytdlp_update');
    this.updateAvailable.set(version);
    return version;
  }

  async updateYtdlp(): Promise<void> {
    await invoke('update_ytdlp');
    this.updateAvailable.set(null);
  }

  async listenForUpdateNotification(): Promise<void> {
    await listen<string>('ytdlp-update-available', (event) => {
      this.updateAvailable.set(event.payload);
    });
  }
}
