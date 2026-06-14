//! Infrastructure layer - Cross-cutting concerns
//!
//! This layer contains configuration, logging, and other
//! infrastructure-related code.

mod log_bridge;
mod logging;
mod sentry;

pub use log_bridge::{recent_logs, start_forwarding, LogPayload, WebviewLayer};
pub use logging::*;
pub use sentry::{init_sentry, resolve_environment, set_install_id, DEBUG_MODE_ENABLED};
