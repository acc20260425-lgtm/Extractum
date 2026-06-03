# UX/UI Improvements Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reduce Extractum's visible UI friction by making Analysis quieter and more task-focused, then polish companion, settings, diagnostics, and accounts surfaces without changing backend behavior.

**Architecture:** Implement this as incremental Svelte component changes guarded by source-contract tests and route smoke checks. Keep the existing three-zone Analysis architecture: compact source rail, report/source canvas, run companion. Do not rewrite `src/routes/analysis/+page.svelte` broadly; touch it only when a component prop or layout hook requires it.

**Tech Stack:** Svelte 5, SvelteKit, Tauri 2, Vitest raw-source contract tests, `npm.cmd run check`, `npm.cmd run smoke:analysis`, lucide-svelte icons, existing `$lib/components/ui/*` primitives.

---

## Scope And Ordering

This plan covers the prioritized UX/UI audit from 2026-06-03. Tasks 1-6 are the first cohesive Analysis UX slice and should be executed first. Tasks 7-9 are secondary route polish and can be executed after the Analysis slice passes verification.

No production code should be changed before Task 1 tests are written and shown failing.

## Current UI Problems

- Analysis repeats the selected source/context across `ReportCanvas`, `ReportSetupPanel`, and `SourceReaderHeader`, pushing useful work downward.
- `SourceSwitcherPanel` mixes navigation with operational actions (`Sync`, `Takeout`, migrated-history import, `Delete`) and detailed recovery status.
- `SourceActivityView` is already the right home for sync/takeout/jobs, but source operations are still duplicated in the switcher.
- `UniversalItemsView` is searchable but reads like a long dump; media-only records often render as "No text content loaded."
- `RunCompanionRunsTab` exposes the full filter toolbar even when there are no runs.
- Settings needs clearer active-vs-editing profile feedback and model search.
- Diagnostics needs a way to scan problem sections before the large count tables.
- Accounts mixes Telegram account management, YouTube auth, and YouTube sync policy without enough section hierarchy.

## File Structure

Expected new tests:

- Create: `src/lib/analysis-priority-ux-contract.test.ts`
- Create: `src/lib/settings-profile-ux-contract.test.ts`
- Create: `src/lib/diagnostics-ux-contract.test.ts`
- Create: `src/lib/accounts-ux-contract.test.ts`

Expected Analysis component changes:

- Modify: `src/lib/components/analysis/report-canvas.svelte`
- Modify: `src/lib/components/analysis/report-workspace-tools.svelte`
- Modify: `src/lib/components/analysis/source-switcher-panel.svelte`
- Modify: `src/lib/components/analysis/compact-source-rail.svelte`
- Modify: `src/lib/components/analysis/source-activity-view.svelte`
- Modify: `src/lib/components/analysis/youtube-source-activity.svelte`
- Modify: `src/lib/components/analysis/universal-items-view.svelte`
- Modify: `src/lib/components/analysis/run-companion-runs-tab.svelte`
- Modify: `src/lib/components/analysis/source-browser-shell.svelte` only if the reader toolbar needs a new prop.
- Modify: `src/routes/analysis/+page.svelte` only for prop plumbing.

Expected secondary route changes:

- Modify: `src/routes/settings/+page.svelte`
- Modify: `src/routes/diagnostics/+page.svelte`
- Modify: `src/routes/accounts/+page.svelte`
- Modify: `src/lib/components/settings/youtube-settings-panel.svelte`

Do not modify Rust, database, API wrappers, or backend command behavior in this plan.

---

### Task 1: Freeze The Analysis UX Target

**Files:**
- Create: `src/lib/analysis-priority-ux-contract.test.ts`
- Test: `src/lib/analysis-redesign-route-contract.test.ts`
- Test: `src/lib/analysis-compact-source-rail.test.ts`
- Test: `src/lib/analysis-run-companion-tabs.test.ts`
- Test: `src/lib/analysis-source-readers.test.ts`

- [x] **Step 1: Write the failing Analysis UX contract**

Create `src/lib/analysis-priority-ux-contract.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import reportCanvasSource from "./components/analysis/report-canvas.svelte?raw";
import reportWorkspaceToolsSource from "./components/analysis/report-workspace-tools.svelte?raw";
import sourceSwitcherPanelSource from "./components/analysis/source-switcher-panel.svelte?raw";
import sourceActivityViewSource from "./components/analysis/source-activity-view.svelte?raw";
import universalItemsViewSource from "./components/analysis/universal-items-view.svelte?raw";
import runsTabSource from "./components/analysis/run-companion-runs-tab.svelte?raw";

describe("analysis priority UX contract", () => {
  it("keeps the report canvas top chrome compact and action-oriented", () => {
    expect(reportCanvasSource).toContain('class="canvas-context-bar"');
    expect(reportCanvasSource).toContain('aria-label="Analysis context"');
    expect(reportCanvasSource).toContain('class="canvas-actions-row"');
    expect(reportCanvasSource).toContain("showInlineWorkspaceTools");
    expect(reportWorkspaceToolsSource).toContain("compact = false");
    expect(reportWorkspaceToolsSource).toContain("class:compact={compact}");
    expect(reportWorkspaceToolsSource).toContain('aria-label="Workspace actions"');
  });

  it("keeps the source switcher primarily focused on source selection", () => {
    expect(sourceSwitcherPanelSource).toContain('class="source-row-operations"');
    expect(sourceSwitcherPanelSource).toContain("<summary>Source operations</summary>");
    expect(sourceSwitcherPanelSource).toContain("Manage operational state in the Activity tab.");
    expect(sourceSwitcherPanelSource).not.toContain('class="row-actions"');
  });

  it("makes source activity the visible home for source operations", () => {
    expect(sourceActivityViewSource).toContain('class="activity-action-grid"');
    expect(sourceActivityViewSource).toContain("Sync source");
    expect(sourceActivityViewSource).toContain("Start Takeout import");
    expect(sourceActivityViewSource).toContain("Detailed jobs");
  });

  it("turns loaded items into a reader instead of a raw dump", () => {
    expect(universalItemsViewSource).toContain("function itemPreviewText");
    expect(universalItemsViewSource).toContain("function itemContextLine");
    expect(universalItemsViewSource).toContain('class="item-preview"');
    expect(universalItemsViewSource).toContain('class:media-only={!item.content && item.hasMedia}');
    expect(universalItemsViewSource).toContain("Media-only item");
  });

  it("keeps run filters progressive when no runs exist", () => {
    expect(runsTabSource).toContain("hasAnyRuns");
    expect(runsTabSource).toContain("showRunsToolbar");
    expect(runsTabSource).toContain('class="runs-empty-guidance"');
    expect(runsTabSource).toContain("Run a report to create the first saved workspace.");
  });
});
```

