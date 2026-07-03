# Активная строка vs чекбокс-выделение — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Разделить клик по строке (активная строка → инспектор, подсветка v10) и чекбокс-выделение (bulk-бар) в таблице источников `/projects/next`.

**Architecture:** svar-паттерн «Selecting rows with checkboxes»: `select={false}` отключает клик-выделение, чекбоксы работают через `api.exec`. Активная строка — состояние страницы; обёртка `ExtractumDataGrid` получает клик-делегирование и подсветку динамическим CSS-правилом (не реактивным svar-пропсом — сортировка в безопасности).

**Tech Stack:** svar `@svar-ui/svelte-grid` (`select`, `data-id` на `.wx-row`), Svelte 5, vitest (`?raw`), Tauri MCP.

**Source spec:** `docs/superpowers/specs/2026-07-03-research-projects-active-row-design.md`

## Global Constraints

- **Урок сортировочной итерации:** любое изменение реактивных пропсов svar `<Grid>` сбрасывает sortMarks — НЕ добавлять новых реактивных пропсов гриду; подсветка активной строки только через динамический CSS вне svar.
- **`data-id` строк** в установленной версии: `":430"` (префикс `:` + наш строковый id). Снятие префикса: `raw.startsWith(":") ? raw.slice(1) : raw`; в CSS-селекторе покрывать обе формы (`:id` и `id`).
- **ConnectFromLibrary не менять**: дефолт `selectOnClick = true` сохраняет прежнее поведение (клик по строке выделяет).
- **Import boundary**: фичевые файлы — только через extractum-ui.
- **Тестовый раннер:** `node scripts/run-vitest.mjs run <files>`; гейт `npm.cmd run check` (baseline 0 ошибок, 2 warning в ProjectsShell.svelte).
- **Живая проверка:** Tauri MCP 9223, приложение запущено.
- **Коммит на задачу; push не делать.** Ветка `feat/research-projects-active-row` от main; план — первым коммитом.

---

### Task 1: ExtractumDataGrid — selectOnClick, onRowClick, activeRowId

**Files:**
- Modify: `src/lib/components/extractum-ui/DataGrid.svelte`

**Interfaces:**
- Produces: пропсы `selectOnClick?: boolean` (default `true`), `activeRowId?: string | null` (default `null`), `onRowClick?: (id: string) => void`. Клик по строке (вне `[data-action="ignore-click"]` и заголовка) → `onRowClick(id без префикса ':')`. Активная строка подсвечивается CSS-правилом по `data-grid-uid` + `data-id`.

- [ ] **Step 1: Implement (без юнит-теста — svar не рендерится в jsdom; верификация `?raw` в Task 2 и вживую в Task 3)**

В `DataGrid.svelte`:

1. Пропсы — в деструктуризацию и тип добавить:

```ts
    selectOnClick = true,
    activeRowId = null,
    onRowClick,
```

```ts
    selectOnClick?: boolean;
    activeRowId?: string | null;
    onRowClick?: (id: string) => void;
```

2. После `const GRID_SIZES = ...` добавить:

```ts
  // Unique host marker for the active-row CSS rule below.
  const gridUid = Math.random().toString(36).slice(2, 10);
```

3. После блока selection-sync `$effect` добавить клик-делегирование (listener через effect, чтобы не ловить a11y-warning на div):

