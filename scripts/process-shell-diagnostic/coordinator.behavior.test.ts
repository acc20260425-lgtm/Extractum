import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { afterEach, describe, expect, it } from "vitest";

import { resumeSession, startSession } from "./coordinator.mjs";
import { runWindowsProcess, terminateWindowsTree } from "./runtime.mjs";

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

function processIsAlive(pid: number) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ESRCH") return false;
    throw error;
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
      taskkillExe: path.join(process.env.SystemRoot ?? "C:\\Windows", "System32", "taskkill.exe"),
      artifactDir: owned.artifactDir,
      label: `test-cleanup-${owned.label}-${owned.pid}`,
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

async function paths() {
  requireWindows();
  const root = await mkdtemp(path.join(os.tmpdir(), "extractum-psd-session-behavior-"));
  roots.push(root);
  const mainRoot = path.join(root, "main");
  const protocolRoot = path.join(root, "protocol");
  const scratchParent = path.join(root, "scratch");
  await mkdir(path.join(protocolRoot, "scripts", "process-shell-diagnostic"), { recursive: true });
  await mkdir(mainRoot, { recursive: true });
  await mkdir(scratchParent, { recursive: true });
  await writeFile(
    path.join(protocolRoot, "scripts", "process-shell-diagnostic", "protocol-lock.json"),
    `${JSON.stringify({
      schemaVersion: 1,
      states: {
        A: { srcTauriTree: "a-tree" }, B: { srcTauriTree: "b-tree" },
        C: { srcTauriTree: "c-tree" }, D: { srcTauriTree: "d-tree" },
        E: { srcTauriTree: "e-tree" },
      },
    })}\n`,
    "utf8",
  );
  return { mainRoot, protocolRoot, scratchParent };
}

function attempt(kind: string) {
  return (spec: Record<string, string>) => ({
    schemaVersion: 1,
    attemptId: spec.attemptId,
    kind,
    reasons: kind === "stability_invalid" ? ["anchor_range_exceeded"] : [],
    evaluation: kind === "valid"
      ? { kind: "valid", classification: "not_reproduced" }
      : { kind: "stability_invalid", reasons: ["anchor_range_exceeded"] },
    finalState: { kind: "A", srcTauriTree: "a-tree" },
    worktree: spec.worktree,
  });
}

function fake(attempts: Array<(spec: Record<string, string>) => Record<string, unknown>>) {
  const queue = [...attempts];
  const worktrees: string[] = [];
  const targets: string[] = [];
  return {
    worktrees,
    targets,
    dependencies: {
      uuidFn: () => "session-fixed",
      nowFn: () => "2026-07-18T12:00:00.000Z",
      processEnv: {},
      verifyFrozenProtocolFn: async () => ({
        protocolCommit: "a".repeat(40),
        lockPath: "scripts/process-shell-diagnostic/protocol-lock.json",
        lockBlob: "b".repeat(40),
        lockSha256: "c".repeat(64),
        protocolVersion: 1,
        protocolLock: {
          schemaVersion: 1,
          states: {
            A: { srcTauriTree: "a-tree" }, B: { srcTauriTree: "b-tree" },
            C: { srcTauriTree: "c-tree" }, D: { srcTauriTree: "d-tree" },
            E: { srcTauriTree: "e-tree" },
          },
        },
      }),
      captureEnvironmentFn: async () => ({
        platform: "win32",
        host: "x86_64-pc-windows-msvc",
        cargo: "cargo 1.95.0",
        rustc: "rustc 1.95.0",
        power: "Balanced",
        defender: "unavailable: Access denied",
        processQuiescence: [],
        operatorProcessAttestation: true,
        cargoEnvironment: {},
        mainRoot: "G:\\main",
        mainSrcTauriTree: "a-tree",
        mainTargetDirectory: "G:\\main\\src-tauri\\target",
        mainTargetSnapshot: { exists: true, records: [], digest: "baseline-target" },
      }),
      createDetachedWorktreeFn: async ({ worktree }: { worktree: string }) => {
        await mkdir(worktree, { recursive: true });
        worktrees.push(worktree);
      },
      restoreAttemptWorktreeFn: async () => ({ kind: "A", srcTauriTree: "a-tree" }),
      runAttemptFn: async (spec: Record<string, string>) => {
        targets.push(path.join(spec.worktree, "src-tauri", "target"));
        const processResult = await runWindowsProcess({
          label: `${spec.attemptId}-owned-fixture`,
          command: process.execPath,
          args: ["-e", "process.stdout.write('owned coordinator fixture')"],
          cwd: spec.worktree,
          env: process.env,
          artifactDir: path.join(spec.sessionDir, "attempts", spec.attemptId),
          timeoutMs: 10_000,
          taskkillExe: path.join(process.env.SystemRoot ?? "C:\\Windows", "System32", "taskkill.exe"),
        });
        ownedProcesses.set(processResult.pid, {
          pid: processResult.pid,
          cwd: spec.worktree,
          artifactDir: path.join(spec.sessionDir, "attempts", spec.attemptId),
          label: `${spec.attemptId}-owned-fixture`,
        });
        if (processResult.classification !== "ok" || processResult.closeObserved !== true) {
          throw new Error(`Windows owned-process failure: ${JSON.stringify(processResult)}`);
        }
        if (processIsAlive(processResult.pid)) {
          throw new Error(`Windows owned-process survivor: ${processResult.pid}`);
        }
        const next = queue.shift();
        if (!next) throw new Error("unexpected extra attempt");
        const result = next(spec);
        await writeFile(
          path.join(spec.sessionDir, "attempts", spec.attemptId, "attempt-result.json"),
          `${JSON.stringify(result, null, 2)}\n`,
          { encoding: "utf8", flag: "wx" },
        );
        return result;
      },
    },
  };
}

