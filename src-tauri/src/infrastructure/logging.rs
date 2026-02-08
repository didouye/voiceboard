//! Logging configuration with Sentry integration

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize the logging system with optional Sentry integration
pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("voiceboard=debug,info"));

    // Enable Sentry tracing layer if DSN is available (compile-time or runtime)
    let has_dsn = option_env!("SENTRY_DSN").is_some_and(|s| !s.is_empty())
        || std::env::var("SENTRY_DSN")
            .ok()
            .is_some_and(|s| !s.is_empty());

    let sentry_layer = if has_dsn {
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
