import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

import { normalizeRelatedFileArgs } from "../../scripts/run-vitest.mjs";
import { VITEST_TEST_CONFIG } from "../../vite.config.js";

const repoRoot = path.resolve(import.meta.dirname, "..", "..");
const readSource = (relativePath: string) =>
  readFileSync(path.join(repoRoot, relativePath), "utf8").replace(/\r\n/g, "\n");
const packageJson = JSON.parse(readSource("package.json")) as {
  scripts: Record<string, string>;
};

describe("daily development loop configuration", () => {
  it("uses adaptive Vitest threads through one owned config object", () => {
    expect(VITEST_TEST_CONFIG.pool).toBe("threads");
    expect(Object.prototype.hasOwnProperty.call(VITEST_TEST_CONFIG, "maxWorkers")).toBe(false);
    expect(readSource("vite.config.js")).toMatch(/\btest:\s*VITEST_TEST_CONFIG\b/);
  });

  it("has no separate root Vitest config", () => {
    for (const extension of ["js", "ts", "mjs", "mts", "cjs", "cts"]) {
      expect(existsSync(path.join(repoRoot, `vitest.config.${extension}`))).toBe(false);
    }
  });

  it("owns the focused package scripts and canonical Rust target", () => {
    expect(packageJson.scripts["test:changed"]).toBe(
      "node scripts/run-vitest.mjs run --changed",
    );
    expect(packageJson.scripts["test:changed:last"]).toBe(
      "node scripts/run-vitest.mjs run --changed=HEAD~1",
    );
    expect(packageJson.scripts["test:related"]).toBe(
      "node scripts/run-vitest.mjs related --run",
    );
    expect(packageJson.scripts["test:rust"]).toBe(
      "cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets",
    );
    expect(packageJson.scripts["test:rust:prompt-pack-runs"]).toBe(
      "cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib prompt_pack_run",
    );
    expect(packageJson.scripts["test:rust"]).not.toContain("--target-dir");
    expect(packageJson.scripts["test:rust:prompt-pack-runs"]).not.toContain("--target-dir");
  });

  it("uses reduced dev debug information without a custom target", () => {
    const cargoToml = readSource("src-tauri/Cargo.toml");
    expect(cargoToml).toMatch(
      /\[profile\.dev\]\s*\ndebug = "line-tables-only"/,
    );
    expect(cargoToml).toMatch(
      /\[profile\.dev\.package\."\*"\]\s*\ndebug = false/,
    );
  });

  it("keeps stable daily-loop documentation anchors", () => {
    expect(readSource("AGENTS.md")).toContain("<!-- daily-development-loop -->");
    expect(readSource("docs/project.md")).toContain("<!-- daily-development-loop -->");
  });
});

describe("related-test path normalization", () => {
  const windowsPath = "src\\lib\\api\\llm.ts";
  const portablePath = "src/lib/api/llm.ts";

  it("normalizes an existing related operand", () => {
    expect(normalizeRelatedFileArgs(["related", windowsPath], repoRoot)).toEqual([
      "related",
      portablePath,
    ]);
  });

  it("leaves options and non-file patterns unchanged", () => {
    expect(normalizeRelatedFileArgs(["related", "-t", "foo\\bar"], repoRoot)).toEqual([
      "related",
      "-t",
      "foo\\bar",
    ]);
  });

  it("leaves a missing operand unchanged", () => {
    expect(
      normalizeRelatedFileArgs(["related", "src\\lib\\missing-file.ts"], repoRoot),
    ).toEqual(["related", "src\\lib\\missing-file.ts"]);
  });

  it("also normalizes an existing path-valued flag argument", () => {
    expect(
      normalizeRelatedFileArgs(["related", "--config", windowsPath], repoRoot),
    ).toEqual(["related", "--config", portablePath]);
  });

  it("does not normalize operands for other Vitest commands", () => {
    expect(normalizeRelatedFileArgs(["run", windowsPath], repoRoot)).toEqual([
      "run",
      windowsPath,
    ]);
  });
});
