import { spawn } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import { constants } from "node:fs";
import {
  copyFile,
  link,
  mkdir,
  open,
  readFile,
  readdir,
  unlink,
} from "node:fs/promises";
import path from "node:path";
import process from "node:process";

import { PROTOCOL } from "./protocol.mjs";

export class ProtocolError extends Error {
  constructor(kind, message, details = {}) {
    super(message);
    this.name = "ProtocolError";
    this.kind = kind;
    this.details = details;
  }
}

export function hasTerminationUnconfirmed(value, seen = new Set()) {
  if (!value || typeof value !== "object" || seen.has(value)) return false;
  seen.add(value);
  if (value.classification === "termination_unconfirmed" || value.kind === "termination_unconfirmed") {
    return true;
  }
  return Object.values(value).some((entry) => hasTerminationUnconfirmed(entry, seen));
}

export async function sha256File(filePath) {
  const bytes = await readFile(filePath);
  return createHash("sha256").update(bytes).digest("hex");
}

export async function writeAtomicBytesExclusive(target, bytes) {
  await mkdir(path.dirname(target), { recursive: true });
  const temporary = `${target}.${process.pid}.${randomUUID()}.tmp`;
  const handle = await open(temporary, "wx");
  try {
    await handle.writeFile(bytes);
    await handle.sync();
  } finally {
    await handle.close();
  }
  try {
    // A same-directory hard link publishes the fully synced bytes atomically
    // and fails with EEXIST instead of replacing an existing artifact.
    await link(temporary, target);
  } catch (error) {
    if (error.code === "EEXIST") {
      throw new ProtocolError("duplicate_artifact", `artifact already exists: ${target}`);
    }
    throw error;
  } finally {
    await unlink(temporary).catch((error) => {
      if (error.code !== "ENOENT") throw error;
    });
  }
}

export async function writeAtomicJsonExclusive(target, value) {
  return writeAtomicBytesExclusive(target, Buffer.from(`${JSON.stringify(value, null, 2)}\n`, "utf8"));
}

function allowlistedEnvironment(env) {
  const names = [
    "CARGO_BUILD_TARGET",
    "CARGO_ENCODED_RUSTFLAGS",
    "CARGO_INCREMENTAL",
    "CARGO_TARGET_DIR",
    "RUSTFLAGS",
  ];
  const result = {};
  for (const name of names) {
    const entry = Object.entries(env).find(([key]) => key.toUpperCase() === name);
    result[name] = entry?.[1] ?? null;
  }
  result.cargo_log_enabled = Object.keys(env).some((key) => key.toUpperCase() === "CARGO_LOG");
  return result;
}

function closeResult(child) {
  return new Promise((resolve) => {
    let spawnError = null;
    let exitObserved = false;
    child.once("error", (error) => { spawnError = error.message; });
    child.once("exit", () => { exitObserved = true; });
    child.once("close", (exitCode, signal) => resolve({
      exitCode,
      signal,
      spawnError,
      exitObserved,
      closeObserved: true,
    }));
  });
}

async function bounded(promise, timeoutMs, timeoutValue) {
  let timer;
  try {
    return await Promise.race([
      promise,
      new Promise((resolve) => {
        timer = setTimeout(() => resolve(timeoutValue), timeoutMs);
      }),
    ]);
  } finally {
    if (timer) clearTimeout(timer);
  }
}

function processAlive(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    if (error.code === "ESRCH") return false;
    throw error;
  }
}

