# Панель проектов v10 (/projects/next) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Довести левую панель проектов на `/projects/next` до паритета с v10: шапка с действиями, поиск, секции, сворачиваемый архив, компактный вид, hover-меню строк, создание/редактирование/удаление проектов.

**Architecture:** Новый презентационный `ProjectRailPanel.svelte` владеет UI-состоянием панели (поиск/компакт/архив/диалог удаления) и заменяет `ProjectRailSections` в shell (prop-бэг `railPanel`). `ProjectRow` расширяется вариантами и контекстным меню (shadcn DropdownMenu через новые реэкспорты extractum-ui). Страница остаётся тонкой: API-вызовы + reload.

**Tech Stack:** Svelte 5 runes, shadcn-svelte (bits-ui DropdownMenu/Dialog), `@testing-library/svelte` + jsdom, Tauri MCP для живой проверки.

**Source spec:** `docs/superpowers/specs/2026-07-03-research-projects-rail-panel-design.md`

## Global Constraints

- **Import boundary** (`src/lib/research-projects-import-boundary.test.ts`): файлы в `src/lib/components/research-projects/` и `src/routes/projects/` НЕ содержат `@svar-ui/`, `bits-ui`, `$lib/components/ui/` (даже в комментариях) — только через `$lib/components/extractum-ui`.
- **UI-тексты русские.** Disabled-пункты «Синхронизировать»/«Экспорт» — `title="Скоро"`.
- **Глобальный CSS-гочтча:** в `base.css` есть правило `button:not([data-slot="button"]) { background: var(--extractum-primary); color: #fff; ... }` (специфичность 0,1,1). Любая нейтральная кнопка в новых компонентах ДОЛЖНА перебивать его scoped-стилем с бОльшей специфичностью (класс на кнопке + scoped `<style>` даёт 0,2,0 — достаточно). Это касается и триггеров DropdownMenu (`data-slot="dropdown-menu-trigger"` ≠ `button`).
- **API (verbatim):** `createProject(input: ProjectEditorInput)`, `updateProject(input: UpdateProjectInput)` где `UpdateProjectInput = ProjectEditorInput & { projectId: number }`, `ProjectEditorInput = { name: string; description: string | null }`, `deleteProject(projectId: number)` — все из `$lib/api/projects`. Pin/archive — существующие `workflow.setPinned(projectId, pinned)` / `workflow.setArchived(projectId, archived)` (сами делают reload).
- **Тестовый раннер:** `node scripts/run-vitest.mjs run <files>`; полный гейт `npm.cmd run check` (baseline: 0 ошибок, 2 warning в `ProjectsShell.svelte` — не наши).
- **jsdom:** bits-ui Popover/Dialog рендерятся; DropdownMenu — вероятно да (floating-ui как Popover); если контент меню не монтируется — fallback на `?raw`-ассерты (конвенция проекта), интеракция проверяется вживую в Tauri.
- **Ошибки** страничных операций → `railState.status` в формате `Не удалось … (${String(error)})`; `railState.saving` на время операции.
- **Коммит на задачу; push не делать.**

---

### Task 1: Чистые функции фильтрации рейла

**Files:**
- Modify: `src/lib/ui/research-projects-rail.ts` (добавить в конец)
- Test: `src/lib/ui/research-projects-rail.test.ts` (существует — добавить describe)

**Interfaces:**
- Consumes: `ProjectRailRow`, `ProjectRailSections` (определены в том же файле).
- Produces:
  - `projectRailRowMatches(row: ProjectRailRow, query: string): boolean`
  - `filterProjectRail(sections: ProjectRailSections, query: string): ProjectRailSections`

- [ ] **Step 1: Write the failing tests**

Добавить в конец `src/lib/ui/research-projects-rail.test.ts`:

```ts
describe("projectRailRowMatches / filterProjectRail", () => {
  const row = (over: Partial<ProjectRailRow> = {}): ProjectRailRow => ({
    id: 1,
    name: "Беларусь",
    status: "ready",
    statusLabel: "готов",
    sourceCountLabel: "3 источника",
    meta: "3 источника · готов",
    pinned: false,
    archived: false,
    ...over,
  });

  it("matches by name case-insensitively", () => {
    expect(projectRailRowMatches(row({ name: "Финтех-мониторинг" }), "финтех")).toBe(true);
    expect(projectRailRowMatches(row({ name: "Финтех" }), "зиг")).toBe(false);
  });

  it("matches by meta text", () => {
    expect(projectRailRowMatches(row({ meta: "6 источников · идёт анализ" }), "анализ")).toBe(true);
  });

  it("blank query matches everything", () => {
    expect(projectRailRowMatches(row(), "")).toBe(true);
    expect(projectRailRowMatches(row(), "   ")).toBe(true);
  });

  it("filterProjectRail filters each section and keeps empty query intact", () => {
    const sections = {
      pinned: [row({ id: 1, name: "Alpha" })],
      normal: [row({ id: 2, name: "Beta" }), row({ id: 3, name: "Gamma" })],
      archived: [row({ id: 4, name: "Beta-архив" })],
    };
    const out = filterProjectRail(sections, "beta");
    expect(out.pinned).toHaveLength(0);
    expect(out.normal.map((r) => r.id)).toEqual([2]);
    expect(out.archived.map((r) => r.id)).toEqual([4]);
    expect(filterProjectRail(sections, "")).toEqual(sections);
  });
});
```

В импорт файла добавить `projectRailRowMatches, filterProjectRail` и тип `ProjectRailRow` из `./research-projects-rail` (проверить существующую строку импорта и дополнить её).

- [ ] **Step 2: Run test to verify it fails**

Run: `node scripts/run-vitest.mjs run src/lib/ui/research-projects-rail.test.ts`
Expected: FAIL — `projectRailRowMatches is not exported / not defined`.

- [ ] **Step 3: Write minimal implementation**

Добавить в конец `src/lib/ui/research-projects-rail.ts`:

```ts
export function projectRailRowMatches(row: ProjectRailRow, query: string): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return true;
  return row.name.toLowerCase().includes(q) || row.meta.toLowerCase().includes(q);
}

export function filterProjectRail(
  sections: ProjectRailSections,
  query: string,
): ProjectRailSections {
  if (!query.trim()) return sections;
  return {
    pinned: sections.pinned.filter((row) => projectRailRowMatches(row, query)),
    normal: sections.normal.filter((row) => projectRailRowMatches(row, query)),
    archived: sections.archived.filter((row) => projectRailRowMatches(row, query)),
  };
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `node scripts/run-vitest.mjs run src/lib/ui/research-projects-rail.test.ts`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/lib/ui/research-projects-rail.ts src/lib/ui/research-projects-rail.test.ts
git commit -m "feat(research-projects): add rail search filtering helpers"
```

