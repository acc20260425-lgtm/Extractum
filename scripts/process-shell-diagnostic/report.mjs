import { createHash } from "node:crypto";
import { lstat, readFile, readdir, unlink } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath, pathToFileURL } from "node:url";
import { isDeepStrictEqual } from "node:util";

import { snapshotDirectory } from "./coordinator.mjs";
import { verifyFrozenProtocol } from "./freeze.mjs";
import { sha256File, writeAtomicBytesExclusive } from "./runtime.mjs";

const ORDER = ["A0", "B", "A1", "C", "A2", "D", "A3", "E", "A4"];
export const REPORT_PATH = "docs/superpowers/verification/2026-07-18-process-shell-regression-diagnostic.md";
const RUNNING_PROTOCOL_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const INTERPRETATION = {
  membership_configuration: "The membership-only configuration is sufficient to reproduce the material tax; threshold policy needs an explicit owner decision.",
  edge_related_configuration: "The first crossing occurs after the app edge; that edge or its interaction with membership is implicated.",
  manifest_related: "E reproduces D, implicating the manifest migration, feature unification, declarations, or their interaction with C.",
  boundary_composite: "D reproduces the effect while E does not; the remaining concrete boundary/facade/ownership composite is implicated.",
  not_reproduced: "The original regression is not reproduced; a separately preregistered direct A/D confirmation is required.",
  threshold_disagreement: "The absolute diagnostic rule and existing dual shell cap disagree; this is an anomaly with no roadmap decision.",
};
const REQUIRED_NEXT_STEP = {
  membership_configuration: "Keep Phase 4 blocked; the owner must explicitly retain, replace, or waive the shell-threshold framework before roadmap work resumes.",
  edge_related_configuration: "Keep Phase 4 blocked; require an explicit owner decision on how the shell cap handles the likely one-time edge-related tax.",
  manifest_related: "Redesign the manifest migration and run a new preregistered confirmation before reconsidering Phase 3.",
  boundary_composite: "Redesign the concrete boundary and seek separate approval for a new Phase 3 attempt; Phase 4 remains blocked.",
  not_reproduced: "Run a new preregistered direct A/D A/B confirmation before changing Phase 3's recorded outcome.",
  threshold_disagreement: "Document the anomaly and preregister any follow-up; do not adapt thresholds or change the roadmap from this result.",
};

export function assertProtocolPinMatches(sessionManifest, verified) {
  const { protocolLock, ...pin } = verified;
  const pinKeys = ["protocolCommit", "lockPath", "lockBlob", "lockSha256", "protocolVersion"];
  if (pinKeys.some((key) => pin[key] !== sessionManifest.protocol[key])
    || JSON.stringify(protocolLock) !== JSON.stringify(sessionManifest.protocolLock)) {
    throw new Error("report protocol pin mismatch");
  }
}

function sameAbsolutePath(left, right) {
  const normalizedLeft = path.normalize(path.resolve(left));
  const normalizedRight = path.normalize(path.resolve(right));
  return process.platform === "win32"
    ? normalizedLeft.toLowerCase() === normalizedRight.toLowerCase()
    : normalizedLeft === normalizedRight;
}

export function assertFixedReportOutput({
  sessionManifest,
  output,
  runningProtocolRoot = RUNNING_PROTOCOL_ROOT,
}) {
  const protocolRoot = path.resolve(sessionManifest.protocolRoot);
  if (!sameAbsolutePath(protocolRoot, runningProtocolRoot)) {
    throw new Error("session protocol root differs from the running frozen reporter");
  }
  const expectedOutput = path.resolve(protocolRoot, ...REPORT_PATH.split("/"));
  if (!sameAbsolutePath(output, expectedOutput)) {
    throw new Error(`report output must equal ${REPORT_PATH}`);
  }
  return { protocolRoot, expectedOutput };
}

export async function verifyReportProtocol({
  sessionManifest,
  output,
  runningProtocolRoot = RUNNING_PROTOCOL_ROOT,
  verifyFn = verifyFrozenProtocol,
}) {
  const { protocolRoot } = assertFixedReportOutput({ sessionManifest, output, runningProtocolRoot });
  const verified = await verifyFn({
    repoRoot: protocolRoot,
    allowedUntrackedPaths: [REPORT_PATH],
  });
  assertProtocolPinMatches(sessionManifest, verified);
}

function regexEscape(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function pidAlive(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    if (error.code === "ESRCH") return false;
    throw error;
  }
}

export async function cleanupOwnedAtomicTemps(target, injected = {}) {
  const aliveFn = injected.processAliveFn ?? pidAlive;
  const parent = path.dirname(target);
  const basename = path.basename(target);
  const pattern = new RegExp(
    `^${regexEscape(basename)}\\.(\\d+)\\.[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}\\.tmp$`,
    "i",
  );
  for (const entry of await readdir(parent, { withFileTypes: true })) {
    const match = entry.name.match(pattern);
    if (!match) continue;
    const candidate = path.join(parent, entry.name);
    const stat = await lstat(candidate);
    if (!stat.isFile() || stat.isSymbolicLink()) {
      throw new Error(`refusing non-regular report temp cleanup: ${candidate}`);
    }
    if (aliveFn(Number(match[1]))) {
      throw new Error(`report publisher is still alive for temp: ${candidate}`);
    }
    await unlink(candidate);
  }
}

