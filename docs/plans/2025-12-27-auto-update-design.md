# Auto-Update System Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement automatic update checking and installation using Tauri's updater plugin with GitHub Releases as the update source.

**Behavior:** Check on startup, show toast notification if update available, auto-download and restart when user clicks "Update now".

---

## Architecture

### Components

1. **Backend (Rust)**
   - `tauri-plugin-updater` - Tauri's official updater plugin for v2
   - Update check triggered on app startup via Tauri command
   - Downloads update and triggers restart

2. **Frontend (Angular)**
   - Toast notification component showing "Update available: v{version}"
   - "Update now" button that triggers download + restart
   - Auto-dismiss after 10 seconds if user ignores

3. **Update Source**
   - GitHub Releases (already configured)
   - Reads `latest.json` manifest from releases
   - No additional server infrastructure needed

### Flow

```
App starts → Check GitHub Releases → If newer version exists → Show toast → User clicks "Update now" → Download → Auto-restart
```

---

## Task 1: Add Updater Plugin Dependency

**Files:**
- Modify: `src-tauri/Cargo.toml`

**Step 1: Add tauri-plugin-updater to dependencies**

In `src-tauri/Cargo.toml`, add to `[dependencies]`:

```toml
tauri-plugin-updater = "2"
```

**Step 2: Verify**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`

Expected: No errors

**Step 3: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "chore: add tauri-plugin-updater dependency"
```

---

## Task 2: Configure Updater in Tauri

**Files:**
- Modify: `src-tauri/tauri.conf.json`

**Step 1: Add updater plugin configuration**

In `src-tauri/tauri.conf.json`, add the `plugins` section:

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Voiceboard",
  "version": "0.1.0",
  "identifier": "com.voiceboard.app",
  "build": { ... },
  "app": { ... },
  "bundle": { ... },
  "plugins": {
    "updater": {
      "endpoints": [
        "https://github.com/didouye/voiceboard/releases/latest/download/latest.json"
      ],
      "pubkey": ""
    }
  }
}
```

Note: `pubkey` is empty because we're deferring code signing to Phase 7. The updater will work but won't verify signatures.

**Step 2: Verify JSON is valid**

Run: `cat src-tauri/tauri.conf.json | jq .`

Expected: Valid JSON output

**Step 3: Commit**

```bash
git add src-tauri/tauri.conf.json
git commit -m "feat: configure updater plugin endpoint"
```

---

## Task 3: Register Updater Plugin

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Step 1: Register the plugin in Tauri builder**

Find the `run()` function and add the updater plugin:

```rust
.plugin(tauri_plugin_updater::Builder::new().build())
```

**Step 2: Verify**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`

Expected: No errors

**Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: register updater plugin"
```

---

## Task 4: Add Update Commands

**Files:**
- Modify: `src-tauri/src/application/commands.rs`

**Step 1: Add check_for_update command**

```rust
use tauri_plugin_updater::UpdaterExt;

#[derive(serde::Serialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub version: Option<String>,
    pub body: Option<String>,
}

#[tauri::command]
pub async fn check_for_update(app: tauri::AppHandle) -> Result<UpdateInfo, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;

    match updater.check().await {
        Ok(Some(update)) => Ok(UpdateInfo {
            available: true,
            version: Some(update.version.clone()),
            body: update.body.clone(),
        }),
        Ok(None) => Ok(UpdateInfo {
            available: false,
            version: None,
            body: None,
        }),
        Err(e) => {
            // Silent fail - just report no update available
            tracing::warn!("Update check failed: {}", e);
            Ok(UpdateInfo {
                available: false,
                version: None,
                body: None,
            })
        }
    }
}

#[tauri::command]
pub async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;

    if let Some(update) = updater.check().await.map_err(|e| e.to_string())? {
        update.download_and_install(|_, _| {}, || {}).await.map_err(|e| e.to_string())?;
        app.restart();
    }

    Ok(())
}
```

**Step 2: Register commands in lib.rs**

Add to `.invoke_handler()`:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    application::commands::check_for_update,
    application::commands::install_update,
])
```

