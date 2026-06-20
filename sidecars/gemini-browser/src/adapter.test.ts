import { describe, expect, it } from "vitest";
import type { Page } from "@playwright/test";
import { waitForFirstVisible } from "./adapter.js";
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
});