function median(values) {
  const sorted = [...values].sort((a, b) => a - b);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
}

function summarize(samplesMs) {
  const medianMs = median(samplesMs);
  const samplesWithinBand = samplesMs.filter((value) => Math.abs(value - medianMs) <= 300).length;
  return { samplesMs: [...samplesMs], medianMs, samplesWithinBand, stable: samplesWithinBand >= 5 };
}

function metric(variant, left, right) {
  const aReferenceMs = (left.medianMs + right.medianMs) / 2;
  const deltaMs = variant.medianMs - aReferenceMs;
  const percentDelta = 100 * deltaMs / aReferenceMs;
  return {
    variantMedianMs: variant.medianMs,
    aReferenceMs,
    deltaMs,
    percentDelta,
    material: deltaMs >= 500,
    shellCapFailed: deltaMs > 500 || percentDelta > 5,
  };
}

function samplesFromRawAttempt(attemptResult) {
  return Object.fromEntries(Object.entries(attemptResult.blocks ?? {}).map(([name, block]) => {
    if (!Array.isArray(block.samples) || block.samples.length !== 7) {
      throw new Error(`raw attempt evidence mismatch: ${attemptResult.attemptId}/${name} does not have seven samples`);
    }
    const values = block.samples.map((sample) => sample.wallMs);
    if (values.some((value) => !Number.isFinite(value))) {
      throw new Error(`raw attempt evidence mismatch: ${attemptResult.attemptId}/${name} has a non-numeric wall sample`);
    }
    return [name, values];
  }));
}

function independentEvaluation(attemptResult) {
  const raw = samplesFromRawAttempt(attemptResult);
  for (const name of ["A0", "B", "A1", "C", "A2", "D", "A3"]) {
    if (!raw[name]) throw new Error(`raw attempt evidence mismatch: missing base block ${attemptResult.attemptId}/${name}`);
  }
  const hasE = Boolean(raw.E || raw.A4);
  if (Boolean(raw.E) !== Boolean(raw.A4)) {
    throw new Error("raw attempt evidence mismatch: conditional E/A4 pair");
  }
  const summaries = Object.fromEntries(Object.entries(raw).map(([name, values]) => [name, summarize(values)]));
  let anchorNames = ["A0", "A1", "A2", "A3"];
  const anchorMedians = anchorNames.map((name) => summaries[name].medianMs);
  let anchorRangeMs = Math.max(...anchorMedians) - Math.min(...anchorMedians);
  let reasons = Object.entries(summaries)
    .filter(([, value]) => !value.stable)
    .map(([name]) => `block_unstable:${name}`);
  if (anchorRangeMs > 300) reasons.push("anchor_range_exceeded");
  if (reasons.length) {
    if (hasE) throw new Error("raw attempt evidence mismatch: E/A4 ran after base stability failure");
    return { kind: "stability_invalid", eRequired: false, anchorRangeMs, reasons, summaries };
  }
  const metrics = {
    B: metric(summaries.B, summaries.A0, summaries.A1),
    C: metric(summaries.C, summaries.A1, summaries.A2),
    D: metric(summaries.D, summaries.A2, summaries.A3),
  };
  if (summaries.E) metrics.E = metric(summaries.E, summaries.A3, summaries.A4);
  const eRequired = !metrics.B.material && !metrics.C.material && metrics.D.material;
  if (hasE !== eRequired) {
    throw new Error("raw attempt evidence mismatch: conditional E/A4 contract");
  }
  if (hasE) {
    anchorNames = [...anchorNames, "A4"];
    anchorRangeMs = Math.max(...anchorNames.map((name) => summaries[name].medianMs))
      - Math.min(...anchorNames.map((name) => summaries[name].medianMs));
    reasons = Object.entries(summaries)
      .filter(([, value]) => !value.stable)
      .map(([name]) => `block_unstable:${name}`);
    if (anchorRangeMs > 300) reasons.push("anchor_range_exceeded");
    if (reasons.length) {
      return { kind: "stability_invalid", eRequired: true, anchorRangeMs, reasons, summaries };
    }
  }
  const disagreement = Object.entries(metrics)
    .filter(([, value]) => value.material !== value.shellCapFailed)
    .map(([name]) => name);
  let classification;
  if (disagreement.length) classification = "threshold_disagreement";
  else if (metrics.B.material) classification = "membership_configuration";
  else if (metrics.C.material) classification = "edge_related_configuration";
  else if (metrics.D.material && metrics.E?.material) classification = "manifest_related";
  else if (metrics.D.material) classification = "boundary_composite";
  else classification = "not_reproduced";
  return {
    kind: "valid",
    classification,
    eRequired,
    anchorRangeMs,
    disagreement,
    summaries,
    metrics,
    contrasts: {
      membershipMs: metrics.B.deltaMs,
      edgeAfterMembershipMs: metrics.C.deltaMs - metrics.B.deltaMs,
      manifestAfterCMs: metrics.E ? metrics.E.deltaMs - metrics.C.deltaMs : null,
      dSpecificCompositeMs: metrics.E ? metrics.D.deltaMs - metrics.E.deltaMs : null,
      dAfterCCompositeMs: metrics.E ? null : metrics.D.deltaMs - metrics.C.deltaMs,
    },
  };
}