describe("process shell diagnostic coordinator", { timeout: 30_000 }, () => {
  it("pins one locator and completes one valid session", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    const result = await startSession({ ...input, processAttested: true }, value.dependencies);
    expect(result).toMatchObject({ status: "completed", classification: "not_reproduced", retryState: { unexplainedStabilityInvalidCount: 0, terminal: true } });
    expect(value.worktrees).toEqual([path.join(input.mainRoot, ".worktrees", "process-shell-session-session-fixed", "attempt-001")]);
    expect(value.targets).toEqual([path.join(value.worktrees[0], "src-tauri", "target")]);
    await expect(startSession({ ...input, processAttested: true }, value.dependencies)).rejects.toMatchObject({ kind: "session_locator_exists" });
    expect(result).toMatchObject({ classification: "not_reproduced", attempts: [{ attemptId: "attempt-001" }] });
  });

  it("invalidates target or toolchain drift before a later attempt creates its worktree", async () => {
    const input = await paths();
    const value = fake([attempt("stability_invalid"), attempt("valid")]);
    const baseCapture = value.dependencies.captureEnvironmentFn;
    let captureNumber = 0;
    const captureEnvironmentFn = async () => ({
      ...await baseCapture(),
      mainRoot: input.mainRoot,
      mainTargetDirectory: path.join(input.mainRoot, "src-tauri", "target"),
      mainTargetSnapshot: { exists: true, records: [], digest: ++captureNumber >= 3 ? "changed-target" : "baseline-target" },
    });
    const first = await startSession({ ...input, processAttested: true }, { ...value.dependencies, captureEnvironmentFn });
    let environmentFailure: Record<string, unknown> | undefined;
    const second = await resumeSession({ sessionDir: first.sessionDir, unexplainedStability: true, processAttested: true }, {
      ...value.dependencies,
      captureEnvironmentFn,
      afterDurableWriteFn: async ({ target, value: event }: { target: string; value: Record<string, unknown> }) => {
        if (target.endsWith("coordinator-failure.json")) environmentFailure = event;
      },
    });
    expect(second.status).toBe("awaiting_correction");
    expect(value.worktrees).toHaveLength(1);
    expect(second.attempts[1]).toMatchObject({ status: "infrastructure_invalid", reasons: ["coordinator_failure"], resultPath: expect.stringMatching(/coordinator-failure\.json$/) });
    expect(environmentFailure).toMatchObject({ error: { kind: "attempt_environment_drift" } });
  });

  it("pins a pre-recovery result path and digest across a crash inside recovery", async () => {
    const input = await paths();
    const value = fake([(spec) => ({ ...attempt("valid")(spec), finalState: null })]);
    await expect(startSession({ ...input, processAttested: true }, { ...value.dependencies, afterAttemptObservedFn: async () => { throw Object.assign(new Error("simulated crash"), { simulatedCrash: true }); } })).rejects.toThrow("simulated crash");
    const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
    let recoveryCrash = false;
    let reservation: Record<string, unknown> | undefined;
    await expect(resumeSession({ sessionDir, processAttested: true }, { ...value.dependencies, afterDurableWriteFn: async ({ value: event }: { value: Record<string, unknown> & { type?: string } }) => { if (!recoveryCrash && event.type === "attempt_recovery_started") { recoveryCrash = true; reservation = event; throw Object.assign(new Error("simulated crash"), { simulatedCrash: true }); } } })).rejects.toThrow("simulated crash");
    const recovered = await resumeSession({ sessionDir, processAttested: true }, value.dependencies);
    expect(recovered.status).toBe("awaiting_correction");
    expect(reservation).toMatchObject({ sourceResultPath: path.join(sessionDir, "attempts", "attempt-001", "attempt-result.json"), sourceResultSha256: expect.stringMatching(/^[0-9a-f]{64}$/) });
    expect(recovered.attempts[0]).toMatchObject({ status: "infrastructure_invalid", reasons: ["final_restore_evidence_missing"] });
  });

  it("materializes missing terminal projections across both publication windows", async () => {
    for (const point of ["session_completed", "session-ledger.json"]) {
      const input = await paths();
      const value = fake([attempt("valid")]);
      let crashed = false;
      await expect(startSession({ ...input, processAttested: true }, { ...value.dependencies, afterDurableWriteFn: async ({ target, value: event }: { target: string; value: { type?: string } }) => { if (!crashed && (event.type === point || target.endsWith(point))) { crashed = true; throw Object.assign(new Error("simulated crash"), { simulatedCrash: true }); } } })).rejects.toThrow("simulated crash");
      const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
      const recovered = await resumeSession({ sessionDir, processAttested: true }, value.dependencies);
      expect(recovered.status).toBe("completed");
      expect(recovered).toMatchObject({ retryState: { terminal: true }, attempts: [{ attemptId: "attempt-001" }] });
      expect(recovered.classification).toBe("not_reproduced");
    }
  });

  it("retries bootstrap materialization in a new artifact directory after two crashes", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    let locatorCrash = false;
    await expect(startSession({ ...input, processAttested: true }, { ...value.dependencies, afterDurableWriteFn: async ({ target }: { target: string }) => { if (!locatorCrash && target.endsWith("process-shell-diagnostic.locator.json")) { locatorCrash = true; throw Object.assign(new Error("simulated crash"), { simulatedCrash: true }); } } })).rejects.toThrow("simulated crash");
    const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
    const baseCapture = value.dependencies.captureEnvironmentFn;
    let bootstrapCrashes = 0;
    const recoveryArtifactDirs: string[] = [];
    const captureEnvironmentFn = async (spec: { artifactDir: string }) => {
      if (path.basename(spec.artifactDir).startsWith("bootstrap-recovery-")) {
        recoveryArtifactDirs.push(spec.artifactDir);
        await mkdir(spec.artifactDir, { recursive: true });
        if (bootstrapCrashes < 2) throw new Error(`bootstrap capture crash ${++bootstrapCrashes}`);
      }
      return baseCapture();
    };
    await expect(resumeSession({ sessionDir, processAttested: true }, { ...value.dependencies, captureEnvironmentFn })).rejects.toThrow("bootstrap capture crash 1");
    await expect(resumeSession({ sessionDir, processAttested: true }, { ...value.dependencies, captureEnvironmentFn })).rejects.toThrow("bootstrap capture crash 2");
    const recovered = await resumeSession({ sessionDir, processAttested: true }, { ...value.dependencies, captureEnvironmentFn });
    expect(recovered.status).toBe("completed");
    expect(new Set(recoveryArtifactDirs).size).toBe(3);
  });

  it("points coordinator failure rows at their real immutable artifact", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    let worktreeFailure: Record<string, unknown> | undefined;
    const result = await startSession({ ...input, processAttested: true }, {
      ...value.dependencies,
      createDetachedWorktreeFn: async () => { throw Object.assign(new Error("forced worktree failure"), { kind: "worktree_create_failed" }); },
      afterDurableWriteFn: async ({ target, value: event }: { target: string; value: Record<string, unknown> }) => {
        if (target.endsWith("coordinator-failure.json")) worktreeFailure = event;
      },
    });
    expect(result.status).toBe("awaiting_correction");
    expect(result.attempts[0].resultPath).toMatch(/coordinator-failure\.json$/);
    expect(result.attempts[0]).toMatchObject({ status: "infrastructure_invalid", reasons: ["coordinator_failure"] });
    expect(worktreeFailure).toMatchObject({ kind: "infrastructure_invalid", error: { kind: "worktree_create_failed" } });
  });

  it("pins the lock-containing commit, blob, and SHA in the immutable manifest", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    let manifest: Record<string, unknown> | undefined;
    await startSession({ ...input, processAttested: true }, {
      ...value.dependencies,
      afterDurableWriteFn: async ({ target, value }: { target: string; value: Record<string, unknown> }) => {
        if (target.endsWith("session-manifest.json")) manifest = value;
      },
    });
    expect(manifest?.protocol).toEqual({ protocolCommit: "a".repeat(40), lockPath: "scripts/process-shell-diagnostic/protocol-lock.json", lockBlob: "b".repeat(40), lockSha256: "c".repeat(64), protocolVersion: 1 });
  });

  it("invalidates a valid Cargo result when the protocol root changes mid-flight", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    const baseRun = value.dependencies.runAttemptFn;
    const baseVerify = value.dependencies.verifyFrozenProtocolFn;
    let cargoCompleted = false;
    let protocolFailure: Record<string, unknown> | undefined;
    const result = await startSession({ ...input, processAttested: true }, {
      ...value.dependencies,
      runAttemptFn: async (spec: Record<string, string>) => { const completed = await baseRun(spec); cargoCompleted = true; return completed; },
      verifyFrozenProtocolFn: async ({ repoRoot }: { repoRoot: string }) => { const verified = await baseVerify(); return cargoCompleted && path.resolve(repoRoot) === path.resolve(input.protocolRoot) ? { ...verified, lockSha256: "d".repeat(64) } : verified; },
      afterDurableWriteFn: async ({ target, value: event }: { target: string; value: Record<string, unknown> }) => {
        if (target.endsWith("coordinator-failure.json")) protocolFailure = event;
      },
    });
    expect(result.status).toBe("awaiting_correction");
    expect(result.attempts[0]).toMatchObject({ status: "infrastructure_invalid", reasons: ["coordinator_failure"], resultPath: expect.stringMatching(/coordinator-failure\.json$/) });
    expect(protocolFailure).toMatchObject({ error: { kind: "protocol_pin_mismatch" } });
  });

  it("rechecks the attempt-worktree pin when resuming a durable pre-check result", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    const baseRun = value.dependencies.runAttemptFn;
    const baseVerify = value.dependencies.verifyFrozenProtocolFn;
    let cargoCompleted = false;
    let crashBeforePostCheck = true;
    await expect(startSession({ ...input, processAttested: true }, { ...value.dependencies, runAttemptFn: async (spec: Record<string, string>) => { const completed = await baseRun(spec); cargoCompleted = true; return completed; }, verifyFrozenProtocolFn: async ({ repoRoot }: { repoRoot: string }) => { if (cargoCompleted && crashBeforePostCheck && path.resolve(repoRoot) !== path.resolve(input.protocolRoot)) { crashBeforePostCheck = false; throw Object.assign(new Error("simulated crash"), { simulatedCrash: true }); } return baseVerify(); } })).rejects.toThrow("simulated crash");
    const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
    let failurePublicationCrash = false;
    let protocolFailure: Record<string, unknown> | undefined;
    await expect(resumeSession({ sessionDir, processAttested: true }, { ...value.dependencies, verifyFrozenProtocolFn: async ({ repoRoot }: { repoRoot: string }) => { const verified = await baseVerify(); return path.resolve(repoRoot) === path.resolve(input.protocolRoot) ? verified : { ...verified, lockSha256: "d".repeat(64) }; }, afterDurableWriteFn: async ({ target, value: event }: { target: string; value: Record<string, unknown> }) => { if (!failurePublicationCrash && target.endsWith("coordinator-failure.json")) { protocolFailure = event; failurePublicationCrash = true; throw Object.assign(new Error("simulated failure-publication crash"), { simulatedCrash: true }); } } })).rejects.toThrow("simulated failure-publication crash");
    const recovered = await resumeSession({ sessionDir, processAttested: true }, { ...value.dependencies, verifyFrozenProtocolFn: baseVerify });
    expect(recovered.status).toBe("awaiting_correction");
    expect(recovered.attempts[0]).toMatchObject({ status: "infrastructure_invalid", reasons: ["coordinator_failure"], resultPath: expect.stringMatching(/coordinator-failure\.json$/) });
    expect(protocolFailure).toMatchObject({ error: { kind: "protocol_pin_mismatch" } });
  });
});
