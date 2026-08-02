import { spawn } from "node:child_process";
import { createRequire } from "node:module";
import path from "node:path";
import process from "node:process";
import { fileURLToPath, pathToFileURL } from "node:url";

const repoRoot = path.resolve(fileURLToPath(new URL("..", import.meta.url)));
const require = createRequire(import.meta.url);
const playwrightCli = require.resolve("@playwright/test/cli");
const runs = 20;
const playwrightArgs = Object.freeze([
  playwrightCli,
  "test",
  "-c",
  "research/gemini_browser_adapter/playwright.config.ts",
  "chromium-lifecycle",
]);
const cimCommand = "Get-CimInstance Win32_Process | Select-Object ProcessId,ParentProcessId,ExecutablePath,CommandLine | ConvertTo-Json -Compress";

export function parseStabilityArgs(args) {
  if (args.length !== 4
    || args[0] !== "--suite"
    || args[1] !== "chromium-lifecycle"
    || args[2] !== "--runs"
    || args[3] !== "20") return null;

  return { suite: "chromium-lifecycle", runs };
}

export function executeProcess(command, args, options = {}) {
  return new Promise((resolve, reject) => {
    let settled = false;
    let stdout = "";
    let stderr = "";
    const child = spawn(command, args, {
      cwd: repoRoot,
      ...options,
      shell: false,
      stdio: ["ignore", "pipe", "pipe"],
    });
    const finish = (action) => {
      if (settled) return;
      settled = true;
      action();
    };

    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk) => { stdout += chunk; });
    child.stderr.on("data", (chunk) => { stderr += chunk; });
    child.on("error", (error) => finish(() => reject(error)));
    child.on("close", (code, signal) => finish(() => resolve({
      exitCode: signal ? 1 : (code ?? 1),
      stdout,
      stderr,
    })));
  });
}

function parseCimJson(stdout) {
  let parsed;
  try {
    parsed = JSON.parse(stdout);
  } catch {
    throw new Error("Unable to parse CIM JSON");
  }

  const processes = Array.isArray(parsed) ? parsed : [parsed];
  if (processes.some((entry) => !entry
    || typeof entry !== "object"
    || !Number.isInteger(entry.ProcessId)
    || !("ParentProcessId" in entry)
    || !("ExecutablePath" in entry)
    || !("CommandLine" in entry))) {
    throw new Error("Invalid CIM JSON process record");
  }
  return processes;
}

export async function listWindowsProcesses({ executeProcess: execute = executeProcess } = {}) {
  const result = await execute(
    "powershell.exe",
    ["-NoProfile", "-NonInteractive", "-Command", cimCommand],
    { cwd: repoRoot, shell: false },
  );
  if (!result || !Number.isInteger(result.exitCode)) throw new Error("Invalid CIM command result");
  if (result.exitCode !== 0) {
    const detail = String(result.stderr ?? "").trim();
    throw new Error(`CIM command exited with code ${result.exitCode}${detail ? `: ${detail}` : ""}`);
  }
  return parseCimJson(result.stdout);
}

function errorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}

function exitCodeOf(result) {
  const exitCode = typeof result === "number" ? result : result?.exitCode;
  if (!Number.isInteger(exitCode)) throw new Error("Invalid child process result");
  return exitCode;
}

