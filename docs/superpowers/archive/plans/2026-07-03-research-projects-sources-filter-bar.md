# Панель фильтров источников (/projects/next) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Панель управления над таблицей источников: кнопка «Фильтры» с бейджем, чипы, «N из M», «Добавить источник» (ConnectFromLibrary), раскрываемая строка фильтров; bulk-бар становится overlay.

**Architecture:** Клиентская фильтрация чистой функцией `filterProjectSources` до передачи в svar-грид (подход A из спеки). Два новых презентационных компонента (`SourcesFilterBar`, `SourcesFilterRow`), рефактор `SourcesBulkBar` в overlay, композиция в shell через проп-бэги, состояние фильтров — на странице.

**Tech Stack:** Svelte 5 runes, bits-ui Popover (через extractum-ui), `@testing-library/svelte` + jsdom, Tauri MCP для живой проверки.

**Source spec:** `docs/superpowers/specs/2026-07-03-research-projects-sources-filter-bar-design.md`

## Global Constraints

- **Import boundary**: файлы в `src/lib/components/research-projects/` и `src/routes/projects/` НЕ содержат `@svar-ui/`, `bits-ui`, `$lib/components/ui/` (даже в комментариях) — только `$lib/components/extractum-ui`.
- **UI-тексты русские** (кроме существующего англ. текста ConnectFromLibrary — не трогаем).
- **CSS-гочтча:** глобальное правило `button:not([data-slot="button"]) { background: var(--extractum-primary); color:#fff; }` — каждая нейтральная кнопка/поповер-триггер требует scoped-override (класс на элементе + scoped `<style>` = специфичность 0,2,0). Триггер `ExtractumPopoverTrigger` имеет `data-slot="popover-trigger"` — попадает под правило.
- **Статусы** источников: `"active" | "syncing" | "error" | "unavailable"` (тип `LibraryCatalogStatus`); цвета: active→`--extractum-success`, syncing→`--extractum-primary`, error→`--extractum-danger`, unavailable→`--extractum-warning`.
- **Провайдер-точки:** `var(--extractum-provider-telegram)`, `var(--extractum-provider-youtube)` (токены есть в base.css).
- **ConnectFromLibrary (verbatim):** каталог `listLibraryCatalog(): Promise<LibraryCatalogResponse>` (поле `.sources: LibraryCatalogRecord[]`) из `$lib/api/library-sources`; вью `buildLibrarySourcesView(catalogRecords, projectSources, selectedProjectViewId)` и `connectableSelection(sources, selectedIds: Set<string>)` и `projectViewId(projectId: number)` из `$lib/ui/research-projects-model`; подключение `addProjectSources({ projectId, sourceIds: number[] })` из `$lib/api/projects`.
- **Поля `ProjectSourceRecord`:** `source_id: number`, `provider`, `title: string|null`, `handle: string|null`, `item_count: number`, `last_synced_at: number|null` (unix-секунды), `sync_status: LibraryCatalogStatus`.
- **Тестовый раннер:** `node scripts/run-vitest.mjs run <files>`; гейт `npm.cmd run check` (baseline: 0 ошибок, 2 warning в `ProjectsShell.svelte`).
- **Ошибки** страничных операций → `railState.status` формата `Не удалось … (${String(error)})`; `railState.saving` на время операции.
- **Коммит на задачу; push не делать.** Работать на feature-ветке (создать `feat/research-projects-sources-filter-bar` от main, план закоммитить первым коммитом).

---

### Task 1: Чистый модуль фильтров

**Files:**
- Create: `src/lib/ui/research-projects-source-filters.ts`
- Test: `src/lib/ui/research-projects-source-filters.test.ts`

**Interfaces:**
- Consumes: `ProjectSourceRecord` из `$lib/types/projects`.
- Produces (используются Task 3 и Task 5):

```ts
export interface SourceFilters {
  query: string;
  types: string[];
  statuses: string[];
  materialsMin: number | null;
  materialsMax: number | null;
  syncedFrom: string | null; // "YYYY-MM-DD"
  syncedTo: string | null;
}
export function emptySourceFilters(): SourceFilters;
export function countActiveSourceFilters(filters: SourceFilters): number;
export function filterProjectSources(records: ProjectSourceRecord[], filters: SourceFilters): ProjectSourceRecord[];
export interface SourceFilterChip { key: string; label: string; dot: string | null }
export function buildSourceFilterChips(filters: SourceFilters): SourceFilterChip[];
export function removeSourceFilterChip(filters: SourceFilters, key: string): SourceFilters;
```

- [ ] **Step 1: Write the failing tests**

