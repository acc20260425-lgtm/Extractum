# Analysis Workspace Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move analysis workspace tools into one `ReportCanvas`-level path so setup, opened runs, report mode, and source mode share the same export/template/group actions.

**Architecture:** Add a presentational `ReportWorkspaceTools.svelte` component and keep all route-derived availability in `ReportCanvas`. `ReportCanvas` owns the template/group drawer state and renders the drawers once below the tools; `ReportSetupPanel` becomes a setup/form surface without editor drawer ownership.

**Tech Stack:** Svelte 5, SvelteKit, TypeScript, Vitest raw-source contract tests, lucide-svelte icons, existing Extractum UI primitives.

---

## File Structure

- Create `src/lib/components/analysis/report-workspace-tools.svelte`
  - Presentational workspace action bar.
  - Receives booleans, disabled reason, labels, and callbacks.
  - Imports UI primitives and lucide icons only; no API modules and no Tauri `invoke`.
- Modify `src/lib/components/ui/Button.svelte`
  - Add pass-through support for `aria-describedby` via an `ariaDescribedby` prop.
- Modify `src/lib/components/analysis/report-canvas.svelte`
  - Import and render `ReportWorkspaceTools`.
  - Replace opened-run-only management block with always-available canvas-level workspace tools.
  - Own `templateEditorOpen` and `groupEditorOpen`.
  - Render `TemplateEditor` and `SourceGroupEditor` drawers exactly once immediately below `ReportWorkspaceTools`.
  - Derive NotebookLM export visibility/availability from `currentSource` and `currentGroup`.
  - Keep `NotebookLmExportDialog` rendered once at canvas level.
  - Stop passing editor-only props/callbacks into `ReportSetupPanel`.
- Modify `src/lib/components/analysis/report-setup-panel.svelte`
  - Remove `TemplateEditor` and `SourceGroupEditor` imports.
  - Remove local editor drawer state, secondary action buttons, drawer markup, editor-only props, and editor-only callbacks.
  - Keep setup controls, setup preflight copy, `selectedTemplate`, `Run report`, and single-source `Sync source`.
- Modify `src/lib/analysis-report-canvas.test.ts`
  - Update canvas/source contract assertions from opened-run management to shared `ReportWorkspaceTools`.
  - Assert one canvas-level `NotebookLmExportDialog`.
  - Assert workspace tools appear before report/source body.
- Modify `src/lib/analysis-redesign-route-contract.test.ts`
  - Update assertions that setup no longer owns editor drawers.
- Modify `src/lib/analysis-group-editor-props.test.ts`
  - Update group editor prop ownership assertions: editor props stay in route and `ReportCanvas`, not `ReportSetupPanel`.
- Modify `src/lib/analysis-report-setup-props.test.ts`
  - Add setup cleanup assertions for removed editor-only props and preserved setup actions.
- Create `src/lib/analysis-workspace-tools.test.ts`
  - Raw-source contract tests for `ReportWorkspaceTools`.
- Modify `docs/project.md` and `docs/design-document.md`
  - Summarize the shipped parity behavior after implementation verification.
- Modify `docs/superpowers/specs/2026-05-30-analysis-workspace-parity-design.md`
  - Mark as shipped after implementation and verification.
- Move `docs/superpowers/specs/2026-05-30-analysis-workspace-parity-design.md` to `docs/superpowers/archive/specs/` after the implementation branch is verified and current-state docs are updated.
- Modify `docs/superpowers/specs/README.md` and `docs/superpowers/archive/specs/README.md`
  - Remove the active spec entry and add archive context when the spec is moved.
- Move this plan to `docs/superpowers/archive/plans/` after all tasks are complete and verified.
- Modify `docs/superpowers/plans/README.md` and `docs/superpowers/archive/plans/README.md`
  - Keep active/archived plan indexes accurate.

---

## Task 0: Preflight And Branch Sanity

**Files:**
- Read: `docs/superpowers/specs/2026-05-30-analysis-workspace-parity-design.md`
- Read: `src/lib/components/analysis/report-canvas.svelte`
- Read: `src/lib/components/analysis/report-setup-panel.svelte`
- Read: `src/lib/analysis-report-canvas.test.ts`

- [ ] **Step 1: Confirm branch and clean tree**

Run:

```powershell
git status --short --branch
```

Expected: branch is the implementation branch based on `analysis-workspace-parity-spec`, and there are no unrelated uncommitted changes.

- [ ] **Step 2: Inspect current editor ownership**

Run:

```powershell
rg -n "openedRunTemplateEditorOpen|openedRunGroupEditorOpen|templateEditorOpen|groupEditorOpen|TemplateEditor|SourceGroupEditor|NotebookLmExportDialog|opened-run-management|setup-secondary-actions" src/lib/components/analysis/report-canvas.svelte src/lib/components/analysis/report-setup-panel.svelte
```

Expected before implementation:

- `report-canvas.svelte` contains opened-run drawer state and opened-run management.
- `report-setup-panel.svelte` contains setup drawer state, editor imports, and setup secondary actions.
- `NotebookLmExportDialog` appears in `report-canvas.svelte`.

- [ ] **Step 3: Confirm current export guard**

Run:

```powershell
rg -n "function openNotebookLmExportDialog|analysisScope !== \"single_source\"|!currentSource\\(\\)|exportDialogOpen = true" src/routes/analysis/+page.svelte
```

Expected: the route guard keeps `NotebookLmExportDialog` from opening unless the live selected scope is a single source with `currentSource()`.

- [ ] **Step 4: Commit preflight note**

No files change in this task. Do not commit.

---

## Task 1: Add Failing Workspace Tools Component Contract Test

Task 1 is intentionally not committed until Task 2 makes the new component test
green. This preserves the red/green TDD evidence without leaving a red commit
in history.

**Files:**
- Create: `src/lib/analysis-workspace-tools.test.ts`

- [ ] **Step 1: Create `src/lib/analysis-workspace-tools.test.ts`**

Add:

```ts
import { describe, expect, it } from "vitest";
import reportWorkspaceToolsSource from "./components/analysis/report-workspace-tools.svelte?raw";

describe("analysis workspace tools component contract", () => {
  it("stays presentational and route-state agnostic", () => {
    expect(reportWorkspaceToolsSource).toContain("showNotebookLmExport");
    expect(reportWorkspaceToolsSource).toContain("canExportNotebookLm");
    expect(reportWorkspaceToolsSource).toContain("exportDisabledReason");
    expect(reportWorkspaceToolsSource).toContain("exportingNotebookLm");
    expect(reportWorkspaceToolsSource).toContain("templateEditorOpen");
    expect(reportWorkspaceToolsSource).toContain("groupEditorOpen");
    expect(reportWorkspaceToolsSource).toContain("onOpenNotebookLmExport");
    expect(reportWorkspaceToolsSource).toContain("onToggleTemplateEditor");
    expect(reportWorkspaceToolsSource).toContain("onToggleGroupEditor");

    expect(reportWorkspaceToolsSource).not.toContain("currentSource");
    expect(reportWorkspaceToolsSource).not.toContain("currentGroup");
    expect(reportWorkspaceToolsSource).not.toContain("currentRun");
    expect(reportWorkspaceToolsSource).not.toContain("workspaceSelection");
  });

  it("does not import APIs or call Tauri invoke", () => {
    expect(reportWorkspaceToolsSource).not.toContain("$lib/api");
    expect(reportWorkspaceToolsSource).not.toContain("@tauri-apps/api");
    expect(reportWorkspaceToolsSource).not.toContain("invoke(");
  });

  it("renders accessible source-group export disabled reason", () => {
    expect(reportWorkspaceToolsSource).toContain('const exportReasonId = "notebooklm-export-disabled-reason"');
    expect(reportWorkspaceToolsSource).toContain('id="notebooklm-export-disabled-reason"');
    expect(reportWorkspaceToolsSource).toContain("ariaDescribedby={exportDisabledReason ? exportReasonId : undefined}");
    expect(reportWorkspaceToolsSource).toContain("{exportDisabledReason}");
    expect(reportWorkspaceToolsSource).toContain('class="workspace-tool-helper"');
  });

  it("uses explicit button types for workspace actions", () => {
    expect(reportWorkspaceToolsSource.match(/type="button"/g)?.length ?? 0).toBe(3);
  });
});
```

- [ ] **Step 2: Run focused test and verify it fails**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-workspace-tools.test.ts
```

Expected: FAIL because `report-workspace-tools.svelte` does not exist yet.

Do not commit this task yet. Commit the test together with the component when the test is green in Task 2.

---

## Task 2: Create Presentational ReportWorkspaceTools

**Files:**
- Modify: `src/lib/components/ui/Button.svelte`
- Create: `src/lib/components/analysis/report-workspace-tools.svelte`
- Test: `src/lib/analysis-workspace-tools.test.ts`

- [ ] **Step 1: Add `ariaDescribedby` support to `Button.svelte`**

In the destructured props, change:

```ts
    ariaControls,
    ariaExpanded,
    tabIndex,
```

to:

```ts
    ariaControls,
    ariaExpanded,
    ariaDescribedby,
    tabIndex,
```

In the prop type, change:

```ts
    ariaControls?: string;
    ariaExpanded?: boolean;
    tabIndex?: number;
```

to:

```ts
    ariaControls?: string;
    ariaExpanded?: boolean;
    ariaDescribedby?: string;
    tabIndex?: number;
```

In the `<button>` element, change:

```svelte
  aria-controls={ariaControls}
  aria-expanded={ariaExpanded}
  tabindex={tabIndex}
```

to:

```svelte
  aria-controls={ariaControls}
  aria-expanded={ariaExpanded}
  aria-describedby={ariaDescribedby}
  tabindex={tabIndex}
