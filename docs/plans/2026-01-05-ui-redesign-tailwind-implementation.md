# UI Redesign - Tailwind Migration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Migrate the entire UI from inline CSS to Tailwind CSS with a new Gaming/Pro audio visual style.

**Architecture:** Complete rewrite of all Angular component styles using Tailwind utility classes. New layout with sidebar (folders) + main area (pads) + status bar. Settings moved to a modal popup. Folder system prepared for future expansion.

**Tech Stack:** Angular 20, Tailwind CSS 3.4, PostCSS, Autoprefixer

---

## Task 1: Install and Configure Tailwind CSS

**Files:**
- Create: `tailwind.config.js`
- Create: `postcss.config.js`
- Modify: `src/styles.css`
- Modify: `package.json`

**Step 1: Install Tailwind dependencies**

Run:
```bash
cd /Users/didouye/Workspace/voiceboard && npm install -D tailwindcss postcss autoprefixer
```

Expected: Packages installed successfully

**Step 2: Initialize Tailwind**

Run:
```bash
cd /Users/didouye/Workspace/voiceboard && npx tailwindcss init -p
```

Expected: Creates `tailwind.config.js` and `postcss.config.js`

**Step 3: Configure Tailwind for Angular**

Replace `tailwind.config.js` with:

```javascript
/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.{html,ts}",
  ],
  theme: {
    extend: {
      colors: {
        background: '#0a0a0f',
        surface: {
          DEFAULT: '#12121a',
          hover: '#1a1a25',
        },
        border: {
          DEFAULT: '#2a2a3a',
        },
        accent: {
          DEFAULT: '#9d4edd',
          glow: '#bf5af2',
          hot: '#ff00ff',
        },
        text: {
          primary: '#ffffff',
          secondary: '#888899',
          muted: '#555566',
        },
        status: {
          success: '#22c55e',
          warning: '#eab308',
          error: '#ef4444',
          info: '#00d4ff',
        },
      },
      animation: {
        'glow-pulse': 'glow-pulse 1s ease-in-out infinite',
        'bounce-click': 'bounce-click 200ms ease-out',
        'slide-in': 'slide-in 150ms ease-out',
        'fade-in': 'fade-in 150ms ease-out',
        'scale-in': 'scale-in 150ms ease-out',
      },
      keyframes: {
        'glow-pulse': {
          '0%, 100%': { boxShadow: '0 0 20px rgba(157, 78, 221, 0.5)' },
          '50%': { boxShadow: '0 0 40px rgba(157, 78, 221, 0.8)' },
        },
        'bounce-click': {
          '0%': { transform: 'scale(0.95)' },
          '100%': { transform: 'scale(1)' },
        },
        'slide-in': {
          '0%': { transform: 'translateX(-100%)', opacity: '0' },
          '100%': { transform: 'translateX(0)', opacity: '1' },
        },
        'fade-in': {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        'scale-in': {
          '0%': { transform: 'scale(0.95)', opacity: '0' },
          '100%': { transform: 'scale(1)', opacity: '1' },
        },
      },
    },
  },
  plugins: [],
}
```

**Step 4: Update global styles**

Replace `src/styles.css` with:

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

/* Base styles */
@layer base {
  * {
    box-sizing: border-box;
  }

  html, body {
    @apply m-0 p-0 font-sans bg-background text-text-primary overflow-x-hidden;
  }

  /* Scrollbar styling */
  ::-webkit-scrollbar {
    @apply w-2 h-2;
  }

  ::-webkit-scrollbar-track {
    @apply bg-white/5;
  }

  ::-webkit-scrollbar-thumb {
    @apply bg-white/20 rounded;
  }

  ::-webkit-scrollbar-thumb:hover {
    @apply bg-white/30;
  }

  button {
    @apply font-sans;
  }

  a {
    @apply text-status-info no-underline hover:underline;
  }
}

/* Grain texture overlay */
@layer components {
  .grain-overlay {
    @apply fixed inset-0 pointer-events-none opacity-[0.03] mix-blend-overlay;
    background-image: url('/assets/noise.svg');
  }
}

/* Custom utilities */
@layer utilities {
  .glow-accent {
    box-shadow: 0 0 20px rgba(157, 78, 221, 0.5),
                0 0 40px rgba(157, 78, 221, 0.3);
  }

  .glow-accent-hot {
    box-shadow: 0 0 20px rgba(255, 0, 255, 0.5),
                0 0 40px rgba(255, 0, 255, 0.3);
  }

  .glow-subtle {
    box-shadow: 0 0 15px rgba(157, 78, 221, 0.3);
  }
}
```

**Step 5: Verify Tailwind is working**

Run:
```bash
cd /Users/didouye/Workspace/voiceboard && npm run build
```

Expected: Build succeeds without errors

**Step 6: Commit**

```bash
git add tailwind.config.js postcss.config.js src/styles.css package.json package-lock.json
git commit -m "build: add Tailwind CSS configuration

- Install tailwindcss, postcss, autoprefixer
- Configure custom color palette (Gaming/Pro audio theme)
- Add custom animations (glow-pulse, bounce-click, slide-in)
- Set up global styles with Tailwind directives
- Add grain overlay and glow utilities"
```

---

## Task 2: Create Noise Texture Asset

**Files:**
- Create: `src/assets/noise.svg`

**Step 1: Create the noise SVG**

Create `src/assets/noise.svg`:

```svg
<svg viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg">
  <filter id="noise">
    <feTurbulence type="fractalNoise" baseFrequency="0.65" numOctaves="3" stitchTiles="stitch"/>
  </filter>
  <rect width="100%" height="100%" filter="url(#noise)"/>
</svg>
```

**Step 2: Commit**

```bash
git add src/assets/noise.svg
git commit -m "feat(ui): add noise texture for grain overlay"
```

---

## Task 3: Create Folder Model and Service

**Files:**
- Modify: `src/app/core/models/audio-device.model.ts`
- Modify: `src/app/core/services/soundboard.service.ts`

**Step 1: Add Folder interface to models**

In `src/app/core/models/audio-device.model.ts`, add at the end:

```typescript
/**
 * Folder for organizing sounds
 */
