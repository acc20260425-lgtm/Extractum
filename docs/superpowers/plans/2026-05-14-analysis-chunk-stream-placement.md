# Analysis Chunk Stream Placement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restore live analysis chunk summaries as a dedicated `Chunks` tab in the redesigned run companion without auto-switching away from the user's current tab.

**Architecture:** Keep chunk data in the existing `LiveRunState.chunkSummaries` path and surface it through `RunCompanionTabs`. Extend companion state to include `"chunks"`, update persistence parsing/normalization, remove the legacy visible-UI dependency on `inspectorMode: "chunks"`, and adapt `ChunkSummaries` to render compactly inside the companion panel.

**Tech Stack:** Svelte 5 runes, TypeScript, Vitest raw-source and state tests, existing analysis route/workflow helpers.

---

## Execution Rules

- Work from clean `main`.
- Use TDD for every behavior change: add failing test, run and confirm expected failure, implement, rerun.
- Commit after each implementation task.
- Before editing Svelte components, run `mcp__svelte_server__.list_sections`.
- Run `mcp__svelte_server__.svelte_autofixer` on every changed Svelte component before committing.
- Use `apply_patch` for manual edits.
- Do not persist chunk summaries in this pass.
- Do not switch `workspaceUiState.companionTab` to `"chunks"` when a chunk summary event arrives.
- After every commit, append a recovery checkpoint to `reference/session-context-2026-05-10-analysis-redesign.md`; that file is ignored and must not be committed.

## File Structure

- Modify `src/lib/analysis-workspace-state.ts`
  - Extend `CompanionTab`.
  - Normalize restored run-bound `chunks` tab to `runs` when no run is open.
- Modify `src/lib/analysis-workspace-persistence.ts`
  - Parse persisted `"chunks"` companion tab.
- Modify `src/lib/analysis-run-workflow.ts`
  - Stop patching legacy `inspectorMode: "chunks"` on chunk summary events.
- Modify tests:
  - `src/lib/analysis-workspace-state.test.ts`
  - `src/lib/analysis-workspace-persistence.test.ts`
  - `src/lib/analysis-run-workflow.test.ts`
- Modify Svelte components:
  - `src/lib/components/analysis/run-companion-tabs.svelte`
  - `src/lib/components/analysis/chunk-summaries.svelte`
- Modify route:
  - `src/routes/analysis/+page.svelte`
- Modify route/component tests:
  - `src/lib/analysis-run-companion-tabs.test.ts`
  - `src/lib/analysis-run-companion-route.test.ts`
- Modify this plan at the end:
  - append `## Verification Evidence`.

---

### Task 1: Companion State And Workflow Contract

**Files:**
- Modify: `src/lib/analysis-workspace-state.ts`
- Modify: `src/lib/analysis-workspace-persistence.ts`
- Modify: `src/lib/analysis-run-workflow.ts`
- Test: `src/lib/analysis-workspace-state.test.ts`
- Test: `src/lib/analysis-workspace-persistence.test.ts`
- Test: `src/lib/analysis-run-workflow.test.ts`

- [ ] **Step 1: Add failing workspace state tests**

In `src/lib/analysis-workspace-state.test.ts`, extend `normalizes restored run-bound UI state when no run is open` with a `chunks` case:

```ts
expect(normalizeRestoredWorkspaceState(baseState({
  openRunState: { kind: "none" },
  canvasMode: "report",
  sourceViewBasis: "run_snapshot",
  companionTab: "chunks",
  selectedTraceRef: "s7-i1",
}))).toEqual({
  workspaceSelection: { kind: "none" },
  openRunState: { kind: "none" },
  canvasMode: "report",
  sourceViewBasis: "live_source",
  companionTab: "runs",
  selectedTraceRef: null,
});
```

Also add a focused valid-open-run assertion:

```ts
expect(normalizeRestoredWorkspaceState(baseState({
  openRunState: { kind: "active", runId: 42 },
  sourceViewBasis: "run_snapshot",
  companionTab: "chunks",
}))).toMatchObject({
  openRunState: { kind: "active", runId: 42 },
  sourceViewBasis: "run_snapshot",
  companionTab: "chunks",
});
```

