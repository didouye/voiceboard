import { inject, Injectable, signal } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import * as Sentry from "@sentry/angular";
import { DemoService } from "./demo.service";

export type LogSource = "rust" | "tauri" | "webview";

export interface LogEntry {
  timestamp: Date;
  level: "debug" | "info" | "warn" | "error";
  message: string;
  context?: Record<string, unknown>;
  source?: LogSource;
}

interface BackendLogPayload {
  timestamp: number;
  level: string;
  source: string;
  target: string;
  message: string;
  fields?: Record<string, unknown>;
}

@Injectable({ providedIn: "root" })
export class DebugConsoleService {
  private readonly MAX_LOGS = 2000;
  private readonly _logs = signal<LogEntry[]>([]);
  private readonly _isOpen = signal(false);
  private readonly _isEnabled = signal(false);
  private demoService = inject(DemoService);

  readonly logs = this._logs.asReadonly();
  readonly isOpen = this._isOpen.asReadonly();
  readonly isEnabled = this._isEnabled.asReadonly();

  constructor() {
    this.initialize();
  }

  private async initialize() {
    // Always capture logs, independent of debug mode. Debug mode only controls
    // whether the console toggle/panel is shown and the Sentry log volume.
    await this.setupEventListeners();
    await this.seedHistory();

    if (this.demoService.isDemoMode) {
      this._isEnabled.set(true);
      return;
    }

    // Get initial debug mode (controls UI visibility only)
    try {
      this._isEnabled.set(await invoke<boolean>("get_debug_mode"));
    } catch {
      // Not running under Tauri (e.g. plain web dev) — leave the panel hidden.
    }

    // Track debug mode changes from the menu toggle for UI visibility
    try {
      await listen<boolean>("debug-mode-changed", (event) => {
        this._isEnabled.set(event.payload);
      });
    } catch {
      // Event listener not available
    }
  }

  private async setupEventListeners() {
    // Unified backend log stream (rust + tauri framework), forwarded from tracing.
    // Audio engine / audio-debug logs now arrive through this same stream, so no
    // separate listeners are needed (avoids duplicate entries).
    try {
      await listen<BackendLogPayload>("app-log", (event) => {
        this.addBackendLog(event.payload);
      });
    } catch {
      // Event listener not available
    }
  }

  /** Seed the console with the backend's recent buffered logs (history before connect). */
  private async seedHistory() {
    try {
      const logs = await invoke<BackendLogPayload[]>("get_recent_logs");
      for (const payload of logs) {
        this.addBackendLog(payload);
      }
    } catch {
      // Not running under Tauri or command unavailable
    }
  }

  private addBackendLog(payload: BackendLogPayload) {
    this.addLog({
      timestamp: new Date(payload.timestamp || Date.now()),
      level: this.parseLevel(payload.level),
      message: payload.message,
      context: payload.fields,
      source: this.parseSource(payload.source),
    });
  }

  private parseLevel(level: string): LogEntry["level"] {
    const normalized = level.toLowerCase();
    if (normalized.includes("error")) return "error";
    if (normalized.includes("warn")) return "warn";
    if (normalized.includes("debug")) return "debug";
    return "info";
  }

  private parseSource(source: string): LogSource {
    if (source === "tauri") return "tauri";
    if (source === "webview") return "webview";
    return "rust";
  }

  /** Programmatic log from frontend code (always buffered, source = webview). */
  log(
    level: LogEntry["level"],
    message: string,
    context?: Record<string, unknown>,
  ) {
    this.record(level, message, "webview", context);
  }

  /** Record a log entry from any source. Always buffered, regardless of debug mode. */
  record(
    level: LogEntry["level"],
    message: string,
    source: LogSource,
    context?: Record<string, unknown>,
  ) {
    this.addLog({ timestamp: new Date(), level, message, context, source });
  }

  private addLog(entry: LogEntry) {
    this._logs.update((logs) => {
      const newLogs = [...logs, entry];
      if (newLogs.length > this.MAX_LOGS) {
        return newLogs.slice(-this.MAX_LOGS);
      }
      return newLogs;
    });

    // Add a Sentry breadcrumb for error context. Webview console logs are already
    // captured by Sentry's console integration, so only breadcrumb backend logs here.
    if (entry.source !== "webview") {
      Sentry.addBreadcrumb({
        category: "log",
        message: entry.message,
        level: this.toSentryLevel(entry.level),
        data: entry.context,
      });
    }
  }

  private toSentryLevel(level: LogEntry["level"]): Sentry.SeverityLevel {
    switch (level) {
      case "error":
        return "error";
      case "warn":
        return "warning";
      case "debug":
        return "debug";
      default:
        return "info";
    }
  }

  toggle() {
    this._isOpen.update((open) => !open);
  }

  open() {
    this._isOpen.set(true);
  }

  close() {
    this._isOpen.set(false);
  }

  clear() {
    this._logs.set([]);
  }

  getLogsByLevel(level: LogEntry["level"]): LogEntry[] {
    return this._logs().filter((log) => log.level === level);
  }

  exportLogs(): string {
    return this._logs()
      .map(
        (log) =>
          `[${log.timestamp.toISOString()}] [${log.level.toUpperCase()}] ${log.message}${log.context ? " " + JSON.stringify(log.context) : ""}`,
      )
      .join("\n");
  }

  /**
   * Open the local log directory in the OS file manager.
   */
  async openLogDir(): Promise<void> {
    try {
      await invoke("open_log_dir");
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      this.log("error", `Failed to open log folder: ${message}`);
    }
  }

  /**
   * Send a test error to Sentry to verify integration
   */
  sendTestError(): void {
    const testError = new Error("Sentry Test Error - Debug Console");
    this.log("warn", "Sending test error to Sentry...");

    try {
      Sentry.captureException(testError);
      this.log("info", "Test error sent to Sentry successfully");
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : String(e);
      this.log("error", `Failed to send test error to Sentry: ${errorMessage}`);
    }
  }
}