async function captureOwnedWindowsPids(rootPid, cwd, env) {
  const script = [
    `$root = [int]${rootPid}`,
    "$all = @(Get-CimInstance Win32_Process | Select-Object ProcessId,ParentProcessId)",
    "$ids = @($root)",
    "do {",
    "  $next = @($all | Where-Object { $ids -contains [int]$_.ParentProcessId -and $ids -notcontains [int]$_.ProcessId } | ForEach-Object { [int]$_.ProcessId })",
    "  $ids += $next",
    "} while ($next.Count -gt 0)",
    "$ids | Sort-Object -Unique | ConvertTo-Json -Compress",
  ].join("; ");
  const child = spawn(path.join(env.SystemRoot, "System32", "WindowsPowerShell", "v1.0", "powershell.exe"), [
    "-NoLogo", "-NoProfile", "-NonInteractive", "-Command", script,
  ], { cwd, env, shell: false, windowsHide: true, stdio: ["ignore", "pipe", "pipe"] });
  let stdout = "";
  let stderr = "";
  child.stdout.on("data", (chunk) => { stdout += chunk.toString("utf8"); });
  child.stderr.on("data", (chunk) => { stderr += chunk.toString("utf8"); });
  const closed = closeResult(child);
  const result = await bounded(closed, 15_000, { timedOut: true });
  if (result.timedOut) {
    child.kill("SIGKILL");
    await bounded(closed, 5_000, null);
    throw new ProtocolError("owned_tree_inventory_timeout", `root ${rootPid}`);
  }
  if (result.exitCode !== 0) throw new ProtocolError("owned_tree_inventory_failed", stderr.trim());
  const parsed = JSON.parse(stdout.trim());
  return [...new Set((Array.isArray(parsed) ? parsed : [parsed]).map(Number))];
}

async function runTaskkill({ pid, cwd, env, taskkillExe, artifactDir, label }) {
  const stdoutPath = path.join(artifactDir, "runs", `${label}.stdout.log`);
  const stderrPath = path.join(artifactDir, "runs", `${label}.stderr.log`);
  const stdout = await open(stdoutPath, "wx");
  const stderr = await open(stderrPath, "wx");
  const args = ["/PID", String(pid), "/T", "/F"];
  let child;
  try {
    child = spawn(taskkillExe, args, {
      cwd,
      env,
      shell: false,
      windowsHide: true,
      stdio: ["ignore", stdout.fd, stderr.fd],
    });
    const closed = closeResult(child);
    let result = await bounded(closed, 15_000, { taskkillTimedOut: true });
    if (result.taskkillTimedOut) {
      child.kill("SIGKILL");
      result = {
        ...result,
        afterFallback: await bounded(closed, 5_000, { closeObserved: false }),
      };
    }
    return { ...result, args, stdoutPath, stderrPath };
  } finally {
    await stdout.close();
    await stderr.close();
  }
}

async function survivingPids(pids, timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  let survivors = pids.filter(processAlive);
  while (survivors.length && Date.now() < deadline) {
    await new Promise((resolve) => setTimeout(resolve, 100));
    survivors = pids.filter(processAlive);
  }
  return survivors;
}

