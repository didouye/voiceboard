import * as Sentry from "@sentry/angular";
import { consoleLoggingIntegration } from "@sentry/angular";
import { bootstrapApplication } from "@angular/platform-browser";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getVersion } from "@tauri-apps/api/app";
import { AppComponent } from "./app/app.component";
import { appConfig } from "./app/app.config";

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
    const [dsn, version] = await Promise.all([
      invoke<string | null>("get_sentry_dsn"),
      getVersion(),
    ]);

    if (dsn) {
      Sentry.init({
        dsn,
        release: version,
        environment: "production",
        enableLogs: true,
        integrations: [
        consoleLoggingIntegration({
          levels: ["log", "info", "warn", "error", "debug"],
        }),
      ],
        beforeSendLog(log) {
          return debugModeEnabled ? log : null;
        },
        tracesSampleRate: 0,
      });
      console.log(`Sentry initialized (release: ${version})`);
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
