# Device Auto-Selection Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Auto-select virtual output device, auto-start mixer, and implement silent restart on device change.

**Architecture:** Backend provides prioritized virtual output list and physical output list. Frontend auto-selects devices on startup and auto-starts mixer when config is valid.

**Tech Stack:** Rust (Tauri backend), Angular (frontend), CPAL (audio)

---

## Task 1: Backend - Add Physical Outputs Command

**Files:**
- Modify: `src-tauri/src/adapters/cpal_device_manager.rs`
- Modify: `src-tauri/src/application/commands.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Add find_physical_outputs method**

In `src-tauri/src/adapters/cpal_device_manager.rs`, add after `find_physical_inputs`:

```rust
/// Find physical output devices (speakers, headphones - for preview/monitoring)
pub fn find_physical_outputs(&self) -> Result<Vec<AudioDevice>, DeviceManagerError> {
    self.list_devices_by_type(DeviceType::OutputPhysical)
}
```

**Step 2: Add Tauri command**

In `src-tauri/src/application/commands.rs`, add after `get_virtual_output_devices`:

```rust
/// Get physical output devices (speakers, headphones - for preview/monitoring)
#[tauri::command]
pub async fn get_physical_output_devices() -> ApiResponse<Vec<AudioDeviceDto>> {
    let manager = CpalDeviceManager::new();

    match manager.find_physical_outputs() {
        Ok(devices) => {
            let dtos: Vec<AudioDeviceDto> = devices.into_iter().map(AudioDeviceDto::from).collect();
            ApiResponse::ok(dtos)
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}
```

**Step 3: Register command in lib.rs**

In `src-tauri/src/lib.rs`, add `get_physical_output_devices` to the `invoke_handler` list.

**Step 4: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Success

**Step 5: Commit**

```bash
git add src-tauri/
git commit -m "feat: add get_physical_output_devices command"
```

---

## Task 2: Backend - Add Virtual Outputs by Priority Command

**Files:**
- Modify: `src-tauri/src/adapters/cpal_device_manager.rs`
- Modify: `src-tauri/src/application/commands.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Add priority constants and method**

In `src-tauri/src/adapters/cpal_device_manager.rs`, add after `VB_CABLE_PATTERNS`:

```rust
/// Priority order for virtual output devices (lower index = higher priority)
const VIRTUAL_OUTPUT_PRIORITY: &[&str] = &[
    "cable output (vb-audio",
    "cable input (vb-audio",
    "vb-audio virtual cable",
    "voicemeeter",
    "blackhole",
    "virtual audio",
    "loopback",
];

impl CpalDeviceManager {
    // ... existing methods ...

    /// Get priority score for a virtual device (lower = higher priority)
    fn get_virtual_device_priority(name: &str) -> usize {
        let name_lower = name.to_lowercase();
        VIRTUAL_OUTPUT_PRIORITY
            .iter()
            .position(|pattern| name_lower.contains(pattern))
            .unwrap_or(usize::MAX)
    }

    /// Find virtual output devices sorted by priority
    pub fn find_virtual_outputs_by_priority(&self) -> Result<Vec<AudioDevice>, DeviceManagerError> {
        let mut devices = self.list_devices_by_type(DeviceType::OutputVirtual)?;
        devices.sort_by_key(|d| Self::get_virtual_device_priority(d.name()));
        Ok(devices)
    }
}
```

**Step 2: Add Tauri command**

In `src-tauri/src/application/commands.rs`, add:

```rust
/// Get virtual output devices sorted by priority (VB-Cable first, then Voicemeeter, etc.)
#[tauri::command]
pub async fn get_virtual_outputs_by_priority() -> ApiResponse<Vec<AudioDeviceDto>> {
    let manager = CpalDeviceManager::new();

    match manager.find_virtual_outputs_by_priority() {
        Ok(devices) => {
            tracing::info!("[get_virtual_outputs_by_priority] Found {} virtual outputs", devices.len());
            for (i, dev) in devices.iter().enumerate() {
                tracing::info!("[get_virtual_outputs_by_priority]   {}: {}", i + 1, dev.name());
            }
            let dtos: Vec<AudioDeviceDto> = devices.into_iter().map(AudioDeviceDto::from).collect();
            ApiResponse::ok(dtos)
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}
```

**Step 3: Register command in lib.rs**

In `src-tauri/src/lib.rs`, add `get_virtual_outputs_by_priority` to the `invoke_handler` list.

**Step 4: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Success

**Step 5: Commit**

```bash
git add src-tauri/
git commit -m "feat: add get_virtual_outputs_by_priority command"
```

---

## Task 3: Frontend - Add TauriService Methods

**Files:**
- Modify: `src/app/core/services/tauri.service.ts`

**Step 1: Add getPhysicalOutputDevices method**

After `getVirtualOutputDevices()`:

```typescript
/**
 * Get physical output devices (speakers, headphones - for preview/monitoring)
 */
async getPhysicalOutputDevices(): Promise<AudioDevice[]> {
  const response = await invoke<ApiResponse<AudioDevice[]>>('get_physical_output_devices');
  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to get physical output devices');
  }
  return this.mapDevices(response.data);
}

/**
 * Get virtual output devices sorted by priority (VB-Cable first)
 */
async getVirtualOutputsByPriority(): Promise<AudioDevice[]> {
  const response = await invoke<ApiResponse<AudioDevice[]>>('get_virtual_outputs_by_priority');
  if (!response.success || !response.data) {
    throw new Error(response.error || 'Failed to get virtual outputs by priority');
  }
  return this.mapDevices(response.data);
}
```

**Step 2: Verify build**

Run: `npm run build`
Expected: Success

**Step 3: Commit**

```bash
git add src/app/core/services/tauri.service.ts
git commit -m "feat: add getPhysicalOutputDevices and getVirtualOutputsByPriority"
```

---

## Task 4: Frontend - Refactor DeviceSelectorComponent

**Files:**
- Modify: `src/app/features/devices/device-selector.component.ts`

**Step 1: Update state and loading logic**

Replace the existing `loadData` method and add new signals:

```typescript
// Add new signals after existing ones
private _virtualOutputDevices = signal<AudioDevice[]>([]);
private _physicalOutputDevices = signal<AudioDevice[]>([]);

readonly virtualOutputDevices = this._virtualOutputDevices.asReadonly();
readonly physicalOutputDevices = this._physicalOutputDevices.asReadonly();

// Computed: show virtual output selector only if multiple devices
readonly showVirtualOutputSelector = computed(() => this._virtualOutputDevices().length > 1);

async loadData(): Promise<void> {
  this._loading.set(true);
  this._error.set(null);

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

    console.log('[DeviceSelector] Input devices:', inputDevices.length);
    console.log('[DeviceSelector] Physical outputs:', physicalOutputs.length);
    console.log('[DeviceSelector] Virtual outputs:', virtualOutputs.length);
  } catch (err) {
    this._error.set(err instanceof Error ? err.message : 'Failed to load devices');
  } finally {
    this._loading.set(false);
  }
}
```

**Step 2: Update template**

Replace the entire template with:

```html
<div class="device-selector">
  <h2>Audio Devices</h2>

  @if (loading()) {
    <div class="loading">Loading devices...</div>
  } @else if (error()) {
    <div class="error">{{ error() }}</div>
  } @else {
    <!-- Input Device Selection -->
    <div class="device-group">
      <label>
        <span class="label-icon">🎤</span>
        <span class="label-text">Input Device (Microphone)</span>
      </label>
      @if (inputDevices().length === 0) {
        <div class="no-device-warning">No input device available</div>
      } @else {
        <select (change)="onInputDeviceChange($event)" class="device-select">
          <option value="" [selected]="!selectedInputId()">-- Select Microphone --</option>
          @for (device of inputDevices(); track device.id) {
            <option [value]="device.id" [selected]="device.id === selectedInputId()">
              {{ device.name }} @if (device.isDefault) { (Default) }
            </option>
          }
        </select>
      }
    </div>

    <!-- Preview Output Device Selection -->
    <div class="device-group">
      <label>
        <span class="label-icon">🎧</span>
        <span class="label-text">Preview Output (Monitoring)</span>
      </label>
      <select (change)="onPreviewDeviceChange($event)" class="device-select">
        <option value="" [selected]="!selectedPreviewId()">-- System Default --</option>
        @for (device of physicalOutputDevices(); track device.id) {
          <option [value]="device.id" [selected]="device.id === selectedPreviewId()">
            {{ device.name }} @if (device.isDefault) { (Default) }
          </option>
        }
      </select>
    </div>

    <!-- Virtual Output Selection (only if multiple) -->
    @if (showVirtualOutputSelector()) {
      <div class="device-group">
        <label>
          <span class="label-icon">🔊</span>
          <span class="label-text">Virtual Output</span>
        </label>
        <select (change)="onOutputDeviceChange($event)" class="device-select">
          @for (device of virtualOutputDevices(); track device.id) {
            <option [value]="device.id" [selected]="device.id === selectedOutputId()">
              {{ device.name }}
            </option>
          }
        </select>
      </div>
    }

    <!-- Status -->
    <div class="status-section">
      <div class="status-item" [class.ready]="isConfigured()" [class.error]="inputDevices().length === 0">
        <span class="status-dot"></span>
        <span>
          @if (inputDevices().length === 0) {
            No input device
          } @else if (isConfigured()) {
            Ready to mix
          } @else {
            Select devices to start
          }
        </span>
      </div>
    </div>

    <!-- Refresh Button -->
    <button class="btn-refresh" (click)="refreshDevices()">
      🔄 Refresh Devices
    </button>
  }
</div>
```

**Step 3: Add no-device-warning style**

Add to styles:

```css
.no-device-warning {
  padding: 12px 15px;
  background: rgba(231, 76, 60, 0.1);
  border: 1px solid rgba(231, 76, 60, 0.3);
  border-radius: 8px;
  color: #e74c3c;
  font-size: 0.9rem;
}

.status-item.error .status-dot {
  background: #e74c3c;
}

.status-item.error {
  color: #e74c3c;
}
```

**Step 4: Remove _outputDevices signal (no longer needed)**

Remove:
```typescript
private _outputDevices = signal<AudioDevice[]>([]);
readonly outputDevices = this._outputDevices.asReadonly();
```

**Step 5: Verify build**

Run: `npm run build`
Expected: Success

**Step 6: Commit**

```bash
git add src/app/features/devices/
git commit -m "refactor: DeviceSelector with physical/virtual separation"
```

---

## Task 5: Frontend - Add MixerService Auto-Start and Restart

**Files:**
- Modify: `src/app/core/services/mixer.service.ts`

**Step 1: Add restartIfRunning method**

```typescript
/**
 * Restart mixing silently if currently running
 * Used when device configuration changes
 */
async restartIfRunning(): Promise<void> {
  if (this._isRunning()) {
    console.log('[MixerService] Restarting mixer silently...');
    try {
      await this.tauri.stopMixing();
      await this.tauri.startMixing();
      console.log('[MixerService] Mixer restarted successfully');
    } catch (err) {
      this._error.set(err instanceof Error ? err.message : 'Failed to restart mixer');
      this._isRunning.set(false);
    }
  }
}
```

**Step 2: Modify initialize to auto-start**

Replace the existing `initialize` method:

```typescript
/**
 * Initialize the mixer service and auto-start if config is valid
 */
async initialize(): Promise<void> {
  this._loading.set(true);
  this._error.set(null);

  try {
    const [config, devices, virtualDriver, isMixing] = await Promise.all([
      this.tauri.getMixerConfig(),
      this.tauri.getAudioDevices(),
      this.tauri.checkVirtualDriver(),
      this.tauri.isMixing()
    ]);

    this._config.set(config);
    this._devices.set(devices);
    this._virtualDriverInstalled.set(virtualDriver);
    this._isRunning.set(isMixing);

    // Auto-start if not already running and config is valid
    if (!isMixing) {
      const settings = await this.tauri.loadSettings();
      const hasInput = !!settings.audio.inputDeviceId;
      const hasOutput = !!settings.audio.outputDeviceId;

      if (hasInput && hasOutput) {
        console.log('[MixerService] Auto-starting mixer...');
        await this.start();
      } else {
        console.log('[MixerService] Cannot auto-start: missing input or output device');
      }
    }
  } catch (err) {
    this._error.set(err instanceof Error ? err.message : 'Initialization failed');
    console.error('Failed to initialize mixer:', err);
  } finally {
    this._loading.set(false);
  }
}
```

**Step 3: Verify build**

Run: `npm run build`
Expected: Success

**Step 4: Commit**

```bash
git add src/app/core/services/mixer.service.ts
git commit -m "feat: add auto-start and restartIfRunning to MixerService"
```

---

## Task 6: Frontend - Auto-Select Devices on Startup

**Files:**
- Modify: `src/app/app.component.ts`
- Modify: `src/app/features/devices/device-selector.component.ts`

**Step 1: Add auto-selection logic to DeviceSelectorComponent**

Add a new method and call it from loadData:

```typescript
/**
 * Auto-select devices if not already configured
 */
private async autoSelectDevices(): Promise<void> {
  const settings = this._settings();
  if (!settings) return;

  let needsSave = false;

  // Auto-select input if not set
  if (!settings.audio.inputDeviceId) {
    const defaultInput = this._inputDevices().find(d => d.isDefault);
    if (defaultInput) {
      console.log('[DeviceSelector] Auto-selecting input:', defaultInput.name);
      await this.tauri.setInputDevice(defaultInput.id);
      settings.audio.inputDeviceId = defaultInput.id;
      needsSave = true;
    }
  }

  // Auto-select virtual output if not set (or if saved one doesn't exist)
  const virtualOutputs = this._virtualOutputDevices();
  if (virtualOutputs.length > 0) {
    const savedOutputExists = virtualOutputs.some(d => d.id === settings.audio.outputDeviceId);
    if (!settings.audio.outputDeviceId || !savedOutputExists) {
      const firstVirtual = virtualOutputs[0]; // Already sorted by priority
      console.log('[DeviceSelector] Auto-selecting virtual output:', firstVirtual.name);
      await this.tauri.setOutputDevice(firstVirtual.id);
      settings.audio.outputDeviceId = firstVirtual.id;
      needsSave = true;
    }
  }

  if (needsSave) {
    this._settings.set({ ...settings });
  }
}
```

**Step 2: Call autoSelectDevices from loadData**

At the end of loadData, after setting all signals:

```typescript
// Auto-select devices if needed
await this.autoSelectDevices();
```

**Step 3: Add restart on device change**

Inject MixerService and call restartIfRunning:

```typescript
import { MixerService } from '../../core/services';

constructor(private tauri: TauriService, private mixer: MixerService) {}

async onInputDeviceChange(event: Event): Promise<void> {
  const select = event.target as HTMLSelectElement;
  const deviceId = select.value || null;

  try {
    await this.tauri.setInputDevice(deviceId);
    const settings = this._settings();
    if (settings) {
      this._settings.set({
        ...settings,
        audio: { ...settings.audio, inputDeviceId: deviceId }
      });
    }
    // Silent restart if mixer is running
    await this.mixer.restartIfRunning();
  } catch (err) {
    console.error('Failed to set input device:', err);
  }
}

async onPreviewDeviceChange(event: Event): Promise<void> {
  const select = event.target as HTMLSelectElement;
  const deviceId = select.value || null;

  try {
    await this.tauri.setPreviewDevice(deviceId);
    const settings = this._settings();
    if (settings) {
      this._settings.set({
        ...settings,
        audio: { ...settings.audio, previewDeviceId: deviceId }
      });
    }
    // Preview doesn't need mixer restart (separate engine)
  } catch (err) {
    console.error('Failed to set preview device:', err);
  }
}

async onOutputDeviceChange(event: Event): Promise<void> {
  const select = event.target as HTMLSelectElement;
  const deviceId = select.value || null;

  try {
    await this.tauri.setOutputDevice(deviceId);
    const settings = this._settings();
    if (settings) {
      this._settings.set({
        ...settings,
        audio: { ...settings.audio, outputDeviceId: deviceId }
      });
    }
    // Silent restart if mixer is running
    await this.mixer.restartIfRunning();
  } catch (err) {
    console.error('Failed to set output device:', err);
  }
}
```

**Step 4: Verify build**

Run: `npm run build`
Expected: Success

**Step 5: Commit**

```bash
git add src/app/
git commit -m "feat: auto-select devices and silent restart on change"
```

---

## Task 7: Format and Final Verification

**Step 1: Format code**

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml
```

**Step 2: Run clippy**

```bash
cargo clippy --manifest-path src-tauri/Cargo.toml
```

**Step 3: Build frontend**

```bash
npm run build
```

**Step 4: Final commit and push**

```bash
git add -A
git commit -m "chore: format code"
git push
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Backend: get_physical_output_devices | cpal_device_manager.rs, commands.rs, lib.rs |
| 2 | Backend: get_virtual_outputs_by_priority | cpal_device_manager.rs, commands.rs, lib.rs |
| 3 | Frontend: TauriService methods | tauri.service.ts |
| 4 | Frontend: Refactor DeviceSelector | device-selector.component.ts |
| 5 | Frontend: MixerService auto-start | mixer.service.ts |
| 6 | Frontend: Auto-select + restart | device-selector.component.ts |
| 7 | Format and verify | all |
