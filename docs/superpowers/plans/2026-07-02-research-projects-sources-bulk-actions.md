# Sources bulk-action bar (/projects/next) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a contextual bulk-action bar above the sources table on `/projects/next` that lets the user sync or delete the selected sources (per the v10 design).

**Architecture:** One presentational component `SourcesBulkBar.svelte` (owns its own confirm dialog) is rendered by `ResearchProjectsShell` above the grid via a `bulkBar` prop bag. The page `/projects/next/+page.svelte` owns all selection/sync/delete logic and passes the bag only when ≥1 source is selected. Sync is limited to youtube video/playlist sources.

**Tech Stack:** Svelte 5 runes, SvelteKit, Tauri (Rust backend via `invoke`), `@testing-library/svelte` + jsdom, extractum-ui wrapper layer.

**Source spec:** `docs/superpowers/specs/2026-07-02-research-projects-sources-bulk-actions-design.md`

## Global Constraints

- **Import boundary** (`src/lib/research-projects-import-boundary.test.ts`): feature files under `src/lib/components/research-projects/` and `src/routes/projects/` must NOT contain `@svar-ui/`, `bits-ui`, or `$lib/components/ui/` — even inside `.ts`/comment text. Use `$lib/components/extractum-ui` wrappers only. `ExtractumDialog` and `ExtractumButton` are the wrappers to use.
- **UI copy is Russian.** Danger styling via shadcn `variant="destructive"` (maps to `--extractum-danger` tokens).
- **API signatures (copy verbatim):**
  - `syncYoutubeSource(sourceId: number, options: YoutubeSyncOptions)` from `$lib/api/source-jobs`; `YoutubeSyncOptions = { metadata: boolean; transcripts: boolean; comments: boolean }`.
  - `removeProjectSources(input: ProjectSourcesInput)` from `$lib/api/projects`; `ProjectSourcesInput = { projectId: number; sourceIds: number[] }`.
  - `listProjectSources(projectId: number): Promise<ProjectSourceRecord[]>` from `$lib/api/projects` (already imported on the page).
