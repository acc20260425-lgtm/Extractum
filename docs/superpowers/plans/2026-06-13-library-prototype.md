# Library Prototype Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first `/projects/library` prototype with a shared projects shell, collapsible Library filter tree, real source table, and resizable source Inspector.

**Architecture:** Introduce `/projects/+layout.svelte` as the shared shell that owns `IconRail`, while `/projects` and `/projects/library` own their own second rail and workspace. Reuse the existing research-projects workflow and `LibrarySourceView` adapter; add only a small Library-specific view-model layer for filter rows, filtering, and selection reconciliation. Keep SVAR usage behind `extractum-ui` wrappers by adding `ExtractumTreeDataGrid`.

**Tech Stack:** Svelte 5, SvelteKit SPA/Tauri, TypeScript, Vitest, `@svar-ui/svelte-grid`, `@svar-ui/svelte-core`, `@lucide/svelte`, existing `extractum-ui` wrappers.

---

## Approved Design Inputs

- Design spec: `docs/superpowers/specs/2026-06-13-library-prototype-design.md`
- Existing new UI plan: `docs/superpowers/plans/2026-06-11-new-ui-research-projects.md`
- Existing SVAR wrapper: `src/lib/components/extractum-ui/DataGrid.svelte`
- SVAR Svelte Grid skill reference: `C:/Users/Dima/.codex/skills/svar-svelte/grid/index.md`

## Scope Check

This plan builds one vertical prototype: `/projects/library`. It does not add durable CRUD persistence, source subtype backend fields, YouTube channel ingestion, or a new library schema.

## File Structure

### Route Shell

- Create: `src/routes/projects/+layout.svelte`
  - Owns shared `IconRail` and the bordered projects route shell.
- Modify: `src/routes/projects/+page.svelte`
  - Continues loading the existing research-projects workflow and renders the Projects screen inside the new layout.
- Create: `src/routes/projects/library/+page.svelte`
  - Loads the same workflow data and renders the Library screen.
- Modify: `src/lib/components/research-projects/ProjectsShell.svelte`
  - Remove embedded `IconRail`; this component becomes only `ProjectRail + ProjectWorkspace + BottomQueue + ConnectFromLibrary`.
- Modify: `src/lib/components/research-projects/IconRail.svelte`
  - Set active state from `$app/state` route pathname.
  - Link `Library` to `/projects/library`.

### View Model

- Modify: `src/lib/ui/research-projects-model.ts`
  - Add Library filter tree types and pure helpers:
    - `LibraryFilterTreeRow`
    - `LIBRARY_ALL_FILTER_ID`
    - `YOUTUBE_SUBTYPE_FILTER_DISABLED_REASON`
    - `buildLibraryFilterTree`
    - `filterLibrarySourcesForLibrary`
    - `reconcileLibrarySourceSelection`
- Modify: `src/lib/ui/research-projects-model.test.ts`
  - Add tests for Library filter tree counts, disabled subtype rows, provider filtering, search filtering, and selection reconciliation.

### Extractum UI Wrapper Layer

- Create: `src/lib/components/extractum-ui/TreeDataGrid.svelte`
  - Wraps SVAR `Grid` in tree mode and owns stable height, stable row ids, selection, empty state, theme, and scoped `.wx-*` overrides.
- Modify: `src/lib/components/extractum-ui/index.ts`
  - Export `ExtractumTreeDataGrid`.
- Modify: `src/lib/research-projects-import-boundary.test.ts`
  - Allow SVAR imports in `TreeDataGrid.svelte` as well as `DataGrid.svelte`.
  - Assert feature components and route files do not import raw SVAR, shadcn primitives, or `bits-ui`.

### Library Feature Components

- Create: `src/lib/components/research-projects/LibraryScreen.svelte`
  - Owns Library-local state: selected filter, selected source, filter rail collapse, Inspector width, and prototype status.
  - Coordinates filtering, selection reconciliation, and Inspector resize.
- Create: `src/lib/components/research-projects/LibraryFilterRail.svelte`
  - Renders the collapsible filter tree through `ExtractumTreeDataGrid`.
- Create: `src/lib/components/research-projects/LibraryWorkspace.svelte`
  - Renders toolbar, search, CRUD command buttons, and `ExtractumDataGrid`.
- Create: `src/lib/components/research-projects/LibraryInspector.svelte`
  - Renders selected-source context and source commands.
- Test: `src/lib/library-prototype-contract.test.ts`
  - Source-level route/component contract tests for the prototype.

### Verification

- Update existing route/import-boundary tests.
- Run focused Vitest tests after each task.
- Final run: `npm.cmd run test`, `npm.cmd run check`.
- Browser verification: open `/projects` and `/projects/library`, inspect desktop and narrower viewport.

---

## Task 0: Baseline And Branch

**Files:**
- No source files changed.

- [x] **Step 1: Create or confirm the implementation branch**

Run:

```powershell
git status --short --branch
```

Expected: worktree is clean before implementation. If still on `main`, create a feature branch:

```powershell
git switch -c feature/library-prototype
```

Expected: branch changes to `feature/library-prototype`.

- [x] **Step 2: Run the current focused baseline**

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-route-contract.test.ts src/lib/research-projects-import-boundary.test.ts src/lib/ui/research-projects-model.test.ts
```

Expected: existing focused tests pass before changes. If they fail, stop and record the existing failure.

---

## Task 1: Shared Projects Route Shell

**Files:**
- Create: `src/routes/projects/+layout.svelte`
- Modify: `src/routes/projects/+page.svelte`
- Modify: `src/lib/components/research-projects/ProjectsShell.svelte`
- Modify: `src/lib/components/research-projects/IconRail.svelte`
- Modify: `src/lib/research-projects-route-contract.test.ts`

- [x] **Step 1: Extend the failing route contract**

Modify `src/lib/research-projects-route-contract.test.ts` imports:

```ts
import projectsLayoutSource from "../routes/projects/+layout.svelte?raw";
import libraryRouteSource from "../routes/projects/library/+page.svelte?raw";
import iconRailSource from "./components/research-projects/IconRail.svelte?raw";
```

Add these tests inside `describe("research projects route contract", () => { ... })`:

```ts
  it("shares IconRail through the projects nested layout", () => {
    expect(projectsLayoutSource).toContain('data-ui-route-shell="projects"');
    expect(projectsLayoutSource).toContain("<IconRail");
    expect(projectsLayoutSource).toContain("{@render children()}");
    expect(shellSource).not.toContain("<IconRail");
    expect(shellSource).not.toContain('data-ui-region="icon-rail"');
  });

  it("routes Library to a separate nested screen", () => {
    expect(iconRailSource).toContain('href: "/projects/library"');
    expect(iconRailSource).toContain('page.url.pathname === "/projects/library"');
    expect(libraryRouteSource).toContain('data-ui-route="library-prototype"');
    expect(libraryRouteSource).toContain("createResearchProjectsWorkflow");
    expect(libraryRouteSource).toContain("listAnalysisSources");
    expect(libraryRouteSource).toContain("<LibraryScreen");
  });
