//! Sentry error tracking configuration

use sentry::ClientInitGuard;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Global flag to control Sentry Logs emission based on debug mode
pub static DEBUG_MODE_ENABLED: AtomicBool = AtomicBool::new(false);

/// Resolve the Sentry DSN: compile-time value (embedded in binary) takes priority,
/// with a runtime env var fallback for local development.
fn resolve_dsn() -> Option<String> {
    // 1. Compile-time: embedded during CI build via SENTRY_DSN env var
    if let Some(dsn) = option_env!("SENTRY_DSN") {
        if !dsn.is_empty() {
            return Some(dsn.to_string());
        }
    }
    // 2. Runtime fallback: for local `cargo run` / `npm run tauri dev`
    std::env::var("SENTRY_DSN")
        .ok()
        .filter(|s| !s.is_empty())
}

/// Initialize Sentry error tracking
/// Returns a guard that must be kept alive for the duration of the application
pub fn init_sentry() -> Option<ClientInitGuard> {
    let dsn = resolve_dsn();

    if let Some(dsn) = dsn {
        tracing::info!("Initializing Sentry error tracking");

        let guard = sentry::init((
            dsn,
            sentry::ClientOptions {
                release: Some(env!("CARGO_PKG_VERSION").into()),
                environment: Some(if cfg!(debug_assertions) {
                    "development".into()
                } else {
                    "production".into()
                }),
                attach_stacktrace: true,
                send_default_pii: false,
                before_send_log: Some(Arc::new(|log| {
                    // Always send error/fatal logs; filter verbose logs by debug mode
                    if matches!(
                        log.level,
                        sentry::protocol::LogLevel::Error | sentry::protocol::LogLevel::Fatal
                    ) || DEBUG_MODE_ENABLED.load(Ordering::Relaxed)
                    {
                        Some(log)
                    } else {
                        None
                    }
                })),
                ..Default::default()
            },
        ));

        tracing::info!("Sentry initialized successfully");
        Some(guard)
    } else {
        tracing::debug!("SENTRY_DSN not set, Sentry disabled");
        None
    }
}
