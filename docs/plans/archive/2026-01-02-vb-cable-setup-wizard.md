# VB-Cable Setup Wizard Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Detect missing VB-Cable on Windows and guide user through automatic download/installation.

**Architecture:** Backend Rust commands for detection/download/install, Angular modal component for UI, integration at app startup before other initialization.

**Tech Stack:** Rust (reqwest, zip), Angular 18+ (signals, standalone components), Tauri IPC

---

## Task 1: Add Rust Dependencies

**Files:**
- Modify: `src-tauri/Cargo.toml`

**Step 1: Add reqwest and zip dependencies**

In `src-tauri/Cargo.toml`, add to `[dependencies]`:

```toml
# VB-Cable installer download
reqwest = { version = "0.12", features = ["stream"] }
zip = "2.2"
```

**Step 2: Verify dependencies compile**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`

Expected: No errors

**Step 3: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "chore: add reqwest and zip dependencies for VB-Cable installer"
```

---

## Task 2: Add VB-Cable Status Command

**Files:**
- Modify: `src-tauri/src/application/commands.rs`

**Step 1: Add VbCableStatus struct and check command**

Add after the Update Commands section:

```rust
// ============================================================================
// VB-Cable Setup Commands
// ============================================================================

/// Status of VB-Cable installation
#[derive(Debug, Serialize)]
pub struct VbCableStatus {
    pub installed: bool,
    pub device_name: Option<String>,
}

/// Check if VB-Cable or any virtual audio device is installed
#[tauri::command]
pub async fn check_vb_cable_installed() -> Result<VbCableStatus, String> {
    let manager = CpalDeviceManager::new();

    match manager.find_virtual_outputs() {
        Ok(devices) => {
            if let Some(device) = devices.first() {
                Ok(VbCableStatus {
                    installed: true,
                    device_name: Some(device.name().to_string()),
                })
            } else {
                Ok(VbCableStatus {
                    installed: false,
                    device_name: None,
                })
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to check virtual devices");
            Ok(VbCableStatus {
                installed: false,
                device_name: None,
            })
        }
    }
}
```

**Step 2: Register command in lib.rs**

In `src-tauri/src/lib.rs`, add to imports:

```rust
check_vb_cable_installed,
```

Add to `invoke_handler`:

```rust
// VB-Cable setup
check_vb_cable_installed,
```