```

Update the existing dense-region assertion:

```ts
  it("renders the dense project control deck regions", () => {
    expect(projectsLayoutSource).toContain('data-ui-region="icon-rail"');
    expect(shellSource).toContain('data-ui-region="project-rail"');
    expect(shellSource).toContain('data-ui-region="top-command-bar"');
    expect(shellSource).toContain('data-ui-region="project-workspace"');
    expect(projectsLayoutSource).toContain("grid-template-columns: 56px minmax(0, 1fr)");
    expect(shellSource).toContain("grid-template-columns: 260px minmax(0, 1fr)");
  });
```

- [x] **Step 2: Run the route contract and verify it fails**

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-route-contract.test.ts
```

Expected: FAIL because `src/routes/projects/+layout.svelte`, `src/routes/projects/library/+page.svelte`, and `LibraryScreen` do not exist yet, and `ProjectsShell` still embeds `IconRail`.

- [x] **Step 3: Create the shared projects layout**

Create `src/routes/projects/+layout.svelte`:

```svelte
<script lang="ts">
  import IconRail from "$lib/components/research-projects/IconRail.svelte";

  let { children } = $props();
</script>

<section data-ui-route-shell="projects" class="projects-route-shell">
  <aside data-ui-region="icon-rail" class="icon-rail">
    <IconRail />
  </aside>

  <div class="projects-route-content">
    {@render children()}
  </div>
</section>

<style>
  .projects-route-shell {
    display: grid;
    grid-template-columns: 56px minmax(0, 1fr);
    min-height: calc(100vh - 68px);
    border: 1px solid var(--extractum-border);
    background: var(--extractum-surface);
  }

  .icon-rail {
    min-width: 0;
    min-height: 0;
    border-right: 1px solid var(--extractum-border);
    background: var(--extractum-surface-raised);
  }

  .projects-route-content {
    min-width: 0;
    min-height: 0;
  }
</style>
```

- [x] **Step 4: Move `IconRail` ownership out of `ProjectsShell`**

Modify `src/lib/components/research-projects/ProjectsShell.svelte`:

1. Remove this import:

```svelte
  import IconRail from "./IconRail.svelte";
```

2. Remove this markup:

```svelte
  <aside data-ui-region="icon-rail" class="icon-rail">
    <IconRail />
  </aside>
```

3. Replace the `.projects-shell` CSS block with:

```css
  .projects-shell {
    display: grid;
    grid-template-columns: 260px minmax(0, 1fr);
    min-height: calc(100vh - 68px);
    background: var(--extractum-surface);
  }
```

4. Remove the `.icon-rail` CSS block from `ProjectsShell.svelte`.

- [x] **Step 5: Make `IconRail` route-aware**

Modify `src/lib/components/research-projects/IconRail.svelte`:

```svelte
<script lang="ts">
  import { page } from "$app/state";
  import {
    Activity,
    FolderKanban,
    Library,
    Settings,
    ShieldCheck,
  } from "@lucide/svelte";

  const items = [
    { href: "/projects", label: "Projects", icon: FolderKanban },
    { href: "/projects/library", label: "Library", icon: Library },
    { href: "/projects#runs", label: "Runs", icon: Activity },
    { href: "/diagnostics", label: "Diagnostics", icon: ShieldCheck },
    { href: "/settings", label: "Settings", icon: Settings },
  ];

  function isActive(href: string) {
    if (href === "/projects") return page.url.pathname === "/projects";
    if (href === "/projects/library") return page.url.pathname === "/projects/library";
    return page.url.pathname === href;
  }
</script>

<nav class="icon-rail-nav" aria-label="Research project sections">
  {#each items as item (item.href)}
    <a
      class:active={isActive(item.href)}
      href={item.href}
      title={item.label}
      aria-label={item.label}
      aria-current={isActive(item.href) ? "page" : undefined}
    >
      <item.icon size={18} aria-hidden="true" />
    </a>
  {/each}
</nav>
```

Keep the existing `<style>` block.

- [x] **Step 6: Add a temporary Library route shell to satisfy imports**

Create `src/routes/projects/library/+page.svelte`:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import LibraryScreen from "$lib/components/research-projects/LibraryScreen.svelte";
  import { listAnalysisSourceGroups, updateAnalysisSourceGroup } from "$lib/api/analysis-source-groups";
  import { listAnalysisSources } from "$lib/api/analysis-workspace";
  import { listActiveAnalysisRuns } from "$lib/api/analysis-runs";
  import { listSourceJobs } from "$lib/api/source-jobs";
  import {
    createResearchProjectsWorkflow,
    type ResearchProjectsWorkflowState,
  } from "$lib/ui/research-projects-workflow";

  const state = $state<ResearchProjectsWorkflowState>({
    groups: [],
    sources: [],
    runs: [],
    sourceJobs: [],
    projects: [],
    librarySources: [],
    projectSourceLinks: [],
    selectedProjectId: null,
    selectedLibrarySourceIds: new Set<string>(),
    loading: false,
    saving: false,
    status: "",
  });

  const workflow = createResearchProjectsWorkflow({
    getState: () => state,
    patch: (patch) => Object.assign(state, patch),
    listGroups: listAnalysisSourceGroups,
    listSources: listAnalysisSources,
    listRuns: listActiveAnalysisRuns,
    listSourceJobs: () => listSourceJobs({ limit: 50 }),
    updateGroup: updateAnalysisSourceGroup,
    formatError: (action, error) => `Ошибка ${action}: ${String(error)}`,
  });

  onMount(() => {
    void workflow.loadWorkspace();
  });
</script>

<section data-ui-route="library-prototype">
  <LibraryScreen {state} onRefresh={workflow.loadWorkspace} />
</section>
```

Create a minimal temporary `src/lib/components/research-projects/LibraryScreen.svelte` so the route compiles until Task 4 replaces it:

```svelte
<script lang="ts">
  import type { ResearchProjectsWorkflowState } from "$lib/ui/research-projects-workflow";

  let {
    state,
    onRefresh,
  }: {
    state: ResearchProjectsWorkflowState;
    onRefresh: () => void | Promise<void>;
  } = $props();
</script>

<div data-ui-screen="library" class="library-screen">
  <button type="button" onclick={onRefresh}>Refresh</button>
  <span>{state.librarySources.length} sources</span>