```ts
  // Row click → onRowClick(id). Delegated on the host so svar's internal
  // re-renders don't detach it; checkbox zones opt out via ignore-click.
  $effect(() => {
    const element = host;
    const handler = (event: MouseEvent) => {
      if (!onRowClick) return;
      const target = event.target as HTMLElement;
      if (target.closest('[data-action="ignore-click"]')) return;
      if (target.closest(".wx-header")) return;
      const rowEl = target.closest(".wx-row") as HTMLElement | null;
      if (!rowEl || !element?.contains(rowEl)) return;
      const raw = rowEl.dataset.id ?? "";
      const id = raw.startsWith(":") ? raw.slice(1) : raw;
      if (id) onRowClick(id);
    };
    element?.addEventListener("click", handler);
    return () => element?.removeEventListener("click", handler);
  });

  // Active-row highlight as a dynamic CSS rule: survives svar re-renders
  // (sorting, data refresh) and never touches svar reactive props.
  let activeRowCss = $derived.by(() => {
    if (!activeRowId || !/^[\w:-]+$/.test(activeRowId)) return "";
    const rule =
      `background: color-mix(in srgb, var(--extractum-primary) 7%, var(--extractum-surface));` +
      ` box-shadow: inset 2px 0 0 var(--extractum-primary);`;
    const scope = `.extractum-data-grid[data-grid-uid="${gridUid}"]`;
    return (
      `<style>` +
      `${scope} .wx-row[data-id=":${activeRowId}"] .wx-cell, ` +
      `${scope} .wx-row[data-id="${activeRowId}"] .wx-cell { ${rule} }` +
      `</style>`
    );
  });
```

Замечание: правило вешается на `.wx-cell` (а не на строку) — svar красит фон ячеек, и подсветка строки перекрывалась бы фоном ячейки; проверить вживую и при необходимости упростить до `.wx-row`.

4. В разметке: host получает `data-grid-uid={gridUid}`, перед host вставить `{@html activeRowCss}`:

```svelte
{@html activeRowCss}
<div
  bind:this={host}
  data-grid-uid={gridUid}
  class={cn("extractum-svar-theme extractum-data-grid", className)}
  ...
```

5. svar Grid: `select` → `select={selectOnClick}` (было безусловное `select`).

- [ ] **Step 2: Check gate**

Run: `npm.cmd run check`
Expected: 0 ошибок.

- [ ] **Step 3: Commit** (сначала `git checkout -b feat/research-projects-active-row`; план — отдельным коммитом `docs: add active-row implementation plan`)

```bash
git add src/lib/components/extractum-ui/DataGrid.svelte
git commit -m "feat(extractum-ui): DataGrid row-click activation and active-row highlight"
```

---

### Task 2: Проводка SourcesGrid → Shell → страница

**Files:**
- Modify: `src/lib/components/research-projects/SourcesGrid.svelte`
- Modify: `src/lib/components/research-projects/ResearchProjectsShell.svelte`
- Modify: `src/routes/projects/next/+page.svelte`
- Test: `src/lib/components/research-projects/SourcesGrid.test.ts`, `src/lib/components/research-projects/ResearchProjectsShell.test.ts`

**Interfaces:**
- Consumes: пропсы DataGrid из Task 1.
- Produces: `SourcesGrid` пропсы `activeSourceId?: string | null`, `onActivateSource?: (id: string) => void`; shell — сквозные пропсы с теми же именами.

- [ ] **Step 1: Write failing `?raw` tests**

В `SourcesGrid.test.ts` добавить:

```ts
  it("separates row activation from checkbox selection (v10)", () => {
    expect(source).toContain("selectOnClick={false}");
    expect(source).toContain("activeRowId={activeSourceId}");
    expect(source).toContain("onRowClick={onActivateSource}");
  });
```

В `ResearchProjectsShell.test.ts` добавить:

```ts
  it("passes row activation through to the sources grid", () => {
    expect(shellSource).toContain("{activeSourceId}");
    expect(shellSource).toContain("{onActivateSource}");
  });
```

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/SourcesGrid.test.ts src/lib/components/research-projects/ResearchProjectsShell.test.ts`
Expected: FAIL (2 новых).

- [ ] **Step 2: SourcesGrid**

Пропсы:

```ts
    overlay = "Нет источников",
    activeSourceId = null,
    onActivateSource,
  }: {
    ...
    overlay?: string;
    activeSourceId?: string | null;
    onActivateSource?: (id: string) => void;
```

Разметка `<ExtractumDataGrid ...>` — добавить:

```svelte
  selectOnClick={false}
  activeRowId={activeSourceId}
  onRowClick={onActivateSource}
```

- [ ] **Step 3: Shell**

Пропсы `activeSourceId = null,` и `onActivateSource,` (+ типы `activeSourceId?: string | null; onActivateSource?: (id: string) => void;`); в `<SourcesGrid ...>` добавить `{activeSourceId} {onActivateSource}`.

- [ ] **Step 4: Page**

1. Состояние: `let activeSourceId = $state<string | null>(null);` (рядом с `selectedSourceIds`).
2. `selectedSourceRow` — заменить чтение:

```ts
  let selectedSourceRow = $derived.by(() => {
    if (!activeSourceId) return null;
    const record = sources.find((source) => String(source.source_id) === activeSourceId);
    return record ? buildSourceRow(record) : null;
  });
```

3. В `selectProject` добавить `activeSourceId = null;`.
4. В `<ResearchProjectsShell` добавить:

```svelte
    {activeSourceId}
    onActivateSource={(id) => (activeSourceId = id)}
```

- [ ] **Step 5: Tests + gates**

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/SourcesGrid.test.ts src/lib/components/research-projects/ResearchProjectsShell.test.ts src/lib/research-projects-import-boundary.test.ts && npm.cmd run check`
Expected: PASS; 0 ошибок.

- [ ] **Step 6: Commit**

```bash
git add src/lib/components/research-projects/SourcesGrid.svelte src/lib/components/research-projects/SourcesGrid.test.ts src/lib/components/research-projects/ResearchProjectsShell.svelte src/lib/components/research-projects/ResearchProjectsShell.test.ts src/routes/projects/next/+page.svelte
git commit -m "feat(research-projects): route row activation to the inspector, checkboxes to bulk"
```

---

### Task 3: Живая проверка

**Files:** нет изменений (кроме возможной правки CSS-правила подсветки по факту).

- [ ] **Step 1:** `/projects/next`, Project 2:
  1. Клик по строке → инспектор с этим источником; строка подсвечена (фон-тинт + левая полоска 2px); bulk-бар НЕ появился; чекбокс НЕ поставился.
  2. Клик по другой строке → инспектор и подсветка переехали.
  3. Чекбокс первой строки → bulk-бар «Выбрано: 1»; инспектор НЕ изменился.
  4. Select-all → «Выбрано: N»; активная строка сохраняет подсветку.
  5. Сортировка по «Материалы» → клик по строке → сортировка и маркер живы; подсветка на правильной строке после пересортировки.
  6. «Добавить источник» → в ConnectFromLibrary клик по строке ПО-ПРЕЖНЕМУ выделяет (дефолт selectOnClick).
  7. Смена проекта → активность сброшена (инспектор пуст до клика).
- [ ] **Step 2:** Скриншот: активная строка + отдельно выделенные чекбоксы.
- [ ] **Step 3:** Если подсветка не видна из-за фона ячеек — скорректировать правило (см. замечание Task 1) и повторить; закоммитить правку в Task 1-файл с сообщением `fix(extractum-ui): adjust active-row highlight rule`.

---

## Self-Review Notes

- **Spec coverage:** select={selectOnClick} + делегирование + динамический CSS + uid (T1), проводка grid/shell/page + инспектор от activeSourceId + сброс при смене проекта (T2), вся живая матрица включая ConnectFromLibrary и сортировку (T3). Активность при фильтрации сохраняется автоматически (state не сбрасывается фильтром); повторный клик по активной строке просто переустанавливает тот же id.
- **Type consistency:** `activeRowId: string | null` (DataGrid) ← `activeSourceId` (SourcesGrid/Shell/page); `onRowClick(id: string)` ← `onActivateSource`.
- **Риск:** специфичность подсветки vs svar-фоны ячеек — рулим правилом на `.wx-cell` с фолбэком-упрощением (T1 замечание, T3 Step 3).
