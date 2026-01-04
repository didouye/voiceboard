# VB-Cable Setup Wizard Design

> **Status:** Approved
> **Date:** 2026-01-02
> **Platform:** Windows (initially)

## Overview

First-run detection and automatic installation of VB-Audio Virtual Cable for users who don't have a virtual audio driver installed.

## User Flow

```
App Startup
    │
    ▼
Check virtual devices (Backend)
    │
    ├── VB-Cable found → Continue to app
    │
    └── No VB-Cable → Show blocking modal
                          │
                          ├── [Download & Install] → Download ZIP → Extract → Run installer (UAC) → "Restart Voiceboard"
                          │
                          └── [Skip for now] → Save preference → Continue with limited functionality
```

## Backend (Rust)

### New Commands

```rust
#[derive(serde::Serialize)]
pub struct VbCableStatus {
    pub installed: bool,
    pub device_name: Option<String>,
}

#[tauri::command]
pub async fn check_vb_cable_installed() -> Result<VbCableStatus, String>

#[tauri::command]
pub async fn install_vb_cable(app: tauri::AppHandle) -> Result<(), String>
```

### Dependencies

- `reqwest` - HTTP download
- `zip` - ZIP extraction
- `std::process::Command` - Run installer

### Download Details

- URL: `https://download.vb-audio.com/Download_CABLE/VBCABLE_Driver_Pack43.zip`
- Extract to: `%TEMP%/voiceboard/vbcable/`
- Run: `VBCABLE_Setup_x64.exe` (requires admin/UAC)

## Frontend (Angular)

### New Component

`src/app/core/components/setup-wizard/setup-wizard.component.ts`

### State Machine

```typescript
type SetupStep = 'checking' | 'missing' | 'downloading' | 'installing' | 'done' | 'error';

interface SetupState {
  step: SetupStep;
  progress?: number;  // 0-100 for download
  error?: string;
}
```

### UI States

| State | Display |
|-------|---------|
| `checking` | Spinner "Checking audio devices..." |
| `missing` | Warning + Download/Skip buttons |
| `downloading` | Progress bar with percentage |
| `installing` | Spinner "Installing... (Administrator required)" |
| `done` | Success message + "Restart Voiceboard" button |
| `error` | Error message + Retry/Open Website buttons |

## Integration

### App Startup (`app.component.ts`)

```typescript
async ngOnInit() {
  // Check VB-Cable BEFORE other initialization
  const vbStatus = await invoke<VbCableStatus>('check_vb_cable_installed');

  if (!vbStatus.installed) {
    this.showSetupWizard = true;
    return;  // Block rest of initialization
  }

  // Normal startup continues...
  await this.checkForUpdate();
}
```

### Persistence

- Store `setup_skipped: boolean` in settings
- If skipped: don't show modal again, but show subtle banner in settings
- Reset flag if user later installs VB-Cable

## Error Handling

| Error | User Message | Recovery |
|-------|--------------|----------|
| No internet | "Internet connection required" | Retry / Open website |
| Download failed | "Download failed" | Retry / Open website |
| Extraction failed | "File corrupted" | Retry |
| UAC denied | "Installation cancelled" | Retry |
| Installer not found | "Installer file missing" | Open folder |

## Security

- HTTPS only for downloads
- Files stored in system temp directory
- Cleanup temp files after installation
- Optional: SHA256 checksum verification

## Future Enhancements

- macOS support (BlackHole detection/installation)
- Linux support (PulseAudio/PipeWire virtual sink creation)
- In-app driver update notifications
