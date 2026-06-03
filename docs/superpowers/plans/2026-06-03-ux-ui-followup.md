# UX/UI Follow-Up Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the next visible UX issues found in the live app after the first UX/UI improvement merge.

**Architecture:** Keep the existing route/component ownership. Add focused raw-source and view-model tests first, then make small layout changes in Analysis, Diagnostics, and Accounts. Do not change backend behavior, data loading semantics, database code, or Tauri commands.

**Tech Stack:** Svelte 5, SvelteKit, Tauri 2, Vitest raw-source contract tests, existing `$lib/components/ui/*` primitives, `npm.cmd run test`, `npm.cmd run check`, `npm.cmd run smoke:analysis`.

---

## Live Audit Summary

Observed in the running app on 2026-06-03 after merge to `main`:

- `/analysis` source mode: the top chrome is now compact, but the Telegram timeline expands the whole document. At 1280x860 the report canvas measured about `39k px` tall and the run companion moved below the main source reader because the three-column layout collapses at `1500px`.
- `/settings`: the page is usable after the first pass. Keep it out of this follow-up unless implementation work reveals a regression.
- `/diagnostics`: the `Only issues / All tables` controls render below the large summary grid, so the problem-first control is not visible in the first viewport. `Only issues` filters tables, but not healthy rows inside a table that contains some issue rows.
- `/accounts`: `YoutubeSettingsPanel embedded` visually sits inside `youtube-access-shell`, but its root still carries `desk-panel desk-panel-subtle`, creating a card-inside-card structure. The first viewport confirms the nested panel feel.
- Route drift note: manual `history.pushState` with `/settings` can make URL/nav state say Settings while Analysis content remains mounted. This is not normal user navigation, so it is not a task here, but future smoke helpers should use real navigation clicks or SvelteKit `goto`.

## Scope

Do:

- Bound the Analysis source reader so long timelines scroll inside the work surface.
- Keep the run companion visible at normal desktop widths around 1280px.
- Move Diagnostics table controls above the dense summary grid and filter rows in issue mode.
- Remove nested card styling from embedded YouTube settings.

Do not:

- Rewrite Analysis state management.
- Virtualize timelines in this pass.
- Change source loading page sizes or backend pagination.
- Redesign Settings model selection in this pass.
- Add new global UI primitives unless an existing component cannot express the behavior.

## File Structure

Expected new or modified tests:

- Modify: `src/lib/analysis-priority-ux-contract.test.ts`
- Modify: `src/lib/analysis-report-canvas.test.ts`
- Modify: `src/lib/analysis-ui-smoke-contract.test.ts`
- Modify: `src/lib/diagnostics-ux-contract.test.ts`
- Modify: `src/lib/diagnostics-view-model.test.ts`
- Modify: `src/lib/accounts-ux-contract.test.ts`
- Modify: `src/lib/source-access-placement.test.ts`

Expected component and route changes:

- Modify: `src/routes/analysis/+page.svelte`
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `src/lib/components/analysis/source-browser-shell.svelte`
- Modify: `src/lib/components/analysis/telegram-timeline-reader.svelte`
- Modify: `src/routes/diagnostics/+page.svelte`
- Modify: `src/lib/diagnostics-view-model.ts`
- Modify: `src/lib/components/diagnostics/DiagnosticCountTable.svelte`
- Modify: `src/routes/accounts/+page.svelte`
- Modify: `src/lib/components/settings/youtube-settings-panel.svelte`

---

### Task 1: Freeze The Follow-Up UX Contract

**Files:**
- Modify: `src/lib/analysis-priority-ux-contract.test.ts`
- Modify: `src/lib/analysis-report-canvas.test.ts`
- Modify: `src/lib/diagnostics-ux-contract.test.ts`
- Modify: `src/lib/diagnostics-view-model.test.ts`
- Modify: `src/lib/accounts-ux-contract.test.ts`
- Modify: `src/lib/source-access-placement.test.ts`

- [x] **Step 1: Add failing Analysis bounded-reader assertions**

