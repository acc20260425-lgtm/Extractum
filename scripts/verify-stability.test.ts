import { createRequire } from "node:module";

import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  listWindowsProcesses,
  parseStabilityArgs,
  runChromiumLifecycleAudit,
  runStabilityCli,
} from "./verify-stability.mjs";

const require = createRequire(import.meta.url);

const cleanProcess = {
  ProcessId: 100,
  ParentProcessId: 1,
  ExecutablePath: "C:\\Windows\\System32\\powershell.exe",
  CommandLine: "powershell.exe -NoProfile",
};

const leakedPlaywrightChromium = {
  ProcessId: 201,
  ParentProcessId: 200,
  ExecutablePath: "C:\\pw\\chromium_headless_shell\\chrome-headless-shell.exe",
  CommandLine: 'chrome-headless-shell.exe --headless --user-data-dir="C:\\Users\\tester\\AppData\\Local\\Temp\\playwright_chromiumdev_profile-a1b2" --remote-debugging-pipe',
};

function successfulRun() {
  return Promise.resolve({ exitCode: 0, stdout: "", stderr: "" });
}

function processSnapshots(...snapshots: Array<Array<typeof cleanProcess>>) {
  let index = 0;
  return vi.fn(async () => snapshots[index++] ?? snapshots.at(-1) ?? []);
}

beforeEach(() => {
  vi.spyOn(console, "log").mockImplementation(() => undefined);
  vi.spyOn(console, "error").mockImplementation(() => undefined);
});

describe("verify:stability argument contract", () => {
  it("accepts only the exact Chromium lifecycle invocation", () => {
    expect(parseStabilityArgs(["--suite", "chromium-lifecycle", "--runs", "20"])).toEqual({
      suite: "chromium-lifecycle",
      runs: 20,
    });

    for (const args of [
      [],
      ["--suite", "chromium-lifecycle", "--runs", "19"],
      ["--runs", "20", "--suite", "chromium-lifecycle"],
      ["--suite=chromium-lifecycle", "--runs=20"],
      ["--suite", "chromium-lifecycle", "--runs", "20", "--extra"],
    ]) {
      expect(parseStabilityArgs(args)).toBeNull();
    }
  });

  it("returns exit 2 without touching processes for invalid input", async () => {
    const runCommand = vi.fn(successfulRun);
    const listProcesses = vi.fn(async () => [cleanProcess]);

    await expect(runStabilityCli(["--suite", "chromium-lifecycle"], { runCommand, listProcesses })).resolves.toBe(2);

    expect(runCommand).not.toHaveBeenCalled();
    expect(listProcesses).not.toHaveBeenCalled();
    expect(console.error).toHaveBeenCalledWith(
      "Usage: npm.cmd run verify:stability -- --suite chromium-lifecycle --runs 20",
    );
  });
});

