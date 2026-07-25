import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

const repoRoot = path.resolve(import.meta.dirname, "..", "..");
const promptPackCrateExtracted = existsSync(
  path.join(repoRoot, "src-tauri/crates/extractum-prompt-packs/Cargo.toml"),
);
const analysisCrateExtracted = existsSync(
  path.join(repoRoot, "src-tauri/crates/extractum-analysis/Cargo.toml"),
);
const readSource = (relativePath: string) =>
  readFileSync(path.join(repoRoot, relativePath), "utf8").replace(/\r\n/g, "\n");
const readOptionalSource = (relativePath: string) =>
  existsSync(path.join(repoRoot, relativePath)) ? readSource(relativePath) : "";
const tomlSection = (source: string, heading: string) => {
  const marker = `[${heading}]`;
  const start = source.indexOf(marker);
  if (start < 0) return "";
  const bodyStart = start + marker.length;
  const next = source.slice(bodyStart).search(/^\[\[?[^\n]+\]?\]$/m);
  return source.slice(bodyStart, next < 0 ? undefined : bodyStart + next).trim();
};
const lockPackage = (source: string, name: string) =>
  source
    .split(/(?=^\[\[package\]\]$)/m)
    .find((block) => block.includes(`\nname = "${name}"\n`)) ?? "";
const analysisLockDependencyPattern =
  /^ "extractum-analysis(?: [^"]+)?",?$/m;

const rootCargo = readSource("src-tauri/Cargo.toml");
const cargoLock = readSource("src-tauri/Cargo.lock");
const rootLib = readSource("src-tauri/src/lib.rs");
const coreCargo = readOptionalSource("src-tauri/crates/extractum-core/Cargo.toml");
const coreLib = readOptionalSource("src-tauri/crates/extractum-core/src/lib.rs");
const packageJson = JSON.parse(readSource("package.json")) as {
  scripts: Record<string, string>;
};
const verifySource = readSource("scripts/verify.mjs");
const agentGuidance = readSource("AGENTS.md");
const projectGuidance = readSource("docs/project.md");

describe("Rust workspace core contract", () => {
  it("owns the application and core from the src-tauri workspace root", () => {
    expect(rootCargo).toContain("[workspace]");
    const members = rootCargo
      .match(/^members\s*=\s*\[([^\]]+)\]$/m)?.[1]
      .split(",")
      .map((member) => member.trim().replace(/^"|"$/g, ""));
    const expectedMembers = [
      ".",
      "crates/extractum-core",
      "crates/extractum-gemini-browser",
      "crates/extractum-llm",
      ...(promptPackCrateExtracted ? ["crates/extractum-prompt-packs"] : []),
      ...(analysisCrateExtracted ? ["crates/extractum-analysis"] : []),
    ];
    expect(members).toEqual(expectedMembers);
    expect(
      tomlSection(rootCargo, "dependencies").match(
        /^extractum-analysis = \{ path = "crates\/extractum-analysis" \}$/gm,
      ) ?? [],
    ).toHaveLength(analysisCrateExtracted ? 1 : 0);
    expect(
      rootCargo
        .split("\n")
        .filter((line) => line.includes("extractum-analysis")),
    ).toHaveLength(analysisCrateExtracted ? 2 : 0);
    expect(rootCargo).toMatch(/resolver\s*=\s*"2"/);
    expect(rootCargo).toContain("[workspace.dependencies]");
    expect(rootCargo).toContain("[profile.dev]");
    expect(rootCargo).toContain('debug = "line-tables-only"');
    expect(rootCargo).toContain('[profile.dev.package."*"]');
    expect(rootCargo).toMatch(/\[profile\.dev\.package\."\*"\][\s\S]*debug\s*=\s*false/);
  });

  it("defines the minimal core package through workspace dependencies", () => {
    expect(coreCargo).not.toBe("");
    expect(coreCargo).toMatch(/name\s*=\s*"extractum-core"/);
    for (const dependency of ["serde", "serde_json", "time", "zstd"]) {
      expect(coreCargo).toMatch(new RegExp(`${dependency}\\.workspace\\s*=\\s*true`));
    }
    expect(coreCargo).not.toMatch(/\[profile\./);
    expect(coreCargo).not.toContain("extractum-prompt-packs");
    expect(coreCargo).not.toContain("extractum-analysis");
    expect(' "extractum-analysis 0.2.0",').toMatch(
      analysisLockDependencyPattern,
    );
    expect(lockPackage(cargoLock, "extractum-core")).not.toMatch(
      analysisLockDependencyPattern,
    );
  });

  it("keeps a curated core and explicit private application wrappers", () => {
    expect(coreLib).toBe(
      [
        "pub mod compression;",
        "pub mod error;",
        "pub mod media_metadata;",
        "pub mod time;",
        "",
      ].join("\n"),
    );
    expect(coreLib).not.toMatch(/pub\s+use\s+[^;]*\*/);

    for (const moduleName of ["compression", "error", "time"]) {
      expect(rootLib).not.toMatch(new RegExp(`mod\\s+${moduleName}\\s*;`));
      expect(rootLib).toMatch(new RegExp(`mod\\s+${moduleName}\\s*\\{[\\s\\S]*extractum_core::${moduleName}::\\{`));
    }
    expect(rootLib).not.toMatch(/extractum_core::(?:compression|error|time)::\*/);
  });

  it("runs complete Cargo gates for the entire workspace", () => {
    expect(packageJson.scripts["test:rust"]).toBe(
      "cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets",
    );
    expect(packageJson.scripts["test:rust:prompt-pack-runs"]).toBe(
      promptPackCrateExtracted
        ? "cargo test --manifest-path src-tauri/Cargo.toml -p extractum-prompt-packs --lib prompt_pack_run"
        : "cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib prompt_pack_run",
    );
    expect(packageJson.scripts["check:rustfmt"]).toBe(
      "cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check",
    );
    expect(verifySource).toMatch(/args:\s*\['check',[\s\S]*'--workspace',[\s\S]*'--all-targets'[\s\S]*\]/);
    expect(verifySource).toMatch(/args:\s*\['test',[\s\S]*'--workspace',[\s\S]*'--all-targets'[\s\S]*\]/);
  });

  it("documents the workspace-aware daily loop and canonical target", () => {
    for (const guidance of [agentGuidance, projectGuidance]) {
      expect(guidance).toContain("<!-- daily-development-loop -->");
      expect(guidance).toContain("src-tauri/target");
      expect(guidance).toContain("--workspace --all-targets");
    }
  });
});
