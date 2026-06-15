import {
  ApplicationConfig,
  ErrorHandler,
  inject,
  provideAppInitializer,
  provideBrowserGlobalErrorListeners,
  provideZoneChangeDetection,
} from "@angular/core";
import { provideRouter } from "@angular/router";
import * as Sentry from "@sentry/angular";

import { routes } from "./app.routes";
import { LoggerService } from "./core/services/logger.service";

export const appConfig: ApplicationConfig = {
  providers: [
    provideBrowserGlobalErrorListeners(),
    provideZoneChangeDetection({ eventCoalescing: true }),
    provideRouter(routes),
    {
      provide: ErrorHandler,
      useValue: Sentry.createErrorHandler({
        showDialog: false,
      }),
    },
    // Patch console as early as possible (before the root component) so the earliest
    // bootstrap logs also reach the in-app console and the unified log file.
    provideAppInitializer(() => {
      inject(LoggerService).install();
    }),
  ],
};