- [ ] **Step 2: Add failing persistence test**

In `src/lib/analysis-workspace-persistence.test.ts`, add:

```ts
it("loads persisted chunks companion tab and normalizes it without an opened run", () => {
  const storage = new MemoryStorage();
  storage.setItem(ANALYSIS_WORKSPACE_STATE_KEY, JSON.stringify({
    version: 1,
    workspaceSelection: { kind: "source", sourceId: 7 },
    canvasMode: "report",
    sourceViewBasis: "run_snapshot",
    companionTab: "chunks",
    runs: {
      historyScope: "current",
      runFilter: "all",
      runsFilter: runsFilterDefaults(),
    },
  }));

  const persisted = loadPersistedAnalysisWorkspaceState(storage);
  expect(persisted?.companionTab).toBe("chunks");
  expect(restoredUiStateFromPersisted(persisted!)).toMatchObject({
    sourceViewBasis: "live_source",
    companionTab: "runs",
  });
});
```

- [ ] **Step 3: Replace failing workflow auto-switch test**

In `src/lib/analysis-run-workflow.test.ts`, rename the current test:

```ts
it("applies run events and switches the inspector to chunks when chunk summaries arrive", () => {
```

to:

```ts
it("applies chunk summary run events without auto-switching visible companion state", () => {
```

Change the final assertion from:

```ts
expect(state.inspectorMode).toBe("chunks");
```

to:

```ts
expect(state.inspectorMode).toBe("history");
expect(deps.patch).not.toHaveBeenCalledWith(expect.objectContaining({
  inspectorMode: "chunks",
}));
```

- [ ] **Step 4: Run tests and confirm RED**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-workspace-state.test.ts src/lib/analysis-workspace-persistence.test.ts src/lib/analysis-run-workflow.test.ts
```

Expected:

- workspace state TypeScript or runtime failure because `"chunks"` is not part of `CompanionTab`;
- persistence test fails because `parseCompanionTab` rejects `"chunks"`;
- workflow test fails because `handleRunEvent` still patches `inspectorMode: "chunks"`.

- [ ] **Step 5: Implement state and workflow changes**

In `src/lib/analysis-workspace-state.ts`, change:

```ts
export type CompanionTab = "evidence" | "chat" | "runs";
```

to:

```ts
export type CompanionTab = "evidence" | "chat" | "chunks" | "runs";
```

In `normalizeRestoredWorkspaceState`, change:

```ts
companionTab:
  state.companionTab === "evidence" || state.companionTab === "chat"
    ? "runs"
    : state.companionTab,
```

to:

```ts
companionTab:
  state.companionTab === "evidence" ||
  state.companionTab === "chat" ||
  state.companionTab === "chunks"
    ? "runs"
    : state.companionTab,
```

In `src/lib/analysis-workspace-persistence.ts`, change:

```ts
return value === "evidence" || value === "chat" || value === "runs" ? value : null;
```

to:

```ts
return value === "evidence" || value === "chat" || value === "chunks" || value === "runs"
  ? value
  : null;
```

In `src/lib/analysis-run-workflow.ts`, remove this block from `handleRunEvent`:

```ts
if (payload.chunk_summary) {
  deps.patch({ inspectorMode: "chunks" });
}
```

- [ ] **Step 6: Verify and commit**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-workspace-state.test.ts src/lib/analysis-workspace-persistence.test.ts src/lib/analysis-run-workflow.test.ts
npm.cmd run check
git diff --check
```

Commit:

```powershell
git add src/lib/analysis-workspace-state.ts src/lib/analysis-workspace-persistence.ts src/lib/analysis-run-workflow.ts src/lib/analysis-workspace-state.test.ts src/lib/analysis-workspace-persistence.test.ts src/lib/analysis-run-workflow.test.ts
git commit -m "feat: add chunks companion state"
```

