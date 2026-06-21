# Gemini Browser Answer Extraction Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Gemini Browser Provider answer extraction return the complete visible assistant response, expose extraction diagnostics, and prevent partial browser text from silently entering prompt-pack automation.

**Architecture:** Move answer extraction out of `adapter.ts` into a focused sidecar extractor module with structural baseline, grouped candidate scoring, and explicit completion semantics. Keep sidecar extraction debug compact and privacy-safe, mirror optional DTO fields in Rust/frontend, and make downstream prompt-pack completion reject `ok + timeout_latest` as partial-risk.

**Tech Stack:** TypeScript sidecar, Playwright DOM fixture tests, Vitest, Rust/Tauri DTOs and prompt-pack stage, Svelte Settings UI, existing Browser Provider run-log artifacts.

---

## Scope And File Map

Create:

- `sidecars/gemini-browser/src/answer-extractor.ts`
  Owns extraction constants, candidate/group types, structural baseline, DOM grouping, scoring, stable/timeout polling, compact debug conversion, and reduced artifact payload generation.

- `sidecars/gemini-browser/src/answer-extractor.test.ts`
  Unit and static Playwright DOM fixture coverage for split answers, slow growth, structural baseline, one-turn grouping, broad container rejection, and `timeout_latest`.

Modify:

- `sidecars/gemini-browser/src/protocol.ts`
  Adds optional `artifacts.answer_extraction`, nested optional `debug_summary.extraction`, and reject reason/debug DTO types.

- `sidecars/gemini-browser/src/adapter.ts`
  Uses the extractor instead of `captureAnswerState()`, writes reduced extraction artifacts, fills nested extraction debug, and keeps artifact write errors non-fatal.

- `sidecars/gemini-browser/src/adapter.test.ts`
  Updates existing expectations for nested extraction debug and adds adapter-level checks for artifact wiring and partial-risk result behavior.

- `src-tauri/src/gemini_browser/types.rs`
  Mirrors optional extraction DTOs and artifact reference field.

- `src-tauri/src/gemini_browser/run_log.rs`
  Extends round-trip tests for optional extraction debug and `answer_extraction` artifact ref.

- `src-tauri/src/prompt_packs/gemini_browser_stage.rs`
  Rejects `Ok` browser results whose debug completion reason is `TimeoutLatest`.

- `src/lib/types/gemini-browser.ts`
  Mirrors optional extraction DTOs and artifact reference field in frontend types.

- `src/lib/gemini-browser-run-inspector.ts`
  Adds extraction availability, partial-risk helpers, copy diagnostics fields, and sanitizer coverage.

- `src/lib/gemini-browser-run-inspector.test.ts`
  Covers extraction diagnostics, partial-risk flags, `answer_extraction` availability, and privacy constraints.

- `src/lib/components/settings/gemini-browser-provider-panel.svelte`
  Shows extraction facts and visibly marks `timeout_latest` partial-risk runs.

- `src/lib/gemini-browser-provider-panel.test.ts`
  Source-contract coverage for the new inspector labels and partial-risk copy.

- `docs/browser-providers-llm-troubleshooting.md`
  Documents how to debug answer extraction using the new inspector fields and artifact.

Execution setup:

- Create a branch before implementation: `git switch -c gemini-browser-answer-extraction-hardening`.
- After each task: mark completed checkboxes in this plan, run the task verification command, then commit.

---

### Task 1: Sidecar Extraction Core And DOM Fixture Tests

**Files:**

- Create: `sidecars/gemini-browser/src/answer-extractor.ts`
- Create: `sidecars/gemini-browser/src/answer-extractor.test.ts`
- Modify: `sidecars/gemini-browser/src/protocol.ts`

- [x] **Step 1: Add failing extractor tests**

Create `sidecars/gemini-browser/src/answer-extractor.test.ts` with these tests:

```ts
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
```

- [x] **Step 2: Run extractor tests and verify they fail**

Run:

```powershell
npm.cmd run test -- --run sidecars/gemini-browser/src/answer-extractor.test.ts
```

Expected: FAIL because `sidecars/gemini-browser/src/answer-extractor.ts` does not exist.

- [x] **Step 3: Add protocol DTO types**

Modify `sidecars/gemini-browser/src/protocol.ts` by adding these types after `GeminiBrowserAnswerCompletionReason`:

```ts
export type GeminiBrowserCandidateRejectReason =
  | "baseline"
  | "composer"
  | "prompt_container"
  | "navigation"
  | "account_or_login"
  | "controls"
  | "multi_turn"
  | "not_visible"
  | "empty"
  | "lower_score";

export type GeminiBrowserAnswerGrouping = "assistant_turn" | "single_node" | "unknown";

export interface GeminiBrowserAnswerExtractionDebug {
  raw_candidate_count: number;
  grouped_candidate_count: number;
  selected_candidate_length: number;
  returned_text_length: number;
  selected_grouping: GeminiBrowserAnswerGrouping;
  selected_candidate_rank: number | null;
  selected_score: number | null;
  largest_candidate_length: number;
  larger_valid_candidate_available: boolean;
  larger_rejected_candidate_count: number;
  larger_rejected_reasons: GeminiBrowserCandidateRejectReason[];
  top_candidate_lengths: number[];
  busy_visible_at_completion: boolean;
  last_growth_elapsed_ms: number | null;
  candidate_signature_changed_count: number;
  stable_poll_count_after_last_candidate_change: number;
}
```

Extend `GeminiBrowserRunDebugSummary`:

```ts
  extraction?: GeminiBrowserAnswerExtractionDebug | null;
```

Extend `GeminiBrowserRunResult["artifacts"]`:

```ts
    answer_extraction?: string | null;
```

- [x] **Step 4: Implement extraction core**

Create `sidecars/gemini-browser/src/answer-extractor.ts` with these public exports and behavior:

