//! Bridge that forwards every tracing event to the webview so the in-app debug
//! console can display a unified stream (rust / tauri / webview).
//!
//! A custom tracing `Layer` builds a [`LogPayload`] for each event, keeps it in a
//! bounded ring buffer (for seeding the console with recent history on open) and
//! hands it to a dedicated forwarder thread which emits the `app-log` Tauri event.
//! Emitting from a separate thread keeps it off the tracing dispatch path, avoiding
//! re-entrancy or lock issues with `app.emit`.
//!
//! Webview-originated events (target `webview`, forwarded from the frontend) are
//! skipped here: the frontend already shows them in the console directly, so echoing
//! them back would duplicate. They still reach the rotating log file via the fmt layer.

use serde::Serialize;
use std::collections::VecDeque;
use std::sync::mpsc::{self, Sender};
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Emitter};
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, Layer};

/// Tauri event name carrying a single forwarded log line to the webview.
pub const LOG_EVENT: &str = "app-log";

/// Maximum number of recent log lines retained for console seeding.
const RING_CAPACITY: usize = 1000;

static RING: Mutex<VecDeque<LogPayload>> = Mutex::new(VecDeque::new());
static LOG_TX: OnceLock<Sender<LogPayload>> = OnceLock::new();

/// One forwarded log line, serialized to the webview and returned by `get_recent_logs`.
#[derive(Clone, Serialize)]
pub struct LogPayload {
    /// Milliseconds since the Unix epoch.
    pub timestamp: u64,
    /// Lower-case level: `error` | `warn` | `info` | `debug` | `trace`.
    pub level: String,
    /// Origin bucket: `rust` | `tauri` | `webview`.
    pub source: String,
    /// The tracing target (module path / crate).
    pub target: String,
    /// The log message.
    pub message: String,
    /// Structured fields other than the message.
    #[serde(skip_serializing_if = "serde_json::Map::is_empty")]
    pub fields: serde_json::Map<String, serde_json::Value>,
}

/// The tracing layer that captures events for the in-app console.
pub struct WebviewLayer;

impl<S: Subscriber> Layer<S> for WebviewLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let meta = event.metadata();
        // Webview logs are already shown by the frontend; don't echo them back.
        if meta.target() == "webview" {
            return;
        }

        let mut visitor = FieldVisitor::default();
        event.record(&mut visitor);

        let payload = LogPayload {
            timestamp: now_millis(),
            level: level_str(*meta.level()).to_string(),
            source: super::sentry::source_for_target(meta.target()).to_string(),
            target: meta.target().to_string(),
            message: visitor.message,
            fields: visitor.fields,
        };

        push_ring(payload.clone());

        if let Some(tx) = LOG_TX.get() {
            let _ = tx.send(payload);
        }
    }
}

/// Start forwarding buffered and live log lines to the webview via `app-log`.
/// Called once from the Tauri `setup` hook, when the `AppHandle` is available.
pub fn start_forwarding(app: AppHandle) {
    let (tx, rx) = mpsc::channel::<LogPayload>();
    if LOG_TX.set(tx).is_err() {
        return; // already started
    }
    std::thread::spawn(move || {
        while let Ok(payload) = rx.recv() {
            let _ = app.emit(LOG_EVENT, &payload);
        }
    });
}

/// Snapshot of the most recent log lines, oldest first. Used to seed the console.
pub fn recent_logs() -> Vec<LogPayload> {
    RING.lock()
        .map(|ring| ring.iter().cloned().collect())
        .unwrap_or_default()
}

fn push_ring(payload: LogPayload) {
    if let Ok(mut ring) = RING.lock() {
        if ring.len() >= RING_CAPACITY {
            ring.pop_front();
        }
        ring.push_back(payload);
    }
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn level_str(level: Level) -> &'static str {
    match level {
        Level::ERROR => "error",
        Level::WARN => "warn",
        Level::INFO => "info",
        Level::DEBUG => "debug",
        Level::TRACE => "trace",
    }
}

/// Collects the `message` field and any other structured fields from a tracing event.
#[derive(Default)]
struct FieldVisitor {
    message: String,
    fields: serde_json::Map<String, serde_json::Value>,
}

impl FieldVisitor {
    fn put(&mut self, field: &Field, value: serde_json::Value) {
        if field.name() == "message" {
            if let serde_json::Value::String(s) = value {
                self.message = s;
            } else {
                self.message = value.to_string();
            }
        } else {
            self.fields.insert(field.name().to_string(), value);
        }
    }
}

impl Visit for FieldVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.put(field, serde_json::Value::String(format!("{value:?}")));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.put(field, serde_json::Value::String(value.to_string()));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.put(field, serde_json::Value::from(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.put(field, serde_json::Value::from(value));
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.put(field, serde_json::Value::from(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.put(field, serde_json::Value::from(value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_str_maps_all_levels() {
        assert_eq!(level_str(Level::ERROR), "error");
        assert_eq!(level_str(Level::WARN), "warn");
        assert_eq!(level_str(Level::INFO), "info");
        assert_eq!(level_str(Level::DEBUG), "debug");
        assert_eq!(level_str(Level::TRACE), "trace");
    }

    #[test]
    fn ring_buffer_is_bounded_and_ordered() {
        // Note: shares the global RING; assert on relative behavior, not absolute size.
        let before = recent_logs().len();
        for i in 0..5 {
            push_ring(LogPayload {
                timestamp: i,
                level: "info".into(),
                source: "rust".into(),
                target: "voiceboard::test".into(),
                message: format!("msg {i}"),
                fields: serde_json::Map::new(),
            });
        }
        let after = recent_logs();
        assert!(after.len() >= before);
        assert!(after.len() <= RING_CAPACITY);
        assert_eq!(after.last().unwrap().message, "msg 4");
    }
}