function playwrightUserDataDir(commandLine) {
  const match = commandLine.match(
    /(?:^|\s)(?:"--user-data-dir=([^"]+)"|--user-data-dir="([^"]+)"|--user-data-dir=([^"\s]+))(?=\s|$)/i,
  );
  return match ? (match[1] ?? match[2] ?? match[3]) : null;
}

function isPlaywrightChromiumLeak(processRecord) {
  const commandLine = processRecord.CommandLine;
  if (typeof commandLine !== "string") return false;
  const hasHeadlessMarker = /(?:^|\s)"?--headless(?:=(?:new|old|chrome))?"?(?=\s|$)/i.test(commandLine);
  const userDataDir = playwrightUserDataDir(commandLine);
  const hasPlaywrightTempProfile = typeof userDataDir === "string"
    && /^playwright_chromiumdev_profile-.+/i.test(path.win32.basename(userDataDir));
  return hasHeadlessMarker && hasPlaywrightTempProfile;
}

function boundedProcessOutput(result) {
  const streams = [
    ["stderr", String(result?.stderr ?? "").trim()],
    ["stdout", String(result?.stdout ?? "").trim()],
  ].filter(([, output]) => output.length > 0);
  if (streams.length === 0) return "";

  const limit = streams.length === 1 ? 1_900 : 900;
  return streams.map(([name, output]) => {
    const excerpt = output.length > limit ? `${output.slice(0, limit - 3)}...` : output;
    return `${name}:\n${excerpt}`;
  }).join("\n");
}

async function finalProcessSnapshot(listProcesses) {
  try {
    return { processes: await listProcesses() };
  } catch (error) {
    return { error: { exitCode: 3, diagnostic: `Process enumeration failed: ${errorMessage(error)}` } };
  }
}

export async function runChromiumLifecycleAudit({ runCommand, listProcesses }) {
  let before;
  try {
    before = await listProcesses();
  } catch (error) {
    return { exitCode: 3, diagnostic: `Process enumeration failed: ${errorMessage(error)}` };
  }

  let failedRun;
  for (let run = 1; run <= runs; run += 1) {
    let result;
    try {
      result = await runCommand(process.execPath, [...playwrightArgs], { cwd: repoRoot, shell: false });
    } catch (error) {
      console.log(`chromium-lifecycle run ${run}/20: fail`);
      return { exitCode: 3, diagnostic: `Playwright spawn failed: ${errorMessage(error)}` };
    }

    let exitCode;
    try {
      exitCode = exitCodeOf(result);
    } catch (error) {
      console.log(`chromium-lifecycle run ${run}/20: fail`);
      return { exitCode: 3, diagnostic: `Playwright spawn failed: ${errorMessage(error)}` };
    }

    const passed = exitCode === 0;
    console.log(`chromium-lifecycle run ${run}/20: ${passed ? "pass" : "fail"}`);
    if (!passed) {
      failedRun = { run, exitCode, stdout: result?.stdout, stderr: result?.stderr };
      break;
    }
  }

  const afterSnapshot = await finalProcessSnapshot(listProcesses);
  if (afterSnapshot.error) return afterSnapshot.error;

  const beforePids = new Set(before.map((processRecord) => processRecord.ProcessId));
  const leakedPids = afterSnapshot.processes
    .filter((processRecord) => !beforePids.has(processRecord.ProcessId) && isPlaywrightChromiumLeak(processRecord))
    .map((processRecord) => processRecord.ProcessId)
    .sort((left, right) => left - right);

  if (failedRun) {
    const leakDetail = leakedPids.length > 0 ? `; leaked Playwright Chromium process IDs: ${leakedPids.join(", ")}` : "";
    const outputExcerpt = boundedProcessOutput(failedRun);
    const outputDetail = outputExcerpt ? `\nPlaywright output excerpt:\n${outputExcerpt}` : "";
    return {
      exitCode: 1,
      diagnostic: `Playwright run ${failedRun.run}/20 exited with code ${failedRun.exitCode}${leakDetail}${outputDetail}`,
    };
  }
  if (leakedPids.length > 0) {
    return { exitCode: 1, diagnostic: `Leaked Playwright Chromium process IDs: ${leakedPids.join(", ")}` };
  }
  return { exitCode: 0 };
}

export async function runStabilityCli(args, {
  runCommand = executeProcess,
  listProcesses = () => listWindowsProcesses(),
} = {}) {
  if (!parseStabilityArgs(args)) {
    console.error("Usage: npm.cmd run verify:stability -- --suite chromium-lifecycle --runs 20");
    return 2;
  }

  const result = await runChromiumLifecycleAudit({ runCommand, listProcesses });
  if (result.diagnostic) console.error(result.diagnostic);
  return result.exitCode;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  process.exitCode = await runStabilityCli(process.argv.slice(2));
}
