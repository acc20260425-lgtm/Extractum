# Табы разделов проекта (/projects/next) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ряд из 6 табов разделов проекта на `/projects/next`; работающий раздел — «Источники», остальные — заглушки «Раздел … в разработке».

**Architecture:** Презентационный `ProjectTabs.svelte` (подчёркивание по v10, без content-слотов); shell получает бэг `tabs` (под тулбаром) и проп `sectionPlaceholder` (заглушка вместо statsbar/фильтров/грида); активная секция — `$state` страницы, сброс на «Источники» при смене проекта.

**Tech Stack:** Svelte 5 runes, vitest + jsdom, Tauri MCP для живой проверки.

**Source spec:** `docs/superpowers/specs/2026-07-03-research-projects-tabs-design.md`

## Global Constraints

- **Import boundary**: фичевые файлы без `@svar-ui`/`bits-ui`/`$lib/components/ui/`; ProjectTabs — чистая разметка.
- **Ярлыки (verbatim):** Обзор · Источники · Факты · Отчёты · Запуски · Промпты; заглушка — `Раздел «{label}» в разработке`.
- **CSS-гочтча:** глобальное `button:not([data-slot="button"]) { background: var(--extractum-primary); color:#fff; }` — кнопки табов требуют scoped-override.
- **v10-стиль активного таба:** текст `--extractum-primary` 600 + `box-shadow: inset 0 -2px 0 var(--extractum-primary)`; ряд 40px с нижней границей `--extractum-border`.
- **Тестовый раннер:** `node scripts/run-vitest.mjs run <files>`; гейт `npm.cmd run check` (baseline 0 ошибок, 2 warning в ProjectsShell.svelte).
- **Живая проверка:** Tauri MCP-мост 9223, приложение запущено; фиксируется поведение сортировки после возврата на «Источники» (не гарантируется — грид размонтируется).
- **Коммит на задачу; push не делать.** Ветка `feat/research-projects-tabs` от main; план — первым коммитом.

---

### Task 1: ProjectTabs.svelte

**Files:**
- Create: `src/lib/components/research-projects/ProjectTabs.svelte`
- Test: `src/lib/components/research-projects/ProjectTabs.test.ts`

**Interfaces:**
- Produces: default export `ProjectTabs` с пропсами `{ active: ProjectSectionId; onSelect?: (id: ProjectSectionId) => void }`; именованные экспорты из module-script:

```ts
export type ProjectSectionId =
  | "overview" | "sources" | "evidence" | "reports" | "runs" | "prompts";
export const PROJECT_SECTIONS: { id: ProjectSectionId; label: string }[];
```

- [ ] **Step 1: Write the failing tests**

`src/lib/components/research-projects/ProjectTabs.test.ts`:

```ts
// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import ProjectTabs, { PROJECT_SECTIONS } from "./ProjectTabs.svelte";

afterEach(cleanup);

describe("ProjectTabs", () => {
  it("renders all six sections with Russian labels", () => {
    render(ProjectTabs, { props: { active: "sources" } });
    for (const label of ["Обзор", "Источники", "Факты", "Отчёты", "Запуски", "Промпты"]) {
      expect(screen.getByRole("tab", { name: label })).toBeTruthy();
    }
    expect(PROJECT_SECTIONS).toHaveLength(6);
  });

  it("marks only the active tab as selected", () => {
    render(ProjectTabs, { props: { active: "sources" } });
    expect(screen.getByRole("tab", { name: "Источники" }).getAttribute("aria-selected")).toBe(
      "true",
    );
    expect(screen.getByRole("tab", { name: "Обзор" }).getAttribute("aria-selected")).toBe("false");
    expect(screen.getAllByRole("tab", { selected: true })).toHaveLength(1);
  });

  it("forwards tab selection", async () => {
    const onSelect = vi.fn();
    render(ProjectTabs, { props: { active: "sources", onSelect } });
    await fireEvent.click(screen.getByRole("tab", { name: "Отчёты" }));
    expect(onSelect).toHaveBeenCalledWith("reports");
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/ProjectTabs.test.ts`
Expected: FAIL — module not found.

- [ ] **Step 3: Implement**

`src/lib/components/research-projects/ProjectTabs.svelte`:

