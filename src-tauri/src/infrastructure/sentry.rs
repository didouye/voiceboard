//! Sentry error tracking configuration

use sentry::ClientInitGuard;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Global flag to control Sentry Logs emission based on debug mode
pub static DEBUG_MODE_ENABLED: AtomicBool = AtomicBool::new(false);

/// Initialize Sentry error tracking
/// Returns a guard that must be kept alive for the duration of the application
pub fn init_sentry() -> Option<ClientInitGuard> {
    let dsn = env::var("SENTRY_DSN").ok();

    if let Some(dsn) = dsn {
        if dsn.is_empty() {
            tracing::debug!("Sentry DSN is empty, skipping initialization");
            return None;
        }

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
                    if DEBUG_MODE_ENABLED.load(Ordering::Relaxed) {
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
