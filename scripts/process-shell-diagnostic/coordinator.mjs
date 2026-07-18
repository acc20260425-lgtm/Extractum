import { createHash, randomUUID } from "node:crypto";
import { access, mkdir, readFile, readdir } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { pathToFileURL } from "node:url";

import { installState } from "./git-state.mjs";
import { runAttempt } from "./attempt.mjs";
import { PROTOCOL, reduceRetry } from "./protocol.mjs";
import {
  hasTerminationUnconfirmed,
  ProtocolError,
  runWindowsProcess,
  writeAtomicJsonExclusive,
} from "./runtime.mjs";

const LOCATOR_NAME = "process-shell-diagnostic.locator.json";
const INITIAL_RETRY_STATE = Object.freeze({
  unexplainedStabilityInvalidCount: 0,
  terminal: false,
});

async function assertMissing(filePath, kind) {
  try {
    await access(filePath);
  } catch (error) {
    if (error.code === "ENOENT") return;
    throw error;
  }
  throw new ProtocolError(kind, `path already exists: ${filePath}`);
}

function assertCargoTargetDirUnset(environment) {
  const entry = Object.entries(environment).find(([key]) => key.toUpperCase() === "CARGO_TARGET_DIR");
  if (entry) throw new ProtocolError("cargo_target_dir_set", `${entry[0]} must be absent`, { value: entry[1] });
}

function taskkillExe() {
  if (!process.env.SystemRoot) throw new ProtocolError("missing_system_root", "SystemRoot is required");
  return path.join(process.env.SystemRoot, "System32", "taskkill.exe");
}

async function controlCommand({ label, command, args, cwd, artifactDir, allowFailure = false }) {
  const result = await runWindowsProcess({
    label,
    command,
    args,
    cwd,
    env: Object.fromEntries(
      Object.entries(process.env).filter(([key]) => key.toUpperCase() !== "CARGO_LOG"),
    ),
    artifactDir,
    timeoutMs: PROTOCOL.commandTimeoutMs,
    taskkillExe: taskkillExe(),
  });
  if (hasTerminationUnconfirmed(result)) {
    throw new ProtocolError("termination_unconfirmed", label, {
      result,
      operatorActionRequired: true,
    });
  }
  const stdout = await readFile(result.stdoutPath, "utf8");
  const stderr = await readFile(result.stderrPath, "utf8");
  assertControlCommandResult(result, label, allowFailure, stderr);
  return { result, stdout: stdout.trim(), stderr: stderr.trim() };
}

export function assertControlCommandResult(result, label, allowFailure = false, stderr = "") {
  if (hasTerminationUnconfirmed(result)) {
    throw new ProtocolError("termination_unconfirmed", label, {
      result,
      stderr,
      operatorActionRequired: true,
    });
  }
  if (result?.timedOut === true || result?.classification === "timeout") {
    throw new ProtocolError("command_timeout", label, { result, stderr });
  }
  const completedNonzero = result?.classification === "command_failed"
    && result?.closeObserved === true
    && result?.timedOut !== true
    && Number.isInteger(result?.exitCode)
    && result.exitCode !== 0;
  if (result?.classification !== "ok" && !(allowFailure && completedNonzero)) {
    throw new ProtocolError("preflight_command_failed", label, { result, stderr });
  }
}

async function resolveProtocolCommitProduction({ protocolRoot, artifactDir }) {
  const value = await controlCommand({
    label: "protocol-head",
    command: "git.exe",
    args: ["rev-parse", "HEAD"],
    cwd: protocolRoot,
    artifactDir,
  });
  if (!/^[0-9a-f]{40}$/.test(value.stdout)) {
    throw new ProtocolError("protocol_commit_invalid", value.stdout);
  }
  return value.stdout;
}

