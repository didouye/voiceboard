import { Component, OnInit, inject, signal } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { MixerComponent } from "./features/mixer/mixer.component";
import { ToastComponent } from "./core/components/toast/toast.component";
import { DebugConsoleComponent } from "./core/components/debug-console/debug-console.component";
import { SetupWizardComponent } from "./core/components/setup-wizard/setup-wizard.component";
import { ToastService } from "./core/services/toast.service";
import { DebugConsoleService } from "./core/services/debug-console.service";
import { SetupWizardService } from "./core/services/setup-wizard.service";
import { TauriService } from "./core/services/tauri.service";

interface UpdateInfo {
  available: boolean;
  version?: string;
  body?: string;
}

@Component({
  selector: "app-root",
  standalone: true,
  imports: [
    MixerComponent,
    ToastComponent,
    DebugConsoleComponent,
    SetupWizardComponent,
  ],
  template: `
    <!-- Grain overlay -->
    <div class="grain-overlay"></div>

    @if (showSetupWizard()) {
      <app-setup-wizard (completed)="onSetupCompleted($event)" />
    } @else {
      <app-mixer />
    }
    <app-toast />
    <app-debug-console />
  `,
  styles: [
    `
      :host {
        @apply block min-h-screen;
      }
    `,
  ],
})
export class AppComponent implements OnInit {
  private toastService = inject(ToastService);
  private debugConsole = inject(DebugConsoleService);
  private setupWizard = inject(SetupWizardService);
  private tauri = inject(TauriService);

  showSetupWizard = signal(false);

  async ngOnInit() {
    // Log startup info
    await this.logStartupInfo();

    // Cleanup orphaned images
    await this.cleanupOrphanedImages();

    // Check VB-Cable first (Windows only)
    const isWin = await this.isWindows();
    this.debugConsole.log("info", `Platform check: isWindows=${isWin}`);

    if (isWin) {
      this.debugConsole.log("info", "Checking VB-Cable installation...");
      const hasVbCable = await this.setupWizard.checkVbCable();
      this.debugConsole.log(
        "info",
        `VB-Cable check result: installed=${hasVbCable}`,
      );

      if (!hasVbCable && this.setupWizard.state().step !== "skipped") {
        this.debugConsole.log(
          "info",
          "Showing setup wizard (VB-Cable not found)",
        );
        this.showSetupWizard.set(true);
        return; // Don't continue initialization
      }
    } else {
      this.debugConsole.log("info", "Skipping VB-Cable check (not Windows)");
    }

    // Continue normal startup
    await this.checkForUpdate();
  }

  private async logStartupInfo() {
    try {
      const version = await getVersion();
      const { platform } = await import("@tauri-apps/plugin-os");
      const platformName = await platform();
      this.debugConsole.log(
        "info",
        `Voiceboard v${version} starting on ${platformName}`,
      );
    } catch (error) {
      this.debugConsole.log("warn", "Could not get app info");
    }
  }

  private async isWindows(): Promise<boolean> {
    try {
      const { platform } = await import("@tauri-apps/plugin-os");
      return (await platform()) === "windows";
    } catch {
      return false;
    }
  }

  onSetupCompleted(installed: boolean) {
    this.showSetupWizard.set(false);
    if (!installed) {
      this.debugConsole.log(
        "warn",
        "VB-Cable not installed - some features may be limited",
      );
    }
    this.checkForUpdate();
  }

  private async checkForUpdate() {
    this.debugConsole.log("info", "Checking for updates...");
    try {
      const update = await invoke<UpdateInfo>("check_for_update");

      if (update.available && update.version) {
        this.debugConsole.log("info", `Update available: v${update.version}`);
        this.toastService.show({
          message: `Update available: v${update.version}`,
          action: {
            label: "Update now",
            callback: () => this.installUpdate(),
          },
          duration: 10000,
        });
      } else {
        this.debugConsole.log("info", "No update available");
      }
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      this.debugConsole.log("error", `Update check failed: ${errorMessage}`);
      console.error("Update check failed:", errorMessage);
    }
  }

  private async installUpdate() {
    this.debugConsole.log("info", "Starting update installation...");
    try {
      await invoke("install_update");
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      this.debugConsole.log(
        "error",
        `Update installation failed: ${errorMessage}`,
      );
      console.error("Update installation failed:", errorMessage);
      this.toastService.show({
        message: `Update failed: ${errorMessage}`,
        duration: 10000,
      });
    }
  }

  private async cleanupOrphanedImages(): Promise<void> {
    try {
      const count = await this.tauri.cleanupOrphanedImages();
      if (count > 0) {
        console.log(`Cleaned up ${count} orphaned images`);
      }
    } catch (err) {
      console.error("Failed to cleanup orphaned images:", err);
    }
  }
}
