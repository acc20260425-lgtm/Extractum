import { access, link, mkdtemp, readFile, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";

import { assertProtocolWorktreeStatus } from "./freeze.mjs";
import {
  assertProtocolPinMatches,
  assertRetryProtocol,
  cleanupOwnedAtomicTemps,
  deriveLedgerProjection,
  publishReportPair,
  renderVerification,
  verifyReportProtocol,
} from "./report.mjs";
import { writeAtomicBytesExclusive } from "./runtime.mjs";

const samples = (value: number) => Array(7).fill(value);
const summary = (value: number) => ({
  samplesMs: samples(value),
  medianMs: value,
  samplesWithinBand: 7,
  stable: true,
});
const metric = (variant: number, reference: number) => {
  const deltaMs = variant - reference;
  return {
    variantMedianMs: variant,
    aReferenceMs: reference,
    deltaMs,
    percentDelta: 100 * deltaMs / reference,
    material: deltaMs >= 500,
    shellCapFailed: deltaMs > 500 || (100 * deltaMs / reference) > 5,
  };
};
const block = (value: number, extractumProcessExtern = false) => ({
  samples: samples(value).map((wallMs, index) => ({
    index: index + 1,
    wallMs,
    cargoReportedMs: wallMs - 20,
    checkedPackages: ["extractum"],
  })),
  summary: summary(value),
  noOp: { elapsedMs: 80, cargoReportedMs: 60 },
  diagnostic: {
    extractumProcessExtern,
    timingArtifact: { path: `timing-${value}.html`, sha256: "f".repeat(64) },
  },
});

const sessionManifest = {
  schemaVersion: 1,
  sessionId: "session-fixed",
  createdAt: "2026-07-18T08:00:00.000Z",
  sessionDir: "G:\\raw\\session-fixed",
  protocolRoot: "G:\\protocol",
  protocol: {
    protocolCommit: "1".repeat(40),
    lockPath: "scripts/process-shell-diagnostic/protocol-lock.json",
    lockBlob: "2".repeat(40),
    lockSha256: "3".repeat(64),
    protocolVersion: 1,
  },
  protocolLock: { schemaVersion: 1, protocolVersion: 1, states: {} },
  environment: {
    platform: "win32",
    host: "x86_64-pc-windows-msvc",
    cargo: "cargo 1.95.0",
    rustc: "rustc 1.95.0",
    power: "Balanced",
    defender: "unavailable: Access denied",
    mainTargetDirectory: "G:\\Develop\\Extractum\\src-tauri\\target",
  },
};

const evaluation = {
  kind: "valid",
  classification: "boundary_composite",
  eRequired: true,
  anchorRangeMs: 0,
  disagreement: [],
  summaries: {
    A0: summary(9_000), B: summary(9_100), A1: summary(9_000),
    C: summary(9_200), A2: summary(9_000), D: summary(9_800),
    A3: summary(9_000), E: summary(9_250), A4: summary(9_000),
  },
  metrics: {
    B: metric(9_100, 9_000),
    C: metric(9_200, 9_000),
    D: metric(9_800, 9_000),
    E: metric(9_250, 9_000),
  },
  contrasts: {
    membershipMs: 100,
    edgeAfterMembershipMs: 100,
    manifestAfterCMs: 50,
    dSpecificCompositeMs: 550,
    dAfterCCompositeMs: null,
  },
};

const causal = {
  sessionManifest,
  measurementEnvironment: {
    ...sessionManifest.environment,
    power: "High performance",
  },
  ledger: {
    schemaVersion: 1,
    sessionId: "session-fixed",
    unexplainedStabilityInvalidCount: 0,
    terminal: true,
    attempts: [{
      attemptId: "attempt-001",
      status: "valid",
      startedAt: "2026-07-18T08:01:00.000Z",
      endedAt: "2026-07-18T09:18:00.000Z",
      reasons: [],
    }],
  },
  decision: {
    schemaVersion: 1,
    classification: "boundary_composite",
    attemptId: "attempt-001",
    unexplainedStabilityInvalidCount: 0,
    evaluation,
  },
  attemptResults: [{
    attemptId: "attempt-001",
    kind: "valid",
    evaluation: structuredClone(evaluation),
    blocks: Object.fromEntries(
      Object.entries(evaluation.summaries).map(([name, value]) => [
        name,
        block(value.medianMs, ["C", "D", "E"].includes(name)),
      ]),
    ),
  }],
  artifactIndex: { sha256: "4".repeat(64), files: 211, bytes: 123_456 },
};

describe("process shell diagnostic report", () => {
  it("renders causal raw evidence, no-op timings, and cumulative contrasts", () => {
    const report = renderVerification(causal);
    expect(report).toContain("**Outcome:** `boundary_composite`");
    expect(report).toContain("**Protocol-lock blob:** `2222222222222222222222222222222222222222`");
    expect(report).toContain("| D | 9800, 9800, 9800, 9800, 9800, 9800, 9800 | 9800 ms | 7/7 | 80 ms | 60 ms |");
    expect(report).toContain("| D | 9780, 9780, 9780, 9780, 9780, 9780, 9780 | true |");
    expect(report).toContain("| D-specific composite | +550 ms |");
    expect(report).toContain("descriptive contrasts between cumulative configurations");
    expect(report).toContain("does not automatically retain `extractum-process` or unblock Phase 4");
    expect(report).toContain("High performance");
    expect(report).not.toContain("| power | Balanced |");
    expect(report).toContain("## Retry and invalidation audit");
    expect(report).toContain("### Attempt error details");
    expect(report).toContain("### Corrected environment deltas");
  });

  it("rejects a recorded decision that disagrees with independent arithmetic", () => {
    const corrupted = structuredClone(causal);
    corrupted.decision.evaluation.metrics.D.deltaMs = 799;
    expect(() => renderVerification(corrupted)).toThrow("independent recalculation mismatch");
  });

  it("rejects a decision detached from the raw attempt samples", () => {
    const corrupted = structuredClone(causal);
    for (const sample of corrupted.attemptResults[0].blocks.D.samples) sample.wallMs = 9_100;
    expect(() => renderVerification(corrupted)).toThrow("raw attempt evidence mismatch");
  });

  it("derives the aggregate from numbered events and rejects an attempt after valid", () => {
    const retryState = { unexplainedStabilityInvalidCount: 0, terminal: false };
    const finished = causal.ledger.attempts[0];
    const events = [
      { schemaVersion: 1, sequence: 1, type: "session_started", sessionId: "session-fixed", retryState },
      {
        schemaVersion: 1,
        sequence: 2,
        type: "attempt_started",
        attemptId: "attempt-001",
        retryState,
      },
      {
        schemaVersion: 1,
        sequence: 3,
        type: "attempt_finished",
        attemptId: "attempt-001",
        kind: "valid",
        startedAt: finished.startedAt,
        endedAt: finished.endedAt,
        reasons: [],
        worktree: "G:\\attempt-001",
        targetDirectory: "G:\\attempt-001\\src-tauri\\target",
        environment: causal.measurementEnvironment,
        resultPath: "G:\\raw\\attempt-001.json",
        classification: "boundary_composite",
        retryState,
      },
      {
        schemaVersion: 1,
        sequence: 4,
        type: "session_completed",
        classification: "boundary_composite",
        retryState: { unexplainedStabilityInvalidCount: 0, terminal: true },
        decision: causal.decision,
      },
    ];
    const derived = deriveLedgerProjection("session-fixed", events);
    expect(derived).toMatchObject({ terminal: true, attempts: [{ attemptId: "attempt-001", status: "valid" }] });
    expect(() => assertRetryProtocol({
      events,
      attemptResults: causal.attemptResults,
      decision: causal.decision,
    })).not.toThrow();
    const illegal = structuredClone(events);
    illegal.splice(3, 0, {
      schemaVersion: 1,
      sequence: 4,
      type: "attempt_started",
      attemptId: "attempt-002",
      retryState,
    });
    illegal[4].sequence = 5;
    expect(() => assertRetryProtocol({
      events: illegal,
      attemptResults: causal.attemptResults,
      decision: causal.decision,
    })).toThrow("retry protocol replay mismatch");
    const beforeSessionStart = [events[1], events[0], ...events.slice(2)];
    expect(() => deriveLedgerProjection("session-fixed", beforeSessionStart)).toThrow(
      "session_started must be first",
    );
    expect(() => assertRetryProtocol({
      events: beforeSessionStart,
      attemptResults: causal.attemptResults,
      decision: causal.decision,
    })).toThrow("event before session_started");
  });

  it("renders terminal environment precision without a causal claim", () => {
    const value = structuredClone(causal);
    value.ledger.unexplainedStabilityInvalidCount = 2;
    const invalidBlocks = () => ({
      A0: block(9_000), B: block(9_100), A1: block(9_400),
      C: block(9_200, true), A2: block(9_000), D: block(9_300, true), A3: block(9_000),
    });
    value.ledger.attempts = [
      { attemptId: "attempt-001", status: "stability_invalid", startedAt: "08:00", endedAt: "09:00", reasons: ["anchor_range_exceeded"], resultPath: "attempt-001.json" },
      { attemptId: "attempt-002", status: "stability_invalid", startedAt: "09:10", endedAt: "10:10", reasons: ["anchor_range_exceeded"], resultPath: "attempt-002.json" },
    ];
    value.ledger.events = [
      { type: "retry_disposition", attemptId: "attempt-001", unexplainedStability: true, retryAction: "retry" },
      { type: "retry_disposition", attemptId: "attempt-002", unexplainedStability: true, retryAction: "environment_precision_insufficient" },
    ];
    value.decision = {
      schemaVersion: 1,
      classification: "environment_precision_insufficient",
      attemptId: null,
      unexplainedStabilityInvalidCount: 2,
      evaluation: null,
    };
    value.attemptResults = value.ledger.attempts.map((attempt) => ({
      attemptId: attempt.attemptId,
      kind: "stability_invalid",
      reasons: ["anchor_range_exceeded"],
      blocks: invalidBlocks(),
      evaluation: {
        kind: "stability_invalid",
        eRequired: false,
        anchorRangeMs: 400,
        reasons: ["anchor_range_exceeded"],
        summaries: Object.fromEntries(Object.entries(invalidBlocks()).map(([name, value]) => [name, value.summary])),
      },
    }));
    const report = renderVerification(value);
    expect(report).toContain("No B/C/D/E causal classification is made.");
    expect(report).toContain("**Unexplained stability-invalid count:** 2");
    expect(report).toContain("## Attempt attempt-001 raw measurements");
    expect(report).toContain("## Attempt attempt-002 raw measurements");
    expect(report).not.toContain("## Variant metrics");
  });

  it("is deterministic and uses only recorded timestamps", () => {
    expect(renderVerification(structuredClone(causal))).toBe(renderVerification(causal));
  });

  it("rejects a reporter-time protocol pin mismatch before publication", () => {
    const verified = {
      ...sessionManifest.protocol,
      protocolLock: structuredClone(sessionManifest.protocolLock),
    };
    expect(() => assertProtocolPinMatches(sessionManifest, verified)).not.toThrow();
    expect(() => assertProtocolPinMatches(sessionManifest, {
      ...verified,
      lockSha256: "9".repeat(64),
    })).toThrow("report protocol pin mismatch");
  });

  it("allows only its output and replays index-link and between-publication crashes", async () => {
    const outputRelative = "docs/superpowers/verification/2026-07-18-process-shell-regression-diagnostic.md";
    assertProtocolWorktreeStatus(`?? ${outputRelative}`, [outputRelative]);
    for (const status of [` M ${outputRelative}`, "?? unrelated.txt"]) {
      let thrown: unknown = null;
      try {
        assertProtocolWorktreeStatus(status, [outputRelative]);
      } catch (error) {
        thrown = error;
      }
      expect(thrown).toMatchObject({ kind: "protocol_worktree_dirty" });
    }

    let verifyInput: Record<string, unknown> | null = null;
    const verified = {
      ...sessionManifest.protocol,
      protocolLock: structuredClone(sessionManifest.protocolLock),
    };
    await verifyReportProtocol({
      sessionManifest,
      output: path.join(sessionManifest.protocolRoot, ...outputRelative.split("/")),
      runningProtocolRoot: sessionManifest.protocolRoot,
      verifyFn: async (input: Record<string, unknown>) => {
        verifyInput = input;
        return verified;
      },
    });
    expect(verifyInput).toEqual({
      repoRoot: sessionManifest.protocolRoot,
      allowedUntrackedPaths: [outputRelative],
    });
    let arbitraryOutputReachedGit = false;
    await expect(verifyReportProtocol({
      sessionManifest,
      output: path.join(sessionManifest.protocolRoot, "unrelated", "important.txt"),
      runningProtocolRoot: sessionManifest.protocolRoot,
      verifyFn: async () => {
        arbitraryOutputReachedGit = true;
        return verified;
      },
    })).rejects.toThrow("report output must equal");
    expect(arbitraryOutputReachedGit).toBe(false);

    const root = await mkdtemp(path.join(os.tmpdir(), "extractum-report-replay-"));
    const artifactIndex = {
      path: path.join(root, "artifact-index.json"),
      content: Buffer.from("index\n"),
    };
    const output = path.join(root, "verification.md");
    const reportBytes = Buffer.from("report\n");
    const staleTemp = `${output}.4242.123e4567-e89b-12d3-a456-426614174000.tmp`;
    const unrelated = `${output}.not-owned.tmp`;
    await writeFile(staleTemp, "partial", "utf8");
    await writeFile(unrelated, "keep", "utf8");
    await cleanupOwnedAtomicTemps(output, { processAliveFn: () => false });
    await expect(access(staleTemp)).rejects.toMatchObject({ code: "ENOENT" });
    await expect(access(unrelated)).resolves.toBeUndefined();

    // Simulate a hard kill after the atomic index temp was linked to its final
    // target but before the writer unlinked the sibling temp.
    const strandedIndexTemp = `${artifactIndex.path}.4242.123e4567-e89b-12d3-a456-426614174001.tmp`;
    await writeFile(strandedIndexTemp, artifactIndex.content);
    await link(strandedIndexTemp, artifactIndex.path);
    await cleanupOwnedAtomicTemps(artifactIndex.path, { processAliveFn: () => false });
    await expect(access(strandedIndexTemp)).rejects.toMatchObject({ code: "ENOENT" });
    await publishReportPair({ artifactIndex, output, reportBytes });
    expect(await readFile(artifactIndex.path)).toEqual(artifactIndex.content);
    expect(await readFile(output)).toEqual(reportBytes);

    const peerRoot = await mkdtemp(path.join(os.tmpdir(), "extractum-report-peer-replay-"));
    const peerArtifactIndex = {
      path: path.join(peerRoot, "artifact-index.json"),
      content: artifactIndex.content,
    };
    const peerOutput = path.join(peerRoot, "verification.md");
    let crashed = false;
    await expect(publishReportPair({
      artifactIndex: peerArtifactIndex,
      output: peerOutput,
      reportBytes,
    }, async (target: string, bytes: Buffer) => {
      await writeAtomicBytesExclusive(target, bytes);
      if (!crashed) {
        crashed = true;
        throw new Error("simulated publication crash");
      }
    })).rejects.toThrow("simulated publication crash");
    await publishReportPair({ artifactIndex: peerArtifactIndex, output: peerOutput, reportBytes });
    expect(await readFile(peerArtifactIndex.path)).toEqual(peerArtifactIndex.content);
    expect(await readFile(peerOutput)).toEqual(reportBytes);
  });
});
