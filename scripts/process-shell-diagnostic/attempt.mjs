import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

import { installState, verifyTargetIsolation } from "./git-state.mjs";
import { PROTOCOL, evaluateAttempt, summarizeBlock } from "./protocol.mjs";
import {
  assertCommandOk,
  hasTerminationUnconfirmed,
  ProtocolError,
  runCargoCheck,
  runDirtyCargoProbe,
  runWindowsProcess,
  writeAtomicJsonExclusive,
} from "./runtime.mjs";

function ordinaryEnvironment() {
  return Object.fromEntries(
    Object.entries(process.env).filter(([key]) => key.toUpperCase() !== "CARGO_LOG"),
  );
}

function taskkillExe() {
  if (!process.env.SystemRoot) throw new ProtocolError("missing_system_root", "SystemRoot is required");
  return path.join(process.env.SystemRoot, "System32", "taskkill.exe");
}

function requireCargoMetadata(result, label) {
  assertCommandOk(result, label);
  if (
    result.closeObserved !== true ||
    !Number.isFinite(result.elapsedMs) ||
    !Number.isFinite(result.cargoReportedMs)
  ) {
    throw new ProtocolError("required_cargo_metadata_missing", label, { result });
  }
}

function extractumProcessDirectDependency(metadata, block) {
  const app = metadata.packages?.find((pkg) => pkg.name === "extractum");
  if (!app) throw new ProtocolError("extractum_metadata_missing", block);
  return app.dependencies.some((dependency) =>
    dependency.name === "extractum-process"
    && dependency.kind === null
    && typeof dependency.path === "string",
  );
}

async function evidenceCommand({ label, args, worktree, artifactDir }) {
  const result = await runWindowsProcess({
    label,
    command: "cargo.exe",
    args,
    cwd: worktree,
    env: ordinaryEnvironment(),
    artifactDir,
    timeoutMs: PROTOCOL.commandTimeoutMs,
    taskkillExe: taskkillExe(),
  });
  assertCommandOk(result, label, "state_inventory_failed");
  if (result.closeObserved !== true) throw new ProtocolError("state_inventory_failed", label, { result });
  return {
    result,
    stdout: await readFile(result.stdoutPath, "utf8"),
    stderr: await readFile(result.stderrPath, "utf8"),
  };
}

export async function captureStateInventory({ block, worktree, mainRoot, artifactDir }) {
  const metadataRun = await evidenceCommand({
    label: `${block}.metadata`,
    args: [
      "metadata",
      "--manifest-path",
      "src-tauri/Cargo.toml",
      "--format-version",
      "1",
      "--no-deps",
      "--locked",
    ],
    worktree,
    artifactDir,
  });
  let metadata;
  try {
    metadata = JSON.parse(metadataRun.stdout);
  } catch (error) {
    throw new ProtocolError("cargo_metadata_parse_failed", block, { message: error.message });
  }
  await verifyTargetIsolation({ metadata, worktree, mainRoot });
  const directProcessDependency = extractumProcessDirectDependency(metadata, block);

  const treeRun = await evidenceCommand({
    label: `${block}.feature-tree`,
    args: [
      "tree",
      "--manifest-path",
      "src-tauri/Cargo.toml",
      "--workspace",
      "-e",
      "features",
      "--locked",
    ],
    worktree,
    artifactDir,
  });
  return {
    metadata,
    extractumProcessDirectDependency: directProcessDependency,
    metadataProcess: metadataRun.result,
    featureTreePath: treeRun.result.stdoutPath,
    featureTreeSha256Input: treeRun.stdout,
  };
}

