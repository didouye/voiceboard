import { shouldSendLog } from "./log-gating";

describe("shouldSendLog", () => {
  it("sends WARN and above by default (debug mode off)", () => {
    expect(shouldSendLog("trace", false)).toBe(false);
    expect(shouldSendLog("debug", false)).toBe(false);
    expect(shouldSendLog("info", false)).toBe(false);
    expect(shouldSendLog("warn", false)).toBe(true);
    expect(shouldSendLog("error", false)).toBe(true);
    expect(shouldSendLog("fatal", false)).toBe(true);
  });

  it("sends everything when debug mode is on", () => {
    for (const level of ["trace", "debug", "info", "warn", "error", "fatal"]) {
      expect(shouldSendLog(level, true)).toBe(true);
    }
  });

  it("treats unknown levels conservatively (dropped unless debug mode)", () => {
    expect(shouldSendLog("verbose", false)).toBe(false);
    expect(shouldSendLog("verbose", true)).toBe(true);
  });
});