async function captureEnvironmentProduction({
  mainRoot,
  artifactDir,
  processAttested,
  protocolLock,
}) {
  const rustc = await controlCommand({
    label: "environment-rustc",
    command: "rustc.exe",
    args: ["-vV"],
    cwd: mainRoot,
    artifactDir,
  });
  const cargo = await controlCommand({
    label: "environment-cargo",
    command: "cargo.exe",
    args: ["-V"],
    cwd: mainRoot,
    artifactDir,
  });
  const host = rustc.stdout.match(/^host:\s+(.+)$/m)?.[1] ?? null;
  if (process.platform !== "win32" || host !== "x86_64-pc-windows-msvc") {
    throw new ProtocolError("unsupported_host", `${process.platform}:${host}`);
  }

  const active = await controlCommand({
    label: "environment-build-processes",
    command: "powershell.exe",
    args: [
      "-NoLogo",
      "-NoProfile",
      "-Command",
      "$names=@('cargo.exe','rustc.exe','rust-analyzer.exe'); Get-CimInstance Win32_Process -ErrorAction Stop | Where-Object { $names -contains $_.Name -or $_.CommandLine -match '(?i)(tauri\\s+dev|vite(\\.js)?\\s+--host|npm\\.cmd\\s+run\\s+tauri)' } | Select-Object ProcessId,Name,CommandLine | ConvertTo-Json -Compress",
    ],
    cwd: mainRoot,
    artifactDir,
  });
  if (active.stdout !== "") {
    throw new ProtocolError("build_process_active", active.stdout);
  }

  const mainStatus = await controlCommand({
    label: "environment-main-src-status",
    command: "git.exe",
    args: ["status", "--porcelain=v1", "--untracked-files=all", "--", "src-tauri"],
    cwd: mainRoot,
    artifactDir,
  });
  if (mainStatus.stdout !== "") throw new ProtocolError("main_src_tauri_dirty", mainStatus.stdout);
  const mainTree = await controlCommand({
    label: "environment-main-src-tree",
    command: "git.exe",
    args: ["rev-parse", "HEAD:src-tauri"],
    cwd: mainRoot,
    artifactDir,
  });
  if (mainTree.stdout !== protocolLock.states.A.srcTauriTree) {
    throw new ProtocolError("main_baseline_tree_mismatch", mainTree.stdout);
  }

  const power = await controlCommand({
    label: "environment-power",
    command: "powercfg.exe",
    args: ["/GETACTIVESCHEME"],
    cwd: mainRoot,
    artifactDir,
    allowFailure: true,
  });
  const defender = await controlCommand({
    label: "environment-defender",
    command: "powershell.exe",
    args: [
      "-NoLogo",
      "-NoProfile",
      "-Command",
      "Get-MpComputerStatus | Select-Object RealTimeProtectionEnabled,AntivirusEnabled,QuickScanAge | ConvertTo-Json -Compress",
    ],
    cwd: mainRoot,
    artifactDir,
    allowFailure: true,
  });
  const cargoEnvironment = {};
  for (const name of ["CARGO_BUILD_TARGET", "CARGO_ENCODED_RUSTFLAGS", "CARGO_INCREMENTAL", "CARGO_TARGET_DIR", "RUSTFLAGS"]) {
    const entry = Object.entries(process.env).find(([key]) => key.toUpperCase() === name);
    cargoEnvironment[name] = entry?.[1] ?? null;
  }
  const mainTargetDirectory = path.join(mainRoot, "src-tauri", "target");
  const mainTargetSnapshot = await snapshotDirectory(mainTargetDirectory);
  return {
    platform: process.platform,
    architecture: process.arch,
    host,
    cargo: cargo.stdout,
    rustc: rustc.stdout,
    node: process.version,
    power: power.result.classification === "ok" ? power.stdout : `unavailable: ${power.stderr || power.result.classification}`,
    defender: defender.result.classification === "ok"
      ? defender.stdout
      : `unavailable: ${defender.stderr || defender.result.classification}`,
    processQuiescence: [],
    operatorProcessAttestation: processAttested,
    cargoEnvironment,
    mainRoot,
    mainSrcTauriTree: mainTree.stdout,
    mainTargetDirectory,
    mainTargetSnapshot,
  };
}

async function createDetachedWorktreeProduction({
  protocolRoot,
  worktree,
  protocolCommit,
  artifactDir,
}) {
  await mkdir(path.dirname(worktree), { recursive: true });
  await controlCommand({
    label: `${path.basename(worktree)}-worktree-add`,
    command: "git.exe",
    args: ["worktree", "add", "--detach", worktree, protocolCommit],
    cwd: protocolRoot,
    artifactDir,
  });
  const head = await controlCommand({
    label: `${path.basename(worktree)}-worktree-head`,
    command: "git.exe",
    args: ["rev-parse", "HEAD"],
    cwd: worktree,
    artifactDir,
  });
  if (head.stdout !== protocolCommit) {
    throw new ProtocolError("detached_worktree_commit_mismatch", `${head.stdout} != ${protocolCommit}`);
  }
}

export async function snapshotDirectory(root) {
  const records = [];
  async function walk(current) {
    const entries = await readdir(current, { withFileTypes: true });
    for (const entry of entries.sort((left, right) => left.name.localeCompare(right.name))) {
      const absolute = path.join(current, entry.name);
      const relative = path.relative(root, absolute).replaceAll("\\", "/");
      if (entry.isSymbolicLink()) {
        throw new ProtocolError("target_reparse_point", relative);
      }
      if (entry.isDirectory()) {
        records.push({ path: relative, type: "directory" });
        await walk(absolute);
      } else if (entry.isFile()) {
        const bytes = await readFile(absolute);
        records.push({ path: relative, type: "file", bytes: bytes.length, sha256: sha256Bytes(bytes) });
      } else {
        throw new ProtocolError("target_special_file", relative);
      }
    }
  }
  try {
    await walk(root);
  } catch (error) {
    if (error.code === "ENOENT") {
      const value = { exists: false, records: [] };
      return { ...value, digest: sha256Bytes(canonicalJson(value)) };
    }
    throw error;
  }
  const value = { exists: true, records };
  return { ...value, digest: sha256Bytes(canonicalJson(value)) };
}

const DEFAULT_DEPENDENCIES = Object.freeze({
  uuidFn: randomUUID,
  nowFn: () => new Date().toISOString(),
  processEnv: process.env,
  resolveProtocolCommitFn: resolveProtocolCommitProduction,
  captureEnvironmentFn: captureEnvironmentProduction,
  createDetachedWorktreeFn: createDetachedWorktreeProduction,
  runAttemptFn: runAttempt,
  restoreAttemptWorktreeFn: installState,
  writeJsonFn: writeAtomicJsonExclusive,
  afterDurableWriteFn: async () => {},
  afterAttemptObservedFn: async () => {},
});

function canonicalJson(value) {
  return Buffer.from(`${JSON.stringify(value, null, 2)}\n`, "utf8");
}

