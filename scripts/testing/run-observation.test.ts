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

function processIsAlive(pid: number) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ESRCH") return false;
    throw error;
  }
}

async function waitForPidFile(file: string) {
  const deadline = Date.now() + 5_000;
  while (Date.now() < deadline) {
    try {
      return (await readFile(file, "utf8")).trim().split(",").map(Number);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
      await new Promise((resolve) => setTimeout(resolve, 25));
    }
  }
  throw new Error(`PID file was not created: ${file}`);
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

  it("uses explicit bounded Windows tree termination and waits for both settlements", async () => {
    const controller = new AbortController();
    const child = new EventEmitter() as EventEmitter & { pid: number; kill: ReturnType<typeof vi.fn> };
    child.pid = 123;
    child.kill = vi.fn((signal: NodeJS.Signals) => {
      queueMicrotask(() => child.emit("close", null, signal));
      return true;
    });
    const helper = new EventEmitter();
    const order: string[] = [];
    const spawnTerminationHelper = vi.fn((command, args, options) => {
      expect(command).toBe("C:\\Windows\\System32\\taskkill.exe");
      expect(args).toEqual(["/PID", "123", "/T", "/F"]);
      expect(options).toMatchObject({ shell: false, windowsHide: true });
      queueMicrotask(() => {
        order.push("helper close");
        helper.emit("close", 0, null);
        order.push("child close");
        child.emit("close", null, "SIGTERM");
      });
      return helper;
    });

    const observation = runObservedCommand({
      command: "node",
      cwd: "repo",
      signal: controller.signal,
      dependencies: dependencies({
        spawn: vi.fn(() => child),
        spawnTerminationHelper,
        platform: "win32",
        systemRoot: "C:\\Windows",
      }),
    });
    void observation.then(() => order.push("resolved"));
    controller.abort("SIGTERM");
    await Promise.resolve();

    expect(order).toEqual(["helper close", "child close"]);

    await expect(observation).resolves.toMatchObject({
      exitCode: 130,
      termination: "signal",
      signal: "SIGTERM",
      cancellationConfirmed: true,
    });
    expect(child.kill).not.toHaveBeenCalled();
    expect(order).toEqual(["helper close", "child close", "resolved"]);
  });

  it("fails closed when tree termination is unconfirmed after the observed child closes", async () => {
    const controller = new AbortController();
    const child = new EventEmitter() as EventEmitter & { pid: number; kill: ReturnType<typeof vi.fn> };
    child.pid = 456;
    child.kill = vi.fn(() => {
      queueMicrotask(() => child.emit("close", null, "SIGTERM"));
      return true;
    });
    const terminateProcessTree = vi.fn(async ({ child: observedChild }) => {
      observedChild.kill("SIGTERM");
      return { confirmed: false, failure: "injected helper failure" };
    });

    const observation = runObservedCommand({
      command: "node",
      cwd: "repo",
      signal: controller.signal,
      dependencies: dependencies({ spawn: vi.fn(() => child), terminateProcessTree }),
    });
    controller.abort("SIGTERM");

    await expect(observation).resolves.toMatchObject({
      exitCode: 3,
      termination: "signal",
      signal: "SIGTERM",
      cancellationConfirmed: false,
    });
    expect(terminateProcessTree).toHaveBeenCalledOnce();
  });

  it("bounds a stuck Windows termination helper and closes the direct child as unconfirmed cleanup", async () => {
    const controller = new AbortController();
    const child = new EventEmitter() as EventEmitter & { pid: number; kill: ReturnType<typeof vi.fn> };
    child.pid = 321;
    child.kill = vi.fn((signal: NodeJS.Signals) => {
      queueMicrotask(() => child.emit("close", null, signal));
      return true;
    });
    const helper = new EventEmitter() as EventEmitter & { pid: number; kill: ReturnType<typeof vi.fn> };
    helper.pid = 654;
    helper.kill = vi.fn((signal: NodeJS.Signals) => {
      queueMicrotask(() => helper.emit("close", null, signal));
      return true;
    });
    const observation = runObservedCommand({
      command: "node",
      cwd: "repo",
      signal: controller.signal,
      dependencies: dependencies({
        spawn: vi.fn(() => child),
        spawnTerminationHelper: vi.fn(() => helper),
        platform: "win32",
        systemRoot: "C:\\Windows",
        terminationTimeoutMs: 10,
      }),
    });
    controller.abort("SIGTERM");

    await expect(observation).resolves.toMatchObject({ exitCode: 3, cancellationConfirmed: false });
    expect(helper.kill).toHaveBeenCalledWith("SIGKILL");
    expect(child.kill).toHaveBeenCalledWith("SIGKILL");
  });

  it("does not leak a real observed child when the injected termination proof fails closed", async () => {
    const controller = new AbortController();
    const terminateProcessTree = vi.fn(async ({ child: observedChild }) => {
      observedChild.kill("SIGTERM");
      return { confirmed: false, failure: "injected proof failure" };
    });
    const observation = runObservedCommand({
      command: process.execPath,
      args: ["-e", "console.log(process.pid); setTimeout(() => process.exit(0), 1000)"],
      cwd: process.cwd(),
      capture: true,
      signal: controller.signal,
      dependencies: dependencies({ spawn: undefined, terminateProcessTree }),
    });
    setTimeout(() => controller.abort("SIGTERM"), 150);

    const result = await observation;
    expect(result).toMatchObject({ exitCode: 3, cancellationConfirmed: false });
    const childPid = Number(result.stdout.trim());
    expect(Number.isInteger(childPid)).toBe(true);
    expect(processIsAlive(childPid)).toBe(false);
  }, 5_000);

  it("uses a detached process group and bounded group proof on non-Windows", async () => {
    const controller = new AbortController();
    const child = new EventEmitter() as EventEmitter & { pid: number };
    child.pid = 789;
    const processKill = vi.fn((pid: number, signal: NodeJS.Signals | 0) => {
      if (signal === "SIGTERM") {
        queueMicrotask(() => child.emit("close", null, "SIGTERM"));
        return true;
      }
      throw Object.assign(new Error("process group is gone"), { code: "ESRCH" });
    });
    const spawn = vi.fn(() => child);
    const observation = runObservedCommand({
      command: "node",
      cwd: "repo",
      signal: controller.signal,
      dependencies: dependencies({ spawn, platform: "linux", processKill }),
    });
    controller.abort("SIGTERM");

    await expect(observation).resolves.toMatchObject({
      exitCode: 130,
      cancellationConfirmed: true,
      cancellation: { tree: { strategy: "posix-process-group" } },
    });
    expect(spawn.mock.calls[0][2]).toMatchObject({ detached: true, shell: false });
    expect(processKill).toHaveBeenCalledWith(-789, "SIGTERM");
  });

  it.runIf(process.platform === "win32")("does not resolve until a real child and grandchild are dead", async () => {
    const repoRoot = await mkdtemp(path.join(tmpdir(), "extractum-observer-tree-"));
    temporaryRoots.push(repoRoot);
    const pidFile = path.join(repoRoot, "tree.pid");
    const controller = new AbortController();
    let pids: number[] = [];
    const observation = runObservedCommand({
      command: process.execPath,
      args: ["-e", [
        "const { spawn } = require('node:child_process');",
        "const { writeFileSync } = require('node:fs');",
        "const grandchild = spawn(process.execPath, ['-e', 'setInterval(() => {}, 1000)'], { stdio: 'ignore' });",
        `writeFileSync(${JSON.stringify(pidFile)}, process.pid + ',' + grandchild.pid);`,
        "setInterval(() => {}, 1000);",
      ].join(" ")],
      cwd: repoRoot,
      repoRoot,
      capture: true,
      signal: controller.signal,
      dependencies: dependencies({ spawn: undefined }),
    });
    try {
      pids = await waitForPidFile(pidFile);
      expect(pids).toHaveLength(2);
      expect(pids.every(processIsAlive)).toBe(true);
      controller.abort("SIGTERM");

      await expect(observation).resolves.toMatchObject({
        exitCode: 130,
        termination: "signal",
        signal: "SIGTERM",
        cancellationConfirmed: true,
      });
      expect(pids.every((pid) => !processIsAlive(pid))).toBe(true);
    } finally {
      for (const pid of pids.reverse()) {
        if (processIsAlive(pid)) process.kill(pid, "SIGKILL");
      }
    }
  }, 30_000);
});
