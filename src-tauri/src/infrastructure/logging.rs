//! Logging configuration with Sentry integration

use sentry_tracing::EventFilter;
use std::path::PathBuf;
use std::sync::OnceLock;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Application bundle identifier, kept in sync with `tauri.conf.json`. Used to locate
/// the per-OS log directory without an `AppHandle` (logging starts before the app does).
const BUNDLE_ID: &str = "com.voiceboard.app";

/// Default tracing filter used when `RUST_LOG` is not set.
///
/// Captures our own crate verbosely and the Tauri framework / plugins / audio
/// stack at INFO and above. Those dependencies log through the `log` crate, which
/// is bridged into `tracing` by the global `LogTracer` that `SubscriberInitExt::init`
/// installs (the `tracing-log` feature is enabled by default). `webview` carries logs
/// forwarded from the Angular frontend (captured fully so the unified file/console see
/// everything). Override with `RUST_LOG` for deeper framework debugging.
const DEFAULT_FILTER: &str =
    "voiceboard=debug,tauri=info,tauri_plugin_updater=info,wry=info,cpal=info,webview=trace,info";

/// Keeps the non-blocking file appender's worker thread alive for the whole process.
static FILE_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

/// Directory where the rotating log file is written. Mirrors Tauri's `app_log_dir`
/// for our bundle identifier, computed without an `AppHandle`.
///
/// - macOS: `~/Library/Logs/com.voiceboard.app`
/// - Windows: `%LOCALAPPDATA%\com.voiceboard.app\logs`
/// - Linux: `~/.local/share/com.voiceboard.app/logs`
pub fn log_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|home| home.join("Library/Logs").join(BUNDLE_ID))
    }
    #[cfg(not(target_os = "macos"))]
    {
        dirs::data_local_dir().map(|data| data.join(BUNDLE_ID).join("logs"))
    }
}

/// Initialize the logging system with optional Sentry integration
pub fn init_logging() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));

    // Enable Sentry tracing layer if DSN is available (compile-time or runtime)
    let has_dsn = option_env!("SENTRY_DSN").is_some_and(|s| !s.is_empty())
        || std::env::var("SENTRY_DSN")
            .ok()
            .is_some_and(|s| !s.is_empty());

    let sentry_layer = if has_dsn {
        Some(sentry_tracing::layer().event_filter(|md| {
            // Webview logs reach Sentry through the JS SDK (with browser context and
            // source maps); don't duplicate them from the Rust side.
            if md.target() == "webview" {
                return EventFilter::Ignore;
            }
            match *md.level() {
                tracing::Level::ERROR => EventFilter::Event | EventFilter::Log,
                tracing::Level::TRACE => EventFilter::Ignore,
                _ => EventFilter::Breadcrumb | EventFilter::Log,
            }
        }))
    } else {
        None
    };

    // Rotating daily log file in the OS log directory (best-effort: skipped if the
    // directory can't be resolved or created). Non-blocking so logging never stalls
    // the audio threads; the worker guard is stored to live for the whole process.
    let file_layer = log_dir().and_then(|dir| {
        std::fs::create_dir_all(&dir).ok()?;
        let appender = tracing_appender::rolling::daily(&dir, "voiceboard.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(appender);
        let _ = FILE_GUARD.set(guard);
        Some(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(non_blocking),
        )
    });

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .with(file_layer)
        .with(sentry_layer)
        .init();

    tracing::info!("Logging initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_filter_is_valid() {
        // EnvFilter::new panics on malformed directives; try_new surfaces it as an error.
        assert!(EnvFilter::try_new(DEFAULT_FILTER).is_ok());
    }

    #[test]
    fn log_dir_points_under_bundle_id() {
        if let Some(dir) = log_dir() {
            assert!(
                dir.to_string_lossy().contains(BUNDLE_ID),
                "log dir {dir:?} should live under {BUNDLE_ID}"
            );
        }
    }
}