function evaluationProjection(value) {
  return {
    kind: value.kind,
    classification: value.classification,
    eRequired: value.eRequired,
    anchorRangeMs: value.anchorRangeMs,
    reasons: value.reasons ?? [],
    disagreement: value.disagreement ?? [],
    summaries: Object.fromEntries(Object.entries(value.summaries).map(([name, summary]) => [name, {
      samplesMs: summary.samplesMs,
      medianMs: summary.medianMs,
      samplesWithinBand: summary.samplesWithinBand,
      stable: summary.stable,
    }])),
    metrics: value.metrics ?? null,
    contrasts: value.contrasts ?? null,
  };
}

function sameStrings(left, right) {
  return JSON.stringify([...(left ?? [])].sort()) === JSON.stringify([...(right ?? [])].sort());
}

export function deriveLedgerProjection(sessionId, events) {
  const starts = new Map();
  const finished = new Set();
  const attempts = [];
  let terminal = null;
  let sessionStarted = false;
  for (const [index, event] of events.entries()) {
    if (terminal) throw new Error("ledger projection mismatch: event follows session_completed");
    if (event.type === "session_started") {
      if (index !== 0 || sessionStarted || event.sessionId !== sessionId) {
        throw new Error("ledger projection mismatch: session_started identity/count");
      }
      sessionStarted = true;
    } else if (!sessionStarted) {
      throw new Error("ledger projection mismatch: session_started must be first");
    } else if (event.type === "attempt_started") {
      if (starts.has(event.attemptId)) throw new Error("ledger projection mismatch: duplicate attempt_started");
      starts.set(event.attemptId, event);
    } else if (event.type === "attempt_finished") {
      if (!starts.has(event.attemptId) || finished.has(event.attemptId)) {
        throw new Error("ledger projection mismatch: orphan/duplicate attempt_finished");
      }
      finished.add(event.attemptId);
      attempts.push({
        attemptId: event.attemptId,
        status: event.kind,
        startedAt: event.startedAt,
        endedAt: event.endedAt,
        reasons: event.reasons,
        worktree: event.worktree,
        targetDirectory: event.targetDirectory,
        environment: event.environment,
        resultPath: event.resultPath,
      });
    } else if (event.type === "session_completed") {
      terminal = event;
    }
  }
  if (!sessionStarted || !terminal || starts.size !== finished.size) {
    throw new Error("ledger projection mismatch: session is not exactly terminal and complete");
  }
  return {
    schemaVersion: 1,
    sessionId,
    unexplainedStabilityInvalidCount: terminal.retryState.unexplainedStabilityInvalidCount,
    terminal: true,
    attempts,
    events,
  };
}

function replayMismatch(message, details = null) {
  throw new Error(`retry protocol replay mismatch: ${message}${details ? ` ${JSON.stringify(details)}` : ""}`);
}

