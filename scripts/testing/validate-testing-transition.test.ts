import { describe, expect, it, vi } from "vitest";
import path from "node:path";

import {
  collectPlaywrightFiles,
  collectVitestFiles,
  discoverFilesystemCandidates,
  normalizeRepoPath,
  runTransitionValidation,
  validateCensusSchema,
  validateRunnerCensus,
} from "./testing-transition.mjs";

const root = "C:/repo";
const census = {
  schemaVersion: 1,
  vitestOwners: [{ id: "vitest:root", args: [], ownerScript: "test" }],
  playwrightOwners: [{ id: "playwright:adapter", config: "research/adapter/playwright.config.ts", ownerScript: "test:adapter:e2e" }],
  nonstandardTests: [],
  fixtureExceptions: [],
};

function check(overrides: Record<string, unknown> = {}) {
  return {
    census,
    filesystemFiles: ["src/a.test.ts", "research/adapter/tests/e2e.spec.ts"],
    vitestFiles: { "vitest:root": ["src/a.test.ts"] },
    playwrightFiles: { "playwright:adapter": ["research/adapter/tests/e2e.spec.ts"] },
    ...overrides,
  };
}

describe("runner census validation", () => {
  it("accepts exact bidirectional ownership", () => {
    expect(validateRunnerCensus(check())).toEqual([]);
  });

  it("reports sorted actionable ownership, candidate, empty-owner, and runner failures", () => {
    expect(validateRunnerCensus(check({
      filesystemFiles: ["src/missing.test.ts"],
      vitestFiles: { "vitest:root": ["src/extra.test.ts"] },
      playwrightFiles: { "playwright:adapter": [] },
      runnerIssues: ["runner failed: vitest:root"],
    }))).toEqual([
      "collected non-candidate: vitest:root -> src/extra.test.ts",
      "empty owner: playwright:adapter",
      "runner failed: vitest:root",
      "unowned filesystem candidate: src/missing.test.ts",
    ]);
  });

  it("rejects duplicate ownership even if a fixture exception names the path", () => {
    const issues = validateRunnerCensus(check({
      vitestFiles: { "vitest:root": ["src/a.test.ts"] },
      playwrightFiles: { "playwright:adapter": ["src/a.test.ts", "research/adapter/tests/e2e.spec.ts"] },
      census: { ...census, fixtureExceptions: [{ path: "src/a.test.ts", reason: "fixture", owner: "fixture" }] },
    }));

    expect(issues).toContain("duplicate ownership: src/a.test.ts -> playwright:adapter, vitest:root");
    expect(issues).toContain("stale fixture exception: src/a.test.ts");
  });

  it("requires exact, necessary exceptions with a path, reason, and owner", () => {
    const issues = validateRunnerCensus(check({
      filesystemFiles: ["src/fixture.test.ts"],
      vitestFiles: { "vitest:root": ["scripts/nonstandard.test.ts"] },
      playwrightFiles: { "playwright:adapter": [] },
      census: {
        ...census,
        nonstandardTests: [{ path: "scripts/nonstandard.test.ts", reason: "custom runner", owner: "vitest:root" }],
        fixtureExceptions: [{ path: "src/fixture.test.ts", reason: "fixture", owner: "fixture" }],
      },
    }));
    expect(issues).toContain("empty owner: playwright:adapter");
    expect(issues).not.toContain("unowned filesystem candidate: src/fixture.test.ts");
    expect(issues).not.toContain("collected non-candidate: vitest:root -> scripts/nonstandard.test.ts");
  });

  it("rejects unsafe, duplicate, stale, and broad exception paths", () => {
    const issues = validateRunnerCensus(check({
      census: {
        ...census,
        fixtureExceptions: [
          { path: "src/*.test.ts", reason: "broad", owner: "fixture" },
          { path: "src/*.test.ts", reason: "duplicate", owner: "fixture" },
          { path: "src/stale.test.ts", reason: "stale", owner: "fixture" },
        ],
      },
    }));
    expect(issues).toEqual(expect.arrayContaining([
      "duplicate fixture exception: src/*.test.ts",
      "invalid fixture exception path: src/*.test.ts",
      "stale fixture exception: src/stale.test.ts",
    ]));
  });

  it.each([null, "not-an-entry", { path: "src/a.test.ts" }])("returns schema issues for malformed nonstandard exceptions: %j", (entry) => {
    expect(() => validateRunnerCensus(check({
      census: { ...census, nonstandardTests: [entry] },
    }))).not.toThrow();
    expect(validateRunnerCensus(check({ census: { ...census, nonstandardTests: [entry] } }))).toEqual(
      expect.arrayContaining([expect.stringMatching(/^invalid nonstandard exception/)]),
    );
  });

  it("reports unknown census and exception fields", () => {
    expect(validateCensusSchema({ ...census, unexpected: true })).toEqual(["unknown runner census field: unexpected"]);
    expect(validateRunnerCensus(check({
      census: { ...census, fixtureExceptions: [{ path: "src/a.test.ts", reason: "fixture", owner: "fixture", extra: true }] },
    }))).toEqual(expect.arrayContaining(["unknown fixture exception field: extra"]));
  });
});

