import { inject, Injectable } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { DebugConsoleService } from "./debug-console.service";

type ConsoleMethod = "log" | "info" | "warn" | "error" | "debug";

const LEVEL_MAP: Record<ConsoleMethod, "debug" | "info" | "warn" | "error"> = {
  log: "info",
  info: "info",
  warn: "warn",
  error: "error",
  debug: "debug",
};

/**
 * Captures every frontend `console.*` call so it is:
 * 1. still printed to the browser devtools (original behaviour preserved),
 * 2. buffered in the in-app debug console (tagged `webview`),
 * 3. forwarded to the Rust backend for the unified rotating log file.
 *
 * Sentry's own console integration continues to send these to Sentry Logs, so this
 * service deliberately does not re-send them remotely.
 */
@Injectable({ providedIn: "root" })
export class LoggerService {
  private readonly debugConsole = inject(DebugConsoleService);
  private installed = false;

  /** Patch the console once. Safe to call multiple times. */
  install(): void {
    if (this.installed) {
      return;
    }
    this.installed = true;

    (["log", "info", "warn", "error", "debug"] as ConsoleMethod[]).forEach(
      (method) => {
        const original = console[method].bind(console);
        console[method] = (...args: unknown[]) => {
          original(...args);

          const level = LEVEL_MAP[method];
          const message = args.map(formatArg).join(" ");

          // Buffer locally (never throws).
          this.debugConsole.record(level, message, "webview");

          // Forward to the unified log file (fire-and-forget; swallow errors to
          // avoid any recursion back into the patched console).
          invoke("log_from_webview", { level, message }).catch(() => {});
        };
      },
    );
  }
}

function formatArg(arg: unknown): string {
  if (typeof arg === "string") {
    return arg;
  }
  if (arg instanceof Error) {
    return arg.stack ?? `${arg.name}: ${arg.message}`;
  }
  try {
    return JSON.stringify(arg);
  } catch {
    return String(arg);
  }
}