export async function verifyTargetPreflight({ block, worktree, mainRoot, artifactDir }) {
  const metadataRun = await evidenceCommand({
    label: `${block}.target-preflight-metadata`,
    args: [
      "metadata",
      "--manifest-path",
      "src-tauri/Cargo.toml",
      "--format-version",
      "1",
      "--no-deps",
      "--locked",
    ],
    worktree,
    artifactDir,
  });
  let metadata;
  try {
    metadata = JSON.parse(metadataRun.stdout);
  } catch (error) {
    throw new ProtocolError("cargo_metadata_parse_failed", `${block}.target-preflight`, {
      message: error.message,
    });
  }
  await verifyTargetIsolation({ metadata, worktree, mainRoot });
  return {
    targetDirectory: metadata.target_directory,
    metadataProcess: metadataRun.result,
  };
}

const productionDependencies = {
  installStateFn: installState,
  verifyTargetPreflightFn: verifyTargetPreflight,
  captureStateInventoryFn: captureStateInventory,
  runCargoCheckFn: runCargoCheck,
  runDirtyProbeFn: runDirtyCargoProbe,
  evaluateAttemptFn: evaluateAttempt,
  writeJsonFn: writeAtomicJsonExclusive,
};

async function runBlock({
  block,
  spec,
  attemptDir,
  deps,
}) {
  const stateEvidence = await deps.installStateFn({
    state: block,
    worktree: spec.worktree,
    mainRoot: spec.mainRoot,
    protocolLock: spec.protocolLock,
    artifactDir: attemptDir,
  });
  const cargoShared = {
    worktree: spec.worktree,
    artifactDir: attemptDir,
    cargoExe: "cargo.exe",
    taskkillExe: taskkillExe(),
    timeoutMs: PROTOCOL.commandTimeoutMs,
  };

  // cargo metadata --no-deps is the only allowed pre-build command: it proves
  // this state resolves to the isolated attempt target before Cargo may write.
  const targetPreflight = await deps.verifyTargetPreflightFn({
    block,
    worktree: spec.worktree,
    mainRoot: spec.mainRoot,
    artifactDir: attemptDir,
  });
  // The measured cache order is canonical sync first, then the resolved
  // metadata, feature graph, and unit inventory captured for evidence.
  const inventorySync = await deps.runCargoCheckFn({
    label: `${block}.inventory-sync`,
    diagnostic: false,
    ...cargoShared,
  });
  requireCargoMetadata(inventorySync, `${block}.inventory-sync`);
  const inventory = await deps.captureStateInventoryFn({
    block,
    worktree: spec.worktree,
    mainRoot: spec.mainRoot,
    artifactDir: attemptDir,
  });
  const expectsProcessExtern = ["C", "D", "E"].includes(block);
  if (inventory.extractumProcessDirectDependency !== expectsProcessExtern) {
    throw new ProtocolError("metadata_edge_mismatch", block, {
      expected: expectsProcessExtern,
      actual: inventory.extractumProcessDirectDependency,
    });
  }

  const dirtyShared = {
    ...cargoShared,
    sourcePath: path.join(spec.worktree, "src-tauri", "src", "lib.rs"),
    expectedCanonicalSha256: stateEvidence.canonicalLibSha256,
    requireExtractum: true,
  };
  const warmups = [];
  for (let index = 1; index <= PROTOCOL.warmupsPerBlock; index += 1) {
    const warmup = await deps.runDirtyProbeFn({
      label: `${block}.warmup-${index}`,
      diagnostic: false,
      ...dirtyShared,
    });
    requireCargoMetadata(warmup, `${block}.warmup-${index}`);
    if (!warmup.extractumChecked) throw new ProtocolError("extractum_not_checked", `${block}.warmup-${index}`);
    warmups.push(warmup);
  }

  const noOpSync = await deps.runCargoCheckFn({
    label: `${block}.noop-sync`,
    diagnostic: false,
    ...cargoShared,
  });
  requireCargoMetadata(noOpSync, `${block}.noop-sync`);
  const noOp = await deps.runCargoCheckFn({
    label: `${block}.noop`,
    diagnostic: false,
    ...cargoShared,
  });
  requireCargoMetadata(noOp, `${block}.noop`);

  const samples = [];
  for (let index = 1; index <= PROTOCOL.samplesPerBlock; index += 1) {
    const sample = await deps.runDirtyProbeFn({
      label: `${block}.sample-${index}`,
      diagnostic: false,
      ...dirtyShared,
    });
    requireCargoMetadata(sample, `${block}.sample-${index}`);
    if (!sample.extractumChecked) throw new ProtocolError("extractum_not_checked", `${block}.sample-${index}`);
    samples.push({
      index,
      wallMs: sample.elapsedMs,
      cargoReportedMs: sample.cargoReportedMs,
      checkedPackages: sample.checkedPackages ?? [PROTOCOL.expectedCheckedPackage],
      processMetadataPath: sample.stdoutPath ? sample.stdoutPath.replace(/\.stdout\.log$/, ".process.json") : null,
    });
  }
  const summary = summarizeBlock(samples.map((sample) => sample.wallMs));

  const diagnostic = await deps.runDirtyProbeFn({
    label: `${block}.diagnostic`,
    diagnostic: true,
    ...dirtyShared,
  });
  requireCargoMetadata(diagnostic, `${block}.diagnostic`);
  if (!diagnostic.extractumChecked) throw new ProtocolError("extractum_not_checked", `${block}.diagnostic`);
  if (!diagnostic.timingArtifact) throw new ProtocolError("timing_artifact_missing", block);
  if (!diagnostic.extractumLibRustcObserved) {
    throw new ProtocolError("extractum_lib_rustc_missing", block);
  }
  if (diagnostic.extractumProcessExtern !== expectsProcessExtern) {
    throw new ProtocolError("rustc_edge_mismatch", block, {
      expected: expectsProcessExtern,
      actual: diagnostic.extractumProcessExtern,
    });
  }

  const result = {
    schemaVersion: 1,
    block,
    stateEvidence,
    targetPreflight,
    inventorySync,
    inventory,
    warmups,
    noOpSync,
    noOp,
    samples,
    summary,
    diagnostic,
  };
  await deps.writeJsonFn(path.join(attemptDir, "blocks", `${block}.json`), result);
  return result;
}

