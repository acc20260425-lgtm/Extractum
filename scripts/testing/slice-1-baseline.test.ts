import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  BASELINE_COMMANDS,
  resolveNpmScript,
  runBaseline,
} from "./slice-1-baseline.mjs";

const roots: string[] = [];
afterEach(async () => {
  await Promise.all(roots.splice(0).map((root) => rm(root, { recursive: true, force: true })));
});

async function repoRoot() {
  const root = await mkdtemp(path.join(tmpdir(), "extractum-baseline-"));
  roots.push(root);
  return root;
}

function result(exitCode = 0, termination = "exit") {
  return {
    command: "child command",
    startedAt: "2026-08-02T10:11:12.123Z",
    duration: 17,
    exitCode,
    termination,
  };
}

async function writeReporterOutput(command: { args: string[]; env?: NodeJS.ProcessEnv }, playwright = false) {
  const pathArgument = command.args.find((argument) => argument.startsWith("--outputFile="));
  const reportPath = playwright ? command.env?.PLAYWRIGHT_JSON_OUTPUT_FILE : pathArgument?.slice("--outputFile=".length);
  if (!reportPath) return;
  const report = playwright
    ? { suites: [{ file: "research/example.spec.ts", suites: [], specs: [{ tests: [{ results: [{ status: "passed" }] }] }] }] }
    : { numTotalTestSuites: 1, numPassedTestSuites: 1, numTotalTests: 1, numPassedTests: 1, testResults: [{ name: "src/example.test.ts" }] };
  await writeFile(reportPath, JSON.stringify(report));
}