In `src/lib/analysis-priority-ux-contract.test.ts`, add these imports:

```ts
import sourceBrowserShellSource from "./components/analysis/source-browser-shell.svelte?raw";
import telegramTimelineSource from "./components/analysis/telegram-timeline-reader.svelte?raw";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";
```

Add this test:

```ts
it("keeps long source readers bounded while preserving desktop companion visibility", () => {
  expect(analysisPageSource).toContain("minmax(21rem, clamp(22rem, 26vw, 26rem))");
  expect(analysisPageSource).toContain("@media (max-width: 1180px)");
  expect(sourceBrowserShellSource).toContain("bounded = false");
  expect(sourceBrowserShellSource).toContain("class:bounded={bounded}");
  expect(sourceBrowserShellSource).toContain('class="source-browser-body"');
  expect(sourceBrowserShellSource).toContain("max-height: calc(100vh - 13rem)");
  expect(sourceBrowserShellSource).toContain("overflow: auto");
  expect(telegramTimelineSource).toContain("scroll-margin-top");
});
```

- [x] **Step 2: Add failing source surface plumbing assertions**

In `src/lib/analysis-report-canvas.test.ts`, add:

```ts
it("passes bounded source browser mode only for live source canvas review", () => {
  expect(reportCanvasSource).toContain('sourceBrowserBounded={canvasMode === "source" && sourceViewBasis === "live_source"}');
  expect(reportSourceSurfaceSource).toContain("sourceBrowserBounded = false");
  expect(reportSourceSurfaceSource).toContain("bounded={sourceBrowserBounded}");
});
```

- [x] **Step 3: Add failing Diagnostics issue-row assertions**

In `src/lib/diagnostics-ux-contract.test.ts`, add this import:

```ts
import diagnosticsViewModelSource from "./diagnostics-view-model?raw";
```

Add:

```ts
it("puts diagnostics table controls before summary cards and filters issue rows", () => {
  const controlsIndex = diagnosticsPageSource.indexOf('class="diagnostics-table-controls"');
  const gridIndex = diagnosticsPageSource.indexOf('class="diagnostics-grid"');

  expect(controlsIndex).toBeGreaterThan(0);
  expect(gridIndex).toBeGreaterThan(0);
  expect(controlsIndex).toBeLessThan(gridIndex);
  expect(diagnosticsPageSource).toContain("visibleDiagnosticRows");
  expect(diagnosticsPageSource).toContain("diagnosticRowHasIssue");
  expect(diagnosticsViewModelSource).toContain("export function diagnosticRowHasIssue");
  expect(diagnosticsViewModelSource).toContain("export function filterDiagnosticIssueRows");
});
```

In `src/lib/diagnostics-view-model.test.ts`, add:

```ts
import { diagnosticRowHasIssue, filterDiagnosticIssueRows } from "$lib/diagnostics-view-model";

it("detects issue diagnostic rows without treating healthy rows as issues", () => {
  expect(diagnosticRowHasIssue({ status: "Failed", error: "Internal", count: 1 })).toBe(true);
  expect(diagnosticRowHasIssue({ status: "Completed", error: "None", count: 11 })).toBe(false);
});

it("filters diagnostic rows down to issue rows", () => {
  const rows = [
    { status: "Completed", error: "None", count: 11 },
    { status: "Failed", error: "Internal", count: 1 },
    { status: "Cancelled", completeness: "Partial", count: 4 },
  ];

  expect(filterDiagnosticIssueRows(rows)).toEqual([
    { status: "Failed", error: "Internal", count: 1 },
    { status: "Cancelled", completeness: "Partial", count: 4 },
  ]);
});
```

- [x] **Step 4: Add failing Accounts embedded-panel assertions**

In `src/lib/accounts-ux-contract.test.ts`, add:

```ts
it("does not render embedded YouTube settings as a nested desk panel", () => {
  expect(youtubeSettingsPanelSource).toContain('class={`youtube-settings-panel ${embedded ? "embedded" : "desk-panel desk-panel-subtle"}`.trim()}');
  expect(youtubeSettingsPanelSource).not.toContain('class="desk-panel desk-panel-subtle youtube-settings-panel" class:embedded');
});
```