export async function terminateWindowsTree(
  { pid, cwd, env, taskkillExe, artifactDir, label },
  injected = {},
) {
  const captureFn = injected.captureOwnedWindowsPidsFn ?? captureOwnedWindowsPids;
  const taskkillFn = injected.runTaskkillFn ?? runTaskkill;
  const survivorsFn = injected.survivingPidsFn ?? survivingPids;
  const aliveFn = injected.processAliveFn ?? processAlive;
  const writeJsonFn = injected.writeJsonFn ?? writeAtomicJsonExclusive;
  const observed = new Set([pid]);
  const inventoryErrors = [];
  const terminationErrors = [];
  let primary = null;
  let survivors = [pid];
  let confirmed = false;
  try {
    try {
      for (const ownedPid of await captureFn(pid, cwd, env)) observed.add(ownedPid);
    } catch (error) {
      inventoryErrors.push({ phase: "pre-kill", kind: error.kind ?? error.name, message: error.message });
    }
    primary = await taskkillFn({
      pid, cwd, env, taskkillExe, artifactDir, label: `${label}.taskkill-primary`,
    });
    for (let pass = 1; pass <= 3; pass += 1) {
      try {
        // ParentProcessId remains queryable after the root exits, so a child
        // born between snapshots is still discovered before proof succeeds.
        for (const ownedPid of await captureFn(pid, cwd, env)) observed.add(ownedPid);
      } catch (error) {
        inventoryErrors.push({ phase: `post-kill-${pass}`, kind: error.kind ?? error.name, message: error.message });
      }
      survivors = await survivorsFn([...observed], 1_000);
      if (survivors.length === 0 && inventoryErrors.length === 0) {
        await new Promise((resolve) => setTimeout(resolve, 250));
        try {
          for (const ownedPid of await captureFn(pid, cwd, env)) observed.add(ownedPid);
        } catch (error) {
          inventoryErrors.push({ phase: `confirm-${pass}`, kind: error.kind ?? error.name, message: error.message });
        }
        survivors = [...observed].filter(aliveFn);
        if (survivors.length === 0 && inventoryErrors.length === 0) {
          confirmed = true;
          break;
        }
      }
      for (const survivor of [...survivors].reverse()) {
        await taskkillFn({
          pid: survivor,
          cwd,
          env,
          taskkillExe,
          artifactDir,
          label: `${label}.taskkill-pass-${pass}-${survivor}`,
        });
      }
    }
  } catch (error) {
    terminationErrors.push({
      phase: "termination-proof",
      kind: error.kind ?? error.name,
      message: error.message ?? String(error),
    });
    confirmed = false;
  }
  const evidence = {
    ...(primary ?? {}),
    args: ["/PID", String(pid), "/T", "/F"],
    observedPids: [...observed].sort((left, right) => left - right),
    survivors,
    inventoryErrors,
    terminationErrors,
    confirmed: confirmed && terminationErrors.length === 0,
  };
  if (!evidence.confirmed) {
    try {
      await writeJsonFn(path.join(artifactDir, "runs", `${label}.termination-unconfirmed.json`), {
        schemaVersion: 1,
        rootPid: pid,
        ...evidence,
        operatorActionRequired: true,
      });
    } catch (error) {
      evidence.evidencePublicationError = {
        kind: error.kind ?? error.name,
        message: error.message ?? String(error),
      };
    }
  }
  return evidence;
}

