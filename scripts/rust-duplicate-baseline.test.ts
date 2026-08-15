import { describe, expect, it } from "vitest";
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import {
  checkRustDuplicatePolicy,
  compareDuplicateGrowth,
  generateRustDuplicateBaseline,
  validateCurrentDuplicateState,
  validateDuplicateBaseline,
  writeRustDuplicateBaseline,
} from "./rust-duplicate-baseline.mjs";

describe("Rust duplicate baseline", () => {
  it("counts unique package versions and localizes duplicate cardinality", () => {
    const baseline = generateRustDuplicateBaseline([
      "extractum v0.2.0 (G:\\Develop\\Extractum\\src-tauri)",
      "getrandom v0.2.17",
      "getrandom v0.4.3 (*)",
      "getrandom v0.2.17 (*)",
      "serde v1.0.229",
    ].join("\n"));

    expect(baseline).toEqual({
      schemaVersion: 1,
      target: "x86_64-pc-windows-msvc",
      duplicateNameCount: 1,
      duplicateVersionInstanceCount: 2,
      duplicateCardinality: { getrandom: 2 },
    });
  });

  it("sorts duplicate names stably and ignores non-package lines", () => {
    const baseline = generateRustDuplicateBaseline([
      "warning: ignored by cargo tree",
      "zeta v1.0.0",
      "alpha v1.0.0 (*)",
      "zeta v2.0.0 (*)",
      "alpha v2.0.0",
      "alpha v1.0.0",
    ].join("\n"));

    expect(baseline).toEqual({
      schemaVersion: 1,
      target: "x86_64-pc-windows-msvc",
      duplicateNameCount: 2,
      duplicateVersionInstanceCount: 4,
      duplicateCardinality: { alpha: 2, zeta: 2 },
    });
  });
});

