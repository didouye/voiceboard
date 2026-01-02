import { Component, inject, output } from '@angular/core';
import { SetupWizardService } from '../../services/setup-wizard.service';

@Component({
  selector: 'app-setup-wizard',
  standalone: true,
  template: `
    <div class="setup-overlay">
      <div class="setup-modal">
        <h2>Setup Required</h2>

        @switch (setupService.state().step) {
          @case ('checking') {
            <div class="setup-content">
              <div class="spinner"></div>
              <p>Checking audio devices...</p>
            </div>
          }

          @case ('missing') {
            <div class="setup-content">
              <div class="warning-icon">⚠️</div>
              <h3>Virtual Audio Driver Not Found</h3>
              <p>
                Voiceboard needs VB-Audio Virtual Cable to create a virtual microphone
                for Discord, Zoom, and other applications.
              </p>
              <div class="setup-actions">
                <button class="btn-primary" (click)="install()">
                  Download & Install
                </button>
                <button class="btn-secondary" (click)="skip()">
                  Skip for now
                </button>
              </div>
            </div>
          }

          @case ('downloading') {
            <div class="setup-content">
              <div class="spinner"></div>
              <p>Downloading VB-Cable...</p>
            </div>
          }

          @case ('installing') {
            <div class="setup-content">
              <div class="spinner"></div>
              <p>Installing VB-Cable...</p>
              <p class="hint">Administrator permission may be required</p>
            </div>
          }

          @case ('done') {
            <div class="setup-content">
              <div class="success-icon">✅</div>
              <h3>Installation Complete</h3>
              <p>Please restart Voiceboard to use the virtual microphone.</p>
              <div class="setup-actions">
                <button class="btn-primary" (click)="restart()">
                  Restart Now
                </button>
              </div>
            </div>
          }

          @case ('error') {
            <div class="setup-content">
              <div class="error-icon">❌</div>
              <h3>Installation Failed</h3>
              <p class="error-message">{{ setupService.state().error }}</p>
              <div class="setup-actions">
                <button class="btn-primary" (click)="install()">
                  Retry
                </button>
                <button class="btn-secondary" (click)="openWebsite()">
                  Download Manually
                </button>
              </div>
            </div>
          }
        }
      </div>
    </div>
  `,
  styles: [`
    .setup-overlay {
      position: fixed;
      inset: 0;
      background: rgba(0, 0, 0, 0.8);
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: 10000;
    }

    .setup-modal {
      background: #1e1e1e;
      border-radius: 12px;
      padding: 32px;
      max-width: 480px;
      width: 90%;
      text-align: center;
      border: 1px solid #333;
    }

    h2 {
      margin: 0 0 24px;
      color: #fff;
      font-size: 24px;
    }

    h3 {
      margin: 16px 0 8px;
      color: #fff;
      font-size: 18px;
    }

    p {
      color: #aaa;
      margin: 8px 0;
      line-height: 1.5;
    }

    .hint {
      font-size: 12px;
      color: #666;
    }

    .setup-content {
      padding: 16px 0;
    }

    .warning-icon, .success-icon, .error-icon {
      font-size: 48px;
      margin-bottom: 16px;
    }

    .setup-actions {
      display: flex;
      gap: 12px;
      justify-content: center;
      margin-top: 24px;
    }

    button {
      padding: 12px 24px;
      border-radius: 6px;
      font-size: 14px;
      font-weight: 500;
      cursor: pointer;
      border: none;
      transition: background 0.2s;
    }

    .btn-primary {
      background: #007bff;
      color: white;
    }

    .btn-primary:hover {
      background: #0056b3;
    }

    .btn-secondary {
      background: #333;
      color: #aaa;
    }

    .btn-secondary:hover {
      background: #444;
    }

    .spinner {
      width: 40px;
      height: 40px;
      border: 3px solid #333;
      border-top-color: #007bff;
      border-radius: 50%;
      animation: spin 1s linear infinite;
      margin: 0 auto 16px;
    }

    @keyframes spin {
      to { transform: rotate(360deg); }
    }

    .error-message {
      color: #ff6b6b;
      font-size: 14px;
      background: rgba(255, 107, 107, 0.1);
      padding: 8px 12px;
      border-radius: 4px;
    }
  `]
})
export class SetupWizardComponent {
  setupService = inject(SetupWizardService);
  completed = output<boolean>();

  async install() {
    const success = await this.setupService.downloadAndInstall();
    if (success) {
      // Will show restart prompt
    }
  }

  skip() {
    this.setupService.skip();
    this.completed.emit(false);
  }

  restart() {
    // Tauri restart
    import('@tauri-apps/plugin-process').then(({ relaunch }) => relaunch());
  }

  openWebsite() {
    import('@tauri-apps/plugin-opener').then(({ openUrl }) => {
      openUrl('https://vb-audio.com/Cable/');
    });
  }
}