export interface Folder {
  id: string;
  name: string;
  createdAt: number; // timestamp
}
```

**Step 2: Update SoundboardService with folder support**

In `src/app/core/services/soundboard.service.ts`:

Add after the imports:
```typescript
import { Folder } from '../models';
```

Add new signals after `_previewDeviceId`:
```typescript
// Folder state
private _folders = signal<Folder[]>([{ id: 'default', name: 'Default', createdAt: Date.now() }]);
private _activeFolderId = signal<string>('default');

// Public readonly signals for folders
readonly folders = this._folders.asReadonly();
readonly activeFolderId = this._activeFolderId.asReadonly();
readonly activeFolder = computed(() =>
  this._folders().find(f => f.id === this._activeFolderId()) || this._folders()[0]
);
```

Add method to switch folders:
```typescript
/**
 * Switch to a different folder
 */
setActiveFolder(folderId: string): void {
  if (this._folders().some(f => f.id === folderId)) {
    this._activeFolderId.set(folderId);
  }
}
```

**Step 3: Run tests**

Run:
```bash
cd /Users/didouye/Workspace/voiceboard && npm run build
```

Expected: Build passes

**Step 4: Commit**

```bash
git add src/app/core/models/audio-device.model.ts src/app/core/services/soundboard.service.ts
git commit -m "feat(folders): add folder model and basic folder support

- Add Folder interface to models
- Add folder signals to SoundboardService
- Create default folder on init
- Add setActiveFolder method"
```

---

## Task 4: Create VU Meter Component

**Files:**
- Create: `src/app/shared/components/vu-meter/vu-meter.component.ts`

**Step 1: Create shared components directory**

Run:
```bash
mkdir -p /Users/didouye/Workspace/voiceboard/src/app/shared/components/vu-meter
```

**Step 2: Create VU Meter component**

Create `src/app/shared/components/vu-meter/vu-meter.component.ts`:

```typescript
import { Component, Input } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-vu-meter',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="h-1.5 bg-surface rounded-full overflow-hidden">
      <div
        class="h-full rounded-full transition-[width] duration-50 ease-out"
        [style.width.%]="level * 100"
        [style.background]="gradient"
      ></div>
    </div>
  `,
  styles: []
})
export class VuMeterComponent {
  @Input() level = 0; // 0-1

  readonly gradient = 'linear-gradient(to right, #22c55e, #eab308, #ef4444)';
}
```

**Step 3: Verify build**

Run:
```bash
cd /Users/didouye/Workspace/voiceboard && npm run build
```

Expected: Build passes

**Step 4: Commit**

```bash
git add src/app/shared/components/vu-meter/vu-meter.component.ts
git commit -m "feat(ui): add VuMeter component with Tailwind"
```

---

## Task 5: Create Settings Popup Component

**Files:**
- Create: `src/app/shared/components/settings-popup/settings-popup.component.ts`

**Step 1: Create directory**

Run:
```bash
mkdir -p /Users/didouye/Workspace/voiceboard/src/app/shared/components/settings-popup
```

**Step 2: Create Settings Popup component**

Create `src/app/shared/components/settings-popup/settings-popup.component.ts`:

```typescript
import { Component, Input, Output, EventEmitter, OnInit, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { TauriService } from '../../../core/services/tauri.service';
import { MixerService } from '../../../core/services/mixer.service';
import { AudioDevice, AppSettings } from '../../../core/models';

@Component({
  selector: 'app-settings-popup',
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
    <div class="fixed inset-0 z-50 flex items-center justify-center" (click)="close.emit()">
      <!-- Backdrop -->
      <div class="absolute inset-0 bg-black/60 backdrop-blur-sm"></div>

      <!-- Modal -->
      <div
        class="relative bg-surface border border-border rounded-xl p-6 w-full max-w-md animate-scale-in"
        (click)="$event.stopPropagation()"
      >
        <!-- Header -->
        <div class="flex items-center justify-between mb-6">
          <h2 class="text-lg font-semibold text-text-primary flex items-center gap-2">
            <span>&#9881;</span> Settings
          </h2>
          <button
            class="w-8 h-8 flex items-center justify-center rounded-lg hover:bg-surface-hover text-text-secondary hover:text-text-primary transition-colors"
            (click)="close.emit()"
          >
            &#10005;
          </button>
        </div>

        @if (loading()) {
          <div class="text-center py-8 text-text-secondary">Loading...</div>
        } @else {
          <!-- Audio Devices Section -->
          <div class="mb-6">
            <h3 class="text-xs font-semibold text-text-muted uppercase tracking-wider mb-4">Audio Devices</h3>

            <!-- Input Device -->
            <div class="mb-4">
              <label class="flex items-center gap-2 text-sm text-text-secondary mb-2">
                <span>&#127908;</span> Input
              </label>
              <select
                class="w-full px-4 py-3 bg-background border border-border rounded-lg text-text-primary focus:outline-none focus:border-accent transition-colors"
                [value]="selectedInputId()"
                (change)="onInputChange($event)"
              >
                <option value="">-- Select Microphone --</option>
                @for (device of inputDevices(); track device.id) {
                  <option [value]="device.id">
                    {{ device.name }}{{ device.isDefault ? ' (Default)' : '' }}
                  </option>
                }
              </select>
            </div>

            <!-- Output Device -->
            <div class="mb-4">
              <label class="flex items-center gap-2 text-sm text-text-secondary mb-2">
                <span>&#128266;</span> Output (Virtual Mic)
              </label>
              <select
                class="w-full px-4 py-3 bg-background border border-border rounded-lg text-text-primary focus:outline-none focus:border-accent transition-colors"
                [value]="selectedOutputId()"
                (change)="onOutputChange($event)"
              >
                @for (device of virtualOutputDevices(); track device.id) {
                  <option [value]="device.id">{{ device.name }}</option>
                }
              </select>
            </div>

            <!-- Preview Device -->
            <div class="mb-4">
              <label class="flex items-center gap-2 text-sm text-text-secondary mb-2">
                <span>&#127911;</span> Preview
              </label>
              <select
                class="w-full px-4 py-3 bg-background border border-border rounded-lg text-text-primary focus:outline-none focus:border-accent transition-colors"
                [value]="selectedPreviewId()"
                (change)="onPreviewChange($event)"
              >
                <option value="">-- System Default --</option>
                @for (device of physicalOutputDevices(); track device.id) {
                  <option [value]="device.id">
                    {{ device.name }}{{ device.isDefault ? ' (Default)' : '' }}
                  </option>
                }
              </select>
            </div>
          </div>

          <!-- Mixer Section -->
          <div class="mb-6">
            <h3 class="text-xs font-semibold text-text-muted uppercase tracking-wider mb-4">Mixer</h3>

            <!-- Mic Volume -->
            <div class="mb-4">
              <div class="flex items-center justify-between mb-2">
                <label class="text-sm text-text-secondary">Mic Volume</label>
                <div class="flex items-center gap-2">
                  <span class="text-sm text-text-primary font-mono">{{ Math.round(micVolume() * 100) }}%</span>
                  <button
                    class="w-8 h-8 flex items-center justify-center rounded-lg transition-colors"
                    [class]="micMuted() ? 'bg-status-error text-white' : 'bg-surface-hover text-text-secondary hover:text-text-primary'"
                    (click)="toggleMicMute()"
                  >
                    {{ micMuted() ? '&#128263;' : '&#128264;' }}
                  </button>
                </div>
              </div>
              <input
                type="range"
                min="0"
                max="100"
                [value]="micVolume() * 100"
                (input)="onMicVolumeChange($event)"
                class="w-full h-2 bg-surface rounded-full appearance-none cursor-pointer
                       [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:h-4
                       [&::-webkit-slider-thumb]:bg-white [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:cursor-pointer
                       [&::-webkit-slider-thumb]:transition-transform [&::-webkit-slider-thumb]:hover:scale-110"
              >
            </div>

            <!-- Master Volume -->
            <div class="mb-4">
              <div class="flex items-center justify-between mb-2">
                <label class="text-sm text-text-secondary">Master Volume</label>
                <span class="text-sm text-text-primary font-mono">{{ Math.round(masterVolume() * 100) }}%</span>
              </div>
              <input
                type="range"
                min="0"
                max="100"
                [value]="masterVolume() * 100"
                (input)="onMasterVolumeChange($event)"
                class="w-full h-2 bg-surface rounded-full appearance-none cursor-pointer
                       [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:h-4
                       [&::-webkit-slider-thumb]:bg-white [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:cursor-pointer
                       [&::-webkit-slider-thumb]:transition-transform [&::-webkit-slider-thumb]:hover:scale-110"
              >
            </div>

            <!-- Mic Monitoring -->
            <div class="flex items-center justify-between py-3 px-4 bg-background rounded-lg">
              <span class="text-sm text-text-secondary">Mic Monitoring</span>
              <button
                class="w-12 h-6 rounded-full transition-colors relative"
                [class]="micMonitoring() ? 'bg-accent' : 'bg-surface'"
                (click)="toggleMicMonitoring()"
              >
                <div
                  class="absolute top-1 w-4 h-4 bg-white rounded-full transition-transform"
                  [class]="micMonitoring() ? 'left-7' : 'left-1'"
                ></div>
              </button>
            </div>
          </div>

          <!-- Start/Stop Button -->
          <button
            class="w-full py-4 rounded-lg font-semibold text-white transition-all"
            [class]="isRunning()
              ? 'bg-accent-hot hover:bg-accent-hot/80 animate-glow-pulse'
              : 'bg-accent hover:bg-accent-glow'"
            (click)="toggleMixing()"
          >
            {{ isRunning() ? '&#9632; STOP MIXING' : '&#9654; START MIXING' }}
          </button>
        }
      </div>
    </div>
  `,
  styles: []
})
export class SettingsPopupComponent implements OnInit {
  @Output() close = new EventEmitter<void>();

  // State
  private _inputDevices = signal<AudioDevice[]>([]);
  private _virtualOutputDevices = signal<AudioDevice[]>([]);
  private _physicalOutputDevices = signal<AudioDevice[]>([]);
  private _settings = signal<AppSettings | null>(null);
  private _loading = signal(true);

  // Public signals
  readonly inputDevices = this._inputDevices.asReadonly();
  readonly virtualOutputDevices = this._virtualOutputDevices.asReadonly();
  readonly physicalOutputDevices = this._physicalOutputDevices.asReadonly();
  readonly loading = this._loading.asReadonly();

  // Computed from settings
  readonly selectedInputId = computed(() => this._settings()?.audio.inputDeviceId ?? '');
  readonly selectedOutputId = computed(() => this._settings()?.audio.outputDeviceId ?? '');
  readonly selectedPreviewId = computed(() => this._settings()?.audio.previewDeviceId ?? '');
  readonly micMonitoring = computed(() => this._settings()?.audio.micMonitoring ?? false);

  // Mixer state from service
  readonly masterVolume = computed(() => this.mixer.masterVolume());
  readonly micVolume = computed(() => this.mixer.micVolume());
  readonly micMuted = computed(() => this.mixer.micMuted());
  readonly isRunning = computed(() => this.mixer.isRunning());

  Math = Math;

  constructor(
    private tauri: TauriService,
    private mixer: MixerService
  ) {}

  async ngOnInit(): Promise<void> {
    await this.loadData();
  }

  private async loadData(): Promise<void> {
    this._loading.set(true);
    try {
      const [inputDevices, physicalOutputs, virtualOutputs, settings] = await Promise.all([
        this.tauri.getInputDevices(),
        this.tauri.getPhysicalOutputDevices(),
        this.tauri.getVirtualOutputsByPriority(),
        this.tauri.loadSettings()
      ]);

      this._inputDevices.set(inputDevices);
      this._physicalOutputDevices.set(physicalOutputs);
      this._virtualOutputDevices.set(virtualOutputs);
      this._settings.set(settings);
    } catch (err) {
      console.error('Failed to load settings data:', err);
    } finally {
      this._loading.set(false);
    }
  }

  async onInputChange(event: Event): Promise<void> {
    const select = event.target as HTMLSelectElement;
    const deviceId = select.value || null;
    await this.tauri.setInputDevice(deviceId);
    await this.updateSettingsLocal('inputDeviceId', deviceId);
    await this.mixer.restartIfRunning();
  }

  async onOutputChange(event: Event): Promise<void> {
    const select = event.target as HTMLSelectElement;
    const deviceId = select.value || null;
    await this.tauri.setOutputDevice(deviceId);
    await this.updateSettingsLocal('outputDeviceId', deviceId);
    await this.mixer.restartIfRunning();
  }