```ts
import type { Page } from "@playwright/test";
import { answerCandidates } from "./dom-contract.js";
import type {
  GeminiBrowserAnswerCompletionReason,
  GeminiBrowserAnswerExtractionDebug,
  GeminiBrowserAnswerGrouping,
  GeminiBrowserCandidateRejectReason,
} from "./protocol.js";

export const ANSWER_POLL_INTERVAL_MS = 500;
export const ANSWER_STABLE_MS = 8_000;
export const MIN_STABLE_POLLS_AFTER_SIGNATURE_CHANGE = 3;
export const MAX_ANSWER_TIMEOUT_MS = 120_000;

export interface AnswerExtractionBaseline {
  groups: Array<{
    group_id: string;
    group_order: number;
    selector: string;
    text_length: number;
    block_lengths: number[];
  }>;
  highest_group_order: number;
}

export interface AnswerCandidateSummary {
  group_id: string;
  selector: string;
  grouping: GeminiBrowserAnswerGrouping;
  text: string;
  text_length: number;
  block_lengths: number[];
  block_count: number;
  group_order: number;
  score: number;
  signature: string;
  reject_reasons: GeminiBrowserCandidateRejectReason[];
}

export interface RejectedAnswerCandidate {
  selector: string;
  text_length: number;
  reasons: GeminiBrowserCandidateRejectReason[];
}

export interface AnswerExtractionSnapshot {
  elapsed_ms: number;
  busy_visible: boolean;
  raw_candidate_count: number;
  grouped_candidates: AnswerCandidateSummary[];
  rejected_candidates: RejectedAnswerCandidate[];
  selected_candidate_id: string | null;
  selected_candidate_signature: string | null;
  selected_candidate: AnswerCandidateSummary | null;
  selection_reason: string | null;
}

export interface AnswerExtractionResult {
  text: string | null;
  selector: string | null;
  waitedMs: number;
  completionReason: GeminiBrowserAnswerCompletionReason;
  debug: GeminiBrowserAnswerExtractionDebug;
  artifact: AnswerExtractionArtifactPayload;
}

export interface AnswerExtractionArtifactPayload {
  completion_reason: GeminiBrowserAnswerCompletionReason;
  raw_candidate_count: number;
  grouped_candidate_count: number;
  selected_candidate: {
    selector: string | null;
    grouping: GeminiBrowserAnswerGrouping;
    text_length: number;
    score: number | null;
    rank: number | null;
  };
  top_candidates: Array<{
    selector: string;
    grouping: GeminiBrowserAnswerGrouping;
    text_length: number;
    block_lengths: number[];
    score: number;
  }>;
  rejected: Array<{
    selector: string;
    text_length: number;
    reasons: GeminiBrowserCandidateRejectReason[];
  }>;
}

export class AnswerExtractionError extends Error {
  constructor(
    message: string,
    readonly artifact: AnswerExtractionArtifactPayload,
    cause: unknown,
  ) {
    super(message, { cause });
  }
}
```

Implementation requirements for the same file:

- `captureAnswerBaseline(page, prompt)` uses the same DOM grouping code as snapshots, but runs in `mode: "baseline"`: collect structural groups without post-submit/new-candidate filtering and without rejecting previous assistant turns merely because they are not new. It returns `groups` plus `highest_group_order`.
- `captureAnswerExtractionSnapshot(page, input)` uses `page.evaluate()` to inspect the real DOM. It must:
  - query selectors from `answerCandidates`;
  - climb at most six ancestors;
  - reject composer/account/navigation/control containers;
  - split multi-turn ancestors by `data-response-index` or role-like message groups;
  - aggregate visible `message-content` descendants in DOM order;
  - use structural baseline first, then relative order after `highest_group_order`;
  - avoid copying prompt/answer text into diagnostics, except the selected candidate `text` returned internally for the final result.
- `signature` is internal and composed from selector, group id, group order, grouping mode, block count, block lengths, and total length.
- `buildExtractionDebug(snapshot, returnedTextLength, completionReason, counters)` returns all fields in `GeminiBrowserAnswerExtractionDebug`.
- internal helper `toAnswerExtractionArtifact(resultOrSnapshot)` returns `AnswerExtractionArtifactPayload` with lengths/score facts only. It does not need to be exported unless tests need direct unit coverage.
- internal helper `emptyAnswerExtractionArtifact(completionReason)` returns a safe empty payload for extraction-started failures before the first usable snapshot. `AnswerExtractionError.artifact` is always non-null; use this empty fallback when constructing `AnswerExtractionError` would otherwise have no payload.
- `pollAnswerSnapshotsUntilComplete(options)` is the pure polling engine used by tests. It accepts `readSnapshot(elapsedMs)`, `now()`, `waitForTimeout(ms)`, `answerStableMs`, `answerTimeoutMs`, `pollIntervalMs`, `minStablePollsAfterSignatureChange`, and `isBusyVisible()`.
- `pollAnswerUntilComplete(page, options)` implements:
  - a thin wrapper that calls `captureAnswerExtractionSnapshot()` and delegates to `pollAnswerSnapshotsUntilComplete()`;
  - earliest stable return formula from the spec;
  - timeout at `MAX_ANSWER_TIMEOUT_MS` unless `answerTimeoutMs` is passed in tests;
  - `timeout_latest` when text exists but stability is not proven;
  - `missing` with `text: null` when no valid answer exists.
  - never returns `null`; tests must assert the missing-answer case is an `AnswerExtractionResult` with `text: null`;
  - selector/evaluation errors that are not closed-target errors should be converted into `missing` with an artifact carrying rejection/error facts;
  - closed-target or unexpected fatal extraction errors should throw `AnswerExtractionError` carrying the latest reduced artifact payload when available, or `emptyAnswerExtractionArtifact("missing")` when extraction failed before the first snapshot, so adapter catch paths can still write `answer-extraction.json`.

- [x] **Step 5: Run extractor tests**

Run:

```powershell
npm.cmd run test -- --run sidecars/gemini-browser/src/answer-extractor.test.ts
```

Expected: PASS.

- [x] **Step 6: Typecheck sidecar**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar:typecheck
```

Expected: PASS.

- [x] **Step 7: Mark task complete and commit**

Update this task's checkboxes to `[x]`, then run:

```powershell
git add docs/superpowers/plans/2026-06-21-gemini-browser-answer-extraction-hardening-plan.md sidecars/gemini-browser/src/protocol.ts sidecars/gemini-browser/src/answer-extractor.ts sidecars/gemini-browser/src/answer-extractor.test.ts
git commit -m "feat: add Gemini answer extraction core"
```

---

### Task 2: Adapter Integration, Reduced Extraction Artifact, And Partial-Risk Results

**Files:**

- Modify: `sidecars/gemini-browser/src/adapter.ts`
- Modify: `sidecars/gemini-browser/src/adapter.test.ts`
- Modify: `sidecars/gemini-browser/src/protocol.ts`

- [x] **Step 1: Add failing adapter tests for extraction integration**

Append these tests to `sidecars/gemini-browser/src/adapter.test.ts`:

```ts
it("adds extraction debug to stable answer results", async () => {
  vi.useFakeTimers();
  vi.setSystemTime(new Date("2026-06-21T00:00:00Z"));
  try {
    const finalAnswer = "Complete grouped answer.";
    const page = pageWithSendFlow();
    const adapter = new GeminiBrowserAdapter({
      env: {},
      answerExtractor: {
        captureBaseline: async () => ({ groups: [], highest_group_order: -1 }),
        pollUntilComplete: async () => ({
          text: finalAnswer,
          selector: "message-content",
          waitedMs: 8_500,
          completionReason: "stable",
          debug: {
            raw_candidate_count: 2,
            grouped_candidate_count: 1,
            selected_candidate_length: finalAnswer.length,
            returned_text_length: finalAnswer.length,
            selected_grouping: "assistant_turn",
            selected_candidate_rank: 1,
            selected_score: 120,
            largest_candidate_length: finalAnswer.length,
            larger_valid_candidate_available: false,
            larger_rejected_candidate_count: 0,
            larger_rejected_reasons: [],
            top_candidate_lengths: [finalAnswer.length],
            busy_visible_at_completion: false,
            last_growth_elapsed_ms: 8_000,
            candidate_signature_changed_count: 1,
            stable_poll_count_after_last_candidate_change: 3,
          },
          artifact: {
            completion_reason: "stable",
            raw_candidate_count: 2,
            grouped_candidate_count: 1,
            selected_candidate: {
              selector: "message-content",
              grouping: "assistant_turn",
              text_length: finalAnswer.length,
              score: 120,
              rank: 1,
            },
            top_candidates: [],
            rejected: [],
          },
        }),
      },
    });
    adapter.__setTestPage(page as never);

    const result = await adapter.sendSingle({
      browserProfileDir: "C:/Extractum/gemini-browser/profile",
      artifactDir: "artifacts/gemini-browser-adapter-test/run-extraction-stable",
      request: {
        run_id: "run-extraction-stable",
        prompt: "hello",
        source: "settings_test",
        artifact_mode: "reduced",
      },
    });

    expect(result).toMatchObject({
      status: "ok",
      text: finalAnswer,
      debug_summary: {
        answer_completion_reason: "stable",
        final_text_length: finalAnswer.length,
        extraction: {
          selected_candidate_length: finalAnswer.length,
          returned_text_length: finalAnswer.length,
          selected_grouping: "assistant_turn",
          larger_valid_candidate_available: false,
        },
      },
    });
  } finally {
    vi.useRealTimers();
  }
});

