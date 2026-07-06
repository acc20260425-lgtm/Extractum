# Projects Next v11 Source Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring the `/projects/next` source-management slice closer to the canonical v11 handoff while preserving current source workflows and visible copy.

**Architecture:** Keep the current Svelte 5 route state and component composition. Add one source-table layout contract next to the existing source-grid column definitions, then reuse it from `SourcesGrid` and `SourcesFilterRow`. Polish the existing Svelte/SVAR components with token-driven CSS rather than porting `.dc.html` prototype code.

**Tech Stack:** Svelte 5 runes, TypeScript, Vitest, Testing Library Svelte, SVAR `@svar-ui/svelte-grid` (`width` and lowercase `flexgrow` column sizing), Extractum UI wrappers, Tauri MCP for visual verification.

## Global Constraints

- Visual source: `reference/tauri-mcp-bridge-connection/project/design_handoff_research_projects_v11`.
- Scope is limited to `ProjectToolbar`, project tabs, source filter/stats bar, selected-source bulk bar, source filter row, and source table.
- Do not redesign the left rail/project list, inspector, or run dock.
- Do not change backend behavior, project data flow, source actions, or visible product copy.
- Keep visible text, `aria-label`, and `title` synchronized for each source-slice action.
- Keep `Add source`, `Connect from Library`, and existing `Delete from Library` strings unchanged in this slice.
- Use existing `--extractum-*` tokens from `src/lib/styles/base.css`; do not introduce a new palette or theme file.
- `--extractum-density-row-height` is the row-height target and is currently `34px`.
- Keep `data-ui-action="add-source"` and `data-ui-action="connect-library"`.
- Use SVAR DataGrid; do not replace it with a hand-rolled table.
- Do not copy `.dc.html`, inline prototype runtime code, or `support.js`.
- Use `npm.cmd`, not `npm`, for validation commands on Windows.
- Use the official Svelte MCP before editing Svelte components and again after component fixes.
- Do not stage `.claude/settings.local.json` or `.tmp-current-projects-next.png`.

---

## File Structure

- `src/lib/ui/research-projects-source-row.ts`
  - Owns source row view mapping, sort comparators, source grid column definitions, and the new shared source-table layout contract.
- `src/lib/ui/research-projects-source-row.test.ts`
  - Pure unit tests for row mapping, sort comparators, column sizing, and layout-contract consistency.
- `src/lib/components/research-projects/SourcesGrid.svelte`
  - Consumes the shared select-column width and existing source column definitions, keeps SVAR DataGrid behavior.
- `src/lib/components/research-projects/SourcesGrid.test.ts`
  - Raw-source contract tests for DataGrid wiring and layout constant usage.
- `src/lib/components/research-projects/SourcesFilterRow.svelte`
  - Consumes the shared filter-row grid template and keeps current filter interactions.
- `src/lib/components/research-projects/SourcesFilterRow.test.ts`
  - Rendering tests for current filter interactions plus grid-template wiring.
- `src/lib/components/research-projects/SourcesFilterBar.svelte`
  - Token-driven visual polish only; current copy, labels, titles, and callbacks stay.
- `src/lib/components/research-projects/SourcesFilterBar.test.ts`
  - Keeps accessible-name action tests and adds assertions for synchronized labels/titles.
- `src/lib/components/research-projects/SourcesBulkBar.svelte`
  - Token-driven visual polish only; current confirmation behavior and copy stay.
- `src/lib/components/research-projects/SourcesBulkBar.test.ts`
  - Keeps existing behavior tests and adds minimal contract coverage if markup changes.
- `src/lib/components/research-projects/ProjectTabs.svelte`
  - Tighten v11 tab-row styling while keeping current section ids and labels.
- `src/lib/components/research-projects/ProjectTabs.test.ts`
  - Existing behavioral tests plus optional raw CSS contract for 40px row.
- `src/lib/components/research-projects/ProjectToolbar.svelte`
  - Small visual adjustment only if needed after screenshot comparison; no selector behavior changes.
- `src/lib/components/research-projects/ResearchProjectsShell.svelte`
  - Tighten seams between toolbar, tabs, stats/bulk bar, filter row, and grid.

---

### Task 1: Shared Source Table Layout Contract

