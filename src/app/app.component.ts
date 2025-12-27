import { Component, OnInit, inject } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { MixerComponent } from './features/mixer/mixer.component';
import { ToastComponent } from './core/components/toast/toast.component';
import { DebugConsoleComponent } from './core/components/debug-console/debug-console.component';
import { ToastService } from './core/services/toast.service';
import { DebugConsoleService } from './core/services/debug-console.service';

interface UpdateInfo {
  available: boolean;
  version?: string;
  body?: string;
}

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [MixerComponent, ToastComponent, DebugConsoleComponent],
  template: `
    <app-mixer />
    <app-toast />
    <app-debug-console />
  `,
  styles: [`
    :host {
      display: block;
      min-height: 100vh;
    }
  `]
})
export class AppComponent implements OnInit {
  private toastService = inject(ToastService);
  private debugConsole = inject(DebugConsoleService);

  async ngOnInit() {
    await this.checkForUpdate();
  }

  private async checkForUpdate() {
    this.debugConsole.log('info', 'Checking for updates...');
    try {
      const update = await invoke<UpdateInfo>('check_for_update');

      if (update.available && update.version) {
        this.debugConsole.log('info', `Update available: v${update.version}`);
        this.toastService.show({
          message: `Update available: v${update.version}`,
          action: {
            label: 'Update now',
            callback: () => this.installUpdate()
          },
          duration: 10000
        });
      } else {
        this.debugConsole.log('info', 'No update available');
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      this.debugConsole.log('error', `Update check failed: ${errorMessage}`);
      console.error('Update check failed:', errorMessage);
    }
  }

  private async installUpdate() {
    this.debugConsole.log('info', 'Starting update installation...');
    try {
      await invoke('install_update');
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      this.debugConsole.log('error', `Update installation failed: ${errorMessage}`);
      console.error('Update installation failed:', errorMessage);
      this.toastService.show({
        message: `Update failed: ${errorMessage}`,
        duration: 10000
      });
    }
  }
}