it("writes a reduced extraction artifact for ok timeout_latest without changing status", async () => {
  vi.useFakeTimers();
  vi.setSystemTime(new Date("2026-06-21T00:00:00Z"));
  try {
    const artifactDir = "artifacts/gemini-browser-adapter-test/run-timeout-latest-artifact";
    const page = pageWithSendFlow();
    const adapter = new GeminiBrowserAdapter({
      env: {},
      answerExtractor: {
        captureBaseline: async () => ({ groups: [], highest_group_order: -1 }),
        pollUntilComplete: async () => ({
          text: "partial answer",
          selector: "message-content",
          waitedMs: 120_000,
          completionReason: "timeout_latest",
          debug: {
            raw_candidate_count: 1,
            grouped_candidate_count: 1,
            selected_candidate_length: 14,
            returned_text_length: 14,
            selected_grouping: "assistant_turn",
            selected_candidate_rank: 1,
            selected_score: 90,
            largest_candidate_length: 14,
            larger_valid_candidate_available: false,
            larger_rejected_candidate_count: 0,
            larger_rejected_reasons: [],
            top_candidate_lengths: [14],
            busy_visible_at_completion: false,
            last_growth_elapsed_ms: 500,
            candidate_signature_changed_count: 50,
            stable_poll_count_after_last_candidate_change: 0,
          },
          artifact: {
            completion_reason: "timeout_latest",
            raw_candidate_count: 1,
            grouped_candidate_count: 1,
            selected_candidate: {
              selector: "message-content",
              grouping: "assistant_turn",
              text_length: 14,
              score: 90,
              rank: 1,
            },
            top_candidates: [{ selector: "message-content", grouping: "assistant_turn", text_length: 14, block_lengths: [14], score: 90 }],
            rejected: [],
          },
        }),
      },
    });
    adapter.__setTestPage(page as never);

    const result = await adapter.sendSingle({
      browserProfileDir: "C:/Extractum/gemini-browser/profile",
      artifactDir,
      request: {
        run_id: "run-timeout-latest-artifact",
        prompt: "slow prompt",
        source: "settings_test",
        artifact_mode: "reduced",
      },
    });

    expect(result.status).toBe("ok");
    expect(result.debug_summary?.answer_completion_reason).toBe("timeout_latest");
    expect(result.artifacts.answer_extraction).toMatch(/answer-extraction\.json$/);
    expect(result.artifacts.artifact_write_error).toBeNull();
  } finally {
    vi.useRealTimers();
  }
});

it("writes a reduced extraction artifact for missing answer timeout failures", async () => {
  const page = pageWithSendFlow();
  const adapter = new GeminiBrowserAdapter({
    env: {},
    answerExtractor: {
      captureBaseline: async () => ({ groups: [], highest_group_order: -1 }),
      pollUntilComplete: async () => ({
        text: null,
        selector: null,
        waitedMs: 120_000,
        completionReason: "missing",
        debug: {
          raw_candidate_count: 0,
          grouped_candidate_count: 0,
          selected_candidate_length: 0,
          returned_text_length: 0,
          selected_grouping: "unknown",
          selected_candidate_rank: null,
          selected_score: null,
          largest_candidate_length: 0,
          larger_valid_candidate_available: false,
          larger_rejected_candidate_count: 0,
          larger_rejected_reasons: [],
          top_candidate_lengths: [],
          busy_visible_at_completion: false,
          last_growth_elapsed_ms: null,
          candidate_signature_changed_count: 0,
          stable_poll_count_after_last_candidate_change: 0,
        },
        artifact: {
          completion_reason: "missing",
          raw_candidate_count: 0,
          grouped_candidate_count: 0,
          selected_candidate: {
            selector: null,
            grouping: "unknown",
            text_length: 0,
            score: null,
            rank: null,
          },
          top_candidates: [],
          rejected: [],
        },
      }),
    },
  });
  adapter.__setTestPage(page as never);

  const result = await adapter.sendSingle({
    browserProfileDir: "C:/Extractum/gemini-browser/profile",
    artifactDir: "artifacts/gemini-browser-adapter-test/run-missing-answer-artifact",
    request: {
      run_id: "run-missing-answer-artifact",
      prompt: "missing answer",
      source: "settings_test",
      artifact_mode: "reduced",
    },
  });

  expect(result.status).toBe("timeout");
  expect(result.artifacts.answer_extraction).toMatch(/answer-extraction\.json$/);
  expect(result.debug_summary?.extraction?.raw_candidate_count).toBe(0);
});