  async onPreviewChange(event: Event): Promise<void> {
    const select = event.target as HTMLSelectElement;
    const deviceId = select.value || null;
    await this.tauri.setPreviewDevice(deviceId);
    await this.updateSettingsLocal('previewDeviceId', deviceId);
  }

  async onMicVolumeChange(event: Event): Promise<void> {
    const input = event.target as HTMLInputElement;
    const volume = parseFloat(input.value) / 100;
    await this.mixer.setMicVolume(volume);
  }

  async onMasterVolumeChange(event: Event): Promise<void> {
    const input = event.target as HTMLInputElement;
    const volume = parseFloat(input.value) / 100;
    await this.mixer.setMasterVolume(volume);
  }

  async toggleMicMute(): Promise<void> {
    await this.mixer.toggleMicMute();
  }

  async toggleMicMonitoring(): Promise<void> {
    const newValue = !this.micMonitoring();
    await this.tauri.setMicMonitoring(newValue);
    await this.updateSettingsLocal('micMonitoring', newValue);
  }

  async toggleMixing(): Promise<void> {
    if (this.isRunning()) {
      await this.mixer.stop();
    } else {
      await this.mixer.start();
    }
  }

  private async updateSettingsLocal(key: string, value: unknown): Promise<void> {
    const settings = this._settings();
    if (settings) {
      this._settings.set({
        ...settings,
        audio: { ...settings.audio, [key]: value }
      });
    }
  }
}
```

**Step 3: Verify build**

Run:
```bash
cd /Users/didouye/Workspace/voiceboard && npm run build
```

Expected: Build passes

**Step 4: Commit**

```bash
git add src/app/shared/components/settings-popup/settings-popup.component.ts
git commit -m "feat(ui): add SettingsPopup component with Tailwind

- Modal with backdrop blur
- Device selection (input, output, preview)
- Mixer controls (volumes, mute, monitoring)
- Start/Stop mixing button with glow effect"
```

---

## Task 6: Rewrite App Component with Grain Overlay

**Files:**
- Modify: `src/app/app.component.ts`

**Step 1: Update app component**

Replace `src/app/app.component.ts` with:

```typescript
import { Component, OnInit, inject, signal } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';
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
  styles: [`
    :host {
      @apply block min-h-screen;
    }
  `]
})
export class AppComponent implements OnInit {
  private toastService = inject(ToastService);
  private debugConsole = inject(DebugConsoleService);
  private setupWizard = inject(SetupWizardService);

  showSetupWizard = signal(false);

  async ngOnInit() {
    await this.logStartupInfo();

    const isWin = await this.isWindows();
    this.debugConsole.log('info', `Platform check: isWindows=${isWin}`);

    if (isWin) {
      this.debugConsole.log('info', 'Checking VB-Cable installation...');
      const hasVbCable = await this.setupWizard.checkVbCable();
      this.debugConsole.log('info', `VB-Cable check result: installed=${hasVbCable}`);

      if (!hasVbCable && this.setupWizard.state().step !== 'skipped') {
        this.debugConsole.log('info', 'Showing setup wizard (VB-Cable not found)');
        this.showSetupWizard.set(true);
        return;
      }
    } else {
      this.debugConsole.log('info', 'Skipping VB-Cable check (not Windows)');
    }

    await this.checkForUpdate();
  }