Append a recovery checkpoint to `reference/session-context-2026-05-10-analysis-redesign.md`.

---

### Task 2: Chunks Companion Tab UI

**Files:**
- Modify: `src/lib/components/analysis/run-companion-tabs.svelte`
- Modify: `src/lib/components/analysis/chunk-summaries.svelte`
- Test: `src/lib/analysis-run-companion-tabs.test.ts`

- [ ] **Step 1: Add failing component contract tests**

In `src/lib/analysis-run-companion-tabs.test.ts`, add an import:

```ts
import chunkSummariesSource from "./components/analysis/chunk-summaries.svelte?raw";
```

Change the first test name from:

```ts
it("uses accessible Evidence, Chat, and Runs tabs", () => {
```

to:

```ts
it("uses accessible Evidence, Chat, Chunks, and Runs tabs", () => {
```

Add expectations inside that test:

```ts
expect(companionTabsSource).toContain('onChangeCompanionTab("chunks")');
expect(companionTabsSource).toContain("<ChunkSummaries");
expect(companionTabsSource).toContain("chunkTabLabel");
expect(companionTabsSource).toContain("chunksDisabled");
```

Add a new test:

```ts
it("renders chunk summaries compactly inside the companion", () => {
  expect(companionTabsSource).toContain("focusedChunkSummaries");
  expect(companionTabsSource).toContain("selectedRunIsActive");
  expect(companionTabsSource).toContain('framed={false}');
  expect(chunkSummariesSource).toContain("framed = true");
  expect(chunkSummariesSource).toContain("terminalEmptyMessage");
  expect(chunkSummariesSource).toContain("Waiting for the first chunk summary.");
  expect(chunkSummariesSource).toContain("Chunk summaries are only available while the run is streaming.");
  expect(chunkSummariesSource).toContain("class:card={framed}");
});
```