it("keeps timeout_latest ok when answer extraction artifact write fails", async () => {
  const page = pageWithSendFlow();
  const adapter = new GeminiBrowserAdapter({
    env: {},
    writeAnswerExtractionArtifact: async () => ({ path: null, error: "disk full" }),
    answerExtractor: {
      captureBaseline: async () => ({ groups: [], highest_group_order: -1 }),
      pollUntilComplete: async () => ({
        text: "partial answer",
        selector: "message-content",
        waitedMs: 120_000,
        completionReason: "timeout_latest",
        debug: {
          raw_candidate_count: 1,
          grouped_candidate_count: 1,
          selected_candidate_length: 14,
          returned_text_length: 14,
          selected_grouping: "assistant_turn",
          selected_candidate_rank: 1,
          selected_score: 90,
          largest_candidate_length: 14,
          larger_valid_candidate_available: false,
          larger_rejected_candidate_count: 0,
          larger_rejected_reasons: [],
          top_candidate_lengths: [14],
          busy_visible_at_completion: false,
          last_growth_elapsed_ms: 500,
          candidate_signature_changed_count: 50,
          stable_poll_count_after_last_candidate_change: 0,
        },
        artifact: {
          completion_reason: "timeout_latest",
          raw_candidate_count: 1,
          grouped_candidate_count: 1,
          selected_candidate: {
            selector: "message-content",
            grouping: "assistant_turn",
            text_length: 14,
            score: 90,
            rank: 1,
          },
          top_candidates: [],
          rejected: [],
        },
      }),
    },
  });
  adapter.__setTestPage(page as never);

  const result = await adapter.sendSingle({
    browserProfileDir: "C:/Extractum/gemini-browser/profile",
    artifactDir: "artifacts/gemini-browser-adapter-test/unwritable",
    request: {
      run_id: "run-timeout-latest-write-fail",
      prompt: "slow prompt",
      source: "settings_test",
      artifact_mode: "reduced",
    },
  });

  expect(result.status).toBe("ok");
  expect(result.debug_summary?.answer_completion_reason).toBe("timeout_latest");
  expect(result.artifacts.answer_extraction).toBeNull();
  expect(result.artifacts.artifact_write_error).toContain("disk full");
});
```

If `adapter.test.ts` does not have a reusable `pageWithSendFlow()` helper, add one near the other test helpers:

```ts
function pageWithSendFlow() {
  const composer = {
    count: async () => 1,
    nth: () => composer,
    isVisible: async () => true,
    fill: vi.fn(async () => undefined),
  };
  const send = {
    count: async () => 1,
    nth: () => send,
    isVisible: async () => true,
    click: vi.fn(async () => undefined),
  };
  const empty = {
    count: async () => 0,
    nth: () => empty,
    isVisible: async () => false,
    allTextContents: async () => [],
  };
  return {
    isClosed: () => false,
    locator: (selector: string) => {
      if (selector === "rich-textarea textarea") return composer;
      if (selector === "button[aria-label*='send' i]") return send;
      return empty;
    },
    waitForTimeout: async (ms: number) => {
      vi.advanceTimersByTime(ms);
    },
  };
}
```

- [x] **Step 2: Run adapter tests and verify they fail**

Run:

```powershell
npm.cmd run test -- --run sidecars/gemini-browser/src/adapter.test.ts
```

Expected: FAIL because `debug_summary.extraction` and `artifacts.answer_extraction` are not populated.

- [x] **Step 3: Integrate extractor in adapter**

Modify `sidecars/gemini-browser/src/adapter.ts`:

- add an adapter test seam:

```ts
type AnswerExtractorPort = {
  captureBaseline: typeof captureAnswerBaseline;
  pollUntilComplete: typeof pollAnswerUntilComplete;
};

type WriteAnswerExtractionArtifact = (input: {
  artifactDir: string;
  payload: AnswerExtractionArtifactPayload;
}) => Promise<{ path: string | null; error: string | null }>;

interface GeminiBrowserAdapterOptions {
  env?: Record<string, string | undefined>;
  fetchLike?: FetchLike;
  connectOverCdp?: ConnectOverCdp;
  answerExtractor?: AnswerExtractorPort;
  writeAnswerExtractionArtifact?: WriteAnswerExtractionArtifact;
}
```

- add a private field and constructor assignment:

```ts
private readonly answerExtractor: AnswerExtractorPort;
private readonly writeAnswerExtractionArtifact: WriteAnswerExtractionArtifact;
```

```ts
this.answerExtractor = options.answerExtractor ?? {
  captureBaseline: captureAnswerBaseline,
  pollUntilComplete: pollAnswerUntilComplete,
};
this.writeAnswerExtractionArtifact =
  options.writeAnswerExtractionArtifact ?? writeAnswerExtractionArtifact;
```

- import extractor APIs:

```ts
import {
  AnswerExtractionError,
  ANSWER_STABLE_MS,
  MAX_ANSWER_TIMEOUT_MS,
  captureAnswerBaseline,
  pollAnswerUntilComplete,
  type AnswerExtractionArtifactPayload,
} from "./answer-extractor.js";
```

- replace `const answerBaseline = await captureAnswerState(page, input.request.prompt);` with:

```ts
const answerBaseline = await this.answerExtractor.captureBaseline(page, input.request.prompt);
```

- replace `const answer = await waitForAnswerText(...)` with:

```ts
const answer = await this.answerExtractor.pollUntilComplete(page, {
  prompt: input.request.prompt,
  baseline: answerBaseline,
  isBusyVisible: () => hasVisibleLocator(page, generationBusySelectors),
  now: () => Date.now(),
  waitForTimeout: (ms) => page.waitForTimeout(ms),
});
```

- populate summary:

```ts
debugSummary = {
  ...debugSummary,
  answer_found: Boolean(answer.text),
  answer_selector: answer.selector,
  waited_for_answer_ms: answer.waitedMs,
  answer_stable_ms: ANSWER_STABLE_MS,
  answer_completion_reason: answer.completionReason,
  final_text_length: answer.text ? answer.text.length : 0,
  extraction: answer.debug,
};
```

- include `answer_extraction: null` in `emptyArtifacts()` and every artifact literal.
- replace the old `if (!answer)` branch with:

```ts
if (!answer.text) {
  const extractionArtifact = await this.writeAnswerExtractionArtifact({
    artifactDir: input.artifactDir,
    payload: answer.artifact,
  });
  return this.failure(
    page,
    input.request,
    input.artifactDir,
    "timeout",
    "Answer did not appear before timeout.",
    start,
    markErrorStage(debugSummary, "answer"),
    extractionArtifact,
  );
}
```

Update `failure(...)` to accept an optional pre-written extraction artifact:

```ts
private async failure(
  page: Page,
  request: GeminiBrowserRunRequest,
  artifactDir: string,
  status: GeminiBrowserRunResult["status"],
  message: string,
  start: number,
  debugSummary: GeminiBrowserRunDebugSummary,
  extractionArtifact: { path: string | null; error: string | null } = { path: null, error: null },
): Promise<GeminiBrowserRunResult>
```

Inside `failure(...)`, merge `extractionArtifact.path` into `artifacts.answer_extraction` and combine `artifact_write_error` without throwing. If both failure artifact capture and extraction artifact write report errors, join them with `"; "`.

- [x] **Step 4: Add non-fatal reduced extraction artifact writer**

In `sidecars/gemini-browser/src/adapter.ts`, add:

```ts
async function writeAnswerExtractionArtifact(input: {
  artifactDir: string;
  payload: AnswerExtractionArtifactPayload;
}): Promise<{ path: string | null; error: string | null }> {
  const fs = await import("node:fs/promises");
  const path = await import("node:path");
  const artifactPath = path.join(input.artifactDir, "answer-extraction.json");
  try {
    await fs.mkdir(input.artifactDir, { recursive: true });
    await fs.writeFile(artifactPath, JSON.stringify(input.payload, null, 2), "utf8");
    return { path: artifactPath, error: null };
  } catch (error) {
    return { path: null, error: String(error) };
  }
}
```

When `answer.completionReason !== "stable"`, call this writer and merge the returned path/error into artifacts:

```ts
const extractionArtifact =
  answer.completionReason !== "stable"
    ? await this.writeAnswerExtractionArtifact({
        artifactDir: input.artifactDir,
        payload: answer.artifact,
      })
    : { path: null, error: null };