`src/lib/ui/research-projects-source-filters.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  buildSourceFilterChips,
  countActiveSourceFilters,
  emptySourceFilters,
  filterProjectSources,
  removeSourceFilterChip,
  type SourceFilters,
} from "./research-projects-source-filters";
import type { ProjectSourceRecord } from "$lib/types/projects";

function record(overrides: Partial<ProjectSourceRecord> = {}): ProjectSourceRecord {
  return {
    project_id: 1,
    source_id: 10,
    provider: "telegram",
    source_subtype: "channel",
    title: "ФинБеларусь",
    subtitle: null,
    item_count: 339,
    added_at: 0,
    last_synced_at: Date.UTC(2026, 5, 2, 12, 0, 0) / 1000, // 2026-06-02 12:00 UTC
    sync_status: "active",
    handle: "@finbelarus",
    ...overrides,
  };
}

function filters(overrides: Partial<SourceFilters> = {}): SourceFilters {
  return { ...emptySourceFilters(), ...overrides };
}

describe("filterProjectSources", () => {
  it("returns the same array for empty filters", () => {
    const records = [record()];
    expect(filterProjectSources(records, emptySourceFilters())).toEqual(records);
  });

  it("filters by query over title and handle, case-insensitively", () => {
    const records = [
      record({ source_id: 1, title: "ФинБеларусь", handle: "@fin" }),
      record({ source_id: 2, title: "WhiteBird", handle: "@whitebird_io" }),
    ];
    expect(filterProjectSources(records, filters({ query: "финбел" })).map((r) => r.source_id)).toEqual([1]);
    expect(filterProjectSources(records, filters({ query: "BIRD_io" })).map((r) => r.source_id)).toEqual([2]);
  });

  it("filters by provider types and sync statuses", () => {
    const records = [
      record({ source_id: 1, provider: "telegram", sync_status: "active" }),
      record({ source_id: 2, provider: "youtube", sync_status: "error" }),
    ];
    expect(filterProjectSources(records, filters({ types: ["youtube"] })).map((r) => r.source_id)).toEqual([2]);
    expect(filterProjectSources(records, filters({ statuses: ["active"] })).map((r) => r.source_id)).toEqual([1]);
  });

  it("filters by materials range", () => {
    const records = [
      record({ source_id: 1, item_count: 10 }),
      record({ source_id: 2, item_count: 500 }),
    ];
    expect(filterProjectSources(records, filters({ materialsMin: 100 })).map((r) => r.source_id)).toEqual([2]);
    expect(filterProjectSources(records, filters({ materialsMax: 100 })).map((r) => r.source_id)).toEqual([1]);
  });

  it("filters by last-synced date range and drops null dates when range set", () => {
    const day = (iso: string) => new Date(`${iso}T12:00:00`).getTime() / 1000;
    const records = [
      record({ source_id: 1, last_synced_at: day("2026-05-10") }),
      record({ source_id: 2, last_synced_at: day("2026-06-20") }),
      record({ source_id: 3, last_synced_at: null }),
    ];
    expect(
      filterProjectSources(records, filters({ syncedFrom: "2026-06-01" })).map((r) => r.source_id),
    ).toEqual([2]);
    expect(
      filterProjectSources(records, filters({ syncedTo: "2026-05-31" })).map((r) => r.source_id),
    ).toEqual([1]);
    // границы включительно: запись, синхронизированная в тот же день, проходит
    expect(
      filterProjectSources(
        records,
        filters({ syncedFrom: "2026-05-10", syncedTo: "2026-05-10" }),
      ).map((r) => r.source_id),
    ).toEqual([1]);
  });
});

describe("chips", () => {
  it("counts active filters and builds chips with dots", () => {
    const f = filters({
      query: "фин",
      types: ["telegram"],
      statuses: ["error"],
      materialsMin: 10,
      syncedFrom: "2026-05-01",
    });
    expect(countActiveSourceFilters(f)).toBe(5);
    const chips = buildSourceFilterChips(f);
    expect(chips.map((c) => c.key)).toEqual([
      "query",
      "type:telegram",
      "status:error",
      "materials",
      "period",
    ]);
    expect(chips[0].label).toBe("Источник: фин");
    expect(chips[1].dot).toBe("var(--extractum-provider-telegram)");
    expect(chips[2].dot).toBe("var(--extractum-danger)");
    expect(chips[3].label).toBe("Материалы: 10–∞");
    expect(chips[4].label).toBe("Период: 01.05.2026–…");
  });

  it("removeSourceFilterChip resets only the matching part", () => {
    const f = filters({ query: "x", types: ["telegram", "youtube"], materialsMin: 1, materialsMax: 2 });
    expect(removeSourceFilterChip(f, "query").query).toBe("");
    expect(removeSourceFilterChip(f, "type:telegram").types).toEqual(["youtube"]);
    const noMaterials = removeSourceFilterChip(f, "materials");
    expect(noMaterials.materialsMin).toBeNull();
    expect(noMaterials.materialsMax).toBeNull();
    // остальное не тронуто
    expect(noMaterials.query).toBe("x");
  });

  it("empty filters produce no chips and zero count", () => {
    expect(buildSourceFilterChips(emptySourceFilters())).toEqual([]);
    expect(countActiveSourceFilters(emptySourceFilters())).toBe(0);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node scripts/run-vitest.mjs run src/lib/ui/research-projects-source-filters.test.ts`
Expected: FAIL — module not found.

- [ ] **Step 3: Write the implementation**

`src/lib/ui/research-projects-source-filters.ts`:

