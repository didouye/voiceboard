# Sentry Integration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Integrate Sentry error tracking in both Rust backend and Angular frontend, add detailed auto-updater logging, and create a debug console UI.

**Architecture:** Sentry SDK initialized early in both runtimes. Rust uses `sentry` crate with tracing integration. Angular uses `@sentry/angular` with global error handler. Debug console stores logs in memory and displays via overlay panel.

**Tech Stack:** sentry 0.38+, sentry-tracing, @sentry/angular 9+, Angular 20 signals

---

## Task 1: Add Sentry Dependencies (Rust)

**Files:**
- Modify: `src-tauri/Cargo.toml`

**Step 1: Add sentry dependencies**

Add to `[dependencies]` section after the logging dependencies:

```toml
# Error tracking
sentry = { version = "0.38", default-features = false, features = ["backtrace", "contexts", "panic", "reqwest", "rustls"] }
sentry-tracing = "0.38"
```

**Step 2: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compilation succeeds with no errors

**Step 3: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "chore(deps): add sentry crates for error tracking"
```

---

## Task 2: Create Sentry Infrastructure Module (Rust)

**Files:**
- Create: `src-tauri/src/infrastructure/sentry.rs`
- Modify: `src-tauri/src/infrastructure/mod.rs`

**Step 1: Create sentry module**

Create `src-tauri/src/infrastructure/sentry.rs`:

```rust
//! Sentry error tracking configuration

use sentry::ClientInitGuard;

