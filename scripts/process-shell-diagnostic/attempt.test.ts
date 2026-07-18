import path from "node:path";
import { describe, expect, it } from "vitest";

import { evaluateAttempt } from "./protocol.mjs";
import { runAttempt } from "./attempt.mjs";

function fixture(overrides: Record<string, unknown> = {}) {
  const calls: Array<Record<string, unknown>> = [];
  const writes: Array<{ target: string; value: Record<string, unknown> }> = [];
  const stateValues: Record<string, number> = {
    A0: 9_000,
    B: 9_000,
    A1: 9_000,
    C: 9_000,
    A2: 9_000,
    D: 9_000,
    A3: 9_000,
    E: 9_000,
    A4: 9_000,
  };
  Object.assign(stateValues, overrides.stateValues ?? {});
  let currentBlock = "A0";

  const deps = {
    installStateFn: async ({ state }: { state: string }) => {
      currentBlock = state === "A-final" ? currentBlock : state;
      calls.push({ kind: "install", state });
      return {
        state,
        kind: state.startsWith("A") ? "A" : state,
        srcTauriTree: state.startsWith("A") ? "a-tree" : `${state}-tree`,
        canonicalLibSha256: `${state}-sha`,
      };
    },
    captureStateInventoryFn: async ({ block }: { block: string }) => {
      calls.push({ kind: "inventory", block });
      return {
        metadata: { target_directory: `G:\\attempt\\src-tauri\\target` },
        extractumProcessDirectDependency: ["C", "D", "E"].includes(block),
      };
    },
    verifyTargetPreflightFn: async ({ block }: { block: string }) => {
      calls.push({ kind: "target-preflight", block });
      return { targetDirectory: "G:\\attempt\\src-tauri\\target" };
    },
    runCargoCheckFn: async ({ label }: { label: string }) => {
      calls.push({ kind: "cargo", label });
      return overrides.cargoResult ?? {
        classification: "ok",
        elapsedMs: 50,
        cargoReportedMs: 40,
        closeObserved: true,
      };
    },
    runDirtyProbeFn: async ({ label, diagnostic }: { label: string; diagnostic: boolean }) => {
      calls.push({ kind: "dirty", label, diagnostic });
      if (overrides.failLabel === label) throw Object.assign(new Error("forced"), { kind: "forced_failure" });
      if (overrides.timeoutLabel === label) {
        throw Object.assign(new Error("timed out"), {
          kind: "cargo_failed",
          details: { dirtyResult: { classification: "timeout", timedOut: true } },
        });
      }
      if (overrides.terminationLabel === label) {
        throw Object.assign(new Error("termination unconfirmed"), {
          kind: "cargo_failed",
          details: { dirtyResult: { classification: "termination_unconfirmed", timedOut: true } },
        });
      }
      return {
        classification: "ok",
        elapsedMs: stateValues[currentBlock],
        cargoReportedMs: stateValues[currentBlock] - 20,
        extractumChecked: overrides.missingCheckedLabel !== label,
        extractumLibRustcObserved: true,
        extractumProcessExtern: ["C", "D", "E"].includes(currentBlock),
        closeObserved: true,
        timingArtifact: diagnostic ? { path: `${label}.html`, sha256: "f".repeat(64) } : null,
      };
    },
    evaluateAttemptFn: evaluateAttempt,
    writeJsonFn: async (target: string, value: Record<string, unknown>) => {
      writes.push({ target, value });
    },
  };
  return { calls, deps, writes };
}

const spec = {
  worktree: "G:\\attempt",
  mainRoot: "G:\\main",
  sessionDir: "G:\\artifacts",
  attemptId: "attempt-001",
  protocolLock: {
    states: {
      A: { srcTauriTree: "a-tree" },
      B: { srcTauriTree: "B-tree" },
      C: { srcTauriTree: "C-tree" },
      D: { srcTauriTree: "D-tree" },
      E: { srcTauriTree: "E-tree" },
    },
  },
};