**Files:**
- Modify: `src/lib/ui/research-projects-source-row.ts`
- Modify: `src/lib/ui/research-projects-source-row.test.ts`
- Modify: `src/lib/components/research-projects/SourcesGrid.svelte`
- Modify: `src/lib/components/research-projects/SourcesGrid.test.ts`
- Modify: `src/lib/components/research-projects/SourcesFilterRow.svelte`
- Modify: `src/lib/components/research-projects/SourcesFilterRow.test.ts`

**Interfaces:**
- Produces `SOURCE_TABLE_LAYOUT`, consumed by `sourceGridColumns`, `SourcesGrid.svelte`, and tests.
- Produces `SOURCE_FILTER_ROW_GRID_TEMPLATE`, consumed by `SourcesFilterRow.svelte` and tests.
- Keeps `sourceGridColumns(): ExtractumDataGridColumn[]`.
- Uses confirmed SVAR column sizing properties: `width` for fixed pixel width and lowercase `flexgrow` for flexible width.

- [ ] **Step 1: Add failing layout-contract tests**

Modify imports in `src/lib/ui/research-projects-source-row.test.ts`:

```ts
import {
  SOURCE_FILTER_ROW_GRID_TEMPLATE,
  SOURCE_TABLE_LAYOUT,
  buildSourceGridRows,
  buildSourceRow,
  compareSourceLastSynced,
  compareSourceMaterials,
  compareSourceTitles,
  sourceGridColumns,
  sourceSyncStatusLabel,
} from "./research-projects-source-row";
```

Add inside the existing `sourceGridColumns` describe block:

```ts
  it("uses one v11 source-table layout contract for grid columns and filter row", () => {
    const columns = sourceGridColumns();
    const byId = new Map(columns.map((column) => [String(column.id), column]));

    expect(SOURCE_TABLE_LAYOUT).toEqual({
      select: 34,
      titleMin: 160,
      titleFlexGrow: 1,
      type: 116,
      materials: 116,
      lastSync: 150,
      status: 104,
    });
    expect(SOURCE_FILTER_ROW_GRID_TEMPLATE).toBe(
      "34px minmax(160px, 1fr) 116px 116px 150px 104px",
    );

    expect(byId.get("title")?.flexgrow).toBe(SOURCE_TABLE_LAYOUT.titleFlexGrow);
    expect(byId.get("title")?.width).toBeUndefined();
    expect(byId.get("typeLabel")?.width).toBe(SOURCE_TABLE_LAYOUT.type);
    expect(byId.get("materialsLabel")?.width).toBe(SOURCE_TABLE_LAYOUT.materials);
    expect(byId.get("lastSyncedAt")?.width).toBe(SOURCE_TABLE_LAYOUT.lastSync);
    expect(byId.get("statusLabel")?.width).toBe(SOURCE_TABLE_LAYOUT.status);
  });
```

Modify `src/lib/components/research-projects/SourcesGrid.test.ts` by adding to the first test:

```ts
    expect(source).toContain("SOURCE_TABLE_LAYOUT");
    expect(source).toContain("width: SOURCE_TABLE_LAYOUT.select");
```

Modify `src/lib/components/research-projects/SourcesFilterRow.test.ts` imports:

```ts
import { SOURCE_FILTER_ROW_GRID_TEMPLATE } from "$lib/ui/research-projects-source-row";
```

Add this test:

```ts
  it("uses the shared source table grid template", () => {
    render(SourcesFilterRow, { props: { filters: emptySourceFilters() } });
    const row = document.querySelector(".sources-filter-row") as HTMLElement | null;

    expect(row?.getAttribute("style")).toContain(
      `grid-template-columns: ${SOURCE_FILTER_ROW_GRID_TEMPLATE}`,
    );
  });
```

- [ ] **Step 2: Run focused tests to verify red**

Run:

```powershell
npm.cmd run test -- src/lib/ui/research-projects-source-row.test.ts src/lib/components/research-projects/SourcesGrid.test.ts src/lib/components/research-projects/SourcesFilterRow.test.ts
```

Expected: FAIL because `SOURCE_TABLE_LAYOUT` and `SOURCE_FILTER_ROW_GRID_TEMPLATE` are not exported and the components do not consume them yet.

- [ ] **Step 3: Implement layout constants in the source-row UI module**

Modify `src/lib/ui/research-projects-source-row.ts` near the top:

```ts
export const SOURCE_TABLE_LAYOUT = {
  select: 34,
  titleMin: 160,
  titleFlexGrow: 1,
  type: 116,
  materials: 116,
  lastSync: 150,
  status: 104,
} as const;

export const SOURCE_FILTER_ROW_GRID_TEMPLATE = [
  `${SOURCE_TABLE_LAYOUT.select}px`,
  `minmax(${SOURCE_TABLE_LAYOUT.titleMin}px, 1fr)`,
  `${SOURCE_TABLE_LAYOUT.type}px`,
  `${SOURCE_TABLE_LAYOUT.materials}px`,
  `${SOURCE_TABLE_LAYOUT.lastSync}px`,
  `${SOURCE_TABLE_LAYOUT.status}px`,
].join(" ");
```

Modify `sourceGridColumns()` in the same file:

```ts
export function sourceGridColumns(): ExtractumDataGridColumn[] {
  return [
    {
      id: "title",
      header: "Источник",
      flexgrow: SOURCE_TABLE_LAYOUT.titleFlexGrow,
      sort: compareSourceTitles,
    },
    { id: "typeLabel", header: "Тип", width: SOURCE_TABLE_LAYOUT.type, sort: true },
    {
      id: "materialsLabel",
      header: "Материалы",
      width: SOURCE_TABLE_LAYOUT.materials,
      sort: compareSourceMaterials,
    },
    {
      id: "lastSyncedAt",
      header: "Последний сбор",
      width: SOURCE_TABLE_LAYOUT.lastSync,
      dateTimeFormat: "datetime",
      sort: compareSourceLastSynced,
    },
    { id: "statusLabel", header: "Статус", width: SOURCE_TABLE_LAYOUT.status, sort: true },
  ];
}
```

Important: do not set `width` on the title column. SVAR docs confirm `flexgrow` has no effect when `width` is explicitly set.

Do not add a `minWidth`/`minwidth` field to the SVAR title column. The installed `@svar-ui/svelte-grid` `IColumnConfig` exposes `width` and `flexgrow`, but no minimum-width property. The title lower bound is handled by `SOURCE_FILTER_ROW_GRID_TEMPLATE` and by the live header-fit verification in Task 4.

- [ ] **Step 4: Consume select width in `SourcesGrid.svelte`**

Modify the import from `$lib/ui/research-projects-source-row`:

```ts
  import {
    SOURCE_TABLE_LAYOUT,
    buildSourceGridRows,
    sourceGridColumns,
  } from "$lib/ui/research-projects-source-row";
```

Modify `SELECT_COLUMN`:

```ts
  const SELECT_COLUMN: ExtractumDataGridColumn = {
    id: "selected",
    header: { cell: GridSelectAllCell } as unknown as ExtractumDataGridColumn["header"],
    width: SOURCE_TABLE_LAYOUT.select,
    cell: GridSelectCell as unknown as ExtractumDataGridColumn["cell"],
  };
```

- [ ] **Step 5: Consume the shared grid template in `SourcesFilterRow.svelte`**

Add the import:

```ts
  import { SOURCE_FILTER_ROW_GRID_TEMPLATE } from "$lib/ui/research-projects-source-row";
```

Modify the root element:

```svelte
<div class="sources-filter-row" style={`grid-template-columns: ${SOURCE_FILTER_ROW_GRID_TEMPLATE};`}>
```

Remove this CSS declaration from `.sources-filter-row`:

```css
grid-template-columns: 34px minmax(160px, 1fr) 116px 116px 150px 104px;
```

- [ ] **Step 6: Run focused tests to verify green**

Run:

```powershell
npm.cmd run test -- src/lib/ui/research-projects-source-row.test.ts src/lib/components/research-projects/SourcesGrid.test.ts src/lib/components/research-projects/SourcesFilterRow.test.ts
```

Expected: PASS.

- [ ] **Step 7: Commit Task 1**

Run:

```powershell
git add src/lib/ui/research-projects-source-row.ts src/lib/ui/research-projects-source-row.test.ts src/lib/components/research-projects/SourcesGrid.svelte src/lib/components/research-projects/SourcesGrid.test.ts src/lib/components/research-projects/SourcesFilterRow.svelte src/lib/components/research-projects/SourcesFilterRow.test.ts
git commit -m "refactor: share projects source table layout"
```

---

### Task 2: Source Control Bars Visual Polish

**Files:**
- Modify: `src/lib/components/research-projects/SourcesFilterBar.svelte`
- Modify: `src/lib/components/research-projects/SourcesFilterBar.test.ts`
- Modify: `src/lib/components/research-projects/SourcesBulkBar.svelte`
- Modify: `src/lib/components/research-projects/SourcesBulkBar.test.ts`

