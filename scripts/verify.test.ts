import { describe, expect, it, vi } from "vitest";

import { createVerifySteps, runVerification } from "./verify.mjs";

describe("verify", () => {
  it("runs distinct adapter and app e2e gates between the transition and preserved static gates", () => {
    const steps = createVerifySteps({ npmExecPath: "npm-cli.js", platform: "win32" });
    expect(steps.map((step) => step.npmScript ?? step.command)).toEqual([
      "check:gemini-browser-sidecar-binary",
      process.execPath,
      "test:unit",
      "test:component",
      "test:architecture",
      "test:legacy-contract",
      "test:integration:os",
      "test:e2e",
      "test:app:e2e",
      "check",
      "check:rustfmt",
      "cargo",
      "cargo",
      "git",
    ]);
    expect(steps[1]).toMatchObject({ command: process.execPath, args: ["scripts/validate-testing-transition.mjs"] });
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