  private async logStartupInfo() {
    try {
      const version = await getVersion();
      const { platform } = await import('@tauri-apps/plugin-os');
      const platformName = await platform();
      this.debugConsole.log('info', `Voiceboard v${version} starting on ${platformName}`);
    } catch (error) {
      this.debugConsole.log('warn', 'Could not get app info');
    }
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

**Step 2: Verify build**

Run:
```bash
cd /Users/didouye/Workspace/voiceboard && npm run build
```

Expected: Build passes

**Step 3: Commit**

```bash
git add src/app/app.component.ts
git commit -m "refactor(ui): update AppComponent with grain overlay and Tailwind"
```

---

## Task 7: Rewrite Mixer Component with New Layout

**Files:**
- Modify: `src/app/features/mixer/mixer.component.ts`

**Step 1: Rewrite mixer component with sidebar layout**

Replace `src/app/features/mixer/mixer.component.ts` with:

```typescript
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
```

**Step 2: Verify build (will fail - need StatusBarComponent)**

This step will fail because StatusBarComponent doesn't exist yet. Continue to next task.

---

## Task 8: Create Status Bar Component

**Files:**
- Create: `src/app/features/mixer/status-bar/status-bar.component.ts`

**Step 1: Create directory**

Run:
```bash
mkdir -p /Users/didouye/Workspace/voiceboard/src/app/features/mixer/status-bar
```

**Step 2: Create Status Bar component**

Create `src/app/features/mixer/status-bar/status-bar.component.ts`:

```typescript
import { Component, OnInit, OnDestroy, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { TauriService } from '../../../core/services/tauri.service';
import { MixerService } from '../../../core/services/mixer.service';
import { VuMeterComponent } from '../../../shared/components/vu-meter/vu-meter.component';
import { AudioDevice, AppSettings } from '../../../core/models';

interface AudioLevels {
  inputRms: number;
  inputPeak: number;
  outputRms: number;
  outputPeak: number;
}

@Component({
  selector: 'app-status-bar',
  standalone: true,
  imports: [CommonModule, VuMeterComponent],
  template: `
    <div class="h-14 bg-background border-t border-border flex">
      <!-- Input -->
      <div class="flex-1 px-4 py-2 flex flex-col justify-center border-r border-border">
        <div class="flex items-center gap-2 mb-1">
          <span class="text-sm">&#127908;</span>
          <span class="text-xs text-text-secondary truncate flex-1">{{ inputDeviceName() }}</span>
          <span class="flex items-center gap-1">
            <span
              class="w-2 h-2 rounded-full"
              [class]="mixer.isRunning() ? 'bg-status-success animate-pulse' : 'bg-text-muted'"
            ></span>
            <span class="text-[10px] text-text-muted">{{ mixer.isRunning() ? 'Recording' : 'Ready' }}</span>
          </span>
        </div>
        <app-vu-meter [level]="inputLevel()" />
      </div>

      <!-- Output -->
      <div class="flex-1 px-4 py-2 flex flex-col justify-center border-r border-border">
        <div class="flex items-center gap-2 mb-1">
          <span class="text-sm">&#128266;</span>
          <span class="text-xs text-text-secondary truncate flex-1">{{ outputDeviceName() }}</span>
          <span class="flex items-center gap-1">
            <span
              class="w-2 h-2 rounded-full"
              [class]="mixer.isRunning() ? 'bg-accent animate-pulse' : 'bg-text-muted'"
            ></span>
            <span class="text-[10px] text-text-muted">{{ mixer.isRunning() ? 'Streaming' : 'Ready' }}</span>
          </span>
        </div>
        <app-vu-meter [level]="outputLevel()" />
      </div>

      <!-- Preview -->
      <div class="flex-1 px-4 py-2 flex flex-col justify-center">
        <div class="flex items-center gap-2 mb-1">
          <span class="text-sm">&#127911;</span>
          <span class="text-xs text-text-secondary truncate flex-1">{{ previewDeviceName() }}</span>
          <span class="flex items-center gap-1">
            <span class="w-2 h-2 rounded-full bg-text-muted"></span>
            <span class="text-[10px] text-text-muted">Ready</span>
          </span>
        </div>
        <app-vu-meter [level]="0" />
      </div>
    </div>
  `,
  styles: []
})
export class StatusBarComponent implements OnInit, OnDestroy {
  // Audio levels
  inputLevel = signal(0);
  outputLevel = signal(0);

  // Device info
  private _settings = signal<AppSettings | null>(null);
  private _inputDevices = signal<AudioDevice[]>([]);
  private _outputDevices = signal<AudioDevice[]>([]);

  // Computed device names
  readonly inputDeviceName = computed(() => {
    const settings = this._settings();
    const devices = this._inputDevices();
    if (!settings?.audio.inputDeviceId) return 'Not selected';
    return devices.find(d => d.id === settings.audio.inputDeviceId)?.name || 'Unknown';
  });

  readonly outputDeviceName = computed(() => {
    const settings = this._settings();
    const devices = this._outputDevices();
    if (!settings?.audio.outputDeviceId) return 'Not selected';
    return devices.find(d => d.id === settings.audio.outputDeviceId)?.name || 'Unknown';
  });

  readonly previewDeviceName = computed(() => {
    const settings = this._settings();
    if (!settings?.audio.previewDeviceId) return 'System Default';
    return 'Custom';
  });

  private unlisten?: UnlistenFn;

  constructor(
    private tauri: TauriService,
    public mixer: MixerService
  ) {}

  async ngOnInit(): Promise<void> {
    // Load device info
    const [settings, inputDevices, outputDevices] = await Promise.all([
      this.tauri.loadSettings(),
      this.tauri.getInputDevices(),
      this.tauri.getVirtualOutputsByPriority()
    ]);

    this._settings.set(settings);
    this._inputDevices.set(inputDevices);
    this._outputDevices.set(outputDevices);

    // Listen for audio levels
    this.unlisten = await listen<AudioLevels>('audio-levels', (event) => {
      this.inputLevel.set(Math.min(event.payload.inputRms * 3, 1));
      this.outputLevel.set(Math.min(event.payload.outputRms * 3, 1));
    });
  }

  ngOnDestroy(): void {
    this.unlisten?.();
  }
}
```

**Step 3: Verify build**

Run:
```bash
cd /Users/didouye/Workspace/voiceboard && npm run build
```

Expected: Build passes

**Step 4: Commit**

```bash
git add src/app/features/mixer/mixer.component.ts src/app/features/mixer/status-bar/status-bar.component.ts
git commit -m "refactor(ui): rewrite Mixer with new sidebar layout and StatusBar

- New layout: Sidebar (folders) + Main (soundboard) + Status bar
- Sidebar shows folder list with active folder highlight
- Settings button opens modal popup
- Status bar shows 3 devices with VU meters"
```

---

## Task 9: Rewrite Soundboard Component

**Files:**
- Modify: `src/app/features/soundboard/soundboard.component.ts`

**Step 1: Rewrite soundboard component**

Replace `src/app/features/soundboard/soundboard.component.ts` with:

```typescript
import { Component, HostListener, signal, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { SoundboardService } from '../../core/services/soundboard.service';
import { SoundPadComponent } from './sound-pad/sound-pad.component';
import { listen, TauriEvent } from '@tauri-apps/api/event';

const DEFAULT_HOTKEYS = ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '='];

@Component({
  selector: 'app-soundboard',
  standalone: true,
  imports: [CommonModule, SoundPadComponent],
  template: `
    <div class="h-full flex flex-col">
      <!-- Header -->
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-lg font-semibold text-text-primary">
          {{ soundboard.activeFolder()?.name || 'Soundboard' }}
        </h2>
        <div class="flex items-center gap-2">
          @if (soundboard.playingCount() > 0) {
            <button
              class="px-4 py-2 bg-status-error hover:bg-status-error/80 text-white rounded-lg text-sm font-medium transition-colors"
              (click)="soundboard.stopAll()"
            >
              &#9632; Stop All ({{ soundboard.playingCount() }})
            </button>
          }
        </div>
      </div>

      <!-- Error message -->
      @if (soundboard.error()) {
        <div class="mb-4 px-4 py-3 bg-status-error/20 border border-status-error/50 rounded-lg flex items-center justify-between">
          <span class="text-status-error text-sm">{{ soundboard.error() }}</span>
          <button
            class="px-3 py-1 text-xs text-status-error border border-status-error/50 rounded hover:bg-status-error/20 transition-colors"
            (click)="soundboard.clearError()"
          >
            Dismiss
          </button>
        </div>
      }

      <!-- Pads grid -->
      <div class="flex-1 relative">
        <div class="grid grid-cols-4 gap-3">
          @for (pad of soundboard.pads(); track pad.id; let i = $index) {
            <app-sound-pad
              [pad]="pad"
              [hotkey]="getHotkey(i)"
              [loading]="soundboard.loading()"
              [isPreviewing]="soundboard.previewingPadId() === pad.id"
              (play)="soundboard.playSound(pad.id)"
              (preview)="soundboard.previewSound(pad.id)"
              (import)="soundboard.importSound(pad.id)"
              (remove)="soundboard.removeSound(pad.id)"
              (volumeChange)="soundboard.setPadVolume(pad.id, $event)"
              (speedChange)="soundboard.setPadSpeed(pad.id, $event)"
            />
          }
        </div>

        <!-- Drop overlay -->
        @if (isDragging()) {
          <div class="absolute inset-0 bg-accent/80 border-2 border-dashed border-white rounded-xl flex items-center justify-center z-10">
            <span class="text-white text-lg font-medium">
              Drop to import {{ dragFileCount() }} file{{ dragFileCount() > 1 ? 's' : '' }}
            </span>
          </div>
        }
      </div>

      <!-- Footer -->
      <div class="mt-4 flex justify-center">
        <button
          class="px-6 py-3 bg-surface-hover border border-dashed border-border hover:border-accent text-text-secondary hover:text-text-primary rounded-lg text-sm transition-all flex items-center gap-2"
          [class.opacity-50]="soundboard.loading()"
          [class.cursor-not-allowed]="soundboard.loading()"
          [disabled]="soundboard.loading()"
          (click)="importMultiple()"
        >
          <span>&#128193;</span>
          Import Multiple
        </button>
      </div>
    </div>
  `,
  styles: []
})
export class SoundboardComponent implements OnInit, OnDestroy {
  constructor(public soundboard: SoundboardService) {}

  isDragging = signal(false);
  dragFileCount = signal(0);

  private readonly AUDIO_EXTENSIONS = ['mp3', 'ogg', 'wav', 'flac'];
  private unlistenDragEnter?: () => void;
  private unlistenDragOver?: () => void;
  private unlistenDragLeave?: () => void;
  private unlistenDragDrop?: () => void;

  async ngOnInit(): Promise<void> {
    await this.initDragDropListeners();
  }

  ngOnDestroy(): void {
    this.unlistenDragEnter?.();
    this.unlistenDragOver?.();
    this.unlistenDragLeave?.();
    this.unlistenDragDrop?.();
  }

  private async initDragDropListeners(): Promise<void> {
    this.unlistenDragEnter = await listen<{ paths: string[]; position: { x: number; y: number } }>(
      TauriEvent.DRAG_ENTER,
      (event) => {
        const audioPaths = event.payload.paths.filter(path => {
          const ext = path.split('.').pop()?.toLowerCase();
          return ext && this.AUDIO_EXTENSIONS.includes(ext);
        });
        if (audioPaths.length > 0) {
          this.dragFileCount.set(audioPaths.length);
          this.isDragging.set(true);
        }
      }
    );

    this.unlistenDragOver = await listen(TauriEvent.DRAG_OVER, () => {});

    this.unlistenDragLeave = await listen(TauriEvent.DRAG_LEAVE, () => {
      this.isDragging.set(false);
      this.dragFileCount.set(0);
    });

    this.unlistenDragDrop = await listen<{ paths: string[]; position: { x: number; y: number } }>(
      TauriEvent.DRAG_DROP,
      async (event) => {
        this.isDragging.set(false);
        this.dragFileCount.set(0);

        const audioPaths = event.payload.paths.filter(path => {
          const ext = path.split('.').pop()?.toLowerCase();
          return ext && this.AUDIO_EXTENSIONS.includes(ext);
        });

        if (audioPaths.length === 0) return;

        const result = await this.soundboard.importSoundsFromPaths(audioPaths);

        if (result.errors.length > 0) {
          console.warn(`Imported ${result.imported} files. Failed: ${result.errors.join(', ')}`);
        }
      }
    );
  }

  @HostListener('window:keydown', ['$event'])
  handleKeydown(event: KeyboardEvent): void {
    if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
      return;
    }

    if (event.key === 'Escape') {
      this.soundboard.stopAll();
      return;
    }

    const pads = this.soundboard.pads();
    const padIndex = pads.findIndex(p => {
      const hotkey = p.hotkey || DEFAULT_HOTKEYS[pads.indexOf(p)];
      return hotkey === event.key;
    });

    if (padIndex >= 0) {
      const pad = pads[padIndex];
      if (pad.sound) {
        event.preventDefault();
        this.soundboard.playSound(pad.id);
      }
    }
  }

  getHotkey(padIndex: number): string | undefined {
    const pads = this.soundboard.pads();
    if (padIndex < pads.length) {
      return pads[padIndex].hotkey || DEFAULT_HOTKEYS[padIndex];
    }
    return undefined;
  }

  async importMultiple(): Promise<void> {
    const result = await this.soundboard.importMultipleSounds();

    if (result.errors.length > 0) {
      console.warn(`Imported ${result.imported} files.\nFailed (${result.errors.length}):\n${result.errors.join('\n')}`);
    }
  }
}
```

**Step 2: Verify build**

Run:
```bash
cd /Users/didouye/Workspace/voiceboard && npm run build
```

Expected: Build passes

**Step 3: Commit**

```bash
git add src/app/features/soundboard/soundboard.component.ts
git commit -m "refactor(ui): rewrite Soundboard component with Tailwind

- 4-column grid layout
- Gaming/Pro audio styling
- Tailwind utility classes
- Drop overlay with accent color"
```

---

## Task 10: Rewrite Sound Pad Component

**Files:**
- Modify: `src/app/features/soundboard/sound-pad/sound-pad.component.ts`

**Step 1: Rewrite sound pad component**

Replace `src/app/features/soundboard/sound-pad/sound-pad.component.ts` with:

```typescript
import { Component, Input, Output, EventEmitter, HostListener, HostBinding } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { SoundPad } from '../../../core/models';
import { SoundboardService } from '../../../core/services/soundboard.service';

@Component({
  selector: 'app-sound-pad',
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
    <div
      class="aspect-square rounded-xl cursor-pointer relative overflow-visible transition-all duration-150 flex items-center justify-center group"
      [class]="padClasses"
      [style.--pad-color]="pad.color"
      (click)="onClick($event)"
      (contextmenu)="onRightClick($event)"
    >
      <!-- Hotkey badge -->
      @if (hotkey) {
        <span class="absolute top-2 left-2 px-1.5 py-0.5 bg-black/50 text-white/80 text-[10px] font-semibold rounded font-mono uppercase">
          {{ hotkey }}
        </span>
      }

      @if (pad.sound) {
        <!-- Sound content -->
        <div class="text-center px-2 w-full">
          <span class="block text-xs font-semibold text-white truncate mb-1 drop-shadow-md">
            {{ pad.sound.name }}
          </span>
          <span class="block text-[10px] text-white/70">
            {{ formatDuration(pad.sound.duration) }}
          </span>
        </div>

        <!-- Playing indicator -->
        @if (pad.isPlaying) {
          <div class="absolute bottom-2 left-1/2 -translate-x-1/2 flex gap-0.5 items-end h-4">
            <span class="w-1 bg-white rounded-sm animate-[soundbar_0.4s_ease-in-out_infinite_alternate]" style="height: 8px"></span>
            <span class="w-1 bg-white rounded-sm animate-[soundbar_0.4s_ease-in-out_infinite_alternate_0.1s]" style="height: 14px"></span>
            <span class="w-1 bg-white rounded-sm animate-[soundbar_0.4s_ease-in-out_infinite_alternate_0.2s]" style="height: 10px"></span>
          </div>
        }

        <!-- Action buttons -->
        <div class="absolute top-2 right-2 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
          <button
            class="w-6 h-6 rounded-full flex items-center justify-center text-[10px] transition-colors"
            [class]="pad.volume !== 1.0 ? 'bg-status-warning text-black' : 'bg-black/50 text-white hover:bg-accent'"
            (click)="toggleSettingsPopup($event)"
            title="Settings"
          >
            &#9881;
          </button>
          <button
            class="w-6 h-6 bg-black/50 hover:bg-status-success rounded-full flex items-center justify-center text-[10px] text-white transition-colors"
            [class.bg-status-info]="isPreviewing"
            (click)="onPreview($event)"
            [title]="isPreviewing ? 'Stop preview' : 'Preview'"
          >
            {{ isPreviewing ? '&#9632;' : '&#9654;' }}
          </button>
          <button
            class="w-6 h-6 bg-black/50 hover:bg-status-error rounded-full flex items-center justify-center text-xs text-white transition-colors"
            (click)="onRemove($event)"
            title="Remove"
          >
            &times;
          </button>
        </div>

        <!-- Settings popup -->
        @if (showSettingsPopup) {
          <div
            class="absolute top-full left-1/2 -translate-x-1/2 mt-2 bg-surface border border-border rounded-lg p-3 min-w-[180px] z-50 shadow-xl"
            (click)="$event.stopPropagation()"
          >
            <!-- Volume -->
            <div class="mb-3">
              <div class="flex justify-between items-center mb-2 text-xs">
                <span class="text-text-secondary">Volume</span>
                <span class="text-text-primary font-semibold">{{ Math.round(pad.volume * 100) }}%</span>
              </div>
              <input
                type="range"
                [ngModel]="pad.volume"
                (ngModelChange)="onVolumeChange($event)"
                min="0" max="2" step="0.05"
                class="w-full h-1.5 bg-surface-hover rounded-full appearance-none cursor-pointer
                       [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3
                       [&::-webkit-slider-thumb]:bg-white [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:cursor-pointer"
              >
              <div class="flex justify-between text-[10px] text-text-muted mt-1">
                <span>0%</span>
                <span>100%</span>
                <span>200%</span>
              </div>
            </div>

            <!-- Speed -->
            <div class="pt-3 border-t border-border">
              <div class="flex justify-between items-center mb-2 text-xs">
                <span class="text-text-secondary">Speed</span>
                <span class="text-text-primary font-semibold">{{ pad.speed }}x</span>
              </div>
              <div class="flex flex-wrap gap-1">
                @for (s of speedOptions; track s) {
                  <button
                    class="flex-1 min-w-[40px] px-2 py-1 text-[10px] rounded border transition-colors"
                    [class]="pad.speed === s
                      ? 'bg-accent border-accent text-white'
                      : 'bg-surface-hover border-border text-text-secondary hover:text-text-primary'"
                    (click)="onSpeedChange(s)"
                  >
                    {{ s }}x
                  </button>
                }
              </div>
            </div>

            <!-- Reset button -->
            <button
              class="w-full mt-3 py-1.5 text-xs text-text-secondary hover:text-text-primary bg-surface-hover hover:bg-border rounded transition-colors"
              (click)="resetAll()"
            >
              Reset to defaults
            </button>
          </div>
        }
      } @else {
        <!-- Empty pad -->
        <div class="flex flex-col items-center text-text-muted group-hover:text-text-secondary transition-colors">
          <span class="text-3xl font-light">+</span>
          <span class="text-[10px] uppercase tracking-wide">Import</span>
        </div>
      }
    </div>
  `,
  styles: [`
    @keyframes soundbar {
      from { height: 4px; }
      to { height: 16px; }
    }
  `]
})
export class SoundPadComponent {
  @Input({ required: true }) pad!: SoundPad;
  @Input() hotkey?: string;
  @Input() loading = false;
  @Input() isPreviewing = false;