---

### Task 2: ProjectRow v10 — варианты, компакт, hover-своп, контекстное меню

**Files:**
- Modify: `src/lib/components/extractum-ui/index.ts` (реэкспорты DropdownMenu)
- Rewrite: `src/lib/components/research-projects/ProjectRow.svelte`
- Test: `src/lib/components/research-projects/ProjectRow.test.ts` (расширить)

**Interfaces:**
- Consumes: `ProjectRailRow`; новые реэкспорты `ExtractumDropdownMenu`, `ExtractumDropdownMenuTrigger`, `ExtractumDropdownMenuContent`, `ExtractumDropdownMenuItem`, `ExtractumDropdownMenuSeparator`.
- Produces: `ProjectRow` с пропсами
  `{ row: ProjectRailRow; variant?: "active" | "normal" | "archived"; compact?: boolean; onSelect?: (id: number) => void; onEdit?: (id: number) => void; onTogglePin?: (id: number, pinned: boolean) => void; onToggleArchive?: (id: number, archived: boolean) => void; onRequestDelete?: (id: number, name: string) => void }`.
  Доступные имена: кнопка меню `title="Действия"`, пункты «Редактировать», «Закрепить»/«Открепить», «Синхронизировать» (disabled), «Экспорт» (disabled), «В архив»/«Из архива», «Удалить».

- [ ] **Step 1: Добавить реэкспорты DropdownMenu в extractum-ui**

В `src/lib/components/extractum-ui/index.ts` после блока Command добавить:

```ts
export {
  DropdownMenu as ExtractumDropdownMenu,
  DropdownMenuTrigger as ExtractumDropdownMenuTrigger,
  DropdownMenuContent as ExtractumDropdownMenuContent,
  DropdownMenuItem as ExtractumDropdownMenuItem,
  DropdownMenuSeparator as ExtractumDropdownMenuSeparator,
} from "$lib/components/ui/dropdown-menu/index.js";
```

- [ ] **Step 2: Write the failing tests**

Дополнить `src/lib/components/research-projects/ProjectRow.test.ts` (существующие тесты сохранить; тест «calls onSelect…» переписать — строка станет div, кликаем по имени):

Заменить тест `calls onSelect with the project id when clicked` на:

```ts
  it("calls onSelect with the project id when clicked", async () => {
    const onSelect = vi.fn();
    const row = buildProjectRailRow(summary({ id: 42, name: "Pick" }), NOW);

    render(ProjectRow, { props: { row, onSelect } });
    await fireEvent.click(screen.getByText("Pick"));

    expect(onSelect).toHaveBeenCalledWith(42);
  });
```

Добавить новые тесты:

```ts
  it("marks the active variant with a data attribute and shows the accent bar", () => {
    const row = buildProjectRailRow(summary({ name: "Act" }), NOW);
    const { container } = render(ProjectRow, { props: { row, variant: "active" } });
    const root = container.querySelector(".project-row");
    expect(root?.getAttribute("data-variant")).toBe("active");
    expect(container.querySelector(".project-row__active-bar")).toBeTruthy();
  });

  it("hides the meta line and sets a title in compact mode", () => {
    const row = buildProjectRailRow(summary({ name: "Cmp", source_count: 3 }), NOW);
    const { container } = render(ProjectRow, { props: { row, compact: true } });
    expect(screen.queryByText("3 источника · готов")).toBeNull();
    expect(container.querySelector(".project-row")?.getAttribute("title")).toBe(
      "Cmp — 3 источника · готов",
    );
  });

  it("renders the actions trigger for the context menu", () => {
    const row = buildProjectRailRow(summary(), NOW);
    render(ProjectRow, { props: { row } });
    expect(screen.getByTitle("Действия")).toBeTruthy();
  });

  it("opens the menu and forwards edit / pin / delete-request actions", async () => {
    const onEdit = vi.fn();
    const onTogglePin = vi.fn();
    const onRequestDelete = vi.fn();
    const row = buildProjectRailRow(summary({ id: 7, name: "Menu", pinned: false }), NOW);
    render(ProjectRow, { props: { row, onEdit, onTogglePin, onRequestDelete } });

    await fireEvent.click(screen.getByTitle("Действия"));
    await fireEvent.click(await screen.findByText("Редактировать"));
    expect(onEdit).toHaveBeenCalledWith(7);

    await fireEvent.click(screen.getByTitle("Действия"));
    await fireEvent.click(await screen.findByText("Закрепить"));
    expect(onTogglePin).toHaveBeenCalledWith(7, true);

    await fireEvent.click(screen.getByTitle("Действия"));
    await fireEvent.click(await screen.findByText("Удалить"));
    expect(onRequestDelete).toHaveBeenCalledWith(7, "Menu");
  });

  it("shows only unarchive + delete for the archived variant", async () => {
    const row = buildProjectRailRow(summary({ archived: true }), NOW);
    render(ProjectRow, { props: { row, variant: "archived" } });
    await fireEvent.click(screen.getByTitle("Действия"));
    expect(await screen.findByText("Из архива")).toBeTruthy();
    expect(screen.queryByText("Редактировать")).toBeNull();
    expect(screen.queryByText("Закрепить")).toBeNull();
  });
```

> **Contingency:** если bits-ui DropdownMenu не монтирует контент в jsdom (пункты не находятся по `findByText`), два интеракционных теста заменить `?raw`-ассертами (`import rawSource from "./ProjectRow.svelte?raw"`): наличие `<ExtractumDropdownMenu`, `onRequestDelete`, `"Из архива"`, `disabled` у «Синхронизировать»/«Экспорт». Интеракцию проверить вживую в Task 5.