</div>
```

- [x] **Step 7: Run route contract**

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-route-contract.test.ts
```

Expected: PASS.

- [x] **Step 8: Commit route shell extraction**

Run:

```powershell
git add src/routes/projects/+layout.svelte src/routes/projects/+page.svelte src/routes/projects/library/+page.svelte src/lib/components/research-projects/ProjectsShell.svelte src/lib/components/research-projects/IconRail.svelte src/lib/components/research-projects/LibraryScreen.svelte src/lib/research-projects-route-contract.test.ts
git commit -m "feat: add projects library route shell"
```

---

## Task 2: Library Filter View Model

**Files:**
- Modify: `src/lib/ui/research-projects-model.ts`
- Modify: `src/lib/ui/research-projects-model.test.ts`

- [x] **Step 1: Add failing view-model tests**

Modify the import list in `src/lib/ui/research-projects-model.test.ts`:

```ts
  LIBRARY_ALL_FILTER_ID,
  YOUTUBE_SUBTYPE_FILTER_DISABLED_REASON,
  buildLibraryFilterTree,
  filterLibrarySourcesForLibrary,
  reconcileLibrarySourceSelection,
```

Add these tests inside `describe("research projects model", () => { ... })`:

```ts
  it("builds the Library filter tree with disabled YouTube subtype rows", () => {
    const rows = buildLibrarySourcesView(
      [
        source({ id: 1, source_type: "telegram", title: "Radar BPLA" }),
        source({ id: 2, source_type: "youtube", title: "Alpha Drones" }),
        source({ id: 3, source_type: "youtube", title: "Research Playlist" }),
      ],
      [],
      null,
    );

    expect(buildLibraryFilterTree(rows)).toEqual([
      expect.objectContaining({ id: "all", label: "All sources", count: 3 }),
      expect.objectContaining({
        id: "provider:youtube",
        label: "YouTube",
        provider: "youtube",
        count: 2,
        data: [
          {
            id: "provider:youtube/subtype:video",
            label: "Videos",
            provider: "youtube",
            subtype: "video",
            count: 0,
            disabled: true,
            disabledReason: YOUTUBE_SUBTYPE_FILTER_DISABLED_REASON,
          },
          {
            id: "provider:youtube/subtype:playlist",
            label: "Playlists",
            provider: "youtube",
            subtype: "playlist",
            count: 0,
            disabled: true,
            disabledReason: YOUTUBE_SUBTYPE_FILTER_DISABLED_REASON,
          },
          {
            id: "provider:youtube/subtype:channel",
            label: "Channels",
            provider: "youtube",
            subtype: "channel",
            count: 0,
            disabled: true,
            disabledReason: YOUTUBE_SUBTYPE_FILTER_DISABLED_REASON,
          },
        ],
      }),
      expect.objectContaining({ id: "provider:telegram", label: "Telegram", provider: "telegram", count: 1 }),
    ]);
  });

  it("filters Library sources by selected tree row and search query", () => {
    const rows = buildLibrarySourcesView(
      [
        source({ id: 1, source_type: "telegram", title: "Radar BPLA" }),
        source({ id: 2, source_type: "youtube", title: "Alpha Drones" }),
        source({ id: 3, source_type: "youtube", title: "Research Playlist" }),
      ],
      [],
      null,
    );

    expect(filterLibrarySourcesForLibrary(rows, { filterId: LIBRARY_ALL_FILTER_ID, query: "alpha" }).map((row) => row.id))
      .toEqual(["source:2"]);
    expect(filterLibrarySourcesForLibrary(rows, { filterId: "provider:youtube", query: "" }).map((row) => row.id))
      .toEqual(["source:2", "source:3"]);
    expect(filterLibrarySourcesForLibrary(rows, { filterId: "provider:youtube/subtype:video", query: "" }))
      .toEqual([]);
  });

  it("reconciles selected Library source with the visible rows", () => {
    const rows = buildLibrarySourcesView(
      [
        source({ id: 1, source_type: "telegram", title: "Radar BPLA" }),
        source({ id: 2, source_type: "youtube", title: "Alpha Drones" }),
      ],
      [],
      null,
    );

    expect(reconcileLibrarySourceSelection(rows, "source:2")).toBe("source:2");
    expect(reconcileLibrarySourceSelection([rows[0]], "source:2")).toBe("source:1");
    expect(reconcileLibrarySourceSelection([], "source:2")).toBeNull();
  });
```

- [x] **Step 2: Run tests and verify they fail**

Run:

```powershell
npm.cmd run test -- src/lib/ui/research-projects-model.test.ts
```

Expected: FAIL because the new exports are missing.

- [x] **Step 3: Add Library filter helpers**

Append this code after `LibraryFilterState` in `src/lib/ui/research-projects-model.ts`:

```ts
export type LibrarySourceSubtype = "video" | "playlist" | "channel";
export type LibraryFilterId =
  | "all"
  | `provider:${LibrarySourceProvider}`
  | `provider:youtube/subtype:${LibrarySourceSubtype}`;

export type LibraryFilterTreeRow = {
  id: LibraryFilterId;
  label: string;
  provider: LibrarySourceProvider | "all";
  subtype?: LibrarySourceSubtype;
  count: number;
  disabled?: boolean;
  disabledReason?: string;
  data?: LibraryFilterTreeRow[];
};

export type LibraryTableFilterState = {
  filterId: LibraryFilterId;
  query: string;
};

export const LIBRARY_ALL_FILTER_ID: LibraryFilterId = "all";
export const YOUTUBE_SUBTYPE_FILTER_DISABLED_REASON =
  "Subtype filtering requires source subtype metadata.";
```

Append these helper functions near `filterLibrarySources`:

```ts
function countProvider(sources: LibrarySourceView[], provider: LibrarySourceProvider) {
  return sources.filter((source) => source.provider === provider).length;
}

function disabledYoutubeSubtypeRow(
  subtype: LibrarySourceSubtype,
  label: string,
): LibraryFilterTreeRow {
  return {
    id: `provider:youtube/subtype:${subtype}`,
    label,
    provider: "youtube",
    subtype,
    count: 0,
    disabled: true,
    disabledReason: YOUTUBE_SUBTYPE_FILTER_DISABLED_REASON,
  };
}

export function buildLibraryFilterTree(sources: LibrarySourceView[]): LibraryFilterTreeRow[] {
  return [
    {
      id: LIBRARY_ALL_FILTER_ID,
      label: "All sources",
      provider: "all",
      count: sources.length,
    },
    {
      id: "provider:youtube",
      label: "YouTube",
      provider: "youtube",
      count: countProvider(sources, "youtube"),
      data: [
        disabledYoutubeSubtypeRow("video", "Videos"),
        disabledYoutubeSubtypeRow("playlist", "Playlists"),
        disabledYoutubeSubtypeRow("channel", "Channels"),
      ],
    },
    {
      id: "provider:telegram",
      label: "Telegram",
      provider: "telegram",
      count: countProvider(sources, "telegram"),
    },
  ];
}

function providerFromFilterId(filterId: LibraryFilterId): LibrarySourceProvider | null {
  if (filterId === LIBRARY_ALL_FILTER_ID) return null;
  if (filterId.startsWith("provider:youtube/subtype:")) return "youtube";
  return filterId.replace("provider:", "") as LibrarySourceProvider;
}

export function filterLibrarySourcesForLibrary(
  sources: LibrarySourceView[],
  filters: LibraryTableFilterState,
) {
  if (filters.filterId.startsWith("provider:youtube/subtype:")) {
    return [];
  }

  const provider = providerFromFilterId(filters.filterId);
  const providers = provider ? [provider] : [];
  return filterLibrarySources(sources, { query: filters.query, providers });
}

export function reconcileLibrarySourceSelection(
  visibleSources: LibrarySourceView[],
  selectedSourceId: string | null,
) {
  if (selectedSourceId && visibleSources.some((source) => source.id === selectedSourceId)) {
    return selectedSourceId;
  }
  return visibleSources[0]?.id ?? null;
}
```

- [x] **Step 4: Run model tests**

Run:

```powershell
npm.cmd run test -- src/lib/ui/research-projects-model.test.ts
```

Expected: PASS.

- [x] **Step 5: Commit view-model helpers**

Run:

```powershell
git add src/lib/ui/research-projects-model.ts src/lib/ui/research-projects-model.test.ts
git commit -m "feat: add library filter view model"
```

---

## Task 3: ExtractumTreeDataGrid Wrapper

**Files:**
- Create: `src/lib/components/extractum-ui/TreeDataGrid.svelte`
- Modify: `src/lib/components/extractum-ui/index.ts`
- Modify: `src/lib/research-projects-import-boundary.test.ts`

- [x] **Step 1: Update import-boundary tests for the tree wrapper**

Modify `src/lib/research-projects-import-boundary.test.ts`.

In the `routes SVAR Grid through ExtractumDataGrid only` test, rename it to:

```ts
  it("routes SVAR Grid through Extractum grid wrappers only", () => {
```

After the existing `DataGrid.svelte` assertions, add:

```ts
    const treeGridSource = readFileSync(
      path.join(repoRoot, "src/lib/components/extractum-ui/TreeDataGrid.svelte"),
      "utf8",
    );
    expect(treeGridSource).toContain('from "@svar-ui/svelte-grid"');
    expect(treeGridSource).toContain("tree");
    expect(treeGridSource).toContain("treetoggle");
    expect(treeGridSource).toContain("selectedRows");
    expect(treeGridSource).toContain("onselectrow");
    expect(treeGridSource).toContain("Willow");
    expect(treeGridSource).toContain("Locale");
    expect(treeGridSource).toContain("fonts={false}");
    expect(treeGridSource).toContain(".extractum-tree-data-grid :global(.wx-");
```

The first feature-boundary test already collects `src/lib/components/research-projects` and `src/routes/projects`; keep it unchanged so Library components are covered automatically.

- [x] **Step 2: Run import-boundary test and verify it fails**

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-import-boundary.test.ts
```

Expected: FAIL because `TreeDataGrid.svelte` does not exist yet.

- [x] **Step 3: Create `ExtractumTreeDataGrid`**

Create `src/lib/components/extractum-ui/TreeDataGrid.svelte`:

```svelte
<script lang="ts">
  import { Grid, Willow, type IColumnConfig } from "@svar-ui/svelte-grid";
  import { Locale } from "@svar-ui/svelte-core";
  import { en as gridEn } from "@svar-ui/grid-locales";
  import { ru as coreRu } from "@svar-ui/core-locales";
  import { cn } from "$lib/utils.js";

  export type TreeGridRow = {
    id: string;
    label: string;
    count?: number;
    disabled?: boolean;
    disabledReason?: string;
    data?: TreeGridRow[];
    [key: string]: unknown;
  };

  let {
    rows,
    selectedRowId = null,
    height = "100%",
    collapsed = false,
    class: className,
    overlay = "Нет данных",
    onSelectedRowIdChange = () => {},
  }: {
    rows: TreeGridRow[];
    selectedRowId?: string | null;
    height?: string;
    collapsed?: boolean;
    class?: string;
    overlay?: string;
    onSelectedRowIdChange?: (id: string | null) => void;
  } = $props();

  let api = $state<any>(null);
  let selectedRows = $derived(selectedRowId ? [selectedRowId] : []);
  let visibleOverlay = $derived(rows.length === 0 ? overlay : undefined);
  let columns = $derived<IColumnConfig[]>(collapsed
    ? [
        { id: "label", header: "", width: 48, treetoggle: true },
      ]
    : [
        { id: "label", header: "Фильтр", flexgrow: 1, treetoggle: true },
        { id: "count", header: "", width: 54 },
      ]);

  function init(gridApi: any) {
    api = gridApi;
    api.intercept("select-row", (event: { id?: string }) => {
      if (!event.id) return true;
      return api.getRow(event.id)?.disabled ? false : true;
    });
  }

  function rowStyle(row: TreeGridRow) {
    return row.disabled ? "is-disabled" : "";
  }

  function emitSelection() {
    if (!api) return;
    const nextId = api.getState().selectedRows.map(String)[0] ?? null;
    onSelectedRowIdChange(nextId);
  }
</script>

<div
  class={cn("extractum-svar-theme extractum-tree-data-grid", className)}
  data-collapsed={collapsed}
  style={`height:${height};`}
>
  <Locale words={{ ...coreRu, ...gridEn }}>
    <Willow fonts={false}>
      <Grid
        data={rows}
        {columns}
        {rowStyle}
        {selectedRows}
        init={init}
        tree
        select
        multiselect={false}
        sizes={{ rowHeight: 30, headerHeight: collapsed ? 0 : 30, columnWidth: 140 }}
        overlay={visibleOverlay}
        onselectrow={emitSelection}
      />
    </Willow>
  </Locale>
</div>

<style>
  .extractum-tree-data-grid {
    min-height: 0;
    min-width: 0;
    width: 100%;
    max-width: 100%;
    overflow: hidden;
  }

  .extractum-tree-data-grid :global(.wx-grid),
  .extractum-tree-data-grid :global(.wx-table-box) {
    height: 100%;
  }

  .extractum-tree-data-grid :global(.wx-cell) {
    padding: 4px 8px;
    font-size: 12.5px;
  }

  .extractum-tree-data-grid :global(.wx-row.is-disabled:not(.wx-selected) .wx-cell) {
    color: var(--extractum-muted);
    background: color-mix(in srgb, var(--extractum-surface-subtle) 80%, transparent);
  }

  .extractum-tree-data-grid[data-collapsed="true"] :global(.wx-header) {
    display: none;
  }
