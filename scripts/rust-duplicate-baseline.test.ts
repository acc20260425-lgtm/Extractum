import { describe, expect, it } from "vitest";

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
  });

  it("validates schema through the current-state entry point", () => {
    expect(() => validateDuplicateBaseline({ ...baseline, duplicateNameCount: 9 })).toThrow(/duplicateNameCount/);
    expect(validateCurrentDuplicateState({ treeText: treeFor({ alpha: 2 }), baseline, exceptions: { ...exceptions, duplicateGrowthExceptions: [{ package: "alpha" }] } }).violations[0]).toMatch(/exception/);
  });

  it("exports the check and writer entry points", () => {
    expect(typeof checkRustDuplicatePolicy).toBe("function");
    expect(typeof writeRustDuplicateBaseline).toBe("function");
  });
});