- **Syncable predicate:** a source supports sync iff `provider === "youtube"` AND `source_subtype ∈ {"video","playlist"}` (`ProjectSourceRecord` fields `provider`, `source_subtype`).
- **Selection id shape:** `selectedSourceIds: string[]`, each entry is `String(source.source_id)` (row id === `String(sourceId)`). Convert with `Number(id)` when calling APIs.
- **svar grid does not render in jsdom.** Component render tests use `// @vitest-environment jsdom` + `afterEach(cleanup)`. bits-ui Dialog DOES render in jsdom (like Popover), so `SourcesBulkBar` gets a real render test; shell/page grid wiring is verified by `?raw` source assertions plus live Tauri checks.
- **Verification:** run `npm.cmd run check` at task boundaries; verify the page-level sync/delete flow LIVE in the running Tauri app (svar grid + `invoke` don't run in jsdom).
- **Commits:** one commit per task, only within this plan's execution. Do not push.

---

### Task 1: `SourcesBulkBar.svelte` presentational component + confirm dialog

**Files:**
- Create: `src/lib/components/research-projects/SourcesBulkBar.svelte`
- Test: `src/lib/components/research-projects/SourcesBulkBar.test.ts`

**Interfaces:**
- Consumes: `ExtractumButton`, `ExtractumDialog` from `$lib/components/extractum-ui`.
- Produces: default export `SourcesBulkBar` with props
  `{ count: number; syncDisabled?: boolean; syncTitle?: string; onClear?: () => void; onSync?: () => void; onDelete?: () => void }`.
  Accessible button names: `"Синхронизировать"`, `"Удалить"` (opens dialog), `"Да, удалить"` (confirm), `"Отмена"`, `"Снять выделение"`. Count text: `"Выбрано: {count}"`.

- [ ] **Step 1: Write the failing test**

`src/lib/components/research-projects/SourcesBulkBar.test.ts`:

```ts
// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import SourcesBulkBar from "./SourcesBulkBar.svelte";

afterEach(cleanup);

describe("SourcesBulkBar", () => {
  it("shows the selected count", () => {
    render(SourcesBulkBar, { props: { count: 3 } });
    expect(screen.getByText("Выбрано: 3")).toBeTruthy();
  });

  it("disables the sync button and exposes the title when syncDisabled", () => {
    render(SourcesBulkBar, {
      props: { count: 2, syncDisabled: true, syncTitle: "Нет источников для синхронизации" },
    });
    const sync = screen.getByRole("button", { name: "Синхронизировать" }) as HTMLButtonElement;
    expect(sync.disabled).toBe(true);
    expect(sync.getAttribute("title")).toBe("Нет источников для синхронизации");
  });

  it("calls onClear when clicking «Снять выделение»", async () => {
    const onClear = vi.fn();
    render(SourcesBulkBar, { props: { count: 1, onClear } });
    await fireEvent.click(screen.getByText("Снять выделение"));
    expect(onClear).toHaveBeenCalledOnce();
  });

  it("calls onSync when the enabled sync button is clicked", async () => {
    const onSync = vi.fn();
    render(SourcesBulkBar, { props: { count: 1, onSync } });
    await fireEvent.click(screen.getByRole("button", { name: "Синхронизировать" }));
    expect(onSync).toHaveBeenCalledOnce();
  });

  it("confirms before deleting: opens a dialog, deletes only on confirm", async () => {
    const onDelete = vi.fn();
    render(SourcesBulkBar, { props: { count: 2, onDelete } });

    // The bar's delete button opens the dialog; it must NOT delete immediately.
    await fireEvent.click(screen.getByRole("button", { name: "Удалить" }));
    expect(onDelete).not.toHaveBeenCalled();

    // Confirm inside the dialog.
    await fireEvent.click(screen.getByRole("button", { name: "Да, удалить" }));
    expect(onDelete).toHaveBeenCalledOnce();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm.cmd run test:unit -- src/lib/components/research-projects/SourcesBulkBar.test.ts`
Expected: FAIL — cannot resolve `./SourcesBulkBar.svelte` (module not found).

> If your repo uses a different vitest invocation, use `npx vitest run src/lib/components/research-projects/SourcesBulkBar.test.ts`. Confirm the exact script name in `package.json` before running.

- [ ] **Step 3: Write minimal implementation**

`src/lib/components/research-projects/SourcesBulkBar.svelte`:

```svelte
<script lang="ts">
  import { ExtractumButton, ExtractumDialog } from "$lib/components/extractum-ui";

  let {
    count,
    syncDisabled = false,
    syncTitle = "",
    onClear = () => {},
    onSync = () => {},
    onDelete = () => {},
  }: {
    count: number;
    syncDisabled?: boolean;
    syncTitle?: string;
    onClear?: () => void;
    onSync?: () => void;
    onDelete?: () => void;
  } = $props();

  let confirmOpen = $state(false);

  function confirmDelete() {
    confirmOpen = false;
    onDelete();
  }
</script>

<div class="sources-bulk-bar" role="region" aria-label="Массовые действия">
  <span class="sources-bulk-bar__count">Выбрано: {count}</span>
  <button type="button" class="sources-bulk-bar__clear" onclick={() => onClear()}>
    Снять выделение
  </button>
  <div class="sources-bulk-bar__spacer"></div>
  <ExtractumButton
    variant="outline"
    disabled={syncDisabled}
    title={syncDisabled ? syncTitle : ""}
    onclick={() => onSync()}
  >
    Синхронизировать
  </ExtractumButton>
  <ExtractumButton variant="destructive" onclick={() => (confirmOpen = true)}>
    Удалить
  </ExtractumButton>
</div>

<ExtractumDialog bind:open={confirmOpen} title="Удалить источники">
  <div class="sources-bulk-bar__confirm">
    <p>
      Удалить {count} источник(ов) из проекта? Материалы останутся в библиотеке —
      удаляется только связь с проектом.
    </p>
    <footer>
      <ExtractumButton type="button" variant="outline" onclick={() => (confirmOpen = false)}>
        Отмена
      </ExtractumButton>
      <ExtractumButton type="button" variant="destructive" onclick={confirmDelete}>
        Да, удалить
      </ExtractumButton>
    </footer>
  </div>
</ExtractumDialog>

<style>
  .sources-bulk-bar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 12px;
    border-bottom: 1px solid var(--extractum-border);
    background: var(--extractum-surface-2, var(--extractum-surface));
    font: 400 13px/1.4 var(--extractum-font);
    color: var(--extractum-text);
  }

  .sources-bulk-bar__count {
    font-weight: 600;
  }

  .sources-bulk-bar__clear {
    background: none;
    border: none;
    padding: 0;
    color: var(--extractum-primary);
    cursor: pointer;
    font: inherit;
    text-decoration: underline;
  }

  .sources-bulk-bar__spacer {
    flex: 1;
  }

  .sources-bulk-bar__confirm {
    display: flex;
    min-width: min(420px, calc(100vw - 96px));
    flex-direction: column;
    gap: 16px;
    padding: 16px;
  }

  .sources-bulk-bar__confirm footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm.cmd run test:unit -- src/lib/components/research-projects/SourcesBulkBar.test.ts`
Expected: PASS (all 5 tests).

> **Contingency (only if the confirm-dialog test fails because bits-ui Dialog content does not mount under jsdom):** keep the 4 non-dialog tests as render tests, and replace the 5th test with a `?raw` source assertion in the same file, mirroring the SourcesGrid convention:
> ```ts
> import rawSource from "./SourcesBulkBar.svelte?raw";
> const source = rawSource.replace(/\r\n/g, "\n");
> it("wires a confirm dialog to onDelete", () => {
>   expect(source).toContain("<ExtractumDialog");
>   expect(source).toContain("bind:open={confirmOpen}");
>   expect(source).toContain("onclick={confirmDelete}");
>   expect(source).toContain("onDelete()");
> });
> ```
> Then still verify the real dialog interaction live in Tauri in Task 3.

- [ ] **Step 5: Run the full check gate**

Run: `npm.cmd run check`
Expected: 0 errors (warnings unchanged from baseline).

- [ ] **Step 6: Commit**

```bash
git add src/lib/components/research-projects/SourcesBulkBar.svelte src/lib/components/research-projects/SourcesBulkBar.test.ts
git commit -m "feat(research-projects): add SourcesBulkBar with sync/delete + confirm dialog"
```

---

### Task 2: Wire `bulkBar` prop into `ResearchProjectsShell`

**Files:**
- Modify: `src/lib/components/research-projects/ResearchProjectsShell.svelte` (script props block + main column markup)
- Test: `src/lib/components/research-projects/ResearchProjectsShell.test.ts` (add one `?raw` assertion)

**Interfaces:**
- Consumes: `SourcesBulkBar` (Task 1) — prop bag `ComponentProps<typeof SourcesBulkBar>`.
- Produces: new optional shell prop `bulkBar?: ComponentProps<typeof SourcesBulkBar>`, rendered `<SourcesBulkBar {...bulkBar} />` above `.research-projects-shell__grid`, only when `bulkBar` is truthy.

- [ ] **Step 1: Write the failing test**

Add this test to `src/lib/components/research-projects/ResearchProjectsShell.test.ts` inside the existing `describe("ResearchProjectsShell", ...)`:

```ts
  it("renders the bulk-action bar above the grid when a bulkBar bag is provided", () => {
    expect(shellSource).toContain("<SourcesBulkBar");
    expect(shellSource).toContain("{...bulkBar}");
    // The bar must sit above the grid container in the main column.
    const barIndex = shellSource.indexOf("<SourcesBulkBar");
    const gridIndex = shellSource.indexOf('class="research-projects-shell__grid"');
    expect(barIndex).toBeGreaterThan(-1);
    expect(barIndex).toBeLessThan(gridIndex);
  });
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm.cmd run test:unit -- src/lib/components/research-projects/ResearchProjectsShell.test.ts`
Expected: FAIL — `shellSource` does not contain `<SourcesBulkBar` (barIndex === -1).

- [ ] **Step 3: Write minimal implementation**

In `ResearchProjectsShell.svelte`, add the import (after the `SourcesGrid` import, line 7):

```svelte
  import SourcesBulkBar from "./SourcesBulkBar.svelte";
```

Add the prop to the destructured props and the type block. In the `let { ... } = $props()` block add `bulkBar,` next to `inspector,`:

```svelte
    inspector,
    bulkBar,
    onSelectProject,
```

and in the type annotation add (next to `inspector?:`):

```svelte
    inspector?: ComponentProps<typeof Inspector>;
    bulkBar?: ComponentProps<typeof SourcesBulkBar>;
```

Render the bar above the grid — change the main column body so it reads:

```svelte
      {#if toolbar}
        <ProjectToolbar {...toolbar} />
      {/if}
      {#if bulkBar}
        <SourcesBulkBar {...bulkBar} />
      {/if}
      <div class="research-projects-shell__grid">
        <SourcesGrid {sources} {selectedSourceIds} {onSelectedSourceIdsChange} />
      </div>
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm.cmd run test:unit -- src/lib/components/research-projects/ResearchProjectsShell.test.ts`
Expected: PASS (existing tests + the new one).

- [ ] **Step 5: Run the full check gate**

Run: `npm.cmd run check`
Expected: 0 errors.

- [ ] **Step 6: Commit**

```bash
git add src/lib/components/research-projects/ResearchProjectsShell.svelte src/lib/components/research-projects/ResearchProjectsShell.test.ts
git commit -m "feat(research-projects): render SourcesBulkBar above the grid via bulkBar prop"
```

---

### Task 3: Page wiring in `/projects/next/+page.svelte` (syncable, sync, delete, clear)

**Files:**
- Modify: `src/routes/projects/next/+page.svelte`

**Interfaces:**
- Consumes: `SourcesBulkBar` bag shape from Task 1; `bulkBar` shell prop from Task 2; `syncYoutubeSource`, `removeProjectSources`, `listProjectSources` (Global Constraints).
- Produces: nothing downstream (leaf page).

- [ ] **Step 1: Add API imports**

Add `syncYoutubeSource` import (new line under existing imports):

```svelte
  import { syncYoutubeSource } from "$lib/api/source-jobs";
```

Add `removeProjectSources` to the existing `$lib/api/projects` import list (it currently imports `getProjectDataRange, listProjectSources, listResearchProjects, setProjectArchived, setProjectPinned, startProjectAnalysis`):

```svelte
    listResearchProjects,
    removeProjectSources,
    setProjectArchived,
```

- [ ] **Step 2: Add derived syncable state**

After the existing `let selectedSourceRow = $derived.by(...)` block (around line 72), add:

```svelte
  let syncableIds = $derived(
    sources
      .filter(
        (source) =>
          selectedSourceIds.includes(String(source.source_id)) &&
          source.provider === "youtube" &&
          (source.source_subtype === "video" || source.source_subtype === "playlist"),
      )
      .map((source) => source.source_id),
  );
  let bulkSyncDisabled = $derived(railState.saving || syncableIds.length === 0);
  let bulkSyncTitle = $derived(
    syncableIds.length === 0 ? "Нет источников, поддерживающих синхронизацию" : "",
  );
```

- [ ] **Step 3: Add the bulk-action handlers**

After the existing `runAnalysis` function (around line 117), add:

```svelte
  function clearSelection() {
    selectedSourceIds = [];
  }

  async function syncSelectedSources() {
    if (selectedProjectId === null || syncableIds.length === 0) return;
    railState = { ...railState, saving: true, status: "" };
    try {
      for (const id of syncableIds) {
        await syncYoutubeSource(id, { metadata: true, transcripts: true, comments: false });
      }
      sources = await listProjectSources(selectedProjectId);
    } catch (error) {
      railState = {
        ...railState,
        status: `Не удалось синхронизировать источники (${String(error)})`,
      };
    } finally {
      railState = { ...railState, saving: false };
    }
  }

  async function deleteSelectedSources() {
    if (selectedProjectId === null || selectedSourceIds.length === 0) return;
    railState = { ...railState, saving: true, status: "" };
    try {
      await removeProjectSources({
        projectId: selectedProjectId,
        sourceIds: selectedSourceIds.map((id) => Number(id)),
      });
      selectedSourceIds = [];
      sources = await listProjectSources(selectedProjectId);
      await workflow.reload();
    } catch (error) {
      railState = {
        ...railState,
        status: `Не удалось удалить источники (${String(error)})`,
      };
    } finally {
      railState = { ...railState, saving: false };
    }
  }
```

- [ ] **Step 4: Pass the `bulkBar` bag to the shell**

In the `<ResearchProjectsShell ... />` call, add a `bulkBar` attribute (place it next to `inspector={...}`):

```svelte
    bulkBar={selectedSourceIds.length > 0
      ? {
          count: selectedSourceIds.length,
          syncDisabled: bulkSyncDisabled,
          syncTitle: bulkSyncTitle,
          onClear: clearSelection,
          onSync: syncSelectedSources,
          onDelete: deleteSelectedSources,
        }
      : undefined}
```

- [ ] **Step 5: Run the full check gate**

Run: `npm.cmd run check`
Expected: 0 errors. In particular the import-boundary test must still pass (no `@svar-ui`/`bits-ui`/`$lib/components/ui/` added to the page).

- [ ] **Step 6: Verify live in the running Tauri app**

The svar grid selection + `invoke` flows do not run under jsdom, so verify in the app (Tauri MCP bridge, port 9223 — app is running):

1. Select a project that has youtube sources → select 1+ rows in the grid.
2. Confirm the bulk bar appears above the grid with «Выбрано: N» and «Снять выделение».
3. «Снять выделение» clears selection and the bar disappears.
4. With only non-youtube (or channel) sources selected, «Синхронизировать» is disabled and its tooltip reads «Нет источников, поддерживающих синхронизацию».
5. With a youtube video/playlist selected, «Синхронизировать» is enabled; clicking it triggers sync (watch backend logs / sync status) and the grid reloads.
6. «Удалить» opens the confirm dialog; «Да, удалить» removes the sources, clears selection, reloads the grid, and the rail summary counts update (`source_count`/`material_count`).

Capture a screenshot of the bulk bar (selected state) and of the confirm dialog as evidence.

- [ ] **Step 7: Commit**

```bash
git add src/routes/projects/next/+page.svelte
git commit -m "feat(research-projects): wire bulk sync/delete on /projects/next"
```

---

## Self-Review Notes

- **Spec coverage:** actions set (sync+delete) → Task 1/3; ExtractumDialog confirm → Task 1; sync only supported (youtube video/playlist), disabled + tooltip when none → Task 3 (`syncableIds`, `bulkSyncDisabled`, `bulkSyncTitle`); placement above grid, push not overlay → Task 2; page owns logic, component presentational → Tasks 1/3; error → `railState.status` via inline message; `saving` blocks buttons → `bulkSyncDisabled` includes `railState.saving`, delete guarded by `saving` set true during op; import boundary → wrappers only, asserted by `npm.cmd run check`. Testing: render test for the component, `?raw` for shell, live Tauri for page.
- **Type consistency:** `syncableIds: number[]` (source_id) feeds `syncYoutubeSource(id: number, ...)`; `selectedSourceIds: string[]` → `Number(id)` for `removeProjectSources.sourceIds: number[]`; bag matches `SourcesBulkBar` props exactly (`count`, `syncDisabled`, `syncTitle`, `onClear`, `onSync`, `onDelete`).
- **Note on delete button state:** the confirm dialog gates deletion, so an extra `disabled={saving}` on the bar's Удалить trigger is optional; `saving` already blocks re-entry inside `deleteSelectedSources`. Left out to keep the component free of a `saving` prop; add one later only if double-open becomes an issue.