- [x] **Step 2: Run the focused test and verify it fails**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-priority-ux-contract.test.ts
```

Expected: FAIL because the new contract tokens are not present yet.

- [x] **Step 3: Confirm existing guardrails still describe current behavior**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-redesign-route-contract.test.ts src/lib/analysis-compact-source-rail.test.ts src/lib/analysis-run-companion-tabs.test.ts src/lib/analysis-source-readers.test.ts
```

Expected: PASS before implementation starts.

- [x] **Step 4: Commit the failing test**

```powershell
git add src/lib/analysis-priority-ux-contract.test.ts
git commit -m "test: capture analysis UX priority contract"
```

---

### Task 2: Compact The Analysis Canvas Header

**Files:**
- Modify: `src/lib/components/analysis/report-canvas.svelte`
- Modify: `src/lib/components/analysis/report-workspace-tools.svelte`
- Test: `src/lib/analysis-priority-ux-contract.test.ts`
- Test: `src/lib/analysis-redesign-route-contract.test.ts`
- Test: `src/lib/analysis-workspace-tools.test.ts`

- [x] **Step 1: Extend `ReportWorkspaceTools` for compact inline rendering**

In `src/lib/components/analysis/report-workspace-tools.svelte`, add a `compact` prop with default `false`:

```svelte
let {
  compact = false,
  showNotebookLmExport,
  canExportNotebookLm,
  exportDisabledReason,
  exportingNotebookLm,
  templateEditorOpen,
  groupEditorOpen,
  onOpenNotebookLmExport,
  onToggleTemplateEditor,
  onToggleGroupEditor,
}: {
  compact?: boolean;
  showNotebookLmExport: boolean;
  canExportNotebookLm: boolean;
  exportDisabledReason: string | null;
  exportingNotebookLm: boolean;
  templateEditorOpen: boolean;
  groupEditorOpen: boolean;
  onOpenNotebookLmExport: () => void;
  onToggleTemplateEditor: () => void;
  onToggleGroupEditor: () => void;
} = $props();
```

Change the section opening tag:

```svelte
<section
  class="report-workspace-tools"
  class:compact={compact}
  aria-label="Workspace actions"
  data-smoke-id="analysis-workspace-tools"
>
```

Keep all three actions available. In compact mode, use the existing icon+text buttons but reduce spacing and remove the repeated eyebrow from the visual surface:

```svelte
{#if !compact}
  <div class="workspace-tools-copy">
    <span class="eyebrow">Workspace tools</span>
  </div>
{/if}
```

- [x] **Step 2: Move tools into the report canvas top bar**

In `src/lib/components/analysis/report-canvas.svelte`, add:

```svelte
const showInlineWorkspaceTools = $derived(true);
```

Replace the current `canvas-toolbar` block with this structure:

```svelte
<div class="canvas-context-bar" aria-label="Analysis context">
  <div class="canvas-title">
    <span class="eyebrow">{currentRun ? "Run workspace" : "Analysis workspace"}</span>
    <h2>{currentRun ? runTargetLabel(currentRun) : currentScopeTitle}</h2>
    <p>{currentRun ? "Report and source basis stay side by side." : currentScopeSummary}</p>
  </div>

  <div class="canvas-actions-row">
    <div class="canvas-tabs" role="tablist" aria-label="Report canvas mode">
      <Button
        type="button"
        role="tab"
        variant="secondary"
        selected={canvasMode === "report"}
        ariaSelected={canvasMode === "report"}
        smokeId="report-canvas-mode-report"
        onclick={() => onChangeCanvasMode("report")}
      >
        Report
      </Button>
      <Button
        type="button"
        role="tab"
        variant="secondary"
        selected={canvasMode === "source"}
        ariaSelected={canvasMode === "source"}
        smokeId="report-canvas-mode-source"
        onclick={() => onChangeCanvasMode("source")}
      >
        Source
      </Button>
    </div>

    {#if showInlineWorkspaceTools}
      <ReportWorkspaceTools
        compact
        {showNotebookLmExport}
        {canExportNotebookLm}
        exportDisabledReason={notebookLmExportDisabledReason}
        {exportingNotebookLm}
        {templateEditorOpen}
        {groupEditorOpen}
        onOpenNotebookLmExport={onOpenNotebookLmExport}
        onToggleTemplateEditor={() => (templateEditorOpen = !templateEditorOpen)}
        onToggleGroupEditor={() => (groupEditorOpen = !groupEditorOpen)}
      />
    {/if}
  </div>
</div>
```

