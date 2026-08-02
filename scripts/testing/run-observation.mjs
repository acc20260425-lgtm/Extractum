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
  stdio,
  capture = false,
  env,
  mirror = false,
  dependencies = {},
}) {
  const spawn = dependencies.spawn ?? defaultSpawn;
  const nowDate = dependencies.nowDate ?? (() => new Date());
  const nowMonotonic = dependencies.nowMonotonic ?? (() => performance.now());
  const readHeadCommit = dependencies.readHeadCommit ?? defaultReadHeadCommit;
  const createTimingRow = dependencies.createTimingRow ?? defaultCreateTimingRow;
  const recordTimingBestEffort = dependencies.recordTimingBestEffort ?? defaultRecordTimingBestEffort;
  const warn = dependencies.warn ?? console.warn;
  const completedCommand = formatCommand(command, args);
  const commit = readHeadCommit(cwd);
  const startedAt = nowDate().toISOString();
  const monotonicStart = nowMonotonic();
  const effectiveStdio = capture ? "pipe" : (stdio ?? "inherit");
  const mirrorOutput = capture && (mirror || stdio === "inherit");
  const output = { stdout: [], stderr: [] };

  return new Promise((resolve) => {
    let finished = false;
    const finish = async (details) => {
      if (finished) return;
      finished = true;
      const duration = Math.round(nowMonotonic() - monotonicStart);
      const row = createTimingRow({
        command: completedCommand,
        startedAt,
        duration,
        exitCode: details.exitCode,
        commit,
      });
      try {
        await recordTimingBestEffort(row);
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
      child = spawn(command, args, {
        cwd,
        env: { ...process.env, ...env },
        shell: false,
        stdio: effectiveStdio,
      });
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
    child.once("close", (exitCode, signal) => void finish(exitDetails(exitCode, signal)));
    child.once("error", () => void finish({ exitCode: 3, signal: null, termination: "spawn-error" }));
  });
}
