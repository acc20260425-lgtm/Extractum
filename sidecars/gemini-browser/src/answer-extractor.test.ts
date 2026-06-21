import { chromium, type Browser, type Page } from "@playwright/test";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import {
  ANSWER_POLL_INTERVAL_MS,
  ANSWER_STABLE_MS,
  captureAnswerBaseline,
  captureAnswerExtractionSnapshot,
  pollAnswerSnapshotsUntilComplete,
} from "./answer-extractor.js";

let browser: Browser;
let page: Page;

beforeAll(async () => {
  browser = await chromium.launch({ headless: true });
});

afterAll(async () => {
  await browser.close();
});

async function loadFixture(html: string) {
  page = await browser.newPage();
  await page.setContent(html, { waitUntil: "domcontentloaded" });
}

afterAll(async () => {
  await page?.close().catch(() => undefined);
});

function shell(body: string) {
  return `
    <main>
      <section data-turn="previous" data-response-index="0">
        <message-content>Previous answer that must remain baseline.</message-content>
      </section>
      <form aria-label="Prompt composer">
        <textarea>What is happening in football?</textarea>
        <button aria-label="send">Send</button>
      </form>
      ${body}
    </main>
  `;
}

describe("Gemini answer extractor", () => {
  it("groups split message-content nodes into one assistant answer", async () => {
    await loadFixture(shell(""));
    const baseline = await captureAnswerBaseline(page, "What is happening in football?");
    await page.evaluate(() => {
      document.querySelector("main")?.insertAdjacentHTML(
        "beforeend",
        `<section data-turn="assistant" data-response-index="1">
          <message-content>First paragraph.</message-content>
          <message-content><ul><li>Point A.</li><li>Point B.</li></ul></message-content>
        </section>`,
      );
    });

    const snapshot = await captureAnswerExtractionSnapshot(page, {
      prompt: "What is happening in football?",
      baseline,
      elapsedMs: 0,
      busyVisible: false,
    });

    expect(snapshot.grouped_candidates).toHaveLength(1);
    expect(snapshot.grouped_candidates[0]).toMatchObject({
      grouping: "assistant_turn",
      text_length: "First paragraph.\nPoint A.\nPoint B.".length,
    });
    expect(snapshot.selected_candidate?.text).toBe("First paragraph.\nPoint A.\nPoint B.");
    expect(snapshot.selected_candidate_signature).toBeTruthy();
  });

  it("does not select a broad page container with composer controls", async () => {
    await loadFixture(shell(""));
    const baseline = await captureAnswerBaseline(page, "What is happening in football?");
    await page.evaluate(() => {
      document.querySelector("main")?.insertAdjacentHTML(
        "beforeend",
        `<section data-turn="assistant" data-response-index="1">
          <message-content>Assistant answer.</message-content>
        </section>
        <section data-noisy="true">
          <message-content>Assistant answer plus composer noise</message-content>
          <textarea>composer text</textarea>
        </section>`,
      );
    });

    const snapshot = await captureAnswerExtractionSnapshot(page, {
      prompt: "What is happening in football?",
      baseline,
      elapsedMs: 0,
      busyVisible: false,
    });

    expect(snapshot.selected_candidate?.text).toBe("Assistant answer.");
    expect(snapshot.rejected_candidates.some((candidate) => candidate.reasons.includes("composer"))).toBe(true);
  });

  it("uses structural baseline so repeated answer text is not dropped", async () => {
    await loadFixture(shell(`
      <section data-turn="assistant" data-response-index="1">
        <message-content>Same answer.</message-content>
      </section>
    `));

    const baseline = await captureAnswerBaseline(page, "repeat");
    await page.evaluate(() => {
      document.querySelector("main")?.insertAdjacentHTML(
        "beforeend",
        "<section data-turn='assistant' data-response-index='2'><message-content>Same answer.</message-content></section>",
      );
    });

    const snapshot = await captureAnswerExtractionSnapshot(page, {
      prompt: "repeat",
      baseline,
      elapsedMs: 0,
      busyVisible: false,
    });

    expect(snapshot.selected_candidate?.group_id).toContain("2");
    expect(snapshot.selected_candidate?.text).toBe("Same answer.");
  });

  it("returns stable after the numeric quiet window rather than hard timeout", async () => {
    let now = 0;
    const selected = {
      group_id: "response:1",
      selector: "message-content",
      grouping: "assistant_turn" as const,
      text: "Complete answer.",
      text_length: "Complete answer.".length,
      block_lengths: ["Complete answer.".length],
      block_count: 1,
      group_order: 1,
      score: 120,
      signature: "message-content|response:1|1|assistant_turn|1|16",
      reject_reasons: [],
    };

    await expect(
      pollAnswerSnapshotsUntilComplete({
        readSnapshot: async (elapsedMs) => ({
          elapsed_ms: elapsedMs,
          busy_visible: false,
          raw_candidate_count: 1,
          grouped_candidates: [selected],
          rejected_candidates: [],
          selected_candidate_id: selected.group_id,
          selected_candidate_signature: selected.signature,
          selected_candidate: selected,
          selection_reason: "highest_score",
        }),
        answerStableMs: ANSWER_STABLE_MS,
        pollIntervalMs: ANSWER_POLL_INTERVAL_MS,
        minStablePollsAfterSignatureChange: 3,
        isBusyVisible: async () => false,
        now: () => now,
        waitForTimeout: async (ms) => {
          now += ms;
        },
      }),
    ).resolves.toMatchObject({
      text: "Complete answer.",
      completionReason: "stable",
      debug: {
        stable_poll_count_after_last_candidate_change: 3,
      },
    });
  });

  it("returns timeout_latest with debug when candidate never stabilizes", async () => {
    let now = 0;
    let counter = 0;

    await expect(
      pollAnswerSnapshotsUntilComplete({
        readSnapshot: async (elapsedMs) => {
          counter += 1;
          const text = `partial ${counter}`;
          const selected = {
            group_id: "response:1",
            selector: "message-content",
            grouping: "assistant_turn" as const,
            text,
            text_length: text.length,
            block_lengths: [text.length],
            block_count: 1,
            group_order: 1,
            score: 90,
            signature: `message-content|response:1|1|assistant_turn|1|${text.length}|${counter}`,
            reject_reasons: [],
          };
          return {
            elapsed_ms: elapsedMs,
            busy_visible: false,
            raw_candidate_count: 1,
            grouped_candidates: [selected],
            rejected_candidates: [],
            selected_candidate_id: selected.group_id,
            selected_candidate_signature: selected.signature,
            selected_candidate: selected,
            selection_reason: "highest_score",
          };
        },
        answerTimeoutMs: 2_000,
        answerStableMs: ANSWER_STABLE_MS,
        pollIntervalMs: ANSWER_POLL_INTERVAL_MS,
        minStablePollsAfterSignatureChange: 3,
        isBusyVisible: async () => false,
        now: () => now,
        waitForTimeout: async (ms) => {
          now += ms;
        },
      }),
    ).resolves.toMatchObject({
      completionReason: "timeout_latest",
      debug: {
        larger_valid_candidate_available: false,
      },
    });
  });

  it("returns missing result instead of null when no answer candidate appears", async () => {
    let now = 0;

    await expect(
      pollAnswerSnapshotsUntilComplete({
        readSnapshot: async (elapsedMs) => ({
          elapsed_ms: elapsedMs,
          busy_visible: false,
          raw_candidate_count: 0,
          grouped_candidates: [],
          rejected_candidates: [],
          selected_candidate_id: null,
          selected_candidate_signature: null,
          selected_candidate: null,
          selection_reason: null,
        }),
        answerTimeoutMs: 1_000,
        answerStableMs: ANSWER_STABLE_MS,
        pollIntervalMs: ANSWER_POLL_INTERVAL_MS,
        minStablePollsAfterSignatureChange: 3,
        isBusyVisible: async () => false,
        now: () => now,
        waitForTimeout: async (ms) => {
          now += ms;
        },
      }),
    ).resolves.toMatchObject({
      text: null,
      selector: null,
      completionReason: "missing",
      debug: {
        raw_candidate_count: 0,
        grouped_candidate_count: 0,
      },
    });
  });
});