- [ ] **Step 2: Run test and confirm RED**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-run-companion-tabs.test.ts
```

Expected: fails because `Chunks`, `ChunkSummaries`, `focusedChunkSummaries`, and compact rendering do not exist in `RunCompanionTabs`.

- [ ] **Step 3: Fetch Svelte docs before editing**

Run:

```text
mcp__svelte_server__.list_sections
```

Use docs only if the existing Svelte 5 syntax is unclear.

- [ ] **Step 4: Implement `ChunkSummaries` compact mode**

In `src/lib/components/analysis/chunk-summaries.svelte`, change props to:

```svelte
let {
  summaries,
  running,
  framed = true,
  waitingMessage = "Waiting for the first chunk summary.",
  terminalEmptyMessage = "Chunk summaries are only available while the run is streaming.",
}: {
  summaries: AnalysisChunkSummaryEvent[];
  running: boolean;
  framed?: boolean;
  waitingMessage?: string;
  terminalEmptyMessage?: string;
} = $props();
```

Change the outer markup from:

```svelte
{#if running || summaries.length > 0}
  <section class="card chunk-summaries">
```

to:

```svelte
<section class="chunk-summaries" class:card={framed}>
```

Change the empty-state branches to:

```svelte
{#if summaries.length === 0}
  <EmptyState description={running ? waitingMessage : terminalEmptyMessage} />
{:else}
```

Remove the closing `{/if}` that belonged to the old outer condition.

Update CSS:

```css
.card {
  background: var(--panel);
  border: 1px solid var(--border);
  box-shadow: var(--shadow);
  border-radius: 8px;
  padding: 1rem;
}

.chunk-summaries {
  display: flex;
  flex-direction: column;
  gap: 0.85rem;
}

.chunk-list {
  display: flex;
  flex-direction: column;
  gap: 0.65rem;
  max-height: 32rem;
  overflow: auto;
  padding-right: 0.25rem;
}

.chunk-item {
  padding: 0.75rem 0.8rem;
  background: var(--panel-strong);
  border: 1px solid var(--border);
  border-radius: 8px;
}
```

Keep the existing topic/notable/candidate ref rendering.

- [ ] **Step 5: Implement `Chunks` in `RunCompanionTabs`**

In `src/lib/components/analysis/run-companion-tabs.svelte`, add imports:

```svelte
import ChunkSummaries from "$lib/components/analysis/chunk-summaries.svelte";
import EmptyState from "$lib/components/ui/EmptyState.svelte";
import type { AnalysisChunkSummaryEvent, ... } from "$lib/types/analysis";
```

Add props after `selectedTrace`:

```ts
focusedChunkSummaries,
selectedRunIsActive,
```

and type them:

```ts
focusedChunkSummaries: AnalysisChunkSummaryEvent[];
selectedRunIsActive: boolean;
```

Add helper functions:

```ts
function chunkTabLabel() {
  const count = focusedChunkSummaries.length;
  if (count === 0) return "Chunks";
  const total = focusedChunkSummaries.at(-1)?.total ?? null;
  return total && total > 0 ? `Chunks ${count}/${total}` : `Chunks ${count}`;
}

function chunksDisabled() {
  return currentRun === null;
}
```

Add a tab button between Chat and Runs:

```svelte
<Button
  id={tabId("chunks")}
  role="tab"
  size="sm"
  variant="secondary"
  selected={companionTab === "chunks"}
  ariaSelected={companionTab === "chunks"}
  ariaControls="run-companion-panel"
  disabled={chunksDisabled()}
  title={chunksDisabled() ? "Open a run to inspect chunk summaries." : undefined}
  onclick={() => {
    if (!chunksDisabled()) onChangeCompanionTab("chunks");
  }}
>
  {chunkTabLabel()}
</Button>
```

Add a panel branch between Chat and Runs:

```svelte
{:else if companionTab === "chunks"}
  {#if currentRun}
    <ChunkSummaries
      summaries={focusedChunkSummaries}
      running={selectedRunIsActive}
      framed={false}
    />
  {:else}
    <EmptyState description="Open a run to inspect chunk summaries." />
  {/if}
```

- [ ] **Step 6: Autofix changed Svelte components**

Run Svelte autofixer with the actual component contents:

```text
mcp__svelte_server__.svelte_autofixer
filename: RunCompanionTabs.svelte
desired_svelte_version: 5
code: <full changed component>
```

and:

```text
mcp__svelte_server__.svelte_autofixer
filename: ChunkSummaries.svelte
desired_svelte_version: 5
code: <full changed component>
```

- [ ] **Step 7: Verify and commit**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-run-companion-tabs.test.ts
npm.cmd run check
git diff --check
```

Commit:

```powershell
git add src/lib/components/analysis/run-companion-tabs.svelte src/lib/components/analysis/chunk-summaries.svelte src/lib/analysis-run-companion-tabs.test.ts
git commit -m "feat: add chunks companion tab"
```

Append a recovery checkpoint to `reference/session-context-2026-05-10-analysis-redesign.md`.

---

### Task 3: Route Wiring And No Auto-Focus Contract

**Files:**
- Modify: `src/routes/analysis/+page.svelte`
- Modify: `src/lib/analysis-run-companion-route.test.ts`

- [ ] **Step 1: Add failing route contract tests**

In `src/lib/analysis-run-companion-route.test.ts`, add:

```ts
it("passes focused chunk summaries into the companion without auto-opening chunks", () => {
  expect(analysisPageSource).toContain("focusedRunChunkSummaries");
  expect(analysisPageSource).toContain("focusedChunkSummaries={focusedRunChunkSummaries(focusedLiveRun)}");
  expect(analysisPageSource).toContain("{selectedRunIsActive}");
  expect(analysisPageSource).not.toContain('companionTab: "chunks"');
});
```

Update the existing `uses workspaceUiState.companionTab as the only companion tab source` test to include:

```ts
expect(analysisPageSource).toContain("companionTab={workspaceUiState.companionTab}");
expect(analysisPageSource).not.toContain("inspectorMode: \"chunks\"");
```

- [ ] **Step 2: Run route test and confirm RED**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-run-companion-route.test.ts
```

Expected: fails because the route does not import `focusedRunChunkSummaries` and does not pass `focusedChunkSummaries` into `RunCompanionTabs`.

- [ ] **Step 3: Fetch Svelte docs before editing**

Run:

```text
mcp__svelte_server__.list_sections
```

- [ ] **Step 4: Wire route data**

In `src/routes/analysis/+page.svelte`, add `focusedRunChunkSummaries` to the existing `$lib/analysis-state` import list near `focusedLiveRunState`:

```ts
focusedLiveRunState,
focusedRunChunkSummaries,
focusedRunStreamedOutput,
```

In the `<RunCompanionTabs ... />` props, add after `{selectedTrace}`:

```svelte
focusedChunkSummaries={focusedRunChunkSummaries(focusedLiveRun)}
{selectedRunIsActive}
```

Do not add any `companionTab: "chunks"` patch in run event handling.

- [ ] **Step 5: Autofix route component**

Run Svelte autofixer with the actual changed `+page.svelte` contents:

```text
mcp__svelte_server__.svelte_autofixer
filename: +page.svelte
desired_svelte_version: 5
code: <full changed component>
```

- [ ] **Step 6: Verify and commit**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-run-companion-route.test.ts src/lib/analysis-run-companion-tabs.test.ts
npm.cmd run check
git diff --check
```

Commit:

```powershell
git add src/routes/analysis/+page.svelte src/lib/analysis-run-companion-route.test.ts
git commit -m "feat: wire chunk summaries into companion"
```

Append a recovery checkpoint to `reference/session-context-2026-05-10-analysis-redesign.md`.

---

### Task 4: Full Verification And Evidence

**Files:**
- Modify: `docs/superpowers/plans/2026-05-14-analysis-chunk-stream-placement.md`

- [ ] **Step 1: Run targeted tests**

Run:

```powershell
npm.cmd test -- --run src/lib/analysis-workspace-state.test.ts src/lib/analysis-workspace-persistence.test.ts src/lib/analysis-run-workflow.test.ts src/lib/analysis-run-companion-tabs.test.ts src/lib/analysis-run-companion-route.test.ts
```

- [ ] **Step 2: Run full frontend verification**

Run:

```powershell
npm.cmd run check
npm.cmd test -- --run
git diff --check
```

- [ ] **Step 3: Runtime smoke if app bridge is available**

Try:

```text
mcp__tauri__.driver_session action=start port=9223
```

If available, verify:

- `RunCompanionTabs` shows `Evidence | Chat | Chunks | Runs`.
- A running analysis run can receive chunk summaries without switching away from the current tab.
- `Chunks` shows a count when summaries exist.
- Opening `Chunks` shows the chunk stream.
- A terminal opened run without in-memory summaries shows the terminal empty state.

If no running app or fixture state is available, record the exact skipped part.

- [ ] **Step 4: Record verification evidence and commit**

Append a `## Verification Evidence` section to this plan with exact commands and pass/fail counts.

Commit:

```powershell
git add docs/superpowers/plans/2026-05-14-analysis-chunk-stream-placement.md
git commit -m "docs: record chunk stream placement verification"
```

Append a final recovery checkpoint to `reference/session-context-2026-05-10-analysis-redesign.md`.

---

## Plan Self-Review

Spec coverage:

- Dedicated `Chunks` tab: Task 2.
- No automatic focus stealing: Tasks 1 and 3.
- Count/progress indicator: Task 2.
- Running and terminal empty states: Task 2.
- No persistence for chunk summaries: preserved by using existing live state only.
- Route data flow from live run state into companion: Task 3.
- Verification evidence: Task 4.

Placeholder scan:

- No TBD/TODO/fill-in placeholders are intentionally left.
- Autofixer steps require the executor to paste actual full component contents because the tool call cannot be embedded literally in a markdown plan.

Type consistency:

- `focusedChunkSummaries` is consistently typed as `AnalysisChunkSummaryEvent[]`.
- `selectedRunIsActive` is consistently typed as `boolean`.
- `CompanionTab` consistently includes `"chunks"`.