artifacts: {
  run_dir: input.artifactDir,
  html: null,
  screenshot: null,
  telemetry: null,
  answer_extraction: extractionArtifact.path,
  artifact_write_error: mergeArtifactWriteErrors(null, extractionArtifact.error),
},
```

Do not throw when writing `answer-extraction.json` fails.

Add a small merge helper so future telemetry/html artifact write errors are preserved instead of overwritten:

```ts
function mergeArtifactWriteErrors(...errors: Array<string | null | undefined>): string | null {
  const present = errors.filter((error): error is string => Boolean(error));
  return present.length > 0 ? present.join("; ") : null;
}
```

If `answer.text` is null, return the existing timeout failure shape, but pass along `debugSummary.extraction` and the extraction artifact path/error in `artifacts`.

For thrown failures or `browser_crashed` after extraction started, keep a local `latestExtractionArtifactPayload: AnswerExtractionArtifactPayload | null` in `sendSingle()`. Set it as soon as the extractor returns a payload. If the caught error is `AnswerExtractionError`, use `error.artifact` as the latest payload; the extractor must provide `emptyAnswerExtractionArtifact("missing")` when it has no real snapshot yet. In the catch branch, if a payload exists, call `this.writeAnswerExtractionArtifact(...)` before `failure(...)` and pass the returned `{ path, error }` into `failure(...)`. If no extraction payload exists because setup/composer/send failed before answer polling, keep `answer_extraction: null`.

- [x] **Step 5: Remove old answer polling helpers**

Delete old local `AnswerEntry`, `AnswerState`, `AnswerResult`, `waitForAnswerText()`, `captureAnswerState()`, and `bestNewAnswerText()` from `adapter.ts` once the extractor owns that behavior.

- [x] **Step 6: Run sidecar tests and typecheck**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar
```

Expected: PASS.

- [x] **Step 7: Mark task complete and commit**

Update this task's checkboxes to `[x]`, then run:

```powershell
git add docs/superpowers/plans/2026-06-21-gemini-browser-answer-extraction-hardening-plan.md sidecars/gemini-browser/src/adapter.ts sidecars/gemini-browser/src/adapter.test.ts sidecars/gemini-browser/src/protocol.ts sidecars/gemini-browser/src/answer-extractor.ts
git commit -m "feat: integrate Gemini answer extraction diagnostics"
```

---

### Task 3: Rust DTOs, Run Log, And Prompt-Pack Partial Guard

**Files:**

- Modify: `src-tauri/src/gemini_browser/types.rs`
- Modify: `src-tauri/src/gemini_browser/run_log.rs`
- Modify: `src-tauri/src/prompt_packs/gemini_browser_stage.rs`

- [x] **Step 1: Add failing Rust tests for DTO round-trip and automation guard**

Modify `src-tauri/src/gemini_browser/types.rs` test `run_result_serializes_optional_debug_summary()` so the constructed `GeminiBrowserRunResult` includes:

```rust
artifacts: GeminiBrowserArtifactRefs {
    run_dir: None,
    html: None,
    screenshot: None,
    telemetry: None,
    answer_extraction: Some("answer-extraction.json".to_string()),
    artifact_write_error: None,
},
debug_summary: Some(GeminiBrowserRunDebugSummary {
    mode: GeminiBrowserProviderMode::CdpAttach,
    composer_found: true,
    send_button_found: true,
    generation_busy_observed: true,
    answer_found: true,
    answer_selector: Some("message-content".to_string()),
    waited_for_send_ms: 15_000,
    waited_for_answer_ms: 12_000,
    answer_stable_ms: 8_000,
    answer_completion_reason: GeminiBrowserAnswerCompletionReason::Stable,
    final_text_length: 6,
    error_stage: None,
    extraction: Some(GeminiBrowserAnswerExtractionDebug {
        raw_candidate_count: 2,
        grouped_candidate_count: 1,
        selected_candidate_length: 6,
        returned_text_length: 6,
        selected_grouping: GeminiBrowserAnswerGrouping::AssistantTurn,
        selected_candidate_rank: Some(1),
        selected_score: Some(120),
        largest_candidate_length: 6,
        larger_valid_candidate_available: false,
        larger_rejected_candidate_count: 1,
        larger_rejected_reasons: vec![GeminiBrowserCandidateRejectReason::Composer],
        top_candidate_lengths: vec![6],
        busy_visible_at_completion: false,
        last_growth_elapsed_ms: Some(8_000),
        candidate_signature_changed_count: 1,
        stable_poll_count_after_last_candidate_change: 3,
    }),
}),
```

Add assertions:

```rust
assert_eq!(json["artifacts"]["answer_extraction"], "answer-extraction.json");
assert_eq!(json["debug_summary"]["extraction"]["selected_grouping"], "assistant_turn");
assert_eq!(
    decoded
        .debug_summary
        .expect("debug summary")
        .extraction
        .expect("extraction")
        .larger_rejected_reasons,
    vec![GeminiBrowserCandidateRejectReason::Composer]
);
```

Modify `src-tauri/src/prompt_packs/gemini_browser_stage.rs` tests:

```rust
#[test]
fn timeout_latest_ok_result_is_not_prompt_completion() {
    let mut result = result(GeminiBrowserRunStatus::Ok, Some("partial answer"));
    result.debug_summary = Some(GeminiBrowserRunDebugSummary {
        mode: GeminiBrowserProviderMode::CdpAttach,
        composer_found: true,
        send_button_found: true,
        generation_busy_observed: false,
        answer_found: true,
        answer_selector: Some("message-content".to_string()),
        waited_for_send_ms: 0,
        waited_for_answer_ms: 120_000,
        answer_stable_ms: 8_000,
        answer_completion_reason: GeminiBrowserAnswerCompletionReason::TimeoutLatest,
        final_text_length: 14,
        error_stage: None,
        extraction: None,
    });

    let error =
        browser_result_to_completion_text(result).expect_err("partial-risk must not complete");
    assert_eq!(error.kind, AppErrorKind::Validation);
    assert!(error.message.contains("partial"));
    assert!(error.message.contains("timeout_latest"));
}
```

Update imports in the test module:

```rust
use crate::error::AppErrorKind;

GeminiBrowserAnswerCompletionReason, GeminiBrowserAnswerExtractionDebug,
GeminiBrowserAnswerGrouping, GeminiBrowserArtifactRefs,
GeminiBrowserCandidateRejectReason, GeminiBrowserDebugErrorStage,
GeminiBrowserProviderMode, GeminiBrowserRunDebugSummary,
GeminiBrowserRunResult, GeminiBrowserRunStatus,
```