/// Initialize Sentry error tracking
/// Returns a guard that must be kept alive for the duration of the application
pub fn init_sentry() -> Option<ClientInitGuard> {
    let dsn = option_env!("SENTRY_DSN");

    if let Some(dsn) = dsn {
        if dsn.is_empty() {
            tracing::debug!("Sentry DSN is empty, skipping initialization");
            return None;
        }

        tracing::info!("Initializing Sentry error tracking");

        let guard = sentry::init((dsn, sentry::ClientOptions {
            release: Some(env!("CARGO_PKG_VERSION").into()),
            environment: Some(if cfg!(debug_assertions) {
                "development".into()
            } else {
                "production".into()
            }),
            attach_stacktrace: true,
            send_default_pii: false,
            ..Default::default()
        }));

        tracing::info!("Sentry initialized successfully");
        Some(guard)
    } else {
        tracing::debug!("SENTRY_DSN not set, Sentry disabled");
        None
    }
}
```

**Step 2: Export from mod.rs**

Modify `src-tauri/src/infrastructure/mod.rs` to add:

```rust
mod sentry;
pub use sentry::init_sentry;
```

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compilation succeeds

**Step 4: Commit**

```bash
git add src-tauri/src/infrastructure/sentry.rs src-tauri/src/infrastructure/mod.rs
git commit -m "feat(infra): add sentry initialization module"
```

---

## Task 3: Integrate Sentry with Tracing (Rust)

**Files:**
- Modify: `src-tauri/src/infrastructure/logging.rs`

**Step 1: Update logging.rs to integrate sentry-tracing**

Replace the entire file with:

```rust
//! Logging configuration with Sentry integration

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize the logging system with optional Sentry integration
pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("voiceboard=debug,info"));

    // Check if Sentry is configured
    let sentry_layer = if option_env!("SENTRY_DSN").is_some_and(|dsn| !dsn.is_empty()) {
        Some(sentry_tracing::layer())
    } else {
        None
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .with(sentry_layer)
        .init();

    tracing::info!("Logging initialized");
}
```

**Step 2: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compilation succeeds

**Step 3: Commit**

```bash
git add src-tauri/src/infrastructure/logging.rs
git commit -m "feat(infra): integrate sentry-tracing layer with logging"
```

---

## Task 4: Initialize Sentry in Application (Rust)

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Step 1: Import and initialize Sentry before logging**

Update `lib.rs` to initialize Sentry first. Add to imports:

```rust
use crate::infrastructure::init_sentry;
```

Then update the `run()` function to initialize Sentry before logging:

```rust
pub fn run() {
    // Initialize Sentry first (returns guard that must be kept alive)
    let _sentry_guard = init_sentry();

    // Initialize logging (with Sentry integration if enabled)
    infrastructure::init_logging();

    tracing::info!("Starting Voiceboard application");
    // ... rest of the function
```

**Step 2: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compilation succeeds

**Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: initialize sentry at application startup"
```

---

## Task 5: Add Detailed Auto-Updater Logging (Rust)

**Files:**
- Modify: `src-tauri/src/application/commands.rs` (lines 832-881)

**Step 1: Enhance check_for_update with detailed logging**

Replace the `check_for_update` function:

```rust
/// Check if an update is available
#[tauri::command]
pub async fn check_for_update(app: tauri::AppHandle) -> Result<UpdateInfo, String> {
    tracing::info!("Starting update check");

    let updater = match app.updater() {
        Ok(u) => {
            tracing::debug!("Updater instance created successfully");
            u
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create updater instance");
            return Err(e.to_string());
        }
    };

    tracing::debug!("Checking for updates from remote endpoint");

    match updater.check().await {
        Ok(Some(update)) => {
            tracing::info!(
                version = %update.version,
                current_version = env!("CARGO_PKG_VERSION"),
                "Update available"
            );
            Ok(UpdateInfo {
                available: true,
                version: Some(update.version.clone()),
                body: update.body.clone(),
            })
        }
        Ok(None) => {
            tracing::info!(
                current_version = env!("CARGO_PKG_VERSION"),
                "No update available - already on latest version"
            );
            Ok(UpdateInfo {
                available: false,
                version: None,
                body: None,
            })
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                error_debug = ?e,
                current_version = env!("CARGO_PKG_VERSION"),
                "Update check failed"
            );
            // Return error instead of silently failing
            Err(format!("Update check failed: {}", e))
        }
    }
}
```

**Step 2: Enhance install_update with detailed logging**

Replace the `install_update` function:

```rust
/// Download and install an available update, then restart
#[tauri::command]
pub async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    tracing::info!("Starting update installation");

    let updater = match app.updater() {
        Ok(u) => {
            tracing::debug!("Updater instance created for installation");
            u
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create updater instance for installation");
            return Err(format!("Failed to initialize updater: {}", e));
        }
    };

    tracing::debug!("Checking for update before installation");

    let update = match updater.check().await {
        Ok(Some(update)) => {
            tracing::info!(version = %update.version, "Update found, proceeding with download");
            update
        }
        Ok(None) => {
            tracing::warn!("No update available when trying to install");
            return Err("No update available".to_string());
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to check for update during installation");
            return Err(format!("Failed to check for update: {}", e));
        }
    };

    tracing::info!(version = %update.version, "Starting download and installation");

    let download_result = update
        .download_and_install(
            |downloaded, total| {
                if let Some(total) = total {
                    let percent = (downloaded as f64 / total as f64 * 100.0) as u32;
                    if percent % 25 == 0 {
                        tracing::debug!(
                            downloaded_bytes = downloaded,
                            total_bytes = total,
                            percent = percent,
                            "Download progress"
                        );
                    }
                }
            },
            || {
                tracing::info!("Download complete, starting installation");
            },
        )
        .await;

    match download_result {
        Ok(()) => {
            tracing::info!("Update installed successfully, restarting application");
            app.restart();
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                error_debug = ?e,
                "Failed to download and install update"
            );
            return Err(format!("Failed to install update: {}", e));
        }
    }

    Ok(())
}
```

**Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compilation succeeds

**Step 4: Commit**

```bash
git add src-tauri/src/application/commands.rs
git commit -m "feat(updater): add detailed logging with sentry integration"
```

---

## Task 6: Update Angular Frontend - Handle Update Errors

**Files:**
- Modify: `src/app/app.component.ts`

**Step 1: Update error handling to show detailed messages**

Replace the `installUpdate` method to show detailed error:

```typescript
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
```

Also update `checkForUpdate` to handle errors:

```typescript
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
```

**Step 2: Verify build**

Run: `npm run build`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/app/app.component.ts
git commit -m "fix(updater): show detailed error messages on update failure"
```

---

## Task 7: Add Sentry Dependencies (Angular)

**Files:**
- Modify: `package.json`

**Step 1: Install Sentry packages**

Run: `npm install @sentry/angular @sentry/types`

**Step 2: Verify installation**

Run: `npm run build`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add package.json package-lock.json
git commit -m "chore(deps): add sentry angular sdk"
```

---

## Task 8: Initialize Sentry in Angular

**Files:**
- Modify: `src/main.ts`
- Modify: `src/app/app.config.ts`

**Step 1: Initialize Sentry before Angular bootstrap**

Update `src/main.ts`:

```typescript
import * as Sentry from "@sentry/angular";
import { bootstrapApplication } from "@angular/platform-browser";
import { AppComponent } from "./app/app.component";
import { appConfig } from "./app/app.config";

// Initialize Sentry before Angular
const sentryDsn = (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__
  ? import.meta.env?.['VITE_SENTRY_DSN']
  : undefined;

if (sentryDsn) {
  Sentry.init({
    dsn: sentryDsn,
    environment: import.meta.env?.['MODE'] === 'production' ? 'production' : 'development',
    integrations: [],
    tracesSampleRate: 0,
  });
}

bootstrapApplication(AppComponent, appConfig).catch((err) => {
  console.error(err);
  Sentry.captureException(err);
});
```

**Step 2: Add Sentry error handler to app config**

Update `src/app/app.config.ts`:

```typescript
import {
  ApplicationConfig,
  ErrorHandler,
  provideBrowserGlobalErrorListeners,
  provideZoneChangeDetection,
} from "@angular/core";
import { provideRouter } from "@angular/router";
import * as Sentry from "@sentry/angular";

import { routes } from "./app.routes";

export const appConfig: ApplicationConfig = {
  providers: [
    provideBrowserGlobalErrorListeners(),
    provideZoneChangeDetection({ eventCoalescing: true }),
    provideRouter(routes),
    {
      provide: ErrorHandler,
      useValue: Sentry.createErrorHandler({
        showDialog: false,
      }),
    },
  ],
};
```

**Step 3: Verify build**

Run: `npm run build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add src/main.ts src/app/app.config.ts
git commit -m "feat: initialize sentry error tracking in angular"
```

---

## Task 9: Create Debug Console Service

**Files:**
- Create: `src/app/core/services/debug-console.service.ts`
- Modify: `src/app/core/services/index.ts`

**Step 1: Create the debug console service**

Create `src/app/core/services/debug-console.service.ts`:

```typescript
import { Injectable, signal } from '@angular/core';
import { listen } from '@tauri-apps/api/event';

export interface LogEntry {
  timestamp: Date;
  level: 'debug' | 'info' | 'warn' | 'error';
  message: string;
  context?: Record<string, unknown>;
}

@Injectable({ providedIn: 'root' })
export class DebugConsoleService {
  private readonly MAX_LOGS = 500;
  private readonly _logs = signal<LogEntry[]>([]);
  private readonly _isOpen = signal(false);

  readonly logs = this._logs.asReadonly();
  readonly isOpen = this._isOpen.asReadonly();

  constructor() {
    this.setupEventListeners();
    this.log('info', 'Debug console initialized');
  }

  private async setupEventListeners() {
    // Listen for backend log events
    try {
      await listen<{ level: string; message: string; fields?: Record<string, unknown> }>('log-event', (event) => {
        const level = this.parseLevel(event.payload.level);
        this.addLog({
          timestamp: new Date(),
          level,
          message: event.payload.message,
          context: event.payload.fields,
        });
      });
    } catch {
      // Event listener not available
    }
  }

  private parseLevel(level: string): LogEntry['level'] {
    const normalized = level.toLowerCase();
    if (normalized.includes('error')) return 'error';
    if (normalized.includes('warn')) return 'warn';
    if (normalized.includes('debug')) return 'debug';
    return 'info';
  }

  log(level: LogEntry['level'], message: string, context?: Record<string, unknown>) {
    this.addLog({
      timestamp: new Date(),
      level,
      message,
      context,
    });
  }

  private addLog(entry: LogEntry) {
    this._logs.update(logs => {
      const newLogs = [...logs, entry];
      if (newLogs.length > this.MAX_LOGS) {
        return newLogs.slice(-this.MAX_LOGS);
      }
      return newLogs;
    });
  }

  toggle() {
    this._isOpen.update(open => !open);
  }

  open() {
    this._isOpen.set(true);
  }

  close() {
    this._isOpen.set(false);
  }

  clear() {
    this._logs.set([]);
  }

  getLogsByLevel(level: LogEntry['level']): LogEntry[] {
    return this._logs().filter(log => log.level === level);
  }

  exportLogs(): string {
    return this._logs()
      .map(log => `[${log.timestamp.toISOString()}] [${log.level.toUpperCase()}] ${log.message}${log.context ? ' ' + JSON.stringify(log.context) : ''}`)
      .join('\n');
  }
}
```

**Step 2: Export from index**

Update `src/app/core/services/index.ts` to add:

```typescript
export * from './debug-console.service';
```

**Step 3: Verify build**

Run: `npm run build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add src/app/core/services/debug-console.service.ts src/app/core/services/index.ts
git commit -m "feat: add debug console service for log management"
```

---

## Task 10: Create Debug Console Component

**Files:**
- Create: `src/app/core/components/debug-console/debug-console.component.ts`

**Step 1: Create the debug console component**

Create `src/app/core/components/debug-console/debug-console.component.ts`:

```typescript
import { Component, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DebugConsoleService, LogEntry } from '../../services/debug-console.service';

@Component({
  selector: 'app-debug-console',
  standalone: true,
  imports: [CommonModule],
  template: `
    <!-- Toggle Button -->
    <button
      class="debug-toggle"
      (click)="debugConsole.toggle()"
      title="Debug Console"
    >
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <path d="M12 16v-4M12 8h.01"/>
      </svg>
    </button>

    <!-- Console Panel -->
    @if (debugConsole.isOpen()) {
      <div class="debug-panel">
        <div class="debug-header">
          <h3>Debug Console</h3>
          <div class="debug-actions">
            <button (click)="copyLogs()" title="Copy logs">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
              </svg>
            </button>
            <button (click)="debugConsole.clear()" title="Clear logs">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3 6 5 6 21 6"/>
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
              </svg>
            </button>
            <button (click)="debugConsole.close()" title="Close">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="6" x2="6" y2="18"/>
                <line x1="6" y1="6" x2="18" y2="18"/>
              </svg>
            </button>
          </div>
        </div>
        <div class="debug-content">
          @for (log of debugConsole.logs(); track log.timestamp.getTime()) {
            <div class="log-entry" [class]="'log-' + log.level">
              <span class="log-time">{{ formatTime(log.timestamp) }}</span>
              <span class="log-level">{{ log.level.toUpperCase() }}</span>
              <span class="log-message">{{ log.message }}</span>
              @if (log.context) {
                <span class="log-context">{{ formatContext(log.context) }}</span>
              }
            </div>
          } @empty {
            <div class="log-empty">No logs yet</div>
          }
        </div>
      </div>
    }
  `,
  styles: [`
    .debug-toggle {
      position: fixed;
      bottom: 16px;
      right: 16px;
      width: 40px;
      height: 40px;
      border-radius: 50%;
      background: rgba(59, 130, 246, 0.9);
      border: none;
      color: white;
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: 1000;
      transition: transform 0.2s, background 0.2s;

      &:hover {
        transform: scale(1.1);
        background: rgba(59, 130, 246, 1);
      }
    }

    .debug-panel {
      position: fixed;
      bottom: 70px;
      right: 16px;
      width: 500px;
      max-width: calc(100vw - 32px);
      max-height: 400px;
      background: rgba(17, 24, 39, 0.98);
      border: 1px solid rgba(75, 85, 99, 0.5);
      border-radius: 8px;
      z-index: 999;
      display: flex;
      flex-direction: column;
      box-shadow: 0 10px 40px rgba(0, 0, 0, 0.5);
    }

    .debug-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 12px 16px;
      border-bottom: 1px solid rgba(75, 85, 99, 0.5);

      h3 {
        margin: 0;
        font-size: 14px;
        font-weight: 600;
        color: #e5e7eb;
      }
    }

    .debug-actions {
      display: flex;
      gap: 8px;

      button {
        background: none;
        border: none;
        color: #9ca3af;
        cursor: pointer;
        padding: 4px;
        border-radius: 4px;
        display: flex;
        align-items: center;
        justify-content: center;

        &:hover {
          background: rgba(75, 85, 99, 0.5);
          color: #e5e7eb;
        }
      }
    }

    .debug-content {
      flex: 1;
      overflow-y: auto;
      padding: 8px;
      font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
      font-size: 12px;
    }

    .log-entry {
      display: flex;
      gap: 8px;
      padding: 4px 8px;
      border-radius: 4px;
      line-height: 1.4;

      &.log-error {
        background: rgba(239, 68, 68, 0.1);
        color: #fca5a5;
      }

      &.log-warn {
        background: rgba(245, 158, 11, 0.1);
        color: #fcd34d;
      }

      &.log-info {
        color: #93c5fd;
      }

      &.log-debug {
        color: #9ca3af;
      }
    }

    .log-time {
      color: #6b7280;
      flex-shrink: 0;
    }

    .log-level {
      flex-shrink: 0;
      width: 50px;
      font-weight: 600;
    }

    .log-message {
      flex: 1;
      word-break: break-word;
    }

    .log-context {
      color: #6b7280;
      font-size: 11px;
    }

    .log-empty {
      color: #6b7280;
      text-align: center;
      padding: 20px;
    }
  `]
})
export class DebugConsoleComponent {
  debugConsole = inject(DebugConsoleService);

  formatTime(date: Date): string {
    return date.toLocaleTimeString('en-US', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      fractionalSecondDigits: 3
    });
  }

  formatContext(context: Record<string, unknown>): string {
    return JSON.stringify(context);
  }

  copyLogs() {
    const logs = this.debugConsole.exportLogs();
    navigator.clipboard.writeText(logs);
  }
}
```

