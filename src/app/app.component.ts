import { Component, OnInit, inject } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { MixerComponent } from './features/mixer/mixer.component';
import { ToastComponent } from './core/components/toast/toast.component';
import { ToastService } from './core/services/toast.service';

interface UpdateInfo {
  available: boolean;
  version?: string;
  body?: string;
}

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [MixerComponent, ToastComponent],
  template: `
    <app-mixer />
    <app-toast />
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

  async ngOnInit() {
    await this.checkForUpdate();
  }

  private async checkForUpdate() {
    try {
      const update = await invoke<UpdateInfo>('check_for_update');

      if (update.available && update.version) {
        this.toastService.show({
          message: `Update available: v${update.version}`,
          action: {
            label: 'Update now',
            callback: () => this.installUpdate()
          },
          duration: 10000
        });
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      console.error('Update check failed:', errorMessage);
      // Don't show toast for check failures, just log
    }
  }

  private async installUpdate() {
    try {
      await invoke('install_update');
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      console.error('Update installation failed:', errorMessage);
      this.toastService.show({
        message: `Update failed: ${errorMessage}`,
        duration: 10000
      });
    }
  }
}