export async function runWindowsProcess({
  label,
  command,
  args,
  cwd,
  env,
  artifactDir,
  timeoutMs,
  taskkillExe,
}) {
  const runs = path.join(artifactDir, "runs");
  await mkdir(runs, { recursive: true });
  const stdoutPath = path.join(runs, `${label}.stdout.log`);
  const stderrPath = path.join(runs, `${label}.stderr.log`);
  const metaPath = path.join(runs, `${label}.process.json`);
  await writeAtomicJsonExclusive(path.join(runs, `${label}.intent.json`), {
    schema_version: 1,
    label,
    command,
    args,
    cwd,
    environment: allowlistedEnvironment(env),
  });

  const stdout = await open(stdoutPath, "wx");
  const stderr = await open(stderrPath, "wx");
  const startedAt = new Date().toISOString();
  const startedNs = process.hrtime.bigint();
  let child;
  let outcome;
  let timedOut = false;
  let taskkill = null;
  const streamCloseErrors = [];
  try {
    child = spawn(command, args, {
      cwd,
      env,
      shell: false,
      windowsHide: true,
      stdio: ["ignore", stdout.fd, stderr.fd],
    });
    const closed = closeResult(child);
    const first = await bounded(
      closed.then((value) => ({ kind: "closed", value })),
      timeoutMs,
      { kind: "timeout" },
    );
    if (first.kind === "closed") {
      outcome = first.value;
    } else {
      timedOut = true;
      if (!child.pid) {
        outcome = await closed;
      } else {
        try {
          taskkill = await terminateWindowsTree({
            pid: child.pid,
            cwd,
            env,
            taskkillExe,
            artifactDir,
            label,
          });
        } catch (error) {
          // Termination evidence must dominate unexpected helper failures too.
          taskkill = {
            confirmed: false,
            terminationErrors: [{
              phase: "terminateWindowsTree-call",
              kind: error.kind ?? error.name,
              message: error.message ?? String(error),
            }],
          };
        }
        outcome = await bounded(
          closed,
          10_000,
          { closeObserved: false, exitObserved: false, exitCode: null, signal: null, spawnError: null },
        );
      }
    }
  } finally {
    for (const [stream, handle] of [["stdout", stdout], ["stderr", stderr]]) {
      try {
        await handle.close();
      } catch (error) {
        streamCloseErrors.push({
          stream,
          kind: error.kind ?? error.name,
          message: error.message ?? String(error),
        });
      }
    }
  }

  const elapsedMs = Number(process.hrtime.bigint() - startedNs) / 1_000_000;
  let classification = "ok";
  if (outcome.spawnError) classification = "spawn_error";
  else if (!outcome.closeObserved) classification = "termination_unconfirmed";
  else if (timedOut && (!taskkill || taskkill.confirmed !== true)) classification = "termination_unconfirmed";
  else if (timedOut) classification = "timeout";
  else if (outcome.exitCode !== 0) classification = "command_failed";

  const result = {
    schemaVersion: 1,
    label,
    command,
    args,
    cwd,
    startedAt,
    endedAt: new Date().toISOString(),
    elapsedMs,
    pid: child?.pid ?? null,
    exitCode: outcome.exitCode ?? null,
    signal: outcome.signal ?? null,
    spawnError: outcome.spawnError ?? null,
    exitObserved: outcome.exitObserved ?? false,
    closeObserved: outcome.closeObserved ?? false,
    timedOut,
    taskkill,
    streamCloseErrors,
    operatorActionRequired: classification === "termination_unconfirmed",
    stdoutPath,
    stderrPath,
    classification,
  };
  try {
    await writeAtomicJsonExclusive(metaPath, result);
  } catch (error) {
    if (classification !== "termination_unconfirmed") throw error;
    result.evidencePublicationError = {
      kind: error.kind ?? error.name,
      message: error.message ?? String(error),
    };
  }
  if (classification !== "termination_unconfirmed" && streamCloseErrors.length > 0) {
    throw new ProtocolError("process_log_close_failed", "stdout/stderr artifact close failed", {
      streamCloseErrors,
      metaPath,
    });
  }
  return result;
}

export function cargoInvocation({ diagnostic, baseEnv = process.env }) {
  const env = Object.fromEntries(
    Object.entries(baseEnv).filter(([key]) => key.toUpperCase() !== "CARGO_LOG"),
  );
  const args = [...PROTOCOL.cargoArgs];
  if (diagnostic) {
    args.push("--timings", "-vv");
    env.CARGO_LOG = "cargo::core::compiler::fingerprint=info";
  }
  return { args, env };
}

export function parseCargoOutput(output) {
  const durations = [...output.matchAll(
    /Finished[^\r\n]*? in (?:(\d+)h\s*)?(?:(\d+)m\s*)?([0-9.]+)s/g,
  )].map((match) => (
    Number(match[1] ?? 0) * 3_600_000
    + Number(match[2] ?? 0) * 60_000
    + Number(match[3]) * 1_000
  ));
  const checkedPackages = [...output.matchAll(/^\s*Checking\s+([^\s]+)/gm)].map((match) => match[1]);
  const extractumLibRustcLines = output.split(/\r?\n/).filter((line) =>
    /\bRunning\b/.test(line) && /--crate-name\s+extractum_lib\b/.test(line),
  );
  return {
    cargoReportedMs: durations.length === 1 ? durations[0] : null,
    checkedPackages: [...new Set(checkedPackages)],
    extractumChecked: checkedPackages.includes(PROTOCOL.expectedCheckedPackage),
    extractumLibRustcObserved: extractumLibRustcLines.length > 0,
    extractumProcessExtern: extractumLibRustcLines.some((line) =>
      /--extern\s+extractum_process(?:=|\s)/.test(line),
    ),
  };
}

