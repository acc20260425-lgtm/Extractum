import { describe, expect, it, vi } from "vitest";
import type { Page } from "@playwright/test";
import { GeminiBrowserAdapter, waitForFirstVisible } from "./adapter.js";
import {
  GEMINI_DOM_CONTRACT_VERSION,
  answerCandidates,
  composerCandidates,
  sendCandidates,
} from "./dom-contract.js";

describe("production Gemini DOM contract", () => {
  it("keeps the selected resilient-scoring contract version explicit", () => {
    expect(GEMINI_DOM_CONTRACT_VERSION).toBe("2026-06-20-resilient-scoring");
  });

  it("has candidates for composer, send, and answer extraction", () => {
    expect(composerCandidates.length).toBeGreaterThan(0);
    expect(sendCandidates.length).toBeGreaterThan(0);
    expect(answerCandidates.length).toBeGreaterThan(0);
    expect(answerCandidates.some((candidate) => candidate.selector === "main section")).toBe(false);
  });

  it("waits for a delayed composer candidate before reporting it missing", async () => {
    let attempts = 0;
    const locator = {
      count: async () => {
        attempts += 1;
        return attempts >= 3 ? 1 : 0;
      },
      isVisible: async () => true,
    };
    const page = {
      locator: (selector: string) => {
        expect(selector).toBe("[contenteditable='true']");
        return { last: () => locator };
      },
      waitForTimeout: async () => undefined,
    } as unknown as Pick<Page, "locator" | "waitForTimeout">;

    await expect(
      waitForFirstVisible(page, ["[contenteditable='true']"], { timeoutMs: 1_000, intervalMs: 0 }),
    ).resolves.toBe(locator);
    expect(attempts).toBe(3);
  });

  it("reports CDP endpoint setup action before long-lived attach", async () => {
    const adapter = new GeminiBrowserAdapter({
      env: { EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT: "http://127.0.0.1:9222" },
      fetchLike: async () => {
        throw new Error("ECONNREFUSED");
      },
    });

    await expect(adapter.status("C:/Extractum/gemini-browser/profile")).resolves.toMatchObject({
      status: "needs_manual_action",
      manual_action: "start_chrome_cdp",
      browser_profile_dir: "C:/Extractum/gemini-browser/profile",
      latest_message: "Chrome CDP endpoint is unavailable. Start Chrome with remote debugging enabled.",
    });
  });

  it("reports attached CDP session without a Gemini page distinctly", async () => {
    const adapter = new GeminiBrowserAdapter({
      env: { EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT: "http://127.0.0.1:9222" },
    });

    adapter.__setTestSession({
      type: "cdp_attach",
      browser: null,
      context: { pages: () => [] } as never,
      page: null,
    });

    await expect(adapter.status("C:/Extractum/gemini-browser/profile")).resolves.toMatchObject({
      status: "needs_manual_action",
      manual_action: "start_chrome_cdp",
      latest_message: "Chrome CDP attached, but no Gemini tab is available.",
    });
  });

  it("opens Gemini in an existing CDP context without creating a browser context", async () => {
    const page = {
      goto: vi.fn(async () => undefined),
      isClosed: () => false,
    };
    const context = {
      pages: () => [],
      newPage: vi.fn(async () => page),
    };
    const browser = {
      contexts: () => [context],
      newContext: vi.fn(),
    };
    const adapter = new GeminiBrowserAdapter({
      env: { EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT: "http://127.0.0.1:9222" },
      connectOverCdp: async () => browser as never,
    });

    await expect(adapter.openBrowser("C:/Extractum/gemini-browser/profile")).resolves.toMatchObject({
      status: "ready",
      latest_message: "Chrome CDP attached.",
    });
    expect(context.newPage).toHaveBeenCalledOnce();
    expect(browser.newContext).not.toHaveBeenCalled();
    expect(page.goto).toHaveBeenCalledWith("https://gemini.google.com/app", {
      waitUntil: "domcontentloaded",
    });
  });

  it("does not create a Gemini page from sendSingle in CDP attach-only mode", async () => {
    const context = {
      pages: () => [],
      newPage: vi.fn(),
    };
    const browser = {
      contexts: () => [context],
    };
    const adapter = new GeminiBrowserAdapter({
      env: { EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT: "http://127.0.0.1:9222" },
      connectOverCdp: async () => browser as never,
    });

    await expect(
      adapter.sendSingle({
        browserProfileDir: "C:/Extractum/gemini-browser/profile",
        artifactDir: "artifacts/gemini-browser-adapter-test/run-1",
        request: {
          run_id: "run-1",
          prompt: "hello",
          source: "settings_test",
          artifact_mode: "reduced",
        },
      }),
    ).resolves.toMatchObject({
      status: "needs_manual_action",
      manual_action: "start_chrome_cdp",
      message: "Open Gemini in the attached Chrome profile or use Open to create a Gemini tab.",
    });
    expect(context.newPage).not.toHaveBeenCalled();
  });

  it("preserves CDP attach setup errors from sendSingle", async () => {
    const adapter = new GeminiBrowserAdapter({
      env: { EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT: "http://127.0.0.1:9222" },
      connectOverCdp: async () => {
        throw new Error("ECONNREFUSED");
      },
    });

    await expect(
      adapter.sendSingle({
        browserProfileDir: "C:/Extractum/gemini-browser/profile",
        artifactDir: "artifacts/gemini-browser-adapter-test/run-1",
        request: {
          run_id: "run-1",
          prompt: "hello",
          source: "settings_test",
          artifact_mode: "reduced",
        },
      }),
    ).resolves.toMatchObject({
      status: "needs_manual_action",
      manual_action: "start_chrome_cdp",
      message: "Chrome CDP endpoint is unavailable. Start Chrome with remote debugging enabled.",
    });
  });

  it("maps CDP closed-target send failures to browser_crashed", async () => {
    const adapter = new GeminiBrowserAdapter({
      env: {},
    });
    const page = {
      isClosed: () => false,
      locator: () => {
        throw new Error("Target closed");
      },
      waitForTimeout: async () => undefined,
    };
    adapter.__setTestPage(page as never, "cdp_attach");

    await expect(
      adapter.sendSingle({
        browserProfileDir: "C:/Extractum/gemini-browser/profile",
        artifactDir: "artifacts/gemini-browser-adapter-test/run-1",
        request: {
          run_id: "run-1",
          prompt: "hello",
          source: "settings_test",
          artifact_mode: "reduced",
        },
      }),
    ).resolves.toMatchObject({
      status: "browser_crashed",
      manual_action: null,
      message: "Chrome CDP connection closed during the run.",
    });
  });

  it("maps an already closed attached CDP page before send to browser_crashed", async () => {
    const adapter = new GeminiBrowserAdapter({
      env: { EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT: "http://127.0.0.1:9222" },
    });
    const page = {
      isClosed: () => true,
    };
    adapter.__setTestPage(page as never, "cdp_attach");

    await expect(
      adapter.sendSingle({
        browserProfileDir: "C:/Extractum/gemini-browser/profile",
        artifactDir: "artifacts/gemini-browser-adapter-test/run-1",
        request: {
          run_id: "run-1",
          prompt: "hello",
          source: "settings_test",
          artifact_mode: "reduced",
        },
      }),
    ).resolves.toMatchObject({
      status: "browser_crashed",
      manual_action: null,
      message: "Chrome CDP page closed before the run could send.",
    });
  });
});
