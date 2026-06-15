/** Minimal shape needed to filter a log entry (decoupled from LogEntry). */
export interface FilterableLog {
  level: "debug" | "info" | "warn" | "error";
  message: string;
  source?: "rust" | "tauri" | "webview";
}

export interface LogFilter {
  /** Minimum level to show ("all" = no level filter). */
  level: "all" | "debug" | "info" | "warn" | "error";
  /** Source to show ("all" = any source). */
  source: "all" | "rust" | "tauri" | "webview";
  /** Case-insensitive substring match on the message ("" = no search). */
  search: string;
}

const LEVEL_ORDER: Record<FilterableLog["level"], number> = {
  debug: 0,
  info: 1,
  warn: 2,
  error: 3,
};

/**
 * Filter log entries by minimum level, source, and a message substring.
 * The level filter is a threshold: "warn" shows warn and error.
 */
export function filterLogs<T extends FilterableLog>(
  logs: readonly T[],
  filter: LogFilter,
): T[] {
  const query = filter.search.trim().toLowerCase();
  const minLevel = filter.level === "all" ? -1 : LEVEL_ORDER[filter.level];

  return logs.filter(
    (log) =>
      (minLevel < 0 || LEVEL_ORDER[log.level] >= minLevel) &&
      (filter.source === "all" || log.source === filter.source) &&
      (query === "" || log.message.toLowerCase().includes(query)),
  );
}