Remove the separate `<ReportWorkspaceTools ... />` block below the toolbar.

- [x] **Step 3: Add compact CSS**

In `report-canvas.svelte`, rename `.canvas-toolbar` styles to `.canvas-context-bar` and add:

```css
.canvas-context-bar {
  display: flex;
  justify-content: space-between;
  gap: 0.9rem;
  align-items: flex-start;
}

.canvas-actions-row {
  display: flex;
  gap: 0.55rem;
  align-items: flex-start;
  justify-content: flex-end;
  flex-wrap: wrap;
}

@media (max-width: 980px) {
  .canvas-context-bar {
    flex-direction: column;
  }

  .canvas-actions-row {
    width: 100%;
    justify-content: space-between;
  }
}
```

In `report-workspace-tools.svelte`, add:

```css
.report-workspace-tools.compact {
  border: 0;
  padding: 0;
  background: transparent;
  box-shadow: none;
}

.report-workspace-tools.compact .workspace-tools-actions {
  justify-content: flex-end;
}
```

- [x] **Step 4: Run focused tests**

```powershell
npm.cmd run test -- src/lib/analysis-priority-ux-contract.test.ts -t "keeps the report canvas top chrome compact and action-oriented"
npm.cmd run test -- src/lib/analysis-redesign-route-contract.test.ts src/lib/analysis-workspace-tools.test.ts
```

Expected: PASS.

- [x] **Step 5: Check Svelte types**

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

- [x] **Step 6: Commit**

```powershell
git add src/lib/components/analysis/report-canvas.svelte src/lib/components/analysis/report-workspace-tools.svelte src/lib/analysis-priority-ux-contract.test.ts
git commit -m "feat(analysis): compact workspace header actions"
```

---

### Task 3: Make Source Switcher Selection-First

**Files:**
- Modify: `src/lib/components/analysis/source-switcher-panel.svelte`
- Modify: `src/lib/components/analysis/compact-source-rail.svelte`
- Test: `src/lib/analysis-priority-ux-contract.test.ts`
- Test: `src/lib/analysis-compact-source-rail.test.ts`

- [x] **Step 1: Add a source operations disclosure helper**

In `source-switcher-panel.svelte`, add this helper near the existing formatting helpers:

```ts
function sourceOperationSummary(source: Source, takeoutActive: boolean, sourceJobActive: boolean) {
  if (takeoutActive) return "Operation running";
  if (sourceJobActive) return "Job running";
  if (source.sourceType === "telegram") return "Sync and import";
  if (source.sourceType === "youtube") return "Sync and delete";
  return "Manage";
}
```

- [x] **Step 2: Replace visible row actions with a disclosure**

Replace the current `<div class="row-actions">...</div>` block with:

```svelte
<details class="source-row-operations">
  <summary>
    <span>Source operations</span>
    <Badge variant={takeoutActive || sourceJobActive ? "info" : "neutral"}>
      {sourceOperationSummary(source, takeoutActive, sourceJobActive)}
    </Badge>
  </summary>
  <p class="operation-note">Manage operational state in the Activity tab.</p>
  <div class="operation-actions">
    {#if capabilities.canSync}
      <Button
        size="sm"
        variant="secondary"
        onclick={() => onSyncSource(source.id)}
        disabled={!!syncingIds[source.id] || deleting || takeoutActive || syncReason !== null}
        title={takeoutActive ? "Takeout import is active." : syncReason ?? undefined}
      >
        <RefreshCw size={13} aria-hidden="true" />
        {syncingIds[source.id] ? "Syncing..." : "Sync"}
      </Button>
    {/if}

    {#if capabilities.canImportArchive}
      {#if takeoutActive && takeoutJob}
        <Button
          size="sm"
          variant="secondary"
          onclick={() => onCancelTakeoutImport(takeoutJob.job_id)}
          disabled={takeoutJob.status === "cancel_requested"}
        >
          <Square size={13} aria-hidden="true" />
          {takeoutJob.status === "cancel_requested" ? "Cancelling..." : "Cancel"}
        </Button>
      {:else}
        <Button
          size="sm"
          variant="secondary"
          onclick={() => onStartTakeoutImport(source.id)}
          disabled={!!startingTakeoutSourceIds[source.id] || deleting || !!syncingIds[source.id] || syncReason !== null}
          title={syncReason ?? undefined}
        >
          <Archive size={13} aria-hidden="true" />
          {startingTakeoutSourceIds[source.id] ? "Starting..." : "Takeout"}
        </Button>
      {/if}

      {#if source.migratedHistoryStatus === "available"}
        {@const migratedHistoryReason = migratedHistoryActionDisabledReason(
          source,
          sourceJobActive,
          takeoutActive,
          !!startingMigratedHistorySourceIds[source.id],
        )}
        <Button
          size="sm"
          variant="secondary"
          onclick={() => onStartMigratedHistoryImport(source.id)}
          disabled={migratedHistoryReason !== null}
          title={migratedHistoryReason ?? undefined}
        >
          <Archive size={13} aria-hidden="true" />
          {startingMigratedHistorySourceIds[source.id] ? "Starting historical import..." : migratedHistoryActionLabel()}
        </Button>
      {/if}
    {/if}

    {#if capabilities.canDelete}
      <Button
        size="sm"
        variant="danger-soft"
        onclick={() => onDeleteSource(source)}
        disabled={deleting || !!syncingIds[source.id] || takeoutActive || sourceJobActive}
        title={takeoutActive ? "Takeout import is active." : sourceJobActive ? "Source job is active." : undefined}
      >
        <Trash2 size={13} aria-hidden="true" />
        {deleting ? "Deleting..." : "Delete"}
      </Button>
    {/if}
  </div>
</details>
```

