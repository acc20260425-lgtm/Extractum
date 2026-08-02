import { spawn as defaultSpawn } from "node:child_process";
import {
  createTimingRow as defaultCreateTimingRow,
  formatCommand,
  readHeadCommit as defaultReadHeadCommit,
  recordTimingBestEffort as defaultRecordTimingBestEffort,
} from "./timing-log.mjs";

function exitDetails(exitCode, signal) {
  if (signal) {
    return { exitCode: signal === "SIGINT" ? 130 : 3, signal, termination: "signal" };
  }
  return { exitCode: Number.isInteger(exitCode) ? exitCode : 3, signal: null, termination: "exit" };
}

function timingWarning(warn, error) {
  const message = error instanceof Error ? error.message : String(error);
  warn(`Timing log warning: ${message}`);
}

export async function runObservedCommand({
  command,
  args = [],
  cwd,
  repoRoot = cwd,
  stdio,
  capture = false,
  env,
  mirror = false,
  signal: abortSignal,
  dependencies = {},
}) {
  if (typeof cwd !== "string" || cwd.length === 0) throw new TypeError("cwd");
  if (typeof repoRoot !== "string" || repoRoot.length === 0) throw new TypeError("repoRoot");
  if (abortSignal !== undefined
    && (typeof abortSignal !== "object"
      || typeof abortSignal.addEventListener !== "function"
      || typeof abortSignal.removeEventListener !== "function")) {
    throw new TypeError("signal");
  }
  const spawn = dependencies.spawn ?? defaultSpawn;
  const nowDate = dependencies.nowDate ?? (() => new Date());
  const nowMonotonic = dependencies.nowMonotonic ?? (() => performance.now());
  const readHeadCommit = dependencies.readHeadCommit ?? defaultReadHeadCommit;
  const createTimingRow = dependencies.createTimingRow ?? defaultCreateTimingRow;
  const recordTimingBestEffort = dependencies.recordTimingBestEffort ?? defaultRecordTimingBestEffort;
  const warn = dependencies.warn ?? console.warn;
  const completedCommand = formatCommand(command, args);
  const commit = readHeadCommit(repoRoot);
  const effectiveStdio = capture ? "pipe" : (stdio ?? "inherit");
  const mirrorOutput = capture && (mirror || stdio === "inherit");
  const output = { stdout: [], stderr: [] };
  const spawnOptions = {
    cwd,
    env: { ...process.env, ...env },
    shell: false,
    stdio: effectiveStdio,
  };
  let startedAt;
  let monotonicStart;

  return new Promise((resolve) => {
    let finished = false;
    let removeAbortListener = () => {};
    const finish = async (details) => {
      if (finished) return;
      finished = true;
      removeAbortListener();
      const duration = Math.round(nowMonotonic() - monotonicStart);
      const row = createTimingRow({
        command: completedCommand,
        startedAt,
        duration,
        exitCode: details.exitCode,
        commit,
      });
      try {
        await recordTimingBestEffort(row, { repoRoot });
      } catch (error) {
        timingWarning(warn, error);
      }
      resolve({
        command: completedCommand,
        startedAt,
        duration,
        exitCode: details.exitCode,
        commit,
        stdout: Buffer.concat(output.stdout).toString("utf8"),
        stderr: Buffer.concat(output.stderr).toString("utf8"),
        signal: details.signal,
        termination: details.termination,
      });
    };

    let child;
    try {
      startedAt = nowDate().toISOString();
      monotonicStart = nowMonotonic();
      child = spawn(command, args, spawnOptions);
    } catch (error) {
      void finish({ exitCode: 3, signal: null, termination: "spawn-error" });
      return;
    }

    if (capture) {
      child.stdout?.on("data", (chunk) => {
        const buffer = Buffer.from(chunk);
        output.stdout.push(buffer);
        if (mirrorOutput) process.stdout.write(buffer);
      });
      child.stderr?.on("data", (chunk) => {
        const buffer = Buffer.from(chunk);
        output.stderr.push(buffer);
        if (mirrorOutput) process.stderr.write(buffer);
      });
    }
    let cancellationSignal = null;
    let spawnedChildError = false;
    const requestCancellation = () => {
      if (cancellationSignal || finished) return;
      cancellationSignal = ["SIGINT", "SIGTERM"].includes(abortSignal?.reason)
        ? abortSignal.reason
        : "SIGTERM";
      try {
        child.kill(cancellationSignal);
      } catch {
        // Settlement remains tied to close; without confirmed termination the
        // observation intentionally stays pending.
      }
    };
    if (abortSignal) {
      abortSignal.addEventListener("abort", requestCancellation, { once: true });
      removeAbortListener = () => abortSignal.removeEventListener("abort", requestCancellation);
      if (abortSignal.aborted) requestCancellation();
    }
    child.once("close", (exitCode, signal) => {
      if (cancellationSignal) {
        void finish({ exitCode: 130, signal: cancellationSignal, termination: "signal" });
      } else if (spawnedChildError) {
        void finish({ exitCode: 3, signal: null, termination: "spawn-error" });
      } else {
        void finish(exitDetails(exitCode, signal));
      }
    });
    child.once("error", () => {
      if (child.pid == null) {
        void finish({ exitCode: 3, signal: null, termination: "spawn-error" });
      } else {
        spawnedChildError = true;
      }
    });
  });
}