export function assertRetryProtocol({ events, attemptResults, decision }) {
  const results = new Map(attemptResults.map((result) => [result.attemptId, result]));
  if (results.size !== attemptResults.length) replayMismatch("duplicate result id");
  let state = { unexplainedStabilityInvalidCount: 0, terminal: false };
  let activeAttempt = null;
  let pendingResult = null;
  let allowNextAttempt = true;
  let attemptsStarted = 0;
  let terminalEvent = null;
  let sessionStarted = false;
  for (const [index, event] of events.entries()) {
    if (terminalEvent) replayMismatch("event after terminal", event);
    if (event.type === "session_started") {
      if (index !== 0 || sessionStarted) replayMismatch("session_started must occur exactly once and first", event);
      sessionStarted = true;
      if (!isDeepStrictEqual(event.retryState, state)) replayMismatch("initial retry state", event);
    } else if (!sessionStarted) {
      replayMismatch("event before session_started", event);
    } else if (event.type === "attempt_started") {
      if (activeAttempt || pendingResult || state.terminal || !allowNextAttempt) {
        replayMismatch("illegal attempt start", event);
      }
      attemptsStarted += 1;
      allowNextAttempt = false;
      if (!isDeepStrictEqual(event.retryState, state)) replayMismatch("attempt retry state", event);
      activeAttempt = event.attemptId;
    } else if (event.type === "attempt_finished") {
      if (activeAttempt !== event.attemptId || pendingResult) replayMismatch("attempt finish ordering", event);
      const result = results.get(event.attemptId);
      if (!result || result.kind !== event.kind) replayMismatch("attempt result identity/kind", event);
      activeAttempt = null;
      pendingResult = result;
    } else if (event.type === "retry_disposition") {
      if (!pendingResult || pendingResult.attemptId !== event.attemptId || pendingResult.kind === "valid") {
        replayMismatch("disposition without matching invalid attempt", event);
      }
      const correctedCause = typeof event.correctedCause === "string" ? event.correctedCause.trim() : "";
      const unexplained = event.unexplainedStability === true;
      if (correctedCause && unexplained) replayMismatch("ambiguous disposition", event);
      let action;
      if (pendingResult.kind === "infrastructure_invalid") {
        if (!correctedCause || unexplained) replayMismatch("infrastructure retry lacks corrected cause", event);
        action = "retry";
      } else if (pendingResult.kind === "stability_invalid") {
        if (correctedCause) action = "retry";
        else if (unexplained) {
          state = {
            unexplainedStabilityInvalidCount: state.unexplainedStabilityInvalidCount + 1,
            terminal: false,
          };
          action = state.unexplainedStabilityInvalidCount >= 2
            ? "environment_precision_insufficient"
            : "retry";
        } else replayMismatch("stability disposition missing cause/unexplained choice", event);
      } else replayMismatch("unknown invalidation kind", pendingResult);
      if (action === "environment_precision_insufficient") state = { ...state, terminal: true };
      const expectedState = { ...state };
      if (event.invalidationKind !== pendingResult.kind
        || event.retryAction !== action
        || !isDeepStrictEqual(event.retryState, expectedState)) {
        replayMismatch("disposition action/state", { event, action, expectedState });
      }
      pendingResult = null;
      allowNextAttempt = action === "retry";
    } else if (event.type === "session_completed") {
      if (activeAttempt) replayMismatch("terminal with active attempt", event);
      if (pendingResult?.kind === "valid") {
        state = { ...state, terminal: true };
        if (event.classification !== pendingResult.evaluation?.classification) {
          replayMismatch("valid terminal classification", event);
        }
        pendingResult = null;
      } else if (!(state.terminal && event.classification === "environment_precision_insufficient" && !pendingResult)) {
        replayMismatch("terminal without valid/precision outcome", event);
      }
      if (!isDeepStrictEqual(event.retryState, state) || !isDeepStrictEqual(event.decision, decision)) {
        replayMismatch("terminal state/decision", event);
      }
      terminalEvent = event;
    }
  }
  if (!sessionStarted || !terminalEvent || activeAttempt || pendingResult || attemptsStarted !== results.size) {
    replayMismatch("incomplete replay", { attemptsStarted, results: results.size });
  }
}

function assertIndependentEvidence({ ledger, decision, attemptResults }) {
  const results = new Map();
  for (const result of attemptResults) {
    if (!result?.attemptId || results.has(result.attemptId)) {
      throw new Error("raw attempt evidence mismatch: missing or duplicate attempt id");
    }
    results.set(result.attemptId, result);
  }
  if (results.size !== ledger.attempts.length) {
    throw new Error("raw attempt evidence mismatch: every ledger attempt needs exactly one immutable result");
  }
  const calculations = new Map();
  for (const row of ledger.attempts) {
    const result = results.get(row.attemptId);
    if (!result || result.kind !== row.status || !sameStrings(result.reasons, row.reasons)) {
      throw new Error(`raw attempt evidence mismatch: ledger/result projection for ${row.attemptId}`);
    }
    if (["valid", "stability_invalid"].includes(result.kind)) {
      const calculation = independentEvaluation(result);
      calculations.set(row.attemptId, calculation);
      if (calculation.kind !== result.kind) {
        throw new Error(`raw attempt evidence mismatch: recalculated kind for ${row.attemptId}`);
      }
      if (JSON.stringify(evaluationProjection(result.evaluation)) !== JSON.stringify(evaluationProjection(calculation))) {
        throw new Error(`raw attempt evidence mismatch: recalculated evaluation for ${row.attemptId}`);
      }
    } else if (result.evaluation != null) {
      throw new Error(`raw attempt evidence mismatch: infrastructure result carries an evaluation for ${row.attemptId}`);
    }
  }

  if (decision.classification === "environment_precision_insufficient") {
    if (decision.attemptId !== null || decision.evaluation !== null) {
      throw new Error("independent recalculation mismatch: precision outcome carries a causal decision");
    }
    const consumed = (ledger.events ?? []).filter((event) =>
      event.type === "retry_disposition" && event.unexplainedStability === true,
    );
    if (
      decision.unexplainedStabilityInvalidCount !== 2
      || ledger.unexplainedStabilityInvalidCount !== 2
      || consumed.length !== 2
      || new Set(consumed.map((event) => event.attemptId)).size !== 2
      || consumed.some((event) => calculations.get(event.attemptId)?.kind !== "stability_invalid")
    ) throw new Error("independent recalculation mismatch: environment precision evidence");
    return calculations;
  }

  const terminal = results.get(decision.attemptId);
  const recalculated = calculations.get(decision.attemptId);
  if (!terminal || terminal.kind !== "valid" || !recalculated) {
    throw new Error("independent recalculation mismatch: terminal valid attempt");
  }
  if (JSON.stringify(evaluationProjection(decision.evaluation)) !== JSON.stringify(evaluationProjection(recalculated))) {
    throw new Error("independent recalculation mismatch: recorded decision evaluation");
  }
  if (decision.classification !== recalculated.classification) {
    throw new Error("independent recalculation mismatch: terminal classification");
  }
  return calculations;
}

