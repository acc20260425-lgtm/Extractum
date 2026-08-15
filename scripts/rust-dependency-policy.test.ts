import { describe, expect, it } from "vitest";

import {
  cargoRequirementIdentity,
  generateRustDependencyPolicy,
  npmRequirementIdentity,
  validateRustDependencyPolicy,
} from "./rust-dependency-policy.mjs";

const reviewed = {
  schemaVersion: 2,
  toolchain: { channel: "1.95.0", rustVersion: "1.95", edition: "2021", target: "x86_64-pc-windows-msvc", workspacePackages: ["extractum"] },
  tauriFamily: {
    pairs: [{ id: "tauri-api", cargo: { package: "extractum", dependency: "tauri", rename: null, kind: "normal", target: null }, cargoRequirement: "^2", npm: { owner: "extractum", name: "@tauri-apps/api", kind: "dependencies" }, npmRequirement: "^2" }],
    cargoOnlyRequirements: [{ id: "tauri-plugin-mcp-bridge", cargo: { package: "extractum", dependency: "tauri-plugin-mcp-bridge", rename: null, kind: "normal", target: null }, cargoRequirement: "^0.11" }],
  },
};

function fixture() {
  return {
    metadata: { workspace_members: ["extractum-id"], packages: [{ id: "extractum-id", name: "extractum", rust_version: "1.95", edition: "2021", publish: [], dependencies: [
      { name: "tauri", rename: null, kind: null, target: null, req: "^2" },
      { name: "tauri-plugin-mcp-bridge", rename: null, kind: null, target: null, req: "^0.11" },
    ] }] },
    packageJson: { dependencies: { "@tauri-apps/api": "^2" }, devDependencies: {}, optionalDependencies: {} },
  };
}

