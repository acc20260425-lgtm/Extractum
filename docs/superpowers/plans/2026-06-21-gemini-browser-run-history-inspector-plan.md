# Gemini Browser Run History Inspector Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the Gemini Browser Provider's passive recent-runs list into a clickable, filterable run history that drives the existing inline `Run inspector`.

**Architecture:** Keep the implementation frontend-only. Add pure run-history derivation helpers next to the existing run-inspector helper, then wire the Svelte Settings panel so the history list becomes the master selection surface and the current inspector remains the detail pane.

**Tech Stack:** Svelte 5 runes, TypeScript, Vitest source/helper tests, existing Tauri `gemini_bridge_list_runs` and run DTOs.

---

## File Structure

- Modify: `src/lib/gemini-browser-run-inspector.ts`
  - Add pure run-history filter, row derivation, badge, and selection helpers.
  - Keep existing copy diagnostics and artifact helper behavior intact.
- Modify: `src/lib/gemini-browser-run-inspector.test.ts`
  - Add helper tests for filters, partial-risk classification, row summaries, old DTO tolerance, and selection preservation.
- Modify: `src/lib/components/settings/gemini-browser-provider-panel.svelte`
  - Replace passive `Recent browser runs` rows with filter buttons and clickable history rows.
  - Keep existing `Run inspector`, `Copy diagnostics`, and `Open run folder` behavior pointed at the selected history run.
- Modify: `src/lib/gemini-browser-provider-panel.test.ts`
  - Add source-contract assertions for filter labels, clickable rows, selection state, and helper usage.
- Modify: `docs/browser-providers-llm-troubleshooting.md`
  - Briefly document how agents/operators should use Run History before opening artifacts manually.

No Rust, sidecar, or protocol DTO changes are required.

---

### Task 1: Run History Helper Model

**Files:**
- Modify: `src/lib/gemini-browser-run-inspector.ts`
- Modify: `src/lib/gemini-browser-run-inspector.test.ts`

- [x] **Step 1: Write failing helper tests**

Append these imports in `src/lib/gemini-browser-run-inspector.test.ts`:

```ts
import {
  filterRunHistoryRows,
  runHistoryRow,
  selectRunForHistory,
  type GeminiBrowserRunHistoryFilter,
} from "./gemini-browser-run-inspector";
```

If the file already has a grouped import from `./gemini-browser-run-inspector`, merge these names into that existing import instead of creating a duplicate import.

Append this test block inside the existing `describe("gemini browser run inspector", () => { ... })`:

```ts
  function runWithResult(
    runId: string,
    runStatus: GeminiBrowserRun["status"],
    resultOverrides: Partial<GeminiBrowserRunResult> = {},
  ): GeminiBrowserRun {
    return run({
      run_id: runId,
      status: runStatus,
      prompt_preview: `${runId} prompt`,
      updated_at: `2026-06-21T00:00:${runId.length.toString().padStart(2, "0")}Z`,
      result: result({ run_id: runId, ...resultOverrides }),
    });
  }

  it("derives compact history rows without exposing answer text or artifact paths", () => {
    const row = runHistoryRow(
      runWithResult("stable-run", "ok", {
        elapsed_ms: 24_660,
        text: "full answer text",
        debug_summary: {
          ...result().debug_summary!,
          answer_completion_reason: "stable",
          final_text_length: 16,
        },
      }),
    );

    expect(row.run.run_id).toBe("stable-run");
    expect(row.status).toBe("ok");
    expect(row.badge).toBe("stable");
    expect(row.isProblem).toBe(false);
    expect(row.isPartialRisk).toBe(false);
    expect(row.elapsedMs).toBe(24_660);
    expect(row.resultTextLength).toBe(16);
    expect(row.answerCompletionReason).toBe("stable");
  });

  it("classifies partial risk, manual action, failed, running, queued, and stable rows", () => {
    const partial = runWithResult("partial", "ok", {
      debug_summary: {
        ...result().debug_summary!,
        answer_completion_reason: "timeout_latest",
      },
    });
    const manual = runWithResult("manual", "needs_manual_action", {
      status: "needs_manual_action",
      manual_action: "start_chrome_cdp",
    });
    const failed = runWithResult("failed", "failed", { status: "failed" });
    const blocked = runWithResult("blocked", "blocked", { status: "blocked" });
    const running = run({ run_id: "running", status: "running", result: null });
    const queued = run({ run_id: "queued", status: "queued", result: null });
    const stable = runWithResult("stable", "ok");

    expect(runHistoryRow(partial).badge).toBe("partial");
    expect(runHistoryRow(partial).isProblem).toBe(true);
    expect(runHistoryRow(manual).badge).toBe("manual");
    expect(runHistoryRow(manual).isProblem).toBe(true);
    expect(runHistoryRow(failed).badge).toBe("failed");
    expect(runHistoryRow(blocked).badge).toBe("failed");
    expect(runHistoryRow(running).badge).toBe("running");
    expect(runHistoryRow(queued).badge).toBe("queued");
    expect(runHistoryRow(stable).badge).toBe("stable");
  });

  it("filters history rows by operator-focused buckets", () => {
    const runs: GeminiBrowserRun[] = [
      runWithResult("stable", "ok"),
      runWithResult("partial", "ok", {
        debug_summary: {
          ...result().debug_summary!,
          answer_completion_reason: "timeout_latest",
        },
      }),
      runWithResult("manual", "needs_manual_action", {
        status: "needs_manual_action",
        manual_action: "login",
      }),
      runWithResult("timeout", "timeout", { status: "timeout" }),
      run({ run_id: "running", status: "running", result: null }),
    ];

    const ids = (filter: GeminiBrowserRunHistoryFilter) =>
      filterRunHistoryRows(runs, filter).map((row) => row.run.run_id);

    expect(ids("all")).toEqual(["stable", "partial", "manual", "timeout", "running"]);
    expect(ids("problems")).toEqual(["partial", "manual", "timeout"]);
    expect(ids("partial_risk")).toEqual(["partial"]);
    expect(ids("manual_action")).toEqual(["manual"]);
    expect(ids("failed")).toEqual(["timeout"]);
  });

  it("preserves selected history run across refresh and falls back within the visible filter", () => {
    const newest = runWithResult("newest", "ok");
    const active = run({ run_id: "active", status: "running", result: null });
    const partial = runWithResult("partial", "ok", {
      debug_summary: {
        ...result().debug_summary!,
        answer_completion_reason: "timeout_latest",
      },
    });
    const failed = runWithResult("failed", "failed", { status: "failed" });
    const runs = [newest, active, partial, failed];

    expect(selectRunForHistory(runs, "active", null, "all")?.run_id).toBe("active");
    expect(selectRunForHistory(runs, "active", "partial", "all")?.run_id).toBe("partial");
    expect(selectRunForHistory(runs, "active", "partial", "partial_risk")?.run_id).toBe(
      "partial",
    );
    expect(selectRunForHistory(runs, "active", "newest", "partial_risk")?.run_id).toBe(
      "partial",
    );
    expect(selectRunForHistory(runs, null, "missing", "failed")?.run_id).toBe("failed");
    expect(selectRunForHistory([], null, "missing", "all")).toBeNull();
  });

  it("tolerates old run records without debug summary", () => {
    const oldRun = run({
      run_id: "old",
      status: "ok",
      result: result({ debug_summary: null, text: null }),
    });

    const row = runHistoryRow(oldRun);

    expect(row.badge).toBe("ok");
    expect(row.answerCompletionReason).toBeNull();
    expect(row.resultTextLength).toBe(0);
  });
```

- [x] **Step 2: Run helper tests and verify they fail**

Run:

```powershell
npm.cmd run test -- --run src/lib/gemini-browser-run-inspector.test.ts
```

Expected: FAIL with TypeScript/runtime errors for missing exports such as `filterRunHistoryRows`, `runHistoryRow`, and `selectRunForHistory`.

- [x] **Step 3: Implement pure helper functions**

In `src/lib/gemini-browser-run-inspector.ts`, replace the first import with:

```ts
import type {
  GeminiBrowserAnswerCompletionReason,
  GeminiBrowserRun,
  GeminiBrowserRunResult,
  GeminiBrowserRunStatus,
} from "./types/gemini-browser";
```

Add these exports after `selectedRunForInspector(...)`:

```ts
export type GeminiBrowserRunHistoryFilter =
  | "all"
  | "problems"
  | "partial_risk"
  | "manual_action"
  | "failed";

export type GeminiBrowserRunHistoryBadge =
  | "ok"
  | "stable"
  | "partial"
  | "manual"
  | "failed"
  | "running"
  | "queued";

export interface GeminiBrowserRunHistoryRow {
  run: GeminiBrowserRun;
  status: GeminiBrowserRunStatus;
  badge: GeminiBrowserRunHistoryBadge;
  isProblem: boolean;
  isPartialRisk: boolean;
  elapsedMs: number | null;
  resultTextLength: number;
  answerCompletionReason: GeminiBrowserAnswerCompletionReason | null;
}

const FAILED_RUN_STATUSES = new Set<GeminiBrowserRunStatus>([
  "failed",
  "timeout",
  "browser_crashed",
  "blocked",
]);

const MANUAL_ACTION_STATUSES = new Set<GeminiBrowserRunStatus>([
  "needs_login",
  "needs_manual_action",
]);

export function effectiveRunStatus(run: GeminiBrowserRun): GeminiBrowserRunStatus {
  return run.result?.status ?? run.status;
}

function isFailedHistoryRun(run: GeminiBrowserRun): boolean {
  return FAILED_RUN_STATUSES.has(run.status) || FAILED_RUN_STATUSES.has(effectiveRunStatus(run));
}

function isManualActionHistoryRun(run: GeminiBrowserRun): boolean {
  return (
    MANUAL_ACTION_STATUSES.has(run.status) ||
    MANUAL_ACTION_STATUSES.has(effectiveRunStatus(run)) ||
    Boolean(run.result?.manual_action)
  );
}

export function runHistoryRow(run: GeminiBrowserRun): GeminiBrowserRunHistoryRow {
  const status = effectiveRunStatus(run);
  const isPartialRisk = isPartialRiskBrowserResult(run.result);
  const isManualAction = isManualActionHistoryRun(run);
  const isFailed = isFailedHistoryRun(run);
  const answerCompletionReason = run.result?.debug_summary?.answer_completion_reason ?? null;
  const isProblem = isPartialRisk || isManualAction || isFailed;
  let badge: GeminiBrowserRunHistoryBadge = "ok";

  if (run.status === "queued") {
    badge = "queued";
  } else if (run.status === "running" || status === "running") {
    badge = "running";
  } else if (isPartialRisk) {
    badge = "partial";
  } else if (isManualAction) {
    badge = "manual";
  } else if (isFailed) {
    badge = "failed";
  } else if (answerCompletionReason === "stable") {
    badge = "stable";
  }

  return {
    run,
    status,
    badge,
    isProblem,
    isPartialRisk,
    elapsedMs: run.result?.elapsed_ms ?? null,
    resultTextLength: resultTextLength(run.result),
    answerCompletionReason,
  };
}

export function filterRunHistoryRows(
  runs: GeminiBrowserRun[],
  filter: GeminiBrowserRunHistoryFilter,
): GeminiBrowserRunHistoryRow[] {
  const rows = runs.map(runHistoryRow);
  if (filter === "all") return rows;
  return rows.filter((row) => {
    if (filter === "problems") return row.isProblem;
    if (filter === "partial_risk") return row.isPartialRisk;
    if (filter === "manual_action") return isManualActionHistoryRun(row.run);
    if (filter === "failed") return isFailedHistoryRun(row.run);
    return true;
  });
}

export function selectRunForHistory(
  runs: GeminiBrowserRun[],
  activeRunId: string | null,
  selectedRunId: string | null,
  filter: GeminiBrowserRunHistoryFilter,
): GeminiBrowserRun | null {
  const visibleRows = filterRunHistoryRows(runs, filter);
  if (visibleRows.length === 0) return null;

  if (selectedRunId) {
    const selected = visibleRows.find((row) => row.run.run_id === selectedRunId);
    if (selected) return selected.run;
  }

  if (activeRunId) {
    const active = visibleRows.find((row) => row.run.run_id === activeRunId);
    if (active) return active.run;
  }

  return visibleRows[0]?.run ?? null;
}
```

Keep the existing `selectedRunForInspector(...)` export for backwards-compatible tests and any existing call sites until Task 2 rewires the panel.

- [x] **Step 4: Run helper tests and verify they pass**

Run:

```powershell
npm.cmd run test -- --run src/lib/gemini-browser-run-inspector.test.ts
```

Expected: PASS, including the new run-history helper tests.

- [x] **Step 5: Commit Task 1**

Run:

```powershell
git add src/lib/gemini-browser-run-inspector.ts src/lib/gemini-browser-run-inspector.test.ts
git commit -m "feat: derive Gemini browser run history rows"
```

---

### Task 2: Settings Panel Run History UI

**Files:**
- Modify: `src/lib/components/settings/gemini-browser-provider-panel.svelte`
- Modify: `src/lib/gemini-browser-provider-panel.test.ts`

- [x] **Step 1: Write failing Svelte source-contract tests**

Append this test in `src/lib/gemini-browser-provider-panel.test.ts`:

```ts
  it("renders run history filters and selectable rows for the inline inspector", () => {
    expect(componentSource).toContain("Run history");
    expect(componentSource).toContain("runHistoryFilter");
    expect(componentSource).toContain("filterRunHistoryRows");
    expect(componentSource).toContain("selectRunForHistory");
    expect(componentSource).toContain('data-filter="all"');
    expect(componentSource).toContain('data-filter="problems"');
    expect(componentSource).toContain('data-filter="partial_risk"');
    expect(componentSource).toContain('data-filter="manual_action"');
    expect(componentSource).toContain('data-filter="failed"');
    expect(componentSource).toContain("selectHistoryRun(row.run.run_id)");
    expect(componentSource).toContain("class:selected={selectedInspectorRun?.run_id === row.run.run_id}");
    expect(componentSource).toContain("row.badge");
    expect(componentSource).toContain("row.answerCompletionReason");
    expect(componentSource).not.toContain("{run.prompt_preview}</p>");
  });
```

- [x] **Step 2: Run panel source tests and verify they fail**

Run:

```powershell
npm.cmd run test -- --run src/lib/gemini-browser-provider-panel.test.ts
```

Expected: FAIL because the panel still contains passive `Recent browser runs` rows and does not import/use run-history helpers.

- [x] **Step 3: Wire selection state and derived rows**

In `src/lib/components/settings/gemini-browser-provider-panel.svelte`, update the helper import from `$lib/gemini-browser-run-inspector` to include the new helpers and type:

```ts
  import {
    artifactAvailability,
    copyableRunDiagnostics,
    debugFinalTextLength,
    filterRunHistoryRows,
    isPartialRiskBrowserResult,
    resultTextLength,
    sanitizeDiagnosticMessage,
    selectRunForHistory,
    type GeminiBrowserRunHistoryFilter,
  } from "$lib/gemini-browser-run-inspector";
```

Remove `selectedRunForInspector` from the import.

Add state after `let inspectorMessage = $state("");`:

```ts
  let runHistoryFilter = $state<GeminiBrowserRunHistoryFilter>("all");
  let selectedHistoryRunId = $state<string | null>(null);
```

Replace the current selected inspector derivation:

```ts
  const selectedInspectorRun = $derived(selectedRunForInspector(runs, activeTestRunId));
```

with:

```ts
  const activeInspectorRunId = $derived(activeTestRunId ?? status?.active_run_id ?? null);
  const runHistoryRows = $derived(filterRunHistoryRows(runs, runHistoryFilter));
  const selectedInspectorRun = $derived(
    selectRunForHistory(runs, activeInspectorRunId, selectedHistoryRunId, runHistoryFilter),
  );
```

Add these functions near the existing UI helper functions:

```ts
  function selectRunHistoryFilter(filter: GeminiBrowserRunHistoryFilter) {
    runHistoryFilter = filter;
  }

  function selectHistoryRun(runId: string) {
    selectedHistoryRunId = runId;
    inspectorMessage = "";
  }

  function historyFilterLabel(filter: GeminiBrowserRunHistoryFilter) {
    if (filter === "all") return "All";
    if (filter === "problems") return "Problems";
    if (filter === "partial_risk") return "Partial risk";
    if (filter === "manual_action") return "Manual action";
    return "Failed";
  }

  function formatRunUpdatedAt(value: string) {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return value;
    return date.toLocaleString();
  }

  function formatRunElapsed(ms: number | null) {
    if (ms === null) return "pending";
    return `${ms} ms`;
  }
```

In `sendTestPrompt()`, after `activeTestRunId = runId;`, add:

```ts
    selectedHistoryRunId = runId;
```

This ensures the new run remains selected once it appears in the refreshed run log.

- [x] **Step 4: Replace passive recent-runs markup with filterable history**

