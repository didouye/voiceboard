# Sentry Logs Integration - Design

> **Date:** 2026-02-07
> **Status:** Ready for implementation

## Overview

Enable Sentry Logs (Explore > Logs) across all 3 components to allow remote debugging without direct access to user machines.

- **Backend Django**: All logs (DEBUG+) always sent to Sentry Logs
- **Frontend Angular**: Browser console logs sent to Sentry Logs only when debug mode is ON
- **Rust/Tauri**: Tracing logs sent to Sentry Logs only when debug mode is ON
- **All components**: Errors continue to create Sentry issues as before

## Backend Django

### Changes to `backend/config/settings/production.py`

Add `enable_logs=True` and configure `LoggingIntegration`:

```python
import logging
import sentry_sdk
from sentry_sdk.integrations.logging import LoggingIntegration

SENTRY_DSN = os.environ.get("SENTRY_DSN")
if SENTRY_DSN:
    sentry_sdk.init(
        dsn=SENTRY_DSN,
        enable_logs=True,
        traces_sample_rate=0.1,
        profiles_sample_rate=0.1,
        integrations=[
            LoggingIntegration(
                level=logging.DEBUG,
                event_level=logging.ERROR,
                sentry_logs_level=logging.DEBUG,
            ),
        ],
    )
```

- `sentry_logs_level=logging.DEBUG`: All Python logs (DEBUG+) → Sentry Logs
- `event_level=logging.ERROR`: Only ERROR+ creates Sentry issues
- No dependency change needed (`sentry-sdk[django]` already in prod deps)

## Frontend Angular

### Changes to `src/main.ts`

Add debug mode state tracking and configure Sentry Logs:

```typescript
import { consoleLoggingIntegration } from "@sentry/angular";

let debugModeEnabled = false;

// Listen for debug mode changes
listen<boolean>("debug-mode-changed", (event) => {
  debugModeEnabled = event.payload;
});
invoke<boolean>("get_debug_mode").then((enabled) => {
  debugModeEnabled = enabled;
});

Sentry.init({
  dsn,
  release: version,
  environment: "production",
  enableLogs: true,
  integrations: [consoleLoggingIntegration()],
  beforeSendLog(log) {
    return debugModeEnabled ? log : null;
  },
  tracesSampleRate: 0,
});
```

- `consoleLoggingIntegration()`: Captures all `console.log/warn/error/debug` calls
- `beforeSendLog`: Gates on debug mode — drops all logs when disabled
- No duplication with Rust logs (Rust sends its own, browser console is separate)
- No version upgrade needed (`@sentry/angular ^10.32.1` already supports this)

## Rust/Tauri

### Changes to `src-tauri/Cargo.toml`

Upgrade sentry crates and add `logs` feature:

```toml
sentry = { version = "0.46", default-features = false, features = ["backtrace", "contexts", "panic", "reqwest", "rustls", "logs"] }
sentry-tracing = "0.46"
```

### Changes to `src-tauri/src/infrastructure/sentry.rs`

Add `AtomicBool` for debug mode and `before_send_log` callback:

```rust
use std::sync::atomic::{AtomicBool, Ordering};

pub static DEBUG_MODE_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn init_sentry() -> Option<ClientInitGuard> {
    // ... existing DSN check ...
    let guard = sentry::init((dsn, sentry::ClientOptions {
        // ... existing options ...
        before_send_log: Some(Arc::new(|log| {
            if DEBUG_MODE_ENABLED.load(Ordering::Relaxed) {
                Some(log)
            } else {
                None
            }
        })),
        ..Default::default()
    }));
    Some(guard)
}
```

### Changes to `src-tauri/src/application/commands.rs`

Update `set_debug_mode` to sync the `AtomicBool`:

```rust
use crate::infrastructure::sentry::DEBUG_MODE_ENABLED;

pub fn set_debug_mode(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    // ... existing store logic ...
    DEBUG_MODE_ENABLED.store(enabled, Ordering::Relaxed);
    // ... existing event emit ...
}
```

Also initialize `AtomicBool` in `get_debug_mode` on first call or at startup.

## Files Changed

| File | Change |
|------|--------|
| `backend/config/settings/production.py` | Enable Sentry Logs, add LoggingIntegration |
| `src/main.ts` | Enable Sentry Logs, consoleLoggingIntegration, beforeSendLog |
| `src-tauri/Cargo.toml` | Upgrade sentry 0.38→0.46, add "logs" feature |
| `src-tauri/src/infrastructure/sentry.rs` | AtomicBool + before_send_log |
| `src-tauri/src/application/commands.rs` | Sync AtomicBool in set_debug_mode |