```svelte
<script module lang="ts">
  export type ProjectSectionId =
    | "overview"
    | "sources"
    | "evidence"
    | "reports"
    | "runs"
    | "prompts";

  export const PROJECT_SECTIONS: { id: ProjectSectionId; label: string }[] = [
    { id: "overview", label: "Обзор" },
    { id: "sources", label: "Источники" },
    { id: "evidence", label: "Факты" },
    { id: "reports", label: "Отчёты" },
    { id: "runs", label: "Запуски" },
    { id: "prompts", label: "Промпты" },
  ];
</script>

<script lang="ts">
  let {
    active,
    onSelect,
  }: {
    active: ProjectSectionId;
    onSelect?: (id: ProjectSectionId) => void;
  } = $props();
</script>

<div class="project-tabs" role="tablist" aria-label="Разделы проекта">
  {#each PROJECT_SECTIONS as section (section.id)}
    <button
      type="button"
      role="tab"
      class="project-tabs__tab"
      aria-selected={active === section.id}
      onclick={() => onSelect?.(section.id)}
    >
      {section.label}
    </button>
  {/each}
</div>

<style>
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

  /* scoped override глобального button-правила */
  .project-tabs .project-tabs__tab {
    display: flex;
    align-items: center;
    padding: 0;
    border: none;
    background: transparent;
    font: 500 13px/1 var(--extractum-font);
    color: var(--extractum-muted);
    cursor: pointer;
  }

  .project-tabs .project-tabs__tab[aria-selected="true"] {
    font-weight: 600;
    color: var(--extractum-primary);
    box-shadow: inset 0 -2px 0 var(--extractum-primary);
  }

  .project-tabs .project-tabs__tab:hover:not([aria-selected="true"]) {
    color: var(--extractum-text);
  }
</style>
```

- [ ] **Step 4: Run test to verify it passes**

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/ProjectTabs.test.ts`
Expected: PASS (3 теста).

- [ ] **Step 5: Full check gate + commit** (сначала `git checkout -b feat/research-projects-tabs`; план — отдельным коммитом `docs: add project tabs implementation plan`)

Run: `npm.cmd run check` — 0 ошибок.

```bash
git add src/lib/components/research-projects/ProjectTabs.svelte src/lib/components/research-projects/ProjectTabs.test.ts
git commit -m "feat(research-projects): add ProjectTabs section switcher"
```

---

### Task 2: Shell — бэг tabs + sectionPlaceholder

**Files:**
- Modify: `src/lib/components/research-projects/ResearchProjectsShell.svelte`
- Test: `src/lib/components/research-projects/ResearchProjectsShell.test.ts`

**Interfaces:**
- Consumes: `ProjectTabs` (Task 1).
- Produces: shell-пропсы `tabs?: ComponentProps<typeof ProjectTabs>`, `sectionPlaceholder?: string`. Порядок в main-колонке: Toolbar → Tabs → (placeholder ЛИБО statsbar+filterRow+grid) → RunDock.

- [ ] **Step 1: Write the failing tests**

Добавить в `ResearchProjectsShell.test.ts`:

```ts
  it("renders the tabs row under the toolbar and the section placeholder instead of the grid", () => {
    expect(shellSource).toContain("<ProjectTabs");
    expect(shellSource).toContain("{...tabs}");
    const toolbarIndex = shellSource.indexOf("<ProjectToolbar");
    const tabsIndex = shellSource.indexOf("<ProjectTabs");
    const statsIndex = shellSource.indexOf('class="research-projects-shell__statsbar"');
    expect(tabsIndex).toBeGreaterThan(toolbarIndex);
    expect(tabsIndex).toBeLessThan(statsIndex);
    expect(shellSource).toContain("sectionPlaceholder");
    expect(shellSource).toContain("research-projects-shell__section-placeholder");
  });

  it("shows the placeholder text when sectionPlaceholder is provided", () => {
    render(ResearchProjectsShell, {
      props: {
        railPanel: { summaries: [], selectedProjectId: null, now: NOW },
        selectedProjectId: 1,
        sectionPlaceholder: "Раздел «Обзор» в разработке",
      },
    });
    expect(screen.getByText("Раздел «Обзор» в разработке")).toBeTruthy();
  });
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/ResearchProjectsShell.test.ts`
Expected: FAIL.

- [ ] **Step 3: Implement shell changes**

Импорт: после `ProjectToolbar` добавить

```svelte
  import ProjectTabs from "./ProjectTabs.svelte";
```

Пропсы: в деструктуризацию добавить `tabs,` и `sectionPlaceholder = "",` рядом с `bulkBar`; в тип:

```svelte
    tabs?: ComponentProps<typeof ProjectTabs>;
    sectionPlaceholder?: string;