function sha256Bytes(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

async function writeNewJson(target, value, deps) {
  await deps.writeJsonFn(target, value);
  await deps.afterDurableWriteFn({ target, value });
}

async function publishJsonIdempotent(target, value, deps) {
  try {
    await writeNewJson(target, value, deps);
    return;
  } catch (error) {
    if (error.kind !== "duplicate_artifact") throw error;
  }
  const existing = await readFile(target);
  if (!existing.equals(canonicalJson(value))) {
    throw new ProtocolError("immutable_projection_conflict", target);
  }
}

async function readLedger(sessionDir) {
  const directory = path.join(sessionDir, "ledger");
  let names;
  try {
    names = (await readdir(directory)).filter((name) => /^\d{6}\.json$/.test(name)).sort();
  } catch (error) {
    if (error.code === "ENOENT") return [];
    throw error;
  }
  const entries = [];
  for (let index = 0; index < names.length; index += 1) {
    const expected = `${String(index + 1).padStart(6, "0")}.json`;
    if (names[index] !== expected) throw new ProtocolError("ledger_sequence_gap", `${expected} != ${names[index]}`);
    const entry = JSON.parse(await readFile(path.join(directory, names[index]), "utf8"));
    if (entry.sequence !== index + 1) throw new ProtocolError("ledger_sequence_mismatch", names[index]);
    entries.push(entry);
  }
  return entries;
}

async function appendLedger(sessionDir, value, deps) {
  const sequence = (await readLedger(sessionDir)).length + 1;
  const entry = { schemaVersion: 1, sequence, recordedAt: deps.nowFn(), ...value };
  await writeNewJson(
    path.join(sessionDir, "ledger", `${String(sequence).padStart(6, "0")}.json`),
    entry,
    deps,
  );
  return entry;
}

function latestRetryState(entries) {
  return [...entries].reverse().find((entry) => entry.retryState)?.retryState
    ?? { ...INITIAL_RETRY_STATE };
}

function attemptRows(entries) {
  return entries.filter((entry) => entry.type === "attempt_finished").map((entry) => ({
    attemptId: entry.attemptId,
    status: entry.kind,
    startedAt: entry.startedAt,
    endedAt: entry.endedAt,
    reasons: entry.reasons,
    worktree: entry.worktree,
    targetDirectory: entry.targetDirectory,
    environment: entry.environment,
    resultPath: entry.resultPath,
  }));
}

function snapshot(manifest, entries, status, classification = null) {
  return {
    schemaVersion: 1,
    sessionDir: manifest.sessionDir,
    status,
    classification,
    retryState: latestRetryState(entries),
    attempts: attemptRows(entries),
  };
}

function decisionValue({ result, classification, retryState }) {
  return {
    schemaVersion: 1,
    classification,
    attemptId: result?.attemptId ?? null,
    unexplainedStabilityInvalidCount: retryState.unexplainedStabilityInvalidCount,
    evaluation: result?.evaluation ?? null,
  };
}

async function materializeTerminal(manifest, entries, deps) {
  const terminal = [...entries].reverse().find((entry) => entry.type === "session_completed");
  if (!terminal?.decision) throw new ProtocolError("terminal_event_incomplete", "session_completed lacks decision");
  const aggregate = {
    schemaVersion: 1,
    sessionId: manifest.sessionId,
    unexplainedStabilityInvalidCount: terminal.retryState.unexplainedStabilityInvalidCount,
    terminal: true,
    attempts: attemptRows(entries),
    events: entries,
  };
  await publishJsonIdempotent(path.join(manifest.sessionDir, "session-ledger.json"), aggregate, deps);
  // decision.json is the final commit marker; its presence proves that the
  // aggregate peer was already published byte-identically.
  await publishJsonIdempotent(path.join(manifest.sessionDir, "decision.json"), terminal.decision, deps);
  return snapshot(manifest, entries, "completed", terminal.decision.classification);
}

async function completeSession({ manifest, result, classification, retryState, deps }) {
  let entries = await readLedger(manifest.sessionDir);
  let terminal = [...entries].reverse().find((entry) => entry.type === "session_completed");
  const decision = decisionValue({ result, classification, retryState });
  if (!terminal) {
    terminal = await appendLedger(manifest.sessionDir, {
      type: "session_completed",
      classification,
      retryState,
      decision,
    }, deps);
    entries = await readLedger(manifest.sessionDir);
  } else if (JSON.stringify(terminal.decision) !== JSON.stringify(decision)) {
    throw new ProtocolError("terminal_recovery_conflict", "durable terminal decision differs");
  }
  return materializeTerminal(manifest, entries, deps);
}

function manifestFromLocator(locatorPath, locatorRecord, environment) {
  return {
    schemaVersion: 1,
    sessionId: locatorRecord.sessionId,
    createdAt: locatorRecord.createdAt,
    mainRoot: locatorRecord.mainRoot,
    protocolRoot: locatorRecord.protocolRoot,
    scratchParent: locatorRecord.scratchParent,
    sessionDir: locatorRecord.sessionDir,
    worktreeParent: locatorRecord.worktreeParent,
    locatorPath,
    locatorRecord,
    locatorSha256: sha256Bytes(canonicalJson(locatorRecord)),
    protocolLockPath: locatorRecord.protocolLockPath,
    protocolLock: locatorRecord.protocolLock,
    protocol: locatorRecord.protocol,
    environment,
  };
}

async function loadManifest(sessionDir, deps, { processAttested = false } = {}) {
  const locatorPath = path.join(path.dirname(sessionDir), LOCATOR_NAME);
  const locatorRecord = JSON.parse(await readFile(locatorPath, "utf8"));
  if (path.resolve(locatorRecord.sessionDir) !== path.resolve(sessionDir)) {
    throw new ProtocolError("session_locator_path_mismatch", locatorRecord.sessionDir);
  }
  await mkdir(sessionDir, { recursive: true });
  let manifest = await readOptionalJson(locatorRecord.sessionManifestPath);
  if (!manifest) {
    if (processAttested !== true) {
      throw new ProtocolError("process_attestation_missing", "bootstrap recovery needs fresh quiescence evidence");
    }
    const recoveryNumber = (await readdir(sessionDir, { withFileTypes: true }))
      .filter((entry) => entry.isDirectory() && /^bootstrap-recovery-\d{3}$/.test(entry.name)).length + 1;
    const recoveryDir = path.join(
      sessionDir,
      `bootstrap-recovery-${String(recoveryNumber).padStart(3, "0")}`,
    );
    const environment = await deps.captureEnvironmentFn({
      mainRoot: locatorRecord.mainRoot,
      artifactDir: recoveryDir,
      processAttested: true,
      protocolLock: locatorRecord.protocolLock,
    });
    manifest = manifestFromLocator(locatorPath, locatorRecord, environment);
    await publishJsonIdempotent(locatorRecord.sessionManifestPath, manifest, deps);
  }
  if (
    manifest.locatorRecord.sessionId !== manifest.sessionId
    || path.resolve(manifest.locatorRecord.sessionDir) !== path.resolve(sessionDir)
    || sha256Bytes(canonicalJson(manifest.locatorRecord)) !== manifest.locatorSha256
  ) throw new ProtocolError("session_locator_anchor_mismatch", "manifest locator anchor is invalid");
  await publishJsonIdempotent(manifest.locatorPath, manifest.locatorRecord, deps);
  return manifest;
}

async function assertResumeMaySpawn(sessionDir, processAttested) {
  const manifestExists = await pathExists(path.join(sessionDir, "session-manifest.json"));
  const entries = await readLedger(sessionDir);
  const finished = new Set(
    entries.filter((entry) => entry.type === "attempt_finished").map((entry) => entry.attemptId),
  );
  const unfinishedAttempt = [...entries].reverse().find((entry) =>
    entry.type === "attempt_started" && !finished.has(entry.attemptId),
  );
  const haltedAttempt = unfinishedAttempt && [...entries].reverse().find((entry) =>
    entry.type === "attempt_termination_unconfirmed" && entry.attemptId === unfinishedAttempt.attemptId,
  );
  if ((!manifestExists || unfinishedAttempt) && processAttested !== true) {
    throw new ProtocolError(
      "process_attestation_missing",
      haltedAttempt
        ? `attempt ${haltedAttempt.attemptId} halted with unconfirmed termination; fresh quiescence is required`
        : unfinishedAttempt
          ? `unfinished attempt ${unfinishedAttempt.attemptId} requires fresh quiescence before any child command`
          : "bootstrap recovery requires fresh quiescence before any child command",
    );
  }
}

async function readOptionalJson(filePath) {
  try {
    return JSON.parse(await readFile(filePath, "utf8"));
  } catch (error) {
    if (error.code === "ENOENT") return null;
    throw error;
  }
}

async function readOptionalResult(filePath) {
  try {
    const bytes = await readFile(filePath);
    return {
      result: JSON.parse(bytes.toString("utf8")),
      resultPath: filePath,
      resultSha256: sha256Bytes(bytes),
    };
  } catch (error) {
    if (error.code === "ENOENT") return null;
    throw error;
  }
}

async function readReservedResult(reservation) {
  if (!reservation.sourceResultPath) return null;
  const selected = await readOptionalResult(reservation.sourceResultPath);
  if (!selected || selected.resultSha256 !== reservation.sourceResultSha256) {
    throw new ProtocolError("recovery_source_result_mismatch", reservation.attemptId, {
      expectedPath: reservation.sourceResultPath,
      expectedSha256: reservation.sourceResultSha256,
      actualSha256: selected?.resultSha256 ?? null,
    });
  }
  return selected;
}

async function finishAttempt({ manifest, startedEvent, result, resultPath, environment, deps }) {
  await appendLedger(manifest.sessionDir, {
    type: "attempt_finished",
    attemptId: startedEvent.attemptId,
    kind: result.kind,
    startedAt: startedEvent.recordedAt,
    endedAt: deps.nowFn(),
    reasons: result.reasons ?? [],
    worktree: startedEvent.worktree,
    targetDirectory: startedEvent.targetDirectory,
    environment: environment ?? startedEvent.environment ?? null,
    resultPath,
    classification: result.evaluation?.classification ?? null,
    retryState: startedEvent.retryState,
  }, deps);
  if (result.kind === "valid") {
    const reduced = reduceRetry(startedEvent.retryState, { kind: "valid", objectiveCauseCorrected: false });
    return completeSession({
      manifest,
      result,
      classification: result.evaluation.classification,
      retryState: reduced.state,
      deps,
    });
  }
  const entries = await readLedger(manifest.sessionDir);
  return snapshot(
    manifest,
    entries,
    result.kind === "stability_invalid" ? "awaiting_stability_disposition" : "awaiting_correction",
  );
}

function coordinatorArtifactPath(manifest, attemptId, name) {
  return path.join(manifest.sessionDir, "coordinator", attemptId, name);
}

function finalAProven(manifest, result) {
  return result?.finalState?.kind === "A"
    && result.finalState?.srcTauriTree === manifest.protocolLock.states.A.srcTauriTree;
}

function environmentInvariantProjection(environment) {
  const names = [
    "platform",
    "architecture",
    "host",
    "cargo",
    "rustc",
    "node",
    "cargoEnvironment",
    "mainRoot",
    "mainSrcTauriTree",
    "mainTargetDirectory",
    "mainTargetSnapshot",
  ];
  return Object.fromEntries(names.map((name) => [name, environment?.[name] ?? null]));
}

function operationalEnvironmentProjection(environment) {
  let defender = environment?.defender ?? null;
  try {
    const parsed = JSON.parse(defender);
    defender = {
      RealTimeProtectionEnabled: parsed.RealTimeProtectionEnabled ?? null,
      AntivirusEnabled: parsed.AntivirusEnabled ?? null,
    };
  } catch {
    // Preserve an unavailable/access-denied string, but intentionally ignore
    // naturally drifting QuickScanAge when structured status is available.
  }
  return { power: environment?.power ?? null, defender };
}

function assertAttemptEnvironmentCompatible(entries, current) {
  const baseline = entries.find((entry) => entry.type === "attempt_environment");
  if (!baseline) return { environmentBaseline: true, correctedEnvironmentDelta: null };
  const expected = environmentInvariantProjection(baseline.environment);
  const actual = environmentInvariantProjection(current);
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new ProtocolError("attempt_environment_drift", "attempt preflight differs from first measurement baseline", {
      expected,
      actual,
    });
  }
  const operationalExpected = operationalEnvironmentProjection(baseline.environment);
  const operationalActual = operationalEnvironmentProjection(current);
  const operationalChanged = JSON.stringify(operationalActual) !== JSON.stringify(operationalExpected);
  const disposition = [...entries].reverse().find((entry) => entry.type === "retry_disposition");
  const correctedCause = typeof disposition?.correctedCause === "string"
    ? disposition.correctedCause.trim()
    : "";
  if (operationalChanged && correctedCause === "") {
    throw new ProtocolError(
      "attempt_environment_drift",
      "power/Defender drift requires the immediately preceding corrected-cause disposition",
      { operationalExpected, operationalActual },
    );
  }
  return {
    environmentBaseline: false,
    correctedEnvironmentDelta: operationalChanged
      ? { correctedCause, before: operationalExpected, after: operationalActual }
      : null,
  };
}