In `src/lib/components/settings/gemini-browser-provider-panel.svelte`, replace the entire block:

```svelte
  <div class="runs-list">
    <h3>Recent browser runs</h3>
    {#each runs as run (run.run_id)}
      <div class="run-row">
        <span>{run.status}</span>
        <code>{run.run_id}</code>
        <p>{run.prompt_preview}</p>
      </div>
    {:else}
      <p class="empty">No browser runs yet.</p>
    {/each}
  </div>
```

with:

```svelte
  <section class="runs-list" aria-label="Run history">
    <div class="row history-head">
      <div>
        <h3>Run history</h3>
        <p>Choose a Browser Provider run to inspect.</p>
      </div>
      <div class="history-filters" aria-label="Run history filters">
        <button
          type="button"
          data-filter="all"
          class:active={runHistoryFilter === "all"}
          onclick={() => selectRunHistoryFilter("all")}
        >
          {historyFilterLabel("all")}
        </button>
        <button
          type="button"
          data-filter="problems"
          class:active={runHistoryFilter === "problems"}
          onclick={() => selectRunHistoryFilter("problems")}
        >
          {historyFilterLabel("problems")}
        </button>
        <button
          type="button"
          data-filter="partial_risk"
          class:active={runHistoryFilter === "partial_risk"}
          onclick={() => selectRunHistoryFilter("partial_risk")}
        >
          {historyFilterLabel("partial_risk")}
        </button>
        <button
          type="button"
          data-filter="manual_action"
          class:active={runHistoryFilter === "manual_action"}
          onclick={() => selectRunHistoryFilter("manual_action")}
        >
          {historyFilterLabel("manual_action")}
        </button>
        <button
          type="button"
          data-filter="failed"
          class:active={runHistoryFilter === "failed"}
          onclick={() => selectRunHistoryFilter("failed")}
        >
          {historyFilterLabel("failed")}
        </button>
      </div>
    </div>

    {#each runHistoryRows as row (row.run.run_id)}
      <button
        type="button"
        class="run-row"
        class:selected={selectedInspectorRun?.run_id === row.run.run_id}
        class:warning={row.isProblem}
        onclick={() => selectHistoryRun(row.run.run_id)}
      >
        <span class="run-status">{row.status}</span>
        <span class="run-badge">{row.badge}</span>
        <span class="run-preview">{row.run.prompt_preview || "No prompt preview"}</span>
        <span class="run-meta">{formatRunUpdatedAt(row.run.updated_at)}</span>
        <span class="run-meta">{formatRunElapsed(row.elapsedMs)}</span>
        <span class="run-meta">{row.resultTextLength} chars</span>
        <span class="run-meta">{row.answerCompletionReason ?? "no debug"}</span>
      </button>
    {:else}
      <p class="empty">No browser runs match this filter.</p>
    {/each}
  </section>
```

- [x] **Step 5: Update CSS for history rows**

In the component `<style>` block, replace this selector:

```css
  .provider-card button,
  .run-inspector button {
```

with:

```css
  .provider-card button,
  .run-inspector button,
  .history-filters button {
```

Replace the existing `.runs-list`, `.run-row`, `.run-row span`, and `.run-row p` styles with:

```css
  .runs-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px;
    background: var(--card);
  }

  .history-head {
    justify-content: space-between;
    align-items: flex-start;
  }

  .history-head h3 {
    margin: 0;
    font-size: 16px;
  }

  .history-filters {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    justify-content: flex-end;
  }

  .history-filters button.active {
    background: var(--accent);
    color: var(--accent-foreground);
  }

  .run-row {
    display: grid;
    grid-template-columns: minmax(90px, 0.8fr) minmax(78px, 0.7fr) minmax(220px, 2fr) repeat(4, minmax(88px, 1fr));
    gap: 8px;
    align-items: center;
    width: 100%;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 8px;
    background: var(--background);
    color: var(--foreground);
    text-align: left;
  }

  .run-row.selected {
    outline: 2px solid color-mix(in srgb, var(--accent) 65%, transparent);
    outline-offset: 1px;
  }

  .run-status,
  .run-badge {
    font-weight: 700;
  }

  .run-badge {
    justify-self: start;
    border: 1px solid var(--border);
    border-radius: 999px;
    padding: 2px 7px;
    font-size: 11px;
  }

  .run-preview,
  .run-meta {
    min-width: 0;
    overflow-wrap: anywhere;
  }

  .run-meta {
    color: var(--muted-foreground);
    font-size: 12px;
  }
```