describe("Chromium lifecycle audit", () => {
  it("runs the resolved Playwright CLI twenty times sequentially with shell disabled", async () => {
    let active = 0;
    let maximumActive = 0;
    const runCommand = vi.fn(async () => {
      active += 1;
      maximumActive = Math.max(maximumActive, active);
      await Promise.resolve();
      active -= 1;
      return { exitCode: 0, stdout: "", stderr: "" };
    });
    const listProcesses = processSnapshots([cleanProcess], [cleanProcess]);

    await expect(runChromiumLifecycleAudit({ runCommand, listProcesses })).resolves.toEqual({ exitCode: 0 });

    expect(runCommand).toHaveBeenCalledTimes(20);
    expect(maximumActive).toBe(1);
    expect(runCommand).toHaveBeenNthCalledWith(
      1,
      process.execPath,
      [
        require.resolve("@playwright/test/cli"),
        "test",
        "-c",
        "research/gemini_browser_adapter/playwright.config.ts",
        "chromium-lifecycle",
      ],
      expect.objectContaining({ shell: false }),
    );
    expect(console.log).toHaveBeenCalledTimes(20);
    expect(console.log).toHaveBeenNthCalledWith(1, "chromium-lifecycle run 1/20: pass");
    expect(console.log).toHaveBeenNthCalledWith(20, "chromium-lifecycle run 20/20: pass");
    expect(listProcesses).toHaveBeenCalledTimes(2);
  });

  it("does not retry and stops at the first failed normal run", async () => {
    const runCommand = vi
      .fn()
      .mockResolvedValueOnce({ exitCode: 0, stdout: "", stderr: "" })
      .mockResolvedValueOnce({ exitCode: 1, stdout: "", stderr: "assertion failed" });
    const listProcesses = processSnapshots([cleanProcess], [cleanProcess]);

    await expect(runChromiumLifecycleAudit({ runCommand, listProcesses })).resolves.toEqual({
      exitCode: 1,
      diagnostic: "Playwright run 2/20 exited with code 1",
    });

    expect(runCommand).toHaveBeenCalledTimes(2);
    expect(console.log).toHaveBeenCalledWith("chromium-lifecycle run 1/20: pass");
    expect(console.log).toHaveBeenCalledWith("chromium-lifecycle run 2/20: fail");
    expect(listProcesses).toHaveBeenCalledTimes(2);
  });

  it("maps child spawn and process enumeration errors to exit 3", async () => {
    const snapshots = processSnapshots([cleanProcess], [cleanProcess]);
    const spawnError = await runChromiumLifecycleAudit({
      runCommand: vi.fn(async () => { throw new Error("spawn denied"); }),
      listProcesses: snapshots,
    });
    expect(spawnError).toEqual({ exitCode: 3, diagnostic: "Playwright spawn failed: spawn denied" });

    const enumerationError = await runChromiumLifecycleAudit({
      runCommand: vi.fn(successfulRun),
      listProcesses: vi.fn(async () => { throw new Error("CIM unavailable"); }),
    });
    expect(enumerationError).toEqual({ exitCode: 3, diagnostic: "Process enumeration failed: CIM unavailable" });
  });

  it("fails only for a newly created Playwright headless Chromium temp profile", async () => {
    const preExisting = { ...leakedPlaywrightChromium, ProcessId: 200 };
    const ordinaryUserChrome = {
      ProcessId: 202,
      ParentProcessId: 1,
      ExecutablePath: "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
      CommandLine: 'chrome.exe --user-data-dir="C:\\Users\\tester\\Chrome Profile"',
    };
    const listProcesses = processSnapshots(
      [cleanProcess, preExisting],
      [cleanProcess, preExisting, ordinaryUserChrome, leakedPlaywrightChromium],
    );

    await expect(runChromiumLifecycleAudit({ runCommand: vi.fn(successfulRun), listProcesses })).resolves.toEqual({
      exitCode: 1,
      diagnostic: "Leaked Playwright Chromium process IDs: 201",
    });
  });

  it("ignores pre-existing Playwright matches and newly created ordinary user Chrome", async () => {
    const ordinaryUserChrome = {
      ProcessId: 202,
      ParentProcessId: 1,
      ExecutablePath: "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
      CommandLine: 'chrome.exe --user-data-dir="C:\\Users\\tester\\Chrome Profile"',
    };
    const listProcesses = processSnapshots(
      [cleanProcess, leakedPlaywrightChromium],
      [cleanProcess, leakedPlaywrightChromium, ordinaryUserChrome],
    );

    await expect(runChromiumLifecycleAudit({ runCommand: vi.fn(successfulRun), listProcesses })).resolves.toEqual({
      exitCode: 0,
    });
  });
});

describe("Windows process snapshots", () => {
  it("uses the bounded CIM projection and parses a single compact JSON object", async () => {
    const executeProcess = vi.fn(async () => ({
      exitCode: 0,
      stdout: JSON.stringify(cleanProcess),
      stderr: "",
    }));

    await expect(listWindowsProcesses({ executeProcess })).resolves.toEqual([cleanProcess]);
    expect(executeProcess).toHaveBeenCalledWith(
      "powershell.exe",
      [
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        "Get-CimInstance Win32_Process | Select-Object ProcessId,ParentProcessId,ExecutablePath,CommandLine | ConvertTo-Json -Compress",
      ],
      expect.objectContaining({ shell: false }),
    );
  });

  it("rejects CIM failures and malformed JSON", async () => {
    await expect(listWindowsProcesses({
      executeProcess: vi.fn(async () => ({ exitCode: 1, stdout: "", stderr: "Access denied" })),
    })).rejects.toThrow("CIM command exited with code 1: Access denied");

    await expect(listWindowsProcesses({
      executeProcess: vi.fn(async () => ({ exitCode: 0, stdout: "not-json", stderr: "" })),
    })).rejects.toThrow("Unable to parse CIM JSON");
  });
});
