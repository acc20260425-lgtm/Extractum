# Сортировка колонок таблицы источников — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Кликабельная сортировка заголовков таблицы источников на `/projects/next` (asc/desc + мультисортировка по Ctrl) средствами svar.

**Architecture:** Встроенная сортировка svar: параметр `sort` (true либо компаратор значений ячеек) в `sourceGridColumns()`. Три именованных компаратора в чистом модуле. Условный Task 3 — рефактор `GridSelectCell` на реактивное выделение, если живая проверка покажет сброс сортировки при выделении строк.

**Tech Stack:** svar `@svar-ui/svelte-grid` (column.sort), Svelte 5, vitest, Tauri MCP.

**Source spec:** `docs/superpowers/specs/2026-07-03-research-projects-sources-sorting-design.md`

## Global Constraints

- **svar-контракт:** column-level `sort: true | (a, b) => 1|-1|0`, где `a`/`b` — ЗНАЧЕНИЯ ячеек колонки; компаратор заменяет встроенную сортировку и обновляет sortMarks. Мультисортировка Ctrl+клик — по умолчанию, не отключать.
- **Import boundary**: без `@svar-ui`/`bits-ui`/`$lib/components/ui/` в фичевых файлах (`research-projects/`, `routes/projects/`); `research-projects-source-row.ts` — модуль в `src/lib/ui/`, ему разрешён только тип `ExtractumDataGridColumn` из extractum-ui (уже импортирует).
- **Тип колонок:** `ExtractumDataGridColumn = IColumnConfig & {...}` — `sort` уже входит в svar `IColumnConfig`; если check споткнётся, расширить тип в `src/lib/components/extractum-ui/data-grid-date-format.ts`.
- **`GridSelectCell` общий** для SourcesGrid и ConnectFromLibrary — при условном рефакторе сохранить `connectable`/`disabledReason`-поведение.
- **Тестовый раннер:** `node scripts/run-vitest.mjs run <files>`; гейт `npm.cmd run check` (baseline 0 ошибок, 2 warning в ProjectsShell.svelte).
- **Живая проверка:** Tauri MCP-мост, порт 9223, приложение запущено; svar в jsdom не рендерится.
- **Критерий из спеки:** сброс сортировки при изменении ВЫДЕЛЕНИЯ недопустим (лечится Task 3); сброс при изменении набора данных фильтром — допустим, зафиксировать фактическое поведение.
- **Коммит на задачу; push не делать.** Ветка `feat/research-projects-sources-sorting` от main; план — первым коммитом.

---

### Task 1: Компараторы + sort в конфигурации колонок

**Files:**
- Modify: `src/lib/ui/research-projects-source-row.ts`
- Test: `src/lib/ui/research-projects-source-row.test.ts` (добавить describe)

**Interfaces:**
- Produces:
  - `compareRuStrings(a: unknown, b: unknown): number` — localeCompare "ru", без регистра;
  - `compareMaterialsLabels(a: unknown, b: unknown): number` — числовое сравнение форматированных строк («4 317» → 4317);
  - `compareNullableTimestamps(a: unknown, b: unknown): number` — числа; `null`/не-число всегда последним при любом направлении НЕВОЗМОЖНО гарантировать на уровне компаратора (svar инвертирует знак при desc) — принято: `null` трактуется как `-Infinity` (внизу при asc, вверху при desc); зафиксировано в тестах;
  - `sourceGridColumns()` — колонки с `sort`.

Примечание к null-датам: «null всегда внизу» на column-компараторе svar недостижимо (desc = инверсия компаратора). Принято: null = «самая старая дата» → при asc (старые сверху) null вверху, при desc (новые сверху) null внизу; практический сценарий «новые сверху» даёт null внизу. Спека обновлена соответствующе; проверяется вживую.

- [ ] **Step 1: Write the failing tests**

Добавить в конец `src/lib/ui/research-projects-source-row.test.ts` (импорт дополнить `compareMaterialsLabels, compareNullableTimestamps, compareRuStrings`):

```ts
describe("sort comparators", () => {
  it("compareRuStrings orders Cyrillic case-insensitively", () => {
    expect(compareRuStrings("аист", "Бобр")).toBeLessThan(0);
    expect(compareRuStrings("Яблоко", "аист")).toBeGreaterThan(0);
    expect(compareRuStrings("ФИН", "фин")).toBe(0);
  });

  it("compareMaterialsLabels compares formatted numbers numerically", () => {
    expect(compareMaterialsLabels("4 317", "339")).toBeGreaterThan(0);
    expect(compareMaterialsLabels("76 070", "4 317")).toBeGreaterThan(0);
    expect(compareMaterialsLabels("10", "10")).toBe(0);
  });

  it("compareNullableTimestamps treats null as the oldest value", () => {
    expect(compareNullableTimestamps(null, 100)).toBeLessThan(0);
    expect(compareNullableTimestamps(100, null)).toBeGreaterThan(0);
    expect(compareNullableTimestamps(null, null)).toBe(0);
    expect(compareNullableTimestamps(200, 100)).toBeGreaterThan(0);
  });

  it("sourceGridColumns enables sorting on every data column", () => {
    const columns = sourceGridColumns();
    const byId = new Map(columns.map((c) => [String(c.id), c]));
    expect(byId.get("title")?.sort).toBe(compareRuStrings);
    expect(byId.get("typeLabel")?.sort).toBe(true);
    expect(byId.get("materialsLabel")?.sort).toBe(compareMaterialsLabels);
    expect(byId.get("lastSyncedAt")?.sort).toBe(compareNullableTimestamps);
    expect(byId.get("statusLabel")?.sort).toBe(true);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node scripts/run-vitest.mjs run src/lib/ui/research-projects-source-row.test.ts`