function safeCell(value) {
  return String(value ?? "n/a").replaceAll("|", "\\|").replaceAll("\r", " ").replaceAll("\n", " ");
}

function environmentCell(name, value) {
  if (name === "mainTargetSnapshot") {
    return safeCell(JSON.stringify({ exists: value.exists, digest: value.digest, records: value.records.length }));
  }
  return safeCell(typeof value === "object" ? JSON.stringify(value) : value);
}

function signedMs(value) {
  return value == null ? "n/a" : `${value >= 0 ? "+" : ""}${value} ms`;
}

function attemptRows(ledger) {
  return ledger.attempts.map((attempt) =>
    `| ${attempt.attemptId} | ${attempt.status} | ${safeCell(attempt.reasons?.join(", ") || "none")} | ${safeCell(attempt.startedAt)} | ${safeCell(attempt.endedAt)} |`,
  ).join("\n");
}

function attemptEnvironmentRows(ledger) {
  return ledger.attempts.map((attempt) =>
    `| ${attempt.attemptId} | ${safeCell(attempt.environment?.host)} | ${safeCell(attempt.environment?.power)} | ${safeCell(attempt.environment?.defender)} | ${safeCell(attempt.targetDirectory)} |`,
  ).join("\n");
}

function retryDispositionRows(ledger) {
  const rows = (ledger.events ?? []).filter((event) => event.type === "retry_disposition").map((event) =>
    `| ${event.attemptId} | ${event.invalidationKind} | ${event.unexplainedStability === true} | ${safeCell(event.correctedCause || "none")} | ${event.retryAction} | ${event.retryState?.unexplainedStabilityInvalidCount ?? "n/a"} |`,
  );
  return rows.length ? rows.join("\n") : "| none | none | false | none | none | 0 |";
}

function attemptErrorRows(attemptResults) {
  return attemptResults.map((attempt) =>
    `| ${attempt.attemptId} | ${attempt.kind} | ${safeCell(attempt.error?.kind || "none")} | ${safeCell(attempt.error?.category || "none")} | ${safeCell(attempt.error?.message || "none")} |`,
  ).join("\n");
}

function correctedEnvironmentRows(ledger) {
  const rows = (ledger.events ?? []).filter((event) =>
    event.type === "attempt_environment" && event.correctedEnvironmentDelta,
  ).map((event) =>
    `| ${event.attemptId} | ${safeCell(event.correctedEnvironmentDelta.correctedCause)} | ${safeCell(JSON.stringify(event.correctedEnvironmentDelta.before))} | ${safeCell(JSON.stringify(event.correctedEnvironmentDelta.after))} |`,
  );
  return rows.length ? rows.join("\n") : "| none | none | none | none |";
}

function elapsedSummary(ledger) {
  const starts = ledger.attempts.map((attempt) => Date.parse(attempt.startedAt)).filter(Number.isFinite);
  const ends = ledger.attempts.map((attempt) => Date.parse(attempt.endedAt)).filter(Number.isFinite);
  if (!starts.length || !ends.length) return "unavailable from recorded timestamps";
  return `${((Math.max(...ends) - Math.min(...starts)) / 60_000).toFixed(1)} minutes`;
}

function blockRows(evaluation, attemptResult) {
  return ORDER.filter((name) => evaluation.summaries[name]).map((name) => {
    const summary = evaluation.summaries[name];
    const block = attemptResult?.blocks?.[name];
    return `| ${name} | ${summary.samplesMs.join(", ")} | ${summary.medianMs} ms | ${summary.samplesWithinBand}/7 | ${block?.noOp?.elapsedMs ?? "n/a"} ms | ${block?.noOp?.cargoReportedMs ?? "n/a"} ms |`;
  }).join("\n");
}

function cargoRows(attemptResult) {
  return ORDER.filter((name) => attemptResult?.blocks?.[name]).map((name) => {
    const block = attemptResult.blocks[name];
    return `| ${name} | ${block.samples.map((sample) => sample.cargoReportedMs).join(", ")} | ${block.diagnostic?.extractumProcessExtern === true} | ${safeCell(block.inventory?.featureTreePath)} | ${safeCell(block.diagnostic?.timingArtifact?.path)} | ${safeCell(block.diagnostic?.timingArtifact?.sha256)} |`;
  }).join("\n");
}