describe("strict duplicate current state", () => {
  const baseline = generateRustDuplicateBaseline("alpha v1.0.0\nalpha v2.0.0");
  const exceptions = { schemaVersion: 1, licenseExceptions: [], advisoryExceptions: [], duplicateGrowthExceptions: [] };
  const treeFor = (names: Record<string, number>) => Object.entries(names).flatMap(([name, count]) =>
    Array.from({ length: count }, (_, index) => `${name} v${index + 1}.0.0`)).join("\n");

  it.each([
    ["equality", { alpha: 2 }, []],
    ["stale reduction", { alpha: 1 }, ["current duplicate graph differs from committed baseline"]],
    ["growth", { alpha: 3 }, ["current duplicate graph differs from committed baseline"]],
    ["name replacement", { beta: 2 }, ["current duplicate graph differs from committed baseline"]],
  ])("enforces %s", (_name, current, expected) => {
    expect(validateCurrentDuplicateState({ treeText: treeFor(current), baseline, exceptions }).violations).toEqual(expected);
  });

  it("accumulates exact historical approvals in package order", () => {
    expect(compareDuplicateGrowth({ current: { duplicateCardinality: { alpha: 3, beta: 3 } }, base: { duplicateCardinality: { alpha: 2, beta: 2 } }, exceptions })).toEqual([
      "alpha: duplicate growth 2 -> 3 requires exact approval",
      "beta: duplicate growth 2 -> 3 requires exact approval",
    ]);
    expect(compareDuplicateGrowth({ current: { duplicateCardinality: { getrandom: 3 } }, base: { duplicateCardinality: { getrandom: 2 } }, exceptions: {
      ...exceptions,
      duplicateGrowthExceptions: [{ package: "getrandom", previousCount: 2, approvedCount: 3, owner: "owner", reason: "reason", reviewAfter: "2027-01-01" }],
    } })).toEqual([]);
    expect(compareDuplicateGrowth({ current: { duplicateCardinality: { syn: 2 } }, base: { duplicateCardinality: {} }, exceptions })).toEqual([
      "syn: duplicate growth 1 -> 2 requires exact approval",
    ]);
    expect(compareDuplicateGrowth({ current: { duplicateCardinality: { getrandom: 3 } }, base: { duplicateCardinality: { getrandom: 2 } }, exceptions: {
      ...exceptions,
      duplicateGrowthExceptions: [{ package: "getrandom", previousCount: 1, approvedCount: 3, owner: "owner", reason: "reason", reviewAfter: "2027-01-01" }],
    } })).toEqual(["getrandom: duplicate growth 2 -> 3 requires exact approval"]);
  });

  it("validates schema through the current-state entry point", () => {
    expect(() => validateDuplicateBaseline({ ...baseline, duplicateNameCount: 9 })).toThrow(/duplicateNameCount/);
    expect(() => validateDuplicateBaseline({ ...baseline, extra: true })).toThrow(/schema/);
    expect(validateCurrentDuplicateState({ treeText: treeFor({ alpha: 2 }), baseline, exceptions: { ...exceptions, duplicateGrowthExceptions: [{ package: "alpha" }] } }).violations[0]).toMatch(/exception/);
    expect(validateCurrentDuplicateState({ treeText: treeFor({ alpha: 2 }), baseline, exceptions: { schemaVersion: 1, duplicateGrowthExceptions: [] } }).violations[0]).toMatch(/schema/);
    const approval = { package: "alpha", previousCount: 2, approvedCount: 3, owner: "owner", reason: "reason", reviewAfter: "2027-01-01" };
    expect(compareDuplicateGrowth({ current: { duplicateCardinality: { alpha: 3 } }, base: { duplicateCardinality: { alpha: 2 } }, exceptions: { ...exceptions, duplicateGrowthExceptions: [approval, { ...approval }] } })).toContain("duplicate exception packages must be sorted and unique");
    expect(validateCurrentDuplicateState({ treeText: treeFor({ alpha: 2 }), baseline, exceptions: {
      ...exceptions,
      duplicateGrowthExceptions: [
        { ...approval, package: "zeta" },
        { ...approval, package: "beta" },
      ],
    } }).violations).toContain("duplicate exception packages must be sorted and unique");
  });

  it("exports the check and writer entry points", () => {
    expect(typeof checkRustDuplicatePolicy).toBe("function");
    expect(typeof writeRustDuplicateBaseline).toBe("function");
  });
});