- [x] **Step 3: Style source operations as secondary**

Replace `.row-actions` CSS with:

```css
.source-row-operations {
  border: 1px solid color-mix(in srgb, var(--border) 72%, transparent);
  border-radius: 8px;
  background: color-mix(in srgb, var(--panel) 76%, transparent);
}

.source-row-operations summary {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  cursor: pointer;
  padding: 0.55rem 0.65rem;
  color: var(--muted);
  font-size: 0.78rem;
  font-weight: 650;
}

.operation-note {
  margin: 0;
  padding: 0 0.65rem;
  color: var(--muted);
  font-size: 0.78rem;
}

.operation-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.45rem;
  padding: 0.55rem 0.65rem 0.65rem;
}
```

- [x] **Step 4: Keep the compact rail quiet**

In `compact-source-rail.svelte`, keep the existing primary sync affordance only for the currently selected source. Do not add `Delete`, `Takeout`, or source job details to the collapsed rail.

- [x] **Step 5: Run focused tests**

```powershell
npm.cmd run test -- src/lib/analysis-priority-ux-contract.test.ts -t "keeps the source switcher primarily focused on source selection"
npm.cmd run test -- src/lib/analysis-compact-source-rail.test.ts
```

Expected: PASS. Update `src/lib/analysis-compact-source-rail.test.ts` assertions that currently require visible `onSyncSource(source.id)`, `onStartTakeoutImport(source.id)`, and `onDeleteSource(source)` in the expanded panel so they assert the disclosure instead.

- [x] **Step 6: Commit**

```powershell
git add src/lib/components/analysis/source-switcher-panel.svelte src/lib/components/analysis/compact-source-rail.svelte src/lib/analysis-priority-ux-contract.test.ts src/lib/analysis-compact-source-rail.test.ts
git commit -m "feat(analysis): make source switcher selection focused"
```

---

### Task 4: Strengthen Source Activity As The Operations Home

**Files:**
- Modify: `src/lib/components/analysis/source-activity-view.svelte`
- Modify: `src/lib/components/analysis/youtube-source-activity.svelte`
- Test: `src/lib/analysis-priority-ux-contract.test.ts`
- Test: `src/lib/analysis-source-readers.test.ts`

- [x] **Step 1: Add an activity action grid to Telegram activity**

In `source-activity-view.svelte`, group the primary actions directly below the activity intro:

```svelte
<div class="activity-action-grid">
  <Button type="button" variant="secondary" onclick={() => onSyncSource(source.id)}>
    Sync source
  </Button>
  <Button type="button" variant="secondary" onclick={() => onStartTakeoutImport(source.id)}>
    Start Takeout import
  </Button>
</div>
```

Keep the existing disabled rules and active job cancellation logic from the current sections. The action grid should call the same callbacks that the existing per-section buttons call.

- [x] **Step 2: Keep detailed state in the existing cards**

Keep these sections visible below the action grid:

```svelte
<section class="activity-section" aria-label="Source sync">
<section class="activity-section" aria-label="Telegram recovery">
<section class="activity-section" aria-label="Migrated history">
<section class="activity-section" aria-label="Detailed jobs">
```

The first visible action in each card can remain, but avoid rendering duplicate primary buttons immediately adjacent to the new action grid. Prefer status and cancellation controls inside cards.

- [x] **Step 3: Mirror the pattern for YouTube activity**

In `youtube-source-activity.svelte`, add:

```svelte
<div class="activity-action-grid">
  <Button type="button" variant="secondary" onclick={() => onSyncSource(source.id)}>
    Sync source
  </Button>
</div>
```

Keep retry actions where they are tied to a specific YouTube job or failed playlist item.

- [x] **Step 4: Add CSS for the action grid**

Use the same class in both activity components:

```css
.activity-action-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  align-items: center;
}
```

- [x] **Step 5: Run focused tests**

```powershell
npm.cmd run test -- src/lib/analysis-priority-ux-contract.test.ts -t "makes source activity the visible home for source operations"
npm.cmd run test -- src/lib/analysis-source-readers.test.ts
```

Expected: PASS.

- [x] **Step 6: Commit**

```powershell
git add src/lib/components/analysis/source-activity-view.svelte src/lib/components/analysis/youtube-source-activity.svelte src/lib/analysis-priority-ux-contract.test.ts src/lib/analysis-source-readers.test.ts
git commit -m "feat(analysis): emphasize source activity operations"
```

---

### Task 5: Make Loaded Items Readable And Evidence-Friendly

**Files:**
- Modify: `src/lib/components/analysis/universal-items-view.svelte`
- Modify: `src/lib/source-browser-model.ts`
- Test: `src/lib/analysis-priority-ux-contract.test.ts`
- Test: `src/lib/analysis-source-readers.test.ts`
- Test: `src/lib/source-browser-model.test.ts`

- [x] **Step 1: Add model helpers for item copy**

In `src/lib/source-browser-model.ts`, add:

```ts
export function sourceItemPreviewText(item: Pick<SourceItem, "content" | "hasMedia" | "mediaKind">): string {
  if (item.content && item.content.trim().length > 0) return item.content;
  if (item.hasMedia) return `Media-only item${item.mediaKind ? ` (${item.mediaKind})` : ""}. Text was not loaded.`;
  return "No text content loaded.";
}

export function sourceItemContextLine(
  item: Pick<SourceItem, "author" | "externalId" | "hasMedia" | "mediaKind">,
  sourceLabel: string,
): string {
  return [
    item.author,
    sourceLabel,
    item.externalId,
    item.hasMedia ? item.mediaKind ?? "media" : null,
  ].filter(Boolean).join(" - ");
}
```

- [x] **Step 2: Add tests for model helpers**

In `src/lib/source-browser-model.test.ts`, add:

```ts
import { sourceItemContextLine, sourceItemPreviewText } from "$lib/source-browser-model";

it("labels media-only source items without treating them as empty text", () => {
  expect(sourceItemPreviewText({ content: null, hasMedia: true, mediaKind: "photo" }))
    .toBe("Media-only item (photo). Text was not loaded.");
  expect(sourceItemPreviewText({ content: "Body", hasMedia: false, mediaKind: null })).toBe("Body");
});

it("builds compact item context lines", () => {
  expect(sourceItemContextLine(
    { author: "Alice", externalId: "42", hasMedia: true, mediaKind: "photo" },
    "Source #7",
  )).toBe("Alice - Source #7 - 42 - photo");
});
```

- [x] **Step 3: Use helpers in `UniversalItemsView`**

In `universal-items-view.svelte`, import the helpers:

```ts
import {
  filterLoadedSourceItems,
  sortLoadedSourceItems,
  sourceItemContextLine,
  sourceItemKindChips,
  sourceItemPreviewText,
  type LoadedSourceItemSort,
} from "$lib/source-browser-model";
```

Add local wrappers:

```ts
function itemPreviewText(item: SourceItem) {
  return sourceItemPreviewText(item);
}

function itemContextLine(item: SourceItem) {
  return sourceItemContextLine(item, itemSourceLabel(item));
}
```

Replace the metadata and paragraph area inside each item article with:

```svelte
<div class="item-meta">
  <Badge variant="neutral">{itemContextLine(item)}</Badge>
  {#if item.hasMedia}<Badge variant="info">{item.mediaKind ?? "media"}</Badge>{/if}
</div>
<p class="item-preview" class:media-only={!item.content && item.hasMedia}>
  {itemPreviewText(item)}
</p>
```

- [x] **Step 4: Add reader CSS**

```css
.item-preview {
  margin: 0;
  line-height: 1.5;
  white-space: pre-wrap;
}

.item-preview.media-only {
  color: var(--muted);
  font-style: italic;
}
```

- [x] **Step 5: Run focused tests**

```powershell
npm.cmd run test -- src/lib/analysis-priority-ux-contract.test.ts -t "turns loaded items into a reader instead of a raw dump"
npm.cmd run test -- src/lib/source-browser-model.test.ts src/lib/analysis-source-readers.test.ts
```

Expected: PASS.

- [x] **Step 6: Commit**

```powershell
git add src/lib/source-browser-model.ts src/lib/source-browser-model.test.ts src/lib/components/analysis/universal-items-view.svelte src/lib/analysis-priority-ux-contract.test.ts src/lib/analysis-source-readers.test.ts
git commit -m "feat(analysis): improve source item reading"
```

---

### Task 6: Make Runs Companion Progressive

**Files:**
- Modify: `src/lib/components/analysis/run-companion-runs-tab.svelte`
- Test: `src/lib/analysis-priority-ux-contract.test.ts`
- Test: `src/lib/analysis-run-companion-tabs.test.ts`

- [x] **Step 1: Add derived toolbar visibility**

In `run-companion-runs-tab.svelte`, add:

```ts
const hasAnyRuns = $derived(activeRuns.length > 0 || savedRuns.length > 0);
const showRunsToolbar = $derived(hasAnyRuns || hasActiveFilters);
```

- [x] **Step 2: Hide filters until they are useful**

Wrap the current `<div class="runs-toolbar">...</div>`:

```svelte
{#if showRunsToolbar}
  <div class="runs-toolbar">
    <!-- existing search, scope, status, advanced filters, and refresh row stay here -->
  </div>
{/if}
```

Set advanced filters open only when filters are active:

```svelte
<details class="advanced-filters" open={hasActiveFilters}>
```

- [x] **Step 3: Improve the no-runs empty state**

Replace the no-filter empty state with:

```svelte
<div class="runs-empty-guidance">
  <EmptyState description="Run a report to create the first saved workspace." />
  <p>Completed reports will appear here with provider, model, snapshot, and error metadata.</p>
  <div class="refresh-row">
    <Button size="sm" variant="secondary" onclick={onRefreshActiveRuns}>
      <RefreshCw size={14} aria-hidden="true" /> Active
    </Button>
    <Button size="sm" variant="secondary" onclick={onRefreshRuns}>
      <RefreshCw size={14} aria-hidden="true" /> Saved
    </Button>
  </div>
</div>
```

- [x] **Step 4: Add empty guidance CSS**

```css
.runs-empty-guidance {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.6rem;
  color: var(--muted);
  font-size: 0.84rem;
}

.runs-empty-guidance p {
  margin: 0;
}
```

- [x] **Step 5: Run focused tests**

```powershell
npm.cmd run test -- src/lib/analysis-priority-ux-contract.test.ts -t "keeps run filters progressive when no runs exist"
npm.cmd run test -- src/lib/analysis-run-companion-tabs.test.ts
```

