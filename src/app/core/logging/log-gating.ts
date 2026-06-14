/**
 * Sentry Logs gating shared by the webview SDK.
 *
 * Baseline is WARN and above; enabling debug mode unlocks everything (DEBUG+).
 * This mirrors the Rust-side `should_send_log` so both sources behave identically.
 */

/** Sentry log severity levels, lowest to highest. */
export type SentryLogLevel =
  | "trace"
  | "debug"
  | "info"
  | "warn"
  | "error"
  | "fatal";

const ALWAYS_SENT: ReadonlySet<string> = new Set(["warn", "error", "fatal"]);

/**
 * Whether a log at `level` should be forwarded to Sentry Logs.
 *
 * @param level Sentry log level (e.g. "info", "warn").
 * @param debugMode Whether the in-app debug mode is enabled.
 */
export function shouldSendLog(level: string, debugMode: boolean): boolean {
  return debugMode || ALWAYS_SENT.has(level);
}