**Step 3: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`

Expected: No errors

**Step 4: Commit**

```bash
git add src-tauri/src/application/commands.rs src-tauri/src/lib.rs
git commit -m "feat: add check_vb_cable_installed command"
```

---

## Task 3: Add VB-Cable Download Command

**Files:**
- Modify: `src-tauri/src/application/commands.rs`

**Step 1: Add download and install command**

Add after `check_vb_cable_installed`:

```rust
/// Download and install VB-Cable
/// Returns Ok(()) on success, Err with message on failure
#[tauri::command]
pub async fn download_and_install_vb_cable(app: tauri::AppHandle) -> Result<(), String> {
    use std::io::{Read, Write};

    const VB_CABLE_URL: &str = "https://download.vb-audio.com/Download_CABLE/VBCABLE_Driver_Pack43.zip";

    tracing::info!("Starting VB-Cable download");

    // Get temp directory
    let temp_dir = std::env::temp_dir().join("voiceboard").join("vbcable");
    std::fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let zip_path = temp_dir.join("VBCABLE_Driver_Pack.zip");

    // Download ZIP file
    tracing::info!(url = VB_CABLE_URL, "Downloading VB-Cable");
    let response = reqwest::get(VB_CABLE_URL)
        .await
        .map_err(|e| format!("Download failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let bytes = response.bytes().await.map_err(|e| format!("Failed to read response: {}", e))?;

    // Write ZIP to disk
    std::fs::write(&zip_path, &bytes).map_err(|e| format!("Failed to save ZIP: {}", e))?;
    tracing::info!(path = ?zip_path, "ZIP downloaded");

    // Extract ZIP
    let zip_file = std::fs::File::open(&zip_path).map_err(|e| format!("Failed to open ZIP: {}", e))?;
    let mut archive = zip::ZipArchive::new(zip_file).map_err(|e| format!("Invalid ZIP: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("ZIP error: {}", e))?;
        let outpath = temp_dir.join(file.name());

        if file.is_dir() {
            std::fs::create_dir_all(&outpath).ok();
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| format!("Failed to create file: {}", e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to extract file: {}", e))?;
        }
    }
    tracing::info!("ZIP extracted");

    // Find and run installer
    let installer_path = temp_dir.join("VBCABLE_Setup_x64.exe");
    if !installer_path.exists() {
        // Try alternative name
        let alt_path = temp_dir.join("VBCABLE_Setup.exe");
        if alt_path.exists() {
            run_installer(&alt_path)?;
        } else {
            return Err("Installer not found in ZIP".to_string());
        }
    } else {
        run_installer(&installer_path)?;
    }

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);

    tracing::info!("VB-Cable installation completed");
    Ok(())
}

fn run_installer(path: &std::path::Path) -> Result<(), String> {
    tracing::info!(path = ?path, "Running VB-Cable installer");

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        use std::os::windows::process::CommandExt;

        // CREATE_NO_WINDOW flag
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let status = Command::new(path)
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map_err(|e| format!("Failed to run installer: {}", e))?;

        if !status.success() {
            return Err(format!("Installer exited with code: {:?}", status.code()));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        return Err("VB-Cable installation is only supported on Windows".to_string());
    }

    Ok(())
}
```

**Step 2: Register command in lib.rs**

Add to imports and invoke_handler:

```rust
download_and_install_vb_cable,
```

**Step 3: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`

Expected: No errors

**Step 4: Commit**

```bash
git add src-tauri/src/application/commands.rs src-tauri/src/lib.rs
git commit -m "feat: add download_and_install_vb_cable command"
```

---

## Task 4: Create Setup Wizard Service (Angular)

**Files:**
- Create: `src/app/core/services/setup-wizard.service.ts`

**Step 1: Create the service**

```typescript
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
```

**Step 2: Commit**

```bash
git add src/app/core/services/setup-wizard.service.ts
git commit -m "feat: add SetupWizardService for VB-Cable detection"
```

---

## Task 5: Create Setup Wizard Component (Angular)

**Files:**
- Create: `src/app/core/components/setup-wizard/setup-wizard.component.ts`

**Step 1: Create the component**

```typescript
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
    import('@tauri-apps/api/process').then(({ relaunch }) => relaunch());
  }

  openWebsite() {
    import('@tauri-apps/plugin-opener').then(({ openUrl }) => {
      openUrl('https://vb-audio.com/Cable/');
    });
  }
}
```

**Step 2: Commit**

```bash
git add src/app/core/components/setup-wizard/setup-wizard.component.ts
git commit -m "feat: add SetupWizardComponent UI"
```

---

## Task 6: Integrate Setup Wizard in App Component

**Files:**
- Modify: `src/app/app.component.ts`

**Step 1: Import and add setup wizard**

Update the component:

```typescript
import { Component, OnInit, inject, signal } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { MixerComponent } from './features/mixer/mixer.component';
import { ToastComponent } from './core/components/toast/toast.component';
import { DebugConsoleComponent } from './core/components/debug-console/debug-console.component';
import { SetupWizardComponent } from './core/components/setup-wizard/setup-wizard.component';
import { ToastService } from './core/services/toast.service';
import { DebugConsoleService } from './core/services/debug-console.service';
import { SetupWizardService } from './core/services/setup-wizard.service';

interface UpdateInfo {
  available: boolean;
  version?: string;
  body?: string;
}

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [MixerComponent, ToastComponent, DebugConsoleComponent, SetupWizardComponent],
  template: `
    @if (showSetupWizard()) {
      <app-setup-wizard (completed)="onSetupCompleted($event)" />
    } @else {
      <app-mixer />
    }
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
  private setupWizard = inject(SetupWizardService);

  showSetupWizard = signal(false);

  async ngOnInit() {
    // Check VB-Cable first (Windows only)
    if (await this.isWindows()) {
      const hasVbCable = await this.setupWizard.checkVbCable();
      if (!hasVbCable && this.setupWizard.state().step !== 'skipped') {
        this.showSetupWizard.set(true);
        return; // Don't continue initialization
      }
    }

    // Continue normal startup
    await this.checkForUpdate();
  }

  private async isWindows(): Promise<boolean> {
    try {
      const { platform } = await import('@tauri-apps/plugin-os');
      return (await platform()) === 'windows';
    } catch {
      return false;
    }
  }

  onSetupCompleted(installed: boolean) {
    this.showSetupWizard.set(false);
    if (!installed) {
      this.debugConsole.log('warn', 'VB-Cable not installed - some features may be limited');
    }
    this.checkForUpdate();
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
```

**Step 2: Install OS plugin dependency**

Run: `npm install @tauri-apps/plugin-os`

**Step 3: Verify build**

Run: `npm run build`

Expected: No errors

**Step 4: Commit**

```bash
git add src/app/app.component.ts package.json package-lock.json
git commit -m "feat: integrate setup wizard at app startup"
```

---

## Task 7: Add OS Plugin to Tauri

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/capabilities/default.json`

**Step 1: Add OS plugin dependency**

In `src-tauri/Cargo.toml`:

```toml
tauri-plugin-os = "2"
```

**Step 2: Register plugin in lib.rs**

Add after other plugins:

```rust
.plugin(tauri_plugin_os::init())
```

**Step 3: Add capability**

In `src-tauri/capabilities/default.json`, add to permissions:

```json
"os:default"
```

**Step 4: Verify build**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`

Expected: No errors

**Step 5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/lib.rs src-tauri/capabilities/default.json
git commit -m "feat: add tauri-plugin-os for platform detection"
```

---

## Task 8: Test End-to-End

**Step 1: Run the application**

Run: `npm run tauri dev`

**Step 2: Test scenarios**

1. If VB-Cable NOT installed:
   - Should see setup wizard modal
   - Click "Download & Install" should download and run installer
   - After installation, "Restart Now" should restart app

2. If VB-Cable IS installed:
   - Should skip wizard and go to main app

3. Click "Skip for now":
   - Should close wizard and continue to app

**Step 3: Commit final changes**

```bash
git add -A
git commit -m "feat: complete VB-Cable setup wizard implementation"
```

---

## Summary

| Task | Description | Commit |
|------|-------------|--------|
| 1 | Add Rust dependencies | `chore: add reqwest and zip dependencies` |
| 2 | Add check command | `feat: add check_vb_cable_installed command` |
| 3 | Add install command | `feat: add download_and_install_vb_cable command` |
| 4 | Create Angular service | `feat: add SetupWizardService` |
| 5 | Create Angular component | `feat: add SetupWizardComponent UI` |
| 6 | Integrate in app | `feat: integrate setup wizard at app startup` |
| 7 | Add OS plugin | `feat: add tauri-plugin-os` |
| 8 | Test E2E | `feat: complete VB-Cable setup wizard` |