describe("Rust dependency policy", () => {
  it("generates and validates a reviewed schema-2 policy", () => {
    const generated = generateRustDependencyPolicy({ ...fixture(), reviewed });
    const committed = { ...reviewed, directRequirements: generated.directRequirements, npmRequirements: generated.npmRequirements, exactPins: generated.exactPins, approvedPrereleases: generated.approvedPrereleases };
    expect(validateRustDependencyPolicy({ generated, committed })).toEqual([]);
    expect(cargoRequirementIdentity(reviewed.tauriFamily.pairs[0].cargo)).toBe('["extractum","tauri",null,"normal",null]');
    expect(npmRequirementIdentity(reviewed.tauriFamily.pairs[0].npm)).toBe('["extractum","@tauri-apps/api","dependencies"]');
  });

  it("inventories optional npm dependencies and exact Cargo pins", () => {
    const input = fixture();
    input.packageJson.optionalDependencies["@tauri-apps/plugin-policy-gap"] = "^2";
    input.metadata.packages[0].dependencies.push({ name: "pinned", rename: null, kind: null, target: null, req: "=1.2.3" });
    const generated = generateRustDependencyPolicy({ ...input, reviewed });
    expect(generated.npmRequirements).toContainEqual({ owner: "extractum", name: "@tauri-apps/plugin-policy-gap", kind: "optionalDependencies", requirement: "^2" });
    expect(generated.exactPins).toContainEqual({ package: "extractum", dependency: "pinned", rename: null, kind: "normal", target: null, requirement: "=1.2.3" });
    const committed = structuredClone({ ...reviewed, directRequirements: generated.directRequirements, npmRequirements: generated.npmRequirements, exactPins: generated.exactPins, approvedPrereleases: generated.approvedPrereleases });
    expect(validateRustDependencyPolicy({ generated, committed })).toContain("paired npm bijection drifted");
    committed.tauriFamily.pairs.push({
      id: "tauri-policy-gap",
      cargo: structuredClone(committed.tauriFamily.pairs[0].cargo),
      cargoRequirement: "^2",
      npm: { owner: "extractum", name: "@tauri-apps/plugin-policy-gap", kind: "optionalDependencies" },
      npmRequirement: "^2",
    });
    committed.tauriFamily.pairs[1].cargo.dependency = "tauri-policy-gap";
    expect(validateRustDependencyPolicy({ generated, committed })).not.toEqual([]);
  });

  it("rejects exact-pin audit inventory drift", () => {
    const input = fixture();
    input.metadata.packages[0].dependencies.push({ name: "pinned", rename: null, kind: null, target: null, req: "=1.2.3" });
    const generated = generateRustDependencyPolicy({ ...input, reviewed });
    const committed = structuredClone({ ...generated, exactPins: [] });
    expect(validateRustDependencyPolicy({ generated, committed })).toContain("exactPins drifted");
  });

  it.each([
    ["requirement drift", (value: any) => { value.tauriFamily.pairs[0].cargoRequirement = "^3"; }],
    ["moved npm kind", (value: any) => { value.tauriFamily.pairs[0].npm.kind = "devDependencies"; }],
    ["orphan identity", (value: any) => { value.tauriFamily.pairs[0].cargo.dependency = "missing"; }],
    ["missing pair", (value: any) => { value.tauriFamily.pairs = []; }],
    ["widened Cargo-only", (value: any) => { value.tauriFamily.cargoOnlyRequirements[0].cargoRequirement = ">=0.11"; }],
    ["malformed nested", (value: any) => { value.tauriFamily.pairs[0].cargo = null; }],
    ["wrong pair container", (value: any) => { value.tauriFamily.pairs = {}; }],
    ["duplicate pair ID", (value: any) => { value.tauriFamily.pairs.push(structuredClone(value.tauriFamily.pairs[0])); }],
    ["unsorted Cargo-only IDs", (value: any) => { value.tauriFamily.cargoOnlyRequirements.unshift({ ...structuredClone(value.tauriFamily.cargoOnlyRequirements[0]), id: "zzz" }); }],
    ["retargeted Cargo-only identity", (value: any) => { value.tauriFamily.cargoOnlyRequirements[0].cargo.dependency = "tauri"; }],
    ["duplicate Cargo-only identity", (value: any) => { value.tauriFamily.cargoOnlyRequirements.push({ ...structuredClone(value.tauriFamily.cargoOnlyRequirements[0]), id: "zzz" }); }],
    ["extra Cargo-only authority", (value: any) => { value.tauriFamily.cargoOnlyRequirements.push({ id: "zzz", cargo: { package: "extractum", dependency: "serde", rename: null, kind: "normal", target: null }, cargoRequirement: "^1" }); }],
    ["Cargo-only overlaps pair", (value: any) => { value.tauriFamily.cargoOnlyRequirements[0].cargo = structuredClone(value.tauriFamily.pairs[0].cargo); }],
    ["extra authority key", (value: any) => { value.tauriFamily.pairs[0].approval = true; }],
    ["extra Cargo-only key", (value: any) => { value.tauriFamily.cargoOnlyRequirements[0].approval = true; }],
    ["wrong authority scalar", (value: any) => { value.tauriFamily.pairs[0].id = 42; }],
    ["invalid npm kind", (value: any) => { value.tauriFamily.pairs[0].npm.kind = "peerDependencies"; }],
    ["extra toolchain key", (value: any) => { value.toolchain.components = []; }],
    ["reordered pair IDs", (value: any) => { value.tauriFamily.pairs.unshift({ ...structuredClone(value.tauriFamily.pairs[0]), id: "zzz", cargo: { ...structuredClone(value.tauriFamily.pairs[0].cargo), dependency: "tauri-z" }, npm: { ...structuredClone(value.tauriFamily.pairs[0].npm), name: "@tauri-apps/z" } }); }],
  ])("reports %s without throwing", (_name, mutate) => {
    const generated = generateRustDependencyPolicy({ ...fixture(), reviewed });
    const committed = structuredClone({ ...reviewed, directRequirements: generated.directRequirements, npmRequirements: generated.npmRequirements, exactPins: generated.exactPins, approvedPrereleases: generated.approvedPrereleases });
    mutate(committed);
    expect(validateRustDependencyPolicy({ generated, committed })).not.toEqual([]);
  });

  it("rejects schema 1 and preserves reviewed tauri authority on generation", () => {
    const generated = generateRustDependencyPolicy({ ...fixture(), reviewed });
    expect(JSON.stringify(generated.tauriFamily)).toBe(JSON.stringify(reviewed.tauriFamily));
    expect(validateRustDependencyPolicy({ generated, committed: { schemaVersion: 1 } })).toContain("schemaVersion must be exactly 2");
  });
});