- [ ] **Step 3: Run tests to verify new ones fail**

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/ProjectRow.test.ts`
Expected: FAIL — нет `data-variant`, нет `Действия` и т.д.

- [ ] **Step 4: Rewrite ProjectRow.svelte**

Полная замена `src/lib/components/research-projects/ProjectRow.svelte`:

```svelte
<script lang="ts">
  import {
    ExtractumDropdownMenu,
    ExtractumDropdownMenuTrigger,
    ExtractumDropdownMenuContent,
    ExtractumDropdownMenuItem,
    ExtractumDropdownMenuSeparator,
  } from "$lib/components/extractum-ui";
  import type { ProjectRailRow } from "$lib/ui/research-projects-rail";

  let {
    row,
    variant = "normal",
    compact = false,
    onSelect,
    onEdit,
    onTogglePin,
    onToggleArchive,
    onRequestDelete,
  }: {
    row: ProjectRailRow;
    variant?: "active" | "normal" | "archived";
    compact?: boolean;
    onSelect?: (id: number) => void;
    onEdit?: (id: number) => void;
    onTogglePin?: (id: number, pinned: boolean) => void;
    onToggleArchive?: (id: number, archived: boolean) => void;
    onRequestDelete?: (id: number, name: string) => void;
  } = $props();

  let menuOpen = $state(false);

  function openMenu(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    menuOpen = true;
  }
</script>

<div
  class="project-row"
  data-variant={variant}
  data-status={row.status}
  role="button"
  tabindex="0"
  title={compact ? `${row.name} — ${row.meta}` : undefined}
  onclick={() => onSelect?.(row.id)}
  onkeydown={(e) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onSelect?.(row.id);
    }
  }}
  oncontextmenu={openMenu}
