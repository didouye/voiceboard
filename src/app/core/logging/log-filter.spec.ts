import { filterLogs, FilterableLog, LogFilter } from "./log-filter";

const LOGS: FilterableLog[] = [
  { level: "debug", message: "decoding sound", source: "rust" },
  { level: "info", message: "Mixer started", source: "webview" },
  { level: "warn", message: "device fallback", source: "tauri" },
  { level: "error", message: "update check FAILED", source: "rust" },
];

const base: LogFilter = { level: "all", source: "all", search: "" };

describe("filterLogs", () => {
  it("returns everything with the default filter", () => {
    expect(filterLogs(LOGS, base).length).toBe(4);
  });

  it("filters by minimum level (threshold, not exact)", () => {
    const warnPlus = filterLogs(LOGS, { ...base, level: "warn" });
    expect(warnPlus.map((l) => l.level)).toEqual(["warn", "error"]);
  });

  it("filters by source", () => {
    const rust = filterLogs(LOGS, { ...base, source: "rust" });
    expect(rust.map((l) => l.message)).toEqual([
      "decoding sound",
      "update check FAILED",
    ]);
  });

  it("searches the message case-insensitively", () => {
    expect(filterLogs(LOGS, { ...base, search: "failed" }).length).toBe(1);
    expect(filterLogs(LOGS, { ...base, search: "  MIXER " }).length).toBe(1);
  });

  it("combines level, source and search (AND)", () => {
    const result = filterLogs(LOGS, {
      level: "warn",
      source: "rust",
      search: "fail",
    });
    expect(result.map((l) => l.message)).toEqual(["update check FAILED"]);
  });

  it("excludes entries without a source when a specific source is selected", () => {
    const noSource: FilterableLog[] = [{ level: "info", message: "x" }];
    expect(filterLogs(noSource, { ...base, source: "rust" }).length).toBe(0);
    expect(filterLogs(noSource, base).length).toBe(1);
  });
});
