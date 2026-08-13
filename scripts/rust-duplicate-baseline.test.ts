import { describe, expect, it } from "vitest";

import { generateRustDuplicateBaseline } from "./rust-duplicate-baseline.mjs";

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