In `src/lib/source-access-placement.test.ts`, add:

```ts
it("keeps embedded YouTube access visually inside one shell", () => {
  expect(accountsPageSource).toContain('<section class="desk-panel youtube-access-shell">');
  expect(accountsPageSource).toContain("<YoutubeSettingsPanel embedded />");
  expect(youtubeSettingsPanelSource).toContain('embedded ? "embedded" : "desk-panel desk-panel-subtle"');
});
```

- [x] **Step 5: Run the focused tests and verify they fail**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-priority-ux-contract.test.ts src/lib/analysis-report-canvas.test.ts src/lib/diagnostics-ux-contract.test.ts src/lib/diagnostics-view-model.test.ts src/lib/accounts-ux-contract.test.ts src/lib/source-access-placement.test.ts
```

Expected: FAIL because the new bounded reader, Diagnostics row filtering, and flattened embedded YouTube panel do not exist yet.

- [x] **Step 6: Commit the failing contract**

Run:

```powershell
git add src/lib/analysis-priority-ux-contract.test.ts src/lib/analysis-report-canvas.test.ts src/lib/diagnostics-ux-contract.test.ts src/lib/diagnostics-view-model.test.ts src/lib/accounts-ux-contract.test.ts src/lib/source-access-placement.test.ts
git commit -m "test: capture UX follow-up contract"
```

---

### Task 2: Bound Analysis Source Reading And Keep Companion Visible

**Files:**
- Modify: `src/routes/analysis/+page.svelte`
- Modify: `src/lib/components/analysis/report-canvas.svelte`
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `src/lib/components/analysis/source-browser-shell.svelte`
- Modify: `src/lib/components/analysis/telegram-timeline-reader.svelte`
- Test: `src/lib/analysis-priority-ux-contract.test.ts`
- Test: `src/lib/analysis-report-canvas.test.ts`
- Test: `src/lib/analysis-ui-smoke-contract.test.ts`

- [x] **Step 1: Add bounded prop to `SourceBrowserShell`**

In `src/lib/components/analysis/source-browser-shell.svelte`, add `bounded = false` to props:

```svelte
type Props = {
  bounded?: boolean;
  subject?: SourceBrowserSubject | null;
  sourceBrowserData?: SourceBrowserData | null;
  groupBrowserData?: SourceGroupBrowserData | null;
  snapshotBrowserData?: SnapshotBrowserData | null;
  selectedTraceRef?: string | null;
  highlightToken?: EvidenceHighlightToken | null;
  loadingItems?: boolean;
  formatTimestamp: (value: number | null) => string;
};

let {
  bounded = false,
  subject: explicitSubject = null,
  sourceBrowserData = null,
  groupBrowserData = null,
  snapshotBrowserData = null,
  selectedTraceRef = null,
  highlightToken = null,
  loadingItems = false,
  formatTimestamp,
}: Props = $props();
```

Change the outer section:

```svelte
<section class="source-browser-shell" class:bounded={bounded}>
```

- [x] **Step 2: Wrap active tab content in a bounded body**

In `source-browser-shell.svelte`, keep the existing `<nav>` where it is. Insert this opening wrapper immediately after the nav:

```svelte
<div class="source-browser-body">
```

Move the existing active-tab body chain inside that wrapper. Close the wrapper immediately after the final `{/if}` and immediately before `</section>`:

```svelte
</div>
```

Keep all current branch conditions and branch bodies unchanged except for indentation.

- [x] **Step 3: Add bounded reader CSS**

In `source-browser-shell.svelte`, add:

```css
.source-browser-body {
  min-width: 0;
}

.source-browser-shell.bounded {
  min-height: 0;
}

.source-browser-shell.bounded .source-browser-body {
  max-height: calc(100vh - 13rem);
  min-height: min(32rem, calc(100vh - 13rem));
  overflow: auto;
  padding-right: 0.15rem;
  overscroll-behavior: contain;
}

