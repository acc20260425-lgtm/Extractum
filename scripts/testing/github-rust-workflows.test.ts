import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { parse } from "yaml";

type Mapping = Record<string, unknown>;

function workflow(name: string) {
  return readFileSync(path.resolve(".github", "workflows", name), "utf8");
}

function parsedWorkflow(name: string) {
  return asMapping(parse(workflow(name)), `workflow ${name}`);
}

function asMapping(value: unknown, name: string): Mapping {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`missing ${name}`);
  }
  return value as Mapping;
}

function asSteps(value: unknown, name: string) {
  if (!Array.isArray(value)) throw new Error(`missing ${name}`);
  return value.map((step) => asMapping(step, name));
}

function jobs(name: string) {
  return asMapping(parsedWorkflow(name).jobs, "jobs");
}

function requireInvariant(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

function equalMapping(value: unknown, expected: Mapping) {
  if (!value || typeof value !== "object" || Array.isArray(value)) return false;
  const actual = value as Mapping;
  return Object.keys(actual).length === Object.keys(expected).length
    && Object.entries(expected).every(([key, entry]) => actual[key] === entry);
}

function remoteUses(value: unknown): string[] {
  if (Array.isArray(value)) return value.flatMap(remoteUses);
  if (!value || typeof value !== "object") return [];

  return Object.entries(value as Mapping).flatMap(([key, child]) => {
    const current = key === "uses" && typeof child === "string" && !child.startsWith("./") ? [child] : [];
    return [...current, ...remoteUses(child)];
  });
}

function assertRemotePins() {
  for (const name of ["rust-fast.yml", "rust-full.yml", "rust-release.yml"]) {
    for (const action of remoteUses(parsedWorkflow(name))) {
      requireInvariant(/^[^@\s]+@[0-9a-f]{40}$/.test(action), `mutable ref: ${action}`);
    }
  }
}

function jobFrom(allJobs: Mapping, name: string) {
  return asMapping(allJobs[name], `job ${name}`);
}

function assertScannerPermissions(release: Mapping) {
  const permissions = jobFrom(release, "advisory-scan").permissions;
  requireInvariant(equalMapping(permissions, { contents: "read" }), "scanner permissions");
}

function stepById(steps: readonly Mapping[], id: string) {
  const step = steps.find((candidate) => candidate.id === id);
  if (!step) throw new Error(`missing ${id} step`);
  return step;
}

function assertWriterOutcomeIsolation(release: Mapping) {
  const scanner = jobFrom(release, "advisory-scan");
  requireInvariant(equalMapping(scanner.outputs, { advisories_failed: "${{ steps.classify.outputs.advisories_failed }}" }), "writer outcome wiring");

  const scannerSteps = asSteps(scanner.steps, "advisory-scan steps");
  const deny = stepById(scannerSteps, "deny");
  requireInvariant(deny["continue-on-error"] === true && deny.run === "npm.cmd run check:rust:advisories", "writer outcome wiring");
  const classify = stepById(scannerSteps, "classify");
  requireInvariant(
    classify.if === "always()"
      && classify.shell === "pwsh"
      && typeof classify.run === "string"
      && /^"advisories_failed=\$\{\{ steps\.deny\.outcome == 'failure' \}\}" >> \$env:GITHUB_OUTPUT\s*$/.test(classify.run),
    "writer outcome wiring",
  );
  const fail = scannerSteps.find((step) => step.name === "Fail when advisory scan failed");
  requireInvariant(
    fail?.if === "${{ steps.deny.outcome == 'failure' }}" && fail.shell === "pwsh" && fail.run === "exit 1",
    "writer outcome wiring",
  );

  const writer = jobFrom(release, "advisory-issue-writer");
  requireInvariant(
    writer.needs === "advisory-scan"
      && equalMapping(writer.permissions, { issues: "write" })
      && writer.if === "${{ always() && needs.advisory-scan.outputs.advisories_failed == 'true' && (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch') }}",
    "writer outcome wiring",
  );
  for (const [name, candidate] of Object.entries(release)) {
    if (name !== "advisory-issue-writer") {
      requireInvariant(asMapping(candidate, `job ${name}`).permissions === undefined
        || asMapping(asMapping(candidate, `job ${name}`).permissions, `permissions ${name}`).issues !== "write", "writer outcome wiring");
    }
  }
}

function assertBundleScannerSuccess(release: Mapping) {
  const bundle = jobFrom(release, "windows-bundle");
  const condition = String(bundle.if);
  requireInvariant(
    bundle.needs === "advisory-scan"
      && condition.includes("needs.advisory-scan.result == 'success'")
      && !condition.includes("always()")
      && !condition.includes("!cancelled()"),
    "bundle scanner success",
  );
}

describe("Rust GitHub workflow contracts", () => {
  it("enforces remote pins", () => {
    assertRemotePins();
    expect(() => {
      const mutant = workflow("rust-fast.yml").replace(/uses: [^\n]+/, "uses: actions/checkout@v6");
      for (const action of remoteUses(asMapping(parse(mutant), "mutant workflow"))) {
        requireInvariant(/^[^@\s]+@[0-9a-f]{40}$/.test(action), `mutable ref: ${action}`);
      }
    }).toThrow("mutable ref: actions/checkout@v6");
  });

  it("enforces scanner permissions", () => {
    const release = jobs("rust-release.yml");
    assertScannerPermissions(release);
    const mutant = structuredClone(release);
    asMapping(mutant["advisory-scan"], "advisory-scan").permissions = { contents: "read", issues: "write" };
    expect(() => assertScannerPermissions(mutant)).toThrow("scanner permissions");
  });

  it("enforces writer outcome wiring", () => {
    const release = jobs("rust-release.yml");
    assertWriterOutcomeIsolation(release);
    const mutant = structuredClone(release);
    const classify = stepById(asSteps(asMapping(mutant["advisory-scan"], "advisory-scan").steps, "advisory-scan steps"), "classify");
    classify.run = `${String(classify.run)}\n"advisories_failed=true" >> $env:GITHUB_OUTPUT`;
    expect(() => assertWriterOutcomeIsolation(mutant)).toThrow("writer outcome wiring");
  });

  it("requires bundle scanner success", () => {
    const release = jobs("rust-release.yml");
    assertBundleScannerSuccess(release);
    const mutant = structuredClone(release);
    asMapping(mutant["windows-bundle"], "windows-bundle").if = "always()";
    expect(() => assertBundleScannerSuccess(mutant)).toThrow("bundle scanner success");
  });
});