</style>
```

- [x] **Step 4: Export the wrapper**

Modify `src/lib/components/extractum-ui/index.ts`:

```ts
export { default as ExtractumTreeDataGrid } from "./TreeDataGrid.svelte";
```

- [x] **Step 5: Run focused checks**

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-import-boundary.test.ts
npm.cmd run check
```

Expected: both commands pass.

- [x] **Step 6: Commit tree wrapper**

Run:

```powershell
git add src/lib/components/extractum-ui/TreeDataGrid.svelte src/lib/components/extractum-ui/index.ts src/lib/research-projects-import-boundary.test.ts
git commit -m "feat: add extractum tree data grid"
```

---

## Task 4: Library Prototype Components

**Files:**
- Create: `src/lib/library-prototype-contract.test.ts`
- Create: `src/lib/components/research-projects/LibraryFilterRail.svelte`
- Replace: `src/lib/components/research-projects/LibraryScreen.svelte`
- Create: `src/lib/components/research-projects/LibraryWorkspace.svelte`
- Create: `src/lib/components/research-projects/LibraryInspector.svelte`

- [ ] **Step 1: Write the failing Library prototype contract**

Create `src/lib/library-prototype-contract.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import routeSource from "../routes/projects/library/+page.svelte?raw";
import screenSource from "./components/research-projects/LibraryScreen.svelte?raw";
import filterRailSource from "./components/research-projects/LibraryFilterRail.svelte?raw";
import workspaceSource from "./components/research-projects/LibraryWorkspace.svelte?raw";
import inspectorSource from "./components/research-projects/LibraryInspector.svelte?raw";

describe("Library prototype contract", () => {
  it("renders Library as a separate route backed by the current workflow", () => {
    expect(routeSource).toContain('data-ui-route="library-prototype"');
    expect(routeSource).toContain("createResearchProjectsWorkflow");
    expect(routeSource).toContain("listAnalysisSources");
    expect(routeSource).toContain("<LibraryScreen");
  });

  it("uses the TreeDataGrid wrapper for the collapsible filter rail", () => {
    expect(filterRailSource).toContain("ExtractumTreeDataGrid");
    expect(filterRailSource).toContain('data-ui-region="library-filter-rail"');
    expect(filterRailSource).toContain("collapsed");
    expect(filterRailSource).toContain("onSelectedFilterIdChange");
    expect(filterRailSource).not.toContain("@svar-ui/");
  });

  it("renders source CRUD commands and disables selected-source commands without a source", () => {
    expect(workspaceSource).toContain("ExtractumDataGrid");
    expect(workspaceSource).toContain('data-ui-region="library-workspace"');
    expect(workspaceSource).toContain('data-ui-action="library-add"');
    expect(workspaceSource).toContain('data-ui-action="library-edit"');
    expect(workspaceSource).toContain('data-ui-action="library-delete"');
    expect(workspaceSource).toContain('disabled={!selectedSource}');
    expect(workspaceSource).not.toContain("@svar-ui/");
    expect(workspaceSource).not.toContain("$lib/components/ui/");
  });

  it("keeps the Inspector bound to selected source context", () => {
    expect(inspectorSource).toContain('data-ui-region="library-inspector"');
    expect(inspectorSource).toContain("selectedSource");
    expect(inspectorSource).toContain("No source selected");
    expect(inspectorSource).toContain("aria-label=\"Inspector commands\"");
  });

  it("coordinates filter selection, row selection, and Inspector resizing in the screen component", () => {
    expect(screenSource).toContain("buildLibraryFilterTree");
    expect(screenSource).toContain("filterLibrarySourcesForLibrary");
    expect(screenSource).toContain("reconcileLibrarySourceSelection");
    expect(screenSource).toContain("inspectorWidth");
    expect(screenSource).toContain("clampInspectorWidth");
    expect(screenSource).toContain('role="separator"');
    expect(screenSource).toContain("onpointerdown");
  });
});
```

- [ ] **Step 2: Run the contract and verify it fails**

Run:

```powershell
npm.cmd run test -- src/lib/library-prototype-contract.test.ts
```

Expected: FAIL because the Library components are missing or still temporary.

- [ ] **Step 3: Create `LibraryFilterRail.svelte`**

Create `src/lib/components/research-projects/LibraryFilterRail.svelte`:

```svelte
<script lang="ts">
  import { PanelLeftClose, PanelLeftOpen } from "@lucide/svelte";
  import { ExtractumButton, ExtractumTreeDataGrid } from "$lib/components/extractum-ui";
  import type { LibraryFilterTreeRow, LibraryFilterId } from "$lib/ui/research-projects-model";

  let {
    rows,
    selectedFilterId,
    collapsed,
    onSelectedFilterIdChange,
    onCollapsedChange,
  }: {
    rows: LibraryFilterTreeRow[];
    selectedFilterId: LibraryFilterId;
    collapsed: boolean;
    onSelectedFilterIdChange: (id: LibraryFilterId) => void;
    onCollapsedChange: (collapsed: boolean) => void;
  } = $props();
</script>

<aside
  data-ui-region="library-filter-rail"
  class:collapsed
  class="library-filter-rail"
  aria-label="Library filters"
>
  <div class="rail-header">
    {#if !collapsed}
      <span>Library</span>
    {/if}
    <ExtractumButton
      variant="ghost"
      size="icon"
      aria-label={collapsed ? "Expand Library filters" : "Collapse Library filters"}
      onclick={() => onCollapsedChange(!collapsed)}
    >
      {#if collapsed}
        <PanelLeftOpen size={15} aria-hidden="true" />
      {:else}
        <PanelLeftClose size={15} aria-hidden="true" />
      {/if}
    </ExtractumButton>
  </div>

  <ExtractumTreeDataGrid
    rows={rows}
    selectedRowId={selectedFilterId}
    {collapsed}
    height="100%"
    onSelectedRowIdChange={(id) => {
      if (id) onSelectedFilterIdChange(id as LibraryFilterId);
    }}
  />
</aside>

<style>
  .library-filter-rail {
    display: flex;
    min-width: 0;
    min-height: 0;
    flex-direction: column;
    border-right: 1px solid var(--extractum-border);
    background: var(--extractum-surface-raised);
  }

  .rail-header {
    display: flex;
    min-height: 40px;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 6px 8px;
    border-bottom: 1px solid var(--extractum-border);
    color: var(--extractum-muted);
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
  }

  .library-filter-rail.collapsed .rail-header {
    justify-content: center;
    padding-inline: 4px;
  }
</style>
```

