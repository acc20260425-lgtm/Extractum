import { spawn } from "node:child_process";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { afterEach, describe, expect, it } from "vitest";

import {
  runDirtyCargoProbe,
  runWindowsProcess,
  sha256File,
  terminateWindowsTree,
  writeAtomicJsonExclusive,
} from "./runtime.mjs";

const roots: string[] = [];
type OwnedProcess = { pid: number; cwd: string; artifactDir: string; label: string };
const ownedProcesses = new Map<number, OwnedProcess>();

afterEach(async () => {
  await cleanupOwnedProcesses();
  await Promise.all(roots.splice(0).map((root) => rm(root, { recursive: true, force: true })));
});

function requireWindows() {
  if (process.platform !== "win32") {
    throw new Error(`process shell diagnostic behavior requires Windows; received ${process.platform}`);
  }
}

function taskkillExe() {
  return path.join(process.env.SystemRoot ?? "C:\\Windows", "System32", "taskkill.exe");
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

function trackOwnedProcess(process: OwnedProcess) {
  ownedProcesses.set(process.pid, process);
}

function trackProcessResult(
  result: { pid: number; taskkill?: { observedPids?: number[] } | null },
  cwd: string,
  artifactDir: string,
  label: string,
) {
  for (const pid of new Set([result.pid, ...(result.taskkill?.observedPids ?? [])])) {
    trackOwnedProcess({ pid, cwd, artifactDir, label: `${label}-${pid}` });
  }
}

async function cleanupOwnedProcesses() {
  const failures: string[] = [];
  for (const owned of [...ownedProcesses.values()].reverse()) {
    if (!processIsAlive(owned.pid)) {
      ownedProcesses.delete(owned.pid);
      continue;
    }
    await mkdir(path.join(owned.artifactDir, "runs"), { recursive: true });
    const evidence = await terminateWindowsTree({
      pid: owned.pid,
      cwd: owned.cwd,
      env: process.env,
      taskkillExe: taskkillExe(),
      artifactDir: owned.artifactDir,
      label: `test-cleanup-${owned.label}`,
    });
    if (evidence.confirmed !== true || processIsAlive(owned.pid)) {
      failures.push(`PID ${owned.pid}: ${JSON.stringify(evidence)}`);
      continue;
    }
    ownedProcesses.delete(owned.pid);
  }
  if (failures.length) {
    throw new Error(`Windows owned-process cleanup failed:\n${failures.join("\n")}`);
  }
}

async function scratch() {
  requireWindows();
  const root = await mkdtemp(path.join(os.tmpdir(), "extractum-psd-runtime-behavior-"));
  roots.push(root);
  return root;
}

describe("process shell diagnostic runtime", { timeout: 30_000 }, () => {
  it("writes JSON once and refuses a duplicate artifact", async () => {
    const dir = await scratch();
    const target = path.join(dir, "value.json");
    await writeAtomicJsonExclusive(target, { value: 1 });
    const before = await sha256File(target);
    await expect(writeAtomicJsonExclusive(target, { value: 2 })).rejects.toMatchObject({ kind: "duplicate_artifact" });
    expect(await sha256File(target)).toBe(before);
  });

  it("lets exactly one concurrent publisher claim an artifact", async () => {
    const dir = await scratch();
    const target = path.join(dir, "race.json");
    const settled = await Promise.allSettled([
      writeAtomicJsonExclusive(target, { writer: 1 }),
      writeAtomicJsonExclusive(target, { writer: 2 }),
    ]);
    expect(settled.filter((entry) => entry.status === "fulfilled")).toHaveLength(1);
    expect(settled.filter((entry) => entry.status === "rejected")).toHaveLength(1);
    expect(await sha256File(target)).toMatch(/^[0-9a-f]{64}$/);
  });

  it("captures direct child stdout/stderr through close", async () => {
    const dir = await scratch();
    const result = await runWindowsProcess({
      label: "echo",
      command: process.execPath,
      args: ["-e", "console.log('out'); console.error('err')"],
      cwd: dir,
      env: { ...process.env, EXTRACTUM_TEST_SECRET: "do-not-persist" },
      artifactDir: dir,
      timeoutMs: 10_000,
      taskkillExe: taskkillExe(),
    });
    trackProcessResult(result, dir, dir, "capture");
    expect(result.classification, `Windows process failure: ${JSON.stringify(result)}`).toBe("ok");
    expect(await sha256File(result.stdoutPath)).toBe(createHash("sha256").update("out\n").digest("hex"));
    expect(await sha256File(result.stderrPath)).toBe(createHash("sha256").update("err\n").digest("hex"));
    expect(await sha256File(path.join(dir, "runs", "echo.intent.json"))).toMatch(/^[0-9a-f]{64}$/);
  });

  it("does not return until an owned grandchild tree is dead", async () => {
    const dir = await scratch();
    const result = await runWindowsProcess({
      label: "timeout",
      command: process.execPath,
      args: ["-e", [
        "const { spawn } = require('node:child_process');",
        "const child = spawn(process.execPath, ['-e', 'setInterval(() => {}, 1000)'], { stdio: 'ignore' });",
        "setInterval(() => {}, 1000);",
      ].join(" ")],
      cwd: dir,
      env: process.env,
      artifactDir: dir,
      timeoutMs: 500,
      taskkillExe: taskkillExe(),
    });
    trackProcessResult(result, dir, dir, "descendant-cleanup");
    expect(result.timedOut).toBe(true);
    expect(result.taskkill).toMatchObject({ args: ["/PID", String(result.pid), "/T", "/F"] });
    expect(result.classification, `Windows termination failure: ${JSON.stringify(result)}`).toBe("timeout");
    expect(result.taskkill.survivors).toEqual([]);
    expect(result.taskkill.observedPids.length).toBeGreaterThan(1);
    expect(result.taskkill.observedPids.every((pid: number) => !processIsAlive(pid)), `owned Windows process survived: ${result.taskkill.observedPids.join(",")}`).toBe(true);
  });

  it("runs sync before mutation and restores from the disk recovery copy", async () => {
    const dir = await scratch();
    const sourcePath = path.join(dir, "lib.rs");
    await writeFile(sourcePath, "fn canonical() {}\n", "utf8");
    const expectedCanonicalSha256 = await sha256File(sourcePath);
    const calls: string[] = [];
    const result = await runDirtyCargoProbe({
      label: "A0-sample-1", worktree: dir, artifactDir: dir, sourcePath,
      expectedCanonicalSha256, cargoExe: "cargo.exe", taskkillExe: "taskkill.exe",
      timeoutMs: 1_000, requireExtractum: true,
      runCargoFn: async (spec: { label: string }) => {
        calls.push(spec.label);
        if (spec.label.endsWith(".dirty")) {
          expect(await sha256File(sourcePath)).not.toBe(expectedCanonicalSha256);
          return { classification: "ok", extractumChecked: true };
        }
        return { classification: "ok", extractumChecked: false };
      },
    });
    expect(result.classification).toBe("ok");
    expect(calls).toEqual(["A0-sample-1.sync", "A0-sample-1.dirty"]);
    expect(await sha256File(sourcePath)).toBe(expectedCanonicalSha256);
    expect(await sha256File(path.join(dir, "recovery", "A0-sample-1.lib.rs"))).toBe(expectedCanonicalSha256);
  });

  it("downgrades any survivor-proof exception to termination_unconfirmed evidence", async () => {
    const dir = await scratch();
    await mkdir(path.join(dir, "runs"), { recursive: true });
    const child = spawn(process.execPath, ["-e", "setInterval(() => {}, 1000)"], {
      cwd: dir,
      env: process.env,
      shell: false,
      windowsHide: true,
      stdio: "ignore",
    });
    await new Promise<void>((resolve, reject) => {
      child.once("spawn", resolve);
      child.once("error", reject);
    });
    if (!child.pid) throw new Error("Windows failure-reporting fixture did not expose a PID");
    trackOwnedProcess({ pid: child.pid, cwd: dir, artifactDir: dir, label: "proof-failure" });
    const evidence = await terminateWindowsTree({
      pid: child.pid,
      cwd: dir,
      env: process.env,
      taskkillExe: path.join(dir, "missing-taskkill.exe"),
      artifactDir: dir,
      label: "proof-failure",
    });
    expect(evidence).toMatchObject({
      confirmed: false,
      spawnError: expect.stringContaining("missing-taskkill.exe"),
      terminationErrors: [],
    });
    expect(evidence.spawnError).toContain("ENOENT");
    expect(evidence.survivors).toContain(child.pid);
    expect(evidence.survivors.every((pid: number) => evidence.observedPids.includes(pid))).toBe(true);
    expect(evidence.survivors.every(processIsAlive)).toBe(true);
    await cleanupOwnedProcesses();
    expect(processIsAlive(child.pid), `owned Windows process survived failed proof: ${child.pid}`).toBe(false);
    expect(await sha256File(path.join(dir, "runs", "proof-failure.termination-unconfirmed.json"))).toMatch(/^[0-9a-f]{64}$/);
  });

  it("halts with durable pending recovery after an unconfirmed Cargo-tree termination", async () => {
    const dir = await scratch();
    const sourcePath = path.join(dir, "lib.rs");
    await writeFile(sourcePath, "fn canonical() {}\n", "utf8");
    const expectedCanonicalSha256 = await sha256File(sourcePath);
    const calls: string[] = [];
    let pendingEvidence: Record<string, unknown> | undefined;
    await expect(runDirtyCargoProbe({
      label: "failure", worktree: dir, artifactDir: dir, sourcePath,
      expectedCanonicalSha256, cargoExe: "cargo.exe", taskkillExe: "taskkill.exe",
      timeoutMs: 1_000, requireExtractum: true,
      runCargoFn: async (spec: { label: string }) => {
        calls.push(spec.label);
        return spec.label.endsWith(".sync")
          ? { classification: "ok", extractumChecked: false }
          : { classification: "termination_unconfirmed", extractumChecked: false };
      },
      writeJsonFn: async (target: string, value: Record<string, unknown>) => {
        pendingEvidence = value;
        await writeAtomicJsonExclusive(target, value);
      },
    })).rejects.toMatchObject({ kind: "termination_unconfirmed" });
    expect(calls).toEqual(["failure.sync", "failure.dirty"]);
    expect(await sha256File(sourcePath)).toBe(expectedCanonicalSha256);
    expect(pendingEvidence).toMatchObject({
      label: "failure",
      canonical_sha256: expectedCanonicalSha256,
      source_restored_locally: true,
      operator_action_required: true,
    });
  });

  it("does not mutate when canonical sync fails", async () => {
    const dir = await scratch();
    const sourcePath = path.join(dir, "lib.rs");
    await writeFile(sourcePath, "fn canonical() {}\n", "utf8");
    const expectedCanonicalSha256 = await sha256File(sourcePath);
    let publicationCount = 0;
    await expect(runDirtyCargoProbe({
      label: "sync-failure", worktree: dir, artifactDir: dir, sourcePath,
      expectedCanonicalSha256, cargoExe: "cargo.exe", taskkillExe: "taskkill.exe",
      timeoutMs: 1_000, requireExtractum: true,
      runCargoFn: async () => ({ classification: "timeout", timedOut: true, extractumChecked: false }),
      writeJsonFn: async () => { publicationCount += 1; },
    })).rejects.toMatchObject({ kind: "command_timeout" });
    expect(await sha256File(sourcePath)).toBe(expectedCanonicalSha256);
    expect(publicationCount).toBe(0);
  });
});