**Step 3: Verify**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`

Expected: No errors

**Step 4: Commit**

```bash
git add src-tauri/src/application/commands.rs src-tauri/src/lib.rs
git commit -m "feat: add update check and install commands"
```

---

## Task 5: Generate latest.json in CI

**Files:**
- Modify: `.github/workflows/release.yml`

**Step 1: Add step to generate latest.json**

Add after the "Generate checksums" step in the release job:

```yaml
      - name: Generate update manifest
        run: |
          VERSION="${{ needs.version.outputs.release_tag }}"
          APP_VERSION="${{ needs.version.outputs.app_version }}"

          cat > artifacts/latest.json << EOF
          {
            "version": "$APP_VERSION",
            "notes": "Voiceboard $VERSION - Automated release",
            "pub_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
            "platforms": {
              "darwin-aarch64": {
                "url": "https://github.com/${{ github.repository }}/releases/download/v$VERSION/voiceboard-$VERSION-macos-arm64.tar.gz",
                "signature": ""
              },
              "darwin-x86_64": {
                "url": "https://github.com/${{ github.repository }}/releases/download/v$VERSION/voiceboard-$VERSION-macos-x64.tar.gz",
                "signature": ""
              },
              "linux-x86_64": {
                "url": "https://github.com/${{ github.repository }}/releases/download/v$VERSION/voiceboard-$VERSION-linux-x64.tar.gz",
                "signature": ""
              },
              "windows-x86_64": {
                "url": "https://github.com/${{ github.repository }}/releases/download/v$VERSION/voiceboard-$VERSION-windows-x64.zip",
                "signature": ""
              }
            }
          }
          EOF

          cat artifacts/latest.json
```

**Step 2: Update files list in release step**

The existing `artifacts/voiceboard-*` glob should catch `latest.json`, but explicitly add it:

```yaml
          files: |
            artifacts/voiceboard-*
            artifacts/SHA256SUMS.txt
            artifacts/latest.json
```

**Step 3: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: generate latest.json update manifest"
```

---

## Task 6: Create Toast Component

**Files:**
- Create: `src/app/shared/components/toast/toast.component.ts`
- Create: `src/app/shared/components/toast/toast.component.html`
- Create: `src/app/shared/components/toast/toast.component.css`
- Create: `src/app/shared/services/toast.service.ts`

**Step 1: Create toast service**

```typescript
// src/app/shared/services/toast.service.ts
import { Injectable, signal } from '@angular/core';

export interface Toast {
  id: string;
  message: string;
  action?: {
    label: string;
    callback: () => void;
  };
  duration?: number;
}

@Injectable({ providedIn: 'root' })
export class ToastService {
  toasts = signal<Toast[]>([]);

  show(toast: Omit<Toast, 'id'>): string {
    const id = crypto.randomUUID();
    this.toasts.update(t => [...t, { ...toast, id }]);

    if (toast.duration !== 0) {
      setTimeout(() => this.dismiss(id), toast.duration ?? 10000);
    }

    return id;
  }

  dismiss(id: string): void {
    this.toasts.update(t => t.filter(toast => toast.id !== id));
  }
}
```

**Step 2: Create toast component**

```typescript
// src/app/shared/components/toast/toast.component.ts
import { Component, inject } from '@angular/core';
import { ToastService } from '../../services/toast.service';

@Component({
  selector: 'app-toast',
  standalone: true,
  templateUrl: './toast.component.html',
  styleUrls: ['./toast.component.css']
})
export class ToastComponent {
  toastService = inject(ToastService);
}
```

```html
<!-- src/app/shared/components/toast/toast.component.html -->
<div class="toast-container">
  @for (toast of toastService.toasts(); track toast.id) {
    <div class="toast">
      <span class="toast-message">{{ toast.message }}</span>
      @if (toast.action) {
        <button class="toast-action" (click)="toast.action.callback()">
          {{ toast.action.label }}
        </button>
      }
      <button class="toast-dismiss" (click)="toastService.dismiss(toast.id)">×</button>
    </div>
  }
</div>
```