async function pathExists(filePath) {
  try {
    await access(filePath);
    return true;
  } catch (error) {
    if (error.code === "ENOENT") return false;
    throw error;
  }
}

async function normalizeResultArtifact({ manifest, startedEvent, result, resultPath, recoveredFinalState, deps }) {
  if (result.kind === "infrastructure_invalid") return { result, resultPath };
  if (finalAProven(manifest, result)) return { result, resultPath };
  const wrapperPath = coordinatorArtifactPath(manifest, startedEvent.attemptId, "coordinator-failure.json");
  const wrapper = {
    schemaVersion: 1,
    attemptId: startedEvent.attemptId,
    kind: "infrastructure_invalid",
    reasons: ["final_restore_evidence_missing"],
    evaluation: null,
    finalState: recoveredFinalState ?? result.finalState ?? null,
    blocks: result.blocks ?? {},
    sourceResultPath: resultPath,
    error: { kind: "final_restore_evidence_missing" },
  };
  await publishJsonIdempotent(wrapperPath, wrapper, deps);
  return { result: wrapper, resultPath: wrapperPath };
}

async function ensureAttemptRecovered(manifest, startedEvent, selected, deps, processAttested) {
  let entries = await readLedger(manifest.sessionDir);
  const completed = [...entries].reverse().find((entry) =>
    entry.type === "attempt_recovery_completed" && entry.attemptId === startedEvent.attemptId,
  );
  if (completed) return completed;
  if (processAttested !== true) {
    throw new ProtocolError("process_attestation_missing", "unfinished attempt recovery needs fresh quiescence evidence");
  }
  const recoveryNumber = entries.filter((entry) =>
    entry.type === "attempt_recovery_started" && entry.attemptId === startedEvent.attemptId,
  ).length + 1;
  const recoveryId = `recovery-${String(recoveryNumber).padStart(3, "0")}`;
  const recoveryDir = coordinatorArtifactPath(manifest, startedEvent.attemptId, path.join("recoveries", recoveryId));
  await appendLedger(manifest.sessionDir, {
    type: "attempt_recovery_started",
    attemptId: startedEvent.attemptId,
    recoveryId,
    worktree: startedEvent.worktree,
    sourceResultPath: selected?.resultPath ?? null,
    sourceResultSha256: selected?.resultSha256 ?? null,
    retryState: startedEvent.retryState,
  }, deps);
  const environment = await deps.captureEnvironmentFn({
    mainRoot: manifest.mainRoot,
    artifactDir: recoveryDir,
    processAttested: true,
    protocolLock: manifest.protocolLock,
  });
  entries = await readLedger(manifest.sessionDir);
  const worktreeCreated = entries.some((entry) =>
    entry.type === "worktree_created" && entry.attemptId === startedEvent.attemptId,
  );
  const exists = await pathExists(startedEvent.worktree);
  if (!exists && worktreeCreated) {
    throw new ProtocolError("created_worktree_missing", startedEvent.worktree);
  }
  let finalState = null;
  if (exists) {
    finalState = await deps.restoreAttemptWorktreeFn({
      state: "A-final",
      worktree: startedEvent.worktree,
      mainRoot: manifest.mainRoot,
      protocolLock: manifest.protocolLock,
      artifactDir: recoveryDir,
    });
    if (!finalAProven(manifest, { finalState })) {
      throw new ProtocolError("recovery_a_mismatch", startedEvent.attemptId, { finalState });
    }
  }
  return appendLedger(manifest.sessionDir, {
    type: "attempt_recovery_completed",
    attemptId: startedEvent.attemptId,
    recoveryId,
    worktree: startedEvent.worktree,
    worktreeAbsent: !exists,
    finalState,
    environment,
    retryState: startedEvent.retryState,
  }, deps);
}