Expected: PASS.

- [x] **Step 6: Run manual Analysis smoke**

With the app running, run:

```powershell
npm.cmd run smoke:analysis
```

Expected: PASS and no browser-side failure for `/analysis`.

- [x] **Step 7: Commit**

```powershell
git add src/lib/components/analysis/run-companion-runs-tab.svelte src/lib/analysis-priority-ux-contract.test.ts src/lib/analysis-run-companion-tabs.test.ts
git commit -m "feat(analysis): simplify empty runs companion"
```

---

### Task 7: Clarify Settings Profile And Model Selection

**Files:**
- Create: `src/lib/settings-profile-ux-contract.test.ts`
- Modify: `src/routes/settings/+page.svelte`

- [x] **Step 1: Write the settings UX contract**

Create `src/lib/settings-profile-ux-contract.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import settingsPageSource from "../routes/settings/+page.svelte?raw";

describe("settings profile UX contract", () => {
  it("separates active profile from the profile being edited", () => {
    expect(settingsPageSource).toContain('class="profile-status-strip"');
    expect(settingsPageSource).toContain("Active profile");
    expect(settingsPageSource).toContain("Editing profile");
    expect(settingsPageSource).toContain("Set active after save");
  });

  it("adds model search before the large model selector", () => {
    expect(settingsPageSource).toContain("modelQuery");
    expect(settingsPageSource).toContain("filteredAvailableModels");
    expect(settingsPageSource).toContain('ariaLabel="Search models"');
  });
});
```

- [x] **Step 2: Run the new test and verify it fails**

```powershell
npm.cmd run test -- src/lib/settings-profile-ux-contract.test.ts
```

Expected: FAIL.

- [x] **Step 3: Add model search state**

In `src/routes/settings/+page.svelte`, add near model state:

```ts
let modelQuery = $state("");

const filteredAvailableModels = $derived.by(() => {
  const query = modelQuery.trim().toLowerCase();
  if (!query) return availableModels;
  return availableModels.filter((model) =>
    `${model.display_name} ${model.model}`.toLowerCase().includes(query),
  );
});
```

Change the model selector loop:

```svelte
{#each filteredAvailableModels as model (model.model)}
  <option value={model.model}>{model.display_name} - {model.model}</option>
{/each}
```

- [x] **Step 4: Add profile status strip**

Replace the existing `.profile-strip` contents with:

```svelte
<div class="profile-status-strip">
  <MetaPill>Editing profile: {creatingProfile ? "new profile" : selectedProfileId}</MetaPill>
  <MetaPill tone={selectedProfileId === activeProfile && !creatingProfile ? "active" : "default"}>
    Active profile: {activeProfile || "none"}
  </MetaPill>
  {#if !creatingProfile && selectedProfileId !== activeProfile}
    <MetaPill>Set active after save</MetaPill>
  {/if}
</div>
```

- [x] **Step 5: Add search input above the model select**

Inside the `Default model` label, before `<Select>`, add:

```svelte
<Input
  type="search"
  value={modelQuery}
  placeholder="Search models"
  ariaLabel="Search models"
  oninput={(event) => (modelQuery = (event.currentTarget as HTMLInputElement).value)}
/>
```

- [x] **Step 6: Run checks**

```powershell
npm.cmd run test -- src/lib/settings-profile-ux-contract.test.ts src/lib/diagnostics-route-contract.test.ts
npm.cmd run check
```

Expected: PASS and `svelte-check found 0 errors and 0 warnings`.

- [x] **Step 7: Commit**

```powershell
git add src/routes/settings/+page.svelte src/lib/settings-profile-ux-contract.test.ts
git commit -m "feat(settings): clarify profile and model selection"
```

---

### Task 8: Make Diagnostics Tables Scannable

**Files:**
- Create: `src/lib/diagnostics-ux-contract.test.ts`
- Modify: `src/routes/diagnostics/+page.svelte`
- Modify: `src/lib/components/diagnostics/DiagnosticCountTable.svelte`

- [x] **Step 1: Write diagnostics UX contract**

Create `src/lib/diagnostics-ux-contract.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import diagnosticsPageSource from "../routes/diagnostics/+page.svelte?raw";
import diagnosticsTableSource from "./components/diagnostics/DiagnosticCountTable.svelte?raw";

describe("diagnostics UX contract", () => {
  it("adds problem-first diagnostics table controls", () => {
    expect(diagnosticsPageSource).toContain("diagnosticsTableMode");
    expect(diagnosticsPageSource).toContain("Only issues");
    expect(diagnosticsPageSource).toContain("All tables");
    expect(diagnosticsPageSource).toContain("diagnosticsTableSections");
  });

  it("keeps large diagnostics tables collapsible", () => {
    expect(diagnosticsTableSource).toContain("<details");
    expect(diagnosticsTableSource).toContain("<summary");
    expect(diagnosticsTableSource).toContain("open = true");
  });
});
```

- [x] **Step 2: Run the new test and verify it fails**

```powershell
npm.cmd run test -- src/lib/diagnostics-ux-contract.test.ts
```

Expected: FAIL.

- [x] **Step 3: Make `DiagnosticCountTable` collapsible**

In `DiagnosticCountTable.svelte`, add an `open` prop:

```svelte
let {
  title,
  description,
  columns,
  rows,
  emptyMessage = "No diagnostic counts reported",
  open = true,
}: {
  title: string;
  description?: string;
  columns: DiagnosticTableColumn[];
  rows: DiagnosticTableRow[];
  emptyMessage?: string;
  open?: boolean;
} = $props();
```