- [ ] **Step 4: Create `LibraryWorkspace.svelte`**

Create `src/lib/components/research-projects/LibraryWorkspace.svelte`:

```svelte
<script lang="ts">
  import { Edit3, Plus, RefreshCw, Trash2 } from "@lucide/svelte";
  import {
    ExtractumButton,
    ExtractumDataGrid,
    ExtractumTextInput,
  } from "$lib/components/extractum-ui";
  import type { LibrarySourceView } from "$lib/ui/research-projects-model";
  import LibrarySourceCell from "./LibrarySourceCell.svelte";

  let {
    sources,
    query = $bindable(""),
    selectedSource,
    selectedSourceId,
    loading = false,
    onSelectedSourceIdChange,
    onAdd,
    onEdit,
    onDelete,
    onRefresh,
  }: {
    sources: LibrarySourceView[];
    query: string;
    selectedSource: LibrarySourceView | null;
    selectedSourceId: string | null;
    loading?: boolean;
    onSelectedSourceIdChange: (id: string | null) => void;
    onAdd: () => void;
    onEdit: () => void;
    onDelete: () => void;
    onRefresh: () => void | Promise<void>;
  } = $props();

  const columns = [
    { id: "title", header: "Источник", flexgrow: 1, cell: LibrarySourceCell },
    { id: "provider", header: "Тип", width: 100 },
    { id: "status", header: "Статус", width: 118 },
    { id: "projectCount", header: "Проекты", width: 90 },
    { id: "localCopyLabel", header: "Локально", width: 116 },
    { id: "lastCollectedLabel", header: "Обновлен", width: 136 },
  ];
</script>

<section data-ui-region="library-workspace" class="library-workspace">
  <div class="toolbar">
    <ExtractumTextInput bind:value={query} placeholder="Search sources" aria-label="Search Library sources" />
    <ExtractumButton data-ui-action="library-add" onclick={onAdd}>
      <Plus size={14} aria-hidden="true" />
      Add
    </ExtractumButton>
    <ExtractumButton data-ui-action="library-edit" variant="outline" disabled={!selectedSource} onclick={onEdit}>
      <Edit3 size={14} aria-hidden="true" />
      Edit
    </ExtractumButton>
    <ExtractumButton data-ui-action="library-delete" variant="outline" disabled={!selectedSource} onclick={onDelete}>
      <Trash2 size={14} aria-hidden="true" />
      Delete
    </ExtractumButton>
    <ExtractumButton data-ui-action="library-refresh" variant="outline" disabled={loading} onclick={onRefresh}>
      <RefreshCw size={14} aria-hidden="true" />
      Refresh
    </ExtractumButton>
  </div>

  <div class="grid-host">
    <ExtractumDataGrid
      rows={sources}
      {columns}
      selectedRowIds={selectedSourceId ? [selectedSourceId] : []}
      overlay="No sources match this filter"
      onSelectedRowIdsChange={(ids) => onSelectedSourceIdChange(ids.at(-1) ?? null)}
    />
  </div>
</section>

<style>
  .library-workspace {
    display: flex;
    min-width: 0;
    min-height: 0;
    flex-direction: column;
    background: var(--extractum-surface);
  }

  .toolbar {
    display: flex;
    min-height: 46px;
    align-items: center;
    gap: 8px;
    padding: 8px;
    border-bottom: 1px solid var(--extractum-border);
  }

  .toolbar :global(.extractum-input) {
    flex: 1 1 auto;
    min-width: 160px;
  }

  .grid-host {
    min-width: 0;
    min-height: 0;
    flex: 1;
  }
</style>
```

- [ ] **Step 5: Create `LibraryInspector.svelte`**

Create `src/lib/components/research-projects/LibraryInspector.svelte`:

```svelte
<script lang="ts">
  import { ExternalLink, Link2, PlayCircle, RefreshCw } from "@lucide/svelte";
  import { ExtractumButton, ProviderBadge, StatusBadge } from "$lib/components/extractum-ui";
  import type { LibrarySourceView } from "$lib/ui/research-projects-model";

  let { selectedSource }: { selectedSource: LibrarySourceView | null } = $props();
</script>

<aside data-ui-region="library-inspector" class="library-inspector" aria-label="Library source inspector">
  {#if selectedSource}
    <header class="inspector-header">
      <div>
        <p class="eyebrow">Selected source</p>
        <h2>{selectedSource.title}</h2>
      </div>
      <ProviderBadge provider={selectedSource.provider} />
    </header>

    <div class="status-row">
      <StatusBadge status={selectedSource.status} />
      {#if selectedSource.alreadyConnected}
        <span class="meta-pill">Connected</span>
      {/if}
    </div>

    <dl class="meta-list">
      <div><dt>Source ID</dt><dd>{selectedSource.sourceId}</dd></div>
      <div><dt>Projects</dt><dd>{selectedSource.projectCount}</dd></div>
      <div><dt>Local copy</dt><dd>{selectedSource.localCopyLabel ?? "No local copy"}</dd></div>
      <div><dt>Last collected</dt><dd>{selectedSource.lastCollectedLabel ?? "Never"}</dd></div>
    </dl>

    {#if selectedSource.disabledReason}
      <p class="notice">{selectedSource.disabledReason}</p>
    {/if}

    <div class="commands" aria-label="Inspector commands">
      <ExtractumButton variant="outline"><ExternalLink size={14} aria-hidden="true" />Open</ExtractumButton>
      <ExtractumButton variant="outline"><RefreshCw size={14} aria-hidden="true" />Sync</ExtractumButton>
      <ExtractumButton variant="outline"><Link2 size={14} aria-hidden="true" />Connect</ExtractumButton>
      <ExtractumButton variant="outline"><PlayCircle size={14} aria-hidden="true" />Run report</ExtractumButton>
    </div>
  {:else}
    <div class="empty-state">
      <p class="eyebrow">Inspector</p>
      <h2>No source selected</h2>
      <p>Select a source row to inspect metadata and available commands.</p>
    </div>
  {/if}
</aside>

<style>
  .library-inspector {
    min-width: 0;
    min-height: 0;
    padding: 14px;
    overflow: auto;
    background: var(--extractum-surface-raised);
  }

  .inspector-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }

  .eyebrow {
    margin: 0 0 6px;
    color: var(--extractum-muted);
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
  }

  h2 {
    margin: 0;
    font-size: 16px;
    font-weight: 700;
    line-height: 1.25;
  }

  .status-row,
  .commands {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 12px;
  }

  .commands {
    display: grid;
    grid-template-columns: 1fr 1fr;
  }

  .meta-list {
    display: grid;
    gap: 8px;
    margin: 14px 0;
  }

  .meta-list div {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    border-bottom: 1px solid var(--extractum-border);
    padding-bottom: 6px;
  }

  dt {
    color: var(--extractum-muted);
    font-size: 12px;
  }

  dd {
    margin: 0;
    font-size: 12px;
    text-align: right;
  }

  .meta-pill,
  .notice {
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    padding: 4px 7px;
    color: var(--extractum-muted);
    font-size: 12px;
  }

  .notice {
    margin: 12px 0 0;
  }

  .empty-state {
    display: grid;
    min-height: 220px;
    align-content: center;
    gap: 8px;
    color: var(--extractum-muted);
  }
</style>
```

