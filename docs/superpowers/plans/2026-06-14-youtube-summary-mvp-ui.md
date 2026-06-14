# YouTube Summary Prompt Pack UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a usable YouTube Summary MVP interface for starting prompt-pack runs, watching active progress, and reading structured results.

**Architecture:** Keep UI state in small TypeScript workflow modules and compose existing Svelte components. Use the new `prompt-pack-run-event` event stream for active updates and Tauri command wrappers for preflight/start/result reads. The UI should not call legacy `analysis_runs` APIs.

**Tech Stack:** Svelte 5/SvelteKit, Vitest, Tauri invoke/listen, existing `src/lib/components/ui` primitives, `@lucide/svelte` icons already available in the project.

---

## Dependencies

Complete these plans first:

- `docs/superpowers/plans/2026-06-14-youtube-summary-mvp-foundation.md`
- `docs/superpowers/plans/2026-06-14-youtube-summary-mvp-runtime.md`
- `docs/superpowers/plans/2026-06-14-youtube-summary-mvp-execution-result.md`

---

## UI Principles

- Use existing components from `src/lib/components/ui` and `src/lib/components/research-projects`.
- Use icon buttons with Lucide icons for compact actions.
- Use `Badge`, `StatusMessage`, `PanelHeader`, `MetaCell`, `Button`, `Dialog`, `Sheet`, `Tabs`, `ScrollArea`, and existing Extractum UI primitives before creating custom markup.
- Use `flex` with `gap-*`, not `space-*`.
- Use semantic tokens and existing CSS classes; do not introduce a new one-note color theme.
- Do not build a marketing page. The first screen is the working YouTube Summary run experience.

---

## File Structure

- Modify `src/lib/types/prompt-packs.ts`: final UI-facing DTOs.
- Modify `src/lib/api/prompt-packs.ts`: final command and event wrappers.
- Create `src/lib/ui/youtube-summary-workflow.ts`: pure state helpers for preflight/run/result UI.
- Create `src/lib/ui/youtube-summary-workflow.test.ts`: Vitest coverage for state transitions and partitions.
- Create `src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte`: launch dialog with source selection summary and options.
- Create `src/lib/components/research-projects/YoutubeSummaryRunsPanel.svelte`: active/recent runs list and progress surface.
- Create `src/lib/components/research-projects/YoutubeSummaryResultView.svelte`: structured result viewer for videos, claims, evidence, warnings, and limitations.
- Modify `src/lib/components/research-projects/LibraryInspector.svelte`: entry point for synced YouTube video/playlist sources.
- Modify `src/lib/components/research-projects/ProjectRunsTab.svelte`: include Prompt Pack runs or link to the new panel when a project is selected.
- Modify `src/routes/projects/library/+page.svelte` or the current Library screen owner if route composition has moved.

---

## Task 1: UI Workflow State

**Files:**
- Create: `src/lib/ui/youtube-summary-workflow.ts`
- Create: `src/lib/ui/youtube-summary-workflow.test.ts`
- Modify: `src/lib/types/prompt-packs.ts`

- [ ] **Step 1: Write workflow tests**

Add tests:

```ts
import { describe, expect, it } from "vitest";
import {
  canStartYoutubeSummary,
  summarizePreflightPartitions,
  updateRunListFromEvent,
} from "./youtube-summary-workflow";
import type { PromptPackRunEvent, YoutubeSummaryPreflightResponse } from "$lib/types/prompt-packs";

describe("youtube summary workflow", () => {
  it("allows start only with included videos and no blocking failures", () => {
    const preflight: YoutubeSummaryPreflightResponse = {
      packId: "youtube_summary",
      packVersion: "1.0.0",
      includedVideos: [{ sourceId: 1, videoId: "v1", title: "Ready", estimatedInputTokens: 1200 }],
      skippedVideos: [],
      blockingFailures: [],
      estimatedInputTokens: 1200,
      selectedModelInputLimit: 32000,
    };

    expect(canStartYoutubeSummary(preflight)).toBe(true);
  });

  it("summarizes partial playlist partitions", () => {
    const summary = summarizePreflightPartitions({
      includedVideos: [{ sourceId: 1, videoId: "v1", title: "Ready", estimatedInputTokens: 1200 }],
      skippedVideos: [{ sourceId: 2, videoId: "v2", title: "Missing", reason: "no_usable_transcript" }],
      blockingFailures: [],
    });

    expect(summary).toEqual({
      includedCount: 1,
      skippedCount: 1,
      blockingCount: 0,
      hasPartialCoverage: true,
    });
  });

  it("updates run list from prompt pack run event", () => {
    const event: PromptPackRunEvent = {
      runId: 42,
      requestId: "req-42",
      kind: "progress",
      runStatus: "running",
      phase: "stage",
      stageRunId: 1001,
      stageName: "youtube_summary/transcript_analysis",
      sourceSnapshotId: 501,
      queuePosition: null,
      progressCurrent: 1,
      progressTotal: 2,
      message: "Analyzing transcript",
      error: null,
    };

    const runs = updateRunListFromEvent([], event);

    expect(runs[0].runId).toBe(42);
    expect(runs[0].runStatus).toBe("running");
    expect(runs[0].latestMessage).toBe("Analyzing transcript");
  });
});
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```powershell
npm test -- --run src/lib/ui/youtube-summary-workflow.test.ts
```

Expected: fail because workflow module does not exist.

- [ ] **Step 3: Implement workflow module**

Export:

```ts
export function canStartYoutubeSummary(preflight: YoutubeSummaryPreflightResponse | null): boolean
export function summarizePreflightPartitions(preflight: Pick<YoutubeSummaryPreflightResponse, "includedVideos" | "skippedVideos" | "blockingFailures">): YoutubeSummaryPartitionSummary
export function updateRunListFromEvent(runs: PromptPackRunListItem[], event: PromptPackRunEvent): PromptPackRunListItem[]
export function statusLabel(status: PromptPackRunStatus): string
```

- [ ] **Step 4: Run workflow tests**

Run:

```powershell
npm test -- --run src/lib/ui/youtube-summary-workflow.test.ts
```

Expected: pass.

- [ ] **Step 5: Commit**

```powershell
git add src/lib/ui/youtube-summary-workflow.ts src/lib/ui/youtube-summary-workflow.test.ts src/lib/types/prompt-packs.ts
git commit -m "feat: add youtube summary ui workflow state"
```

---

## Task 2: API Wrappers and Event Subscription

**Files:**
- Modify: `src/lib/api/prompt-packs.ts`
- Modify: `src/lib/api/prompt-packs.test.ts`
- Modify: `src/lib/types/prompt-packs.ts`

- [ ] **Step 1: Add wrapper tests**

Extend `src/lib/api/prompt-packs.test.ts`:

```ts
it("starts youtube summary run", async () => {
  vi.mocked(invoke).mockResolvedValueOnce({
    kind: "started",
    run: { runId: 42, runStatus: "queued", latestMessage: "Queued" },
  });

  const outcome = await startYoutubeSummaryRun({
    clientRequestId: "req-ui-start-1",
    projectId: null,
    sourceIds: [10],
    profileId: null,
    modelOverride: null,
    outputLanguage: "en",
    controlPreset: "standard",
    evidenceMode: "standard",
    includeComments: false,
  });

  expect(outcome.kind).toBe("started");
  expect(invoke).toHaveBeenCalledWith("start_youtube_summary_run", {
    clientRequestId: "req-ui-start-1",
    projectId: null,
    sourceIds: [10],
    profileId: null,
    modelOverride: null,
    outputLanguage: "en",
    controlPreset: "standard",
    evidenceMode: "standard",
    includeComments: false,
  });
});

