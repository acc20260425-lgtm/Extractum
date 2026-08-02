import { EventEmitter } from "node:events";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it, vi } from "vitest";
import { runObservedCommand } from "./run-observation.mjs";
import { appendTimingRow } from "./timing-log.mjs";

const commit = "c".repeat(40);
const temporaryRoots: string[] = [];

afterEach(async () => {
  await Promise.all(temporaryRoots.splice(0).map((root) => rm(root, { recursive: true, force: true })));
});

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
    }, { repoRoot: "repo" });
    expect(Object.keys(recordTimingBestEffort.mock.calls[0][0])).toEqual([
      "command", "startedAt", "duration", "exitCode", "commit",
    ]);
  });

  it("persists the timing row below the explicit repository root with that root's commit", async () => {
    const repoRoot = await mkdtemp(path.join(tmpdir(), "extractum-observer-root-"));
    temporaryRoots.push(repoRoot);
    expect(path.resolve(repoRoot)).not.toBe(path.resolve(process.cwd()));
    const readHeadCommit = vi.fn((root: string) => {
      expect(root).toBe(repoRoot);
      return commit;
    });
    const recordTimingBestEffort = vi.fn(async (row, options) => {
      if (!options?.repoRoot) return false;
      return appendTimingRow(row, options);
    });

    const result = await runObservedCommand({
      command: "node",
      args: ["script.mjs"],
      cwd: repoRoot,
      repoRoot,
      dependencies: dependencies({ readHeadCommit, recordTimingBestEffort }),
    });

    const persisted = JSON.parse(await readFile(
      path.join(repoRoot, "artifacts", "testing", "timings.jsonl"),
      "utf8",
    ));
    expect(result.commit).toBe(commit);
    expect(persisted).toMatchObject({ command: "node script.mjs", commit });
    expect(readHeadCommit).toHaveBeenCalledWith(repoRoot);
    expect(recordTimingBestEffort).toHaveBeenCalledWith(expect.any(Object), { repoRoot });
  });

  it("rejects an empty explicit repository root", async () => {
    await expect(runObservedCommand({
      command: "node",
      cwd: "repo",
      repoRoot: "",
      dependencies: dependencies(),
    })).rejects.toThrowError("repoRoot");
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

  it("requests cancellation and waits for the child close event before resolving", async () => {
    const controller = new AbortController();
    const child = new EventEmitter() as EventEmitter & { pid: number; kill: ReturnType<typeof vi.fn> };
    child.pid = 123;
    const order: string[] = [];
    let fallback: ReturnType<typeof setTimeout>;
    child.kill = vi.fn((signal: NodeJS.Signals) => {
      order.push(`kill:${signal}`);
      return true;
    });
    fallback = setTimeout(() => child.emit("close", 0, null), 100);

    const observation = runObservedCommand({
      command: "node",
      cwd: "repo",
      signal: controller.signal,
      dependencies: dependencies({ spawn: vi.fn(() => child) }),
    });
    void observation.then(() => order.push("resolved"));
    controller.abort("SIGINT");
    await Promise.resolve();

    expect(order).toEqual(["kill:SIGINT"]);
    clearTimeout(fallback);
    order.push("close");
    child.emit("close", null, "SIGINT");

    await expect(observation).resolves.toMatchObject({
      exitCode: 130,
      termination: "signal",
      signal: "SIGINT",
    });
    expect(child.kill).toHaveBeenCalledWith("SIGINT");
    expect(order).toEqual(["kill:SIGINT", "close", "resolved"]);
  });

  it("aborts a real child and reports cancellation only after it terminates", async () => {
    const controller = new AbortController();
    const observation = runObservedCommand({
      command: process.execPath,
      args: ["-e", "console.log(process.pid); setTimeout(() => process.exit(0), 1000)"],
      cwd: process.cwd(),
      capture: true,
      signal: controller.signal,
      dependencies: dependencies({ spawn: undefined }),
    });
    setTimeout(() => controller.abort("SIGTERM"), 150);

    const result = await observation;
    expect(result).toMatchObject({
      exitCode: 130,
      termination: "signal",
      signal: "SIGTERM",
    });
    const childPid = Number(result.stdout.trim());
    expect(Number.isInteger(childPid)).toBe(true);
    expect(() => process.kill(childPid, 0)).toThrow();
  }, 5_000);
});