async function recoverStartedAttempt(manifest, startedEvent, deps, { processAttested = false } = {}) {
  const attemptDir = path.join(manifest.sessionDir, "attempts", startedEvent.attemptId);
  const entries = await readLedger(manifest.sessionDir);
  const environment = [...entries].reverse().find((entry) =>
    entry.type === "attempt_environment" && entry.attemptId === startedEvent.attemptId,
  )?.environment ?? startedEvent.environment ?? null;
  const normalPath = path.join(attemptDir, "attempt-result.json");
  const failurePath = coordinatorArtifactPath(manifest, startedEvent.attemptId, "coordinator-failure.json");
  const interruptionPath = coordinatorArtifactPath(manifest, startedEvent.attemptId, "coordinator-interruption.json");
  const recoveryReservation = [...entries].reverse().find((entry) =>
    entry.type === "attempt_recovery_started" && entry.attemptId === startedEvent.attemptId,
  );
  // A durable coordinator failure is terminal evidence and always outranks an
  // earlier recovery reservation. This makes failure publication replay-safe.
  let selected = await readOptionalResult(failurePath)
    ?? (recoveryReservation
      ? await readReservedResult(recoveryReservation)
      : await readOptionalResult(interruptionPath)
        ?? await readOptionalResult(normalPath));
  let result = selected?.result ?? null;
  let resultPath = selected?.resultPath ?? null;
  let recovery = null;
  if (!result || !finalAProven(manifest, result)) {
    recovery = await ensureAttemptRecovered(manifest, startedEvent, selected, deps, processAttested);
  }
  if (!result) {
    resultPath = interruptionPath;
    result = {
      schemaVersion: 1,
      attemptId: startedEvent.attemptId,
      kind: "infrastructure_invalid",
      reasons: ["coordinator_interrupted"],
      evaluation: null,
      finalState: recovery.finalState,
      blocks: {},
      recoveryId: recovery.recoveryId,
      worktreeAbsent: recovery.worktreeAbsent,
      error: { kind: "coordinator_interrupted" },
    };
    await publishJsonIdempotent(resultPath, result, deps);
  }
  if (result.attemptId !== startedEvent.attemptId) {
    throw new ProtocolError("attempt_result_identity_mismatch", startedEvent.attemptId);
  }
  const normalized = await normalizeResultArtifact({
    manifest,
    startedEvent,
    result,
    resultPath,
    recoveredFinalState: recovery?.finalState ?? null,
    deps,
  });
  return finishAttempt({
    manifest,
    startedEvent,
    result: normalized.result,
    resultPath: normalized.resultPath,
    environment: environment ?? recovery?.environment ?? null,
    deps,
  });
}