async function timingFiles(worktree) {
  const directory = path.join(worktree, "src-tauri", "target", "cargo-timings");
  try {
    return new Set((await readdir(directory)).filter((name) => /^cargo-timing-.+\.html$/.test(name)));
  } catch (error) {
    if (error.code === "ENOENT") return new Set();
    throw error;
  }
}

export async function runCargoCheck({
  label,
  worktree,
  artifactDir,
  cargoExe,
  taskkillExe,
  timeoutMs,
  diagnostic = false,
}) {
  const beforeTimings = diagnostic ? await timingFiles(worktree) : new Set();
  const invocation = cargoInvocation({ diagnostic });
  const processResult = await runWindowsProcess({
    label,
    command: cargoExe,
    args: invocation.args,
    cwd: worktree,
    env: invocation.env,
    artifactDir,
    timeoutMs,
    taskkillExe,
  });
  // Once unconfirmed termination is observed, do not parse logs, copy
  // timings, or publish a derived Cargo artifact in this invocation.
  if (hasTerminationUnconfirmed(processResult)) assertCommandOk(processResult, label);
  const stdout = await readFile(processResult.stdoutPath, "utf8");
  const stderr = await readFile(processResult.stderrPath, "utf8");
  const parsed = parseCargoOutput(`${stdout}\n${stderr}`);
  let timingArtifact = null;
  if (diagnostic && processResult.classification === "ok") {
    const afterTimings = await timingFiles(worktree);
    const created = [...afterTimings].filter((name) => !beforeTimings.has(name));
    if (created.length !== 1) {
      throw new ProtocolError("timing_artifact_count", `expected one timing HTML, got ${created.length}`);
    }
    const source = path.join(worktree, "src-tauri", "target", "cargo-timings", created[0]);
    const target = path.join(artifactDir, "timings", `${label}.html`);
    await mkdir(path.dirname(target), { recursive: true });
    await copyFile(source, target, constants.COPYFILE_EXCL);
    timingArtifact = { path: target, sha256: await sha256File(target) };
  }
  const result = { ...processResult, ...parsed, timingArtifact };
  await writeAtomicJsonExclusive(path.join(artifactDir, "runs", `${label}.cargo.json`), result);
  return result;
}

export function assertCommandOk(result, label, failureKind = "command_failed") {
  if (hasTerminationUnconfirmed(result)) {
    throw new ProtocolError("termination_unconfirmed", label, {
      result,
      operatorActionRequired: true,
    });
  }
  if (
    result?.timedOut === true
    || result?.classification === "timeout"
  ) {
    throw new ProtocolError("command_timeout", label, { result });
  }
  if (result?.classification !== "ok") {
    throw new ProtocolError(failureKind, label, { result });
  }
}

async function fsyncPath(filePath, flags) {
  const handle = await open(filePath, flags);
  try {
    await handle.sync();
  } finally {
    await handle.close();
  }
}

async function restoreSourceFromRecovery({ sourcePath, recoveryPath }) {
  const recoveryBytes = await readFile(recoveryPath);
  const source = await open(sourcePath, "w");
  try {
    await source.writeFile(recoveryBytes);
    await source.sync();
  } finally {
    await source.close();
  }
}

