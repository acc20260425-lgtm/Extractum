import { EventEmitter } from "node:events";
import { describe, expect, it, vi } from "vitest";
import { runObservedCommand } from "./run-observation.mjs";

const commit = "c".repeat(40);

function completedChild(exitCode: number | null, signal: NodeJS.Signals | null) {
  const child = new EventEmitter();
  queueMicrotask(() => child.emit("close", exitCode, signal));
  return child;
}

function dependencies(overrides = {}) {
  return {
    spawn: vi.fn(() => completedChild(0, null)),
    nowDate: vi.fn(() => new Date("2026-08-02T10:11:12.123Z")),
    nowMonotonic: vi.fn(() => 100),
    readHeadCommit: vi.fn(() => commit),
    createTimingRow: vi.fn((row) => row),
    recordTimingBestEffort: vi.fn(async () => true),
    warn: vi.fn(),
    ...overrides,
  };
}

describe("runObservedCommand", () => {
  it("reads startedAt immediately before spawning and measures with monotonic time", async () => {
    const calls: string[] = [];
    const child = completedChild(0, null);
    let monotonicCall = 0;
    const env = new Proxy({ EXTRACTUM_OBSERVATION: "1" }, {
      ownKeys(target) {
        calls.push("env");
        return Reflect.ownKeys(target);
      },
    });
    const value = dependencies({
      readHeadCommit: vi.fn(() => { calls.push("commit"); return commit; }),
      nowDate: vi.fn(() => { calls.push("date"); return new Date("2026-08-02T10:11:12.123Z"); }),
      nowMonotonic: vi.fn(() => { calls.push("monotonic"); return [100, 126.4][monotonicCall++]; }),
      spawn: vi.fn(() => { calls.push("spawn"); return child; }),
    });

    const result = await runObservedCommand({
      command: "node", args: ["script.mjs"], cwd: "repo", env, dependencies: value,
    });

    expect(calls).toEqual(["commit", "env", "date", "monotonic", "spawn", "monotonic"]);
    expect(result).toMatchObject({ startedAt: "2026-08-02T10:11:12.123Z", duration: 26, exitCode: 0 });
  });

  it("returns and persists a normal exit code", async () => {
    const recordTimingBestEffort = vi.fn(async () => true);
    const value = dependencies({
      spawn: vi.fn(() => completedChild(9, null)),
      nowMonotonic: vi.fn().mockReturnValueOnce(5).mockReturnValueOnce(18),
      recordTimingBestEffort,
    });

    const result = await runObservedCommand({ command: "node", args: ["script.mjs"], cwd: "repo", dependencies: value });

    expect(result).toMatchObject({ exitCode: 9, termination: "exit", signal: null });
    expect(recordTimingBestEffort).toHaveBeenCalledWith({
      command: "node script.mjs",
      startedAt: "2026-08-02T10:11:12.123Z",
      duration: 13,
      exitCode: 9,
      commit,
    });
    expect(Object.keys(recordTimingBestEffort.mock.calls[0][0])).toEqual([
      "command", "startedAt", "duration", "exitCode", "commit",
    ]);
  });

  it("normalizes SIGINT and other signals to process exit codes", async () => {
    const interrupted = await runObservedCommand({
      command: "node", args: [], cwd: "repo",
      dependencies: dependencies({ spawn: vi.fn(() => completedChild(null, "SIGINT")) }),
    });
    const terminated = await runObservedCommand({
      command: "node", args: [], cwd: "repo",
      dependencies: dependencies({ spawn: vi.fn(() => completedChild(null, "SIGTERM")) }),
    });

    expect(interrupted).toMatchObject({ exitCode: 130, signal: "SIGINT", termination: "signal" });
    expect(terminated).toMatchObject({ exitCode: 3, signal: "SIGTERM", termination: "signal" });
  });

  it("warns when timing persistence rejects without replacing the child result", async () => {
    const warning = new Error("disk unavailable");
    const value = dependencies({
      spawn: vi.fn(() => completedChild(7, null)),
      recordTimingBestEffort: vi.fn().mockRejectedValue(warning),
    });

    const result = await runObservedCommand({ command: "node", args: [], cwd: "repo", dependencies: value });

    expect(result).toMatchObject({ exitCode: 7, termination: "exit" });
    expect(value.warn).toHaveBeenCalledWith("Timing log warning: disk unavailable");
  });
});
