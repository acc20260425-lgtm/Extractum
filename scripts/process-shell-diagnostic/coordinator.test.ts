import { mkdtemp, mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";

import {
  assertControlCommandResult,
  parseCli,
  resumeSession,
  snapshotDirectory,
  startSession,
} from "./coordinator.mjs";

async function paths() {
  const root = await mkdtemp(path.join(os.tmpdir(), "extractum-psd-session-"));
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
        A: { srcTauriTree: "a-tree" },
        B: { srcTauriTree: "b-tree" },
        C: { srcTauriTree: "c-tree" },
        D: { srcTauriTree: "d-tree" },
        E: { srcTauriTree: "e-tree" },
      },
    })}\n`,
    "utf8",
  );
  return { mainRoot, protocolRoot, scratchParent };
}

function attempt(kind: string, classification = "not_reproduced") {
  return (spec: Record<string, string>) => ({
    schemaVersion: 1,
    attemptId: spec.attemptId,
    kind,
    reasons: kind === "stability_invalid" ? ["anchor_range_exceeded"] : kind === "infrastructure_invalid" ? ["command_failed"] : [],
    evaluation: kind === "valid"
      ? { kind: "valid", classification }
      : kind === "stability_invalid"
        ? { kind: "stability_invalid", reasons: ["anchor_range_exceeded"] }
        : null,
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
      resolveProtocolCommitFn: async () => "a".repeat(40),
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

describe("process shell diagnostic coordinator", () => {
  it("pins one locator and completes one valid session", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    const result = await startSession(
      { ...input, processAttested: true },
      value.dependencies,
    );
    expect(result).toMatchObject({
      status: "completed",
      classification: "not_reproduced",
      retryState: { unexplainedStabilityInvalidCount: 0, terminal: true },
    });
    expect(value.worktrees).toEqual([
      path.join(input.mainRoot, ".worktrees", "process-shell-session-session-fixed", "attempt-001"),
    ]);
    expect(value.targets).toEqual([
      path.join(value.worktrees[0], "src-tauri", "target"),
    ]);
    await expect(startSession(
      { ...input, processAttested: true },
      value.dependencies,
    )).rejects.toMatchObject({ kind: "session_locator_exists" });
    const decision = JSON.parse(await readFile(path.join(result.sessionDir, "decision.json"), "utf8"));
    expect(decision).toMatchObject({ classification: "not_reproduced", attemptId: "attempt-001" });
  });

  it("uses two explicit unexplained dispositions before the precision terminal", async () => {
    const input = await paths();
    const value = fake([attempt("stability_invalid"), attempt("stability_invalid")]);
    const first = await startSession({ ...input, processAttested: true }, value.dependencies);
    expect(first).toMatchObject({
      status: "awaiting_stability_disposition",
      retryState: { unexplainedStabilityInvalidCount: 0 },
    });
    const second = await resumeSession(
      { sessionDir: first.sessionDir, unexplainedStability: true, processAttested: true },
      value.dependencies,
    );
    expect(second).toMatchObject({
      status: "awaiting_stability_disposition",
      retryState: { unexplainedStabilityInvalidCount: 1 },
    });
    const terminal = await resumeSession(
      { sessionDir: first.sessionDir, unexplainedStability: true },
      value.dependencies,
    );
    expect(terminal).toMatchObject({
      status: "completed",
      classification: "environment_precision_insufficient",
      retryState: { unexplainedStabilityInvalidCount: 2, terminal: true },
    });
    expect(value.worktrees).toHaveLength(2);
    expect(new Set(value.targets).size).toBe(2);
  });

  it("keeps count one through a corrected infrastructure retry", async () => {
    const input = await paths();
    const value = fake([
      attempt("stability_invalid"),
      attempt("infrastructure_invalid"),
      attempt("valid", "boundary_composite"),
    ]);
    const first = await startSession({ ...input, processAttested: true }, value.dependencies);
    const second = await resumeSession(
      { sessionDir: first.sessionDir, unexplainedStability: true, processAttested: true },
      value.dependencies,
    );
    expect(second.status).toBe("awaiting_correction");
    expect(second.retryState.unexplainedStabilityInvalidCount).toBe(1);
    const stillWaiting = await resumeSession({ sessionDir: first.sessionDir }, value.dependencies);
    expect(stillWaiting.status).toBe("awaiting_correction");
    expect(value.worktrees).toHaveLength(2);
    const final = await resumeSession(
      { sessionDir: first.sessionDir, correctedCause: "Defender scan ended; the approved exclusion was verified.", processAttested: true },
      value.dependencies,
    );
    expect(final).toMatchObject({
      status: "completed",
      classification: "boundary_composite",
      retryState: { unexplainedStabilityInvalidCount: 1, terminal: true },
    });
  });

  it("invalidates target or toolchain drift before a later attempt creates its worktree", async () => {
    const input = await paths();
    const value = fake([attempt("stability_invalid"), attempt("valid")]);
    const baseCapture = value.dependencies.captureEnvironmentFn;
    let captureNumber = 0;
    const captureEnvironmentFn = async () => {
      captureNumber += 1;
      const environment = await baseCapture();
      return {
        ...environment,
        mainRoot: input.mainRoot,
        mainTargetDirectory: path.join(input.mainRoot, "src-tauri", "target"),
        mainTargetSnapshot: {
          exists: true,
          records: [],
          digest: captureNumber >= 3 ? "changed-target" : "baseline-target",
        },
      };
    };
    const first = await startSession(
      { ...input, processAttested: true },
      { ...value.dependencies, captureEnvironmentFn },
    );
    const second = await resumeSession(
      { sessionDir: first.sessionDir, unexplainedStability: true, processAttested: true },
      { ...value.dependencies, captureEnvironmentFn },
    );
    expect(second.status).toBe("awaiting_correction");
    expect(value.worktrees).toHaveLength(1);
    expect(JSON.parse(await readFile(second.attempts[1].resultPath, "utf8"))).toMatchObject({
      reasons: ["coordinator_failure"],
      error: { kind: "attempt_environment_drift" },
    });
  });

  it("requires a corrected-cause disposition before power or Defender drift", async () => {
    async function runCase(firstKind: "stability_invalid" | "infrastructure_invalid", corrected: boolean) {
      const input = await paths();
      const value = fake([attempt(firstKind), attempt("valid")]);
      const baseCapture = value.dependencies.captureEnvironmentFn;
      let captureNumber = 0;
      const captureEnvironmentFn = async () => {
        captureNumber += 1;
        return {
          ...await baseCapture(),
          mainRoot: input.mainRoot,
          mainTargetDirectory: path.join(input.mainRoot, "src-tauri", "target"),
          power: captureNumber >= 3 ? "High performance" : "Balanced",
        };
      };
      const first = await startSession(
        { ...input, processAttested: true },
        { ...value.dependencies, captureEnvironmentFn },
      );
      const options = corrected
        ? { sessionDir: first.sessionDir, correctedCause: "Power plan fixed by operator", processAttested: true }
        : { sessionDir: first.sessionDir, unexplainedStability: true, processAttested: true };
      return {
        result: await resumeSession(options, { ...value.dependencies, captureEnvironmentFn }),
        worktrees: value.worktrees,
      };
    }

    const unexplained = await runCase("stability_invalid", false);
    expect(unexplained.result.status).toBe("awaiting_correction");
    expect(unexplained.worktrees).toHaveLength(1);
    const corrected = await runCase("infrastructure_invalid", true);
    expect(corrected.result).toMatchObject({ status: "completed", classification: "not_reproduced" });
    expect(corrected.worktrees).toHaveLength(2);
  });

  it("recovers a durable attempt result without creating a duplicate attempt", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    const crash = Object.assign(new Error("simulated crash"), { simulatedCrash: true });
    await expect(startSession(
      { ...input, processAttested: true },
      { ...value.dependencies, afterAttemptObservedFn: async () => { throw crash; } },
    )).rejects.toThrow("simulated crash");
    const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
    const recovered = await resumeSession({ sessionDir, processAttested: true }, value.dependencies);
    expect(recovered).toMatchObject({ status: "completed", classification: "not_reproduced" });
    expect(value.worktrees).toHaveLength(1);
  });

  it("pins a pre-recovery result path and digest across a crash inside recovery", async () => {
    const input = await paths();
    const unproven = (spec: Record<string, string>) => ({
      ...attempt("valid")(spec),
      finalState: null,
    });
    const value = fake([unproven]);
    await expect(startSession(
      { ...input, processAttested: true },
      {
        ...value.dependencies,
        afterAttemptObservedFn: async () => {
          throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
        },
      },
    )).rejects.toThrow("simulated crash");
    const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
    let recoveryCrash = false;
    await expect(resumeSession(
      { sessionDir, processAttested: true },
      {
        ...value.dependencies,
        afterDurableWriteFn: async ({ value: event }: { value: { type?: string } }) => {
          if (!recoveryCrash && event.type === "attempt_recovery_started") {
            recoveryCrash = true;
            throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
          }
        },
      },
    )).rejects.toThrow("simulated crash");
    const recovered = await resumeSession({ sessionDir, processAttested: true }, value.dependencies);
    expect(recovered.status).toBe("awaiting_correction");
    const events = (await readdir(path.join(sessionDir, "ledger"))).sort();
    const ledger = await Promise.all(events.map((name) =>
      readFile(path.join(sessionDir, "ledger", name), "utf8").then(JSON.parse),
    ));
    const reservation = ledger.find((event) => event.type === "attempt_recovery_started");
    expect(reservation).toMatchObject({
      sourceResultPath: path.join(sessionDir, "attempts", "attempt-001", "attempt-result.json"),
      sourceResultSha256: expect.stringMatching(/^[0-9a-f]{64}$/),
    });
    expect(JSON.parse(await readFile(recovered.attempts[0].resultPath, "utf8"))).toMatchObject({
      reasons: ["final_restore_evidence_missing"],
      sourceResultPath: reservation.sourceResultPath,
    });
  });

  it("allows only a completed nonzero optional environment command", () => {
    expect(() => assertControlCommandResult(
      { classification: "command_failed", closeObserved: true, timedOut: false, exitCode: 1 },
      "environment-defender",
      true,
    )).not.toThrow();
    for (const { result, expectedKind } of [
      {
        result: { classification: "timeout", closeObserved: true, timedOut: true },
        expectedKind: "command_timeout",
      },
      {
        result: { classification: "termination_unconfirmed", closeObserved: false, timedOut: true },
        expectedKind: "termination_unconfirmed",
      },
    ]) {
      let thrown: unknown = null;
      try {
        assertControlCommandResult(result, "environment-defender", true);
      } catch (error) {
        thrown = error;
      }
      expect(thrown).toMatchObject({ kind: expectedKind });
    }
  });

  it("halts after unconfirmed termination and recovers only on a freshly attested resume", async () => {
    const input = await paths();
    const value = fake([]);
    let captureCount = 0;
    let restoreCount = 0;
    let verifyCount = 0;
    const baseVerify = (value.dependencies as Record<string, unknown>).verifyFrozenProtocolFn as
      | ((input: { repoRoot: string }) => Promise<Record<string, unknown>>)
      | undefined;
    const captureEnvironmentFn = async () => {
      captureCount += 1;
      return value.dependencies.captureEnvironmentFn();
    };
    const restoreAttemptWorktreeFn = async () => {
      restoreCount += 1;
      return { kind: "A", srcTauriTree: "a-tree" };
    };
    const runAttemptFn = async (spec: Record<string, string>) => {
      const result = {
        ...attempt("infrastructure_invalid")(spec),
        finalState: null,
      };
      await writeFile(
        path.join(spec.sessionDir, "attempts", spec.attemptId, "attempt-result.json"),
        `${JSON.stringify(result, null, 2)}\n`,
        { encoding: "utf8", flag: "wx" },
      );
      throw Object.assign(new Error("owned process tree may still be alive"), {
        kind: "termination_unconfirmed",
        details: { operatorActionRequired: true, attemptResult: result },
      });
    };
    const deps = {
      ...value.dependencies,
      captureEnvironmentFn,
      restoreAttemptWorktreeFn,
      runAttemptFn,
      ...(baseVerify
        ? {
            verifyFrozenProtocolFn: async (input: { repoRoot: string }) => {
              verifyCount += 1;
              return baseVerify(input);
            },
          }
        : {}),
    };
    await expect(startSession({ ...input, processAttested: true }, deps)).rejects.toMatchObject({
      kind: "termination_unconfirmed",
    });
    expect(captureCount).toBe(2);
    expect(restoreCount).toBe(0);
    const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
    const verifyCountBeforeRejectedResume = verifyCount;
    await expect(resumeSession({ sessionDir }, deps)).rejects.toMatchObject({
      kind: "process_attestation_missing",
    });
    expect(verifyCount).toBe(verifyCountBeforeRejectedResume);
    expect(captureCount).toBe(2);
    expect(restoreCount).toBe(0);
    const recovered = await resumeSession({ sessionDir, processAttested: true }, deps);
    expect(recovered.status).toBe("awaiting_correction");
    expect(captureCount).toBe(3);
    expect(restoreCount).toBe(1);
    expect(value.worktrees).toHaveLength(1);
  });

  it("replays one durable retry disposition without consuming it twice", async () => {
    const input = await paths();
    const value = fake([attempt("stability_invalid"), attempt("valid")]);
    const first = await startSession({ ...input, processAttested: true }, value.dependencies);
    let crashed = false;
    await expect(resumeSession(
      { sessionDir: first.sessionDir, unexplainedStability: true, processAttested: true },
      {
        ...value.dependencies,
        afterDurableWriteFn: async ({ value: event }: { value: { type?: string } }) => {
          if (!crashed && event.type === "retry_disposition") {
            crashed = true;
            throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
          }
        },
      },
    )).rejects.toThrow("simulated crash");
    const recovered = await resumeSession(
      { sessionDir: first.sessionDir, processAttested: true },
      value.dependencies,
    );
    expect(recovered).toMatchObject({
      status: "completed",
      retryState: { unexplainedStabilityInvalidCount: 1, terminal: true },
    });
    expect(value.worktrees).toHaveLength(2);
  });

  it("materializes missing terminal projections across both publication windows", async () => {
    for (const point of ["session_completed", "session-ledger.json"]) {
      const input = await paths();
      const value = fake([attempt("valid")]);
      let crashed = false;
      await expect(startSession(
        { ...input, processAttested: true },
        {
          ...value.dependencies,
          afterDurableWriteFn: async ({
            target,
            value: event,
          }: { target: string; value: { type?: string } }) => {
            if (!crashed && (event.type === point || target.endsWith(point))) {
              crashed = true;
              throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
            }
          },
        },
      )).rejects.toThrow("simulated crash");
      const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
      const recovered = await resumeSession({ sessionDir, processAttested: true }, value.dependencies);
      expect(recovered.status).toBe("completed");
      expect(JSON.parse(await readFile(path.join(sessionDir, "session-ledger.json"), "utf8"))).toMatchObject({
        terminal: true,
      });
      expect(JSON.parse(await readFile(path.join(sessionDir, "decision.json"), "utf8"))).toMatchObject({
        attemptId: "attempt-001",
      });
    }
  });

  it.each(["session-manifest.json", "process-shell-diagnostic.locator.json", "session_started"])(
    "recovers bootstrap crash after %s",
    async (point) => {
      const input = await paths();
      const value = fake([attempt("valid")]);
      let crashed = false;
      await expect(startSession(
        { ...input, processAttested: true },
        {
          ...value.dependencies,
          afterDurableWriteFn: async ({ target, value: event }: { target: string; value: { type?: string } }) => {
            if (!crashed && (target.endsWith(point) || event.type === point)) {
              crashed = true;
              throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
            }
          },
        },
      )).rejects.toThrow("simulated crash");
      const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
      const recovered = await resumeSession(
        { sessionDir, processAttested: true },
        value.dependencies,
      );
      expect(recovered.status).toBe("completed");
      expect(value.worktrees).toHaveLength(1);
    },
  );

  it("retries bootstrap materialization in a new artifact directory after two crashes", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    let locatorCrash = false;
    await expect(startSession(
      { ...input, processAttested: true },
      {
        ...value.dependencies,
        afterDurableWriteFn: async ({ target }: { target: string }) => {
          if (!locatorCrash && target.endsWith("process-shell-diagnostic.locator.json")) {
            locatorCrash = true;
            throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
          }
        },
      },
    )).rejects.toThrow("simulated crash");
    const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
    const baseCapture = value.dependencies.captureEnvironmentFn;
    let bootstrapCrashes = 0;
    const captureEnvironmentFn = async (spec: { artifactDir: string }) => {
      if (path.basename(spec.artifactDir).startsWith("bootstrap-recovery-")) {
        await mkdir(spec.artifactDir, { recursive: true });
        if (bootstrapCrashes < 2) {
          bootstrapCrashes += 1;
          throw new Error(`bootstrap capture crash ${bootstrapCrashes}`);
        }
      }
      return baseCapture();
    };
    await expect(resumeSession(
      { sessionDir, processAttested: true },
      { ...value.dependencies, captureEnvironmentFn },
    )).rejects.toThrow("bootstrap capture crash 1");
    await expect(resumeSession(
      { sessionDir, processAttested: true },
      { ...value.dependencies, captureEnvironmentFn },
    )).rejects.toThrow("bootstrap capture crash 2");
    const recovered = await resumeSession(
      { sessionDir, processAttested: true },
      { ...value.dependencies, captureEnvironmentFn },
    );
    expect(recovered.status).toBe("completed");
    expect((await readdir(sessionDir)).filter((name) => name.startsWith("bootstrap-recovery-"))).toHaveLength(3);
  });

  it("recovers an attempt reservation committed before its directory exists", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    let crashed = false;
    await expect(startSession(
      { ...input, processAttested: true },
      {
        ...value.dependencies,
        afterDurableWriteFn: async ({ value: event }: { value: { type?: string } }) => {
          if (!crashed && event.type === "attempt_started") {
            crashed = true;
            throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
          }
        },
      },
    )).rejects.toThrow("simulated crash");
    const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
    const recovered = await resumeSession({ sessionDir, processAttested: true }, value.dependencies);
    expect(recovered.status).toBe("awaiting_correction");
    expect(recovered.attempts[0].reasons).toEqual(["coordinator_interrupted"]);
    expect(value.worktrees).toHaveLength(0);
  });

  it("replays a durable coordinator failure before any normal result", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    let crashed = false;
    await expect(startSession(
      { ...input, processAttested: true },
      {
        ...value.dependencies,
        createDetachedWorktreeFn: async () => { throw new Error("forced worktree failure"); },
        afterDurableWriteFn: async ({ target }: { target: string }) => {
          if (!crashed && target.endsWith("coordinator-failure.json")) {
            crashed = true;
            throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
          }
        },
      },
    )).rejects.toThrow("simulated crash");
    const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
    const recovered = await resumeSession({ sessionDir, processAttested: true }, value.dependencies);
    expect(recovered.status).toBe("awaiting_correction");
    expect(recovered.attempts[0].reasons).toEqual(["coordinator_failure"]);
  });

  it("replays an interruption published after recovery and before attempt_finished", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    let reservationCrash = false;
    await expect(startSession(
      { ...input, processAttested: true },
      {
        ...value.dependencies,
        afterDurableWriteFn: async ({ value: event }: { value: { type?: string } }) => {
          if (!reservationCrash && event.type === "attempt_started") {
            reservationCrash = true;
            throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
          }
        },
      },
    )).rejects.toThrow("simulated crash");
    const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
    let interruptionCrash = false;
    await expect(resumeSession(
      { sessionDir, processAttested: true },
      {
        ...value.dependencies,
        afterDurableWriteFn: async ({ target }: { target: string }) => {
          if (!interruptionCrash && target.endsWith("coordinator-interruption.json")) {
            interruptionCrash = true;
            throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
          }
        },
      },
    )).rejects.toThrow("simulated crash");
    const recovered = await resumeSession({ sessionDir, processAttested: true }, value.dependencies);
    expect(recovered.status).toBe("awaiting_correction");
    expect(recovered.attempts).toHaveLength(1);
  });

  it("abandons a late normal result after recovery_started and completes a new recovery", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    let reservationCrash = false;
    await expect(startSession(
      { ...input, processAttested: true },
      {
        ...value.dependencies,
        afterDurableWriteFn: async ({ value: event }: { value: { type?: string } }) => {
          if (!reservationCrash && event.type === "attempt_started") {
            reservationCrash = true;
            throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
          }
        },
      },
    )).rejects.toThrow("simulated crash");
    const sessionDir = path.join(input.scratchParent, "process-shell-session-session-fixed");
    let recoveryCrash = false;
    await expect(resumeSession(
      { sessionDir, processAttested: true },
      {
        ...value.dependencies,
        afterDurableWriteFn: async ({ value: event }: { value: { type?: string } }) => {
          if (!recoveryCrash && event.type === "attempt_recovery_started") {
            recoveryCrash = true;
            throw Object.assign(new Error("simulated crash"), { simulatedCrash: true });
          }
        },
      },
    )).rejects.toThrow("simulated crash");
    const lateResultPath = path.join(sessionDir, "attempts", "attempt-001", "attempt-result.json");
    await mkdir(path.dirname(lateResultPath), { recursive: true });
    await writeFile(lateResultPath, `${JSON.stringify(attempt("valid")({
      attemptId: "attempt-001",
      worktree: path.join(input.mainRoot, ".worktrees", "late"),
    }), null, 2)}\n`, { encoding: "utf8", flag: "wx" });
    const recovered = await resumeSession({ sessionDir, processAttested: true }, value.dependencies);
    expect(recovered.status).toBe("awaiting_correction");
    expect(recovered.attempts[0].reasons).toEqual(["coordinator_interrupted"]);
  });

  it("points coordinator failure rows at their real immutable artifact", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    const result = await startSession(
      { ...input, processAttested: true },
      {
        ...value.dependencies,
        createDetachedWorktreeFn: async () => {
          const error = Object.assign(new Error("forced worktree failure"), { kind: "worktree_create_failed" });
          throw error;
        },
      },
    );
    expect(result.status).toBe("awaiting_correction");
    const finished = result.attempts[0];
    expect(finished.resultPath).toMatch(/coordinator-failure\.json$/);
    expect(JSON.parse(await readFile(finished.resultPath, "utf8"))).toMatchObject({
      kind: "infrastructure_invalid",
      reasons: ["coordinator_failure"],
    });
  });

  it("content-hashes the complete main target tree, not only its root timestamp", async () => {
    const root = await mkdtemp(path.join(os.tmpdir(), "extractum-target-snapshot-"));
    await mkdir(path.join(root, "debug", "incremental"), { recursive: true });
    const artifact = path.join(root, "debug", "incremental", "unit.bin");
    await writeFile(artifact, "before", "utf8");
    const before = await snapshotDirectory(root);
    await writeFile(artifact, "after!", "utf8");
    const after = await snapshotDirectory(root);
    expect(before.digest).not.toBe(after.digest);
    expect(before.records.map((record) => record.path)).toContain("debug/incremental/unit.bin");
  });

  it("rejects shared-target environment before creating a worktree", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    await expect(startSession(
      { ...input, processAttested: true },
      { ...value.dependencies, processEnv: { cargo_target_dir: "G:\\shared" } },
    )).rejects.toMatchObject({ kind: "cargo_target_dir_set" });
    expect(value.worktrees).toHaveLength(0);
  });

  it("requires an explicit operator quiescence attestation", async () => {
    const input = await paths();
    const value = fake([attempt("valid")]);
    await expect(startSession(input, value.dependencies)).rejects.toMatchObject({
      kind: "process_attestation_missing",
    });
  });

  it("parses boolean and value CLI arguments without ambiguity", () => {
    expect(parseCli([
      "start", "--main-root", "G:\\main", "--protocol-root", "G:\\protocol",
      "--scratch-parent", "G:\\scratch", "--process-attested",
    ])).toEqual({
      command: "start",
      options: {
        mainRoot: "G:\\main",
        protocolRoot: "G:\\protocol",
        scratchParent: "G:\\scratch",
        processAttested: true,
      },
    });
    expect(parseCli([
      "resume", "--session-dir", "G:\\scratch\\session", "--unexplained-stability", "--process-attested",
    ])).toEqual({
      command: "resume",
      options: { sessionDir: "G:\\scratch\\session", unexplainedStability: true, processAttested: true },
    });
  });
});
