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

  // Debounced forwarding to the backend to cut IPC chatter on chatty logging.
  private readonly pending: { level: string; message: string }[] = [];
  private flushTimer: ReturnType<typeof setTimeout> | null = null;
  private readonly FLUSH_MS = 300;
  private readonly MAX_BATCH = 100;

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

          // Forward to the unified log file, batched.
          this.queueForward(level, message);
        };
      },
    );
  }

  private queueForward(level: string, message: string): void {
    this.pending.push({ level, message });
    if (this.pending.length >= this.MAX_BATCH) {
      this.flush();
      return;
    }
    this.flushTimer ??= setTimeout(() => this.flush(), this.FLUSH_MS);
  }

  private flush(): void {
    if (this.flushTimer !== null) {
      clearTimeout(this.flushTimer);
      this.flushTimer = null;
    }
    if (this.pending.length === 0) {
      return;
    }
    const entries = this.pending.splice(0, this.pending.length);
    // Fire-and-forget; swallow errors to avoid recursion into the patched console.
    invoke("log_batch_from_webview", { entries }).catch(() => {});
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