function stateRows(attemptResult) {
  return ORDER.filter((name) => attemptResult?.blocks?.[name]).map((name) => {
    const block = attemptResult.blocks[name];
    return `| ${name} | ${safeCell(block.stateEvidence?.srcTauriTree)} | ${safeCell(block.stateEvidence?.canonicalLibSha256)} | ${block.inventory?.extractumProcessDirectDependency === true} | ${safeCell(block.inventory?.metadata?.target_directory)} |`;
  }).join("\n");
}

function rawBlockRows(attemptResult, calculation) {
  return ORDER.filter((name) => attemptResult?.blocks?.[name]).map((name) => {
    const block = attemptResult.blocks[name];
    const summary = calculation?.summaries?.[name] ?? null;
    const wall = block.samples?.map((sample) => sample.wallMs) ?? [];
    return `| ${name} | ${wall.join(", ") || "none"} | ${summary ? `${summary.medianMs} ms` : "n/a"} | ${summary ? `${summary.samplesWithinBand}/7` : "n/a"} | ${block.noOp?.elapsedMs ?? "n/a"} ms | ${block.noOp?.cargoReportedMs ?? "n/a"} ms |`;
  }).join("\n");
}

function attemptEvidenceSections(attemptResults, calculations) {
  const lines = [];
  for (const attempt of attemptResults) {
    const calculation = calculations.get(attempt.attemptId) ?? null;
    lines.push(
      `## Attempt ${attempt.attemptId} raw measurements`, "",
      `**Recorded kind:** \`${attempt.kind}\``, "",
      `**Recalculated stability reasons:** ${safeCell(calculation?.reasons?.join(", ") || "none / infrastructure invalidation")}`, "",
      "| Block | Wall samples | Median | Within 300 ms | No-op wall | No-op Cargo |",
      "| --- | --- | ---: | ---: | ---: | ---: |",
      rawBlockRows(attempt, calculation), "",
      "### State evidence", "",
      "| Block | src-tauri tree | Canonical lib.rs SHA-256 | Metadata direct edge | Cargo target |",
      "| --- | --- | --- | --- | --- |",
      stateRows(attempt), "",
      "### Cargo-reported samples and diagnostics", "",
      "| Block | Cargo durations (ms) | `--extern extractum_process` | Feature graph | Timings HTML | SHA-256 |",
      "| --- | --- | --- | --- | --- | --- |",
      cargoRows(attempt), "",
    );
  }
  return lines;
}

function metricRows(metrics) {
  return ["B", "C", "D", "E"].filter((name) => metrics[name]).map((name) => {
    const value = metrics[name];
    return `| ${name} | ${value.variantMedianMs} ms | ${value.aReferenceMs} ms | ${signedMs(value.deltaMs)} | ${value.percentDelta.toFixed(3)}% | ${value.material} | ${value.shellCapFailed} |`;
  }).join("\n");
}

function contrastRows(contrasts) {
  return [
    ["Membership", contrasts.membershipMs],
    ["Edge after membership", contrasts.edgeAfterMembershipMs],
    ["Manifest after C", contrasts.manifestAfterCMs],
    ["D-specific composite", contrasts.dSpecificCompositeMs],
    ["D after C composite", contrasts.dAfterCCompositeMs],
  ].filter(([, value]) => value != null).map(([name, value]) => `| ${name} | ${signedMs(value)} |`).join("\n");
}

