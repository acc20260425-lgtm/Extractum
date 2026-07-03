# Панель проектов v10 (/projects/next) — design

**Дата:** 2026-07-03
**Статус:** согласовано (brainstorming), готово к плану реализации
**Область:** новый экран `/projects/next` (v10 shell), левая панель проектов.
**Эталон:** `Research Projects v10.dc.html` + `ProjectRow.dc.html` + `ContextMenu.dc.html`
из бандла `C:\Users\Dima\Downloads\Tauri MCP Bridge connection-handoff\tauri-mcp-bridge-connection\project\`
(HANDOFF.md: ContextMenu → shadcn DropdownMenu; инлайн-стили макета не переносить дословно).

## Цель

Довести левую панель проектов до паритета с v10: шапка с действиями, поиск,
секции (активный/закреплённые/обычные), сворачиваемый архив, компактный вид,
hover-своп пин ↔ «⋯», контекстные меню строк, создание/редактирование/удаление
проектов.

## Согласованные решения

1. Контекстное меню строки — **все 6 пунктов** как в макете; «Синхронизировать»
   и «Экспорт» — **disabled** с `title="Скоро"` (нет backend-API).
2. Шапка панели — **все 4 кнопки** как в макете; синхронизация — disabled
   «Скоро»; «⋯» — меню действий над активным проектом.
3. Удаление проекта — через **диалог подтверждения** (ExtractumDialog), не сразу.
4. Объём — полный: поиск, сворачиваемый архив, компактный вид, hover-поведение
   и активная полоска.
5. Архитектура — **вариант A**: новый презентационный `ProjectRailPanel.svelte`,
   владеющий UI-состоянием панели; данные и действия — пропсами.

## Компоненты и файлы

### `ProjectRailPanel.svelte` (новый, `src/lib/components/research-projects/`)

Вся панель целиком. **Заменяет** `ProjectRailSections.svelte` в shell
(старый компонент и его тест удаляются — он используется только там).

Владеет UI-состоянием (session-only, без persistence):
- `query: string` — поиск;
- `compact: boolean` — компактный вид;
- `archiveOpen: boolean` — архив развёрнут (по умолчанию `false`);
- `headerMenuOpen`, открытое контекстное меню строки — внутри соответствующих
  DropdownMenu.

Пропсы (данные + действия):
- `summaries: ProjectSummary[]`, `selectedProjectId: number | null`, `now: number`;
- `onSelect?: (id: number) => void`;
- `onCreate?: () => void` — «+» в шапке;
- `onEdit?: (id: number) => void`;
- `onTogglePin?: (id: number, pinned: boolean) => void`;
- `onToggleArchive?: (id: number, archived: boolean) => void`;
- `onDelete?: (id: number) => void` — вызывается ПОСЛЕ подтверждения в диалоге.

Разметка (по v10):
- Шапка: «ПРОЕКТЫ» (uppercase, muted) + 4 иконки-кнопки 24×24:
  1. компактный вид — toggle; `title` = «Компактный вид» / «Комфортный вид»;
  2. «+» — `onCreate`; `title="Создать проект"`;
  3. синхронизация — **disabled**, `title="Скоро"`;
  4. «⋯» — DropdownMenu активного проекта (видна только при
     `selectedProjectId !== null`): Редактировать → `onEdit(selectedId)`;
     Экспорт — disabled «Скоро»; разделитель; Удалить (danger) → диалог
     подтверждения → `onDelete(selectedId)`.
- Поиск: инпут «Поиск проектов» с иконкой-лупой и очисткой «×» (крестик виден
  только при непустом query).
- Секции: «Закреплённые» (заголовок с пин-иконкой; показывается, если есть
  закреплённые или виден активный), строка активного проекта, закреплённые;
  «Проекты» — обычные; «Проекты не найдены» — когда query непуст и все секции
  пусты (включая архив).
- Архив: строка-тогл «АРХИВ N» с шевроном (поворот 90° при открытии) и полным
  счётчиком `N` (НЕ отфильтрованным); внутри — архивные строки (вариант
  `archived`). Свёрнут по умолчанию.
- Диалог удаления: ExtractumDialog «Удалить проект» с текстом
  «Удалить проект «{name}»? Действие необратимо.», кнопки «Отмена» /
  «Да, удалить» (danger). Один диалог на панель; хранит `pendingDelete:
  { id, name } | null`.

### `ProjectRow.svelte` (расширяется)

Новые пропсы: `variant: "active" | "normal" | "archived"` (default `normal`),
`compact?: boolean` и колбэки действий `onEdit?`, `onTogglePin?`,
`onToggleArchive?`, `onRequestDelete?: (id: number, name: string) => void`.
Меню строка строит сама по `variant` и `row.pinned`/`row.archived`
(пункты — внутри компонента, не пропсом).

Поведение по `ProjectRow.dc.html`:
- `active`: синий фон `color-mix(--extractum-primary 10%)`, левая полоска 3×22px
  `--extractum-primary`, имя — primary/600; точка статуса как сейчас.
- `archived`: приглушённые имя (`--extractum-muted`) и meta (`--extractum-muted-2`),
  меньший паддинг.
- `compact`: meta скрыта, паддинг 5px 10px, `title` = «{name} — {meta}».
- Hover-своп: справа зона 22×22; пин-иконка (если закреплён или active) видима
  в покое, на hover заменяется кнопкой «⋯» (opacity-transition). Если пина
  нет — «⋯» появляется только на hover.
- «⋯» и правый клик (contextmenu) открывают DropdownMenu строки.
- Меню строки (variant normal/active): Редактировать, Закрепить/Открепить,
  Синхронизировать (disabled «Скоро»), Экспорт (disabled «Скоро»),
  В архив/Из архива, разделитель, Удалить (danger → подтверждение).
  Variant archived: Из архива, разделитель, Удалить (danger → подтверждение).
- Удаление из меню строки НЕ вызывает `onDelete` напрямую — строка сообщает
  панели (`onRequestDelete(id, name)`), диалог живёт в панели.

### `ExtractumDropdownMenu` (extractum-ui, новые реэкспорты)

В `src/lib/components/extractum-ui/index.ts` добавить реэкспорты из
`$lib/components/ui/dropdown-menu/index.js` (по образцу Popover):
`ExtractumDropdownMenu` (Root), `ExtractumDropdownMenuTrigger`,
`ExtractumDropdownMenuContent`, `ExtractumDropdownMenuItem`,
`ExtractumDropdownMenuSeparator`.
Danger-пункт — цвет `--extractum-danger`, hover-тинт
`color-mix(... 8%, transparent)`; disabled-пункт — muted + `title`.

### Диалоги создания/редактирования

Существующий `ProjectEditorDialog.svelte` (умеет create/edit). Страница держит
`editorOpen`, `editorProject: ResearchProjectView | null`.

## Логика и данные

### `filterProjectRail` (новая чистая функция, `src/lib/ui/research-projects-rail.ts`)

```
filterProjectRail(sections: ProjectRailSections, query: string): ProjectRailSections
```
Матч без регистра по `row.name` и `row.meta`; пустой/пробельный query возвращает
секции как есть. Юнит-тестируется.

### Страница `/projects/next/+page.svelte`

- `onCreate` → `editorProject = null; editorOpen = true`;
  submit → `createProject` → reload.
- `onEdit(id)` → `editorProject` из summaries; submit → `updateProject` → reload.
- `onTogglePin` / `onToggleArchive` → существующие `workflow.setPinned` /
  `workflow.setArchived` (сами делают reload).
- `onDelete(id)` → `deleteProject(id)`; если удалён активный —
  `selectedProjectId = null`, `sources = []`, `selectedSourceIds = []`;
  затем `workflow.reload()`. Ошибки → `railState.status` (как у bulk-действий),
  `saving` на время операции.
- Shell: prop `summaries/selectedProjectId/now/onSelectProject` заменяется
  prop-бэгом `railPanel` (по образцу `toolbar`/`bulkBar`) — ComponentProps
  ProjectRailPanel; shell рендерит `<ProjectRailPanel {...railPanel} />` в
  `.research-projects-shell__rail`.

## Ошибки и состояние

- Ошибки create/update/delete → `railState.status` через существующий формат
  «Не удалось …»; `saving` блокирует повторный сабмит (ProjectEditorDialog уже
  принимает `saving`/`error`).

## Тестирование

- `ProjectRailPanel.test.ts` (jsdom рендер): поиск фильтрует; «Проекты не
  найдены»; архив свёрнут по умолчанию, счётчик N, раскрытие по клику; компакт
  скрывает meta; 4 кнопки шапки, синхронизация disabled; кнопка «⋯» шапки
  скрыта без выбранного проекта.
- `ProjectRow.test.ts` (расширить): активный вариант (полоска/фон), archived
  приглушён, compact скрывает meta и ставит title, «⋯» присутствует.
  Если bits-ui DropdownMenu не рендерится в jsdom — меню верифицируется
  `?raw`-ассертами (конвенция проекта), интеракция — вживую в Tauri.
- `filterProjectRail` — юнит-тесты (матч по имени, по meta, регистр, пустой query).
- Живая проверка в Tauri: hover-своп пин↔«⋯», контекстное меню по правому клику,
  создание → появление в списке, удаление с подтверждением, архив/пин.

## Границы/изоляция

- Импорты фичевых файлов — только `extractum-ui`/`$lib/*` (контракт
  `research-projects-import-boundary`); bits-ui dropdown — только через новые
  реэкспорты extractum-ui.
- `ProjectRailPanel` презентационный: API-вызовов внутри нет, только колбэки.

## Не в скоупе

Backend для «Экспорт» и «Синхронизировать проект» (пункты disabled «Скоро»);
persistence компакта/архива между сессиями; drag-and-drop сортировка;
прогресс-бар в строке running-проекта (в v10-макете есть `progress: 62` у
running — данных о прогрессе в `ProjectSummary` нет, точка статуса остаётся
единственным индикатором).