async function launchAttempt(manifest, retryState, deps, processAttested) {
  assertCargoTargetDirUnset(deps.processEnv);
  const entries = await readLedger(manifest.sessionDir);
  const attemptNumber = entries.filter((entry) => entry.type === "attempt_started").length + 1;
  const attemptId = `attempt-${String(attemptNumber).padStart(3, "0")}`;
  const attemptDir = path.join(manifest.sessionDir, "attempts", attemptId);
  const worktree = path.join(manifest.worktreeParent, attemptId);
  const targetDirectory = path.join(worktree, "src-tauri", "target");
  await mkdir(path.dirname(attemptDir), { recursive: true });
  await mkdir(path.dirname(worktree), { recursive: true });
  await assertMissing(attemptDir, "attempt_directory_exists");
  await assertMissing(worktree, "attempt_worktree_exists");
  await assertMissing(targetDirectory, "fresh_target_already_exists");
  if (processAttested !== true) {
    throw new ProtocolError("process_attestation_missing", "every fresh attempt needs a new quiescence attestation");
  }
  const startedEvent = await appendLedger(manifest.sessionDir, {
    type: "attempt_started",
    attemptId,
    startedAt: deps.nowFn(),
    worktree,
    targetDirectory,
    processAttested: true,
    environment: null,
    retryState,
  }, deps);

  let coordinatorFailure = null;
  try {
    await mkdir(attemptDir, { recursive: false });
    const attemptEnvironment = await deps.captureEnvironmentFn({
      mainRoot: manifest.mainRoot,
      artifactDir: attemptDir,
      processAttested: true,
      protocolLock: manifest.protocolLock,
    });
    const environmentDisposition = assertAttemptEnvironmentCompatible(
      await readLedger(manifest.sessionDir),
      attemptEnvironment,
    );
    await appendLedger(manifest.sessionDir, {
      type: "attempt_environment",
      attemptId,
      environment: attemptEnvironment,
      ...environmentDisposition,
      retryState,
    }, deps);
    await appendLedger(manifest.sessionDir, {
      type: "worktree_creation_started",
      attemptId,
      worktree,
      retryState,
    }, deps);
    await deps.createDetachedWorktreeFn({
      protocolRoot: manifest.protocolRoot,
      worktree,
      protocolCommit: manifest.protocol.protocolCommit,
      artifactDir: attemptDir,
    });
    await appendLedger(manifest.sessionDir, {
      type: "worktree_created",
      attemptId,
      worktree,
      retryState,
    }, deps);
    await assertMissing(targetDirectory, "fresh_target_already_exists");
    const resultPath = path.join(attemptDir, "attempt-result.json");
    const result = await deps.runAttemptFn({
      worktree,
      mainRoot: manifest.mainRoot,
      sessionDir: manifest.sessionDir,
      attemptId,
      protocolLock: manifest.protocolLock,
    });
    const persisted = JSON.parse(await readFile(resultPath, "utf8"));
    if (JSON.stringify(persisted) !== JSON.stringify(result)) {
      throw new ProtocolError("attempt_return_artifact_mismatch", attemptId);
    }
    await deps.afterAttemptObservedFn({ attemptId, resultPath });
  } catch (error) {
    if (error?.simulatedCrash === true) throw error;
    const terminationUnconfirmed = hasTerminationUnconfirmed(error);
    coordinatorFailure = {
      schemaVersion: 1,
      attemptId,
      kind: "infrastructure_invalid",
      reasons: ["coordinator_failure"],
      evaluation: null,
      finalState: null,
      blocks: {},
      error: {
        name: error?.name ?? "Error",
        kind: error?.kind ?? "coordinator_failure",
        message: error?.message ?? String(error),
      },
    };
    if (terminationUnconfirmed) {
      let markerError = null;
      let failurePublicationError = null;
      try {
        await appendLedger(manifest.sessionDir, {
          type: "attempt_termination_unconfirmed",
          attemptId,
          worktree,
          targetDirectory,
          resultPath: path.join(attemptDir, "attempt-result.json"),
          operatorActionRequired: true,
          retryState,
        }, deps);
      } catch (writeError) {
        markerError = writeError;
      }
      try {
        await publishJsonIdempotent(
          coordinatorArtifactPath(manifest, attemptId, "coordinator-failure.json"),
          coordinatorFailure,
          deps,
        );
      } catch (writeError) {
        failurePublicationError = writeError;
      }
      throw new ProtocolError("termination_unconfirmed", attemptId, {
        operatorActionRequired: true,
        resultPath: path.join(attemptDir, "attempt-result.json"),
        markerError: markerError
          ? { kind: markerError.kind ?? markerError.name, message: markerError.message }
          : null,
        failurePublicationError: failurePublicationError
          ? { kind: failurePublicationError.kind ?? failurePublicationError.name, message: failurePublicationError.message }
          : null,
      });
    }
    await publishJsonIdempotent(
      coordinatorArtifactPath(manifest, attemptId, "coordinator-failure.json"),
      coordinatorFailure,
      deps,
    );
  }
  return recoverStartedAttempt(manifest, startedEvent, deps, { processAttested: true });
}

