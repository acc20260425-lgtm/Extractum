# Табы разделов проекта (/projects/next) — design

**Дата:** 2026-07-03
**Статус:** согласовано (brainstorming), готово к плану реализации
**Область:** новый экран `/projects/next`, ряд табов между тулбаром и панелью фильтров.
**Эталон:** `Research Projects v10.dc.html` (ряд 40px, активный таб — синий текст +
`box-shadow: inset 0 -2px 0 #0f66d8`).

## Цель

Ряд из 6 табов разделов проекта. Работающий раздел — «Источники» (текущая
таблица со всем функционалом); остальные — заглушка «Раздел … в разработке».

## Согласованные решения

1. Остальные разделы — **заглушки** (каждый раздел — отдельная будущая итерация).
2. Ярлыки — **по-русски**: Обзор · Источники · Факты · Отчёты · Запуски · Промпты.
3. Подход A: простой презентационный компонент с подчёркиванием (не shadcn
   ExtractumTabs — другая стилистика и лишний content-механизм; не URL-роутинг —
   оверкилл для одной работающей секции).

## Компоненты

### `ProjectTabs.svelte` (новый, `src/lib/components/research-projects/`)

Презентационный. Пропсы:
`{ active: ProjectSectionId; onSelect?: (id: ProjectSectionId) => void }`.

Экспортирует тип и список секций:

```ts
export type ProjectSectionId =
  | "overview" | "sources" | "evidence" | "reports" | "runs" | "prompts";
export const PROJECT_SECTIONS: { id: ProjectSectionId; label: string }[] = [
  { id: "overview", label: "Обзор" },
  { id: "sources", label: "Источники" },
  { id: "evidence", label: "Факты" },
  { id: "reports", label: "Отчёты" },
  { id: "runs", label: "Запуски" },
  { id: "prompts", label: "Промпты" },
];
```

Разметка: контейнер 40px, `role="tablist"`, кнопки `role="tab"`,
`aria-selected`; активный — `--extractum-primary` текст 600 +
`box-shadow: inset 0 -2px 0 var(--extractum-primary)`; неактивный —
`--extractum-muted` 500. Нижняя граница ряда `--extractum-border`.
Кнопки — scoped-override глобального button-правила
(`button:not([data-slot="button"])`).

### `ResearchProjectsShell.svelte`

- Новый бэг `tabs?: ComponentProps<typeof ProjectTabs>` — рендер
  `<ProjectTabs {...tabs} />` сразу ПОД `<ProjectToolbar>` (до statsbar).
- Новый проп `sectionPlaceholder?: string` — если задан (непустой), ВМЕСТО
  statsbar + строки фильтров + грида рендерится центрированная заглушка
  (класс `research-projects-shell__section-placeholder`, muted-текст).
  Тулбар, табы, RunDock, Inspector — на местах.

### Страница `/projects/next/+page.svelte`

- `let activeSection = $state<ProjectSectionId>("sources")`.
- `selectProject`: сброс `activeSection = "sources"`.
- Бэг `tabs` при выбранном проекте:
  `{ active: activeSection, onSelect: (id) => (activeSection = id) }`.
- `filterBar` / `filterRow` / `bulkBar` передаются только при
  `activeSection === "sources"` (доп. условие к существующим).
- `sectionPlaceholder`: при `activeSection !== "sources"` —
  `Раздел «{label}» в разработке` (label из PROJECT_SECTIONS).
- Состояние источников (фильтры, выделение, сортировка) при переключении
  табов НЕ сбрасывается страницей: фильтры/выделение — state страницы,
  сортировка живёт в svar-гриде. Грид размонтируется при уходе с
  «Источников» — сортировка после возврата не гарантируется (фиксируется
  живой проверкой; допустимо, аналогично поведению фильтрации из прошлой
  спеки).

## Тестирование

- `ProjectTabs.test.ts` (jsdom): 6 табов с русскими ярлыками; `aria-selected`
  только у активного; клик вызывает `onSelect("reports")`.
- `ResearchProjectsShell.test.ts` (`?raw`): `{...tabs}` рендерится после
  тулбара и до statsbar; ветка `sectionPlaceholder` вместо грида.
- Живая проверка в Tauri: переключение всех 6 табов, заглушки с правильными
  названиями, возврат на «Источники» (данные/фильтры/выделение целы),
  сброс на «Источники» при смене проекта; RunDock/Inspector видны на всех табах.

## Границы/изоляция

- Импорты — только `$lib/*`/extractum-ui (контракт import-boundary);
  ProjectTabs — чистая разметка без сторонних примитивов.

## Не в скоупе

Контент разделов Обзор/Факты/Отчёты/Запуски/Промпты; сохранение активного
таба в URL или между сессиями; бейджи-счётчики на табах.