>
  {#if variant === "active"}
    <span class="project-row__active-bar"></span>
  {/if}
  <span class="project-row__dot" data-testid="project-row-status-dot" data-status={row.status}
  ></span>
  <span class="project-row__body">
    <span class="project-row__name">{row.name}</span>
    {#if !compact}
      <span class="project-row__meta">{row.meta}</span>
    {/if}
  </span>
  <span class="project-row__actions" data-pinned={row.pinned || variant === "active"}>
    {#if row.pinned || variant === "active"}
      <span class="project-row__pin" title="Закреплён" aria-label="Закреплён">
        <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor">
          <path
            d="M9.5 1.5l5 5-2 .6-2.3 2.3.5 3.2-1.8-1.8-3.3 3.3-.5.5.5-3.8L1.9 8.7l3.2.5L7.4 6.9 8 4.9z"
          />
        </svg>
      </span>
    {/if}
    <ExtractumDropdownMenu bind:open={menuOpen}>
      <ExtractumDropdownMenuTrigger
        class="project-row__menu-trigger"
        title="Действия"
        onclick={(e: MouseEvent) => e.stopPropagation()}
      >
        ⋯
      </ExtractumDropdownMenuTrigger>
      <ExtractumDropdownMenuContent class="project-row-menu" align="end">
        {#if variant === "archived"}
          <ExtractumDropdownMenuItem onclick={() => onToggleArchive?.(row.id, false)}>
            Из архива
          </ExtractumDropdownMenuItem>
        {:else}
          <ExtractumDropdownMenuItem onclick={() => onEdit?.(row.id)}>
            Редактировать
          </ExtractumDropdownMenuItem>
          <ExtractumDropdownMenuItem onclick={() => onTogglePin?.(row.id, !row.pinned)}>
            {row.pinned ? "Открепить" : "Закрепить"}
          </ExtractumDropdownMenuItem>
          <ExtractumDropdownMenuItem disabled title="Скоро">Синхронизировать</ExtractumDropdownMenuItem>
          <ExtractumDropdownMenuItem disabled title="Скоро">Экспорт</ExtractumDropdownMenuItem>
          <ExtractumDropdownMenuItem onclick={() => onToggleArchive?.(row.id, true)}>
            В архив
          </ExtractumDropdownMenuItem>
        {/if}
        <ExtractumDropdownMenuSeparator />
        <ExtractumDropdownMenuItem
          class="project-row-menu__danger"
          onclick={() => onRequestDelete?.(row.id, row.name)}
        >
          Удалить
        </ExtractumDropdownMenuItem>
      </ExtractumDropdownMenuContent>
    </ExtractumDropdownMenu>
  </span>
</div>

<style>
  .project-row {
    position: relative;
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 7px 10px;
    border-radius: 6px;
    background: transparent;
    text-align: left;
    cursor: pointer;
  }

  .project-row[data-variant="archived"] {
    padding: 6px 10px;
  }

  .project-row:hover {
    background: var(--extractum-surface-subtle);
  }

  .project-row[data-variant="active"],
  .project-row[data-variant="active"]:hover {
    background: color-mix(in srgb, var(--extractum-primary) 10%, transparent);
    cursor: default;
  }

  .project-row__active-bar {
    position: absolute;
    left: 3px;
    top: 50%;
    transform: translateY(-50%);
    width: 3px;
    height: 22px;
    border-radius: 3px;
    background: var(--extractum-primary);
  }

  .project-row__dot {
    flex-shrink: 0;
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--extractum-muted);
  }

  .project-row__dot[data-status="ready"] {
    background: var(--extractum-success);
  }

  .project-row__dot[data-status="running"] {
    background: var(--extractum-primary);
  }

  .project-row__dot[data-status="needs_attention"] {
    background: var(--extractum-danger);
  }

  .project-row__dot[data-status="empty"] {
    background: var(--extractum-muted-2);
  }

  .project-row__body {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }

  .project-row__name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font: 400 13px/1.3 var(--extractum-font);
    color: var(--extractum-text);
  }

  .project-row[data-variant="active"] .project-row__name {
    font-weight: 600;
    color: var(--extractum-primary);
  }

  .project-row[data-variant="archived"] .project-row__name {
    color: var(--extractum-muted);
  }

  .project-row__meta {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font: 400 11px/1.3 var(--extractum-font);
    color: var(--extractum-muted);
  }

  .project-row[data-variant="archived"] .project-row__meta {
    color: var(--extractum-muted-2);
  }

  .project-row__actions {
    position: relative;
    flex-shrink: 0;
    width: 22px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .project-row__pin {
    position: absolute;
    display: inline-flex;
    color: var(--extractum-muted-2);
    transition: opacity 0.12s ease;
  }

  .project-row[data-variant="active"] .project-row__pin {
    color: var(--extractum-primary);
  }

  /* hover-своп: пин прячется, «⋯» появляется */
  .project-row:hover .project-row__pin {
    opacity: 0;
  }

  .project-row__actions :global(.project-row__menu-trigger) {
    position: absolute;
    width: 22px;
    height: 22px;
    border: none;
    border-radius: 5px;
    background: transparent;
    color: var(--extractum-muted);
    cursor: pointer;
    font-size: 14px;
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    transition: opacity 0.12s ease;
    padding: 0;
  }

  .project-row:hover .project-row__actions :global(.project-row__menu-trigger),
  .project-row__actions :global(.project-row__menu-trigger[data-state="open"]) {
    opacity: 1;
  }

  .project-row__actions :global(.project-row__menu-trigger:hover) {
    background: var(--extractum-surface-subtle);
  }

  :global(.project-row-menu__danger) {
    color: var(--extractum-danger) !important;
  }
</style>
```

Замечания для исполнителя:
- Триггер получает глобальное правило `button:not([data-slot="button"])` — scoped-override выше обязателен (background/color/padding заданы явно).
- `findByText` в тестах — потому что контент меню монтируется асинхронно (портал).
- Если svelte-check ругается на `title` у `ExtractumDropdownMenuItem` (rest-props) — допустимо обернуть текст пункта в `<span title="Скоро">`.

- [ ] **Step 5: Run tests to verify they pass**

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/ProjectRow.test.ts`
Expected: PASS (все, включая 4 старых). Если интеракционные упали из-за jsdom — применить contingency из Step 2 и перегнать.

- [ ] **Step 6: Full check gate**

Run: `npm.cmd run check`
Expected: 0 ошибок (2 baseline-warnings в ProjectsShell.svelte допустимы).

- [ ] **Step 7: Commit**

```bash
git add src/lib/components/extractum-ui/index.ts src/lib/components/research-projects/ProjectRow.svelte src/lib/components/research-projects/ProjectRow.test.ts
git commit -m "feat(research-projects): v10 ProjectRow with variants and context menu"
```

---

### Task 3: ProjectRailPanel — шапка, поиск, секции, архив, диалог удаления

**Files:**
- Create: `src/lib/components/research-projects/ProjectRailPanel.svelte`
- Test: `src/lib/components/research-projects/ProjectRailPanel.test.ts`
- Delete: `src/lib/components/research-projects/ProjectRailSections.svelte`, `src/lib/components/research-projects/ProjectRailSections.test.ts` — В Task 4 (shell ещё импортирует секции; удаление здесь сломало бы check).

**Interfaces:**
- Consumes: `ProjectRow` (Task 2, все пропсы), `filterProjectRail`/`projectRailRowMatches`/`groupProjectRail`/`buildProjectRailRow`, `ExtractumDialog`, `ExtractumButton`, `ExtractumDropdownMenu*`.
- Produces: `ProjectRailPanel` с пропсами
  `{ summaries: ProjectSummary[]; selectedProjectId: number | null; now: number; onSelect?: (id: number) => void; onCreate?: () => void; onEdit?: (id: number) => void; onTogglePin?: (id: number, pinned: boolean) => void; onToggleArchive?: (id: number, archived: boolean) => void; onDelete?: (id: number) => void }`.
  Доступные имена: кнопки шапки `title="Компактный вид"|"Комфортный вид"`, `title="Создать проект"`, `title="Синхронизировать"` (disabled), `title="Действия с проектом"`; поле `placeholder="Поиск проектов"`; тогл «Архив»; диалог «Удалить проект» с «Отмена»/«Да, удалить».

- [ ] **Step 1: Write the failing tests**

`src/lib/components/research-projects/ProjectRailPanel.test.ts`:

```ts
// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import ProjectRailPanel from "./ProjectRailPanel.svelte";
import type { ProjectSummary } from "$lib/types/projects";

afterEach(cleanup);

const NOW = 1_000_000_000;

function summary(overrides: Partial<ProjectSummary> = {}): ProjectSummary {
  return {
    id: 1,
    name: "Alpha",
    description: null,
    source_count: 3,
    material_count: 100,
    status: "ready",
    last_run_at: null,
    pinned: false,
    archived: false,
    updated_at: 1,
    ...overrides,
  };
}

const baseProps = {
  selectedProjectId: null as number | null,
  now: NOW,
};

describe("ProjectRailPanel", () => {
  it("renders header actions: compact toggle, create, disabled sync", () => {
    render(ProjectRailPanel, { props: { ...baseProps, summaries: [summary()] } });
    expect(screen.getByTitle("Компактный вид")).toBeTruthy();
    expect(screen.getByTitle("Создать проект")).toBeTruthy();
    const sync = screen.getByTitle("Скоро") as HTMLButtonElement;
    expect(sync.disabled).toBe(true);
  });

  it("hides the header project menu without a selected project and shows it with one", () => {
    const { unmount } = render(ProjectRailPanel, {
      props: { ...baseProps, summaries: [summary()] },
    });
    expect(screen.queryByTitle("Действия с проектом")).toBeNull();
    unmount();

    render(ProjectRailPanel, {
      props: { ...baseProps, summaries: [summary({ id: 5 })], selectedProjectId: 5 },
    });
    expect(screen.getByTitle("Действия с проектом")).toBeTruthy();
  });

  it("filters projects by search and shows an empty state", async () => {
    render(ProjectRailPanel, {
      props: {
        ...baseProps,
        summaries: [summary({ id: 1, name: "Беларусь" }), summary({ id: 2, name: "Финтех" })],
      },
    });
    const input = screen.getByPlaceholderText("Поиск проектов");
    await fireEvent.input(input, { target: { value: "фин" } });
    expect(screen.queryByText("Беларусь")).toBeNull();
    expect(screen.getByText("Финтех")).toBeTruthy();

    await fireEvent.input(input, { target: { value: "нет-такого" } });
    expect(screen.getByText("Проекты не найдены")).toBeTruthy();
  });

  it("keeps the archive collapsed by default with a full count and expands on click", async () => {
    render(ProjectRailPanel, {
      props: {
        ...baseProps,
        summaries: [
          summary({ id: 1, name: "Живой" }),
          summary({ id: 2, name: "Старый аудит", archived: true }),
          summary({ id: 3, name: "Q3 ресёрч", archived: true }),
        ],
      },
    });
    expect(screen.queryByText("Старый аудит")).toBeNull();
    expect(screen.getByText("2")).toBeTruthy();

    await fireEvent.click(screen.getByText("Архив"));
    expect(screen.getByText("Старый аудит")).toBeTruthy();
    expect(screen.getByText("Q3 ресёрч")).toBeTruthy();
  });

  it("compact mode hides meta lines", async () => {
    render(ProjectRailPanel, {
      props: { ...baseProps, summaries: [summary({ name: "Cmp", source_count: 3 })] },
    });
    expect(screen.getByText("3 источника · готов")).toBeTruthy();
    await fireEvent.click(screen.getByTitle("Компактный вид"));
    expect(screen.queryByText("3 источника · готов")).toBeNull();
  });

  it("renders the selected project first as the active row", () => {
    render(ProjectRailPanel, {
      props: {
        ...baseProps,
        summaries: [summary({ id: 1, name: "First" }), summary({ id: 2, name: "Chosen" })],
        selectedProjectId: 2,
      },
    });
    const rows = document.querySelectorAll(".project-row");
    expect(rows[0]?.getAttribute("data-variant")).toBe("active");
    expect(rows[0]?.textContent).toContain("Chosen");
  });

  it("confirms deletion through a dialog before calling onDelete", async () => {
    const onDelete = vi.fn();
    render(ProjectRailPanel, {
      props: {
        ...baseProps,
        summaries: [summary({ id: 9, name: "Del" })],
        selectedProjectId: 9,
        onDelete,
      },
    });
    await fireEvent.click(screen.getByTitle("Действия с проектом"));
    await fireEvent.click(await screen.findByText("Удалить"));
    expect(onDelete).not.toHaveBeenCalled();

    await fireEvent.click(await screen.findByText("Да, удалить"));
    expect(onDelete).toHaveBeenCalledWith(9);
  });

  it("forwards create clicks", async () => {
    const onCreate = vi.fn();
    render(ProjectRailPanel, {
      props: { ...baseProps, summaries: [], onCreate },
    });
    await fireEvent.click(screen.getByTitle("Создать проект"));
    expect(onCreate).toHaveBeenCalledOnce();
  });
});
```

Примечание: у disabled sync-кнопки `title="Скоро"` (а не «Синхронизировать») — так тест однозначен и пользователь видит причину.

- [ ] **Step 2: Run test to verify it fails**

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/ProjectRailPanel.test.ts`
Expected: FAIL — модуль не найден.

- [ ] **Step 3: Implement ProjectRailPanel.svelte**

`src/lib/components/research-projects/ProjectRailPanel.svelte`:

```svelte
<script lang="ts">
  import {
    ExtractumButton,
    ExtractumDialog,
    ExtractumDropdownMenu,
    ExtractumDropdownMenuTrigger,
    ExtractumDropdownMenuContent,
    ExtractumDropdownMenuItem,
    ExtractumDropdownMenuSeparator,
  } from "$lib/components/extractum-ui";
  import ProjectRow from "./ProjectRow.svelte";
  import {
    buildProjectRailRow,
    filterProjectRail,
    groupProjectRail,
    projectRailRowMatches,
  } from "$lib/ui/research-projects-rail";
  import type { ProjectSummary } from "$lib/types/projects";

  let {
    summaries,
    selectedProjectId,
    now,
    onSelect,
    onCreate,
    onEdit,
    onTogglePin,
    onToggleArchive,
    onDelete,
  }: {
    summaries: ProjectSummary[];
    selectedProjectId: number | null;
    now: number;
    onSelect?: (id: number) => void;
    onCreate?: () => void;
    onEdit?: (id: number) => void;
    onTogglePin?: (id: number, pinned: boolean) => void;
    onToggleArchive?: (id: number, archived: boolean) => void;
    onDelete?: (id: number) => void;
  } = $props();

  let query = $state("");
  let compact = $state(false);
  let archiveOpen = $state(false);
  let headerMenuOpen = $state(false);
  let confirmOpen = $state(false);
  let pendingDelete = $state<{ id: number; name: string } | null>(null);

  let selected = $derived(summaries.find((s) => s.id === selectedProjectId) ?? null);
  let activeRow = $derived(selected ? buildProjectRailRow(selected, now) : null);
  let sections = $derived(
    groupProjectRail(
      summaries.filter((s) => s.id !== selectedProjectId),
      now,
    ),
  );
  let filtered = $derived(filterProjectRail(sections, query));
  let activeVisible = $derived(activeRow !== null && projectRailRowMatches(activeRow, query));
  let archiveCount = $derived(sections.archived.length);
  let noProjects = $derived(
    query.trim() !== "" &&
      !activeVisible &&
      filtered.pinned.length === 0 &&
      filtered.normal.length === 0 &&
      filtered.archived.length === 0,
  );

  const rowActions = {
    get onSelect() {
      return onSelect;
    },
    get onEdit() {
      return onEdit;
    },
    get onTogglePin() {
      return onTogglePin;
    },
    get onToggleArchive() {
      return onToggleArchive;
    },
  };

  function requestDelete(id: number, name: string) {
    pendingDelete = { id, name };
    confirmOpen = true;
  }

  function confirmDelete() {
    const target = pendingDelete;
    confirmOpen = false;
    pendingDelete = null;
    if (target) onDelete?.(target.id);
  }
</script>

<div class="rail-panel">
  <div class="rail-panel__header">
    <span class="rail-panel__title">Проекты</span>
    <div class="rail-panel__actions">
      <button
        type="button"
        class="rail-panel__icon-btn"
        title={compact ? "Комфортный вид" : "Компактный вид"}
        onclick={() => (compact = !compact)}
      >
        <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M2.5 4h11M2.5 8h11M2.5 12h11" />
        </svg>
      </button>
      <button
        type="button"
        class="rail-panel__icon-btn"
        title="Создать проект"
        onclick={() => onCreate?.()}
      >
        +
      </button>
      <button type="button" class="rail-panel__icon-btn" title="Скоро" disabled>
        <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M2 8a6 6 0 019.5-4.8M14 8a6 6 0 01-9.5 4.8M11.5 2v3h-3M4.5 14v-3h3" />
        </svg>
      </button>
      {#if selected}
        <ExtractumDropdownMenu bind:open={headerMenuOpen}>
          <ExtractumDropdownMenuTrigger
            class="rail-panel__icon-btn rail-panel__menu-trigger"
            title="Действия с проектом"
          >
            ⋯
          </ExtractumDropdownMenuTrigger>
          <ExtractumDropdownMenuContent align="end">
            <ExtractumDropdownMenuItem onclick={() => selected && onEdit?.(selected.id)}>
              Редактировать
            </ExtractumDropdownMenuItem>
            <ExtractumDropdownMenuItem disabled>
              <span title="Скоро">Экспорт</span>
            </ExtractumDropdownMenuItem>
            <ExtractumDropdownMenuSeparator />
            <ExtractumDropdownMenuItem
              class="project-row-menu__danger"
              onclick={() => selected && requestDelete(selected.id, selected.name)}
            >
              Удалить
            </ExtractumDropdownMenuItem>
          </ExtractumDropdownMenuContent>
        </ExtractumDropdownMenu>
      {/if}
    </div>
  </div>

  <div class="rail-panel__search">
    <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
      <circle cx="7" cy="7" r="4.5" />
      <path d="M10.5 10.5L14 14" />
    </svg>
    <input bind:value={query} placeholder="Поиск проектов" aria-label="Поиск проектов" />
    {#if query.length > 0}
      <button
        type="button"
        class="rail-panel__clear"
        title="Очистить"
        onclick={() => (query = "")}
      >
        ×
      </button>
    {/if}
  </div>

  <div class="rail-panel__list">
    {#if activeVisible || filtered.pinned.length > 0}
      <div class="rail-panel__section-header">
        <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor">
          <path
            d="M9.5 1.5l5 5-2 .6-2.3 2.3.5 3.2-1.8-1.8-3.3 3.3-.5.5.5-3.8L1.9 8.7l3.2.5L7.4 6.9 8 4.9z"
          />
        </svg>
        <span>Закреплённые</span>
      </div>
    {/if}
    {#if activeRow && activeVisible}
      <ProjectRow
        row={activeRow}
        variant="active"
        {compact}
        {...rowActions}
        onRequestDelete={requestDelete}
      />
    {/if}
    {#each filtered.pinned as row (row.id)}
      <ProjectRow {row} {compact} {...rowActions} onRequestDelete={requestDelete} />
    {/each}

    {#if filtered.normal.length > 0}
      <div class="rail-panel__section-header rail-panel__section-header--plain">Проекты</div>
    {/if}
    {#each filtered.normal as row (row.id)}
      <ProjectRow {row} {compact} {...rowActions} onRequestDelete={requestDelete} />
    {/each}

    {#if noProjects}
      <div class="rail-panel__empty">Проекты не найдены</div>
    {/if}

    {#if filtered.archived.length > 0}
      <button
        type="button"
        class="rail-panel__archive-toggle"
        onclick={() => (archiveOpen = !archiveOpen)}
      >
        <svg
          width="13"
          height="13"
          viewBox="0 0 16 16"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          style:transform={archiveOpen ? "rotate(90deg)" : "rotate(0deg)"}
        >
          <path d="M6 3.5L10.5 8 6 12.5" />
        </svg>
        <span class="rail-panel__archive-label">Архив</span>
        <span class="rail-panel__archive-count">{archiveCount}</span>
      </button>
      {#if archiveOpen}
        {#each filtered.archived as row (row.id)}
          <ProjectRow
            {row}
            variant="archived"
            {compact}
            {...rowActions}
            onRequestDelete={requestDelete}
          />
        {/each}
      {/if}
    {/if}
  </div>
</div>

<ExtractumDialog bind:open={confirmOpen} title="Удалить проект">
  <div class="rail-panel__confirm">
    <p>Удалить проект «{pendingDelete?.name ?? ""}»? Действие необратимо.</p>
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
  .rail-panel {
    display: flex;
    flex-direction: column;
    min-height: 0;
    height: 100%;
  }

  .rail-panel__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 12px 8px;
  }

  .rail-panel__title {
    font: 700 11px/1 var(--extractum-font);
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--extractum-muted);
  }

  .rail-panel__actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  /* Перебивает глобальное button:not([data-slot="button"]) */
  .rail-panel__actions .rail-panel__icon-btn,
  .rail-panel__actions :global(.rail-panel__menu-trigger) {
    width: 24px;
    height: 24px;
    padding: 0;
    border: 1px solid var(--extractum-border);
    border-radius: 6px;
    background: var(--extractum-surface);
    color: var(--extractum-muted);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 15px;
    line-height: 1;
    cursor: pointer;
  }

  .rail-panel__actions .rail-panel__icon-btn:hover:not(:disabled),
  .rail-panel__actions :global(.rail-panel__menu-trigger:hover) {
    background: var(--extractum-surface-subtle);
  }

  .rail-panel__actions .rail-panel__icon-btn:disabled {
    opacity: 0.45;
    cursor: default;
  }

  .rail-panel__search {
    display: flex;
    align-items: center;
    gap: 7px;
    height: 30px;
    margin: 0 10px 8px;
    border: 1px solid var(--extractum-border);
    border-radius: 6px;
    background: var(--extractum-surface);
    padding: 0 9px;
    color: var(--extractum-muted-2);
  }

  .rail-panel__search input {
    flex: 1;
    min-width: 0;
    border: none;
    outline: none;
    background: transparent;
    font: 400 12.5px/1 var(--extractum-font);
    color: var(--extractum-text);
  }

  .rail-panel__search .rail-panel__clear {
    border: none;
    background: transparent;
    padding: 0;
    color: var(--extractum-muted-2);
    font-size: 15px;
    line-height: 1;
    cursor: pointer;
  }

  .rail-panel__list {
    flex: 1;
    overflow: auto;
    padding: 2px 8px 8px;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .rail-panel__section-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 8px 4px;
    font: 700 10px/1 var(--extractum-font);
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--extractum-muted-2);
  }

  .rail-panel__section-header--plain {
    padding-top: 12px;
  }

  .rail-panel__empty {
    padding: 22px 12px;
    text-align: center;
    font: 400 12px/1.5 var(--extractum-font);
    color: var(--extractum-muted-2);
  }

  .rail-panel__list .rail-panel__archive-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 9px 10px;
    margin-top: 6px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--extractum-muted-2);
    cursor: pointer;
    text-align: left;
  }

  .rail-panel__list .rail-panel__archive-toggle:hover {
    background: var(--extractum-surface-subtle);
  }

  .rail-panel__archive-toggle svg {
    transition: transform 0.15s ease;
  }

  .rail-panel__archive-label {
    flex: 1;
    font: 600 11px/1 var(--extractum-font);
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .rail-panel__archive-count {
    font: 600 11px/1 var(--extractum-font);
  }

  .rail-panel__confirm {
    display: flex;
    min-width: min(420px, calc(100vw - 96px));
    flex-direction: column;
    gap: 16px;
    padding: 16px;
  }

  .rail-panel__confirm footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
```

Замечания для исполнителя:
- `rowActions` с геттерами — чтобы прокинуть колбэки без потери реактивности при спреде; если svelte-check не примет спред, передать колбэки по одному (`onSelect={onSelect}` и т.д.) — это допустимая замена.
- Архивный toggle — `<button>` со scoped-override (см. Global Constraints).

- [ ] **Step 4: Run test to verify it passes**

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/ProjectRailPanel.test.ts src/lib/components/research-projects/ProjectRow.test.ts`
Expected: PASS. (Contingency для меню — как в Task 2.)

- [ ] **Step 5: Full check gate**

Run: `npm.cmd run check`
Expected: 0 ошибок.

- [ ] **Step 6: Commit**

```bash
git add src/lib/components/research-projects/ProjectRailPanel.svelte src/lib/components/research-projects/ProjectRailPanel.test.ts
git commit -m "feat(research-projects): add ProjectRailPanel with search, archive and header actions"
```

---

### Task 4: Shell — prop-бэг `railPanel` вместо ProjectRailSections

**Files:**
- Modify: `src/lib/components/research-projects/ResearchProjectsShell.svelte`
- Modify: `src/lib/components/research-projects/ResearchProjectsShell.test.ts`
- Delete: `src/lib/components/research-projects/ProjectRailSections.svelte`, `src/lib/components/research-projects/ProjectRailSections.test.ts`

**Interfaces:**
- Consumes: `ProjectRailPanel` (Task 3).
- Produces: shell-пропсы: `summaries`, `now`, `onSelectProject` УДАЛЯЮТСЯ; добавляется `railPanel: ComponentProps<typeof ProjectRailPanel>` (обязательный); `selectedProjectId` остаётся (управляет main-областью).

- [ ] **Step 1: Update shell tests (failing)**

В `ResearchProjectsShell.test.ts`:
- Тест `renders the project rail from summaries` заменить на:

```ts
  it("renders the project rail panel from the railPanel bag", () => {
    render(ResearchProjectsShell, {
      props: {
        railPanel: { summaries: [summary({ name: "Беларусь" })], selectedProjectId: null, now: NOW },
        selectedProjectId: null,
      },
    });

    expect(screen.getByText("Беларусь")).toBeTruthy();
    expect(screen.getByPlaceholderText("Поиск проектов")).toBeTruthy();
  });
```

- Тест `forwards project selection` заменить на:

```ts
  it("forwards project selection through the railPanel bag", async () => {
    const onSelect = vi.fn();
    render(ResearchProjectsShell, {
      props: {
        railPanel: {
          summaries: [summary({ id: 7, name: "Pick me" })],
          selectedProjectId: null,
          now: NOW,
          onSelect,
        },
        selectedProjectId: null,
      },
    });

    await fireEvent.click(screen.getByText("Pick me"));
    expect(onSelect).toHaveBeenCalledWith(7);
  });
```

- В тесте инспектора props `{ summaries: [], selectedProjectId: null, now: NOW, inspector: inspectorBag }` заменить на `{ railPanel: { summaries: [], selectedProjectId: null, now: NOW }, selectedProjectId: null, inspector: inspectorBag }`.
- Добавить `?raw`-ассерт:

```ts
  it("renders the rail panel in the aside", () => {
    expect(shellSource).toContain("<ProjectRailPanel");
    expect(shellSource).toContain("{...railPanel}");
    expect(shellSource).not.toContain("ProjectRailSections");
  });
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/ResearchProjectsShell.test.ts`
Expected: FAIL (нет пропа railPanel).

- [ ] **Step 3: Update shell**

В `ResearchProjectsShell.svelte`:
- Импорт: `import ProjectRailSections from "./ProjectRailSections.svelte";` → `import ProjectRailPanel from "./ProjectRailPanel.svelte";`
- Пропсы: удалить `summaries`, `now`, `onSelectProject`; добавить `railPanel`:

```svelte
  let {
    railPanel,
    selectedProjectId,
    sources = [],
    selectedSourceIds = [],
    toolbar,
    runDock,
    inspector,
    bulkBar,
    onSelectedSourceIdsChange,
  }: {
    railPanel: ComponentProps<typeof ProjectRailPanel>;
    selectedProjectId: number | null;
    sources?: ProjectSourceRecord[];
    selectedSourceIds?: string[];
    toolbar?: ComponentProps<typeof ProjectToolbar>;
    runDock?: ComponentProps<typeof RunDock>;
    inspector?: ComponentProps<typeof Inspector>;
    bulkBar?: ComponentProps<typeof SourcesBulkBar>;
    onSelectedSourceIdsChange?: (ids: string[]) => void;
  } = $props();
```

(`ProjectSummary` больше не нужен в импортах типов — убрать, оставить `ProjectSourceRecord`.)

- Разметка aside:

```svelte
  <aside class="research-projects-shell__rail">
    <ProjectRailPanel {...railPanel} />
  </aside>
```

- [ ] **Step 4: Delete ProjectRailSections**

```bash
git rm src/lib/components/research-projects/ProjectRailSections.svelte src/lib/components/research-projects/ProjectRailSections.test.ts
```

- [ ] **Step 5: Fix the page compile break (minimal bridge)**

`src/routes/projects/next/+page.svelte` перестанет компилироваться (пропы shell изменились). Минимальный мост в этой задаче (полная проводка — Task 5): заменить в вызове `<ResearchProjectsShell` строки

```svelte
    summaries={railState.summaries}
    {selectedProjectId}
    {now}
    ...
    onSelectProject={selectProject}
```

на

```svelte
    railPanel={{
      summaries: railState.summaries,
      selectedProjectId,
      now,
      onSelect: selectProject,
    }}
    {selectedProjectId}
```

(`onSelectProject={selectProject}` удалить.)

- [ ] **Step 6: Run tests + check gate**

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/ResearchProjectsShell.test.ts && npm.cmd run check`
Expected: тесты PASS; check 0 ошибок.

- [ ] **Step 7: Commit**

```bash
git add -A src/lib/components/research-projects/ src/routes/projects/next/+page.svelte
git commit -m "feat(research-projects): shell renders ProjectRailPanel via railPanel bag"
```

---

### Task 5: Страница — создание/редактирование/удаление + живая проверка

**Files:**
- Modify: `src/routes/projects/next/+page.svelte`
- Modify: `src/lib/components/research-projects/ProjectEditorDialog.svelte` (сузить тип пропа)

**Interfaces:**
- Consumes: `railPanel`-бэг (Task 4), `ProjectRailPanel` колбэки (Task 3), API из Global Constraints, `workflow.setPinned`/`workflow.setArchived`.
- Produces: лист-страница; ничего.

- [ ] **Step 1: Narrow ProjectEditorDialog prop type**

В `ProjectEditorDialog.svelte` заменить

```ts
  import type { ResearchProjectView } from "$lib/ui/research-projects-model";
```
на удаление импорта, и тип пропа:

```ts
    project?: { title: string; description: string | null } | null;
```

(Старая страница передаёт `ResearchProjectView` — структурно совместимо; `npm.cmd run check` это подтвердит.)

- [ ] **Step 2: Page wiring**

В `src/routes/projects/next/+page.svelte`:

1. Импорты: добавить `createProject, deleteProject, updateProject` в список из `$lib/api/projects`; добавить `import ProjectEditorDialog from "$lib/components/research-projects/ProjectEditorDialog.svelte";`
2. Состояние (после `selectedSourceIds`):

```ts
  let editorOpen = $state(false);
  let editorProjectId = $state<number | null>(null);
```

3. Derived (после `selectedProject`):

```ts
  let editorProject = $derived.by(() => {
    if (editorProjectId === null) return null;
    const summary = railState.summaries.find((s) => s.id === editorProjectId);
    return summary ? { title: summary.name, description: summary.description } : null;
  });
```

4. Обработчики (после `deleteSelectedSources`):

```ts
  function openCreateProject() {
    editorProjectId = null;
    editorOpen = true;
  }

  function openEditProject(id: number) {
    editorProjectId = id;
    editorOpen = true;
  }

  async function submitProjectEditor(input: { name: string; description: string | null }) {
    railState = { ...railState, saving: true, status: "" };
    try {
      if (editorProjectId === null) {
        await createProject(input);
      } else {
        await updateProject({ projectId: editorProjectId, ...input });
      }
      await workflow.reload();
    } catch (error) {
      railState = { ...railState, status: `Не удалось сохранить проект (${String(error)})` };
    } finally {
      railState = { ...railState, saving: false };
    }
  }

  async function deleteProjectById(id: number) {
    railState = { ...railState, saving: true, status: "" };
    try {
      await deleteProject(id);
      if (selectedProjectId === id) {
        selectedProjectId = null;
        sources = [];
        selectedSourceIds = [];
      }
      await workflow.reload();
    } catch (error) {
      railState = { ...railState, status: `Не удалось удалить проект (${String(error)})` };
    } finally {
      railState = { ...railState, saving: false };
    }
  }
```

5. `railPanel`-бэг расширить:

```svelte
    railPanel={{
      summaries: railState.summaries,
      selectedProjectId,
      now,
      onSelect: selectProject,
      onCreate: openCreateProject,
      onEdit: openEditProject,
      onTogglePin: (id, pinned) => void workflow.setPinned(id, pinned),
      onToggleArchive: (id, archived) => void workflow.setArchived(id, archived),
      onDelete: (id) => void deleteProjectById(id),
    }}
```

6. После `</ResearchProjectsShell>` (внутри `.projects-next`) добавить диалог:

```svelte
  <ProjectEditorDialog
    bind:open={editorOpen}
    project={editorProject}
    saving={railState.saving}
    error={railState.status}
    onSubmit={submitProjectEditor}
  />
```

- [ ] **Step 3: Check gate + import boundary**

Run: `node scripts/run-vitest.mjs run src/lib/research-projects-import-boundary.test.ts && npm.cmd run check`
Expected: boundary PASS; check 0 ошибок.

- [ ] **Step 4: Live verification in Tauri**

Приложение запущено (MCP-мост, порт 9223); Vite подхватит правки по HMR. Проверить на `/projects/next`:

1. Панель: шапка «ПРОЕКТЫ» + 4 кнопки, поиск, секции; выбранный проект — первым с полоской и синим именем.
2. Поиск: ввод фильтрует, «×» очищает, «Проекты не найдены» на мусорный запрос.
3. Компактный вид: тогл скрывает meta, title-подсказка на строке.
4. Hover-своп: пин ↔ «⋯»; «⋯» и правый клик открывают меню.
5. Создание: «+» → диалог → создать проект → появляется в списке (потом удалить его же через меню — заодно проверка удаления с подтверждением; диалог «Удалить проект», «Отмена» не удаляет, «Да, удалить» удаляет).
6. Пин/архив: «Закрепить» перемещает в «Закреплённые»; «В архив» — в свёрнутый «Архив N»; раскрытие архива; «Из архива» возвращает.
7. Disabled: «Синхронизировать»/«Экспорт» в меню и sync-кнопка шапки — серые, title «Скоро».

Скриншоты: панель с открытым меню строки; архив раскрыт; диалог удаления.

- [ ] **Step 5: Commit**

```bash
git add src/routes/projects/next/+page.svelte src/lib/components/research-projects/ProjectEditorDialog.svelte
git commit -m "feat(research-projects): wire project create/edit/delete into /projects/next rail"
```

---

## Self-Review Notes

- **Spec coverage:** шапка 4 кнопки (T3), поиск + «не найдены» (T1/T3), архив свёрнут+счётчик полный (T3), компакт (T2/T3), hover-своп + полоска (T2), меню 6 пунктов c disabled «Скоро» (T2), archived-меню 2 пункта (T2), диалог удаления в панели (T3), header-меню активного проекта (T3), create/edit через ProjectEditorDialog + delete со сбросом выбора (T5), pin/archive через workflow (T5), замена ProjectRailSections (T4), ошибки → railState.status (T5), границы импортов (T5 gate), живая проверка (T5).
- **Type consistency:** `onTogglePin(id, pinned)` — новое значение (T2 передаёт `!row.pinned`; T5 → `workflow.setPinned(id, pinned)`); `onRequestDelete(id, name)` T2 → `requestDelete` T3 → `onDelete(id)` T5; `railPanel: ComponentProps<typeof ProjectRailPanel>` сквозной.
- **Отличие от старого ProjectRow:** корень `<button>` → `<div role="button">` (вложенный триггер меню — nested buttons невалидны); тест выбора кликает по имени.
- **Риск jsdom/DropdownMenu:** contingency в T2 Step 2 и T3 Step 4; финальная истина — живая проверка T5.