**Interfaces:**
- Consumes existing props of `SourcesFilterBar` and `SourcesBulkBar`; no prop additions are required.
- Preserves `aria-label="Add source"`, `title="Add source"`, visible `Add source`, `aria-label="Connect from Library"`, `title="Connect from Library"`, and visible `Connect from Library`.
- Preserves `Delete from Library` accessible names and confirmation behavior.

- [ ] **Step 1: Add accessibility/copy synchronization tests for filter actions**

Add to `src/lib/components/research-projects/SourcesFilterBar.test.ts`:

```ts
  it("keeps visible copy, aria-label and title synchronized for source actions", () => {
    render(SourcesFilterBar, { props: { ...base } });

    const add = screen.getByRole("button", { name: "Add source" });
    const connect = screen.getByRole("button", { name: "Connect from Library" });

    expect(add.textContent?.replace(/\s+/g, " ").trim()).toBe("Add source");
    expect(add.getAttribute("aria-label")).toBe("Add source");
    expect(add.getAttribute("title")).toBe("Add source");
    expect(add.getAttribute("data-ui-action")).toBe("add-source");

    expect(connect.textContent?.replace(/\s+/g, " ").trim()).toBe("Connect from Library");
    expect(connect.getAttribute("aria-label")).toBe("Connect from Library");
    expect(connect.getAttribute("title")).toBe("Connect from Library");
    expect(connect.getAttribute("data-ui-action")).toBe("connect-library");
  });
```

- [ ] **Step 2: Add bulk bar copy-preservation test**

Add to `src/lib/components/research-projects/SourcesBulkBar.test.ts`:

```ts
  it("keeps Delete from Library visible copy and accessible names stable", () => {
    render(SourcesBulkBar, {
      props: {
        count: 1,
        libraryDeleteDisabled: false,
      },
    });

    const button = screen.getByRole("button", { name: "Delete from Library" });
    expect(button.textContent?.replace(/\s+/g, " ").trim()).toBe("Delete from Library");
  });
```

- [ ] **Step 3: Run focused tests before CSS changes**

Run:

```powershell
npm.cmd run test -- src/lib/components/research-projects/SourcesFilterBar.test.ts src/lib/components/research-projects/SourcesBulkBar.test.ts
```

Expected: PASS. These tests lock copy and hooks before visual edits.

- [ ] **Step 4: Polish `SourcesFilterBar.svelte` styles without renaming actions**

Keep the current markup and callbacks. Replace the `.sources-filter-bar` and action button style block with token-driven v11-like density:

```css
  .sources-filter-bar {
    min-height: 42px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    padding: 6px 14px;
    background: var(--extractum-surface-raised);
    border-bottom: 1px solid color-mix(in srgb, var(--extractum-border) 72%, transparent);
  }

  .sources-filter-bar__left {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    min-width: 0;
  }

  .sources-filter-bar__left .sources-filter-bar__filters-btn {
    height: 28px;
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 0 10px;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface);
    color: var(--extractum-text);
    font: 600 12px/1 var(--extractum-font);
    cursor: pointer;
  }

  .sources-filter-bar__actions {
    display: inline-flex;
    flex-shrink: 0;
    align-items: center;
    gap: 8px;
  }

  .sources-filter-bar__actions > button {
    height: 30px;
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 0 11px;
    border-radius: var(--extractum-radius);
    font: 600 12.5px/1 var(--extractum-font);
    cursor: pointer;
    white-space: nowrap;
  }

  .sources-filter-bar__add {
    border: 1px solid var(--extractum-primary);
    background: color-mix(in srgb, var(--extractum-primary) 7%, transparent);
    color: var(--extractum-primary);
  }

  .sources-filter-bar__connect {
    border: 1px solid var(--extractum-primary);
    background: var(--extractum-primary);
    color: #fff;
    box-shadow: 0 1px 2px color-mix(in srgb, var(--extractum-primary) 26%, transparent);
  }
```

Keep the existing hover rules but adjust them to:

```css
  .sources-filter-bar__add:hover {
    background: color-mix(in srgb, var(--extractum-primary) 12%, transparent);
  }

  .sources-filter-bar__connect:hover {
    background: var(--extractum-primary-hover);
  }
```

Do not remove `data-ui-action`, `aria-label`, or `title`.

