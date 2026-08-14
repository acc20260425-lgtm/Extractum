import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

type Step = Readonly<{
  if?: string;
  run?: string;
  text: readonly string[];
  uses?: string;
}>;

type Job = Readonly<{
  if?: string;
  needs?: string;
  text: readonly string[];
  steps: readonly Step[];
}>;

function workflow(name: string) {
  return readFileSync(path.resolve(".github", "workflows", name), "utf8");
}

function stripComment(line: string) {
  let quote: "'" | '"' | undefined;
  for (let index = 0; index < line.length; index += 1) {
    const character = line[index];
    if ((character === "'" || character === '"') && line[index - 1] !== "\\") {
      quote = quote === character ? undefined : quote ?? character;
    } else if (character === "#" && !quote) {
      return line.slice(0, index).trimEnd();
    }
  }
  return line.trimEnd();
}

function lines(source: string) {
  return source.replaceAll("\r\n", "\n").split("\n").map(stripComment);
}

function indentation(line: string) {
  return line.length - line.trimStart().length;
}

function childBlock(source: readonly string[], parentIndex: number) {
  const parentIndent = indentation(source[parentIndex]);
  const block: string[] = [];
  for (let index = parentIndex + 1; index < source.length; index += 1) {
    const line = source[index];
    if (line.trim() && indentation(line) <= parentIndent) break;
    block.push(line);
  }
  return block;
}

function section(source: readonly string[], name: string, indent = 0) {
  const index = source.findIndex((line) => indentation(line) === indent && line.trim() === `${name}:`);
  if (index < 0) throw new Error(`missing ${name} section`);
  return childBlock(source, index);
}

function directValue(source: readonly string[], name: string, indent: number) {
  const match = source.find((line) => indentation(line) === indent && line.trimStart().startsWith(`${name}:`));
  return match?.trimStart().slice(name.length + 1).trim();
}

function permissionMap(source: readonly string[], indent: number) {
  return Object.fromEntries(source
    .filter((line) => indentation(line) === indent && /^\s*[A-Za-z-]+:\s*\S+\s*$/.test(line))
    .map((line) => {
      const [key, value] = line.trim().split(/:\s+/, 2);
      return [key, value];
    }));
}

function eventBlock(source: readonly string[], event: string) {
  const on = section(source, "on");
  const index = on.findIndex((line) => indentation(line) === 2 && line.trim() === `${event}:`);
  if (index < 0) throw new Error(`missing ${event} trigger`);
  return childBlock(on, index);
}

function eventNames(source: readonly string[]) {
  return section(source, "on")
    .filter((line) => indentation(line) === 2 && /^[A-Za-z_]+:$/.test(line.trim()))
    .map((line) => line.trim().slice(0, -1));
}

function jobs(source: readonly string[]) {
  const jobLines = section(source, "jobs");
  const result = new Map<string, Job>();
  for (let index = 0; index < jobLines.length; index += 1) {
    const line = jobLines[index];
    const match = indentation(line) === 2 ? /^\s{2}([A-Za-z][\w-]*):$/.exec(line) : undefined;
    if (!match) continue;
    const text = childBlock(jobLines, index);
    result.set(match[1], {
      if: directValue(text, "if", 4),
      needs: directValue(text, "needs", 4),
      text,
      steps: steps(text),
    });
  }
  return result;
}

function steps(jobLines: readonly string[]) {
  const stepsIndex = jobLines.findIndex((line) => indentation(line) === 4 && line.trim() === "steps:");
  if (stepsIndex < 0) return [];
  const stepLines = childBlock(jobLines, stepsIndex);
  const result: Step[] = [];
  for (let index = 0; index < stepLines.length; index += 1) {
    const line = stepLines[index];
    if (indentation(line) !== 6 || !line.trimStart().startsWith("- ")) continue;
    const text = [line, ...childBlock(stepLines, index)];
    const inline = line.trimStart().slice(2);
    result.push({
      if: directValue(text, "if", 8),
      run: inline.startsWith("run:") ? inline.slice(4).trim() : directValue(text, "run", 8),
      text,
      uses: inline.startsWith("uses:") ? inline.slice(5).trim() : directValue(text, "uses", 8),
    });
  }
  return result;
}

function stepSequence(job: Job) {
  return job.steps.flatMap((step) => step.uses
    ? [`uses:${step.uses}`]
    : step.run ? [`run:${step.run}`] : []);
}

function assertInOrder(actual: readonly string[], expected: readonly string[]) {
  let cursor = 0;
  for (const value of actual) {
    if (value === expected[cursor]) cursor += 1;
  }
  if (cursor !== expected.length) throw new Error(`missing ordered step: ${expected[cursor]}`);
}

