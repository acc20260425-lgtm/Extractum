import { describe, expect, it, vi } from "vitest";

import * as runVitestModule from "./run-vitest.mjs";

describe("run vitest wrapper", () => {
  it("excludes Playwright specs from Vitest discovery", () => {
    const spawnSync = vi.fn(() => ({ status: 0, stdout: Buffer.from(""), stderr: Buffer.from("") }));
    const runVitest = (runVitestModule as {
      runVitest?: (options: {
        argv: string[];
        cwd: string;
        spawnSync: typeof spawnSync;
      }) => { status: number; args: string[] };
    }).runVitest;

    expect(runVitest).toBeTypeOf("function");
    const result = runVitest?.({ argv: ["list", "--filesOnly"], cwd: process.cwd(), spawnSync });
    expect(result?.status).toBe(0);
    expect(result?.args).toEqual(expect.arrayContaining([
      "--exclude",
      "research/gemini_browser_adapter/tests/**",
      "--exclude",
      ".worktrees/**",
    ]));

    const discovery = runVitest?.({
      argv: ["list", "--filesOnly", "e2e/app-harness-smoke.spec.ts"],
      cwd: process.cwd(),
    });
    expect(discovery?.status).toBe(0);
    expect(runVitestModule.normalizeVitestListOutput(discovery?.stdout ?? "").trim()).toBe("");
  }, 20_000);
});
