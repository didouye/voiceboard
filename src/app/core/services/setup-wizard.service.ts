import { Injectable, signal } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';

export type SetupStep = 'checking' | 'missing' | 'downloading' | 'installing' | 'done' | 'error' | 'skipped';

export interface VbCableStatus {
  installed: boolean;
  device_name: string | null;
}

export interface SetupState {
  step: SetupStep;
  error?: string;
}

@Injectable({ providedIn: 'root' })
export class SetupWizardService {
  state = signal<SetupState>({ step: 'checking' });

  async checkVbCable(): Promise<boolean> {
    this.state.set({ step: 'checking' });

    try {
      const status = await invoke<VbCableStatus>('check_vb_cable_installed');

      if (status.installed) {
        this.state.set({ step: 'done' });
        return true;
      } else {
        this.state.set({ step: 'missing' });
        return false;
      }
    } catch (error) {
      console.error('Failed to check VB-Cable:', error);
      this.state.set({ step: 'missing' });
      return false;
    }
  }

  async downloadAndInstall(): Promise<boolean> {
    this.state.set({ step: 'downloading' });

    try {
      // Small delay to show downloading state
      await new Promise(resolve => setTimeout(resolve, 500));
      this.state.set({ step: 'installing' });

      await invoke('download_and_install_vb_cable');

      this.state.set({ step: 'done' });
      return true;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      this.state.set({ step: 'error', error: errorMessage });
      return false;
    }
  }

  skip(): void {
    this.state.set({ step: 'skipped' });
  }

  reset(): void {
    this.state.set({ step: 'checking' });
  }
}