describe("filesystem and runner collection", () => {
  const stats = ({ file = true, symlink = false } = {}) => ({ isFile: () => file, isSymbolicLink: () => symlink });

  it("discovers only existing candidate files from the required nul-delimited Git command", async () => {
    const runGit = vi.fn(async () => ({ exitCode: 0, stdout: "src/a.test.ts\0src/b.spec.mtsx\0src/readme.ts\0deleted.test.ts\0" }));
    const found = await discoverFilesystemCandidates(root, runGit, (candidate) => {
      if (candidate.endsWith("deleted.test.ts")) {
        const error = new Error("missing") as NodeJS.ErrnoException;
        error.code = "ENOENT";
        throw error;
      }
      return stats();
    });
    expect(runGit).toHaveBeenCalledWith(["ls-files", "--cached", "--others", "--exclude-standard", "-z"], root);
    expect(found).toEqual({ files: ["src/a.test.ts", "src/b.spec.mtsx"], issues: [] });
  });

  it("reports Git failure and supported symlink candidates instead of silently omitting them", async () => {
    await expect(discoverFilesystemCandidates(root, async () => ({ exitCode: 2, stdout: "" }))).resolves.toEqual({ files: [], issues: ["git ls-files failed"] });
    await expect(discoverFilesystemCandidates(root, async () => ({ exitCode: 0, stdout: "src/link.test.ts\0" }), () => stats({ file: false, symlink: true })))
      .resolves.toEqual({ files: [], issues: ["unsupported candidate symlink: src/link.test.ts"] });
  });

  it("normalizes Windows and repository-relative paths while rejecting escapes", () => {
    expect(normalizeRepoPath("C:\\repo", "C:\\repo\\src\\a.test.ts")).toEqual({ path: "src/a.test.ts" });
    expect(normalizeRepoPath(root, "/outside/a.test.ts")).toEqual({ issue: "repository escape: /outside/a.test.ts" });
    expect(normalizeRepoPath(root, "../outside/a.test.ts")).toEqual({ issue: "repository escape: ../outside/a.test.ts" });
  });

  it("collects Vitest paths and reports non-zero and spawn errors", async () => {
    const owner = census.vitestOwners[0];
    const good = await collectVitestFiles(owner, { repoRoot: root, runCommand: vi.fn(async () => ({ exitCode: 0, stdout: "src/a.test.ts\r\nsrc/b.spec.ts\n" })) });
    expect(good.files).toEqual(["src/a.test.ts", "src/b.spec.ts"]);
    await expect(collectVitestFiles(owner, { repoRoot: root, runCommand: async () => ({ exitCode: 1, stdout: "", error: new Error("spawn") }) }))
      .resolves.toMatchObject({ issues: ["runner spawn error: vitest:root: spawn", "runner failed: vitest:root"] });
  });

  it("parses Playwright suites and rejects malformed JSON and any errors value", async () => {
    const owner = { ...census.playwrightOwners[0], config: "research/gemini_browser_adapter/playwright.config.ts" };
    const invoke = async (stdout: string) => collectPlaywrightFiles(owner, {
      repoRoot: process.cwd(),
      resolveCli: () => "playwright-cli.mjs",
      runCommand: async () => ({ exitCode: 0, stdout }),
    });
    await expect(invoke(JSON.stringify({ suites: [{ suites: [{ file: "nested.spec.ts", specs: [] }], specs: [{ file: "root.spec.ts" }] }] })))
      .resolves.toMatchObject({ files: ["research/gemini_browser_adapter/tests/nested.spec.ts", "research/gemini_browser_adapter/tests/root.spec.ts"], issues: [] });
    await expect(invoke(JSON.stringify({ suites: [], errors: [] }))).resolves.toMatchObject({ files: [], issues: [] });
    await expect(invoke("not-json")).resolves.toMatchObject({ issues: ["playwright:adapter: malformed Playwright JSON"] });
    await expect(invoke(JSON.stringify({ suites: [], errors: { message: "bad" } }))).resolves.toMatchObject({ issues: ["playwright:adapter: Playwright errors are not empty"] });
    await expect(invoke(JSON.stringify({ suites: [], errors: ["bad"] }))).resolves.toMatchObject({ issues: ["playwright:adapter: Playwright errors are not empty"] });
  });
});

describe("transition carrier", () => {
  it("runs injected checks in order, prints issues, and returns non-zero", async () => {
    const calls: string[] = [];
    const stdout = { write: vi.fn() };
    const stderr = { write: vi.fn() };
    const result = await runTransitionValidation({
      repoRoot: root,
      checks: [
        async () => { calls.push("one"); return []; },
        async () => { calls.push("two"); return { issues: ["second issue"], summary: "Runner census: injected" }; },
      ],
      stdout,
      stderr,
    });

    expect(calls).toEqual(["one", "two"]);
    expect(result.exitCode).toBe(1);
    expect(stderr.write).toHaveBeenCalledWith("second issue\n");
    expect(stdout.write).toHaveBeenCalledWith(expect.stringContaining("Runner census:"));
  });
});