- [ ] **Step 5: Polish `SourcesBulkBar.svelte` styles without changing copy**

Keep markup and dialogs. Replace `.sources-bulk-bar` styles with:

```css
  .sources-bulk-bar {
    position: absolute;
    inset: 0;
    z-index: 5;
    min-height: 42px;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 14px;
    background: color-mix(in srgb, var(--extractum-primary) 10%, var(--extractum-surface-raised));
    border-bottom: 1px solid color-mix(in srgb, var(--extractum-primary) 28%, transparent);
    font: 400 12.5px/1.35 var(--extractum-font);
    color: var(--extractum-text);
  }

  .sources-bulk-bar__count {
    font-weight: 700;
    color: var(--extractum-primary);
  }

  .sources-bulk-bar__clear {
    background: none;
    border: none;
    padding: 0;
    color: var(--extractum-primary);
    cursor: pointer;
    font: 600 12px/1 var(--extractum-font);
    text-decoration: underline;
    text-underline-offset: 2px;
  }
```

Keep the dialog styles unless Svelte check reports an issue.

- [ ] **Step 6: Run focused tests**

Run:

```powershell
npm.cmd run test -- src/lib/components/research-projects/SourcesFilterBar.test.ts src/lib/components/research-projects/SourcesBulkBar.test.ts
```

Expected: PASS.

- [ ] **Step 7: Commit Task 2**

Run:

```powershell
git add src/lib/components/research-projects/SourcesFilterBar.svelte src/lib/components/research-projects/SourcesFilterBar.test.ts src/lib/components/research-projects/SourcesBulkBar.svelte src/lib/components/research-projects/SourcesBulkBar.test.ts
git commit -m "style: polish projects source action bars"
```

---

### Task 3: Toolbar, Tabs, Shell, and Grid Density Polish

**Files:**
- Modify: `src/lib/components/research-projects/ProjectTabs.svelte`
- Modify: `src/lib/components/research-projects/ProjectTabs.test.ts`
- Modify: `src/lib/components/research-projects/ProjectToolbar.svelte`
- Modify: `src/lib/components/research-projects/ProjectToolbar.test.ts`
- Modify: `src/lib/components/research-projects/ResearchProjectsShell.svelte`
- Modify: `src/lib/components/research-projects/ResearchProjectsShell.test.ts`
- Modify: `src/lib/components/research-projects/SourcesGrid.svelte`
- Modify: `src/lib/components/research-projects/SourcesGrid.test.ts`
- Modify only if needed: `src/lib/components/extractum-ui/DataGrid.svelte`

**Interfaces:**
- Consumes Task 1 source-table layout constants.
- Keeps existing Svelte component props unchanged.
- Keeps SVAR DataGrid date formatting through raw values plus `dateTimeFormat`.

- [ ] **Step 1: Add raw contract tests for v11 density and shell composition**

In `src/lib/components/research-projects/ProjectTabs.test.ts`, add:

```ts
import rawSource from "./ProjectTabs.svelte?raw";

const source = rawSource.replace(/\r\n/g, "\n");
```

Add:

```ts
  it("keeps the v11 compact tab row contract", () => {
    expect(source).toContain("height: 40px");
    expect(source).toContain("box-shadow: inset 0 -2px 0 var(--extractum-primary)");
  });
```

In `src/lib/components/research-projects/ResearchProjectsShell.test.ts`, add or extend the existing raw-source test with:

```ts
    expect(source).toContain("research-projects-shell__statsbar");
    expect(source).toContain("<SourcesFilterBar {...filterBar} />");
    expect(source).toContain("<SourcesBulkBar {...bulkBar} />");
    expect(source).toContain("<SourcesFilterRow {...filterRow} />");
```

In `src/lib/components/research-projects/SourcesGrid.test.ts`, add:

```ts
  it("keeps SVAR as the source table implementation", () => {
    expect(source).toContain("<ExtractumDataGrid");
    expect(source).not.toContain("<table");
  });
```

- [ ] **Step 2: Run focused tests before style changes**

Run:

```powershell
npm.cmd run test -- src/lib/components/research-projects/ProjectTabs.test.ts src/lib/components/research-projects/ResearchProjectsShell.test.ts src/lib/components/research-projects/SourcesGrid.test.ts src/lib/components/research-projects/ProjectToolbar.test.ts
```