describe("duplicate policy CLI history", () => {
  const baseline = generateRustDuplicateBaseline("alpha v1.0.0\nalpha v2.0.0");
  const exceptions = { schemaVersion: 1, licenseExceptions: [], advisoryExceptions: [], duplicateGrowthExceptions: [] };

  function fixture(baseBaseline?: string, baseExceptions?: string) {
    const cwd = mkdtempSync(path.join(tmpdir(), "extractum-duplicates-"));
    const environment = { ...process.env, GIT_CONFIG_GLOBAL: process.platform === "win32" ? "NUL" : "/dev/null" };
    const git = (args: string[]) => execFileSync("git", args, { cwd, env: environment, stdio: ["ignore", "pipe", "ignore"] });
    git(["init"]);
    git(["config", "user.email", "test@example.test"]);
    git(["config", "user.name", "Test"]);
    git(["config", "core.autocrlf", "false"]);
    mkdirSync(path.join(cwd, "scripts/testing"), { recursive: true });
    writeFileSync(path.join(cwd, "seed.txt"), "seed\n");
    if (baseBaseline !== undefined) writeFileSync(path.join(cwd, "scripts/testing/rust-duplicate-baseline.json"), baseBaseline);
    if (baseExceptions !== undefined) writeFileSync(path.join(cwd, "scripts/testing/rust-supply-chain-exceptions.json"), baseExceptions);
    git(["add", "."]);
    git(["commit", "-m", "seed"]);
    const base = git(["rev-parse", "HEAD"]).toString("utf8").trim();
    writeFileSync(path.join(cwd, "scripts/testing/rust-duplicate-baseline.json"), `${JSON.stringify(baseline)}\n`);
    writeFileSync(path.join(cwd, "scripts/testing/rust-supply-chain-exceptions.json"), `${JSON.stringify(exceptions)}\n`);
    return { cwd, base };
  }

  it("rejects an invalid revision", () => {
    const repo = fixture();
    try {
      expect(() => checkRustDuplicatePolicy({ base: "missing", cwd: repo.cwd, treeText: "alpha v1.0.0\nalpha v2.0.0" })).toThrow(/revision/);
    } finally { rmSync(repo.cwd, { recursive: true, force: true }); }
  });

  it.each([
    ["both absent", undefined, undefined],
    ["baseline absent", undefined, `${JSON.stringify(exceptions)}\n`],
    ["exceptions absent", `${JSON.stringify(baseline)}\n`, undefined],
  ])("skips when %s", (_name, oldBaseline, oldExceptions) => {
    const repo = fixture(oldBaseline, oldExceptions);
    let output = "";
    try {
      expect(checkRustDuplicatePolicy({ base: repo.base, cwd: repo.cwd, treeText: "alpha v1.0.0\nalpha v2.0.0", stderr: { write: (text: string) => { output += text; } } as any })).toEqual({ historicalSkipped: true });
      expect(output).toBe("historical duplicate policy unavailable; skipping base comparison\n");
    } finally { rmSync(repo.cwd, { recursive: true, force: true }); }
  });

  it("fails current equality before an absent-history skip", () => {
    const repo = fixture();
    let output = "";
    try {
      expect(() => checkRustDuplicatePolicy({ base: repo.base, cwd: repo.cwd, treeText: "alpha v1.0.0\nalpha v2.0.0\nalpha v3.0.0", stderr: { write: (text: string) => { output += text; } } as any })).toThrow(/current duplicate graph/);
      expect(output).toBe("");
    } finally { rmSync(repo.cwd, { recursive: true, force: true }); }
  });

  it("rejects malformed present history and compares known present history", () => {
    const malformed = fixture("{", `${JSON.stringify(exceptions)}\n`);
    try {
      expect(() => checkRustDuplicatePolicy({ base: malformed.base, cwd: malformed.cwd, treeText: "alpha v1.0.0\nalpha v2.0.0" })).toThrow();
    } finally { rmSync(malformed.cwd, { recursive: true, force: true }); }
    const malformedBaseline = fixture(`${JSON.stringify({ ...baseline, extra: true })}\n`, `${JSON.stringify(exceptions)}\n`);
    try {
      expect(() => checkRustDuplicatePolicy({ base: malformedBaseline.base, cwd: malformedBaseline.cwd, treeText: "alpha v1.0.0\nalpha v2.0.0" })).toThrow(/schema/);
    } finally { rmSync(malformedBaseline.cwd, { recursive: true, force: true }); }
    const malformedExceptions = fixture(`${JSON.stringify(baseline)}\n`, `${JSON.stringify({ schemaVersion: 1, duplicateGrowthExceptions: [] })}\n`);
    try {
      expect(() => checkRustDuplicatePolicy({ base: malformedExceptions.base, cwd: malformedExceptions.cwd, treeText: "alpha v1.0.0\nalpha v2.0.0" })).toThrow(/schema/);
    } finally { rmSync(malformedExceptions.cwd, { recursive: true, force: true }); }
    const known = fixture(`${JSON.stringify(baseline)}\n`, `${JSON.stringify(exceptions)}\n`);
    let output = "";
    try {
      expect(checkRustDuplicatePolicy({ base: known.base, cwd: known.cwd, treeText: "alpha v1.0.0\nalpha v2.0.0", stderr: { write: (text: string) => { output += text; } } as any })).toEqual({ historicalSkipped: false });
      expect(output).toBe("");
    } finally { rmSync(known.cwd, { recursive: true, force: true }); }
  }, 15_000);
});
