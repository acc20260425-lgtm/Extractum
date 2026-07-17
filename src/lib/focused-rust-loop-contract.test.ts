import { describe, expect, it } from "vitest";

import agentGuidanceRaw from "../../AGENTS.md?raw";

const agentGuidance = agentGuidanceRaw.replace(/\r\n/g, "\n");
const policyAnchor = "<!-- focused-rust-loop -->";
const policyStart = agentGuidance.indexOf(policyAnchor);
const nextHeading = policyStart < 0 ? -1 : agentGuidance.indexOf("\n## ", policyStart);
const focusedPolicy =
  policyStart < 0
    ? ""
    : agentGuidance.slice(policyStart, nextHeading < 0 ? undefined : nextHeading);

const focusedCheck =
  "cargo check --manifest-path src-tauri/Cargo.toml -p <package> --all-targets";
const focusedTest =
  "cargo test --manifest-path src-tauri/Cargo.toml -p <package> --lib <full-test-name> -- --exact";
const packageCheckpoint =
  "cargo test --manifest-path src-tauri/Cargo.toml -p <package> --all-targets";
const workspaceCheck =
  "cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets";
const workspaceTest =
  "cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets";

describe("focused Rust loop repository policy", () => {
  it("owns canonical focused package commands", () => {
    expect(focusedPolicy, "missing focused Rust loop policy anchor").not.toBe("");
    expect(focusedPolicy).toContain(focusedCheck);
    expect(focusedPolicy).toContain(focusedTest);
    expect(focusedPolicy).toContain(packageCheckpoint);
    expect(focusedPolicy).toContain("-p extractum");
    expect(focusedPolicy).toContain("src-tauri/target");
  });

  it("separates focused feedback from full completion gates", () => {
    expect(focusedPolicy, "missing focused Rust loop policy anchor").not.toBe("");
    expect(focusedPolicy).toContain("`0 tests` is not verification");
    expect(focusedPolicy).toContain(workspaceCheck);
    expect(focusedPolicy).toContain(workspaceTest);
    expect(focusedPolicy).toContain("npm.cmd run verify");
    expect(focusedPolicy).toContain("accelerators, not completion evidence");
  });

  it("documents plan shape, cold starts, and deferred integration feedback", () => {
    expect(focusedPolicy, "missing focused Rust loop policy anchor").not.toBe("");
    expect(focusedPolicy).toContain("`## Rust Verification Loops`");
    expect(focusedPolicy).toContain("first Rust check in a session may be cold and slower");
    expect(focusedPolicy).toContain("public cross-crate interface");
    expect(focusedPolicy).toContain("immediate dependent package");
  });
});