export function renderVerification({
  sessionManifest,
  measurementEnvironment,
  ledger,
  decision,
  attemptResults,
  artifactIndex,
}) {
  const calculations = assertIndependentEvidence({ ledger, decision, attemptResults });
  const lines = [
    "# Process Shell Regression Diagnostic Verification",
    "",
    `**Session:** \`${sessionManifest.sessionId}\``,
    `**Outcome:** \`${decision.classification}\``,
    `**Protocol commit:** \`${sessionManifest.protocol.protocolCommit}\``,
    `**Protocol-lock blob:** \`${sessionManifest.protocol.lockBlob}\``,
    `**Protocol-lock SHA-256:** \`${sessionManifest.protocol.lockSha256}\``,
    `**Raw artifact directory:** \`${sessionManifest.sessionDir}\``,
    `**Artifact-index SHA-256:** \`${artifactIndex.sha256}\` (${artifactIndex.files} files, ${artifactIndex.bytes} bytes)`,
    `**Recorded attempt span:** ${elapsedSummary(ledger)}`,
    "",
    "## Environment",
    "",
    "| Field | Value |",
    "| --- | --- |",
    ...Object.entries(measurementEnvironment).map(([name, value]) => `| ${name} | ${environmentCell(name, value)} |`),
    "",
    "## Attempt ledger",
    "",
    "| Attempt | Status | Reasons | Started | Ended |",
    "| --- | --- | --- | --- | --- |",
    attemptRows(ledger),
    "",
    "## Attempt environments", "",
    "| Attempt | Host | Power | Defender | Target |",
    "| --- | --- | --- | --- | --- |",
    attemptEnvironmentRows(ledger), "",
    "## Retry and invalidation audit", "",
    "| Attempt | Invalidation | Unexplained stability | Corrected cause | Action | Count |",
    "| --- | --- | --- | --- | --- | ---: |",
    retryDispositionRows(ledger), "",
    "### Attempt error details", "",
    "| Attempt | Kind | Error kind | Category | Message |",
    "| --- | --- | --- | --- | --- |",
    attemptErrorRows(attemptResults), "",
    "### Corrected environment deltas", "",
    "| Attempt | Corrected cause | Before | After |",
    "| --- | --- | --- | --- |",
    correctedEnvironmentRows(ledger), "",
    ...attemptEvidenceSections(attemptResults, calculations),
  ];
  if (decision.classification === "environment_precision_insufficient") {
    lines.push(
      "## Decision", "",
      "No B/C/D/E causal classification is made.", "",
      `**Unexplained stability-invalid count:** ${decision.unexplainedStabilityInvalidCount}`, "",
      "The machine did not support the preregistered 300 ms precision twice. Any next run requires a separately frozen anomaly protocol.", "",
      "**Required next step:** Keep Phase 4 blocked and preregister a new design with sample count, interleaving/counterbalancing, and stability rule fixed before new data.", "",
    );
  } else {
    const evaluation = calculations.get(decision.attemptId);
    lines.push(
      `**A-anchor range:** ${evaluation.anchorRangeMs} ms`,
      `**Conditional E required:** ${evaluation.eRequired}`, "",
      "## Variant metrics", "",
      "| Variant | Median | Local A reference | Delta | Delta % | Material | Shell cap failed |",
      "| --- | ---: | ---: | ---: | ---: | --- | --- |",
      metricRows(evaluation.metrics), "",
      "## Descriptive contrasts", "",
      "| Contrast | Value |", "| --- | ---: |",
      contrastRows(evaluation.contrasts), "",
      "These are descriptive contrasts between cumulative configurations; they are not independently randomized component estimates.", "",
      "## Decision", "", INTERPRETATION[decision.classification], "",
      `**Required next step:** ${REQUIRED_NEXT_STEP[decision.classification]}`, "",
    );
  }
  lines.push(
    "## Scope", "",
    "This diagnostic does not automatically retain `extractum-process` or unblock Phase 4. Any roadmap, threshold, or architecture change remains a separate owner-approved decision.", "",
    "The result is conditional on the fixed incremental-cache order. Evidence of order-specific hysteresis requires a separately preregistered counterbalanced experiment, not a post-hoc rerun.", "",
  );
  return `${lines.join("\n")}\n`;
}

async function walkArtifacts(root, current = root) {
  const records = [];
  for (const entry of await readdir(current, { withFileTypes: true })) {
    const absolute = path.join(current, entry.name);
    const relative = path.relative(root, absolute).replaceAll("\\", "/");
    if (relative === "artifact-index.json") continue;
    if (relative === "worktrees" && entry.isDirectory()) continue;
    if (entry.isSymbolicLink()) throw new Error(`artifact symlink is forbidden: ${relative}`);
    if (entry.isDirectory()) records.push(...await walkArtifacts(root, absolute));
    else if (entry.isFile()) {
      const bytes = await readFile(absolute);
      records.push({ path: relative, bytes: bytes.length, sha256: await sha256File(absolute) });
    }
  }
  return records.sort((left, right) => left.path.localeCompare(right.path));
}