function errorReason(error) {
  const kind = String(error?.kind ?? error?.name ?? "unknown").toLowerCase();
  function containsTimeout(value, seen = new Set()) {
    if (!value || typeof value !== "object" || seen.has(value)) return false;
    seen.add(value);
    if (
      value.timedOut === true
      || value.classification === "timeout"
      || value.classification === "termination_unconfirmed"
      || value.kind === "command_timeout"
    ) return true;
    return Object.values(value).some((entry) => containsTimeout(entry, seen));
  }
  if (kind.includes("timeout") || containsTimeout(error)) return "command_timeout";
  const exact = {
    canonical_sync_failed: "command_failed",
    cargo_failed: "command_failed",
    canonical_hash_mismatch: "restore_failed",
    recovery_hash_mismatch: "restore_failed",
    source_restore_failed: "restore_failed",
    extractum_not_checked: "metadata_invalid",
    extractum_lib_rustc_missing: "metadata_invalid",
    rustc_edge_mismatch: "metadata_invalid",
    metadata_edge_mismatch: "metadata_invalid",
    required_cargo_metadata_missing: "metadata_invalid",
    state_inventory_failed: "metadata_invalid",
  };
  if (exact[kind]) return exact[kind];
  if (/(restore|recovery|probe_source)/.test(kind)) return "restore_failed";
  if (/(target|workspace_root|cargo_target_dir)/.test(kind)) return "target_invalid";
  if (/(metadata|timing|checked_package|cargo_reported|inventory|rustc|extern|edge)/.test(kind)) return "metadata_invalid";
  if (/(state|tree|blob|patch|candidate|manifest)/.test(kind)) return "state_invalid";
  if (/(platform|host|attestation|environment|quiescence)/.test(kind)) return "environment_invalid";
  if (/(cargo|git|command|spawn|exit)/.test(kind)) return "command_failed";
  return "protocol_violation";
}