- [x] **Step 2: Run Rust Gemini browser tests and verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib gemini_browser
```

Expected: FAIL because Rust DTOs do not yet define extraction fields.

- [x] **Step 3: Add Rust DTOs**

Modify `src-tauri/src/gemini_browser/types.rs`:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeminiBrowserCandidateRejectReason {
    Baseline,
    Composer,
    PromptContainer,
    Navigation,
    AccountOrLogin,
    Controls,
    MultiTurn,
    NotVisible,
    Empty,
    LowerScore,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeminiBrowserAnswerGrouping {
    AssistantTurn,
    SingleNode,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserAnswerExtractionDebug {
    pub raw_candidate_count: u64,
    pub grouped_candidate_count: u64,
    pub selected_candidate_length: u64,
    pub returned_text_length: u64,
    pub selected_grouping: GeminiBrowserAnswerGrouping,
    pub selected_candidate_rank: Option<u64>,
    pub selected_score: Option<i64>,
    pub largest_candidate_length: u64,
    pub larger_valid_candidate_available: bool,
    pub larger_rejected_candidate_count: u64,
    pub larger_rejected_reasons: Vec<GeminiBrowserCandidateRejectReason>,
    pub top_candidate_lengths: Vec<u64>,
    pub busy_visible_at_completion: bool,
    pub last_growth_elapsed_ms: Option<u64>,
    pub candidate_signature_changed_count: u64,
    pub stable_poll_count_after_last_candidate_change: u64,
}
```

Extend `GeminiBrowserArtifactRefs` and keep the existing `Default` derive:

```rust
pub answer_extraction: Option<String>,
```

Extend `GeminiBrowserRunDebugSummary`:

```rust
#[serde(default)]
pub extraction: Option<GeminiBrowserAnswerExtractionDebug>,
```

- [x] **Step 4: Add prompt-pack partial guard**

Modify `browser_result_to_completion_text()` in `src-tauri/src/prompt_packs/gemini_browser_stage.rs`:

```rust
GeminiBrowserRunStatus::Ok => {
    if result
        .debug_summary
        .as_ref()
        .is_some_and(|summary| {
            summary.answer_completion_reason
                == crate::gemini_browser::GeminiBrowserAnswerCompletionReason::TimeoutLatest
        })
    {
        return Err(AppError::validation(
            "Gemini browser result is partial-risk (timeout_latest) and cannot be used as a prompt completion",
        ));
    }
    result
        .text
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| AppError::internal("Gemini browser result did not include text"))
}
```

- [x] **Step 5: Run Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib gemini_browser
```

Expected: PASS.

- [x] **Step 6: Mark task complete and commit**

Update this task's checkboxes to `[x]`, then run:

```powershell
git add docs/superpowers/plans/2026-06-21-gemini-browser-answer-extraction-hardening-plan.md src-tauri/src/gemini_browser/types.rs src-tauri/src/gemini_browser/run_log.rs src-tauri/src/prompt_packs/gemini_browser_stage.rs
git commit -m "feat: guard partial Gemini browser completions"
```

---

### Task 4: Frontend Types, Run Inspector Diagnostics, And Partial-Risk UI

**Files:**

- Modify: `src/lib/types/gemini-browser.ts`
- Modify: `src/lib/gemini-browser-run-inspector.ts`
- Modify: `src/lib/gemini-browser-run-inspector.test.ts`
- Modify: `src/lib/components/settings/gemini-browser-provider-panel.svelte`
- Modify: `src/lib/gemini-browser-provider-panel.test.ts`

- [x] **Step 1: Add failing frontend model tests**

Modify `src/lib/gemini-browser-run-inspector.test.ts` fixture `result()`:

```ts
artifacts: {
  run_dir: "C:/Users/Dima/AppData/Roaming/org.ai.extractum/gemini-browser/runs/run-1",
  html: null,
  screenshot: null,
  telemetry: "C:/Users/Dima/AppData/Roaming/org.ai.extractum/gemini-browser/runs/run-1/telemetry.json",
  answer_extraction: "C:/Users/Dima/AppData/Roaming/org.ai.extractum/gemini-browser/runs/run-1/answer-extraction.json",
  artifact_write_error: null,
},
debug_summary: {
  // existing fields...
  extraction: {
    raw_candidate_count: 3,
    grouped_candidate_count: 1,
    selected_candidate_length: 95,
    returned_text_length: 95,
    selected_grouping: "assistant_turn",
    selected_candidate_rank: 1,
    selected_score: 120,
    largest_candidate_length: 95,
    larger_valid_candidate_available: false,
    larger_rejected_candidate_count: 1,
    larger_rejected_reasons: ["composer"],
    top_candidate_lengths: [95, 14],
    busy_visible_at_completion: false,
    last_growth_elapsed_ms: 8_000,
    candidate_signature_changed_count: 2,
    stable_poll_count_after_last_candidate_change: 3,
  },
},
```

Update artifact availability expectation:

```ts
expect(artifactAvailability(result())).toEqual({
  run_dir: true,
  html: false,
  screenshot: false,
  telemetry: true,
  answer_extraction: true,
  artifact_write_error: false,
});
```

Add tests:

```ts
it("copies extraction diagnostics without artifact paths or answer text", () => {
  const diagnostics = copyableRunDiagnostics(run());

  expect(diagnostics).toContain("answer_extraction_artifact_available: true");
  expect(diagnostics).toContain("extraction_raw_candidate_count: 3");
  expect(diagnostics).toContain("extraction_grouped_candidate_count: 1");
  expect(diagnostics).toContain("extraction_selected_grouping: assistant_turn");
  expect(diagnostics).toContain("extraction_larger_valid_candidate_available: false");
  expect(diagnostics).not.toContain("answer-extraction.json");
  expect(diagnostics).not.toContain("answer text");
});

it("detects timeout_latest as partial risk", () => {
  const partial = result({
    debug_summary: {
      ...result().debug_summary!,
      answer_completion_reason: "timeout_latest",
    },
  });

  expect(isPartialRiskBrowserResult(partial)).toBe(true);
  expect(isPartialRiskBrowserResult(result())).toBe(false);
});
```

- [x] **Step 2: Run frontend model tests and verify they fail**

Run:

```powershell
npm.cmd run test -- --run src/lib/gemini-browser-run-inspector.test.ts
```

Expected: FAIL because frontend types/model do not yet include extraction fields.

- [x] **Step 3: Update frontend DTO types**

Modify `src/lib/types/gemini-browser.ts` with the TypeScript DTOs from Task 1, using frontend naming:

```ts
export type GeminiBrowserCandidateRejectReason =
  | "baseline"
  | "composer"
  | "prompt_container"
  | "navigation"
  | "account_or_login"
  | "controls"
  | "multi_turn"
  | "not_visible"
  | "empty"
  | "lower_score";

export type GeminiBrowserAnswerGrouping = "assistant_turn" | "single_node" | "unknown";