export async function runDirtyCargoProbe({
  label,
  worktree,
  artifactDir,
  sourcePath,
  expectedCanonicalSha256,
  cargoExe,
  taskkillExe,
  timeoutMs,
  diagnostic = false,
  requireExtractum = false,
  runCargoFn = runCargoCheck,
  restoreSourceFn = restoreSourceFromRecovery,
  writeJsonFn = writeAtomicJsonExclusive,
}) {
  if (await sha256File(sourcePath) !== expectedCanonicalSha256) {
    throw new ProtocolError("canonical_hash_mismatch", label);
  }
  const shared = { worktree, artifactDir, cargoExe, taskkillExe, timeoutMs };
  const sync = await runCargoFn({ label: `${label}.sync`, diagnostic: false, ...shared });
  assertCommandOk(sync, label, "canonical_sync_failed");
  if (await sha256File(sourcePath) !== expectedCanonicalSha256) {
    throw new ProtocolError("canonical_hash_mismatch", `${label} after sync`);
  }

  const recoveryPath = path.join(artifactDir, "recovery", `${label}.lib.rs`);
  await mkdir(path.dirname(recoveryPath), { recursive: true });
  await copyFile(sourcePath, recoveryPath, constants.COPYFILE_EXCL);
  await fsyncPath(recoveryPath, "r+");
  if (await sha256File(recoveryPath) !== expectedCanonicalSha256) {
    throw new ProtocolError("recovery_hash_mismatch", label);
  }

  let dirtyResult;
  let dirtyError = null;
  let recoveryPending = false;
  let restorationError = null;
  try {
    const source = await open(sourcePath, "a");
    try {
      await source.writeFile(PROTOCOL.probeSuffix, "utf8");
      await source.sync();
    } finally {
      await source.close();
    }
    dirtyResult = await runCargoFn({ label: `${label}.dirty`, diagnostic, ...shared });
    assertCommandOk(dirtyResult, label, "cargo_failed");
    if (requireExtractum && !dirtyResult.extractumChecked) {
      throw new ProtocolError("extractum_not_checked", label, { dirtyResult });
    }
  } catch (error) {
    dirtyError = error;
    recoveryPending = hasTerminationUnconfirmed(error);
  } finally {
    try {
      await restoreSourceFn({ sourcePath, recoveryPath });
    } catch (error) {
      restorationError = error;
    }
  }

  if (recoveryPending) {
    let pendingPublicationError = null;
    try {
      await writeJsonFn(
        path.join(artifactDir, "recovery", `${label}.recovery-pending.json`),
        {
          schema_version: 1,
          label,
          source_path: sourcePath,
          recovery_path: recoveryPath,
          canonical_sha256: expectedCanonicalSha256,
          recovery_sha256: await sha256File(recoveryPath).catch(() => null),
          source_restored_locally: await sha256File(sourcePath)
            .then((value) => value === expectedCanonicalSha256)
            .catch(() => false),
          operator_action_required: true,
          restoration_error: restorationError
            ? { kind: restorationError.kind ?? restorationError.name, message: restorationError.message }
            : null,
        },
      );
    } catch (error) {
      pendingPublicationError = error;
    }
    throw new ProtocolError("termination_unconfirmed", label, {
      operatorActionRequired: true,
      original: dirtyError?.details ?? { message: dirtyError?.message ?? String(dirtyError) },
      restorationError: restorationError
        ? { kind: restorationError.kind ?? restorationError.name, message: restorationError.message }
        : null,
      pendingPublicationError: pendingPublicationError
        ? { kind: pendingPublicationError.kind ?? pendingPublicationError.name, message: pendingPublicationError.message }
        : null,
    });
  }
  if (restorationError) throw restorationError;
  const restoredSha256 = await sha256File(sourcePath);
  if (restoredSha256 !== expectedCanonicalSha256) {
    throw new ProtocolError("source_restore_failed", label, { recoveryPath, restoredSha256 });
  }
  await writeJsonFn(path.join(artifactDir, "recovery", `${label}.restored.json`), {
    schema_version: 1,
    label,
    recovery_path: recoveryPath,
    canonical_sha256: expectedCanonicalSha256,
    restored_sha256: restoredSha256,
  });
  if (dirtyError) throw dirtyError;
  return dirtyResult;
}