- [ ] **Step 6: Replace `LibraryScreen.svelte` with the real coordinator**

Replace `src/lib/components/research-projects/LibraryScreen.svelte`:

```svelte
<script lang="ts">
  import LibraryFilterRail from "./LibraryFilterRail.svelte";
  import LibraryInspector from "./LibraryInspector.svelte";
  import LibraryWorkspace from "./LibraryWorkspace.svelte";
  import {
    LIBRARY_ALL_FILTER_ID,
    buildLibraryFilterTree,
    filterLibrarySourcesForLibrary,
    reconcileLibrarySourceSelection,
    type LibraryFilterId,
  } from "$lib/ui/research-projects-model";
  import type { ResearchProjectsWorkflowState } from "$lib/ui/research-projects-workflow";

  let {
    state,
    onRefresh,
  }: {
    state: ResearchProjectsWorkflowState;
    onRefresh: () => void | Promise<void>;
  } = $props();

  let selectedFilterId = $state<LibraryFilterId>(LIBRARY_ALL_FILTER_ID);
  let selectedSourceId = $state<string | null>(null);
  let query = $state("");
  let filterCollapsed = $state(false);
  let inspectorWidth = $state(380);
  let status = $state("");

  let filterRows = $derived(buildLibraryFilterTree(state.librarySources));
  let visibleSources = $derived(
    filterLibrarySourcesForLibrary(state.librarySources, { filterId: selectedFilterId, query }),
  );
  let selectedSource = $derived(
    visibleSources.find((source) => source.id === selectedSourceId) ?? null,
  );

  $effect(() => {
    const nextSelectedId = reconcileLibrarySourceSelection(visibleSources, selectedSourceId);
    if (nextSelectedId !== selectedSourceId) selectedSourceId = nextSelectedId;
  });

  function clampInspectorWidth(width: number) {
    return Math.min(500, Math.max(380, Math.round(width)));
  }

  function startInspectorResize(event: PointerEvent) {
    const startX = event.clientX;
    const startWidth = inspectorWidth;
    const target = event.currentTarget as HTMLElement;
    target.setPointerCapture(event.pointerId);

    function move(moveEvent: PointerEvent) {
      inspectorWidth = clampInspectorWidth(startWidth - (moveEvent.clientX - startX));
    }

    function up(upEvent: PointerEvent) {
      target.releasePointerCapture(upEvent.pointerId);
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
    }

    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  }

  function resizeWithKeyboard(event: KeyboardEvent) {
    if (event.key === "ArrowLeft") {
      inspectorWidth = clampInspectorWidth(inspectorWidth + 16);
      event.preventDefault();
    }
    if (event.key === "ArrowRight") {
      inspectorWidth = clampInspectorWidth(inspectorWidth - 16);
      event.preventDefault();
    }
  }

  function prototypeFeedback(action: string) {
    status = `${action} flow is not implemented in this prototype.`;
  }
</script>

<div
  data-ui-screen="library"
  class="library-screen"
  style={`--library-filter-width:${filterCollapsed ? 64 : 240}px; --library-inspector-width:${inspectorWidth}px;`}
>
  <LibraryFilterRail
    rows={filterRows}
    selectedFilterId={selectedFilterId}
    collapsed={filterCollapsed}
    onSelectedFilterIdChange={(id) => (selectedFilterId = id)}
    onCollapsedChange={(collapsed) => (filterCollapsed = collapsed)}
  />

  <LibraryWorkspace
    sources={visibleSources}
    bind:query
    selectedSource={selectedSource}
    selectedSourceId={selectedSourceId}
    loading={state.loading}
    onSelectedSourceIdChange={(id) => (selectedSourceId = id)}
    onAdd={() => prototypeFeedback("Add source")}
    onEdit={() => prototypeFeedback("Edit source")}
    onDelete={() => prototypeFeedback("Delete source")}
    onRefresh={onRefresh}
  />

  <div
    class="inspector-resize-handle"
    role="separator"
    aria-label="Resize source inspector"
    aria-orientation="vertical"
    aria-valuemin="380"
    aria-valuemax="500"
    aria-valuenow={inspectorWidth}
    tabindex="0"
    onpointerdown={startInspectorResize}
    onkeydown={resizeWithKeyboard}
  ></div>

  <LibraryInspector {selectedSource} />

  {#if status || state.status}
    <div class="library-status" role="status">{status || state.status}</div>
  {/if}
</div>

<style>
  .library-screen {
    position: relative;
    display: grid;
    grid-template-columns: var(--library-filter-width) minmax(0, 1fr) 6px var(--library-inspector-width);
    min-height: calc(100vh - 68px);
    background: var(--extractum-surface);
  }

  .inspector-resize-handle {
    min-width: 6px;
    cursor: col-resize;
    border-inline: 1px solid var(--extractum-border);
    background: var(--extractum-surface-subtle);
  }

  .inspector-resize-handle:focus-visible {
    outline: 2px solid var(--extractum-primary);
    outline-offset: -2px;
  }

  .library-status {
    position: absolute;
    right: 14px;
    bottom: 12px;
    max-width: min(520px, calc(100% - 28px));
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    padding: 8px 10px;
    background: var(--extractum-surface-raised);
    color: var(--extractum-muted);
    font-size: 12px;
    box-shadow: 0 8px 22px rgb(15 23 42 / 0.10);
  }
</style>
```

- [ ] **Step 7: Run the Library contract**

Run:

```powershell
npm.cmd run test -- src/lib/library-prototype-contract.test.ts
```

Expected: PASS.

- [ ] **Step 8: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: PASS.

If `bind:query` with `$bindable` reports a typing issue, apply this exact fallback.

In `src/lib/components/research-projects/LibraryWorkspace.svelte`, replace:

```ts
    query = $bindable(""),
```

with:

```ts
    query,
    onQueryChange,
```

Add `onQueryChange` to the props type:

```ts
    query: string;
    onQueryChange: (query: string) => void;
```

