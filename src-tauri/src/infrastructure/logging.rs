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
