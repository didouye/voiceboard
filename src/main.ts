import * as Sentry from "@sentry/angular";
import { consoleLoggingIntegration } from "@sentry/angular";
import { bootstrapApplication } from "@angular/platform-browser";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getVersion } from "@tauri-apps/api/app";
import { AppComponent } from "./app/app.component";
import { appConfig } from "./app/app.config";
import { shouldSendLog } from "./app/core/logging/log-gating";

// Debug mode state — read by Sentry beforeSendLog callback
let debugModeEnabled = false;

function initDebugModeTracking(): void {
  invoke<boolean>("get_debug_mode")
    .then((enabled) => {
      debugModeEnabled = enabled;
    })
    .catch(() => {});

  listen<boolean>("debug-mode-changed", (event) => {
    debugModeEnabled = event.payload;
  }).catch(() => {});
}

// Initialize Sentry with DSN and version from backend
async function initSentry(): Promise<void> {
  try {
    const [dsn, version, environment, installId] = await Promise.all([
      invoke<string | null>("get_sentry_dsn"),
      getVersion(),
      invoke<string>("get_app_environment").catch(() => "production"),
      invoke<string | null>("get_install_id").catch(() => null),
    ]);

    if (dsn) {
      Sentry.init({
        dsn,
        release: version,
        // Channel-aware environment resolved by the Rust backend (development / beta /
        // production), so webview and Rust events share the exact same value.
        environment,
        enableLogs: true,
        // Tag every webview event so Sentry can be filtered by source, and group all of a
        // machine's events under the same stable (non-PII) install id as the Rust SDK.
        initialScope: {
          tags: { source: "webview" },
          user: installId ? { id: installId } : undefined,
        },
        integrations: [
          consoleLoggingIntegration({
            levels: ["log", "info", "warn", "error", "debug"],
          }),
        ],
        beforeSendLog(log) {
          // Tag with the source as a log attribute (scope tags don't propagate to
          // Sentry Logs), mirroring the Rust side so logs are filterable by source.
          log.attributes = { ...log.attributes, source: "webview" };
          // WARN+ by default; everything when debug mode is on.
          return shouldSendLog(log.level, debugModeEnabled) ? log : null;
        },
        tracesSampleRate: 0,
      });
      console.log(
        `Sentry initialized (release: ${version}, environment: ${environment})`,
      );
    }
  } catch (e) {
    console.warn("Failed to initialize Sentry:", e);
  }
}

// Initialize debug mode tracking, Sentry, then bootstrap Angular
initDebugModeTracking();
initSentry().then(() => {
  bootstrapApplication(AppComponent, appConfig).catch((err) => {
    console.error(err);
    Sentry.captureException(err);
  });
});