Expected: FAIL — компараторы не экспортированы.

- [ ] **Step 3: Implement**

В `src/lib/ui/research-projects-source-row.ts` перед `sourceGridColumns` добавить:

```ts
// svar column.sort receives raw CELL VALUES; comparators are exported for unit tests.
export function compareRuStrings(a: unknown, b: unknown): number {
  return String(a ?? "").localeCompare(String(b ?? ""), "ru", { sensitivity: "base" });
}

export function compareMaterialsLabels(a: unknown, b: unknown): number {
  const num = (v: unknown) => Number(String(v ?? "").replace(/\D/g, "")) || 0;
  return num(a) - num(b);
}

// null = "oldest": sinks below real dates on ascending sort (and inverts with desc,
// which svar does by negating the comparator).
export function compareNullableTimestamps(a: unknown, b: unknown): number {
  const num = (v: unknown) => (typeof v === "number" && Number.isFinite(v) ? v : -Infinity);
  const x = num(a);
  const y = num(b);
  return x < y ? -1 : x > y ? 1 : 0;
}
```

и обновить `sourceGridColumns()`:

```ts
export function sourceGridColumns(): ExtractumDataGridColumn[] {
  return [
    { id: "title", header: "Источник", width: 260, flexgrow: 1, sort: compareRuStrings },
    { id: "typeLabel", header: "Тип", width: 116, sort: true },
    { id: "materialsLabel", header: "Материалы", width: 116, sort: compareMaterialsLabels },
    {
      id: "lastSyncedAt",
      header: "Последний сбор",
      width: 150,
      dateTimeFormat: "datetime",
      sort: compareNullableTimestamps,
    },
    { id: "statusLabel", header: "Статус", width: 104, sort: true },
  ];
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `node scripts/run-vitest.mjs run src/lib/ui/research-projects-source-row.test.ts`
Expected: PASS (все, включая существующие).

- [ ] **Step 5: Full check gate**

Run: `npm.cmd run check`
Expected: 0 ошибок. Если TS отвергает `sort` в `ExtractumDataGridColumn` — расширить тип в `src/lib/components/extractum-ui/data-grid-date-format.ts`:

```ts
export type ExtractumDataGridColumn = IColumnConfig & {
  dateTimeFormat?: ExtractumDateTimeFormat | false;
  sort?: boolean | ((a: unknown, b: unknown) => number);
};
```

- [ ] **Step 6: Commit** (сначала `git checkout -b feat/research-projects-sources-sorting`; план — отдельным коммитом `docs: add sources sorting implementation plan`)

```bash
git add src/lib/ui/research-projects-source-row.ts src/lib/ui/research-projects-source-row.test.ts
git commit -m "feat(research-projects): enable column sorting on sources grid"
```

---

### Task 2: Живая проверка (включая устойчивость к выделению)

**Files:** нет изменений кода (только проверка); при провале критерия выделения — выполнить Task 3 и перепроверить.

**Interfaces:** Consumes: колонки из Task 1 (HMR подхватит).

- [ ] **Step 1: Подключиться и открыть страницу**

Tauri MCP (порт 9223) → `/projects/next` → выбрать проект «Project 2» (6 источников, разные materials/даты).

- [ ] **Step 2: Проверить одиночную сортировку**

Клик по заголовку «Материалы» → строки по возрастанию числа (339 < 4 317 < 76 070 — числами, не строками), стрелка в заголовке; повторный клик — по убыванию. Клик по «Источник» — кириллица по алфавиту без регистра. Клик по «Последний сбор» дважды (desc, «новые сверху») — null-даты внизу.

- [ ] **Step 3: Проверить мультисортировку**

Клик «Тип», затем Ctrl+клик «Материалы» → в заголовках индексы 1/2, порядок — тип, внутри типа — материалы.

- [ ] **Step 4: КРИТЕРИЙ — сортировка переживает выделение**

При активной сортировке кликнуть чекбокс строки (и select-all). Сортировка и стрелки должны сохраниться, порядок строк не должен сброситься.
- Если сохраняется → Task 3 пропустить, зафиксировать в итоге.
- Если сбрасывается → выполнить Task 3, затем повторить Steps 2–4.

- [ ] **Step 5: Зафиксировать поведение при фильтрации**

Открыть «Фильтры», ввести поиск, сузив список: отметить, сохраняется ли сортировка после изменения набора данных (допустимо любое поведение — записать фактическое в итоговый отчёт).

- [ ] **Step 6: Скриншоты**

Скриншот с отсортированной колонкой (стрелка) и с мультисортировкой (индексы).

---

### Task 3 (УСЛОВНАЯ — только если Step 4 Task 2 провалился): реактивное выделение в GridSelectCell

**Files:**
- Modify: `src/lib/components/extractum-ui/GridSelectCell.svelte`
- Modify: `src/lib/components/research-projects/SourcesGrid.svelte` (убрать подмешивание `selected`)
- Test: `src/lib/components/research-projects/SourcesGrid.test.ts` (обновить `?raw`-ассерт)

**Interfaces:**
- Produces: `GridSelectCell` читает выделение реактивно из `api.getReactiveState().selectedRows` (как `GridSelectAllCell`), `row.selected` больше не требуется; `connectable`/`disabledReason` — без изменений (ConnectFromLibrary продолжает работать).

- [ ] **Step 1: Update the `?raw` assert (failing)**

В `SourcesGrid.test.ts` заменить

```ts
    // row.selected synced from the current selection for the per-row checkbox
    expect(source).toContain("selected: selectedSourceIds.includes(row.id)");