describe("process shell diagnostic attempt", () => {
  it("runs the fixed seven-block sequence and restores A", async () => {
    const value = fixture();
    const result = await runAttempt(spec, value.deps);
    expect(result.kind).toBe("valid");
    expect(result.evaluation.classification).toBe("not_reproduced");
    expect(value.calls.filter((call) => call.kind === "install").map((call) => call.state)).toEqual([
      "A0", "B", "A1", "C", "A2", "D", "A3", "A-final",
    ]);
    for (const block of ["A0", "B", "A1", "C", "A2", "D", "A3"]) {
      const dirty = value.calls.filter((call) => call.kind === "dirty" && String(call.label).startsWith(`${block}.`));
      expect(dirty).toHaveLength(10);
      expect(dirty.filter((call) => call.diagnostic)).toEqual([
        expect.objectContaining({ label: `${block}.diagnostic`, diagnostic: true }),
      ]);
      expect(value.calls).toEqual(expect.arrayContaining([
        expect.objectContaining({ kind: "cargo", label: `${block}.inventory-sync` }),
        expect.objectContaining({ kind: "cargo", label: `${block}.noop-sync` }),
        expect.objectContaining({ kind: "cargo", label: `${block}.noop` }),
      ]));
      const inventoryIndex = value.calls.findIndex((call) => call.kind === "inventory" && call.block === block);
      const preflightIndex = value.calls.findIndex((call) =>
        call.kind === "target-preflight" && call.block === block,
      );
      const canonicalSyncIndex = value.calls.findIndex((call) =>
        call.kind === "cargo" && call.label === `${block}.inventory-sync`,
      );
      expect(preflightIndex).toBeLessThan(canonicalSyncIndex);
      expect(canonicalSyncIndex).toBeLessThan(inventoryIndex);
      expect(result.blocks[block].inventory.extractumProcessDirectDependency).toBe(["C", "D", "E"].includes(block));
      expect(result.blocks[block].diagnostic.extractumProcessExtern).toBe(["C", "D", "E"].includes(block));
    }
  });

  it("appends E and A4 only after D crosses while B and C stay fast", async () => {
    const value = fixture({ stateValues: { D: 9_600 } });
    const result = await runAttempt(spec, value.deps);
    expect(result.kind).toBe("valid");
    expect(result.evaluation).toMatchObject({ eRequired: true, classification: "boundary_composite" });
    expect(value.calls.filter((call) => call.kind === "install").map((call) => call.state)).toEqual([
      "A0", "B", "A1", "C", "A2", "D", "A3", "E", "A4", "A-final",
    ]);
    expect(result.blocks.E.diagnostic.extractumProcessExtern).toBe(true);
  });

  it("retains all seven samples and never replaces one", async () => {
    const value = fixture();
    const result = await runAttempt(spec, value.deps);
    expect(result.blocks.B.samples.map((sample: { wallMs: number }) => sample.wallMs)).toEqual(Array(7).fill(9_000));
    expect(result.blocks.B.summary.samplesWithinBand).toBe(7);
  });

  it("classifies a command failure as infrastructure-invalid and still restores A", async () => {
    const value = fixture({ failLabel: "C.sample-4" });
    const result = await runAttempt(spec, value.deps);
    expect(result).toMatchObject({ kind: "infrastructure_invalid", reasons: ["protocol_violation"] });
    expect(value.calls.at(-1)).toEqual({ kind: "install", state: "A-final" });
  });

  it("persists a nested dirty-probe timeout as command_timeout", async () => {
    const value = fixture({ timeoutLabel: "C.sample-4" });
    const result = await runAttempt(spec, value.deps);
    expect(result).toMatchObject({ kind: "infrastructure_invalid", reasons: ["command_timeout"] });
    expect(value.calls.at(-1)).toEqual({ kind: "install", state: "A-final" });
  });

  it("persists then halts without A-final install after unconfirmed termination", async () => {
    const value = fixture({ terminationLabel: "C.sample-4" });
    await expect(runAttempt(spec, value.deps)).rejects.toMatchObject({
      kind: "termination_unconfirmed",
      details: { operatorActionRequired: true },
    });
    expect(value.calls.at(-1)).not.toEqual({ kind: "install", state: "A-final" });
    expect(value.writes.at(-1)).toMatchObject({
      target: path.join("G:\\artifacts", "attempts", "attempt-001", "attempt-result.json"),
      value: {
        kind: "infrastructure_invalid",
        reasons: ["command_timeout"],
        finalState: null,
      },
    });
  });

  it("preserves the termination sentinel when attempt-result publication fails", async () => {
    const value = fixture({ terminationLabel: "C.sample-4" });
    const baseWrite = value.deps.writeJsonFn;
    value.deps.writeJsonFn = async (target: string, result: Record<string, unknown>) => {
      if (target.endsWith("attempt-result.json")) throw new Error("attempt artifact unavailable");
      return baseWrite(target, result);
    };
    await expect(runAttempt(spec, value.deps)).rejects.toMatchObject({
      kind: "termination_unconfirmed",
      details: {
        operatorActionRequired: true,
        persistenceError: { message: "attempt artifact unavailable" },
      },
    });
    expect(value.calls.at(-1)).not.toEqual({ kind: "install", state: "A-final" });
  });

  it("persists a missing extractum checked unit as metadata_invalid", async () => {
    const value = fixture({ missingCheckedLabel: "C.sample-4" });
    const result = await runAttempt(spec, value.deps);
    expect(result).toMatchObject({ kind: "infrastructure_invalid", reasons: ["metadata_invalid"] });
  });

  it.each([
    { classification: "timeout", commandResult: { classification: "timeout", timedOut: true }, reason: "command_timeout" },
    { classification: "command_failed", commandResult: { classification: "command_failed", timedOut: false }, reason: "command_failed" },
  ])("classifies a $classification Cargo result as $reason", async ({ commandResult, reason }) => {
    const value = fixture({ cargoResult: commandResult });
    const result = await runAttempt(spec, value.deps);
    expect(result).toMatchObject({ kind: "infrastructure_invalid", reasons: [reason] });
  });

  it("overrides any prior result if final A restoration fails", async () => {
    const value = fixture();
    const original = value.deps.installStateFn;
    value.deps.installStateFn = async (input: { state: string }) => {
      if (input.state === "A-final") throw Object.assign(new Error("restore"), { kind: "final_restore_failed" });
      return original(input);
    };
    const result = await runAttempt(spec, value.deps);
    expect(result).toMatchObject({ kind: "infrastructure_invalid", reasons: ["restore_failed"] });
  });

  it("writes one terminal attempt result beneath the numbered attempt", async () => {
    const value = fixture();
    await runAttempt(spec, value.deps);
    expect(value.writes).toHaveLength(8);
    expect(value.writes.at(-1)?.target).toBe(path.join("G:\\artifacts", "attempts", "attempt-001", "attempt-result.json"));
  });
});
