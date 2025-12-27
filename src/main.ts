import * as Sentry from "@sentry/angular";
import { bootstrapApplication } from "@angular/platform-browser";
import { invoke } from "@tauri-apps/api/core";
import { AppComponent } from "./app/app.component";
import { appConfig } from "./app/app.config";

// Initialize Sentry with DSN from backend
async function initSentry(): Promise<void> {
  try {
    const dsn = await invoke<string | null>('get_sentry_dsn');
    if (dsn) {
      Sentry.init({
        dsn,
        environment: 'production',
        integrations: [],
        tracesSampleRate: 0,
      });
      console.log('Sentry initialized');
    }
  } catch (e) {
    console.warn('Failed to initialize Sentry:', e);
  }
}

// Initialize Sentry then bootstrap Angular
initSentry().then(() => {
  bootstrapApplication(AppComponent, appConfig).catch((err) => {
    console.error(err);
    Sentry.captureException(err);
  });
});