Add this handler before `const columns = [`:

```ts
  function handleQueryInput(event: Event) {
    onQueryChange((event.currentTarget as HTMLInputElement).value);
  }
```

Replace the search input:

```svelte
    <ExtractumTextInput bind:value={query} placeholder="Search sources" aria-label="Search Library sources" />
```

with:

```svelte
    <ExtractumTextInput
      value={query}
      placeholder="Search sources"
      aria-label="Search Library sources"
      oninput={handleQueryInput}
    />
```

In `src/lib/components/research-projects/LibraryScreen.svelte`, replace:

```svelte
    bind:query
```

with:

```svelte
    query={query}
    onQueryChange={(nextQuery) => (query = nextQuery)}
```

Then rerun:

```powershell
npm.cmd run check
```

Expected: PASS.

- [ ] **Step 9: Commit Library components**

Run:

```powershell
git add src/lib/library-prototype-contract.test.ts src/lib/components/research-projects/LibraryScreen.svelte src/lib/components/research-projects/LibraryFilterRail.svelte src/lib/components/research-projects/LibraryWorkspace.svelte src/lib/components/research-projects/LibraryInspector.svelte
git commit -m "feat: add library prototype workspace"
```

---

## Task 5: Route And Boundary Polish

**Files:**
- Modify: `src/lib/research-projects-import-boundary.test.ts`
- Modify: `src/lib/research-projects-route-contract.test.ts`
- Modify: Library files only if tests reveal drift.

- [ ] **Step 1: Tighten the import-boundary test around Library files**

Add this test to `src/lib/research-projects-import-boundary.test.ts`:

```ts
  it("keeps Library route and feature screens out of direct shadcn and SVAR imports", () => {
    const libraryFiles = [
      path.join(repoRoot, "src/routes/projects/library/+page.svelte"),
      ...collectFiles("src/lib/components/research-projects")
        .filter((file) => path.basename(file).startsWith("Library")),
    ];

    const offenders = libraryFiles
      .map((file) => [path.relative(repoRoot, file).replaceAll("\\", "/"), sourceOf(file)] as const)
      .filter(([, source]) =>
        source.includes("@svar-ui/") ||
        source.includes("bits-ui") ||
        source.includes("$lib/components/ui/"),
      )
      .map(([file]) => file);

    expect(offenders).toEqual([]);
  });
```

- [ ] **Step 2: Run boundary and route tests**

Run:

```powershell
npm.cmd run test -- src/lib/research-projects-import-boundary.test.ts src/lib/research-projects-route-contract.test.ts src/lib/library-prototype-contract.test.ts
```

Expected: PASS.

- [ ] **Step 3: Run all UI model tests touched by Library**

Run:

```powershell
npm.cmd run test -- src/lib/ui/research-projects-model.test.ts src/lib/ui/research-projects-workflow.test.ts
```

Expected: PASS.

- [ ] **Step 4: Commit route and boundary polish**

Run:

```powershell
git add src/lib/research-projects-import-boundary.test.ts src/lib/research-projects-route-contract.test.ts src/lib/library-prototype-contract.test.ts src/lib/ui/research-projects-model.test.ts src/lib/ui/research-projects-workflow.test.ts src/routes/projects src/lib/components/research-projects src/lib/components/extractum-ui src/lib/ui/research-projects-model.ts
git commit -m "test: cover library prototype boundaries"
```

If `git status --short` shows no staged changes because prior tasks already covered everything, skip this commit and note that there was no diff.

---

## Task 6: Full Verification And Browser QA

**Files:**
- No planned source changes.
- Modify only if verification reveals a concrete defect.

- [ ] **Step 1: Run the full test suite**

Run:

```powershell
npm.cmd run test
```

Expected: all Vitest tests pass.

- [ ] **Step 2: Run Svelte/TypeScript check**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check` passes.

- [ ] **Step 3: Start the dev server**

Run:

```powershell
npm.cmd run dev -- --host 127.0.0.1
```

Expected: Vite reports a local URL, usually `http://127.0.0.1:1420/`. Keep the server running for browser verification.

- [ ] **Step 4: Verify `/projects` still shows the Projects screen**

Open `http://127.0.0.1:1420/projects`.

Expected:

- `IconRail` is visible.
- `Projects` item is active.
- `ProjectRail` is visible.
- `ProjectWorkspace` is visible.
- No horizontal page overflow.

- [ ] **Step 5: Verify `/projects/library` screen layout**

Open `http://127.0.0.1:1420/projects/library`.

Expected:

- `Library` item in `IconRail` is active.
- `ProjectRail` is not visible.
- Left Library filter rail is `240px` expanded.
- Main source table shows real `LibrarySourceView` rows when data exists.
- Inspector is visible at `380px`.

- [ ] **Step 6: Verify Library interactions**

In the browser:

- Select `YouTube`; table only shows YouTube rows.
- Select `Telegram`; table only shows Telegram rows.
- Select a disabled YouTube subtype row; it does not become an active backed filter.
- Collapse the filter rail; width becomes `64px`.
- Expand the filter rail; width returns to `240px`.
- Select a table row; Inspector content changes.
- Use a filter that removes the selected row; selection moves to the first visible source or becomes empty when the table is empty.
- Drag Inspector handle; width clamps between `380px` and `500px`.
- Press `Tab` through `IconRail`, filter rail, toolbar, table, resize handle, and Inspector commands.

- [ ] **Step 7: Verify narrower viewport**

Resize browser to `1280x800`.

Expected:

- No incoherent overlap.
- Toolbar controls fit or wrap cleanly.
- Table remains usable.
- Inspector and filter rail keep their fixed/capped widths.

- [ ] **Step 8: Final status**

Run:

```powershell
git status --short
```

Expected: no output. If there are changes from fixes during QA, run the relevant focused tests again and commit them with a focused message.

---

## Self-Review Checklist

- Spec requirement `/projects/library` separate route: Task 1 and Task 6.
- Shared `IconRail` active state: Task 1.
- `ProjectRail` only on `/projects`: Task 1 and Task 6.
- Collapsible `240px` to `64px` filter rail: Task 4 and Task 6.
- Real `LibrarySourceView` table rows: Task 4.
- Filter tree selection affects table rows: Task 2 and Task 4.
- YouTube subtype rows disabled: Task 2 and Task 6.
- Source selection affects Inspector: Task 4 and Task 6.
- Filter changes select first visible row or empty state: Task 2 and Task 4.
- `Edit` and `Delete` disabled without selection: Task 4.
- Resizable `380-500px` Inspector: Task 4 and Task 6.
- Keyboard reachability: Task 4 and Task 6.
- SVAR through wrappers only: Task 3 and Task 5.
- Full verification: Task 6.
