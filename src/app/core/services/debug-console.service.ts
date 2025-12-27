import { Injectable, signal } from '@angular/core';
import { listen } from '@tauri-apps/api/event';

export interface LogEntry {
  timestamp: Date;
  level: 'debug' | 'info' | 'warn' | 'error';
  message: string;
  context?: Record<string, unknown>;
}

@Injectable({ providedIn: 'root' })
export class DebugConsoleService {
  private readonly MAX_LOGS = 500;
  private readonly _logs = signal<LogEntry[]>([]);
  private readonly _isOpen = signal(false);

  readonly logs = this._logs.asReadonly();
  readonly isOpen = this._isOpen.asReadonly();

  constructor() {
    this.setupEventListeners();
    this.log('info', 'Debug console initialized');
  }

  private async setupEventListeners() {
    // Listen for backend log events
    try {
      await listen<{ level: string; message: string; fields?: Record<string, unknown> }>('log-event', (event) => {
        const level = this.parseLevel(event.payload.level);
        this.addLog({
          timestamp: new Date(),
          level,
          message: event.payload.message,
          context: event.payload.fields,
        });
      });
    } catch {
      // Event listener not available
    }
  }

  private parseLevel(level: string): LogEntry['level'] {
    const normalized = level.toLowerCase();
    if (normalized.includes('error')) return 'error';
    if (normalized.includes('warn')) return 'warn';
    if (normalized.includes('debug')) return 'debug';
    return 'info';
  }

  log(level: LogEntry['level'], message: string, context?: Record<string, unknown>) {
    this.addLog({
      timestamp: new Date(),
      level,
      message,
      context,
    });
  }

  private addLog(entry: LogEntry) {
    this._logs.update(logs => {
      const newLogs = [...logs, entry];
      if (newLogs.length > this.MAX_LOGS) {
        return newLogs.slice(-this.MAX_LOGS);
      }
      return newLogs;
    });
  }

  toggle() {
    this._isOpen.update(open => !open);
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

  getLogsByLevel(level: LogEntry['level']): LogEntry[] {
    return this._logs().filter(log => log.level === level);
  }

  exportLogs(): string {
    return this._logs()
      .map(log => `[${log.timestamp.toISOString()}] [${log.level.toUpperCase()}] ${log.message}${log.context ? ' ' + JSON.stringify(log.context) : ''}`)
      .join('\n');
  }
}