export async function startSession(options, overrides = {}) {
  const deps = { ...DEFAULT_DEPENDENCIES, ...overrides };
  if (options.processAttested !== true) {
    throw new ProtocolError("process_attestation_missing", "operator must attest build-process quiescence");
  }
  assertCargoTargetDirUnset(deps.processEnv);
  const mainRoot = path.resolve(options.mainRoot);
  const protocolRoot = path.resolve(options.protocolRoot);
  const scratchParent = path.resolve(options.scratchParent);
  const locatorPath = path.join(scratchParent, LOCATOR_NAME);
  await mkdir(scratchParent, { recursive: true });
  await assertMissing(locatorPath, "session_locator_exists");
  const orphanSessions = (await readdir(scratchParent, { withFileTypes: true }))
    .filter((entry) => entry.isDirectory() && entry.name.startsWith("process-shell-session-"));
  if (orphanSessions.length) {
    throw new ProtocolError("orphan_session_exists", "resume or audit the existing session directory", {
      sessions: orphanSessions.map((entry) => path.join(scratchParent, entry.name)),
    });
  }
  const sessionId = deps.uuidFn();
  const sessionDir = path.join(scratchParent, `process-shell-session-${sessionId}`);
  const protocolLockPath = path.join(
    protocolRoot,
    "scripts",
    "process-shell-diagnostic",
    "protocol-lock.json",
  );
  const protocolLock = JSON.parse(await readFile(protocolLockPath, "utf8"));
  const bootstrapDir = path.join(scratchParent, `bootstrap-${sessionId}`);
  const protocolCommit = await deps.resolveProtocolCommitFn({ protocolRoot, artifactDir: bootstrapDir });
  const locatorRecord = {
    schemaVersion: 1,
    sessionId,
    createdAt: deps.nowFn(),
    mainRoot,
    protocolRoot,
    scratchParent,
    sessionDir,
    worktreeParent: path.join(mainRoot, ".worktrees", `process-shell-session-${sessionId}`),
    sessionManifestPath: path.join(sessionDir, "session-manifest.json"),
    protocolLockPath,
    protocolLock,
    protocol: { protocolCommit },
  };
  // The external locator is the bootstrap reservation WAL. Nothing creates the
  // final session directory until this exact recovery seed is durable.
  await publishJsonIdempotent(locatorPath, locatorRecord, deps);
  await mkdir(sessionDir, { recursive: true });
  const environment = await deps.captureEnvironmentFn({
    mainRoot,
    artifactDir: path.join(sessionDir, "bootstrap"),
    processAttested: true,
    protocolLock,
  });
  const manifest = manifestFromLocator(locatorPath, locatorRecord, environment);
  await publishJsonIdempotent(locatorRecord.sessionManifestPath, manifest, deps);
  await appendLedger(sessionDir, {
    type: "session_started",
    sessionId,
    protocolCommit,
    retryState: { ...INITIAL_RETRY_STATE },
  }, deps);
  return launchAttempt(manifest, { ...INITIAL_RETRY_STATE }, deps, true);
}