Expected: PASS for the existing composition checks and the current `ProjectTabs` raw assertions (`height: 40px` and active underline already exist). If this fails, stop and inspect the current component before editing; do not weaken the raw assertions.

- [ ] **Step 3: Tighten `ResearchProjectsShell.svelte` seams**

Keep markup composition. Adjust styles:

```css
  .research-projects-shell__main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    min-height: 0;
    background: var(--extractum-surface);
  }

  .research-projects-shell__statsbar {
    position: relative;
    flex-shrink: 0;
    min-height: 42px;
  }

  .research-projects-shell__grid {
    flex: 1;
    min-height: 0;
    min-width: 0;
    background: var(--extractum-surface-raised);
  }
```

Do not change rail, inspector, or run dock styles in this task.

- [ ] **Step 4: Tighten `ProjectTabs.svelte` styles**

Keep existing labels and section ids. Ensure the style block contains:

```css
  .project-tabs {
    height: 40px;
    flex-shrink: 0;
    display: flex;
    align-items: stretch;
    gap: 20px;
    padding: 0 16px;
    background: var(--extractum-surface);
    border-bottom: 1px solid var(--extractum-border);
  }

  .project-tabs .project-tabs__tab {
    display: flex;
    align-items: center;
    padding: 0;
    border: none;
    background: transparent;
    font: 600 13px/1 var(--extractum-font);
    color: var(--extractum-muted);
    cursor: pointer;
  }
```

Keep active-state underline:

```css
  .project-tabs .project-tabs__tab[aria-selected="true"] {
    font-weight: 700;
    color: var(--extractum-primary);
    box-shadow: inset 0 -2px 0 var(--extractum-primary);
  }
```

- [ ] **Step 5: Adjust `ProjectToolbar.svelte` only for small visual mismatches**

Before editing, compare the current toolbar screenshot with v11. If only spacing needs tightening, keep behavior and adjust:

```css
  .project-toolbar {
    min-height: 54px;
    padding: 8px 14px;
    background: var(--extractum-surface-raised);
    border-bottom: 1px solid var(--extractum-border);
  }
```

Do not change popover behavior, selector state, or run-disabled logic.

- [ ] **Step 6: Tune `SourcesGrid.svelte` wrapper styles with a scoped class**

Keep SVAR DataGrid. Add a local class to the `ExtractumDataGrid` invocation so source-table polish does not leak to Library, analysis, or other grids:

```svelte
<ExtractumDataGrid
  class="sources-grid__table"
  {rows}
  {columns}
  {columnStyle}
  selectedRowIds={selectedSourceIds}
  multiselect={true}
  onSelectedRowIdsChange={onSelectedSourceIdsChange}
  height="100%"
  ariaLabel="Источники проекта"
  {overlay}
  selectOnClick={false}
  activeRowId={activeSourceId}
  onRowClick={onActivateSource}
/>
```

Add the source-table-only rules in `SourcesGrid.svelte`:

```css
  :global(.sources-grid__table .wx-header .wx-cell) {
    font-weight: 700;
    color: var(--extractum-text);
    background: var(--extractum-surface-subtle);
  }

  :global(.sources-grid__table .wx-row .wx-cell) {
    border-color: color-mix(in srgb, var(--extractum-border) 72%, transparent);
  }
```

- [ ] **Step 7: Run focused tests**

Run:

```powershell
npm.cmd run test -- src/lib/components/research-projects/ProjectTabs.test.ts src/lib/components/research-projects/ResearchProjectsShell.test.ts src/lib/components/research-projects/SourcesGrid.test.ts src/lib/components/research-projects/ProjectToolbar.test.ts
```

Expected: PASS.

- [ ] **Step 8: Commit Task 3**

Run:

```powershell
git add src/lib/components/research-projects/ProjectTabs.svelte src/lib/components/research-projects/ProjectTabs.test.ts src/lib/components/research-projects/ResearchProjectsShell.svelte src/lib/components/research-projects/ResearchProjectsShell.test.ts src/lib/components/research-projects/SourcesGrid.svelte src/lib/components/research-projects/SourcesGrid.test.ts src/lib/components/research-projects/ProjectToolbar.svelte src/lib/components/research-projects/ProjectToolbar.test.ts
git commit -m "style: align projects source shell with v11"
```

If `ProjectToolbar.svelte` or its test was not changed, omit those files from `git add`.

---

### Task 4: Svelte, Type, and Tauri Visual Verification