**Step 2: Verify build**

Run: `npm run build`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/app/core/components/debug-console/debug-console.component.ts
git commit -m "feat(ui): add debug console component with log viewer"
```

---

## Task 11: Integrate Debug Console in App

**Files:**
- Modify: `src/app/app.component.ts`

**Step 1: Add debug console to app component**

Update the imports and template:

```typescript
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
```

**Step 2: Verify build**

Run: `npm run build`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/app/app.component.ts
git commit -m "feat: integrate debug console in app with update logging"
```

---

## Task 12: Add Environment Variables for Sentry DSN

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Create: `.env.example`

**Step 1: Create .env.example**

Create `.env.example`:

```
# Sentry Error Tracking
# Get your DSN from https://sentry.io/
SENTRY_DSN=

# For Angular frontend (optional, uses same DSN)
VITE_SENTRY_DSN=
```

**Step 2: Add to .gitignore if not present**

Ensure `.env` is in `.gitignore`:

```
.env
.env.local
```

**Step 3: Commit**

```bash
git add .env.example .gitignore
git commit -m "docs: add sentry environment variable example"
```

---

## Task 13: Final Integration Test

**Step 1: Run full build**

Run: `cd src-tauri && cargo build && cd .. && npm run build`
Expected: Both builds succeed

**Step 2: Test without Sentry DSN**

Run: `npm run tauri dev`
Expected: App starts, no Sentry errors, debug console works

**Step 3: Commit all remaining changes**

```bash
git add -A
git commit -m "feat: complete sentry integration with debug console"
```

---

## Summary

This plan implements:

1. **Rust Sentry SDK** - Initialized before logging, integrated with tracing
2. **Angular Sentry SDK** - Global error handler with frontend error capture
3. **Enhanced Auto-Updater Logging** - Detailed tracing for every step with structured fields
4. **Debug Console UI** - Floating button in bottom-right, opens log panel with copy/clear
5. **Environment Configuration** - Sentry DSN via environment variables

**Total Tasks:** 13
**Estimated Commits:** 13