function requireInvariant(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

function assertReadOnlyPermissions(source: readonly string[]) {
  const permissions = permissionMap(section(source, "permissions"), 2);
  requireInvariant(permissions.contents === "read", "workflow must grant contents: read");
  requireInvariant(!Object.values(permissions).some((permission) => permission === "write"), "workflow must not grant write scopes");
}

function assertFastWorkflow(source: string) {
  const parsed = lines(source);
  assertReadOnlyPermissions(parsed);
  requireInvariant(eventNames(parsed).join(",") === "push,pull_request", "fast triggers must be push plus pull_request");
  requireInvariant(directValue(eventBlock(parsed, "push"), "branches", 4) === "[main]", "fast push must target only main");
  const fast = jobs(parsed).get("rust-fast");
  requireInvariant(fast, "missing rust-fast job");
  assertInOrder(stepSequence(fast), [
    "uses:actions/checkout@v6",
    "run:rustup show active-toolchain",
    "uses:Swatinem/rust-cache@v2",
    "uses:./.github/actions/setup-cargo-deny",
    "run:npm.cmd run check:rust:fast",
  ]);
  requireInvariant(!/\bnpm(?:\.cmd)?\s+(?:ci|install)\b/.test(fast.text.join("\n")), "fast job must not install npm dependencies");
}

function assertFullWorkflow(source: string) {
  const parsed = lines(source);
  assertReadOnlyPermissions(parsed);
  requireInvariant(eventNames(parsed).join(",") === "pull_request,workflow_dispatch", "full triggers must be PR plus manual dispatch");
  const full = jobs(parsed).get("rust-full");
  requireInvariant(full, "missing rust-full job");
  assertInOrder(stepSequence(full), [
    "uses:actions/checkout@v6",
    "uses:actions/setup-node@v6",
    "run:rustup show active-toolchain",
    "uses:Swatinem/rust-cache@v2",
    "uses:./.github/actions/setup-cargo-deny",
    "run:npm.cmd ci",
    "run:npm.cmd run bootstrap:testing",
    "run:node_modules\\.bin\\playwright.cmd install chromium",
    "run:npm.cmd run check:rust:fast",
    "run:npm.cmd run verify",
  ]);
}

function assertReleaseWorkflow(source: string) {
  const parsed = lines(source);
  const releaseJobs = jobs(parsed);
  const advisories = releaseJobs.get("advisories");
  requireInvariant(advisories, "missing advisories job");
  requireInvariant(stepSequence(advisories).includes("run:npm.cmd run check:rust:advisories"), "advisories job must own advisory command");
  const advisoryPermissions = permissionMap(section(advisories.text, "permissions", 4), 6);
  requireInvariant(advisoryPermissions.contents === "read" && advisoryPermissions.issues === "write", "advisories must own its read/write permissions");
  const concurrency = section(advisories.text, "concurrency", 4);
  requireInvariant(directValue(concurrency, "group", 6) === "rust-advisory-follow-up", "advisory follow-up needs a stable concurrency group");
  requireInvariant(directValue(concurrency, "cancel-in-progress", 6) === "false", "advisory follow-up must not cancel a prior run");
  const followUp = advisories.steps.find((step) => step.uses === "actions/github-script@v8");
  requireInvariant(followUp?.if?.includes("failure()") && followUp.if.includes("schedule") && followUp.if.includes("workflow_dispatch"), "advisory follow-up must run after scheduled/manual failure");
  requireInvariant(source.includes(String.raw`.join("\n")`), "advisory body must join paragraphs with a newline");

  const bundle = releaseJobs.get("windows-bundle");
  requireInvariant(bundle, "missing windows-bundle job");
  requireInvariant(bundle.needs === "advisories", "bundle must need advisories");
  requireInvariant(bundle.if?.includes("needs.advisories.result == 'success'")
    && bundle.if.includes("startsWith(github.ref, 'refs/tags/v')")
    && bundle.if.includes("github.event_name == 'workflow_dispatch'")
    && bundle.if.includes("inputs.bundle"), "bundle must require advisory success and tag/manual gate");
  assertInOrder(stepSequence(bundle), [
    "run:npm.cmd run check:rust:fast",
    "run:npm.cmd run verify",
    "run:npm.cmd run tauri -- build --target x86_64-pc-windows-msvc",
    "run:npm.cmd run smoke:gemini-browser-sidecar:binary",
    "uses:actions/upload-artifact@v4",
    "uses:actions/upload-artifact@v4",
  ]);
  const uploads = bundle.steps.filter((step) => step.uses === "actions/upload-artifact@v4");
  requireInvariant(uploads.length === 2
    && uploads[0].text.join("\n").includes("name: extractum-msi")
    && uploads[1].text.join("\n").includes("name: extractum-nsis"), "bundle must upload MSI then NSIS artifacts");
}

describe("Rust GitHub workflow contracts", () => {
  it("keeps the fast workflow main-only, read-only, and installation-free", () => {
    assertFastWorkflow(workflow("rust-fast.yml"));
  });

  it("keeps the full workflow read-only with its canonical owning-job sequence", () => {
    assertFullWorkflow(workflow("rust-full.yml"));
  });

  it("serializes advisory follow-up and gates bundle release on advisory success", () => {
    assertReleaseWorkflow(workflow("rust-release.yml"));
  });

  it("rejects a feature-branch fast trigger", () => {
    expect(() => assertFastWorkflow(workflow("rust-fast.yml").replace("branches: [main]", "branches: [release]"))).toThrow("fast push must target only main");
  });

  it("rejects a full command moved into the wrong job", () => {
    const moved = workflow("rust-full.yml").replace("      - run: npm.cmd run verify\n", "")
      + "\n  unrelated:\n    runs-on: windows-latest\n    steps:\n      - run: npm.cmd run verify\n";
    expect(() => assertFullWorkflow(moved)).toThrow("missing ordered step: run:npm.cmd run verify");
  });

  it("rejects a bundle without advisory dependency", () => {
    expect(() => assertReleaseWorkflow(workflow("rust-release.yml").replace("needs: advisories", "needs: fast"))).toThrow("bundle must need advisories");
  });

  it("rejects a bundle without the advisory-success gate", () => {
    expect(() => assertReleaseWorkflow(workflow("rust-release.yml").replace("needs.advisories.result == 'success'", "true"))).toThrow("bundle must require advisory success and tag/manual gate");
  });
});
