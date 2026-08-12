import { readFileSync } from "node:fs";
import { describe, expect, it, vi } from "vitest";

import { createVerifySteps, runVerification } from "./verify.mjs";

describe("verify", () => {
  it("runs distinct adapter and app e2e gates between the preserved verification gates", () => {
    const steps = createVerifySteps({ npmExecPath: "npm-cli.js", platform: "win32" });
    expect(steps.map((step) => step.npmScript ?? step.command)).toEqual([
      "check:gemini-browser-sidecar-binary",
      "test:unit",
      "test:component",
      "test:architecture",
      "test:integration:os",
      "test:e2e",
      "test:app:e2e",
      "check",
      "check:rustfmt",
      "cargo",
      "git",
    ]);
    const cargoSteps = steps.filter((step) => step.command === "cargo");
    expect(cargoSteps).toEqual([{
      title: "cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets --locked",
      command: "cargo",
      args: ["test", "--manifest-path", "src-tauri/Cargo.toml", "--workspace", "--all-targets", "--locked"],
    }]);

    const packageJson = JSON.parse(
      readFileSync(new URL("../package.json", import.meta.url), "utf8"),
    );
    expect(packageJson.scripts["bootstrap:testing"]).toBe(
      "svelte-kit sync && npm run build:gemini-browser-sidecar && npm run check:gemini-browser-sidecar-binary",
    );
    const npmScripts = steps.filter((step) => step.npmScript).map((step) => step.npmScript);
    expect(npmScripts).not.toContain("test");
    expect(npmScripts).not.toContain("bootstrap:testing");
    expect(npmScripts).not.toContain("build:gemini-browser-sidecar");
  });

  it("is sequential and fail-fast", async () => {
    const runStep = vi.fn(async (step: { title: string }) => step.title === "second" ? 7 : 0);
    const result = await runVerification({ steps: [{ title: "first" }, { title: "second" }, { title: "third" }], runStep });
    expect(result).toEqual({ exitCode: 7, failedStep: "second" });
    expect(runStep.mock.calls.map(([step]) => step.title)).toEqual(["first", "second"]);
  });
});
