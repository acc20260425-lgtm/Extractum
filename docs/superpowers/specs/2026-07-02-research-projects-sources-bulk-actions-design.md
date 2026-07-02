# Sources bulk-action bar (/projects/next) — design

**Дата:** 2026-07-02
**Статус:** согласовано (brainstorming), готово к плану реализации
**Область:** новый экран `/projects/next` (v10 shell), таблица источников проекта.

## Цель

Добавить контекстную панель массовых действий над таблицей источников: когда
выбраны источники (чекбокс-колонка select-all уже реализована), показать бар с
действиями **Синхронизировать** и **Удалить** над выбранными. По макету v10.

## Согласованные решения

1. Набор действий — **Синхронизировать + Удалить** (как v10). Экспорт не добавляем.
2. Удаление — через **ExtractumDialog** (подтверждение), не нативный `confirm()`.
3. Синхронизация при смешанном выборе — **только поддерживаемые** (youtube
   video/playlist); неподдерживаемые пропускаются. Кнопка активна, если есть ≥1
   поддерживаемый источник.

## Размещение

Контекстная панель **над гридом** (между `ProjectToolbar` и `SourcesGrid` в
main-колонке `ResearchProjectsShell`), появляется при `selectedSourceIds.length > 0`.
Содержимое по v10: «Выбрано: N» · «Снять выделение» · «Синхронизировать» · «Удалить»
(danger). Панель занимает вертикальное место (push, не overlay).

## Компонент `SourcesBulkBar.svelte` (`src/lib/components/research-projects/`)

Презентационный + владеет своим confirm-диалогом.

Пропсы:
- `count: number` — число выбранных.
- `syncDisabled?: boolean` — нет поддерживаемых для синхронизации.
- `syncTitle?: string` — подсказка на disabled-кнопке sync.
- `onClear?: () => void`, `onSync?: () => void`, `onDelete?: () => void`.

Разметка:
- «Выбрано: {count}» + ссылка «Снять выделение» → `onClear`.
- Кнопка «Синхронизировать» — `disabled={syncDisabled}`, `title={syncTitle}`, → `onSync`.
- Кнопка «Удалить» (danger) → открывает `ExtractumDialog` подтверждения
  («Удалить {count} источник(а/ов) из проекта?»); по «Удалить» в диалоге → `onDelete`
  + закрыть; «Отмена» → закрыть. `ExtractumDialog` берётся из `extractum-ui`
  (границу импортов `research-projects-import-boundary` не нарушает).

Тексты — русские; стиль/токены как у существующих кнопок (danger = `--extractum-danger`).

## Встраивание в `ResearchProjectsShell`

Добавить проп-бэг `bulkBar?: ComponentProps<typeof SourcesBulkBar>` (по аналогии с
`toolbar`/`runDock`/`inspector`). Рендерить `<SourcesBulkBar {...bulkBar} />` в
main-колонке **над** `.research-projects-shell__grid`, когда `bulkBar` передан.
Страница передаёт бэг только при `count > 0` (иначе `undefined`).

## Wiring — страница `src/routes/projects/next/+page.svelte`

Из `sources: ProjectSourceRecord[]` + `selectedSourceIds`:
- `count = selectedSourceIds.length`.
- **syncable**: источник поддерживает sync, если `provider === "youtube"` и
  `source_subtype ∈ {"video","playlist"}`. `syncableIds = selectedSourceIds`,
  отфильтрованные по этому предикату (по `sources`).
- `syncDisabled = syncableIds.length === 0`; `syncTitle` при disabled —
  «Нет источников, поддерживающих синхронизацию».

Действия:
- `onClear` → `selectedSourceIds = []` (грид снимет выделение реактивно через
  `selectedRowIds`).
- `onSync` → `saving`-флаг; для каждого `id` из `syncableIds` вызвать
  `syncYoutubeSource(Number(id), { metadata: true, transcripts: true, comments: false })`;
  затем перезагрузить источники (`listProjectSources`).
- `onDelete` → `saving`-флаг; `removeProjectSources({ projectId, sourceIds:
  selectedSourceIds.map(Number) })`; `selectedSourceIds = []`; перезагрузить
  источники + `workflow.reload()` (summaries: `source_count`/`material_count`
  изменятся).

`bulkBar` бэг для shell: `count > 0 ? { count, syncDisabled, syncTitle, onClear,
onSync, onDelete } : undefined`.

## Ошибки и состояние

- Ошибки invoke → `railState.status` через существующий `formatError` (как в
  `createProjectRailWorkflow`).
- `saving` блокирует кнопки sync/delete на время операции.

## Тестирование

- `SourcesBulkBar.test.ts` — рендер-тест (jsdom; bits-ui Dialog рендерится в jsdom):
  счётчик «Выбрано: N»; `syncDisabled` дизейблит кнопку sync; клик «Удалить» →
  открытие диалога → клик подтверждения вызывает `onDelete`; `onClear`/`onSync`.
- `ResearchProjectsShell.test.ts` — `?raw`-ассерт проводки `bulkBar` над гридом.
- Логику syncable/страничный wiring (invoke) — визуально в Tauri (skill run-app /
  Tauri MCP-мост).

## Границы/изоляция

- `SourcesBulkBar` — один презентационный компонент + собственный confirm-диалог;
  логику выбора/синхронизации/удаления держит страница.
- Импорты фичевых файлов — только через `extractum-ui`/`$lib/*` (контракт
  `research-projects-import-boundary`).

## Не в скоупе

Экспорт выбранных; панель фильтров/«N из M» над таблицей (отдельный follow-up);
массовые действия для не-youtube синхронизации.
