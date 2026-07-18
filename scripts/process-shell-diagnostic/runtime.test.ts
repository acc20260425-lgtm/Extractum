import { mkdtemp, readFile, readdir, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { describe, expect, it } from "vitest";

import {
  assertCommandOk,
  ProtocolError,
  cargoInvocation,
  parseCargoOutput,
  runDirtyCargoProbe,
  runWindowsProcess,
  sha256File,
  terminateWindowsTree,
  writeAtomicJsonExclusive,
} from "./runtime.mjs";

async function scratch() {
  return mkdtemp(path.join(os.tmpdir(), "extractum-psd-runtime-"));
}

describe("process shell diagnostic runtime", () => {
  it("writes JSON once and refuses a duplicate artifact", async () => {
    const dir = await scratch();
    const target = path.join(dir, "value.json");
    await writeAtomicJsonExclusive(target, { value: 1 });
    await expect(writeAtomicJsonExclusive(target, { value: 2 })).rejects.toMatchObject({
      kind: "duplicate_artifact",
    });
    expect(JSON.parse(await readFile(target, "utf8"))).toEqual({ value: 1 });
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
    expect([1, 2]).toContain(JSON.parse(await readFile(target, "utf8")).writer);
  });

  it("strips every CARGO_LOG case variant from ordinary Cargo", () => {
    const invocation = cargoInvocation({
      diagnostic: false,
      baseEnv: { Path: "x", CARGO_LOG: "a", cargo_log: "b" },
    });
    expect(invocation.args).not.toContain("--timings");
    expect(invocation.args).not.toContain("-vv");
    expect(Object.keys(invocation.env).map((key) => key.toUpperCase())).not.toContain("CARGO_LOG");
  });

  it("adds timings, verbosity, and fingerprint logging only to diagnostic Cargo", () => {
    const invocation = cargoInvocation({ diagnostic: true, baseEnv: { Path: "x" } });
    expect(invocation.args).toContain("--timings");
    expect(invocation.args).toContain("-vv");
    expect(invocation.env.CARGO_LOG).toBe("cargo::core::compiler::fingerprint=info");
  });

  it("parses Cargo duration and checked packages without treating Compiling as Checking", () => {
    expect(parseCargoOutput(" Compiling helper v0.1.0\n Checking extractum v0.2.0\n Finished `dev` profile in 9.17s\n")).toEqual({
      cargoReportedMs: 9_170,
      checkedPackages: ["extractum"],
      extractumChecked: true,
      extractumLibRustcObserved: false,
      extractumProcessExtern: false,
    });
    const app = parseCargoOutput(
      "Running `rustc --crate-name extractum_lib --extern extractum_process=target\\debug\\deps\\libextractum_process.rmeta`\nFinished `dev` profile in 9.17s\n",
    );
    expect(app).toMatchObject({ extractumLibRustcObserved: true, extractumProcessExtern: true });
    expect(parseCargoOutput(
      "Running `rustc --crate-name helper --extern extractum_process=target\\debug\\deps\\libextractum_process.rmeta`\nFinished `dev` profile in 9.17s\n",
    ).extractumProcessExtern).toBe(false);
  });

  it("parses Cargo hour/minute/second durations from cold builds", () => {
    expect(parseCargoOutput("Finished `dev` profile in 1m 01.25s\n").cargoReportedMs).toBe(61_250);
    expect(parseCargoOutput("Finished `dev` profile in 1h 02m 03s\n").cargoReportedMs).toBe(3_723_000);
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
      taskkillExe: path.join(process.env.SystemRoot ?? "C:\\Windows", "System32", "taskkill.exe"),
    });
    expect(result.classification).toBe("ok");
    expect(await readFile(result.stdoutPath, "utf8")).toContain("out");
    expect(await readFile(result.stderrPath, "utf8")).toContain("err");
    const intent = await readFile(path.join(dir, "runs", "echo.intent.json"), "utf8");
    expect(intent).not.toContain("do-not-persist");
  });

  it.runIf(process.platform === "win32")("does not return until an owned grandchild tree is dead", async () => {
    const dir = await scratch();
    const pidFile = path.join(dir, "grandchild.pid");
    const result = await runWindowsProcess({
      label: "timeout",
      command: process.execPath,
      args: ["-e", [
        "const { spawn } = require('node:child_process');",
        "const { writeFileSync } = require('node:fs');",
        `const child = spawn(process.execPath, ['-e', 'setInterval(() => {}, 1000)'], { stdio: 'ignore' });`,
        `writeFileSync(${JSON.stringify(pidFile)}, String(child.pid));`,
        "setInterval(() => {}, 1000);",
      ].join(" ")],
      cwd: dir,
      env: process.env,
      artifactDir: dir,
      timeoutMs: 500,
      taskkillExe: path.join(process.env.SystemRoot ?? "C:\\Windows", "System32", "taskkill.exe"),
    });
    expect(result.timedOut).toBe(true);
    expect(result.taskkill).toMatchObject({ args: ["/PID", String(result.pid), "/T", "/F"] });
    expect(result.classification).toBe("timeout");
    expect(result.taskkill.survivors).toEqual([]);
    const grandchildPid = Number(await readFile(pidFile, "utf8"));
    expect(() => process.kill(grandchildPid, 0)).toThrow();
  });

  it("runs sync before mutation and restores from the disk recovery copy", async () => {
    const dir = await scratch();
    const sourcePath = path.join(dir, "lib.rs");
    await writeFile(sourcePath, "fn canonical() {}\n", "utf8");
    const expectedCanonicalSha256 = await sha256File(sourcePath);
    const calls: string[] = [];
    const result = await runDirtyCargoProbe({
      label: "A0-sample-1",
      worktree: dir,
      artifactDir: dir,
      sourcePath,
      expectedCanonicalSha256,
      cargoExe: "cargo.exe",
      taskkillExe: "taskkill.exe",
      timeoutMs: 1_000,
      requireExtractum: true,
      runCargoFn: async (spec: { label: string }) => {
        calls.push(spec.label);
        if (spec.label.endsWith(".dirty")) {
          expect(await readFile(sourcePath, "utf8")).toContain("process-shell-diagnostic-probe");
          return { classification: "ok", extractumChecked: true };
        }
        return { classification: "ok", extractumChecked: false };
      },
    });
    expect(result.classification).toBe("ok");
    expect(calls).toEqual(["A0-sample-1.sync", "A0-sample-1.dirty"]);
    expect(await sha256File(sourcePath)).toBe(expectedCanonicalSha256);
    expect(await readdir(path.join(dir, "recovery"))).toContain("A0-sample-1.lib.rs");
  });

  it("downgrades any survivor-proof exception to termination_unconfirmed evidence", async () => {
    const dir = await scratch();
    const evidence = await terminateWindowsTree(
      {
        pid: 4242,
        cwd: dir,
        env: { SystemRoot: "C:\\Windows" },
        taskkillExe: "taskkill.exe",
        artifactDir: dir,
        label: "proof-failure",
      },
      {
        captureOwnedWindowsPidsFn: async () => [4242, 4243],
        runTaskkillFn: async () => ({ closeObserved: true, exitCode: 0 }),
        survivingPidsFn: async () => {
          throw Object.assign(new Error("access denied"), { code: "EPERM" });
        },
      },
    );
    expect(evidence).toMatchObject({
      confirmed: false,
      terminationErrors: [{ phase: "termination-proof", kind: "Error", message: "access denied" }],
    });
    expect(JSON.parse(await readFile(
      path.join(dir, "runs", "proof-failure.termination-unconfirmed.json"),
      "utf8",
    ))).toMatchObject({ operatorActionRequired: true, confirmed: false });
  });

  it("halts with durable pending recovery after an unconfirmed Cargo-tree termination", async () => {
    const dir = await scratch();
    const sourcePath = path.join(dir, "lib.rs");
    await writeFile(sourcePath, "fn canonical() {}\n", "utf8");
    const expectedCanonicalSha256 = await sha256File(sourcePath);
    const calls: string[] = [];
    await expect(runDirtyCargoProbe({
      label: "failure",
      worktree: dir,
      artifactDir: dir,
      sourcePath,
      expectedCanonicalSha256,
      cargoExe: "cargo.exe",
      taskkillExe: "taskkill.exe",
      timeoutMs: 1_000,
      requireExtractum: true,
      runCargoFn: async (spec: { label: string }) => {
        calls.push(spec.label);
        if (spec.label.endsWith(".sync")) return { classification: "ok", extractumChecked: false };
        return { classification: "termination_unconfirmed", extractumChecked: false };
      },
    })).rejects.toMatchObject({ kind: "termination_unconfirmed" });
    expect(calls).toEqual(["failure.sync", "failure.dirty"]);
    expect(await sha256File(sourcePath)).toBe(expectedCanonicalSha256);
    expect(JSON.parse(await readFile(
      path.join(dir, "recovery", "failure.recovery-pending.json"),
      "utf8",
    ))).toMatchObject({
      label: "failure",
      canonical_sha256: expectedCanonicalSha256,
      source_restored_locally: true,
      operator_action_required: true,
    });
  });

  it("never downgrades unconfirmed termination when local restore/publication also fail", async () => {
    const dir = await scratch();
    const sourcePath = path.join(dir, "lib.rs");
    await writeFile(sourcePath, "fn canonical() {}\n", "utf8");
    const expectedCanonicalSha256 = await sha256File(sourcePath);
    await expect(runDirtyCargoProbe({
      label: "compound-failure",
      worktree: dir,
      artifactDir: dir,
      sourcePath,
      expectedCanonicalSha256,
      cargoExe: "cargo.exe",
      taskkillExe: "taskkill.exe",
      timeoutMs: 1_000,
      runCargoFn: async (spec: { label: string }) => spec.label.endsWith(".sync")
        ? { classification: "ok" }
        : { classification: "termination_unconfirmed", timedOut: true },
      restoreSourceFn: async () => {
        throw Object.assign(new Error("source busy"), { code: "EPERM" });
      },
      writeJsonFn: async () => {
        throw new Error("artifact volume unavailable");
      },
    })).rejects.toMatchObject({
      kind: "termination_unconfirmed",
      details: {
        operatorActionRequired: true,
        restorationError: { message: "source busy" },
        pendingPublicationError: { message: "artifact volume unavailable" },
      },
    });
  });

  it("does not mutate when canonical sync fails", async () => {
    const dir = await scratch();
    const sourcePath = path.join(dir, "lib.rs");
    await writeFile(sourcePath, "fn canonical() {}\n", "utf8");
    const expectedCanonicalSha256 = await sha256File(sourcePath);
    await expect(runDirtyCargoProbe({
      label: "sync-failure",
      worktree: dir,
      artifactDir: dir,
      sourcePath,
      expectedCanonicalSha256,
      cargoExe: "cargo.exe",
      taskkillExe: "taskkill.exe",
      timeoutMs: 1_000,
      requireExtractum: true,
      runCargoFn: async () => ({ classification: "timeout", timedOut: true, extractumChecked: false }),
    })).rejects.toMatchObject({ kind: "command_timeout" });
    expect(await sha256File(sourcePath)).toBe(expectedCanonicalSha256);
    await expect(readdir(path.join(dir, "recovery"))).rejects.toMatchObject({ code: "ENOENT" });
  });
});