export interface GeminiBrowserAnswerExtractionDebug {
  raw_candidate_count: number;
  grouped_candidate_count: number;
  selected_candidate_length: number;
  returned_text_length: number;
  selected_grouping: GeminiBrowserAnswerGrouping;
  selected_candidate_rank: number | null;
  selected_score: number | null;
  largest_candidate_length: number;
  larger_valid_candidate_available: boolean;
  larger_rejected_candidate_count: number;
  larger_rejected_reasons: GeminiBrowserCandidateRejectReason[];
  top_candidate_lengths: number[];
  busy_visible_at_completion: boolean;
  last_growth_elapsed_ms: number | null;
  candidate_signature_changed_count: number;
  stable_poll_count_after_last_candidate_change: number;
}
```

Extend `GeminiBrowserArtifactRefs`:

```ts
answer_extraction?: string | null;
```

Extend `GeminiBrowserRunDebugSummary`:

```ts
extraction?: GeminiBrowserAnswerExtractionDebug | null;
```

- [x] **Step 4: Update inspector model**

Modify `src/lib/gemini-browser-run-inspector.ts`:

```ts
export function isPartialRiskBrowserResult(result: GeminiBrowserRunResult | null): boolean {
  return result?.status === "ok" && result.debug_summary?.answer_completion_reason === "timeout_latest";
}
```

Update `artifactAvailability()` to include:

```ts
answer_extraction: Boolean(result?.artifacts.answer_extraction),
```

In `copyableRunDiagnostics()`, add:

```ts
`answer_extraction_artifact_available: ${availability.answer_extraction}`,
`partial_risk: ${isPartialRiskBrowserResult(result)}`,
```

After existing debug lines, if `debug.extraction` exists, push:

```ts
`extraction_raw_candidate_count: ${debug.extraction.raw_candidate_count}`,
`extraction_grouped_candidate_count: ${debug.extraction.grouped_candidate_count}`,
`extraction_selected_candidate_length: ${debug.extraction.selected_candidate_length}`,
`extraction_returned_text_length: ${debug.extraction.returned_text_length}`,
`extraction_selected_grouping: ${debug.extraction.selected_grouping}`,
`extraction_selected_candidate_rank: ${debug.extraction.selected_candidate_rank ?? "none"}`,
`extraction_largest_candidate_length: ${debug.extraction.largest_candidate_length}`,
`extraction_larger_valid_candidate_available: ${debug.extraction.larger_valid_candidate_available}`,
`extraction_larger_rejected_candidate_count: ${debug.extraction.larger_rejected_candidate_count}`,
`extraction_larger_rejected_reasons: ${debug.extraction.larger_rejected_reasons.join(",") || "none"}`,
`extraction_busy_visible_at_completion: ${debug.extraction.busy_visible_at_completion}`,
`extraction_candidate_signature_changed_count: ${debug.extraction.candidate_signature_changed_count}`,
`extraction_stable_poll_count_after_last_candidate_change: ${debug.extraction.stable_poll_count_after_last_candidate_change}`,
```

Do not include `result.artifacts.answer_extraction` path in copied diagnostics.

- [x] **Step 5: Update Settings inspector UI**

Modify imports in `src/lib/components/settings/gemini-browser-provider-panel.svelte`:

```ts
import {
  artifactAvailability,
  copyableRunDiagnostics,
  debugFinalTextLength,
  isPartialRiskBrowserResult,
  resultTextLength,
  sanitizeDiagnosticMessage,
  selectedRunForInspector,
} from "$lib/gemini-browser-run-inspector";
```

Add a derived partial-risk flag:

```ts
const selectedPartialRisk = $derived(isPartialRiskBrowserResult(selectedInspectorResult));
```

In the top inspector grid, add:

```svelte
<div class:warning={selectedPartialRisk}>
  <span class="fact-label">Partial risk</span>
  <span>{selectedPartialRisk ? "yes" : "no"}</span>
</div>
```

In artifact grid, add:

```svelte
<div>
  <span class="fact-label">Answer extraction</span>
  <span>{selectedArtifactAvailability.answer_extraction ? "available" : "not captured"}</span>
