import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

import { normalizeRelatedFileArgs } from "../../scripts/run-vitest.mjs";
import { VITEST_PROJECT_DEFINITIONS } from "../../vitest.config";

const repoRoot = path.resolve(import.meta.dirname, "..", "..");
const readSource = (relativePath: string) =>
  readFileSync(path.join(repoRoot, relativePath), "utf8").replace(/\r\n/g, "\n");
const packageJson = JSON.parse(readSource("package.json")) as {
  scripts: Record<string, string>;
};
const analysisOrientedPackageScripts = Object.entries(packageJson.scripts).filter(
  ([name, command]) =>
    name.toLowerCase().includes("analysis") ||
    command.toLowerCase().includes("analysis"),
);
const promptPackCrateExtracted = existsSync(
  path.join(
    repoRoot,
    "src-tauri/crates/extractum-prompt-packs/Cargo.toml",
  ),
);

describe("daily development loop configuration", () => {
  it("uses adaptive Vitest threads through dedicated project definitions", () => {
    const threadedProjects = VITEST_PROJECT_DEFINITIONS.filter(({ pool }) => pool === "threads");
    expect(threadedProjects.map(({ name }) => name)).toEqual(["unit-node", "component", "architecture", "legacy-contract"]);
    expect(threadedProjects.every((project) => !Object.prototype.hasOwnProperty.call(project, "maxWorkers"))).toBe(true);
  });

  it("gives the filesystem-heavy coordinator suite an explicit timeout", () => {
    expect(
      readSource("scripts/process-shell-diagnostic/coordinator.test.ts"),
    ).toContain(
      'describe("process shell diagnostic coordinator", { timeout: 30_000 }, () => {',
    );
  });

  it("keeps project ownership in the dedicated root Vitest config", () => {
    expect(existsSync(path.join(repoRoot, "vitest.config.ts"))).toBe(true);
    expect(readSource("vite.config.js")).not.toMatch(/\btest:\s/);
    expect(readSource("vitest.config.ts")).toContain("projects");
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
      promptPackCrateExtracted
        ? "cargo test --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --lib prompt_pack_run"
        : "cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib prompt_pack_run",
    );
    expect(packageJson.scripts["test:rust"]).not.toContain("--target-dir");
    expect(packageJson.scripts["test:rust:prompt-pack-runs"]).not.toContain("--target-dir");
    expect(analysisOrientedPackageScripts).toEqual([
      ["smoke:analysis", "node scripts/analysis-smoke.mjs"],
    ]);
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