it("returns blocked start outcome without hiding fresh preflight failures", async () => {
  vi.mocked(invoke).mockResolvedValueOnce({
    kind: "blocked",
    preflight: {
      packId: "youtube_summary",
      packVersion: "1.0.0",
      includedVideos: [],
      skippedVideos: [],
      blockingFailures: [{ sourceId: 10, reason: "no_included_videos" }],
      estimatedInputTokens: 0,
      selectedModelInputLimit: 32000,
    },
  });

  const outcome = await startYoutubeSummaryRun({
    clientRequestId: "req-ui-blocked-1",
    projectId: null,
    sourceIds: [10],
    profileId: null,
    modelOverride: null,
    outputLanguage: "en",
    controlPreset: "standard",
    evidenceMode: "standard",
    includeComments: false,
  });

  if (outcome.kind !== "blocked") {
    throw new Error(`expected blocked outcome, got ${outcome.kind}`);
  }

  expect(outcome.preflight.blockingFailures).toHaveLength(1);
});

it("lists recent prompt pack runs", async () => {
  await listPromptPackRuns({ projectId: 7, limit: 20 });

  expect(invoke).toHaveBeenCalledWith("list_prompt_pack_runs", {
    projectId: 7,
    limit: 20,
  });
});

it("listens to prompt pack run events", async () => {
  const handler = vi.fn();

  await listenToPromptPackRunEvents(handler);

  expect(listen).toHaveBeenCalledWith("prompt-pack-run-event", expect.any(Function));
});
```

- [ ] **Step 2: Implement wrappers**

Export:

```ts
export const PROMPT_PACK_RUN_EVENT = "prompt-pack-run-event";
export interface ListPromptPackRunsInput {
  projectId?: number | null;
  limit?: number;
}
export function preflightYoutubeSummaryRun(input: PreflightYoutubeSummaryRunInput): Promise<YoutubeSummaryPreflightResponse>
export function startYoutubeSummaryRun(input: StartYoutubeSummaryRunInput): Promise<StartYoutubeSummaryRunOutcome>
export function cancelPromptPackRun(runId: number): Promise<void>
export function listPromptPackRuns(input?: ListPromptPackRunsInput): Promise<PromptPackRunSummary[]>
export function listActivePromptPackRuns(): Promise<PromptPackRunSummary[]>
export function listPromptPackRunStages(runId: number): Promise<PromptPackStageRun[]>
export function getPromptPackResult(runId: number): Promise<PromptPackResult>
export function getPromptPackValidationFindings(runId: number): Promise<PromptPackValidationFinding[]>
export function listenToPromptPackRunEvents(handler: (event: Event<PromptPackRunEvent>) => void): Promise<UnlistenFn>
```

- [ ] **Step 3: Run API tests**

Run:

```powershell
npm test -- --run src/lib/api/prompt-packs.test.ts
```

Expected: pass.

- [ ] **Step 4: Commit**

```powershell
git add src/lib/api/prompt-packs.ts src/lib/api/prompt-packs.test.ts src/lib/types/prompt-packs.ts
git commit -m "feat: add prompt pack frontend api"
```

---

## Task 3: Launch Dialog

**Files:**
- Create: `src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte`
- Modify: `src/lib/components/research-projects/LibraryInspector.svelte`

- [ ] **Step 1: Add contract test**

Create a route/component contract test if this project has one for `LibraryInspector`; otherwise add `src/lib/youtube-summary-launch-contract.test.ts` that reads component source and asserts these strings exist:

```ts
expect(source).toContain("YoutubeSummaryRunDialog");
expect(source).toContain("preflightYoutubeSummaryRun");
expect(source).toContain("startYoutubeSummaryRun");
```

- [ ] **Step 2: Implement dialog**

Dialog behavior:

- accepts selected synced YouTube video or playlist source ids;
- shows source title, source subtype, transcript readiness summary from preflight;
- controls:
  - output language select/input using existing app pattern;
  - LLM profile select using existing LLM profiles API;
  - model override optional field;
  - `controlPreset` with `standard`;
  - `evidenceMode` with `standard`;
  - `includeComments` checkbox default off;
- preflight runs before enabling start;
- if `skippedVideos` is non-empty, show partial coverage warning;
- if `blockingFailures` is non-empty, disable start and show reasons;
- start button generates a stable `clientRequestId`, calls `startYoutubeSummaryRun`, opens the run panel for `{ kind: "started" }`, and renders the fresh blocking preflight for `{ kind: "blocked" }`.

Use existing UI imports:

```svelte
import * as Dialog from "$lib/components/ui/dialog";
import { Button } from "$lib/components/ui/button";
import { Badge } from "$lib/components/ui/badge";
import { Checkbox } from "$lib/components/ui/checkbox";
import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
```

- [ ] **Step 3: Add Library entry point**

In `LibraryInspector.svelte`, show a "YouTube Summary" action only when:

- source is YouTube;
- source subtype is video or playlist;
- source has completed sync state available from current inspector data.

Do not show the action for Telegram or unsupported source kinds.

- [ ] **Step 4: Run tests/check**

Run:

```powershell
npm test -- --run src/lib/youtube-summary-launch-contract.test.ts src/lib/ui/youtube-summary-workflow.test.ts
npm run check
```

Expected: pass.

- [ ] **Step 5: Commit**

```powershell
git add src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte src/lib/components/research-projects/LibraryInspector.svelte src/lib/youtube-summary-launch-contract.test.ts
git commit -m "feat: add youtube summary launch dialog"
```

---

## Task 4: Active Runs Panel

**Files:**
- Create: `src/lib/components/research-projects/YoutubeSummaryRunsPanel.svelte`
- Modify: `src/lib/components/research-projects/ProjectRunsTab.svelte`

- [ ] **Step 1: Add workflow coverage**

Extend `youtube-summary-workflow.test.ts` to cover terminal event updates:

```ts
it("marks run terminal from completed event", () => {
  const runs = updateRunListFromEvent(
    [{ runId: 42, runStatus: "running", latestMessage: "Running" }],
    {
      runId: 42,
      requestId: "req-42",
      kind: "completed",
      runStatus: "complete",
      phase: "terminal",
      stageRunId: null,
      stageName: null,
      sourceSnapshotId: null,
      queuePosition: null,
      progressCurrent: 2,
      progressTotal: 2,
      message: "Completed",
      error: null,
    },
  );

  expect(runs[0].runStatus).toBe("complete");
  expect(runs[0].latestMessage).toBe("Completed");
});
```

- [ ] **Step 2: Implement panel**

Panel responsibilities:

- load `listPromptPackRuns({ projectId, limit: 20 })` and `listActivePromptPackRuns` on mount;
- subscribe to `prompt-pack-run-event`;
- display active and recent Prompt Pack runs;
- show status badges, current stage, message, and cancel button;
- call `cancelPromptPackRun` for non-terminal runs;
- select terminal run to load result.

- [ ] **Step 3: Wire into Project runs surface**

In `ProjectRunsTab.svelte`, add a Prompt Pack section or tab that hosts `YoutubeSummaryRunsPanel`. Keep legacy analysis runs visually separate.

- [ ] **Step 4: Run tests/check**

Run:

```powershell
npm test -- --run src/lib/ui/youtube-summary-workflow.test.ts
npm run check
```

Expected: pass.

- [ ] **Step 5: Commit**

```powershell
git add src/lib/components/research-projects/YoutubeSummaryRunsPanel.svelte src/lib/components/research-projects/ProjectRunsTab.svelte src/lib/ui/youtube-summary-workflow.test.ts
git commit -m "feat: show active prompt pack runs"
```

---

## Task 5: Result Viewer

**Files:**
- Create: `src/lib/components/research-projects/YoutubeSummaryResultView.svelte`
- Modify: `src/lib/components/research-projects/YoutubeSummaryRunsPanel.svelte`

- [ ] **Step 1: Add result view contract test**

Create `src/lib/youtube-summary-result-view-contract.test.ts` asserting the component imports and displays key structures:

```ts
expect(source).toContain("getPromptPackResult");
expect(source).toContain("claims");
expect(source).toContain("evidence");
expect(source).toContain("limitations");
expect(source).toContain("qualityFlags");
```

- [ ] **Step 2: Implement result viewer**

Viewer sections:

- run header: status, pack version, model/provider, source coverage;
- video summaries;
- segments with timestamps;
- key points;
- notable quotes;
- claims with evidence refs;
- warnings and limitations;
- validation findings if present.

Use existing compact operational styling. Avoid nested cards; use full-width sections, separators, badges, and scroll areas.

- [ ] **Step 3: Wire selected run**

`YoutubeSummaryRunsPanel` passes selected run id into `YoutubeSummaryResultView`. The viewer loads canonical/projection data through `getPromptPackResult`.

- [ ] **Step 4: Run tests/check**

Run:

```powershell
npm test -- --run src/lib/youtube-summary-result-view-contract.test.ts src/lib/ui/youtube-summary-workflow.test.ts
npm run check
```

Expected: pass.

- [ ] **Step 5: Commit**

```powershell
git add src/lib/components/research-projects/YoutubeSummaryResultView.svelte src/lib/components/research-projects/YoutubeSummaryRunsPanel.svelte src/lib/youtube-summary-result-view-contract.test.ts
git commit -m "feat: add youtube summary result viewer"
```

---

## Task 6: Full UI Verification and Browser Smoke

**Files:**
- Modify only files needed to fix issues found by verification.

- [ ] **Step 1: Run frontend checks**

Run:

```powershell
npm test -- --run src/lib/api/prompt-packs.test.ts src/lib/ui/youtube-summary-workflow.test.ts src/lib/youtube-summary-launch-contract.test.ts src/lib/youtube-summary-result-view-contract.test.ts
npm run check
```

Expected: pass.

- [ ] **Step 2: Run app manually**

Run:

```powershell
npm run dev -- --host 127.0.0.1
```

Keep the dev server running for Step 3.

- [ ] **Step 3: Run browser smoke on desktop viewport**

Use the in-app Browser or Playwright against `http://127.0.0.1:1420` with viewport `1440x900`.