Wrap the existing table body inside the existing `SurfaceCard` with:

```svelte
<details class="diagnostic-count-details" {open}>
  <summary>
    <span>{title}</span>
    <span>{rows.length} rows</span>
  </summary>
  <p>{description}</p>
  <!-- existing table markup stays here -->
</details>
```

- [x] **Step 4: Add diagnostics table mode**

In `diagnostics/+page.svelte`, add:

```ts
let diagnosticsTableMode = $state<"issues" | "all">("issues");

function hasDiagnosticIssue(rows: Record<string, string | number>[]) {
  return rows.some((row) => Object.values(row).some((cell) =>
    /failed|error|missing|unavailable|pending|warning/i.test(String(cell)),
  ));
}
```

Build table sections before rendering:

```svelte
{@const diagnosticsTableSections = [
  { title: "Provider profiles", description: "Configured profile counts by provider", columns: providerColumns, rows: providerRows(summary) },
  { title: "Telegram runtimes", description: "Account runtime statuses by coarse state", columns: telegramColumns, rows: telegramRows(summary) },
  { title: "Sources", description: "Source counts by type, subtype, active state, and sync state", columns: sourceColumns, rows: sourceRows(summary) },
  { title: "Items", description: "Item counts by coarse source and content fields", columns: itemColumns, rows: itemRows(summary) },
  { title: "Analysis runs", description: "Run counts by provider, scope, status, snapshot state, and error kind", columns: runColumns, rows: runRows(summary) },
  { title: "LLM requests", description: "Request counts by provider, kind, and state", columns: llmColumns, rows: llmRows(summary) },
  { title: "YouTube jobs", description: "Job aggregates by type, status, warning state, and error kind", columns: youtubeJobColumns, rows: youtubeRows(summary) },
  { title: "Ingest batches", description: "Batch aggregates by provider, kind, status, completeness, and error kind", columns: ingestBatchColumns, rows: ingestBatchRows(summary) },
  { title: "Ingest warnings", description: "Warning aggregates by provider, kind, status, and warning code", columns: ingestWarningColumns, rows: ingestWarningRows(summary) },
]}
```

Render controls:

```svelte
<div class="diagnostics-table-controls" aria-label="Diagnostics table display">
  <Button size="sm" variant="secondary" selected={diagnosticsTableMode === "issues"} onclick={() => (diagnosticsTableMode = "issues")}>Only issues</Button>
  <Button size="sm" variant="secondary" selected={diagnosticsTableMode === "all"} onclick={() => (diagnosticsTableMode = "all")}>All tables</Button>
</div>
```

Render tables through the sections list:

```svelte
{#each diagnosticsTableSections as section (section.title)}
  {#if diagnosticsTableMode === "all" || hasDiagnosticIssue(section.rows)}
    <DiagnosticCountTable
      title={section.title}
      description={section.description}
      columns={section.columns}
      rows={section.rows}
      open={hasDiagnosticIssue(section.rows)}
    />
  {/if}
{/each}
```

- [x] **Step 5: Run diagnostics tests and check**

```powershell
npm.cmd run test -- src/lib/diagnostics-ux-contract.test.ts src/lib/diagnostics-route-contract.test.ts src/lib/diagnostics-view-model.test.ts
npm.cmd run check
```

Expected: PASS and `svelte-check found 0 errors and 0 warnings`.

- [x] **Step 6: Commit**

```powershell
git add src/routes/diagnostics/+page.svelte src/lib/components/diagnostics/DiagnosticCountTable.svelte src/lib/diagnostics-ux-contract.test.ts src/lib/diagnostics-route-contract.test.ts
git commit -m "feat(diagnostics): make count tables scannable"
```

---

### Task 9: Clarify Accounts And YouTube Access Sections

**Files:**
- Create: `src/lib/accounts-ux-contract.test.ts`
- Modify: `src/routes/accounts/+page.svelte`
- Modify: `src/lib/components/settings/youtube-settings-panel.svelte`
- Test: `src/lib/source-access-placement.test.ts`

- [x] **Step 1: Write accounts UX contract**

Create `src/lib/accounts-ux-contract.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import accountsPageSource from "../routes/accounts/+page.svelte?raw";
import youtubeSettingsPanelSource from "./components/settings/youtube-settings-panel.svelte?raw";

describe("accounts UX contract", () => {
  it("separates Telegram identity from YouTube access", () => {
    expect(accountsPageSource).toContain("Telegram accounts");
    expect(accountsPageSource).toContain("YouTube access");
    expect(accountsPageSource).toContain("Sync policy");
  });

  it("keeps YouTube auth and sync settings in separate visual groups", () => {
    expect(youtubeSettingsPanelSource).toContain('class="youtube-auth-section"');
    expect(youtubeSettingsPanelSource).toContain('class="youtube-sync-policy-section"');
    expect(youtubeSettingsPanelSource).toContain("Authentication");
    expect(youtubeSettingsPanelSource).toContain("Sync policy");
  });
});
```

- [x] **Step 2: Run the new test and verify it fails**

```powershell
npm.cmd run test -- src/lib/accounts-ux-contract.test.ts
```

Expected: FAIL.

- [x] **Step 3: Rename account panel copy**

In `accounts/+page.svelte`, change the configured accounts panel heading:

```svelte
<span class="page-eyebrow">Telegram accounts</span>
<h2>Telegram accounts</h2>
<p>Open Telegram auth, check runtime state, and keep sync-capable identities healthy.</p>
```

