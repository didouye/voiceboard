//! Sentry error tracking configuration

use sentry::protocol::LogLevel;
use sentry::ClientInitGuard;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

/// Global flag to control Sentry Logs emission based on debug mode
pub static DEBUG_MODE_ENABLED: AtomicBool = AtomicBool::new(false);

/// Persistent per-install identifier, attached to every event/log so all events
/// from one machine are grouped in Sentry. Set once from the Tauri `setup()` hook.
/// Applied centrally in `before_send` / `before_send_log` so it covers events from
/// any thread (sentry hubs are thread-local; a global static is not).
static INSTALL_ID: OnceLock<String> = OnceLock::new();

/// Record the install id used to tag Sentry events. Idempotent; first value wins.
pub fn set_install_id(install_id: String) {
    let _ = INSTALL_ID.set(install_id);
}

/// Map a tracing target or module path to a Sentry `source` tag value.
///
/// Three buckets:
/// - `webview` — logs forwarded from the Angular webview
/// - `rust` — our own crate (`voiceboard::*`)
/// - `tauri` — the Tauri framework, plugins and other dependencies (`tauri`, `wry`, `cpal`, ...)
pub fn source_for_target(target: &str) -> &'static str {
    if target == "webview" {
        "webview"
    } else if is_own_crate_target(target) {
        "rust"
    } else {
        "tauri"
    }
}

/// True when the tracing target belongs to our own crates: the `voiceboard` binary
/// or the `voiceboard_lib` library (where almost all app code lives), as opposed to
/// the Tauri framework / plugins / dependencies.
fn is_own_crate_target(target: &str) -> bool {
    ["voiceboard", "voiceboard_lib"].iter().any(|name| {
        target == *name
            || target
                .strip_prefix(name)
                .is_some_and(|rest| rest.starts_with("::"))
    })
}

/// Pure environment resolution: a non-empty compile-time value wins, otherwise
/// fall back to the build profile (`development` in debug, `production` in release).
fn environment_from(compiled: Option<&str>, debug: bool) -> String {
    match compiled {
        Some(value) if !value.is_empty() => value.to_string(),
        _ if debug => "development".to_string(),
        _ => "production".to_string(),
    }
}

/// Resolve the Sentry `environment`. The running binary's channel is fixed at build
/// time, so a compile-time `SENTRY_ENVIRONMENT` (injected by CI as `beta`/`production`)
/// takes priority, with a build-profile fallback for local runs.
pub fn resolve_environment() -> String {
    environment_from(option_env!("SENTRY_ENVIRONMENT"), cfg!(debug_assertions))
}

/// Whether a log at `level` should be forwarded to Sentry Logs.
///
/// Baseline is WARN and above; enabling debug mode unlocks everything (DEBUG+).
pub fn should_send_log(level: LogLevel, debug_mode: bool) -> bool {
    debug_mode || matches!(level, LogLevel::Warn | LogLevel::Error | LogLevel::Fatal)
}

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
    std::env::var("SENTRY_DSN").ok().filter(|s| !s.is_empty())
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
                environment: Some(resolve_environment().into()),
                attach_stacktrace: true,
                send_default_pii: false,
                // Tag every issue with its source (rust / tauri / webview), derived from the
                // event logger which sentry-tracing sets to the tracing target.
                before_send: Some(Arc::new(|mut event| {
                    let source = event
                        .logger
                        .as_deref()
                        .map(source_for_target)
                        .unwrap_or("rust");
                    event.tags.insert("source".to_string(), source.to_string());
                    if event.user.is_none() {
                        if let Some(id) = INSTALL_ID.get() {
                            event.user = Some(sentry::protocol::User {
                                id: Some(id.clone()),
                                ..Default::default()
                            });
                        }
                    }
                    Some(event)
                })),
                before_send_log: Some(Arc::new(|mut log| {
                    // WARN+ by default; everything when debug mode is on.
                    if !should_send_log(log.level, DEBUG_MODE_ENABLED.load(Ordering::Relaxed)) {
                        return None;
                    }
                    // sentry-tracing does not copy the target onto logs, but it does set
                    // `code.module.name` (the module path) — derive the source from it.
                    let source = log
                        .attributes
                        .get("code.module.name")
                        .and_then(|attr| attr.0.as_str())
                        .map(source_for_target)
                        .unwrap_or("rust");
                    log.attributes.insert("source".to_string(), source.into());
                    if let Some(id) = INSTALL_ID.get() {
                        log.attributes
                            .insert("user.id".to_string(), id.clone().into());
                    }
                    Some(log)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_for_target_classifies_three_buckets() {
        assert_eq!(source_for_target("webview"), "webview");
        // Our own crates: the binary `voiceboard` and the library `voiceboard_lib`
        // (the lib is where app code actually logs from).
        assert_eq!(source_for_target("voiceboard"), "rust");
        assert_eq!(
            source_for_target("voiceboard::application::commands"),
            "rust"
        );
        assert_eq!(source_for_target("voiceboard_lib"), "rust");
        assert_eq!(
            source_for_target("voiceboard_lib::application::audio_engine"),
            "rust"
        );
        // Framework / plugins / dependencies
        assert_eq!(source_for_target("tauri::app"), "tauri");
        assert_eq!(source_for_target("tauri_plugin_updater::updater"), "tauri");
        assert_eq!(source_for_target("wry"), "tauri");
        assert_eq!(source_for_target("tao::event_loop"), "tauri");
        assert_eq!(source_for_target("cpal::host::coreaudio"), "tauri");
        assert_eq!(source_for_target("hyper::client"), "tauri");
        // A crate that merely starts with the same text is not ours.
        assert_eq!(source_for_target("voiceboardx::foo"), "tauri");
        assert_eq!(source_for_target(""), "tauri");
    }

    #[test]
    fn environment_from_prefers_compiled_value() {
        assert_eq!(environment_from(Some("beta"), true), "beta");
        assert_eq!(environment_from(Some("beta"), false), "beta");
        assert_eq!(environment_from(Some("production"), true), "production");
    }

    #[test]
    fn environment_from_falls_back_to_build_profile() {
        assert_eq!(environment_from(None, true), "development");
        assert_eq!(environment_from(None, false), "production");
        // An empty compile-time value is treated as unset
        assert_eq!(environment_from(Some(""), true), "development");
        assert_eq!(environment_from(Some(""), false), "production");
    }

    #[test]
    fn should_send_log_warn_and_above_by_default() {
        assert!(!should_send_log(LogLevel::Trace, false));
        assert!(!should_send_log(LogLevel::Debug, false));
        assert!(!should_send_log(LogLevel::Info, false));
        assert!(should_send_log(LogLevel::Warn, false));
        assert!(should_send_log(LogLevel::Error, false));
        assert!(should_send_log(LogLevel::Fatal, false));
    }

    #[test]
    fn should_send_log_everything_when_debug_mode() {
        for level in [
            LogLevel::Trace,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
            LogLevel::Fatal,
        ] {
            assert!(
                should_send_log(level, true),
                "debug mode should pass {level:?}"
            );
        }
    }
}