  @Output() play = new EventEmitter<void>();
  @Output() preview = new EventEmitter<void>();
  @Output() import = new EventEmitter<void>();
  @Output() remove = new EventEmitter<void>();
  @Output() volumeChange = new EventEmitter<number>();
  @Output() speedChange = new EventEmitter<number>();

  @HostBinding('class') hostClass = 'relative';
  @HostBinding('class.z-50') get isPopupOpen() { return this.showSettingsPopup; }

  showSettingsPopup = false;
  Math = Math;
  speedOptions = [0.5, 0.75, 1, 1.25, 1.5, 2];

  constructor(private soundboardService: SoundboardService) {}

  get padClasses(): string {
    const base = 'border-2';

    if (!this.pad.sound) {
      return `${base} border-dashed border-white/10 bg-white/5 hover:border-white/25 hover:bg-white/10`;
    }

    let classes = `${base} border-[var(--pad-color)] hover:scale-[1.02] hover:glow-subtle`;
    classes += ` bg-gradient-to-br from-[var(--pad-color)] to-[color-mix(in_srgb,var(--pad-color)_70%,black)]`;

    if (this.pad.isPlaying) {
      classes += ' animate-glow-pulse border-white';
    }

    if (this.isPreviewing) {
      classes += ' border-status-info';
    }

    if (this.loading) {
      classes += ' opacity-50 pointer-events-none';
    }

    return classes;
  }