.source-browser-shell.bounded .source-browser-tabs {
  position: sticky;
  top: 0;
  z-index: 1;
  padding-bottom: 0.15rem;
  background: var(--bg);
}
```

- [x] **Step 4: Add scroll offset for timeline highlight jumps**

In `telegram-timeline-reader.svelte`, add:

```css
li {
  scroll-margin-top: 4.5rem;
}
```

Keep the existing `li` layout rules; add the new property inside the existing `li` rule.

- [x] **Step 5: Pass bounded mode from source surface**

In `report-source-surface.svelte`, add prop:

```svelte
sourceBrowserBounded?: boolean;
```

Default it:

```svelte
sourceBrowserBounded = false,
```

Pass it to every `SourceBrowserShell` call inside live source branches:

```svelte
<SourceBrowserShell
  bounded={sourceBrowserBounded}
  subject={{ kind: "source", source: currentSource }}
/>
```

For run snapshot branches, omit the `bounded` prop so they continue to use the default `false`.

- [x] **Step 6: Pass bounded mode from report canvas**

In `report-canvas.svelte`, pass:

```svelte
sourceBrowserBounded={canvasMode === "source" && sourceViewBasis === "live_source"}
```

on the `<ReportSourceSurface />` call.

- [x] **Step 7: Keep companion visible at normal desktop widths**

In `src/routes/analysis/+page.svelte`, change the grid columns from:

```css
grid-template-columns: minmax(4.25rem, 4.75rem) minmax(0, 1.45fr) minmax(420px, clamp(480px, 30vw, 560px));
```

to:

```css
grid-template-columns: minmax(4.25rem, 4.75rem) minmax(0, 1fr) minmax(21rem, clamp(22rem, 26vw, 26rem));
```

Change the collapse breakpoint from:

```css
@media (max-width: 1500px) {
```

to:

```css
@media (max-width: 1180px) {
```

Replace the existing single-column `@media (max-width: 1180px)` block with this lower breakpoint:

```css
@media (max-width: 900px) {
  .analysis-workspace {
    grid-template-columns: 1fr;
  }

  .companion-slot {
    grid-column: 1;
  }
}
```

- [x] **Step 8: Run focused Analysis tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-priority-ux-contract.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-source-readers.test.ts src/lib/analysis-ui-smoke-contract.test.ts
```

Expected: PASS.

- [x] **Step 9: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

- [x] **Step 10: Inspect the running app**

In the running app, open `/analysis` at about `1280x860` and verify:

- Source mode shows the source reader in a bounded scroll area.
- The run companion is visible in the right column on the first viewport.
- Timeline tabs remain visible while scrolling the source body.
- The document body no longer grows to tens of thousands of pixels for a single loaded source page.

- [x] **Step 11: Commit**

Run:

```powershell
git add src/routes/analysis/+page.svelte src/lib/components/analysis/report-canvas.svelte src/lib/components/analysis/report-source-surface.svelte src/lib/components/analysis/source-browser-shell.svelte src/lib/components/analysis/telegram-timeline-reader.svelte src/lib/analysis-priority-ux-contract.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-ui-smoke-contract.test.ts
git commit -m "feat(analysis): bound live source reading"
```

---

### Task 3: Make Diagnostics Issue Mode Truly Problem-First

**Files:**
- Modify: `src/lib/diagnostics-view-model.ts`
- Modify: `src/lib/diagnostics-view-model.test.ts`
- Modify: `src/routes/diagnostics/+page.svelte`
- Modify: `src/lib/components/diagnostics/DiagnosticCountTable.svelte`
- Test: `src/lib/diagnostics-ux-contract.test.ts`
- Test: `src/lib/diagnostics-route-contract.test.ts`

- [ ] **Step 1: Add issue row helpers to diagnostics view model**

In `src/lib/diagnostics-view-model.ts`, add:

```ts
const diagnosticIssuePattern = /failed|error|missing|unavailable|pending|warning|partial|cancelled/i;

export function diagnosticRowHasIssue(row: Record<string, string | number>) {
  return Object.entries(row).some(([key, value]) => {
    if (key.toLowerCase() === "count") return false;
    return diagnosticIssuePattern.test(String(value));
  });
}

export function filterDiagnosticIssueRows<T extends Record<string, string | number>>(rows: T[]) {
  return rows.filter((row) => diagnosticRowHasIssue(row));
}
```

- [ ] **Step 2: Run helper tests and verify they pass**

Run:

```powershell
npm.cmd run test -- src/lib/diagnostics-view-model.test.ts
```

Expected: PASS after helper implementation.

- [ ] **Step 3: Import helpers in diagnostics route**

In `src/routes/diagnostics/+page.svelte`, add to the existing `$lib/diagnostics-view-model` import:

```ts
diagnosticRowHasIssue,
filterDiagnosticIssueRows,
```

Remove the route-local `hasDiagnosticIssue` function or replace its internals with the helper:

```ts
function hasDiagnosticIssue(rows: Record<string, string | number>[]) {
  return rows.some(diagnosticRowHasIssue);
}
```

Add:

```ts
function visibleDiagnosticRows<T extends Record<string, string | number>>(rows: T[]) {
  return diagnosticsTableMode === "issues" ? filterDiagnosticIssueRows(rows) : rows;
}
```

- [ ] **Step 4: Move diagnostics table controls above summary cards**

In `diagnostics/+page.svelte`, move:

```svelte
<div class="diagnostics-table-controls" aria-label="Diagnostics table display">
  <Button size="sm" variant="secondary" selected={diagnosticsTableMode === "issues"} onclick={() => (diagnosticsTableMode = "issues")}>Only issues</Button>
  <Button size="sm" variant="secondary" selected={diagnosticsTableMode === "all"} onclick={() => (diagnosticsTableMode = "all")}>All tables</Button>
</div>
```

so it appears after `diagnosticsTableSections` and before:

```svelte
<div class="status-strip" aria-label="Diagnostics health overview">
```

- [ ] **Step 5: Pass filtered rows to tables**

In the table render loop, change:

```svelte
rows={section.rows}
```

to:

```svelte
rows={visibleDiagnosticRows(section.rows)}
```

and change the condition to:

```svelte
{#if diagnosticsTableMode === "all" || visibleDiagnosticRows(section.rows).length > 0}
```

Keep:

```svelte
open={hasDiagnosticIssue(section.rows)}
```

so a table with issues opens by default.

- [ ] **Step 6: Add table summary copy for filtered rows**

In `DiagnosticCountTable.svelte`, add an optional `totalRows` prop:

```svelte
totalRows = rows.length,
```

and type:

```ts
totalRows?: number;
```

Change summary count text to:

```svelte
<span>{rows.length === totalRows ? `${rows.length} rows` : `${rows.length}/${totalRows} rows`}</span>
```

Pass it from diagnostics route:

```svelte
totalRows={section.rows.length}
```

- [ ] **Step 7: Run Diagnostics tests**

Run:

```powershell
npm.cmd run test -- src/lib/diagnostics-ux-contract.test.ts src/lib/diagnostics-route-contract.test.ts src/lib/diagnostics-view-model.test.ts
```

Expected: PASS.

- [ ] **Step 8: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

- [ ] **Step 9: Inspect the running app**

Open `/diagnostics` and verify:

- `Only issues / All tables` is visible above summary cards.
- `Only issues` shows only tables with issue rows.
- Tables in `Only issues` do not include rows where status is healthy and error is `None`.
- `All tables` restores full row counts.

- [ ] **Step 10: Commit**

Run:

```powershell
git add src/lib/diagnostics-view-model.ts src/lib/diagnostics-view-model.test.ts src/routes/diagnostics/+page.svelte src/lib/components/diagnostics/DiagnosticCountTable.svelte src/lib/diagnostics-ux-contract.test.ts src/lib/diagnostics-route-contract.test.ts
git commit -m "feat(diagnostics): prioritize issue rows"
```

---

### Task 4: Flatten Embedded YouTube Access Panel

**Files:**
- Modify: `src/lib/components/settings/youtube-settings-panel.svelte`
- Modify: `src/routes/accounts/+page.svelte`
- Test: `src/lib/accounts-ux-contract.test.ts`
- Test: `src/lib/source-access-placement.test.ts`

- [ ] **Step 1: Remove desk panel classes in embedded mode**

In `youtube-settings-panel.svelte`, change the root section from:

```svelte
<section class="desk-panel desk-panel-subtle youtube-settings-panel" class:embedded>
```

to:

```svelte
<section class={`youtube-settings-panel ${embedded ? "embedded" : "desk-panel desk-panel-subtle"}`.trim()}>
```

- [ ] **Step 2: Make embedded spacing rely on the parent shell**

In `youtube-settings-panel.svelte`, keep:

```css
.youtube-settings-panel.embedded {
  padding: 0;
  border: 0;
  background: transparent;
  box-shadow: none;
}
```

Add:

```css
.youtube-settings-panel.embedded .cookie-box {
  background: color-mix(in srgb, var(--panel-strong) 72%, transparent);
}
```

- [ ] **Step 3: Tighten Accounts YouTube shell copy**

In `src/routes/accounts/+page.svelte`, keep the shell heading but avoid repeating wording already inside the YouTube panel. Change the paragraph to:

```svelte
<p>Manage cookies and sync limits without mixing them into Telegram account identity.</p>
```

Keep:

```svelte
<YoutubeSettingsPanel embedded />
```

- [ ] **Step 4: Run Accounts tests**

Run:

```powershell
npm.cmd run test -- src/lib/accounts-ux-contract.test.ts src/lib/source-access-placement.test.ts
```

Expected: PASS.

- [ ] **Step 5: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

- [ ] **Step 6: Inspect the running app**

Open `/accounts` and verify:

- The YouTube section reads as one panel, not a panel inside another panel.
- Authentication and Sync policy remain visually separated.
- The Telegram accounts section is unchanged except for surrounding page flow.

- [ ] **Step 7: Commit**

Run:

```powershell
git add src/lib/components/settings/youtube-settings-panel.svelte src/routes/accounts/+page.svelte src/lib/accounts-ux-contract.test.ts src/lib/source-access-placement.test.ts
git commit -m "feat(accounts): flatten embedded youtube access"
```

---

### Task 5: Final Verification And Merge Readiness

**Files:**
- Modify only files already touched by Tasks 1-4 if verification reveals issues.

- [ ] **Step 1: Run full frontend tests**

Run:

```powershell
npm.cmd run test
```

Expected: all Vitest files pass.

- [ ] **Step 2: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

- [ ] **Step 3: Run Analysis smoke**

Run:

```powershell
npm.cmd run smoke:analysis
```

Expected: all Analysis smoke steps pass. If the app is already running and the smoke script reports a final dev-server port warning after `Analysis UI smoke passed`, record that separately and do not treat it as a UI failure when exit code is `0`.

- [ ] **Step 4: Inspect live routes**

Inspect:

- `/analysis` at about `1280x860`
- `/diagnostics`
- `/accounts`

Acceptance criteria:

- Analysis shows rail, bounded source reader, and companion in the first desktop viewport.
- Diagnostics table controls are visible before the summary card grid and issue mode removes healthy rows from issue tables.
- Accounts YouTube access reads as one panel with separated auth/policy groups.

- [ ] **Step 5: Commit verification fixes when verification changes files**

When verification produces a small fix in files already touched by this plan, run:

```powershell
git add src
git commit -m "fix: polish UX follow-up verification"
```

When verification produces no file changes, do not create an empty commit.

---

## Execution Notes

- Commit after every task.
- Use TDD: write or update the failing contract before changing production code.
- Keep changes scoped to UI layout and view-model filtering.
- Do not keep temporary screenshots under `tmp/` in the commit.
- Do not modify backend commands, database schema, source sync behavior, or source loading page sizes.