export async function resumeSession(options, overrides = {}) {
  const deps = { ...DEFAULT_DEPENDENCIES, ...overrides };
  const sessionDir = path.resolve(options.sessionDir);
  // This filesystem-only gate runs before Task 6's Git-backed loadManifest
  // verifier. A possibly live descendant is never followed by another child
  // command until the operator supplies a new quiescence attestation.
  await assertResumeMaySpawn(sessionDir, options.processAttested === true);
  const manifest = await loadManifest(sessionDir, deps, { processAttested: options.processAttested === true });
  let entries = await readLedger(sessionDir);
  if (entries.some((entry) => entry.type === "session_completed")) {
    return materializeTerminal(manifest, entries, deps);
  }
  if (!entries.some((entry) => entry.type === "session_started")) {
    if (options.processAttested !== true) {
      throw new ProtocolError("process_attestation_missing", "bootstrap recovery needs a fresh quiescence attestation");
    }
    await appendLedger(sessionDir, {
      type: "session_started",
      sessionId: manifest.sessionId,
      protocolCommit: manifest.protocol.protocolCommit,
      retryState: { ...INITIAL_RETRY_STATE },
    }, deps);
    entries = await readLedger(sessionDir);
  }
  const finishedIds = new Set(entries.filter((entry) => entry.type === "attempt_finished").map((entry) => entry.attemptId));
  const unfinished = [...entries].reverse().find((entry) =>
    entry.type === "attempt_started" && !finishedIds.has(entry.attemptId),
  );
  if (unfinished) {
    return recoverStartedAttempt(manifest, unfinished, deps, {
      processAttested: options.processAttested === true,
    });
  }

  const lastAttempt = [...entries].reverse().find((entry) => entry.type === "attempt_finished");
  if (!lastAttempt) {
    if (options.processAttested !== true) {
      throw new ProtocolError("process_attestation_missing", "first attempt recovery needs a fresh quiescence attestation");
    }
    return launchAttempt(manifest, { ...INITIAL_RETRY_STATE }, deps, true);
  }
  const retryState = latestRetryState(entries);
  if (lastAttempt.kind === "valid") {
    const result = JSON.parse(await readFile(lastAttempt.resultPath, "utf8"));
    const reduced = reduceRetry(lastAttempt.retryState, { kind: "valid", objectiveCauseCorrected: false });
    return completeSession({
      manifest,
      result,
      classification: result.evaluation.classification,
      retryState: reduced.state,
      deps,
    });
  }
  const correctedCause = typeof options.correctedCause === "string" && options.correctedCause.trim()
    ? options.correctedCause.trim()
    : null;
  const unexplained = options.unexplainedStability === true;
  if (correctedCause && unexplained) {
    throw new ProtocolError("retry_disposition_ambiguous", "choose corrected cause or unexplained stability");
  }
  const priorDisposition = entries.find((entry) =>
    entry.type === "retry_disposition" && entry.attemptId === lastAttempt.attemptId,
  );
  if (priorDisposition) {
    if (
      (correctedCause && correctedCause !== priorDisposition.correctedCause)
      || (unexplained && priorDisposition.unexplainedStability !== true)
    ) throw new ProtocolError("retry_replay_conflict", lastAttempt.attemptId);
    if (priorDisposition.retryAction === "environment_precision_insufficient") {
      return completeSession({
        manifest,
        result: null,
        classification: "environment_precision_insufficient",
        retryState: priorDisposition.retryState,
        deps,
      });
    }
    if (priorDisposition.retryAction !== "retry") {
      throw new ProtocolError("retry_action_invalid", priorDisposition.retryAction);
    }
    if (options.processAttested !== true) {
      throw new ProtocolError("process_attestation_missing", "recovered retry needs a fresh quiescence attestation");
    }
    return launchAttempt(manifest, priorDisposition.retryState, deps, true);
  }
  if (lastAttempt.kind === "infrastructure_invalid" && !correctedCause) {
    return snapshot(manifest, entries, "awaiting_correction");
  }
  if (lastAttempt.kind === "stability_invalid" && !correctedCause && !unexplained) {
    return snapshot(manifest, entries, "awaiting_stability_disposition");
  }
  if (lastAttempt.kind === "infrastructure_invalid" && unexplained) {
    throw new ProtocolError("retry_disposition_invalid", "infrastructure failure needs a corrected cause");
  }
  const terminatesForPrecision =
    lastAttempt.kind === "stability_invalid" &&
    unexplained &&
    retryState.unexplainedStabilityInvalidCount >= 1;
  if (!terminatesForPrecision && options.processAttested !== true) {
    throw new ProtocolError("process_attestation_missing", "a retry needs a fresh quiescence attestation");
  }
  const reduced = reduceRetry(retryState, {
    kind: lastAttempt.kind,
    objectiveCauseCorrected: correctedCause !== null,
  });
  await appendLedger(sessionDir, {
    type: "retry_disposition",
    attemptId: lastAttempt.attemptId,
    invalidationKind: lastAttempt.kind,
    correctedCause,
    unexplainedStability: unexplained,
    retryAction: reduced.action,
    retryState: reduced.state,
  }, deps);
  if (reduced.action === "environment_precision_insufficient") {
    return completeSession({
      manifest,
      result: null,
      classification: "environment_precision_insufficient",
      retryState: reduced.state,
      deps,
    });
  }
  if (reduced.action !== "retry") throw new ProtocolError("retry_action_invalid", reduced.action);
  return launchAttempt(manifest, reduced.state, deps, true);
}

function parseFlags(tokens) {
  const values = {};
  const booleans = new Set(["--process-attested", "--unexplained-stability"]);
  for (let index = 0; index < tokens.length; index += 1) {
    const flag = tokens[index];
    if (!flag.startsWith("--") || Object.hasOwn(values, flag)) throw new Error(`invalid or duplicate flag ${flag}`);
    if (booleans.has(flag)) values[flag] = true;
    else {
      if (tokens[index + 1] === undefined || tokens[index + 1].startsWith("--")) throw new Error(`missing value for ${flag}`);
      values[flag] = tokens[index + 1];
      index += 1;
    }
  }
  return values;
}

function required(values, flag) {
  if (!values[flag]) throw new Error(`missing required flag ${flag}`);
  return values[flag];
}

export function parseCli(argv) {
  const [command, ...tokens] = argv;
  const values = parseFlags(tokens);
  if (command === "start") return {
    command,
    options: {
      mainRoot: required(values, "--main-root"),
      protocolRoot: required(values, "--protocol-root"),
      scratchParent: required(values, "--scratch-parent"),
      processAttested: values["--process-attested"] === true,
    },
  };
  if (command === "resume") {
    const options = {
      sessionDir: required(values, "--session-dir"),
      unexplainedStability: values["--unexplained-stability"] === true,
      processAttested: values["--process-attested"] === true,
    };
    if (values["--corrected-cause"]) options.correctedCause = values["--corrected-cause"];
    if (!options.unexplainedStability) delete options.unexplainedStability;
    if (!options.processAttested) delete options.processAttested;
    return { command, options };
  }
  throw new Error(`expected start or resume, got ${command ?? "missing"}`);
}

async function main() {
  const parsed = parseCli(process.argv.slice(2));
  const result = parsed.command === "start"
    ? await startSession(parsed.options)
    : await resumeSession(parsed.options);
  process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  await main();
}