```css
/* src/app/shared/components/toast/toast.component.css */
.toast-container {
  position: fixed;
  bottom: 20px;
  right: 20px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.toast {
  background: #333;
  color: white;
  padding: 12px 16px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  gap: 12px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  animation: slideIn 0.3s ease;
}

@keyframes slideIn {
  from { transform: translateX(100%); opacity: 0; }
  to { transform: translateX(0); opacity: 1; }
}

.toast-action {
  background: #007bff;
  color: white;
  border: none;
  padding: 6px 12px;
  border-radius: 4px;
  cursor: pointer;
}

.toast-action:hover {
  background: #0056b3;
}

.toast-dismiss {
  background: none;
  border: none;
  color: #999;
  font-size: 18px;
  cursor: pointer;
  padding: 0 4px;
}
```

**Step 3: Add toast component to app.component.html**

```html
<app-toast />
<!-- existing content -->
```

**Step 4: Commit**

```bash
git add src/app/shared/
git commit -m "feat: add toast notification component"
```

---

## Task 7: Implement Update Check on Init

**Files:**
- Modify: `src/app/app.component.ts`

**Step 1: Add update check on initialization**

```typescript
import { invoke } from '@tauri-apps/api/core';
import { ToastService } from './shared/services/toast.service';

interface UpdateInfo {
  available: boolean;
  version?: string;
  body?: string;
}

@Component({ ... })
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
```

**Step 2: Commit**

```bash
git add src/app/app.component.ts
git commit -m "feat: check for updates on app startup"
```

---

## Task 8: Update ROADMAP

**Files:**
- Modify: `ROADMAP.md`

**Step 1: Mark Auto-Update as done in Phase 2**

Update Phase 2 Done section to include:

```markdown
- [x] **Auto-Update System**
  - Check for updates on startup
  - Toast notification with "Update now" button
  - Auto-download and restart
```

**Step 2: Add Phase 7 for Code Signing**

Add new section after Phase 6:

```markdown
## Phase 7 - Code Signing & Security

### To Do
- [ ] **Windows Code Signing**
  - Obtain code signing certificate
  - Sign executables and installers
  - Eliminate SmartScreen warnings

- [ ] **macOS Code Signing & Notarization**
  - Obtain Apple Developer certificate
  - Sign app bundle
  - Notarize with Apple
  - Eliminate Gatekeeper warnings

- [ ] **Signed Update Manifests**
  - Generate signing keypair
  - Sign latest.json manifests
  - Configure updater pubkey verification
```

**Step 3: Commit**

```bash
git add ROADMAP.md
git commit -m "docs: update ROADMAP with auto-update progress and Phase 7"
```

---

## Task 9: Test and Push

**Files:** None

**Step 1: Run tests**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
npm test
```

**Step 2: Build locally**

```bash
npm run tauri build
```

**Step 3: Push all changes**

```bash
git push origin main
```

**Step 4: Verify**

- Check GitHub Actions completes
- Check release includes `latest.json`
- Test update notification in app (will show "no update" since version matches)

---

## Summary

| Task | Description | Commit |
|------|-------------|--------|
| 1 | Add updater dependency | `chore: add tauri-plugin-updater dependency` |
| 2 | Configure updater endpoint | `feat: configure updater plugin endpoint` |
| 3 | Register updater plugin | `feat: register updater plugin` |
| 4 | Add update commands | `feat: add update check and install commands` |
| 5 | Generate latest.json in CI | `ci: generate latest.json update manifest` |
| 6 | Create toast component | `feat: add toast notification component` |
| 7 | Check for updates on init | `feat: check for updates on app startup` |
| 8 | Update ROADMAP | `docs: update ROADMAP with auto-update progress and Phase 7` |
| 9 | Test and push | (verification) |

## Error Handling

- Network failure → Silent fail, no toast
- Download failure → Show error toast
- Invalid manifest → Silent fail, log warning
- No retry logic - next startup will check again