describe("slice one baseline", () => {
  it("uses the approved current command portfolio in order", () => {
    expect(BASELINE_COMMANDS).toEqual([
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
    ]);
  });

  it("continues sequentially after observed test failures and writes reporter inventories", async () => {
    const root = await repoRoot();
    const runCommand = vi.fn(async (command) => {
      await writeReporterOutput(command, command.env?.PLAYWRIGHT_JSON_OUTPUT_FILE !== undefined);
      return result(runCommand.mock.calls.length === 2 ? 1 : 0);
    });

    const report = await runBaseline({ repoRoot: root, runCommand });

    expect(runCommand).toHaveBeenCalledTimes(BASELINE_COMMANDS.length);
    expect(report).toMatchObject({ exitCode: 0, baselineStatus: "observed-failures" });
    expect(report.observations).toHaveLength(BASELINE_COMMANDS.length);
    expect(report.observations[0].inventory).toMatchObject({ numTotalTests: 1, files: ["src/example.test.ts"] });
    expect(report.observations[7].inventory).toMatchObject({ suiteCount: 1, specCount: 1, testCount: 1, files: ["research/example.spec.ts"] });
    const written = JSON.parse(await readFile(path.join(root, "artifacts/testing/slice-1/baseline.json"), "utf8"));
    expect(written.baselineStatus).toBe("observed-failures");
  });

  it("does not invoke command N plus one until command N resolves", async () => {
    const root = await repoRoot();
    const pending: Array<{
      command: { args: string[]; env?: NodeJS.ProcessEnv };
      resolve: (value: ReturnType<typeof result>) => void;
    }> = [];
    const runCommand = vi.fn((command) => new Promise<ReturnType<typeof result>>((resolve) => {
      pending.push({ command, resolve });
    }));
    const baseline = runBaseline({ repoRoot: root, runCommand });

    for (let index = 0; index < BASELINE_COMMANDS.length; index += 1) {
      await vi.waitFor(() => expect(runCommand).toHaveBeenCalledTimes(index + 1));
      await writeReporterOutput(pending[index].command, pending[index].command.env?.PLAYWRIGHT_JSON_OUTPUT_FILE !== undefined);
      pending[index].resolve(result());
    }

    await expect(baseline).resolves.toMatchObject({ exitCode: 0, baselineStatus: "observed-success" });
  });

  it("uses ComSpec and exactly one npm argument separator on Windows", () => {
    expect(resolveNpmScript("test", ["--reporter=json"], { platform: "win32", ComSpec: "cmd-test.exe" })).toEqual({
      command: "cmd-test.exe",
      args: ["/d", "/s", "/c", "npm.cmd", "run", "test", "--", "--reporter=json"],
    });
  });

  it("accepts Playwright's omitted empty nested suite list", async () => {
    const root = await repoRoot();
    const runCommand = vi.fn(async (command) => {
      const reportPath = command.env?.PLAYWRIGHT_JSON_OUTPUT_FILE;
      if (reportPath) {
        await writeFile(reportPath, JSON.stringify({
          suites: [{ file: "research/actual.spec.ts", specs: [{ tests: [] }] }],
        }));
      } else {
        await writeReporterOutput(command);
      }
      return result();
    });

    const report = await runBaseline({ repoRoot: root, runCommand });

    expect(report).toMatchObject({ exitCode: 0, baselineStatus: "observed-success" });
    expect(report.observations[7].inventory).toMatchObject({ suiteCount: 1, files: ["research/actual.spec.ts"] });
  });

  it("rejects a report path outside the dedicated artifact directory", async () => {
    const root = await repoRoot();
    await expect(runBaseline({ repoRoot: root, outputPath: path.join(root, "outside.json"), runCommand: vi.fn() }))
      .rejects.toThrow("artifacts/testing/slice-1");
  });

  it("retries only a spawn error, retains both attempts, and marks repeated errors as infrastructure failures", async () => {
    const root = await repoRoot();
    let calls = 0;
    const runCommand = vi.fn(async (command) => {
      calls += 1;
      if (calls <= 2) return result(3, "spawn-error");
      await writeReporterOutput(command, command.env?.PLAYWRIGHT_JSON_OUTPUT_FILE !== undefined);
      return result();
    });

    const report = await runBaseline({ repoRoot: root, runCommand });

    expect(runCommand).toHaveBeenCalledTimes(BASELINE_COMMANDS.length + 1);
    expect(report).toMatchObject({ exitCode: 3, baselineStatus: "infrastructure-error" });
    expect(report.observations[0].attempts).toHaveLength(2);
    expect(report.observations[0].attempts.map((attempt: { termination: string }) => attempt.termination)).toEqual(["spawn-error", "spawn-error"]);
  });

  it.each([
    ["Vitest", 0],
    ["Playwright", 7],
  ])("marks missing %s inventory as infrastructure failure without a retry", async (_name, missingIndex) => {
    const root = await repoRoot();
    const runCommand = vi.fn(async (command) => {
      if (runCommand.mock.calls.length - 1 !== missingIndex) {
        await writeReporterOutput(command, command.env?.PLAYWRIGHT_JSON_OUTPUT_FILE !== undefined);
      }
      return result();
    });

    const report = await runBaseline({ repoRoot: root, runCommand });

    expect(runCommand).toHaveBeenCalledTimes(BASELINE_COMMANDS.length);
    expect(report).toMatchObject({ exitCode: 3, baselineStatus: "infrastructure-error" });
    expect(report.observations[missingIndex].inventoryError).toMatch(/missing/i);
  });

  it.each([
    ["Vitest", 0],
    ["Playwright", 7],
  ])("marks malformed %s inventory as infrastructure failure without a retry", async (_name, malformedIndex) => {
    const root = await repoRoot();
    const runCommand = vi.fn(async (command) => {
      if (runCommand.mock.calls.length - 1 === malformedIndex) {
        const reportPath = command.env?.PLAYWRIGHT_JSON_OUTPUT_FILE
          ?? command.args.find((argument) => argument.startsWith("--outputFile="))?.slice("--outputFile=".length);
        if (reportPath) await writeFile(reportPath, "not json");
      } else {
        await writeReporterOutput(command, command.env?.PLAYWRIGHT_JSON_OUTPUT_FILE !== undefined);
      }
      return result();
    });

    const report = await runBaseline({ repoRoot: root, runCommand });

    expect(runCommand).toHaveBeenCalledTimes(BASELINE_COMMANDS.length);
    expect(report).toMatchObject({ exitCode: 3, baselineStatus: "infrastructure-error" });
    expect(report.observations[malformedIndex].inventoryError).toMatch(/malformed/i);
  });
});