```ts
import type { ProjectSourceRecord } from "$lib/types/projects";

export interface SourceFilters {
  query: string;
  types: string[];
  statuses: string[];
  materialsMin: number | null;
  materialsMax: number | null;
  /** "YYYY-MM-DD" (значение input type=date) */
  syncedFrom: string | null;
  syncedTo: string | null;
}

export function emptySourceFilters(): SourceFilters {
  return {
    query: "",
    types: [],
    statuses: [],
    materialsMin: null,
    materialsMax: null,
    syncedFrom: null,
    syncedTo: null,
  };
}

const PROVIDER_DOTS: Record<string, string> = {
  telegram: "var(--extractum-provider-telegram)",
  youtube: "var(--extractum-provider-youtube)",
};

const STATUS_DOTS: Record<string, string> = {
  active: "var(--extractum-success)",
  syncing: "var(--extractum-primary)",
  error: "var(--extractum-danger)",
  unavailable: "var(--extractum-warning)",
};

/** Начало локального дня в unix-секундах. */
function dayStart(iso: string): number {
  return new Date(`${iso}T00:00:00`).getTime() / 1000;
}

/** Конец локального дня (включительно) в unix-секундах. */
function dayEnd(iso: string): number {
  return dayStart(iso) + 86_399;
}

export function countActiveSourceFilters(filters: SourceFilters): number {
  return buildSourceFilterChips(filters).length;
}

export function filterProjectSources(
  records: ProjectSourceRecord[],
  filters: SourceFilters,
): ProjectSourceRecord[] {
  const q = filters.query.trim().toLowerCase();
  return records.filter((record) => {
    if (q) {
      const title = (record.title ?? "").toLowerCase();
      const handle = (record.handle ?? "").toLowerCase();
      if (!title.includes(q) && !handle.includes(q)) return false;
    }
    if (filters.types.length > 0 && !filters.types.includes(record.provider)) return false;
    if (filters.statuses.length > 0 && !filters.statuses.includes(record.sync_status)) return false;
    if (filters.materialsMin !== null && record.item_count < filters.materialsMin) return false;
    if (filters.materialsMax !== null && record.item_count > filters.materialsMax) return false;
    if (filters.syncedFrom !== null || filters.syncedTo !== null) {
      if (record.last_synced_at === null) return false;
      if (filters.syncedFrom !== null && record.last_synced_at < dayStart(filters.syncedFrom)) {
        return false;
      }
      if (filters.syncedTo !== null && record.last_synced_at > dayEnd(filters.syncedTo)) {
        return false;
      }
    }
    return true;
  });
}

export interface SourceFilterChip {
  key: string;
  label: string;
  dot: string | null;
}

function dateLabel(iso: string): string {
  const [y, m, d] = iso.split("-");
  return `${d}.${m}.${y}`;
}

export function buildSourceFilterChips(filters: SourceFilters): SourceFilterChip[] {
  const chips: SourceFilterChip[] = [];
  if (filters.query.trim()) {
    chips.push({ key: "query", label: `Источник: ${filters.query.trim()}`, dot: null });
  }
  for (const type of filters.types) {
    chips.push({ key: `type:${type}`, label: `Тип: ${type}`, dot: PROVIDER_DOTS[type] ?? null });
  }
  for (const status of filters.statuses) {
    chips.push({
      key: `status:${status}`,
      label: `Статус: ${status}`,
      dot: STATUS_DOTS[status] ?? null,
    });
  }
  if (filters.materialsMin !== null || filters.materialsMax !== null) {
    const min = filters.materialsMin ?? 0;
    const max = filters.materialsMax ?? "∞";
    chips.push({ key: "materials", label: `Материалы: ${min}–${max}`, dot: null });
  }
  if (filters.syncedFrom !== null || filters.syncedTo !== null) {
    const from = filters.syncedFrom ? dateLabel(filters.syncedFrom) : "…";
    const to = filters.syncedTo ? dateLabel(filters.syncedTo) : "…";
    chips.push({ key: "period", label: `Период: ${from}–${to}`, dot: null });
  }
  return chips;
}

export function removeSourceFilterChip(filters: SourceFilters, key: string): SourceFilters {
  if (key === "query") return { ...filters, query: "" };
  if (key === "materials") return { ...filters, materialsMin: null, materialsMax: null };
  if (key === "period") return { ...filters, syncedFrom: null, syncedTo: null };
  if (key.startsWith("type:")) {
    const value = key.slice("type:".length);
    return { ...filters, types: filters.types.filter((t) => t !== value) };
  }
  if (key.startsWith("status:")) {
    const value = key.slice("status:".length);
    return { ...filters, statuses: filters.statuses.filter((s) => s !== value) };
  }
  return filters;
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `node scripts/run-vitest.mjs run src/lib/ui/research-projects-source-filters.test.ts`
Expected: PASS.

- [ ] **Step 5: Commit** (сначала создать ветку, если ещё на main: `git checkout -b feat/research-projects-sources-filter-bar`; закоммитить план отдельным коммитом `docs: add sources filter bar implementation plan`)

```bash
git add src/lib/ui/research-projects-source-filters.ts src/lib/ui/research-projects-source-filters.test.ts
git commit -m "feat(research-projects): add source filters pure module"
```

---

### Task 2: SourcesFilterBar (stats-бар)

**Files:**
- Create: `src/lib/components/research-projects/SourcesFilterBar.svelte`
- Test: `src/lib/components/research-projects/SourcesFilterBar.test.ts`

**Interfaces:**
- Consumes: `SourceFilterChip` (Task 1).
- Produces: `SourcesFilterBar` с пропсами
  `{ filtersOpen: boolean; onToggleFilters?: () => void; chips?: SourceFilterChip[]; onRemoveChip?: (key: string) => void; filtersActive?: boolean; onClearAll?: () => void; shownCount: number; totalCount: number; onAddSource?: () => void }`.
  Доступные имена: кнопка «Фильтры», бейдж-счётчик, ссылка «Сбросить», текст «{N} из {M}», кнопка «Добавить источник», чип-кнопки удаления `aria-label="Убрать фильтр {label}"`.

- [ ] **Step 1: Write the failing tests**

`src/lib/components/research-projects/SourcesFilterBar.test.ts`:

```ts
// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import SourcesFilterBar from "./SourcesFilterBar.svelte";

afterEach(cleanup);

const base = { filtersOpen: false, shownCount: 8, totalCount: 10 };