**Files:**
- Modify only if verification finds issues in files changed by Tasks 1-3.

**Interfaces:**
- Consumes the completed component updates.
- Produces verification evidence before the branch is marked done.

- [ ] **Step 1: Run focused frontend tests for the whole slice**

Run:

```powershell
npm.cmd run test -- src/lib/ui/research-projects-source-row.test.ts src/lib/components/research-projects/SourcesGrid.test.ts src/lib/components/research-projects/SourcesFilterRow.test.ts src/lib/components/research-projects/SourcesFilterBar.test.ts src/lib/components/research-projects/SourcesBulkBar.test.ts src/lib/components/research-projects/ProjectTabs.test.ts src/lib/components/research-projects/ProjectToolbar.test.ts src/lib/components/research-projects/ResearchProjectsShell.test.ts
```

Expected: PASS.

- [ ] **Step 2: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: PASS with 0 errors.

- [ ] **Step 3: Run Svelte MCP autofixer on changed Svelte components**

For each changed `.svelte` file, call the official Svelte MCP `svelte_autofixer` with `desired_svelte_version: 5`. Use the component filename only and pass the complete current file contents as the `code` field; do not pass excerpts.

Expected: no required fixes. If the autofixer reports an issue, patch the component, rerun the focused tests from Step 1, rerun `npm.cmd run check`, and rerun the autofixer for the patched component.

- [ ] **Step 4: Verify the live `/projects/next` screen through Tauri MCP**

Use the existing Tauri MCP bridge session. If no session is active, connect to the app and navigate to `/projects/next`.

Check desktop viewport near `1280x860`:

- toolbar controls do not overlap;
- tabs remain 40px-high and active tab is visible;
- stats/filter bar and bulk overlay occupy the same band;
- filter row appears below the stats bar and does not drift dramatically from table columns;
- source headers are visible and not clipped.

Run a DOM header-fit check in the live page:

```js
Array.from(document.querySelectorAll('[role="columnheader"]')).map((el) => ({
  label: el.getAttribute("aria-label") || el.textContent?.trim(),
  clientWidth: el.clientWidth,
  scrollWidth: el.scrollWidth,
  fits: el.scrollWidth <= el.clientWidth + 1,
}));
```

Expected: every configured source-table header reports `fits: true`. If a header does not fit, prefer the least invasive fix in this order:

1. tighten header cell horizontal padding in the source-grid scope;
2. use the existing shorter header copy already present in the app;
3. increase only the relevant shared layout width and update `SOURCE_FILTER_ROW_GRID_TEMPLATE` tests.

- [ ] **Step 5: Verify current interactions in Tauri MCP**

In the live app:

- click each source table header and confirm row order changes;
- select one row and confirm the bulk bar replaces the stats/filter bar;
- clear selection and confirm the stats/filter bar returns;
- click Filters and confirm the filter row toggles;
- click `Add source` and confirm `LibraryAddSourceDialog` opens, then close it;
- click `Connect from Library` and confirm `ConnectFromLibrary` opens, then close it.

Expected: all entry points remain reachable and no visible overlap appears.

- [ ] **Step 6: Commit verification fixes if any**

If Steps 1-5 required code changes, inspect `git status --short`, stage only the verification-fix files listed there, and commit:

```powershell
git status --short
git commit -m "fix: finalize projects source v11 polish"
```

Use `git add` with the exact paths from `git status --short` before the commit. Do not stage `.claude/settings.local.json` or `.tmp-current-projects-next.png`.

If no files changed during verification, do not create an empty commit.

---

## Final Verification Before Completion

- [ ] `npm.cmd run test -- src/lib/ui/research-projects-source-row.test.ts src/lib/components/research-projects/SourcesGrid.test.ts src/lib/components/research-projects/SourcesFilterRow.test.ts src/lib/components/research-projects/SourcesFilterBar.test.ts src/lib/components/research-projects/SourcesBulkBar.test.ts src/lib/components/research-projects/ProjectTabs.test.ts src/lib/components/research-projects/ProjectToolbar.test.ts src/lib/components/research-projects/ResearchProjectsShell.test.ts`
- [ ] `npm.cmd run check`
- [ ] Svelte MCP autofixer checked every changed `.svelte` component.
- [ ] Tauri MCP screenshot/DOM verification confirms header text fits and no source-slice overlap exists.
- [ ] `git status --short` shows only intentional files or known untracked local files.