</div>
```

Inside the `debug_summary` block, add an extraction grid guarded by `debug_summary.extraction`:

```svelte
{#if selectedInspectorResult.debug_summary.extraction}
  <div class="inspector-grid compact">
    <div>
      <span class="fact-label">Raw candidates</span>
      <span>{selectedInspectorResult.debug_summary.extraction.raw_candidate_count}</span>
    </div>
    <div>
      <span class="fact-label">Grouped candidates</span>
      <span>{selectedInspectorResult.debug_summary.extraction.grouped_candidate_count}</span>
    </div>
    <div>
      <span class="fact-label">Selected grouping</span>
      <span>{selectedInspectorResult.debug_summary.extraction.selected_grouping}</span>
    </div>
    <div>
      <span class="fact-label">Selected length</span>
      <span>{selectedInspectorResult.debug_summary.extraction.selected_candidate_length}</span>
    </div>
    <div>
      <span class="fact-label">Largest candidate</span>
      <span>{selectedInspectorResult.debug_summary.extraction.largest_candidate_length}</span>
    </div>
    <div>
      <span class="fact-label">Larger valid</span>
      <span>{selectedInspectorResult.debug_summary.extraction.larger_valid_candidate_available ? "yes" : "no"}</span>
    </div>
    <div>
      <span class="fact-label">Signature changes</span>
      <span>{selectedInspectorResult.debug_summary.extraction.candidate_signature_changed_count}</span>
    </div>
    <div>
      <span class="fact-label">Stable polls</span>
      <span>{selectedInspectorResult.debug_summary.extraction.stable_poll_count_after_last_candidate_change}</span>
    </div>
  </div>
{/if}
```

Add CSS:

```css
.warning {
  border-color: color-mix(in srgb, var(--destructive) 55%, var(--border));
}
```

- [x] **Step 6: Update source-contract UI test**

Modify `src/lib/gemini-browser-provider-panel.test.ts` to assert:

```ts
expect(componentSource).toContain("Partial risk");
expect(componentSource).toContain("Answer extraction");
expect(componentSource).toContain("raw_candidate_count");
expect(componentSource).toContain("grouped_candidate_count");
expect(componentSource).toContain("isPartialRiskBrowserResult");
```

- [x] **Step 7: Run frontend tests**

Run:

```powershell
npm.cmd run test -- --run src/lib/gemini-browser-run-inspector.test.ts src/lib/gemini-browser-provider-panel.test.ts
```

Expected: PASS.

- [x] **Step 8: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: PASS with `svelte-check found 0 errors and 0 warnings`.

- [x] **Step 9: Mark task complete and commit**

Update this task's checkboxes to `[x]`, then run:

```powershell
git add docs/superpowers/plans/2026-06-21-gemini-browser-answer-extraction-hardening-plan.md src/lib/types/gemini-browser.ts src/lib/gemini-browser-run-inspector.ts src/lib/gemini-browser-run-inspector.test.ts src/lib/components/settings/gemini-browser-provider-panel.svelte src/lib/gemini-browser-provider-panel.test.ts
git commit -m "feat: show Gemini extraction diagnostics"
```

---

### Task 5: Documentation, Manual Validation Notes, And Final Verification

**Files:**

- Modify: `docs/browser-providers-llm-troubleshooting.md`
- Modify: `docs/superpowers/plans/2026-06-21-gemini-browser-answer-extraction-hardening-plan.md`

- [x] **Step 1: Update troubleshooting docs**

In `docs/browser-providers-llm-troubleshooting.md`, add a section under the run inspector/extraction troubleshooting area:

```md
### Answer Extraction Diagnostics

Use this when Gemini visibly produced more text than Extractum received.

Check these fields first:

- `answer_completion_reason`: `stable` means the grouped candidate satisfied the quiet window; `timeout_latest` means the sidecar returned visible text without proving completion.
- `partial_risk`: `true` means the Settings UI may show text, but prompt-pack automation must not consume it as a normal completion.
- `result_text_length` vs `debug_final_text_length`: mismatch means UI/run propagation differs from sidecar extraction.
- `extraction_raw_candidate_count` and `extraction_grouped_candidate_count`: raw > grouped is normal when Gemini splits one answer into blocks.
- `extraction_selected_grouping`: `assistant_turn` is preferred; `single_node` is a fallback and should be treated with more suspicion after DOM changes.
- `extraction_largest_candidate_length` and `extraction_larger_valid_candidate_available`: a larger valid candidate means selection/scoring needs review.
- `answer_extraction_artifact_available`: local-only artifact with selector/count/length facts, not safe for external sharing without review.

For `timeout_latest`, inspect the run locally and decide whether to retry, extend the prompt, or treat it as a failed browser-provider completion. Do not feed `timeout_latest` text into long prompt-pack analysis as final output.
```

- [x] **Step 2: Run full verification**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar
```

Expected: PASS.

Run:

```powershell
npm.cmd run test -- --run sidecars/gemini-browser/src/answer-extractor.test.ts sidecars/gemini-browser/src/adapter.test.ts
```

Expected: PASS.

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib gemini_browser
```

Expected: PASS.

Run:

```powershell
npm.cmd run test -- --run src/lib/gemini-browser-run-inspector.test.ts src/lib/gemini-browser-provider-panel.test.ts
```

Expected: PASS.

Run:

```powershell
npm.cmd run check
```

Expected: PASS with `svelte-check found 0 errors and 0 warnings`.

Run:

```powershell
git diff --check
```

Expected: no output and exit code 0.

- [x] **Step 3: Rebuild sidecar binary**

Run:

```powershell
npm.cmd run build:gemini-browser-sidecar
```

Expected:

```text
Wrote src-tauri\binaries\gemini-browser-sidecar-x86_64-pc-windows-msvc.exe
```

Deprecation warnings from `pkg` or Node are acceptable if the binary is written.

- [x] **Step 4: Manual validation**

Start the app:

```powershell
npm.cmd run tauri dev
```

This manual validation checks Browser Provider behavior in the development app.
The rebuilt packaged sidecar binary is validated separately by
`npm.cmd run build:gemini-browser-sidecar`; do not infer packaged sidecar
runtime behavior from this dev-mode manual check alone.

Manual stable validation:

1. Settings -> Browser Providers.
2. Use Attach Chrome mode and Start Chrome, or use an already attached CDP Chrome.
3. Send the Russian football prompt:

```text
Ты знаешь последние новости ЧМ по футболу?
```

4. Wait until Gemini visibly finishes.
5. Refresh Run inspector.
6. Confirm:
   - `Result text length` equals `Debug final length`.
   - `Answer reason` is `stable`.
   - `Partial risk` is `no`.
   - extraction raw/grouped/selected fields are present.
   - Copy diagnostics does not include the answer text.

Manual slow/partial validation:

1. Send a deliberately long prompt that asks for many sections:

```text
Составь длинный структурированный обзор последних новостей ЧМ по футболу: группы, сенсации, травмы, расписание, фавориты, спорные моменты, по 5 пунктов в каждом разделе.
```

2. Acceptable outcomes:
   - `stable` with full visible text and matching lengths; or
   - `timeout_latest` visibly marked as partial-risk, with copied diagnostics including `partial_risk: true`.
3. If `timeout_latest` appears, confirm prompt-pack automation guard test already passed and do not treat this manual run as a normal prompt-pack completion.

- [x] **Step 5: Record manual validation outcome in this plan**

Append a short note here:

```md
## Manual Validation Result

- Date:
- Mode:
- Stable prompt run id:
- Stable result:
- Slow prompt run id:
- Slow/partial result:
- Notes:
```

Fill the fields with the observed run ids and diagnostics.

## Manual Validation Result

- Date: 2026-06-21
- Mode: `cdp_attach`
- Stable prompt run id: `gemini-browser-1782047532781-d92def16e61c8`
- Stable result: `ok`, `answer_completion_reason: stable`, `partial_risk: false`, `result_text_length: 916`, `debug_final_text_length: 916`, `extraction_raw_candidate_count: 2`, `extraction_grouped_candidate_count: 1`
- Slow prompt run id: `gemini-browser-1782047600612-94120d9c27826`
- Slow/partial result: `ok`, `answer_completion_reason: stable`, `partial_risk: false`, `result_text_length: 5065`, `debug_final_text_length: 5065`, `extraction_raw_candidate_count: 3`, `extraction_grouped_candidate_count: 1`
- Notes: Both manual Browser Provider runs completed as stable with matching UI/result lengths. No `timeout_latest` partial-risk run was observed. `answer_extraction_artifact_available: false` is expected for stable completions.

- [x] **Step 6: Mark task complete and commit**

Update this task's checkboxes to `[x]`, then run:

```powershell
git add docs/browser-providers-llm-troubleshooting.md docs/superpowers/plans/2026-06-21-gemini-browser-answer-extraction-hardening-plan.md src-tauri/binaries/gemini-browser-sidecar-x86_64-pc-windows-msvc.exe
git commit -m "docs: document Gemini extraction diagnostics"
```

---

## Final Completion Checklist

- [x] `npm.cmd run test:gemini-browser-sidecar` passes.
- [x] `npm.cmd run test -- --run sidecars/gemini-browser/src/answer-extractor.test.ts sidecars/gemini-browser/src/adapter.test.ts` passes.
- [x] `cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-browser --lib gemini_browser` passes.
- [x] `npm.cmd run test -- --run src/lib/gemini-browser-run-inspector.test.ts src/lib/gemini-browser-provider-panel.test.ts` passes.
- [x] `npm.cmd run check` passes.
- [x] `git diff --check` passes.
- [x] Sidecar binary rebuilt.
- [x] Manual stable validation recorded.
- [x] Manual slow/partial validation recorded.
- [x] Working tree is clean after final commit.