async function persistTerminationResultAndThrow({ deps, resultPath, result, error, attemptId }) {
  let persistenceError = null;
  try {
    await deps.writeJsonFn(resultPath, result);
  } catch (writeError) {
    persistenceError = writeError;
  }
  throw new ProtocolError("termination_unconfirmed", attemptId, {
    operatorActionRequired: true,
    attemptResult: result,
    cause: error?.details ?? null,
    persistenceError: persistenceError
      ? { kind: persistenceError.kind ?? persistenceError.name, message: persistenceError.message }
      : null,
  });
}

export async function runAttempt(spec, injected = {}) {
  const deps = { ...productionDependencies, ...injected };
  const attemptDir = path.join(spec.sessionDir, "attempts", spec.attemptId);
  const startedAt = new Date().toISOString();
  const blocks = {};
  let result;

  try {
    for (const block of PROTOCOL.baseSequence) {
      blocks[block] = await runBlock({ block, spec, attemptDir, deps });
    }
    let evaluation = deps.evaluateAttemptFn(
      Object.fromEntries(Object.entries(blocks).map(([name, value]) => [name, value.samples.map((sample) => sample.wallMs)])),
    );
    if (evaluation.kind === "needs_e") {
      for (const block of PROTOCOL.conditionalSequence) {
        blocks[block] = await runBlock({ block, spec, attemptDir, deps });
      }
      evaluation = deps.evaluateAttemptFn(
        Object.fromEntries(Object.entries(blocks).map(([name, value]) => [name, value.samples.map((sample) => sample.wallMs)])),
      );
    }
    if (!["valid", "stability_invalid"].includes(evaluation.kind)) {
      throw new ProtocolError("incomplete_evaluation", evaluation.kind);
    }
    result = {
      schemaVersion: 1,
      attemptId: spec.attemptId,
      kind: evaluation.kind,
      reasons: evaluation.reasons ?? [],
      startedAt,
      endedAt: new Date().toISOString(),
      blocks,
      evaluation,
    };
  } catch (error) {
    result = {
      schemaVersion: 1,
      attemptId: spec.attemptId,
      kind: "infrastructure_invalid",
      reasons: [errorReason(error)],
      startedAt,
      endedAt: new Date().toISOString(),
      blocks,
      error: {
        name: error?.name ?? "Error",
        kind: error?.kind ?? error?.name ?? "unknown",
        category: errorReason(error),
        message: error?.message ?? String(error),
        details: error?.details ?? null,
      },
    };
    if (hasTerminationUnconfirmed(error)) {
      result.finalState = null;
      await persistTerminationResultAndThrow({
        deps,
        resultPath: path.join(attemptDir, "attempt-result.json"),
        result,
        error,
        attemptId: spec.attemptId,
      });
    }
  }

  try {
    result.finalState = await deps.installStateFn({
      state: "A-final",
      worktree: spec.worktree,
      mainRoot: spec.mainRoot,
      protocolLock: spec.protocolLock,
      artifactDir: attemptDir,
    });
  } catch (error) {
    result = {
      schemaVersion: 1,
      attemptId: spec.attemptId,
      kind: "infrastructure_invalid",
      reasons: [errorReason(error)],
      startedAt,
      endedAt: new Date().toISOString(),
      blocks,
      error: {
        name: error?.name ?? "Error",
        kind: error?.kind ?? error?.name ?? "unknown",
        category: errorReason(error),
        message: error?.message ?? String(error),
        details: error?.details ?? null,
      },
    };
    if (hasTerminationUnconfirmed(error)) {
      result.finalState = null;
      await persistTerminationResultAndThrow({
        deps,
        resultPath: path.join(attemptDir, "attempt-result.json"),
        result,
        error,
        attemptId: spec.attemptId,
      });
    }
  }

  await deps.writeJsonFn(path.join(attemptDir, "attempt-result.json"), result);
  return result;
}