Smoke path:

1. open the Library surface;
2. select a synced YouTube video or playlist source;
3. open `YoutubeSummaryRunDialog`;
4. verify preflight, partial coverage, and blocked-start states fit without text overlap;
5. open the Prompt Pack runs panel;
6. verify active/recent rows update from `prompt-pack-run-event`;
7. open a terminal run result;
8. verify the result viewer renders header, videos, claims/evidence, warnings, limitations, and validation findings without layout overlap.

Capture at least one desktop screenshot of the launch dialog and one desktop screenshot of the result viewer. Save them under `artifacts/` when using Playwright, or attach them in the in-app Browser verification notes.

- [ ] **Step 4: Commit verification fixes**

If Step 1, Step 2, or Step 3 required fixes:

```powershell
git add src/lib src/routes
git commit -m "fix: polish youtube summary ui"
```

If no fixes were needed, do not create an empty commit.

---

## Plan Acceptance

Run:

```powershell
npm test -- --run src/lib/api/prompt-packs.test.ts src/lib/ui/youtube-summary-workflow.test.ts src/lib/youtube-summary-launch-contract.test.ts src/lib/youtube-summary-result-view-contract.test.ts
npm run check
git status --short
```

Then complete the browser smoke from Task 6 Step 3.

Expected:

- frontend tests pass;
- Svelte check passes;
- YouTube Summary launch is available only for synced YouTube video/playlist sources;
- Prompt Pack run list updates from `prompt-pack-run-event`;
- active and recent Prompt Pack runs are loaded through separate wrappers;
- desktop browser smoke screenshots show the dialog and result viewer without layout overlap;
- result viewer renders canonical/projection data without using legacy analysis APIs.
