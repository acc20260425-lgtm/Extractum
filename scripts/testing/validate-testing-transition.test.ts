import { describe, expect, it, vi } from "vitest";
import path from "node:path";

import {
  runTransitionValidation,
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
