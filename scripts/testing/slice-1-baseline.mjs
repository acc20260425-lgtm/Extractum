import { mkdir, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { runObservedCommand } from "./run-observation.mjs";

export const BASELINE_COMMANDS = [
  ["frontend Vitest", { npmScript: "test", vitestReport: "frontend-vitest.json" }],
  ["Svelte check", { npmScript: "check" }],
  ["sidecar typecheck", { npmScript: "test:gemini-browser-sidecar:typecheck" }],
  ["sidecar unit", { npmScript: "test:gemini-browser-sidecar:unit", vitestReport: "sidecar-vitest.json" }],
  ["sidecar build", { npmScript: "test:gemini-browser-sidecar:build" }],
  ["adapter typecheck", { npmScript: "test:gemini-browser-adapter:typecheck" }],
  ["adapter unit", { npmScript: "test:gemini-browser-adapter:unit", vitestReport: "adapter-vitest.json" }],
  ["adapter Playwright", { npmScript: "test:gemini-browser-adapter:e2e", playwrightReport: "adapter-playwright.json" }],
  ["Cargo check", { command: "cargo", args: ["check", "--manifest-path", "src-tauri/Cargo.toml", "--workspace", "--all-targets"] }],
  ["Cargo test", { command: "cargo", args: ["test", "--manifest-path", "src-tauri/Cargo.toml", "--workspace", "--all-targets"] }],
  ["full verify", { npmScript: "verify" }],
];

export function resolveNpmScript(script, extraArgs = [], environment = process.env) {
  const isWindows = (environment.platform ?? process.platform) === "win32";
  const command = isWindows ? (environment.ComSpec ?? "C:\\Windows\\System32\\cmd.exe") : "npm";
  const args = isWindows
    ? ["/d", "/s", "/c", "npm.cmd", "run", script]
    : ["run", script];
  if (extraArgs.length) args.push("--", ...extraArgs);
  return { command, args };
}

function requiredInteger(object, key) {
  if (!Number.isInteger(object?.[key])) throw new Error(`malformed ${key}`);
  return object[key];
}

function parseVitestInventory(value) {
  if (!value || typeof value !== "object" || !Array.isArray(value.testResults)) {
    throw new Error("malformed Vitest report");
  }
  const files = value.testResults.map((testResult) => {
    if (typeof testResult?.name !== "string") throw new Error("malformed Vitest test file");
    return testResult.name;
  });
  return {
    numTotalTestSuites: requiredInteger(value, "numTotalTestSuites"),
    numPassedTestSuites: requiredInteger(value, "numPassedTestSuites"),
    numTotalTests: requiredInteger(value, "numTotalTests"),
    numPassedTests: requiredInteger(value, "numPassedTests"),
    files,
  };
}

function parsePlaywrightInventory(value) {
  if (!value || typeof value !== "object" || !Array.isArray(value.suites)) {
    throw new Error("malformed Playwright report");
  }
  const files = new Set();
  let suiteCount = 0;
  let specCount = 0;
  let testCount = 0;
  const visit = (suite) => {
    if (!suite || typeof suite !== "object" || (suite.suites !== undefined && !Array.isArray(suite.suites)) || !Array.isArray(suite.specs)) {
      throw new Error("malformed Playwright suite");
    }
    suiteCount += 1;
    if (typeof suite.file === "string" && suite.file.length) files.add(suite.file);
    for (const spec of suite.specs) {
      if (!spec || typeof spec !== "object" || !Array.isArray(spec.tests)) throw new Error("malformed Playwright spec");
      specCount += 1;
      testCount += spec.tests.length;
    }
    (suite.suites ?? []).forEach(visit);
  };
  value.suites.forEach(visit);
  return { suiteCount, specCount, testCount, files: [...files] };
}

async function readInventory(reportPath, kind) {
  let text;
  try {
    text = await readFile(reportPath, "utf8");
  } catch (error) {
    if (error && typeof error === "object" && error.code === "ENOENT") throw new Error(`missing ${kind} report`);
    throw new Error(`unable to read ${kind} report`);
  }
  let parsed;
  try {
    parsed = JSON.parse(text);
  } catch {
    throw new Error(`malformed ${kind} report`);
  }
  return kind === "Vitest" ? parseVitestInventory(parsed) : parsePlaywrightInventory(parsed);
}

function resolveOutputPath(repoRoot, outputPath) {
  const artifactRoot = path.resolve(repoRoot, "artifacts", "testing", "slice-1");
  const target = path.resolve(repoRoot, outputPath ?? "artifacts/testing/slice-1/baseline.json");
  if (target === artifactRoot || !target.startsWith(`${artifactRoot}${path.sep}`)) {
    throw new Error("Baseline output must be under artifacts/testing/slice-1");
  }
  return { artifactRoot, target };
}

function descriptorFor(definition, reportPath) {
  const extraArgs = definition.vitestReport
    ? ["--reporter=json", `--outputFile=${reportPath}`]
    : definition.playwrightReport ? ["--reporter=json"] : [];
  const command = definition.npmScript
    ? resolveNpmScript(definition.npmScript, extraArgs)
    : { command: definition.command, args: definition.args };
  return {
    ...command,
    env: definition.playwrightReport ? { PLAYWRIGHT_JSON_OUTPUT_FILE: reportPath } : undefined,
  };
}

export async function runBaseline({ repoRoot, runCommand = runObservedCommand, outputPath } = {}) {
  if (!repoRoot) throw new TypeError("repoRoot");
  const { artifactRoot, target } = resolveOutputPath(repoRoot, outputPath);
  const inventoryRoot = path.join(artifactRoot, "inventory");
  await mkdir(inventoryRoot, { recursive: true });

  let observedFailure = false;
  let infrastructureFailure = false;
  const observations = [];
  for (const [name, definition] of BASELINE_COMMANDS) {
    const reportFile = definition.vitestReport ?? definition.playwrightReport;
    const reportPath = reportFile ? path.join(inventoryRoot, reportFile) : undefined;
    const descriptor = descriptorFor(definition, reportPath);
    const attempts = [];
    for (let attempt = 0; attempt < 2; attempt += 1) {
      if (reportPath) await rm(reportPath, { force: true });
      const result = await runCommand({ ...descriptor, cwd: repoRoot });
      attempts.push(result);
      if (result.termination !== "spawn-error") break;
    }
    const finalAttempt = attempts.at(-1);
    const observation = {
      name,
      command: finalAttempt.command,
      startedAt: finalAttempt.startedAt,
      duration: finalAttempt.duration,
      exitCode: finalAttempt.exitCode,
      attempts,
    };
    if (finalAttempt.termination === "spawn-error") {
      infrastructureFailure = true;
      observation.infrastructureError = "spawn-error";
    } else {
      if (finalAttempt.exitCode !== 0) observedFailure = true;
      if (reportPath) {
        try {
          observation.inventory = await readInventory(reportPath, definition.vitestReport ? "Vitest" : "Playwright");
        } catch (error) {
          infrastructureFailure = true;
          observation.inventoryError = error instanceof Error ? error.message : String(error);
        }
      }
    }
    observations.push(observation);
  }
  const exitCode = infrastructureFailure ? 3 : 0;
  const baselineStatus = infrastructureFailure ? "infrastructure-error" : observedFailure ? "observed-failures" : "observed-success";
  const report = { baselineStatus, exitCode, observations };
  await mkdir(path.dirname(target), { recursive: true });
  await writeFile(target, `${JSON.stringify(report, null, 2)}\n`, "utf8");
  return report;
}

function cliOutputPath(argv) {
  if (argv.length !== 2 || argv[0] !== "--output") throw new Error("Usage: slice-1-baseline.mjs --output <path-under-artifacts/testing/slice-1>");
  return argv[1];
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const report = await runBaseline({ repoRoot: process.cwd(), outputPath: cliOutputPath(process.argv.slice(2)) });
    process.exitCode = report.exitCode;
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 3;
  }
}
