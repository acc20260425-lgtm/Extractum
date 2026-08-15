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
    packageJson: { dependencies: { "@tauri-apps/api": "^2" }, devDependencies: {} },
  };
}

describe("Rust dependency policy", () => {
  it("generates and validates a reviewed schema-2 policy", () => {
    const generated = generateRustDependencyPolicy({ ...fixture(), reviewed });
    const committed = { ...reviewed, directRequirements: generated.directRequirements, npmRequirements: generated.npmRequirements, approvedPrereleases: generated.approvedPrereleases };
    expect(validateRustDependencyPolicy({ generated, committed })).toEqual([]);
    expect(cargoRequirementIdentity(reviewed.tauriFamily.pairs[0].cargo)).toBe('["extractum","tauri",null,"normal",null]');
    expect(npmRequirementIdentity(reviewed.tauriFamily.pairs[0].npm)).toBe('["extractum","@tauri-apps/api","dependencies"]');
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
  ])("reports %s without throwing", (_name, mutate) => {
    const generated = generateRustDependencyPolicy({ ...fixture(), reviewed });
    const committed = structuredClone({ ...reviewed, directRequirements: generated.directRequirements, npmRequirements: generated.npmRequirements, approvedPrereleases: generated.approvedPrereleases });
    mutate(committed);
    expect(validateRustDependencyPolicy({ generated, committed })).not.toEqual([]);
  });

  it("rejects schema 1 and preserves reviewed tauri authority on generation", () => {
    const generated = generateRustDependencyPolicy({ ...fixture(), reviewed });
    expect(JSON.stringify(generated.tauriFamily)).toBe(JSON.stringify(reviewed.tauriFamily));
    expect(validateRustDependencyPolicy({ generated, committed: { schemaVersion: 1 } })).toContain("schemaVersion must be exactly 2");
  });
});
