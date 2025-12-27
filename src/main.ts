import * as Sentry from "@sentry/angular";
import { bootstrapApplication } from "@angular/platform-browser";
import { AppComponent } from "./app/app.component";
import { appConfig } from "./app/app.config";

// Initialize Sentry before Angular (if DSN is configured)
// DSN can be set via build-time replacement or global variable
declare const __SENTRY_DSN__: string | undefined;
const sentryDsn = typeof __SENTRY_DSN__ !== 'undefined' ? __SENTRY_DSN__ : undefined;

if (sentryDsn) {
  Sentry.init({
    dsn: sentryDsn,
    environment: 'production',
    integrations: [],
    tracesSampleRate: 0,
  });
}

bootstrapApplication(AppComponent, appConfig).catch((err) => {
  console.error(err);
  if (sentryDsn) {
    Sentry.captureException(err);
  }
});