describe("SourcesFilterBar", () => {
  it("shows the counter and toggles the filter row", async () => {
    const onToggleFilters = vi.fn();
    render(SourcesFilterBar, { props: { ...base, onToggleFilters } });
    expect(screen.getByText("8 из 10")).toBeTruthy();
    await fireEvent.click(screen.getByText("Фильтры"));
    expect(onToggleFilters).toHaveBeenCalledOnce();
  });

  it("shows the badge with the active filter count only when filters are active", () => {
    const { unmount } = render(SourcesFilterBar, { props: { ...base } });
    expect(document.querySelector(".sources-filter-bar__badge")).toBeNull();
    unmount();

    render(SourcesFilterBar, {
      props: {
        ...base,
        filtersActive: true,
        chips: [
          { key: "type:telegram", label: "Тип: telegram", dot: "var(--extractum-provider-telegram)" },
          { key: "query", label: "Источник: фин", dot: null },
        ],
      },
    });
    expect(document.querySelector(".sources-filter-bar__badge")?.textContent).toBe("2");
  });

  it("renders chips and removes one by its close button", async () => {
    const onRemoveChip = vi.fn();
    render(SourcesFilterBar, {
      props: {
        ...base,
        filtersActive: true,
        chips: [{ key: "query", label: "Источник: фин", dot: null }],
        onRemoveChip,
      },
    });
    expect(screen.getByText("Источник: фин")).toBeTruthy();
    await fireEvent.click(screen.getByLabelText("Убрать фильтр Источник: фин"));
    expect(onRemoveChip).toHaveBeenCalledWith("query");
  });

  it("shows «Сбросить» only when filters are active and forwards the click", async () => {
    const onClearAll = vi.fn();
    const { unmount } = render(SourcesFilterBar, { props: { ...base } });
    expect(screen.queryByText("Сбросить")).toBeNull();
    unmount();

    render(SourcesFilterBar, { props: { ...base, filtersActive: true, onClearAll } });
    await fireEvent.click(screen.getByText("Сбросить"));
    expect(onClearAll).toHaveBeenCalledOnce();
  });

  it("forwards add-source clicks", async () => {
    const onAddSource = vi.fn();
    render(SourcesFilterBar, { props: { ...base, onAddSource } });
    await fireEvent.click(screen.getByText("Добавить источник"));
    expect(onAddSource).toHaveBeenCalledOnce();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/SourcesFilterBar.test.ts`
Expected: FAIL — module not found.

- [ ] **Step 3: Implement**

`src/lib/components/research-projects/SourcesFilterBar.svelte`:

```svelte
<script lang="ts">
  import type { SourceFilterChip } from "$lib/ui/research-projects-source-filters";

  let {
    filtersOpen,
    onToggleFilters,
    chips = [],
    onRemoveChip,
    filtersActive = false,
    onClearAll,
    shownCount,
    totalCount,
    onAddSource,
  }: {
    filtersOpen: boolean;
    onToggleFilters?: () => void;
    chips?: SourceFilterChip[];
    onRemoveChip?: (key: string) => void;
    filtersActive?: boolean;
    onClearAll?: () => void;
    shownCount: number;
    totalCount: number;
    onAddSource?: () => void;
  } = $props();
</script>

<div class="sources-filter-bar">
  <div class="sources-filter-bar__left">
    <button
      type="button"
      class="sources-filter-bar__filters-btn"
      aria-expanded={filtersOpen}
      onclick={() => onToggleFilters?.()}
    >
      <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M2 3.5h12l-4.5 5v4l-3 1.5V8.5z" />
      </svg>
      Фильтры
      {#if filtersActive}
        <span class="sources-filter-bar__badge">{chips.length}</span>
      {/if}
    </button>
    {#each chips as chip (chip.key)}
      <span class="sources-filter-bar__chip">
        {#if chip.dot}
          <span class="sources-filter-bar__chip-dot" style:background={chip.dot}></span>
        {/if}
        {chip.label}
        <button
          type="button"
          class="sources-filter-bar__chip-remove"
          aria-label={`Убрать фильтр ${chip.label}`}
          onclick={() => onRemoveChip?.(chip.key)}
        >
          ✕
        </button>
      </span>
    {/each}
    {#if filtersActive}
      <button type="button" class="sources-filter-bar__clear" onclick={() => onClearAll?.()}>
        Сбросить
      </button>
    {/if}
    <span class="sources-filter-bar__count">{shownCount} из {totalCount}</span>
  </div>
  <button
    type="button"
    class="sources-filter-bar__add"
    title="Подключить источник из библиотеки"
    onclick={() => onAddSource?.()}
  >
    <span class="sources-filter-bar__add-plus">+</span>Добавить источник
  </button>
</div>

<style>
  .sources-filter-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    padding: 11px 14px 9px;
    background: var(--extractum-surface-raised);
    border-bottom: 1px solid var(--extractum-border-subtle, var(--extractum-border));
  }

  .sources-filter-bar__left {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
    min-width: 0;
  }

  /* scoped override глобального button-правила */
  .sources-filter-bar__left .sources-filter-bar__filters-btn {
    height: 30px;
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 0 11px;
    border: 1px solid var(--extractum-border);
    border-radius: 6px;
    background: var(--extractum-surface);
    color: var(--extractum-text);
    font: 600 12px/1 var(--extractum-font);
    cursor: pointer;
  }

  .sources-filter-bar__left .sources-filter-bar__filters-btn:hover {
    background: var(--extractum-surface-subtle);
  }

  .sources-filter-bar__badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    border-radius: 8px;
    background: var(--extractum-primary);
    color: #fff;
    font: 700 10px/1 var(--extractum-font);
  }

  .sources-filter-bar__chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 24px;
    padding: 0 6px 0 9px;
    border-radius: 6px;
    background: color-mix(in srgb, var(--extractum-primary) 8%, var(--extractum-surface));
    border: 1px solid color-mix(in srgb, var(--extractum-primary) 24%, transparent);
    font: 600 11.5px/1 var(--extractum-font);
    color: var(--extractum-primary);
  }

  .sources-filter-bar__chip-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }

  .sources-filter-bar__chip .sources-filter-bar__chip-remove {
    border: none;
    background: transparent;
    padding: 0;
    cursor: pointer;
    color: color-mix(in srgb, var(--extractum-primary) 55%, transparent);
    font-size: 13px;
    line-height: 1;
  }

  .sources-filter-bar__left .sources-filter-bar__clear {
    border: none;
    background: transparent;
    padding: 0;
    font: 500 11.5px/1 var(--extractum-font);
    color: var(--extractum-muted);
    cursor: pointer;
    text-decoration: underline;
    text-underline-offset: 2px;
  }

  .sources-filter-bar__count {
    font: 500 11.5px/1 var(--extractum-font);
    color: var(--extractum-muted-2);
  }

  .sources-filter-bar > .sources-filter-bar__add {
    height: 28px;
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 0 11px;
    border: 1px solid var(--extractum-primary);
    border-radius: 6px;
    background: color-mix(in srgb, var(--extractum-primary) 6%, transparent);
    color: var(--extractum-primary);
    font: 600 12px/1 var(--extractum-font);
    cursor: pointer;
  }

  .sources-filter-bar > .sources-filter-bar__add:hover {
    background: color-mix(in srgb, var(--extractum-primary) 12%, transparent);
  }

  .sources-filter-bar__add-plus {
    font-size: 14px;
  }
</style>
```

- [ ] **Step 4: Run test to verify it passes**

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/SourcesFilterBar.test.ts`
Expected: PASS.

- [ ] **Step 5: Full check gate + commit**

Run: `npm.cmd run check` — 0 ошибок.

```bash
git add src/lib/components/research-projects/SourcesFilterBar.svelte src/lib/components/research-projects/SourcesFilterBar.test.ts
git commit -m "feat(research-projects): add SourcesFilterBar stats bar"
```

---

### Task 3: SourcesFilterRow (строка фильтров)

**Files:**
- Create: `src/lib/components/research-projects/SourcesFilterRow.svelte`
- Test: `src/lib/components/research-projects/SourcesFilterRow.test.ts`

**Interfaces:**
- Consumes: `SourceFilters`, `emptySourceFilters` (Task 1); `ExtractumPopover`, `ExtractumPopoverTrigger`, `ExtractumPopoverContent` из extractum-ui.
- Produces: `SourcesFilterRow` с пропсами `{ filters: SourceFilters; onChange?: (filters: SourceFilters) => void }`. Доступные имена: `placeholder="Поиск"`, `aria-label="Материалы от"|"Материалы до"|"Синхронизирован с"|"Синхронизирован по"`, триггеры `aria-label="Фильтр по типу"` и `aria-label="Фильтр по статусу"`, чекбоксы в поповерах с label текста опции.

- [ ] **Step 1: Write the failing tests**

`src/lib/components/research-projects/SourcesFilterRow.test.ts`:

```ts
// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import SourcesFilterRow from "./SourcesFilterRow.svelte";
import { emptySourceFilters } from "$lib/ui/research-projects-source-filters";

afterEach(cleanup);

describe("SourcesFilterRow", () => {
  it("emits a new filters object when the search query changes", async () => {
    const onChange = vi.fn();
    render(SourcesFilterRow, { props: { filters: emptySourceFilters(), onChange } });
    await fireEvent.input(screen.getByPlaceholderText("Поиск"), { target: { value: "фин" } });
    expect(onChange).toHaveBeenCalledWith({ ...emptySourceFilters(), query: "фин" });
  });

  it("clears the query with the × button", async () => {
    const onChange = vi.fn();
    render(SourcesFilterRow, {
      props: { filters: { ...emptySourceFilters(), query: "фин" }, onChange },
    });
    await fireEvent.click(screen.getByTitle("Очистить поиск"));
    expect(onChange).toHaveBeenCalledWith(emptySourceFilters());
  });

  it("toggles a provider type through the type popover", async () => {
    const onChange = vi.fn();
    render(SourcesFilterRow, { props: { filters: emptySourceFilters(), onChange } });
    await fireEvent.click(screen.getByLabelText("Фильтр по типу"));
    await fireEvent.click(await screen.findByLabelText("telegram"));
    expect(onChange).toHaveBeenCalledWith({ ...emptySourceFilters(), types: ["telegram"] });
  });

  it("shows the selected type in the trigger label", () => {
    render(SourcesFilterRow, {
      props: { filters: { ...emptySourceFilters(), types: ["youtube"] } },
    });
    expect(screen.getByLabelText("Фильтр по типу").textContent).toContain("youtube");
  });

  it("toggles a status through the status popover", async () => {
    const onChange = vi.fn();
    render(SourcesFilterRow, { props: { filters: emptySourceFilters(), onChange } });
    await fireEvent.click(screen.getByLabelText("Фильтр по статусу"));
    await fireEvent.click(await screen.findByLabelText("error"));
    expect(onChange).toHaveBeenCalledWith({ ...emptySourceFilters(), statuses: ["error"] });
  });

  it("emits materials and date range updates", async () => {
    const onChange = vi.fn();
    render(SourcesFilterRow, { props: { filters: emptySourceFilters(), onChange } });

    await fireEvent.input(screen.getByLabelText("Материалы от"), { target: { value: "10" } });
    expect(onChange).toHaveBeenCalledWith({ ...emptySourceFilters(), materialsMin: 10 });

    await fireEvent.input(screen.getByLabelText("Синхронизирован с"), {
      target: { value: "2026-05-01" },
    });
    expect(onChange).toHaveBeenCalledWith({ ...emptySourceFilters(), syncedFrom: "2026-05-01" });
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/SourcesFilterRow.test.ts`
Expected: FAIL — module not found.

- [ ] **Step 3: Implement**

`src/lib/components/research-projects/SourcesFilterRow.svelte`:

```svelte
<script lang="ts">
  import {
    ExtractumPopover,
    ExtractumPopoverTrigger,
    ExtractumPopoverContent,
  } from "$lib/components/extractum-ui";
  import type { SourceFilters } from "$lib/ui/research-projects-source-filters";

  let {
    filters,
    onChange,
  }: {
    filters: SourceFilters;
    onChange?: (filters: SourceFilters) => void;
  } = $props();

  const TYPE_OPTIONS = [
    { value: "telegram", dot: "var(--extractum-provider-telegram)" },
    { value: "youtube", dot: "var(--extractum-provider-youtube)" },
  ];

  const STATUS_OPTIONS = [
    { value: "active", dot: "var(--extractum-success)" },
    { value: "syncing", dot: "var(--extractum-primary)" },
    { value: "error", dot: "var(--extractum-danger)" },
    { value: "unavailable", dot: "var(--extractum-warning)" },
  ];

  function patch(partial: Partial<SourceFilters>) {
    onChange?.({ ...filters, ...partial });
  }

  function toggleIn(list: string[], value: string): string[] {
    return list.includes(value) ? list.filter((v) => v !== value) : [...list, value];
  }

  function multiLabel(selected: string[]): string {
    if (selected.length === 0) return "Все";
    if (selected.length === 1) return selected[0];
    return `${selected.length} выбр.`;
  }

  function numberOrNull(raw: string): number | null {
    const value = Number(raw.replace(/\D/g, ""));
    return raw.trim() === "" || !Number.isFinite(value) ? null : value;
  }
</script>

<div class="sources-filter-row">
  <div></div>

  <div class="sources-filter-row__search">
    <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
      <circle cx="7" cy="7" r="4.5" />
      <path d="M10.5 10.5L14 14" />
    </svg>
    <input
      value={filters.query}
      placeholder="Поиск"
      oninput={(e) => patch({ query: (e.currentTarget as HTMLInputElement).value })}
    />
    {#if filters.query.length > 0}
      <button
        type="button"
        class="sources-filter-row__clear"
        title="Очистить поиск"
        onclick={() => patch({ query: "" })}
      >
        ×
      </button>
    {/if}
  </div>

  <ExtractumPopover>
    <ExtractumPopoverTrigger class="sources-filter-row__dd" aria-label="Фильтр по типу">
      {multiLabel(filters.types)}<span class="sources-filter-row__caret">▾</span>
    </ExtractumPopoverTrigger>
    <ExtractumPopoverContent class="sources-filter-row__popover" align="start">
      {#each TYPE_OPTIONS as option (option.value)}
        <label class="sources-filter-row__option">
          <input
            type="checkbox"
            aria-label={option.value}
            checked={filters.types.includes(option.value)}
            onchange={() => patch({ types: toggleIn(filters.types, option.value) })}
          />
          <span class="sources-filter-row__dot" style:background={option.dot}></span>
          {option.value}
        </label>
      {/each}
    </ExtractumPopoverContent>
  </ExtractumPopover>

  <div class="sources-filter-row__range">
    <input
      type="number"
      aria-label="Материалы от"
      placeholder="от"
      value={filters.materialsMin ?? ""}
      oninput={(e) => patch({ materialsMin: numberOrNull((e.currentTarget as HTMLInputElement).value) })}
    />
    <input
      type="number"
      aria-label="Материалы до"
      placeholder="до"
      value={filters.materialsMax ?? ""}
      oninput={(e) => patch({ materialsMax: numberOrNull((e.currentTarget as HTMLInputElement).value) })}
    />
  </div>

  <div class="sources-filter-row__range">
    <input
      type="date"
      aria-label="Синхронизирован с"
      value={filters.syncedFrom ?? ""}
      oninput={(e) => patch({ syncedFrom: (e.currentTarget as HTMLInputElement).value || null })}
    />
    <input
      type="date"
      aria-label="Синхронизирован по"
      value={filters.syncedTo ?? ""}
      oninput={(e) => patch({ syncedTo: (e.currentTarget as HTMLInputElement).value || null })}
    />
  </div>

  <ExtractumPopover>
    <ExtractumPopoverTrigger class="sources-filter-row__dd" aria-label="Фильтр по статусу">
      {multiLabel(filters.statuses)}<span class="sources-filter-row__caret">▾</span>
    </ExtractumPopoverTrigger>
    <ExtractumPopoverContent class="sources-filter-row__popover" align="end">
      {#each STATUS_OPTIONS as option (option.value)}
        <label class="sources-filter-row__option">
          <input
            type="checkbox"
            aria-label={option.value}
            checked={filters.statuses.includes(option.value)}
            onchange={() => patch({ statuses: toggleIn(filters.statuses, option.value) })}
          />
          <span class="sources-filter-row__dot" style:background={option.dot}></span>
          {option.value}
        </label>
      {/each}
    </ExtractumPopoverContent>
  </ExtractumPopover>
</div>

<style>
  .sources-filter-row {
    display: grid;
    grid-template-columns: 34px minmax(0, 1fr) 116px 116px 150px 104px;
    align-items: center;
    gap: 6px;
    padding: 6px 8px;
    background: var(--extractum-surface);
    border-bottom: 1px solid var(--extractum-border-subtle, var(--extractum-border));
  }

  .sources-filter-row__search {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    border: 1px solid var(--extractum-border);
    border-radius: 5px;
    background: var(--extractum-surface-raised);
    padding: 0 8px;
    color: var(--extractum-muted-2);
  }

  .sources-filter-row__search input {
    flex: 1;
    min-width: 0;
    border: none;
    outline: none;
    background: transparent;
    font: 400 12px/1 var(--extractum-font);
    color: var(--extractum-text);
  }

  .sources-filter-row__search .sources-filter-row__clear {
    border: none;
    background: transparent;
    padding: 0;
    cursor: pointer;
    color: var(--extractum-muted-2);
    font-size: 14px;
    line-height: 1;
  }

  /* поповер-триггеры: override глобального button-правила */
  .sources-filter-row :global(.sources-filter-row__dd) {
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    border: 1px solid var(--extractum-border);
    border-radius: 5px;
    background: var(--extractum-surface-raised);
    padding: 0 8px;
    font: 400 12px/1 var(--extractum-font);
    color: var(--extractum-text);
    cursor: pointer;
  }

  .sources-filter-row__caret {
    color: var(--extractum-muted-2);
    font-size: 10px;
  }

  :global(.sources-filter-row__popover) {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 160px;
    padding: 5px;
  }

  :global(.sources-filter-row__popover) .sources-filter-row__option,
  :global(.sources-filter-row__popover .sources-filter-row__option) {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: 5px;
    font: 500 12px/1 var(--extractum-font);
    color: var(--extractum-text);
    cursor: pointer;
  }

  :global(.sources-filter-row__popover .sources-filter-row__option:hover) {
    background: var(--extractum-surface-subtle);
  }

  .sources-filter-row__dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }

  .sources-filter-row__range {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .sources-filter-row__range input {
    width: 0;
    flex: 1;
    height: 28px;
    border: 1px solid var(--extractum-border);
    border-radius: 5px;
    background: var(--extractum-surface-raised);
    padding: 0 6px;
    font: 400 11.5px/1 var(--extractum-font);
    color: var(--extractum-text);
    outline: none;
  }
</style>
```

Замечания для исполнителя:
- Опции в поповере — `<label>` с нативным чекбоксом (не bits-ui Checkbox): jsdom-надёжно и доступно; `aria-label` на чекбоксе = значение опции.
- Если `ExtractumPopoverContent` не принимает `class` — обернуть содержимое в `<div class="sources-filter-row__popover">`.
- number-инпуты браузер может отдавать пустую строку для невалидного ввода — `numberOrNull` это обрабатывает.

- [ ] **Step 4: Run test to verify it passes**

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/SourcesFilterRow.test.ts`
Expected: PASS. (bits-ui Popover рендерится в jsdom — конвенция проекта; при сюрпризе с монтированием контента — `findByLabelText` уже асинхронный, портал успевает.)

- [ ] **Step 5: Full check gate + commit**

Run: `npm.cmd run check` — 0 ошибок.

```bash
git add src/lib/components/research-projects/SourcesFilterRow.svelte src/lib/components/research-projects/SourcesFilterRow.test.ts
git commit -m "feat(research-projects): add SourcesFilterRow controls"
```

---

### Task 4: BulkBar → overlay; Shell: statsbar-контейнер, filterRow, gridOverlay

**Files:**
- Modify: `src/lib/components/research-projects/SourcesBulkBar.svelte` (только `<style>` корня)
- Modify: `src/lib/components/research-projects/SourcesGrid.svelte` (проп `overlay`)
- Modify: `src/lib/components/research-projects/ResearchProjectsShell.svelte`
- Test: `src/lib/components/research-projects/ResearchProjectsShell.test.ts`, `src/lib/components/research-projects/SourcesGrid.test.ts`

**Interfaces:**
- Consumes: `SourcesFilterBar` (Task 2), `SourcesFilterRow` (Task 3).
- Produces: shell-пропсы `filterBar?: ComponentProps<typeof SourcesFilterBar>`, `filterRow?: ComponentProps<typeof SourcesFilterRow>`, `gridOverlay?: string`; `SourcesGrid` проп `overlay?: string` (default `"Нет источников"`). `bulkBar` рендерится ВНУТРИ statsbar-контейнера (overlay) — но только когда передан `filterBar`; без `filterBar` bulkBar рендерится как раньше (полосой) для обратной совместимости не нужен — страница всегда передаёт filterBar при выбранном проекте, поэтому просто перенести.

- [ ] **Step 1: Update tests (failing)**

В `ResearchProjectsShell.test.ts` заменить тест `renders the bulk-action bar above the grid when a bulkBar bag is provided` на:

```ts
  it("renders the stats bar with the bulk overlay inside and the filter row above the grid", () => {
    expect(shellSource).toContain("<SourcesFilterBar");
    expect(shellSource).toContain("{...filterBar}");
    expect(shellSource).toContain("<SourcesFilterRow");
    expect(shellSource).toContain("{...filterRow}");
    // bulk-бар живёт внутри statsbar-контейнера (overlay поверх фильтров)
    const statsIndex = shellSource.indexOf('class="research-projects-shell__statsbar"');
    const bulkIndex = shellSource.indexOf("<SourcesBulkBar");
    const gridIndex = shellSource.indexOf('class="research-projects-shell__grid"');
    expect(statsIndex).toBeGreaterThan(-1);
    expect(bulkIndex).toBeGreaterThan(statsIndex);
    expect(bulkIndex).toBeLessThan(gridIndex);
    expect(shellSource).toContain("overlay={gridOverlay}");
  });
```

В `SourcesGrid.test.ts` заменить ассерт `expect(source).toContain('overlay="Нет источников"');` на:

```ts
    expect(source).toContain('overlay = "Нет источников"');
    expect(source).toContain("{overlay}");
```

(проп с дефолтом + прокидывание в грид).

- [ ] **Step 2: Run tests to verify they fail**

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/ResearchProjectsShell.test.ts src/lib/components/research-projects/SourcesGrid.test.ts`
Expected: FAIL (нет SourcesFilterBar/overlay-пропа).

- [ ] **Step 3: SourcesGrid — проп overlay**

В `SourcesGrid.svelte` в пропсы добавить:

```svelte
  let {
    sources,
    selectedSourceIds = [],
    onSelectedSourceIdsChange = () => {},
    overlay = "Нет источников",
  }: {
    sources: ProjectSourceRecord[];
    selectedSourceIds?: string[];
    onSelectedSourceIdsChange?: (ids: string[]) => void;
    overlay?: string;
  } = $props();
```

и в разметке `overlay="Нет источников"` → `{overlay}`.

- [ ] **Step 4: SourcesBulkBar — overlay-стили**

В `SourcesBulkBar.svelte` заменить стиль корня `.sources-bulk-bar`:

```css
  .sources-bulk-bar {
    position: absolute;
    inset: 0;
    z-index: 5;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 14px;
    background: color-mix(in srgb, var(--extractum-primary) 8%, var(--extractum-surface));
    border-bottom: 1px solid color-mix(in srgb, var(--extractum-primary) 24%, transparent);
    font: 400 13px/1.4 var(--extractum-font);
    color: var(--extractum-text);
  }
```

(было: обычный поток, `padding: 6px 12px`, фон surface-2. Разметка и пропсы не меняются; счётчик в v10 — primary-цвет: добавить `.sources-bulk-bar__count { color: var(--extractum-primary); }`.)

- [ ] **Step 5: Shell — statsbar-контейнер + filterRow + gridOverlay**

В `ResearchProjectsShell.svelte`:

Импорты:

```svelte
  import SourcesFilterBar from "./SourcesFilterBar.svelte";
  import SourcesFilterRow from "./SourcesFilterRow.svelte";
```

Пропсы (добавить к текущим):

```svelte
    filterBar,
    filterRow,
    gridOverlay = "Нет источников",
```

и в типе:

```svelte
    filterBar?: ComponentProps<typeof SourcesFilterBar>;
    filterRow?: ComponentProps<typeof SourcesFilterRow>;
    gridOverlay?: string;
```

Разметка main-колонки — заменить текущий блок

```svelte
      {#if bulkBar}
        <SourcesBulkBar {...bulkBar} />
      {/if}
      <div class="research-projects-shell__grid">
        <SourcesGrid {sources} {selectedSourceIds} {onSelectedSourceIdsChange} />
      </div>
```

на:

```svelte
      {#if filterBar}
        <div class="research-projects-shell__statsbar">
          <SourcesFilterBar {...filterBar} />
          {#if bulkBar}
            <SourcesBulkBar {...bulkBar} />
          {/if}
        </div>
      {/if}
      {#if filterRow}
        <SourcesFilterRow {...filterRow} />
      {/if}
      <div class="research-projects-shell__grid">
        <SourcesGrid {sources} {selectedSourceIds} {onSelectedSourceIdsChange} overlay={gridOverlay} />
      </div>
```

В `<style>` добавить:

```css
  .research-projects-shell__statsbar {
    position: relative;
    flex-shrink: 0;
  }
```

- [ ] **Step 6: Run tests + check gate**

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/ResearchProjectsShell.test.ts src/lib/components/research-projects/SourcesGrid.test.ts src/lib/components/research-projects/SourcesBulkBar.test.ts && npm.cmd run check`
Expected: тесты PASS (bulk-бар тесты не зависят от layout-стилей); check 0 ошибок.

- [ ] **Step 7: Commit**

```bash
git add src/lib/components/research-projects/SourcesBulkBar.svelte src/lib/components/research-projects/SourcesGrid.svelte src/lib/components/research-projects/SourcesGrid.test.ts src/lib/components/research-projects/ResearchProjectsShell.svelte src/lib/components/research-projects/ResearchProjectsShell.test.ts
git commit -m "feat(research-projects): stats bar container with bulk overlay and filter row in shell"
```

---

### Task 5: Проводка страницы + ConnectFromLibrary + живая проверка

**Files:**
- Modify: `src/routes/projects/next/+page.svelte`
- Modify: `src/lib/components/research-projects/ConnectFromLibrary.svelte` (сузить тип `project`)

**Interfaces:**
- Consumes: всё из Task 1–4; API из Global Constraints.
- Produces: лист-страница.

- [ ] **Step 1: Narrow ConnectFromLibrary project prop**

Компонент использует только `project?.title`. Заменить тип пропа:

```ts
    project: { title: string } | null;
```

и удалить импорт `ResearchProjectView`, если он больше нигде в файле не нужен (проверить: `connectableSelection` и другие импорты из research-projects-model остаются).

- [ ] **Step 2: Page — состояние и derived**

В `src/routes/projects/next/+page.svelte`:

Импорты добавить:

```svelte
  import ConnectFromLibrary from "$lib/components/research-projects/ConnectFromLibrary.svelte";
  import { listLibraryCatalog } from "$lib/api/library-sources";
  import { addProjectSources } from "$lib/api/projects"; // добавить в существующий список импорта
  import {
    buildLibrarySourcesView,
    connectableSelection,
    projectViewId,
  } from "$lib/ui/research-projects-model";
  import {
    buildSourceFilterChips,
    countActiveSourceFilters,
    emptySourceFilters,
    filterProjectSources,
    removeSourceFilterChip,
    type SourceFilters,
  } from "$lib/ui/research-projects-source-filters";
  import type { LibraryCatalogRecord } from "$lib/types/library-sources";
```

Состояние (после `editorProjectId`):

```ts
  let filters = $state<SourceFilters>(emptySourceFilters());
  let filtersOpen = $state(false);
  let connectOpen = $state(false);
  let libraryCatalogRecords = $state<LibraryCatalogRecord[]>([]);
  let selectedLibrarySourceIds = $state<Set<string>>(new Set());
```

Derived (после `editorProject`):

```ts
  let visibleSources = $derived(filterProjectSources(sources, filters));
  let filterChips = $derived(buildSourceFilterChips(filters));
  let filtersActive = $derived(countActiveSourceFilters(filters) > 0);
  let gridOverlay = $derived(
    filtersActive && visibleSources.length === 0 ? "Под условия ничего не подходит" : "Нет источников",
  );
  let librarySources = $derived(
    buildLibrarySourcesView(
      libraryCatalogRecords,
      sources,
      selectedProjectId !== null ? projectViewId(selectedProjectId) : null,
    ),
  );
```

В `selectProject` добавить сброс фильтров:

```ts
    filters = emptySourceFilters();
    filtersOpen = false;
```

- [ ] **Step 3: Page — connect-обработчики**

После `deleteProjectById`:

```ts
  async function openConnectSources() {
    connectOpen = true;
    if (libraryCatalogRecords.length === 0) {
      try {
        const catalog = await listLibraryCatalog();
        libraryCatalogRecords = catalog.sources;
      } catch (error) {
        railState = { ...railState, status: `Не удалось загрузить библиотеку (${String(error)})` };
      }
    }
  }

  async function connectSelectedLibrarySources() {
    if (selectedProjectId === null) return;
    const sourceIds = connectableSelection(librarySources, selectedLibrarySourceIds).map(
      (source) => source.sourceId,
    );
    if (sourceIds.length === 0) return;
    railState = { ...railState, saving: true, status: "" };
    try {
      await addProjectSources({ projectId: selectedProjectId, sourceIds });
      selectedLibrarySourceIds = new Set();
      connectOpen = false;
      sources = await listProjectSources(selectedProjectId);
      await workflow.reload();
    } catch (error) {
      railState = { ...railState, status: `Не удалось подключить источники (${String(error)})` };
    } finally {
      railState = { ...railState, saving: false };
    }
  }
```

- [ ] **Step 4: Page — прокинуть в shell + отрисовать диалог**

В `<ResearchProjectsShell`:
- `{sources}` заменить на `sources={visibleSources}`;
- добавить (рядом с `bulkBar`):

```svelte
    gridOverlay={gridOverlay}
    filterBar={selectedProject
      ? {
          filtersOpen,
          onToggleFilters: () => (filtersOpen = !filtersOpen),
          chips: filterChips,
          onRemoveChip: (key) => (filters = removeSourceFilterChip(filters, key)),
          filtersActive,
          onClearAll: () => (filters = emptySourceFilters()),
          shownCount: visibleSources.length,
          totalCount: sources.length,
          onAddSource: () => void openConnectSources(),
        }
      : undefined}
    filterRow={selectedProject && filtersOpen
      ? {
          filters,
          onChange: (next) => (filters = next),
        }
      : undefined}
```

После `<ProjectEditorDialog ... />` добавить:

```svelte
  <ConnectFromLibrary
    open={connectOpen}
    project={selectedProject ? { title: selectedProject.name } : null}
    {librarySources}
    selectedSourceIds={selectedLibrarySourceIds}
    saving={railState.saving}
    status={railState.status}
    onOpenChange={(open) => (connectOpen = open)}
    onSelectedSourceIdsChange={(ids) => (selectedLibrarySourceIds = new Set(ids))}
    onConnectSelectedSources={connectSelectedLibrarySources}
  />
```

Важно: bulk-бар теперь overlay — существующий `bulkBar`-бэг не меняется.

- [ ] **Step 5: Gates**

Run: `node scripts/run-vitest.mjs run src/lib/research-projects-import-boundary.test.ts && npm.cmd run check`
Expected: boundary PASS; check 0 ошибок.

- [ ] **Step 6: Live verification in Tauri**

Приложение на MCP-мосту (порт 9223), HMR подхватит. На `/projects/next`, выбрав проект с источниками:

1. Stats-бар: «Фильтры», «N из M» (N=M без фильтров), «+ Добавить источник» справа.
2. «Фильтры» раскрывает строку фильтров; контролы стоят под колонками таблицы (визуально сверить с гридом).
3. Поиск: ввод сужает список, «N из M» меняется, чип «Источник: …» появляется; «✕» на чипе убирает фильтр.
4. Тип/статус: поповеры с чекбоксами и точками; выбор фильтрует, бейдж на «Фильтры» показывает число активных.
5. Материалы от/до и даты с/по — фильтруют; «Сбросить» очищает всё.
6. Мусорный запрос → грид показывает «Под условия ничего не подходит».
7. Выделить строки чекбоксами → bulk-бар НАКРЫВАЕТ stats-бар (overlay, высота не прыгает); «Снять выделение» возвращает stats-бар.
8. «Добавить источник» → ConnectFromLibrary открывается, список библиотеки загружен, выбрать подключаемый источник → Connect → появился в таблице, счётчики рейла обновились.

Скриншоты: строка фильтров раскрыта с чипами; bulk-overlay; ConnectFromLibrary.

- [ ] **Step 7: Commit**

```bash
git add src/routes/projects/next/+page.svelte src/lib/components/research-projects/ConnectFromLibrary.svelte
git commit -m "feat(research-projects): wire source filters and connect-from-library on /projects/next"
```

---

## Self-Review Notes

- **Spec coverage:** чистый модуль + правила фильтрации (T1), stats-бар с бейджем/чипами/Сбросить/N из M/Добавить (T2), строка фильтров с 5 контролами и grid-колонками (T3), bulk-overlay + statsbar-контейнер + gridOverlay + SourcesGrid.overlay (T4), состояние на странице + сброс при смене проекта + скрытые выбранные строки остаются (bulkBar-бэг не меняется) + ConnectFromLibrary (T5), живая проверка (T5 Step 6).
- **Type consistency:** `SourceFilters`/`SourceFilterChip` сквозные T1→T2/T3/T5; `onChange(filters: SourceFilters)` T3→T5; `filterBar`/`filterRow`/`gridOverlay` T4→T5; `librarySources: LibrarySourceView[]` строится `buildLibrarySourcesView` и потребляется `connectableSelection` + ConnectFromLibrary.
- **Риски:** порядок вызовов в тесте чипов (`chips.map(key)`) зависит от порядка в `buildSourceFilterChips` — реализация построена в том же порядке; `ExtractumPopoverContent class` — есть fallback-замечание в T3.
