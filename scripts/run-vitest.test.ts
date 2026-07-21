import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import path from "node:path";

const repoRoot = path.resolve(import.meta.dirname, "..");
const runnerSource = readFileSync(path.join(repoRoot, "scripts/run-vitest.mjs"), "utf8");
const packageJson = JSON.parse(readFileSync(path.join(repoRoot, "package.json"), "utf8")) as {
  scripts: Record<string, string>;
};

describe("run-vitest wrapper", () => {
  it("keeps Playwright e2e specs out of Vitest discovery", () => {
    expect(runnerSource).toContain("DEFAULT_EXCLUDES");
    expect(runnerSource).toContain("research/gemini_browser_adapter/tests/**");
    expect(runnerSource).toContain(".worktrees/**");
  });

  it("routes watch mode through the same wrapper defaults", () => {
    expect(packageJson.scripts["test:watch"]).toBe("node scripts/run-vitest.mjs watch");
  });

  it("can be imported without starting Vitest", () => {
    expect(runnerSource).toContain('import { pathToFileURL } from "node:url"');
    expect(runnerSource).toMatch(
      /if \(process\.argv\[1\] && import\.meta\.url === pathToFileURL\(process\.argv\[1\]\)\.href\) \{\s*runVitest\(\);\s*\}/s,
    );
  });
});
