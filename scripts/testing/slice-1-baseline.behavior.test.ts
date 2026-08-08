import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it, vi } from "vitest";

import { BASELINE_COMMANDS, runBaseline } from "./slice-1-baseline.mjs";

const roots: string[] = [];

afterEach(async () => {
  await Promise.all(roots.splice(0).map((root) => rm(root, { recursive: true, force: true })));
});

async function writeReporter(command: { args: string[]; env?: NodeJS.ProcessEnv }) {
  const vitestPath = command.args.find((argument) => argument.startsWith("--outputFile="))?.slice("--outputFile=".length);
  const playwrightPath = command.env?.PLAYWRIGHT_JSON_OUTPUT_FILE;
  if (vitestPath) {
    await writeFile(vitestPath, JSON.stringify({
      numTotalTestSuites: 1,
      numPassedTestSuites: 1,
      numTotalTests: 1,
      numPassedTests: 1,
      testResults: [{ name: "src/example.test.ts" }],
    }));
  }
  if (playwrightPath) {
    await writeFile(playwrightPath, JSON.stringify({
      suites: [{ file: "e2e/example.spec.ts", specs: [{ tests: [{}] }] }],
    }));
  }
}

describe("slice one baseline", () => {
  it("continues sequentially after observed test failures and writes reporter inventories", async () => {
    const root = await mkdtemp(path.join(tmpdir(), "extractum-baseline-contract-"));
    roots.push(root);
    const calls: string[] = [];
    const runCommand = vi.fn(async (command) => {
      calls.push(command.command);
      await writeReporter(command);
      return {
        command: command.command,
        startedAt: "2026-08-08T00:00:00.000Z",
        duration: 1,
        exitCode: calls.length === 2 ? 1 : 0,
        termination: "exit",
      };
    });

    const report = await runBaseline({ repoRoot: root, runCommand });

    expect(runCommand).toHaveBeenCalledTimes(BASELINE_COMMANDS.length);
    expect(report).toMatchObject({ exitCode: 0, baselineStatus: "observed-failures" });
    expect(report.observations[0].inventory.files).toEqual(["src/example.test.ts"]);
    expect(report.observations[7].inventory.files).toEqual(["e2e/example.spec.ts"]);
    const written = JSON.parse(await readFile(path.join(root, "artifacts/testing/slice-1/baseline.json"), "utf8"));
    expect(written.observations).toHaveLength(BASELINE_COMMANDS.length);
  });
});
