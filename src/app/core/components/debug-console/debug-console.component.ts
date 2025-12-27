import { Component, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DebugConsoleService } from '../../services/debug-console.service';

@Component({
  selector: 'app-debug-console',
  standalone: true,
  imports: [CommonModule],
  template: `
    @if (debugConsole.isEnabled) {
      <!-- Toggle Button -->
      <button
        class="debug-toggle"
        (click)="debugConsole.toggle()"
        title="Debug Console"
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <path d="M12 16v-4M12 8h.01"/>
        </svg>
      </button>

      <!-- Console Panel -->
      @if (debugConsole.isOpen()) {
        <div class="debug-panel">
          <div class="debug-header">
            <h3>Debug Console</h3>
            <div class="debug-actions">
              <button (click)="debugConsole.sendTestError()" title="Send test error to Sentry" class="test-sentry-btn">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
                  <line x1="12" y1="9" x2="12" y2="13"/>
                  <line x1="12" y1="17" x2="12.01" y2="17"/>
                </svg>
              </button>
              <button (click)="copyLogs()" title="Copy logs">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
                </svg>
              </button>
              <button (click)="debugConsole.clear()" title="Clear logs">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="3 6 5 6 21 6"/>
                  <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
                </svg>
              </button>
              <button (click)="debugConsole.close()" title="Close">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="18" y1="6" x2="6" y2="18"/>
                  <line x1="6" y1="6" x2="18" y2="18"/>
                </svg>
              </button>
            </div>
          </div>
          <div class="debug-content">
            @for (log of debugConsole.logs(); track log.timestamp.getTime()) {
              <div class="log-entry" [class]="'log-' + log.level">
                <span class="log-time">{{ formatTime(log.timestamp) }}</span>
                <span class="log-level">{{ log.level.toUpperCase() }}</span>
                <span class="log-message">{{ log.message }}</span>
                @if (log.context) {
                  <span class="log-context">{{ formatContext(log.context) }}</span>
                }
              </div>
            } @empty {
              <div class="log-empty">No logs yet</div>
            }
          </div>
        </div>
      }
    }
  `,
  styles: [`
    .debug-toggle {
      position: fixed;
      bottom: 16px;
      right: 16px;
      width: 40px;
      height: 40px;
      border-radius: 50%;
      background: rgba(59, 130, 246, 0.9);
      border: none;
      color: white;
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: 1000;
      transition: transform 0.2s, background 0.2s;

      &:hover {
        transform: scale(1.1);
        background: rgba(59, 130, 246, 1);
      }
    }

    .debug-panel {
      position: fixed;
      bottom: 70px;
      right: 16px;
      width: 500px;
      max-width: calc(100vw - 32px);
      max-height: 400px;
      background: rgba(17, 24, 39, 0.98);
      border: 1px solid rgba(75, 85, 99, 0.5);
      border-radius: 8px;
      z-index: 999;
      display: flex;
      flex-direction: column;
      box-shadow: 0 10px 40px rgba(0, 0, 0, 0.5);
    }

    .debug-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 12px 16px;
      border-bottom: 1px solid rgba(75, 85, 99, 0.5);

      h3 {
        margin: 0;
        font-size: 14px;
        font-weight: 600;
        color: #e5e7eb;
      }
    }

    .debug-actions {
      display: flex;
      gap: 8px;

      button {
        background: none;
        border: none;
        color: #9ca3af;
        cursor: pointer;
        padding: 4px;
        border-radius: 4px;
        display: flex;
        align-items: center;
        justify-content: center;

        &:hover {
          background: rgba(75, 85, 99, 0.5);
          color: #e5e7eb;
        }
      }

      .test-sentry-btn {
        color: #f59e0b;

        &:hover {
          background: rgba(245, 158, 11, 0.2);
          color: #fbbf24;
        }
      }
    }

    .debug-content {
      flex: 1;
      overflow-y: auto;
      padding: 8px;
      font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
      font-size: 12px;
    }

    .log-entry {
      display: flex;
      gap: 8px;
      padding: 4px 8px;
      border-radius: 4px;
      line-height: 1.4;

      &.log-error {
        background: rgba(239, 68, 68, 0.1);
        color: #fca5a5;
      }

      &.log-warn {
        background: rgba(245, 158, 11, 0.1);
        color: #fcd34d;
      }

      &.log-info {
        color: #93c5fd;
      }

      &.log-debug {
        color: #9ca3af;
      }
    }

    .log-time {
      color: #6b7280;
      flex-shrink: 0;
    }

    .log-level {
      flex-shrink: 0;
      width: 50px;
      font-weight: 600;
    }

    .log-message {
      flex: 1;
      word-break: break-word;
    }

    .log-context {
      color: #6b7280;
      font-size: 11px;
    }

    .log-empty {
      color: #6b7280;
      text-align: center;
      padding: 20px;
    }
  `]
})
export class DebugConsoleComponent {
  debugConsole = inject(DebugConsoleService);

  formatTime(date: Date): string {
    return date.toLocaleTimeString('en-US', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      fractionalSecondDigits: 3
    });
  }

  formatContext(context: Record<string, unknown>): string {
    return JSON.stringify(context);
  }

  copyLogs() {
    const logs = this.debugConsole.exportLogs();
    navigator.clipboard.writeText(logs);
  }
}
