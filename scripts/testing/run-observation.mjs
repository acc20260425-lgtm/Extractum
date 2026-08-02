import { spawn as defaultSpawn } from "node:child_process";
import path from "node:path";
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

const DEFAULT_TERMINATION_TIMEOUT_MS = 15_000;
const FALLBACK_CLOSE_TIMEOUT_MS = 2_000;

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

function processSettlement(child) {
  return new Promise((resolve) => {
    let settled = false;
    let spawnError = null;
    const settle = (value) => {
      if (settled) return;
      settled = true;
      resolve(value);
    };
    child.once("error", (error) => {
      spawnError = error instanceof Error ? error.message : String(error);
      if (child.pid == null) settle({ closeObserved: false, exitCode: null, signal: null, spawnError });
    });
    child.once("close", (exitCode, signal) => {
      settle({ closeObserved: true, exitCode, signal, spawnError });
    });
  });
}

function processGroupAlive(pid, processKill) {
  try {
    processKill(-pid, 0);
    return true;
  } catch (error) {
    if (error?.code === "ESRCH") return false;
    if (error?.code === "EPERM") return true;
    throw error;
  }
}

async function waitForProcessGroupExit(pid, processKill, timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  while (processGroupAlive(pid, processKill) && Date.now() < deadline) {
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  return !processGroupAlive(pid, processKill);
}

async function defaultTerminateProcessTree({
  pid,
  cwd,
  env,
  signal,
  platform,
  systemRoot,
  spawnTerminationHelper,
  processKill,
  timeoutMs,
}) {
  if (!Number.isInteger(pid) || pid <= 0) {
    return { confirmed: false, failure: "observed command PID is unavailable" };
  }
  if (platform === "win32") {
    if (typeof systemRoot !== "string" || systemRoot.length === 0) {
      return { confirmed: false, failure: "SystemRoot is unavailable" };
    }
    const command = path.win32.join(systemRoot, "System32", "taskkill.exe");
    const args = ["/PID", String(pid), "/T", "/F"];
    let helper;
    try {
      helper = spawnTerminationHelper(command, args, {
        cwd,
        env,
        shell: false,
        windowsHide: true,
        stdio: "ignore",
      });
    } catch (error) {
      return { confirmed: false, command, args, failure: error instanceof Error ? error.message : String(error) };
    }
    const settled = processSettlement(helper);
    const result = await bounded(settled, timeoutMs, { timedOut: true });
    if (result.timedOut) {
      try {
        helper.kill("SIGKILL");
      } catch {
        // The unconfirmed result below is authoritative.
      }
      await bounded(settled, FALLBACK_CLOSE_TIMEOUT_MS, null);
      return { confirmed: false, command, args, failure: "taskkill timed out" };
    }
    return {
      confirmed: result.closeObserved === true && result.exitCode === 0,
      command,
      args,
      exitCode: result.exitCode,
      signal: result.signal,
      ...(result.spawnError ? { failure: result.spawnError } : {}),
    };
  }

  try {
    processKill(-pid, signal);
  } catch (error) {
    if (error?.code === "ESRCH") return { confirmed: true, strategy: "posix-process-group" };
    return { confirmed: false, strategy: "posix-process-group", failure: error instanceof Error ? error.message : String(error) };
  }
  if (await waitForProcessGroupExit(pid, processKill, timeoutMs)) {
    return { confirmed: true, strategy: "posix-process-group" };
  }
  try {
    processKill(-pid, "SIGKILL");
  } catch (error) {
    if (error?.code !== "ESRCH") {
      return { confirmed: false, strategy: "posix-process-group", failure: error instanceof Error ? error.message : String(error) };
    }
  }
  return {
    confirmed: await waitForProcessGroupExit(pid, processKill, FALLBACK_CLOSE_TIMEOUT_MS),
    strategy: "posix-process-group",
    escalated: true,
  };
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
  const platform = dependencies.platform ?? process.platform;
  const spawnTerminationHelper = dependencies.spawnTerminationHelper ?? defaultSpawn;
  const terminateProcessTree = dependencies.terminateProcessTree ?? defaultTerminateProcessTree;
  const processKill = dependencies.processKill ?? process.kill.bind(process);
  const terminationTimeoutMs = dependencies.terminationTimeoutMs ?? DEFAULT_TERMINATION_TIMEOUT_MS;
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
  const commandEnv = { ...process.env, ...env };
  const spawnOptions = {
    cwd,
    env: commandEnv,
    shell: false,
    stdio: effectiveStdio,
    ...(abortSignal && platform !== "win32" ? { detached: true } : {}),
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
        ...(details.cancellationConfirmed !== undefined
          ? { cancellationConfirmed: details.cancellationConfirmed, cancellation: details.cancellation }
          : {}),
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
    const observedClose = processSettlement(child);
    let cancellationSignal = null;
    let spawnedChildError = false;
    const requestCancellation = () => {
      if (cancellationSignal || finished) return;
      cancellationSignal = ["SIGINT", "SIGTERM"].includes(abortSignal?.reason)
        ? abortSignal.reason
        : "SIGTERM";
      void (async () => {
        let tree;
        try {
          tree = await terminateProcessTree({
            child,
            pid: child.pid,
            cwd,
            env: commandEnv,
            signal: cancellationSignal,
            platform,
            systemRoot: dependencies.systemRoot ?? commandEnv.SystemRoot,
            spawnTerminationHelper,
            processKill,
            timeoutMs: terminationTimeoutMs,
          });
        } catch (error) {
          tree = { confirmed: false, failure: error instanceof Error ? error.message : String(error) };
        }
        let commandSettlement = await bounded(
          observedClose,
          terminationTimeoutMs,
          { closeObserved: false, timedOut: true },
        );
        if (commandSettlement.closeObserved !== true) {
          try {
            child.kill("SIGKILL");
          } catch {
            // The cancellation remains unconfirmed.
          }
          const fallbackSettlement = await bounded(observedClose, FALLBACK_CLOSE_TIMEOUT_MS, null);
          if (fallbackSettlement) commandSettlement = fallbackSettlement;
        }
        const cancellationConfirmed = tree?.confirmed === true
          && commandSettlement?.closeObserved === true;
        await finish({
          exitCode: cancellationConfirmed ? 130 : 3,
          signal: cancellationSignal,
          termination: "signal",
          cancellationConfirmed,
          cancellation: { tree, command: commandSettlement },
        });
      })();
    };
    if (abortSignal) {
      abortSignal.addEventListener("abort", requestCancellation, { once: true });
      removeAbortListener = () => abortSignal.removeEventListener("abort", requestCancellation);
      if (abortSignal.aborted) requestCancellation();
    }
    child.once("close", (exitCode, signal) => {
      if (cancellationSignal) {
        return;
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