```

на

```ts
    // per-row checkbox reads selection reactively from the grid api,
    // so rows no longer depend on selectedSourceIds (sort survives selection)
    expect(source).not.toContain("selected: selectedSourceIds.includes(row.id)");
```

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/SourcesGrid.test.ts`
Expected: FAIL (строка ещё в исходнике).

- [ ] **Step 2: GridSelectCell — реактивное выделение**

Заменить `<script>` в `GridSelectCell.svelte`:

```svelte
<script lang="ts">
  import { untrack } from "svelte";

  let { api, row } = $props<{
    api: {
      exec: (action: string, data: Record<string, unknown>) => void;
      getReactiveState: () => {
        selectedRows: { subscribe: (fn: (v: unknown[]) => void) => () => void };
      };
    };
    row: Record<string, unknown>;
  }>();

  // `api` is a stable grid handle; read its reactive store once.
  const selectedRows = untrack(() => api.getReactiveState().selectedRows);

  let rowId = $derived(String(row.id ?? ""));
  let selected = $derived(($selectedRows ?? []).some((id) => String(id) === rowId));
  let connectable = $derived(row.connectable !== false);
  let disabledReason = $derived(
    typeof row.disabledReason === "string" ? row.disabledReason : null,
  );

  function toggle(event: Event) {
    if (!connectable) return;
    const target = event.currentTarget as HTMLInputElement;
    api.exec("select-row", { id: rowId, mode: target.checked, toggle: true });
  }
</script>
```

(разметка и стили без изменений).

- [ ] **Step 3: SourcesGrid — убрать подмешивание selected**

Заменить

```ts
  let rows = $derived(
    buildSourceGridRows(sources).map((row) => ({
      ...row,
      selected: selectedSourceIds.includes(row.id),
    })),
  );
```

на

```ts
  // Rows depend only on the data: GridSelectCell reads selection reactively
  // from the grid api, so toggling checkboxes does not rebuild rows (and does
  // not reset svar sorting).
  let rows = $derived(buildSourceGridRows(sources));
```

(`selectedSourceIds` остаётся пропсом — уходит в `selectedRowIds` грида.)

- [ ] **Step 4: Tests + gates**

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/SourcesGrid.test.ts src/lib/research-projects-import-boundary.test.ts && npm.cmd run check`
Expected: PASS; 0 ошибок. Если появится warning `state_referenced_locally` на `api` — обернуть чтение в `untrack` уже сделано (см. Step 2).

- [ ] **Step 5: Живая перепроверка**

Повторить Task 2 Steps 2–4: сортировка теперь переживает выделение. Дополнительно проверить ConnectFromLibrary («Добавить источник» → чекбоксы в шите работают, disabled-строки не выбираются).

- [ ] **Step 6: Commit**

```bash
git add src/lib/components/extractum-ui/GridSelectCell.svelte src/lib/components/research-projects/SourcesGrid.svelte src/lib/components/research-projects/SourcesGrid.test.ts
git commit -m "fix(research-projects): reactive row selection so sorting survives checkbox clicks"
```

---

## Self-Review Notes

- **Spec coverage:** компараторы + sort-конфиг (T1), проверка типа обёртки (T1 Step 5 contingency), живая проверка одиночной/мульти/чисел/null/выделения/фильтрации (T2), условный рефактор GridSelectCell с сохранением connectable/disabledReason (T3), «не в скоупе» не затронут.
- **Отклонение от спеки (задокументировано в T1):** «null всегда внизу в обоих направлениях» на column-компараторе svar недостижимо (desc = инверсия) — принято «null = самая старая дата»; спека обновляется при финализации, поведение фиксируется живой проверкой.
- **Type consistency:** компараторы `(a: unknown, b: unknown) => number` соответствуют svar-контракту значений ячеек; тест сравнивает по ссылке (`toBe(compareRuStrings)`).
