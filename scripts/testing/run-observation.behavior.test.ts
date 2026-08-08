import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it, vi } from "vitest";

import { runObservedCommand } from "./run-observation.mjs";

const roots: string[] = [];

afterEach(async () => {
  await Promise.all(roots.splice(0).map((root) => rm(root, { recursive: true, force: true })));
});

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

describe("runObservedCommand", () => {
  it.runIf(process.platform === "win32")("does not resolve until a real child and grandchild are dead", async () => {
    const repoRoot = await mkdtemp(path.join(tmpdir(), "extractum-observer-tree-"));
    roots.push(repoRoot);
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
      dependencies: {
        readHeadCommit: vi.fn(() => "c".repeat(40)),
        recordTimingBestEffort: vi.fn(async () => true),
      },
    });
    try {
      pids = await waitForPidFile(pidFile);
      expect(pids).toHaveLength(2);
      expect(pids.every(processIsAlive)).toBe(true);
      controller.abort("SIGTERM");
      await expect(observation).resolves.toMatchObject({
        exitCode: 130,
        termination: "signal",
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