  @HostListener('document:click', ['$event'])
  onDocumentClick(): void {
    if (this.showSettingsPopup) {
      this.showSettingsPopup = false;
    }
  }

  onClick(event: MouseEvent): void {
    if (this.pad.sound) {
      this.play.emit();
    } else {
      this.import.emit();
    }
  }

  onRightClick(event: MouseEvent): void {
    event.preventDefault();
  }

  onPreview(event: MouseEvent): void {
    event.stopPropagation();
    this.preview.emit();
  }

  onRemove(event: MouseEvent): void {
    event.stopPropagation();
    this.remove.emit();
  }

  toggleSettingsPopup(event: MouseEvent): void {
    event.stopPropagation();
    this.showSettingsPopup = !this.showSettingsPopup;
  }

  onVolumeChange(volume: number): void {
    this.volumeChange.emit(volume);
  }

  onSpeedChange(speed: number): void {
    this.speedChange.emit(speed);
  }

  resetAll(): void {
    this.volumeChange.emit(1.0);
    this.speedChange.emit(1.0);
  }

  formatDuration(seconds: number): string {
    return this.soundboardService.formatDuration(seconds);
  }
}
```

**Step 2: Verify build**

Run:
```bash
cd /Users/didouye/Workspace/voiceboard && npm run build
```

Expected: Build passes

**Step 3: Commit**

```bash
git add src/app/features/soundboard/sound-pad/sound-pad.component.ts
git commit -m "refactor(ui): rewrite SoundPad component with Tailwind