In the existing media query, add `.run-row`:

```css
  @media (max-width: 820px) {
    .provider-grid,
    .inspector-grid,
    .inspector-grid.compact,
    .run-row {
      grid-template-columns: 1fr;
    }
  }
```

- [x] **Step 6: Run panel tests and Svelte check**

Run:

```powershell
npm.cmd run test -- --run src/lib/gemini-browser-provider-panel.test.ts src/lib/gemini-browser-run-inspector.test.ts
npm.cmd run check
```

Expected:

- Vitest PASS for both files.
- `svelte-check found 0 errors and 0 warnings`.

- [x] **Step 7: Commit Task 2**

Run:

```powershell
git add src/lib/components/settings/gemini-browser-provider-panel.svelte src/lib/gemini-browser-provider-panel.test.ts
git commit -m "feat: add Gemini browser run history filters"
```

---

### Task 3: Documentation And Final Verification

**Files:**
- Modify: `docs/browser-providers-llm-troubleshooting.md`

- [x] **Step 1: Document the new history workflow**

In `docs/browser-providers-llm-troubleshooting.md`, find the `## Inline Run Inspector` section. After the paragraph that starts with `The inspector shows status`, insert:

```md
The `Run history` list below the inspector is the first place to compare
multiple attempts. Use the filters before opening artifact folders:

- `Problems` shows failed, blocked, timeout, browser-crashed, manual-action,
  login, and partial-risk runs.
- `Partial risk` isolates `ok + timeout_latest` results that should not be fed
  into prompt-pack automation as normal completions.
- `Manual action` isolates runs that need login, account selection, Chrome CDP
  setup, consent, CAPTCHA, or another operator step.
- `Failed` isolates failed, timeout, blocked, and browser-crashed runs.

Clicking a history row drives the inline inspector. `Copy diagnostics` and
`Open run folder` always operate on the selected history run, not necessarily
the newest run.
```

- [x] **Step 2: Run final verification**

Run:

```powershell
npm.cmd run test -- --run src/lib/gemini-browser-run-inspector.test.ts src/lib/gemini-browser-provider-panel.test.ts
npm.cmd run check
git diff --check
```

Expected:

- Vitest PASS for both test files.
- `svelte-check found 0 errors and 0 warnings`.
- `git diff --check` exits 0.

- [x] **Step 3: Manual validation in running app**

With `npm.cmd run tauri dev` already running, validate manually:

1. Open Settings -> Browser Providers.
2. Confirm the section title is `Run history`, not `Recent browser runs`.
3. Send the short prompt:

```text
Reply with one short sentence confirming the browser provider is connected.
```

4. Confirm the new run appears in history and is selected.
5. Confirm the row shows `stable` for a stable result, `partial` for a `timeout_latest` result, `manual` for a manual-action result, or `failed` for a timeout/blocked/browser-crashed/failed result.
6. Click another older run and confirm `Run inspector` changes to that run ID.
7. Click `Problems`, `Partial risk`, `Manual action`, and `Failed` filters and confirm each filter changes the visible rows without exposing answer text.
8. Select a visible run and click `Copy diagnostics`; confirm the copied diagnostics use the selected run ID.
9. If the selected run has `artifact_run_dir_available: true`, click `Open run folder` and confirm the folder opens for that selected run.

- [x] **Step 4: Commit Task 3**

Run:

```powershell
git add docs/browser-providers-llm-troubleshooting.md
git commit -m "docs: document Gemini run history workflow"
```

---

## Final Checklist

- [x] Helper tests cover row derivation, filters, selection preservation, and old DTO tolerance.
- [x] Source-contract tests cover filter labels, selectable rows, selected state, and inspector actions.
- [x] `Run inspector` uses the selected history run.
- [x] `Copy diagnostics` and `Open run folder` operate on the selected run.
- [x] Partial-risk runs are visibly flagged.
- [x] History rows do not render full answer text, raw artifact paths, or copied diagnostics.
- [x] `npm.cmd run test -- --run src/lib/gemini-browser-run-inspector.test.ts src/lib/gemini-browser-provider-panel.test.ts` passes.
- [x] `npm.cmd run check` passes.
- [x] `git diff --check` passes.
