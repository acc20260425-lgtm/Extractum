# Панель фильтров источников (/projects/next) — design

**Дата:** 2026-07-03
**Статус:** согласовано (brainstorming), готово к плану реализации
**Область:** новый экран `/projects/next` (v10 shell), панель над таблицей источников.
**Эталон:** `Research Projects v10.dc.html` (stats-бар + строка фильтров + bulk-overlay),
`FilterPopover.dc.html`, `Chip.dc.html` из бандла
`C:\Users\Dima\Downloads\Tauri MCP Bridge connection-handoff\tauri-mcp-bridge-connection\project\`.

## Цель

Панель управления списком источников над таблицей: кнопка «Фильтры» с бейджем,
чипы активных фильтров, «Сбросить», счётчик «N из M», кнопка «Добавить
источник»; раскрываемая строка фильтров (поиск, тип, материалы от/до, даты
с/по, статус); бар массовых действий становится overlay поверх панели (как v10).

## Согласованные решения

1. «Добавить источник» — **подключить существующий `ConnectFromLibrary`**
   (подключение источников из библиотеки, addProjectSources).
2. Bulk-бар — **overlay** поверх stats-бара (`position:absolute; inset:0`),
   рефактор существующего `SourcesBulkBar` (был push).
3. Фильтр дат — **нативные `<input type="date">`** «с»/«по» (полная дата,
   без парсинга ДД.ММ из макета).
4. Фильтрация — **подход A**: клиентская, чистой функцией до передачи в грид
   (не svar `filter-rows`, не встроенные header-фильтры — не совпадают с макетом).

## Компоненты

### `SourcesFilterBar.svelte` (новый, `research-projects/`)

Stats-бар. Презентационный, состояние приходит пропсами:
- `filtersOpen: boolean`, `onToggleFilters: () => void` — кнопка «Фильтры»
  (иконка-воронка); при `filterCount > 0` синий бейдж-счётчик на кнопке.
- `chips: SourceFilterChip[]`, `onRemoveChip: (key: string) => void` — чипы
  «Тип: telegram» и т.п., точка провайдера/статуса, «✕» удаляет.
- `filtersActive: boolean`, `onClearAll: () => void` — ссылка «Сбросить»
  (видна только при активных фильтрах).
- `shownCount: number`, `totalCount: number` — текст «{N} из {M}».
- `onAddSource: () => void` — справа кнопка «+ Добавить источник»
  (обводка primary, тинт-фон как в v10).

### `SourcesFilterRow.svelte` (новый, `research-projects/`)

Строка фильтров, рендерится под stats-баром при `filtersOpen`. CSS-grid
`grid-template-columns: 34px minmax(0,1fr) 116px 116px 150px 104px` — колонки
совпадают с ширинами svar-грида, контролы визуально стоят под колонками таблицы:
- (пустая ячейка под чекбокс-колонкой);
- поиск: инпут с лупой и «×» (по `title`/`handle`);
- «Тип»: поповер-мультивыбор (`ExtractumPopover`) с чекбоксами и точками
  провайдеров (`--extractum-provider-telegram/-youtube`); триггер показывает
  «Все» / имя / «K выбр.»;
- «Материалы»: два `<input type="number">` «от»/«до»;
- «Последний сбор»: два `<input type="date">` «с»/«по»;
- «Статус»: поповер-мультивыбор со статусами `active | syncing | error |
  unavailable` и их цветами (success/primary/danger/warning).

Пропсы: `filters: SourceFilters`, `onChange: (filters: SourceFilters) => void`
(компонент отдаёт новый объект целиком; состояние держит страница).

### `SourcesBulkBar.svelte` (рефактор → overlay)

Разметка/пропсы не меняются; стили: `position:absolute; inset:0; z-index:5`,
фон `color-mix(in srgb, var(--extractum-primary) 8%, var(--extractum-surface))`,
нижняя граница тинт primary. Рендерится внутри общего контейнера панели (см. Shell).

### `SourcesGrid.svelte` (минимальное изменение)

Новый проп `overlay?: string` (default `"Нет источников"`) — прокидывается в
`ExtractumDataGrid`. При активных фильтрах и нуле видимых строк страница
передаёт «Под условия ничего не подходит».

### `ResearchProjectsShell.svelte`

Новые проп-бэги: `filterBar?: ComponentProps<typeof SourcesFilterBar>`,
`filterRow?: ComponentProps<typeof SourcesFilterRow>`, плюс существующий
`bulkBar`. Разметка main-колонки:

```
{#if filterBar}
  <div class="research-projects-shell__statsbar">   ← position:relative
    <SourcesFilterBar {...filterBar} />
    {#if bulkBar}<SourcesBulkBar {...bulkBar} />{/if}  ← overlay поверх
  </div>
{/if}
{#if filterRow}<SourcesFilterRow {...filterRow} />{/if}
<div class="research-projects-shell__grid">…
```

`overlay`-текст грида — новый проп shell `gridOverlay?: string`, прокинуть в
`SourcesGrid`. Если `filterBar` не передан (нет выбранного проекта) — как сейчас.

### `ConnectFromLibrary` (существующий, только проводка)

«Добавить источник» → `connectOpen = true`; при первом открытии страница
лениво грузит каталог `listLibrarySources()` и строит
`buildLibrarySourcesView(catalogRecords, sources, selectedProjectViewId)`;
подключение: `connectableSelection(...)` → `addProjectSources({ projectId,
sourceIds })` → перезагрузка `listProjectSources` + `workflow.reload()`.
Точные конверсии id (view-id ↔ number) уточняются в плане по
`research-projects-model.ts`; если проп `project: ResearchProjectView | null`
используется только для заголовка — сузить тип по образцу ProjectEditorDialog.

## Логика — `src/lib/ui/research-projects-source-filters.ts` (новый чистый модуль)

```ts
export interface SourceFilters {
  query: string;
  types: string[];          // provider: "telegram" | "youtube" | …
  statuses: string[];       // sync_status: active | syncing | error | unavailable
  materialsMin: number | null;
  materialsMax: number | null;
  syncedFrom: string | null; // "YYYY-MM-DD" (значение input type=date)
  syncedTo: string | null;
}
export function emptySourceFilters(): SourceFilters;
export function countActiveSourceFilters(filters: SourceFilters): number;
export function filterProjectSources(
  records: ProjectSourceRecord[], filters: SourceFilters,
): ProjectSourceRecord[];
export interface SourceFilterChip { key: string; label: string; dot: string | null }
export function buildSourceFilterChips(filters: SourceFilters): SourceFilterChip[];
export function removeSourceFilterChip(filters: SourceFilters, key: string): SourceFilters;
```

Правила фильтрации (все условия — И):
- `query`: подстрока в `title` или `handle`, без регистра;
- `types`: `provider ∈ types` (пустой список = все);
- `statuses`: `sync_status ∈ statuses` (пустой = все);
- материалы: `item_count >= materialsMin` / `<= materialsMax`;
- даты: `last_synced_at` (unix-секунды) в диапазоне [начало дня syncedFrom,
  конец дня syncedTo]; записи с `last_synced_at === null` не проходят фильтр
  дат, если он задан.

Чипы: `query` → «Источник: {q}» (key `query`); каждый тип → «Тип: {t}»
(key `type:{t}`, dot из `--extractum-provider-*`); каждый статус →
«Статус: {s}» (key `status:{s}`, dot цвета статуса); материалы →
«Материалы: {min ?? 0}–{max ?? ∞}» (key `materials`); даты →
«Период: {from | "…"}–{to | "…"}» (key `period`, формат DD.MM.YYYY).
`removeSourceFilterChip` сбрасывает соответствующее поле.

## Страница `/projects/next/+page.svelte`

- `let filters = $state(emptySourceFilters())`, `let filtersOpen = $state(false)`,
  `let connectOpen = $state(false)`, состояние библиотеки для диалога.
- `visibleSources = $derived(filterProjectSources(sources, filters))` → в shell
  вместо `sources`; `chips = $derived(buildSourceFilterChips(filters))`.
- Сброс фильтров при смене проекта (`selectProject`): `filters =
  emptySourceFilters()`.
- **Массовые действия считаются по `selectedSourceIds` как раньше** — выбранные
  строки, скрытые фильтром, остаются выбранными (как в v10; select-all в
  заголовке оперирует видимыми строками грида).
- `gridOverlay`: `filtersActive && visibleSources.length === 0` →
  «Под условия ничего не подходит», иначе «Нет источников».
- Ошибки подключения источников → `railState.status` (формат «Не удалось …»);
  `saving` на время connect.

## Тестирование

- `research-projects-source-filters.test.ts` — юнит: каждое поле, комбинация,
  пустые фильтры (те же ссылки/равенство), чипы + удаление чипа, счётчик.
- `SourcesFilterBar.test.ts` — jsdom: бейдж-счётчик, чипы с «✕», «Сбросить»
  видна только при активных, «N из M», колбэки (toggle/removeChip/clearAll/
  addSource).
- `SourcesFilterRow.test.ts` — jsdom: поповеры типов/статусов (bits-ui Popover
  рендерится), чекбокс тоглит и вызывает `onChange` с новым объектом, number/
  date-инпуты обновляют фильтры, «×» очищает поиск.
- Shell — `?raw`: statsbar-контейнер с `{...filterBar}`, overlay-бар внутри,
  `{...filterRow}`, `gridOverlay`.
- Живая проверка в Tauri: выравнивание строки фильтров под колонками,
  overlay bulk-бара при выделении, фильтрация + счётчик + чипы, «Добавить
  источник» → подключение из библиотеки, «Под условия ничего не подходит».

## Границы/изоляция

- Импорты фичевых файлов — только `extractum-ui`/`$lib/*` (контракт
  `research-projects-import-boundary`); поповеры — `ExtractumPopover*`.
- Компоненты презентационные; фильтры и данные держит страница; логика —
  в чистом модуле.

## Не в скоупе

Сортировка колонок; табы; сохранение фильтров между сессиями/проектами;
серверная фильтрация; изменение состава колонок грида.
