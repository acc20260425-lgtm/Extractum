import { describe, expect, it, vi } from "vitest";

import { createVerifySteps, runVerification } from "./verify.mjs";

describe("verify", () => {
  it("starts with the binary and transition gates, then preserves the previous gates", () => {
    const steps = createVerifySteps({ npmExecPath: "npm-cli.js", platform: "win32" });
    expect(steps.map((step) => step.npmScript ?? step.command)).toEqual([
      "check:gemini-browser-sidecar-binary",
      process.execPath,
      "test",
      "check",
      "check:rustfmt",
      "cargo",
      "cargo",
      "git",
    ]);
    expect(steps[1]).toMatchObject({ command: process.execPath, args: ["scripts/validate-testing-transition.mjs"] });
    expect(steps.filter((step) => step.npmScript).map((step) => step.npmScript)).not.toEqual(expect.arrayContaining([
      "bootstrap:testing", "build:gemini-browser-sidecar",
    ]));
  });

  it("is sequential and fail-fast", async () => {
    const runStep = vi.fn(async (step: { title: string }) => step.title === "second" ? 7 : 0);
    const result = await runVerification({ steps: [{ title: "first" }, { title: "second" }, { title: "third" }], runStep });
    expect(result).toEqual({ exitCode: 7, failedStep: "second" });
    expect(runStep.mock.calls.map(([step]) => step.title)).toEqual(["first", "second"]);
  });
});