```

Разметка main-колонки — после `{#if toolbar}...{/if}` вставить:

```svelte
      {#if tabs}
        <ProjectTabs {...tabs} />
      {/if}
```

и обернуть секцию источников:

```svelte
      {#if sectionPlaceholder}
        <div class="research-projects-shell__section-placeholder">{sectionPlaceholder}</div>
      {:else}
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
      {/if}
```

Стиль:

```css
  .research-projects-shell__section-placeholder {
    flex: 1;
    display: grid;
    place-items: center;
    font: 400 13px/1.4 var(--extractum-font);
    color: var(--extractum-muted-2);
  }
```

- [ ] **Step 4: Run tests + check gate**

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/ResearchProjectsShell.test.ts && npm.cmd run check`
Expected: PASS; 0 ошибок.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/research-projects/ResearchProjectsShell.svelte src/lib/components/research-projects/ResearchProjectsShell.test.ts
git commit -m "feat(research-projects): shell renders section tabs and placeholder"
```

---

### Task 3: Страница — activeSection + живая проверка

**Files:**
- Modify: `src/routes/projects/next/+page.svelte`

**Interfaces:**
- Consumes: `ProjectTabs` типы (`ProjectSectionId`, `PROJECT_SECTIONS`), shell-пропсы Task 2.

- [ ] **Step 1: Page wiring**

Импорт:

```svelte
  import { PROJECT_SECTIONS, type ProjectSectionId } from "$lib/components/research-projects/ProjectTabs.svelte";
```

Состояние (рядом с `filtersOpen`):

```ts
  let activeSection = $state<ProjectSectionId>("sources");
```

Derived (после `gridOverlay`):

```ts
  let sectionPlaceholder = $derived(
    activeSection === "sources"
      ? ""
      : `Раздел «${PROJECT_SECTIONS.find((s) => s.id === activeSection)?.label ?? ""}» в разработке`,
  );
```

В `selectProject` добавить `activeSection = "sources";`.

В `<ResearchProjectsShell`:
- добавить `sectionPlaceholder={selectedProject ? sectionPlaceholder : ""}`;
- добавить бэг

```svelte
    tabs={selectedProject
      ? { active: activeSection, onSelect: (id) => (activeSection = id) }
      : undefined}
```

- условия бэгов источников дополнить секцией: `filterBar={selectedProject && activeSection === "sources" ? {...} : undefined}`, `filterRow={selectedProject && activeSection === "sources" && filtersOpen ? {...} : undefined}`, `bulkBar={activeSection === "sources" && selectedSourceIds.length > 0 ? {...} : undefined}`.

- [ ] **Step 2: Gates**

Run: `node scripts/run-vitest.mjs run src/lib/research-projects-import-boundary.test.ts && npm.cmd run check`
Expected: PASS; 0 ошибок.

- [ ] **Step 3: Live verification in Tauri**

На `/projects/next`, выбрав проект:

1. Ряд табов под тулбаром, «Источники» активен (синее подчёркивание), таблица работает.
2. Клик «Обзор» → заглушка «Раздел «Обзор» в разработке»; statsbar/фильтры/грид скрыты; тулбар/RunDock/Inspector на местах. Аналогично остальные 4 таба (правильные названия).
3. На «Источниках» задать фильтр + выделить строку → уйти на «Отчёты» → вернуться: фильтр и выделение целы; поведение сортировки зафиксировать (не гарантируется).
4. Смена проекта с не-Sources таба → возврат на «Источники».

Скриншот: таб-ряд с активным «Источники» и заглушка другого раздела.

- [ ] **Step 4: Commit**

```bash
git add src/routes/projects/next/+page.svelte
git commit -m "feat(research-projects): wire section tabs on /projects/next"
```

---

## Self-Review Notes

- **Spec coverage:** компонент с 6 русскими ярлыками и v10-стилем (T1), tabs-бэг под тулбаром + sectionPlaceholder вместо секции источников, RunDock/Inspector нетронуты (T2), state страницы + сброс при смене проекта + условия бэгов + заглушка с label (T3), живая проверка вкл. сохранность фильтров/выделения и фиксацию сортировки (T3 Step 3).
- **Type consistency:** `ProjectSectionId` сквозной; `onSelect(id: ProjectSectionId)`; `sectionPlaceholder: string` (пустая строка = секция источников видна).
- **Примечание:** module-script экспорты (`PROJECT_SECTIONS`, тип) импортируются как именованные из `.svelte` — паттерн уже используется (ComboSelect: `type ComboOption`).