function sha256Bytes(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

export async function prepareArtifactIndex(sessionDir, locatorPath) {
  const records = await walkArtifacts(sessionDir);
  const locatorBytes = await readFile(locatorPath);
  records.push({
    path: "@external/session-locator.json",
    source: locatorPath,
    bytes: locatorBytes.length,
    sha256: sha256Bytes(locatorBytes),
  });
  records.sort((left, right) => left.path.localeCompare(right.path));
  const target = path.join(sessionDir, "artifact-index.json");
  const bytes = Buffer.from(`${JSON.stringify({ schemaVersion: 1, sessionDir, records }, null, 2)}\n`, "utf8");
  return {
    path: target,
    content: bytes,
    sha256: sha256Bytes(bytes),
    files: records.length,
    bytes: records.reduce((sum, record) => sum + record.bytes, 0),
  };
}

async function publishBytesIdempotent(target, bytes, writeFn = writeAtomicBytesExclusive) {
  try {
    await writeFn(target, bytes);
    return;
  } catch (error) {
    if (error.kind !== "duplicate_artifact") throw error;
  }
  const existing = await readFile(target);
  if (!existing.equals(bytes)) throw new Error(`immutable publication conflict: ${target}`);
}

export async function publishReportPair(
  { artifactIndex, output, reportBytes },
  writeFn = writeAtomicBytesExclusive,
) {
  await publishBytesIdempotent(artifactIndex.path, artifactIndex.content, writeFn);
  await publishBytesIdempotent(output, reportBytes, writeFn);
}

async function readNumberedLedger(sessionDir) {
  const directory = path.join(sessionDir, "ledger");
  const names = (await readdir(directory)).filter((name) => /^\d{6}\.json$/.test(name)).sort();
  const events = [];
  for (let index = 0; index < names.length; index += 1) {
    const expected = `${String(index + 1).padStart(6, "0")}.json`;
    if (names[index] !== expected) throw new Error(`numbered ledger gap: ${expected} != ${names[index]}`);
    const event = JSON.parse(await readFile(path.join(directory, names[index]), "utf8"));
    if (event.sequence !== index + 1) throw new Error(`numbered ledger sequence mismatch: ${names[index]}`);
    events.push(event);
  }
  return events;
}

function option(name) {
  const index = process.argv.indexOf(name);
  if (index < 0 || !process.argv[index + 1]) throw new Error(`missing ${name}`);
  return process.argv[index + 1];
}

async function main() {
  const sessionDir = path.resolve(option("--session-dir"));
  const output = path.resolve(option("--output"));
  const sessionManifest = JSON.parse(
    await readFile(path.join(sessionDir, "session-manifest.json"), "utf8"),
  );
  // Validate the fixed workflow-owned destination against this running frozen
  // reporter before deleting even an exact atomic-temp sibling.
  assertFixedReportOutput({ sessionManifest, output });
  // A hard kill can strand only the atomic writer's exact sibling temp. Remove
  // dead-PID regular files matching that workflow-owned pattern before Git
  // status verification; never touch the output or a near-match.
  await cleanupOwnedAtomicTemps(output);
  if (process.argv.includes("--verify-only")) {
    await verifyReportProtocol({ sessionManifest, output });
    process.stdout.write(`${sessionManifest.protocol.lockSha256}\n`);
    return;
  }
  const [recordedLedger, decision] = await Promise.all([
    readFile(path.join(sessionDir, "session-ledger.json"), "utf8").then(JSON.parse),
    readFile(path.join(sessionDir, "decision.json"), "utf8").then(JSON.parse),
  ]);
  const numberedEvents = await readNumberedLedger(sessionDir);
  if (!isDeepStrictEqual(numberedEvents, recordedLedger.events)) {
    throw new Error("aggregate ledger differs from contiguous numbered events");
  }
  const ledger = deriveLedgerProjection(sessionManifest.sessionId, numberedEvents);
  if (!isDeepStrictEqual(ledger, recordedLedger)) {
    throw new Error("aggregate ledger projection differs from numbered events");
  }
  const locatorBytes = await readFile(sessionManifest.locatorPath);
  const locator = JSON.parse(locatorBytes.toString("utf8"));
  if (
    locator.sessionId !== sessionManifest.sessionId
    || !sameAbsolutePath(locator.sessionDir, sessionDir)
    || !sameAbsolutePath(sessionManifest.sessionDir, sessionDir)
    || !sameAbsolutePath(locator.sessionManifestPath, path.join(sessionDir, "session-manifest.json"))
    || !isDeepStrictEqual(locator, sessionManifest.locatorRecord)
    || sha256Bytes(locatorBytes) !== sessionManifest.locatorSha256
  ) throw new Error("session locator differs from immutable manifest anchor");
  await verifyReportProtocol({ sessionManifest, output });
  // The index writer uses the same atomic sibling scheme. Only after both the
  // external locator and frozen reporter have authenticated this session path
  // may a dead publisher's exact index temp be removed.
  await cleanupOwnedAtomicTemps(path.join(sessionDir, "artifact-index.json"));
  const measurementBaselines = numberedEvents.filter((event) =>
    event.type === "attempt_environment" && event.environmentBaseline === true,
  );
  if (measurementBaselines.length !== 1) {
    throw new Error(`expected exactly one authoritative attempt environment, got ${measurementBaselines.length}`);
  }
  const measurementEnvironment = measurementBaselines[0].environment;
  const finalMainTargetSnapshot = await snapshotDirectory(measurementEnvironment.mainTargetDirectory);
  if (JSON.stringify(finalMainTargetSnapshot) !== JSON.stringify(measurementEnvironment.mainTargetSnapshot)) {
    throw new Error("main target content changed during the diagnostic session");
  }
  const attemptResults = [];
  for (const attempt of ledger.attempts) {
    if (!attempt.resultPath) throw new Error(`attempt ${attempt.attemptId} has no immutable result path`);
    attemptResults.push(JSON.parse(await readFile(attempt.resultPath, "utf8")));
  }
  assertRetryProtocol({ events: numberedEvents, attemptResults, decision });
  const artifactIndex = await prepareArtifactIndex(sessionDir, sessionManifest.locatorPath);
  const reportBytes = Buffer.from(
    renderVerification({
      sessionManifest,
      measurementEnvironment,
      ledger,
      decision,
      attemptResults,
      artifactIndex,
    }),
    "utf8",
  );
  // All ledger, locator, raw-result, and arithmetic checks above complete before
  // either publication. A crash between the two writes is recoverable: rerun
  // accepts only byte-identical output and creates the missing peer.
  await publishReportPair({ artifactIndex, output, reportBytes });
  process.stdout.write(`${output}\n`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  await main();
}