```

- [ ] **Step 2: Add `report-workspace-tools.svelte`**

Create the file with:

```svelte
<script lang="ts">
  import { Download, Folder, SquarePen } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";

  let {
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

  const exportReasonId = "notebooklm-export-disabled-reason";
</script>

<section class="report-workspace-tools" aria-label="Workspace tools">
  <div class="workspace-tools-copy">
    <span class="eyebrow">Workspace tools</span>
  </div>

  <div class="workspace-tools-actions">
    {#if showNotebookLmExport}
      <div class="workspace-tool-action">
        <Button
          type="button"
          variant="secondary"
          onclick={onOpenNotebookLmExport}
          disabled={!canExportNotebookLm}
          ariaDescribedby={exportDisabledReason ? exportReasonId : undefined}
          title={exportDisabledReason ?? undefined}
        >
          <Download size={15} aria-hidden="true" />
          {exportingNotebookLm ? "Exporting..." : "Export for NotebookLM"}
        </Button>
        {#if exportDisabledReason}
          <span id={exportReasonId} class="workspace-tool-helper">{exportDisabledReason}</span>
        {/if}
      </div>
    {/if}

    <Button
      type="button"
      variant="secondary"
      ariaExpanded={templateEditorOpen}
      onclick={onToggleTemplateEditor}
    >
      <SquarePen size={15} aria-hidden="true" />
      {templateEditorOpen ? "Hide templates" : "Edit templates"}
    </Button>

    <Button
      type="button"
      variant="secondary"
      ariaExpanded={groupEditorOpen}
      onclick={onToggleGroupEditor}
    >
      <Folder size={15} aria-hidden="true" />
      {groupEditorOpen ? "Hide groups" : "Edit groups"}
    </Button>
  </div>
</section>

<style>
  .report-workspace-tools {
    display: flex;
    justify-content: space-between;
    gap: 0.8rem;
    align-items: center;
    padding: 0.85rem 1rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
    box-shadow: var(--shadow-soft);
  }

  .workspace-tools-copy {
    min-width: 0;
  }

  .workspace-tools-actions {
    display: flex;
    justify-content: flex-end;
    flex-wrap: wrap;
    gap: 0.45rem;
  }

  .workspace-tool-action {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    align-items: flex-start;
  }

  .workspace-tool-helper {
    max-width: 18rem;
    color: var(--muted);
    font-size: 0.74rem;
    line-height: 1.35;
  }

  .eyebrow {
    display: inline-block;
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
    margin-bottom: 0.2rem;
  }

  @media (max-width: 720px) {
    .report-workspace-tools {
      flex-direction: column;
      align-items: stretch;
    }

    .workspace-tools-actions {
      justify-content: flex-start;
    }
  }
</style>
```

The single-source exporting state is intentionally disabled without helper
text. `exportDisabledReason` is only for unsupported scope reasons, such as
source-group export not being implemented yet.

- [ ] **Step 3: Run focused component contract test**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-workspace-tools.test.ts
```

Expected: PASS.

- [ ] **Step 4: Commit component and now-green test**

Run:

```powershell
git add src/lib/components/ui/Button.svelte src/lib/components/analysis/report-workspace-tools.svelte src/lib/analysis-workspace-tools.test.ts
git commit -m "feat: add analysis workspace tools component"
```

Expected: commit succeeds.

---

## Task 3: Wire Workspace Tools Into ReportCanvas

**Files:**
- Modify: `src/lib/components/analysis/report-canvas.svelte`
- Test: `src/lib/analysis-report-canvas.test.ts`

- [ ] **Step 1: Update `src/lib/analysis-report-canvas.test.ts` imports**

Add the raw import near the other raw imports:

```ts
import reportWorkspaceToolsSource from "./components/analysis/report-workspace-tools.svelte?raw";
```

- [ ] **Step 2: Replace opened-run management test**

Replace the full `it("keeps management actions reachable while a run is open", ...)` block with:

```ts
  it("keeps workspace tools reachable before setup report and source bodies", () => {
    const toolsStart = reportCanvasSource.indexOf("<ReportWorkspaceTools");
    const reportBodyStart = reportCanvasSource.indexOf('{#if canvasMode === "report"}');
    const sourceBodyStart = reportCanvasSource.indexOf("<ReportSourceSurface");

    expect(toolsStart).toBeGreaterThan(0);
    expect(reportBodyStart).toBeGreaterThan(0);
    expect(sourceBodyStart).toBeGreaterThan(0);
    expect(toolsStart).toBeLessThan(reportBodyStart);
    expect(toolsStart).toBeLessThan(sourceBodyStart);
    expect(reportCanvasSource).not.toContain('class="opened-run-management"');
    expect(reportCanvasSource).toContain("<TemplateEditor");
    expect(reportCanvasSource).toContain("<SourceGroupEditor");
    expect(reportCanvasSource.match(/<TemplateEditor/g)?.length ?? 0).toBe(1);
    expect(reportCanvasSource.match(/<SourceGroupEditor/g)?.length ?? 0).toBe(1);
    expect(reportCanvasSource).toContain("templateEditorOpen");
    expect(reportCanvasSource).toContain("groupEditorOpen");
    expect(reportWorkspaceToolsSource).toContain("Export for NotebookLM");
    expect(reportWorkspaceToolsSource).toContain("Edit templates");
    expect(reportWorkspaceToolsSource).toContain("Edit groups");
  });
```

- [ ] **Step 3: Add export availability assertions**

Add this test inside the same `describe` block:

```ts
  it("derives NotebookLM export availability from live canvas source or group", () => {
    expect(reportCanvasSource).toContain("showNotebookLmExport");
    expect(reportCanvasSource).toContain("currentSource !== null || currentGroup !== null");
    expect(reportCanvasSource).toContain("canExportNotebookLm");
    expect(reportCanvasSource).toContain("currentSource !== null && !exportingNotebookLm");
    expect(reportCanvasSource).toContain("currentGroup && !currentSource ? sourceGroupNotebookLmExportReason : null");
    expect(reportCanvasSource).toContain("Source-group NotebookLM export is not implemented yet.");
    expect(reportCanvasSource).toContain("<ReportWorkspaceTools");
    expect(reportCanvasSource).toContain("{showNotebookLmExport}");
    expect(reportCanvasSource).toContain("{canExportNotebookLm}");
    expect(reportCanvasSource).toContain("exportDisabledReason={notebookLmExportDisabledReason}");
    expect(reportCanvasSource).toContain("<NotebookLmExportDialog");
    expect(reportCanvasSource.match(/<NotebookLmExportDialog/g)?.length ?? 0).toBe(1);
    expect(reportCanvasSource).toContain("source={currentSource}");
  });
```

- [ ] **Step 4: Run focused canvas test and verify it fails**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-report-canvas.test.ts
```

Expected: FAIL because `ReportCanvas` still renders opened-run-only management and does not import `ReportWorkspaceTools`.

- [ ] **Step 5: Update imports in `report-canvas.svelte`**

Remove lucide imports that moved into `ReportWorkspaceTools`:

```ts
import { Download, Folder, SquarePen } from "@lucide/svelte";
```

Add:

```ts
import ReportWorkspaceTools from "$lib/components/analysis/report-workspace-tools.svelte";
```

Keep this import because the `Report` / `Source` mode tabs still use `Button`:

```ts
import Button from "$lib/components/ui/Button.svelte";
```

- [ ] **Step 6: Rename opened-run drawer state**

Replace:

```ts
  let openedRunTemplateEditorOpen = $state(false);
  let openedRunGroupEditorOpen = $state(false);
```

with:

```ts
  let templateEditorOpen = $state(false);
  let groupEditorOpen = $state(false);
```

- [ ] **Step 7: Add export availability derived values**

Below `currentSourceContentLabel`, add:

```ts
  const showNotebookLmExport = $derived(currentSource !== null || currentGroup !== null);
  const sourceGroupNotebookLmExportReason = "Source-group NotebookLM export is not implemented yet.";
  const notebookLmExportDisabledReason = $derived(
    currentGroup && !currentSource ? sourceGroupNotebookLmExportReason : null,
  );
  const canExportNotebookLm = $derived(currentSource !== null && !exportingNotebookLm);
```

- [ ] **Step 8: Replace opened-run management markup**

Delete the entire block:

```svelte
  {#if currentRun}
    <div class="opened-run-management" aria-label="Opened run management">
      ...
    {/if}
  {/if}
```

Replace it immediately after the canvas toolbar with:

```svelte
  <ReportWorkspaceTools
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

  {#if templateEditorOpen}
    <div class="workspace-template-editor-drawer" aria-label="Template editor drawer">
      <TemplateEditor
        compact={true}
        {selectedTemplate}
        {templateName}
        {templateBody}
        {savingTemplate}
        {deletingTemplate}
        onSaveTemplateCopy={onSaveTemplateCopy}
        onSaveTemplateChanges={onSaveTemplateChanges}
        onDeleteTemplate={onDeleteTemplate}
      />
    </div>
  {/if}

  {#if groupEditorOpen}
    <div class="workspace-group-editor-drawer" aria-label="Source group editor drawer">
      <SourceGroupEditor
        compact={true}
        {groups}
        selectedGroupId={selectedGroupEditorId}
        {selectedGroup}
        {groupName}
        {groupSourceType}
        {groupMemberSourceIds}
        sources={sourceMetricsList}
        {savingGroup}
        {deletingGroup}
        {formatTimestamp}
        {isGroupSourceSelected}
        onChangeSelectedGroupId={onChangeSelectedGroupId}
        onChangeGroupName={onChangeGroupName}
        onChangeGroupSourceType={onChangeGroupSourceType}
        onToggleSource={onToggleGroupSource}
        onStartNewGroup={onStartNewGroup}
        onSaveGroupCopy={onSaveGroupCopy}
        onSaveGroupChanges={onSaveGroupChanges}
        onDeleteGroup={onDeleteGroup}
      />
    </div>
  {/if}
```

- [ ] **Step 9: Update styles in `report-canvas.svelte`**

Replace:

```css
  .opened-run-management,
  .opened-run-template-editor-drawer,
  .opened-run-group-editor-drawer {
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
    box-shadow: var(--shadow-soft);
  }

  .opened-run-management {
    display: flex;
    justify-content: space-between;
    gap: 0.8rem;
    align-items: center;
    padding: 0.85rem 1rem;
  }

  .opened-run-management-copy {
    min-width: 0;
  }

  .opened-run-management-actions {
    display: flex;
    justify-content: flex-end;
    flex-wrap: wrap;
    gap: 0.45rem;
  }

  .opened-run-template-editor-drawer,
  .opened-run-group-editor-drawer {
    padding: 0.85rem;
  }
```

with:

```css
  .workspace-template-editor-drawer,
  .workspace-group-editor-drawer {
    padding: 0.85rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
    box-shadow: var(--shadow-soft);
  }
```

In the mobile media query, replace:

```css
    .canvas-toolbar,
    .opened-run-management {
      flex-direction: column;
      align-items: stretch;
    }

    .opened-run-management-actions {
      justify-content: flex-start;
    }
```

with:

```css
    .canvas-toolbar {
      flex-direction: column;
      align-items: stretch;
    }
```

- [ ] **Step 10: Run focused canvas tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-report-canvas.test.ts src/lib/analysis-workspace-tools.test.ts
```

Expected: PASS for the canvas-level workspace tools assertions. Setup cleanup assertions are added in Task 4.

- [ ] **Step 11: Commit canvas wiring and canvas test updates**

Run:

```powershell
git add src/lib/components/analysis/report-canvas.svelte src/lib/analysis-report-canvas.test.ts
git commit -m "feat: render workspace tools from report canvas"
```

Expected: commit succeeds.

---

## Task 4: Clean ReportSetupPanel Ownership

**Files:**
- Modify: `src/lib/components/analysis/report-canvas.svelte`
- Modify: `src/lib/components/analysis/report-setup-panel.svelte`
- Test: `src/lib/analysis-report-setup-props.test.ts`
- Test: `src/lib/analysis-group-editor-props.test.ts`
- Test: `src/lib/analysis-redesign-route-contract.test.ts`
- Test: `src/lib/analysis-report-canvas.test.ts`

- [ ] **Step 1: Update setup ownership assertions in `src/lib/analysis-report-canvas.test.ts`**

In `it("shows setup only when no run is open and report mode is selected", ...)`, replace:

```ts
    expect(reportSetupPanelSource).toContain("TemplateEditor");
    expect(reportSetupPanelSource).toContain("SourceGroupEditor");
    expect(reportSetupPanelSource).toContain("{#if !startingReport && !selectedRunIsActive}");
    expect(reportSetupPanelSource).toContain('class="template-editor-drawer"');
    expect(reportSetupPanelSource).toContain('class="group-editor-drawer"');
```

with:

```ts
    expect(reportCanvasSource).toContain("<ReportWorkspaceTools");
    expect(reportSetupPanelSource).not.toContain("TemplateEditor");
    expect(reportSetupPanelSource).not.toContain("SourceGroupEditor");
    expect(reportSetupPanelSource).toContain("{#if !startingReport && !selectedRunIsActive}");
    expect(reportSetupPanelSource).not.toContain('class="template-editor-drawer"');
    expect(reportSetupPanelSource).not.toContain('class="group-editor-drawer"');
```

- [ ] **Step 2: Update `src/lib/analysis-redesign-route-contract.test.ts`**

In `it("keeps report setup out of the primary opened-run reading surface", ...)`, replace:

```ts
    expect(reportSetupSource).toContain("template-editor-drawer");
    expect(reportSetupSource).toContain("group-editor-drawer");
```

with:

```ts
    expect(reportSetupSource).not.toContain("template-editor-drawer");
    expect(reportSetupSource).not.toContain("group-editor-drawer");
    expect(reportCanvasSource).toContain("<ReportWorkspaceTools");
```

- [ ] **Step 3: Update `src/lib/analysis-group-editor-props.test.ts`**

Replace the test body with:

```ts
  it("keeps group editor selection owned by the report canvas workspace tools", () => {
    expect(analysisPageSource).toContain("selectedGroupEditorId={selectedGroupEditorId}");
    expect(reportCanvasSource).toContain("selectedGroupEditorId,");
    expect(reportCanvasSource).toContain("selectedGroupEditorId: string;");
    expect(reportCanvasSource).toContain("selectedGroupId={selectedGroupEditorId}");
    expect(reportSetupPanelSource).not.toContain("selectedGroupEditorId,");
    expect(reportSetupPanelSource).not.toContain("selectedGroupEditorId: string;");
    expect(reportCanvasSource).not.toContain("selectedGroupId,");
    expect(reportCanvasSource).not.toContain("selectedGroupId: string;");
    expect(reportSetupPanelSource).not.toContain("selectedGroupId={selectedGroupEditorId}");
  });
```

- [ ] **Step 4: Extend `src/lib/analysis-report-setup-props.test.ts`**

Add this test inside the existing `describe` block:

```ts
  it("keeps setup focused on report configuration and source preparation", () => {
    expect(reportSetupPanelSource).toContain("Run report");
    expect(reportSetupPanelSource).toContain("Sync source");
    expect(reportSetupPanelSource).toContain("selectedTemplate");
    expect(reportSetupPanelSource).toContain("reportLaunchDisabledReason");
    expect(reportSetupPanelSource).not.toContain("templateName");
    expect(reportSetupPanelSource).not.toContain("templateBody");
    expect(reportSetupPanelSource).not.toContain("savingTemplate");
    expect(reportSetupPanelSource).not.toContain("deletingTemplate");
    expect(reportSetupPanelSource).not.toContain("onSaveTemplateCopy");
    expect(reportSetupPanelSource).not.toContain("onSaveTemplateChanges");
    expect(reportSetupPanelSource).not.toContain("onDeleteTemplate");
    expect(reportSetupPanelSource).not.toContain("SourceGroupEditor");
    expect(reportSetupPanelSource).not.toContain("TemplateEditor");
  });
```

- [ ] **Step 5: Run focused setup tests and verify they fail**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-report-setup-props.test.ts src/lib/analysis-group-editor-props.test.ts src/lib/analysis-redesign-route-contract.test.ts src/lib/analysis-report-canvas.test.ts
```

Expected: FAIL because `ReportSetupPanel` still owns editor imports, props, buttons, and drawers.

- [ ] **Step 6: Remove editor imports**

Delete:

```ts
  import SourceGroupEditor from "$lib/components/analysis/source-group-editor.svelte";
  import TemplateEditor from "$lib/components/analysis/template-editor.svelte";
```

- [ ] **Step 7: Remove editor-only destructured props**

From the `let { ... }: { ... } = $props();` destructuring, delete:

```ts
    selectedGroupEditorId,
    templateName,
    templateBody,
    savingTemplate,
    deletingTemplate,
    groups,
    groupName,
    groupSourceType,
    groupMemberSourceIds,
    selectedGroup,
    savingGroup,
    deletingGroup,
    sourceMetricsList,
    isGroupSourceSelected,
    onSaveTemplateCopy,
    onSaveTemplateChanges,
    onDeleteTemplate,
    onChangeSelectedGroupId,
    onChangeGroupName,
    onChangeGroupSourceType,
    onToggleGroupSource,
    onStartNewGroup,
    onSaveGroupCopy,
    onSaveGroupChanges,
    onDeleteGroup,
```

Keep:

```ts
    selectedTemplate,
```

- [ ] **Step 8: Remove editor-only prop types**

From the props type object, delete:

```ts
    selectedGroupEditorId: string;
    templateName: string;
    templateBody: string;
    savingTemplate: boolean;
    deletingTemplate: boolean;
    groups: AnalysisSourceGroup[];
    groupName: string;
    groupSourceType: AnalysisGroupSourceType;
    groupMemberSourceIds: number[];
    selectedGroup: AnalysisSourceGroup | null;
    savingGroup: boolean;
    deletingGroup: boolean;
    sourceMetricsList: AnalysisSourceOption[];
    isGroupSourceSelected: (sourceId: number) => boolean;
    onSaveTemplateCopy: (name: string, body: string) => void | Promise<void>;
    onSaveTemplateChanges: (name: string, body: string) => void | Promise<void>;
    onDeleteTemplate: () => void | Promise<void>;
    onChangeSelectedGroupId: (value: string) => void;
    onChangeGroupName: (value: string) => void;
    onChangeGroupSourceType: (value: AnalysisGroupSourceType) => void;
    onToggleGroupSource: (sourceId: number) => void;
    onStartNewGroup: () => void;
    onSaveGroupCopy: () => void | Promise<void>;
    onSaveGroupChanges: () => void | Promise<void>;
    onDeleteGroup: () => void | Promise<void>;
```

- [ ] **Step 9: Remove editor-only attributes from the `ReportSetupPanel` tag**

In `src/lib/components/analysis/report-canvas.svelte`, inside the `<ReportSetupPanel ... />` tag, delete these attributes:

```svelte
        {templateName}
        {templateBody}
        {savingTemplate}
        {deletingTemplate}
        {groups}
        {groupName}
        {groupSourceType}
        {groupMemberSourceIds}
        {selectedGroup}
        {savingGroup}
        {deletingGroup}
        {sourceMetricsList}
        {selectedGroupEditorId}
        {isGroupSourceSelected}
        onSaveTemplateCopy={onSaveTemplateCopy}
        onSaveTemplateChanges={onSaveTemplateChanges}
        onDeleteTemplate={onDeleteTemplate}
        onChangeSelectedGroupId={onChangeSelectedGroupId}
        onChangeGroupName={onChangeGroupName}
        onChangeGroupSourceType={onChangeGroupSourceType}
        onToggleGroupSource={onToggleGroupSource}
        onStartNewGroup={onStartNewGroup}
        onSaveGroupCopy={onSaveGroupCopy}
        onSaveGroupChanges={onSaveGroupChanges}
        onDeleteGroup={onDeleteGroup}
```

Keep:

```svelte
        {selectedTemplate}
```

- [ ] **Step 10: Remove unused type imports**

After the prop removals, remove unused imports from:

```ts
  import type {
    AnalysisGroupSourceType,
    AnalysisPromptTemplate,
    AnalysisSourceGroup,
    AnalysisSourceOption,
    YoutubeCorpusMode,
  } from "$lib/types/analysis";
```

The import should become:

```ts
  import type {
    AnalysisPromptTemplate,
    AnalysisSourceGroup,
    AnalysisSourceOption,
    YoutubeCorpusMode,
  } from "$lib/types/analysis";
```

`AnalysisSourceGroup` and `AnalysisSourceOption` stay because setup still receives `currentGroup` and `currentSourceMetric`.

- [ ] **Step 11: Remove local editor drawer state**

Delete:

```ts
  let templateEditorOpen = $state(false);
  let groupEditorOpen = $state(false);
```

- [ ] **Step 12: Remove setup secondary actions and drawers**

Delete from markup:

```svelte
  <div class="setup-secondary-actions">
    ...
  </div>

  {#if templateEditorOpen}
    <div class="template-editor-drawer" aria-label="Template editor drawer">
      ...
    </div>
  {/if}

  {#if groupEditorOpen}
    <div class="group-editor-drawer" aria-label="Source group editor drawer">
      ...
    </div>
  {/if}
```

Do not delete the `Run report` and `Sync source` buttons in `.controls-actions`.

- [ ] **Step 13: Clean setup styles**

Remove `.template-editor-drawer`, `.group-editor-drawer`, and `.setup-secondary-actions` selectors from `report-setup-panel.svelte`.

Keep all selectors used by setup form, preflight, live strip, controls, and source sync.

- [ ] **Step 14: Run focused tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-report-setup-props.test.ts src/lib/analysis-group-editor-props.test.ts src/lib/analysis-redesign-route-contract.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-workspace-tools.test.ts
```

Expected: PASS.

- [ ] **Step 15: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: PASS with 0 errors and 0 warnings.

- [ ] **Step 16: Commit setup cleanup**

Run:

```powershell
git add src/lib/components/analysis/report-canvas.svelte src/lib/components/analysis/report-setup-panel.svelte src/lib/analysis-report-setup-props.test.ts src/lib/analysis-group-editor-props.test.ts src/lib/analysis-redesign-route-contract.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-workspace-tools.test.ts
git commit -m "refactor: centralize analysis workspace tools"
```

Expected: commit succeeds.

---

## Task 5: Verify Route Wiring And Guard Rails

**Files:**
- Test: `src/lib/analysis-report-canvas.test.ts`
- Test: `src/lib/analysis-workspace-tools.test.ts`
- Test: `src/lib/analysis-report-workspace-selection-props.test.ts`
- Inspect: `src/routes/analysis/+page.svelte`

- [ ] **Step 1: Inspect no duplicate export dialog**

Run:

```powershell
rg -n "<NotebookLmExportDialog" src/lib/components/analysis/report-canvas.svelte src/lib/components/analysis/report-setup-panel.svelte src/lib/components/analysis/report-workspace-tools.svelte
```

Expected: exactly one match, in `report-canvas.svelte`.

- [ ] **Step 2: Inspect no setup editor ownership remains**

Run:

```powershell
rg -n "TemplateEditor|SourceGroupEditor|template-editor-drawer|group-editor-drawer|setup-secondary-actions|templateEditorOpen|groupEditorOpen" src/lib/components/analysis/report-setup-panel.svelte
```

Expected: no matches.

- [ ] **Step 3: Inspect workspace tools do not know route state**

Run:

```powershell
rg -n "currentSource|currentGroup|currentRun|workspaceSelection|invoke\\(|\\$lib/api|@tauri-apps/api" src/lib/components/analysis/report-workspace-tools.svelte
```

Expected: no matches.

- [ ] **Step 4: Run related frontend tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-workspace-tools.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-report-setup-props.test.ts src/lib/analysis-group-editor-props.test.ts src/lib/analysis-report-workspace-selection-props.test.ts src/lib/analysis-redesign-route-contract.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit guard verification if fixes were needed**

If Steps 1-4 required fixes, commit them:

```powershell
git add src/lib/components/analysis/report-canvas.svelte src/lib/components/analysis/report-setup-panel.svelte src/lib/components/analysis/report-workspace-tools.svelte src/lib/*.test.ts
git commit -m "test: verify analysis workspace tool guard rails"
```

Expected: commit succeeds only if files changed. If no files changed, skip this commit.

---

## Task 6: Update Current-State Docs

**Files:**
- Modify: `docs/project.md`
- Modify: `docs/design-document.md`

- [ ] **Step 1: Update `docs/project.md` product slice**

Find this bullet in the `Implemented:` list:

```md
- result-first `/analysis` workspace with compact source rail, central report/source canvas, and evidence/chat/chunks/runs companion panel
```

Replace it with:

```md
- result-first `/analysis` workspace with compact source rail, central report/source canvas, shared workspace tools for setup and opened runs, and evidence/chat/chunks/runs companion panel
```

- [ ] **Step 2: Update `/analysis` route details in `docs/project.md`**

In the `/analysis` route bullet list, after:

```md
  - use the result-first research workspace layout
```

add:

```md
  - keep NotebookLM export, template editing, and group editing reachable from
    shared canvas-level workspace tools in setup and opened-run states
```

- [ ] **Step 3: Update `docs/design-document.md` frontend workflow paragraph**

Find the paragraph beginning:

```md
The current frontend workflow is result-first: `/analysis` keeps the opened
report or source material in the central canvas, while source switching,
evidence, follow-up chat, live chunk summaries, and saved runs stay nearby.
```

Replace it with:

```md
The current frontend workflow is result-first: `/analysis` keeps setup,
opened reports, or source material in the central canvas, while source
switching, evidence, follow-up chat, live chunk summaries, and saved runs stay
nearby. Canvas-level workspace tools keep NotebookLM export, template editing,
and group editing reachable in both setup and opened-run states.
```

- [ ] **Step 4: Verify doc text**

Run:

```powershell
rg -n "shared workspace tools|canvas-level workspace tools|NotebookLM export, template editing, and group editing" docs/project.md docs/design-document.md
```

Expected: matches in both docs.

- [ ] **Step 5: Commit docs**

Run:

```powershell
git add docs/project.md docs/design-document.md
git commit -m "docs: record analysis workspace parity"
```

Expected: commit succeeds.

---

## Task 7: Full Verification

**Files:**
- Entire repository verification

- [ ] **Step 1: Run focused frontend tests**

Run:

```powershell
npm.cmd run test -- src/lib/analysis-workspace-tools.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-report-setup-props.test.ts src/lib/analysis-group-editor-props.test.ts src/lib/analysis-report-workspace-selection-props.test.ts src/lib/analysis-redesign-route-contract.test.ts
```

Expected: PASS.

- [ ] **Step 2: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: PASS with 0 errors and 0 warnings.

- [ ] **Step 3: Run full verification**

Run:

```powershell
npm.cmd run verify
```

Expected: PASS for frontend tests, Svelte check, Rust check/tests, and `git diff HEAD --check`.

- [ ] **Step 4: Commit verification note**

Modify this plan's task checkboxes for completed verification steps, then commit:

```powershell
git add docs/superpowers/plans/2026-05-30-analysis-workspace-parity-implementation.md
git commit -m "docs: mark analysis workspace parity verified"
```

Expected: commit succeeds.

---

## Task 8: Archive Spec And Plan After Implementation

**Files:**
- Move: `docs/superpowers/specs/2026-05-30-analysis-workspace-parity-design.md` -> `docs/superpowers/archive/specs/2026-05-30-analysis-workspace-parity-design.md`
- Move: `docs/superpowers/plans/2026-05-30-analysis-workspace-parity-implementation.md` -> `docs/superpowers/archive/plans/2026-05-30-analysis-workspace-parity-implementation.md`
- Modify: `docs/superpowers/specs/README.md`
- Modify: `docs/superpowers/archive/specs/README.md`
- Modify: `docs/superpowers/plans/README.md`
- Modify: `docs/superpowers/archive/plans/README.md`

- [ ] **Step 1: Mark spec shipped**

Before moving the spec, change its header to:

```md
> Status: shipped on 2026-05-30; archived for rationale and regression context
```

- [ ] **Step 2: Move the spec**

Run:

```powershell
Move-Item -LiteralPath 'docs/superpowers/specs/2026-05-30-analysis-workspace-parity-design.md' -Destination 'docs/superpowers/archive/specs/2026-05-30-analysis-workspace-parity-design.md'
```

Expected: file exists only under `docs/superpowers/archive/specs/`.

- [ ] **Step 3: Inspect active spec index**

Run:

```powershell
Get-Content -Path 'docs/superpowers/specs/README.md'
```

Expected: the active specs list includes
`2026-05-30-analysis-workspace-parity-design.md`. If other active specs are
present, keep them.

- [ ] **Step 4: Update specs READMEs**

In `docs/superpowers/specs/README.md`, remove only the
`2026-05-30-analysis-workspace-parity-design.md` active entry. If no active
spec entries remain, make the active list:

```md
- None currently.
```

In `docs/superpowers/archive/specs/README.md`, add:

```md
Analysis workspace parity specs record shipped canvas-level workspace tool
contracts for setup and opened-run states.
```

- [ ] **Step 5: Commit spec archive**

Run:

```powershell
git add docs/superpowers/specs/README.md docs/superpowers/archive/specs/README.md docs/superpowers/archive/specs/2026-05-30-analysis-workspace-parity-design.md
git add -u docs/superpowers/specs/2026-05-30-analysis-workspace-parity-design.md
git commit -m "docs: archive analysis workspace parity spec"
```

Expected: commit succeeds.

- [ ] **Step 6: Move this plan**

Run:

```powershell
Move-Item -LiteralPath 'docs/superpowers/plans/2026-05-30-analysis-workspace-parity-implementation.md' -Destination 'docs/superpowers/archive/plans/2026-05-30-analysis-workspace-parity-implementation.md'
```

Expected: file exists only under `docs/superpowers/archive/plans/`.

- [ ] **Step 7: Update plan READMEs**

In `docs/superpowers/plans/README.md`, keep the active-plan guidance and no explicit active plan entry.

In `docs/superpowers/archive/plans/README.md`, add:

```md
Analysis workspace parity plans record the shipped canvas-level workspace tool
cleanup that unified setup and opened-run actions.
```

- [ ] **Step 8: Commit plan archive**

Run:

```powershell
git add docs/superpowers/plans/README.md docs/superpowers/archive/plans/README.md docs/superpowers/archive/plans/2026-05-30-analysis-workspace-parity-implementation.md
git add -u docs/superpowers/plans/2026-05-30-analysis-workspace-parity-implementation.md
git commit -m "docs: archive analysis workspace parity plan"
```

Expected: commit succeeds.

---

## Final Manual Smoke

Run the app only after `npm.cmd run verify` passes.

Manual smoke checklist:

- single-source setup: `Workspace tools` appears below canvas tabs; export opens NotebookLM dialog; template and group editors open below tools; `Run report` and `Sync source` remain in setup;
- source-group setup: export button is visible but disabled; disabled reason is visible; template and group editors open below tools;
- opened single-source run in report mode: same tools are visible; export opens; drawers open below tools;
- opened source-group run in report mode: export is visible but disabled; disabled reason is visible; drawers open;
- source mode for single-source and source-group selections: tools remain in the same location above source body;
- opened saved run with missing/restored-null `currentSource`: export does not open with `source={null}`; source-group runs show disabled source-group affordance if `currentGroup` is available.

If manual smoke requires Tauri fixtures, use the existing analysis redesign fixture flow from prior Source Browser smoke notes and record any new issue before merging.