- Gaming/Pro audio visual style
- Glow effects on playing/hover
- Settings popup with volume/speed controls
- Gear icon for settings button"
```

---

## Task 11: Remove Old Components

**Files:**
- Delete: `src/app/features/devices/device-selector.component.ts`
- Delete: `src/app/features/mixer/master-control/master-control.component.ts`
- Delete: `src/app/features/mixer/channel-strip/channel-strip.component.ts`
- Delete: `src/app/features/mixer/level-meters/level-meters.component.ts`

**Step 1: Remove old components**

Run:
```bash
rm /Users/didouye/Workspace/voiceboard/src/app/features/devices/device-selector.component.ts
rm /Users/didouye/Workspace/voiceboard/src/app/features/mixer/master-control/master-control.component.ts
rm /Users/didouye/Workspace/voiceboard/src/app/features/mixer/channel-strip/channel-strip.component.ts
rm /Users/didouye/Workspace/voiceboard/src/app/features/mixer/level-meters/level-meters.component.ts
rmdir /Users/didouye/Workspace/voiceboard/src/app/features/devices
rmdir /Users/didouye/Workspace/voiceboard/src/app/features/mixer/master-control
rmdir /Users/didouye/Workspace/voiceboard/src/app/features/mixer/channel-strip
rmdir /Users/didouye/Workspace/voiceboard/src/app/features/mixer/level-meters
```

**Step 2: Verify build**

Run:
```bash
cd /Users/didouye/Workspace/voiceboard && npm run build
```

Expected: Build passes

**Step 3: Commit**

```bash
git add -A
git commit -m "refactor(ui): remove old components replaced by new layout

- device-selector.component.ts (moved to SettingsPopup)
- master-control.component.ts (moved to SettingsPopup)
- channel-strip.component.ts (unused)
- level-meters.component.ts (replaced by StatusBar)"
```

---

## Task 12: Update Demo Data for New Folder Structure

**Files:**
- Modify: `src/app/core/services/demo-data.ts`

**Step 1: Check and update demo data if needed**

Read the current demo-data.ts file and ensure it works with the new folder structure. Add folder data if needed for demo mode.

**Step 2: Verify the app runs**

Run:
```bash
cd /Users/didouye/Workspace/voiceboard && npm start
```

Expected: App starts and displays the new UI

**Step 3: Commit if changes were made**

```bash
git add src/app/core/services/demo-data.ts
git commit -m "feat(demo): update demo data for new folder structure"
```

---

## Task 13: Final Build and Test

**Step 1: Run full build**

Run:
```bash
cd /Users/didouye/Workspace/voiceboard && npm run build
```

Expected: Build passes

**Step 2: Run Tauri dev**

Run:
```bash
cd /Users/didouye/Workspace/voiceboard && timeout 30 npm run tauri dev || true
```

Expected: App launches with new UI

**Step 3: Take screenshot for verification**

Run:
```bash
cd /Users/didouye/Workspace/voiceboard && npm run screenshot
```

**Step 4: Final commit**

```bash
git add website/assets/screenshot.png
git commit -m "docs: update screenshot with new UI"
```

---

## Task 14: Update Roadmap

**Files:**
- Modify: `ROADMAP.md`

**Step 1: Mark Tailwind migration as complete**

Update the ROADMAP.md to mark "Interface Redesign" as partially complete (Tailwind migration done).

**Step 2: Commit**

```bash
git add ROADMAP.md
git commit -m "docs: update roadmap - Tailwind migration complete"
```

---

## Summary

This plan implements:

1. **Tailwind CSS setup** - Full configuration with custom colors, animations
2. **New layout** - Sidebar (folders) + Main (pads) + Status bar
3. **Settings popup** - Device selection and mixer controls in modal
4. **Folder system** - Basic folder model (Default folder only)
5. **All components rewritten** - Using Tailwind utility classes
6. **Gaming/Pro audio style** - Violet/Magenta neon, glow effects, grain texture

**Files created:**
- `tailwind.config.js`
- `postcss.config.js`
- `src/assets/noise.svg`
- `src/app/shared/components/vu-meter/vu-meter.component.ts`
- `src/app/shared/components/settings-popup/settings-popup.component.ts`
- `src/app/features/mixer/status-bar/status-bar.component.ts`

**Files modified:**
- `package.json` (Tailwind dependencies)
- `src/styles.css` (Tailwind directives)
- `src/app/core/models/audio-device.model.ts` (Folder interface)
- `src/app/core/services/soundboard.service.ts` (folder support)
- `src/app/app.component.ts` (grain overlay)
- `src/app/features/mixer/mixer.component.ts` (new layout)
- `src/app/features/soundboard/soundboard.component.ts` (Tailwind)
- `src/app/features/soundboard/sound-pad/sound-pad.component.ts` (Tailwind)

**Files deleted:**
- `src/app/features/devices/device-selector.component.ts`
- `src/app/features/mixer/master-control/master-control.component.ts`
- `src/app/features/mixer/channel-strip/channel-strip.component.ts`
- `src/app/features/mixer/level-meters/level-meters.component.ts`
