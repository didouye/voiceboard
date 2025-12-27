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
      console.warn('Update check failed:', error);
    }
  }

  private async installUpdate() {
    try {
      await invoke('install_update');
    } catch (error) {
      this.toastService.show({
        message: 'Update failed, try again later',
        duration: 5000
      });
    }
  }
}