Add a lightweight wrapper heading before `<YoutubeSettingsPanel />`:

```svelte
<section class="desk-panel youtube-access-shell">
  <div class="panel-header">
    <div class="panel-header-copy">
      <span class="page-eyebrow">YouTube access</span>
      <h2>YouTube access</h2>
      <p>Manage cookies, auth state, and provider sync policy separately from Telegram identities.</p>
    </div>
  </div>
  <YoutubeSettingsPanel embedded />
</section>
```

- [x] **Step 4: Add `embedded` prop to YouTube panel**

In `youtube-settings-panel.svelte`, add:

```svelte
let { embedded = false } = $props();
```

Use it on the outer panel class:

```svelte
<section class="desk-panel youtube-settings-panel" class:embedded>
```

Split auth and policy controls:

```svelte
<section class="youtube-auth-section" aria-label="YouTube authentication">
  <h3>Authentication</h3>
  <!-- existing auth status, cookies, auth enable controls -->
</section>

<section class="youtube-sync-policy-section" aria-label="YouTube sync policy">
  <h3>Sync policy</h3>
  <!-- existing captions language, delay, parallelism, limits, retry controls -->
</section>
```

- [x] **Step 5: Run tests and check**

```powershell
npm.cmd run test -- src/lib/accounts-ux-contract.test.ts src/lib/source-access-placement.test.ts
npm.cmd run check
```

Expected: PASS and `svelte-check found 0 errors and 0 warnings`.

- [x] **Step 6: Commit**

```powershell
git add src/routes/accounts/+page.svelte src/lib/components/settings/youtube-settings-panel.svelte src/lib/accounts-ux-contract.test.ts src/lib/source-access-placement.test.ts
git commit -m "feat(accounts): clarify source access sections"
```

---

### Task 10: Full Verification And Visual Review

**Files:**
- Modify only files already touched by Tasks 1-9 if fixes are needed.

- [x] **Step 1: Run all frontend tests**

```powershell
npm.cmd run test
```

Expected: all Vitest suites pass.

- [x] **Step 2: Run Svelte check**

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

- [x] **Step 3: Run analysis smoke**

```powershell
npm.cmd run smoke:analysis
```

Expected: PASS.

- [x] **Step 4: Run full project verification**

```powershell
npm.cmd run verify
```

Expected: frontend tests, Svelte check, Rust tests, and diff check pass.

- [x] **Step 5: Inspect the running Tauri app**

Use the running app and inspect these routes:

- `/analysis`, Report mode, no runs.
- `/analysis`, Source mode, `Timeline`, `Items`, `Activity`.
- Source switcher opened from the compact rail.
- `/settings`, profile selected but not active.
- `/diagnostics`, issue-only and all-table modes.
- `/accounts`, Telegram and YouTube sections.

Acceptance criteria:

- Analysis top area is shorter than the current stacked setup/tools/source header.
- Source switcher still supports source and group selection without exposing all operations as always-visible row buttons.
- Activity remains the visible home for sync, takeout, migrated-history, and jobs.
- Runs companion empty state is calm when no runs exist.
- Settings makes active profile and edited profile obvious.
- Diagnostics tables can be scanned without reading every table first.
- Accounts visually separates Telegram identities from YouTube auth/sync policy.

- [x] **Step 6: Commit verification fixes if any**

If verification required small fixes:

```powershell
git add src
git commit -m "fix: polish UX verification issues"
```

If no fixes were needed, do not create an empty commit.

---

### Task 11: Correct Analysis Workspace Live Review Mismatch

**Files:**
- Modify: `src/lib/analysis-priority-ux-contract.test.ts`
- Modify: `src/lib/components/analysis/report-canvas.svelte`
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `src/lib/components/analysis/source-reader-header.svelte`
- Modify: `src/lib/components/analysis/report-workspace-tools.svelte`

- [x] **Step 1: Capture the mismatch in the Analysis UX contract**

Add contract assertions that require `ReportCanvas` to pass a compact source header into the source surface, `ReportSourceSurface` to forward that state to `SourceReaderHeader`, and compact workspace tools to use small icon-only actions with accessible labels.

- [x] **Step 2: Verify the new contract fails before code changes**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-priority-ux-contract.test.ts -t "keeps the report canvas top chrome compact and action-oriented"
```

Expected: FAIL because live source mode still renders a framed `SourceReaderHeader` and text-heavy workspace actions.

- [x] **Step 3: Compact the real Analysis workspace chrome**

Update the live source header path so source mode renders `SourceReaderHeader` as inline metadata instead of a second card. Reduce `canvas-context-bar` spacing and change compact workspace tools to icon-only buttons with `aria-label` and `title`.

- [x] **Step 4: Re-check the running Tauri app**

Inspect `/analysis` in the running app and confirm the real `canvas-context-bar` no longer wraps action text and the source header has no border, shadow, or panel background.

- [x] **Step 5: Run focused verification**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-priority-ux-contract.test.ts src/lib/analysis-redesign-route-contract.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-source-readers.test.ts src/lib/analysis-workspace-tools.test.ts
npm.cmd run check
```

Expected: PASS and `svelte-check found 0 errors and 0 warnings`.

---

## Execution Notes

- Commit after every task.
- Keep each task focused; do not fold secondary route work into the Analysis tasks.
- Prefer extending existing source-contract tests over adding browser-only tests for every copy change.
- Use browser or Tauri screenshots during Task 10 to catch layout issues that raw-source tests cannot see.
- Do not add a new design system or new global UI primitives unless an existing component cannot express the required control.
