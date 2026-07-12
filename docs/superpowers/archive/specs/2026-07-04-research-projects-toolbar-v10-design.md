# ProjectToolbar по макету v10 (/projects/next) — design

**Дата:** 2026-07-04
**Статус:** согласовано (brainstorming), готово к плану реализации
**Область:** `ProjectToolbar.svelte`, `PeriodPopover.svelte`, `ComboSelect.svelte`
(+ новые панели), проводка страницы `/projects/next`.
**Эталон:** `ProjectToolbar.dc.html` (полный: wide-режим, narrow-режим
«Параметры», поповеры), HANDOFF.md (маппинг: Период → Popover; Промпт/Модель →
Popover + Command; адаптив — container query `@container tb (max-width:600px)`).

## Проблема

Текущий тулбар сильно отличается от макета: заголовок ужимается, триггеры с
текстовыми префиксами без иконок/caret/open-состояния, поповеры — плоские
списки без поиска-описаний-групп-поддиапазонов, нет произвольного диапазона
дат, нет адаптива.

## Согласованный объём (всё в этой итерации)

1. Внешний вид триггеров + заголовок + play-кнопка.
2. Поповеры по макету (период с кастомным диапазоном; промпт с поиском;
   модель с группами).
3. Адаптив «Параметры» (≤600px контейнера).

## Архитектура — панели переиспользуются wide и narrow

### `PeriodPanel.svelte` (новый, `research-projects/`)

Содержимое поповера периода (без Popover-обёртки):
- шапка «Данные проекта: {DD.MM.YY – DD.MM.YY}» (иконка-часы, muted);
- пресеты: label 12.5px/600 + поддиапазон моноширинным 10.5px muted
  (`DD.MM.YY – DD.MM.YY` из unix `from`/`to`), ✓ у выбранного, выбранный ряд —
  тинт `color-mix(primary 6%)`;
- разделитель;
- секция «ПРОИЗВОЛЬНЫЙ ДИАПАЗОН»: два `<input type="date">` со стрелкой «→»
  между ними + кнопка «Применить диапазон» (тинт-кнопка primary 10%).

Пропсы: `{ presets: PeriodPreset[]; selectedId?: string; dataRange:
{ from: number; to: number } | null; onSelect?: (preset: PeriodPreset) => void }`.
«Применить диапазон» эмитит синтетический пресет
`{ id: "custom", label: "DD.MM.YY–DD.MM.YY", from, to }` (unix из значений
инпутов, from = начало дня, to = конец дня); кнопка заблокирована, если не
заданы обе даты либо from > to. Начальные значения инпутов — из выбранного
пресета.

### `OptionsPanel.svelte` (новый, `research-projects/`)

Содержимое комбо-поповера (промпт/модель):
- строка поиска (иконка-лупа, плейсхолдер пропсом);
- список: опциональные заголовки групп (uppercase 10px muted), элементы:
  опциональная цветная точка, название 12.5px/600, вторая строка —
  `description` (11px muted) либо `mono` (моноширинный 10px muted), ✓ у
  выбранного, тинт выбранного ряда;
- «Ничего не найдено» при пустом фильтре.

Тип опции (расширение в `ComboSelect.svelte` module-script):

```ts
export type ComboOption = {
  value: string;
  label: string;
  description?: string; // вторая строка обычным шрифтом
  mono?: string;        // вторая строка моноширинным (model-id)
  dot?: string;         // CSS-цвет точки провайдера
  group?: string;       // заголовок группы (рендерится при смене group)
};
```

Пропсы: `{ options, selectedValue?, placeholder, emptyLabel?, onSelect? }`.
Поиск — по `label` + `description` + `mono`, без регистра (своя фильтрация в
панели; bits-ui Command можно не использовать — панель проще и полностью
контролируема; если Command остаётся, фильтрация через `keywords`).

### `PeriodPopover.svelte` / `ComboSelect.svelte` (тонкие обёртки)

Триггер + `ExtractumPopoverContent` с соответствующей панелью.
Триггеры (общий вид):
- Период: svg-календарь + label выбранного пресета (без «Период:») + caret ▾;
- Промпт: svg-строки + label выбранного шаблона (без «Промпт:») + caret;
- Модель: цветная точка (`dot` выбранной опции, дефолт muted) + label + caret.
ComboSelect получает проп `triggerIcon: "lines" | "dot"`.
Плейсхолдеры триггеров при отсутствии выбора: «Период» / «Промпт» / «Модель».

Open-состояние — чистым CSS в ProjectToolbar по bits-ui-атрибуту:
`[data-slot="popover-trigger"][data-state="open"]` → рамка
`var(--extractum-primary)` + ring `0 0 0 3px color-mix(in srgb,
var(--extractum-primary) 12%, transparent)` + caret `rotate(180deg)`.

### `ProjectToolbar.svelte`

- Заголовок: колонка eyebrow «Research project» (600 10px uppercase
  `--extractum-muted-2`, letter-spacing .05em) + title 600 15px, блок
  `flex: 1; min-width: 0` (title ellipsis).
- Wide-ряд: PeriodPopover, ComboSelect (prompt), ComboSelect (model),
  кнопка «Запустить» (play-svg + label, height 32, primary,
  `box-shadow: 0 1px 2px color-mix(primary 30%)`).
- Narrow-режим: `container-type: inline-size; container-name: tb` на
  корне; `@container tb (max-width: 600px)`: wide-ряд `display:none`,
  narrow-ряд `display:flex`. Narrow: триггер «Параметры» (svg-слайдеры +
  caret, open-подсветка) → `ExtractumPopover` 296px: заголовок «ПАРАМЕТРЫ
  ЗАПУСКА», три секции-аккордеона (Период/Промпт/Модель: строка с иконкой,
  названием, текущим значением справа, caret; клик раскрывает панель этой
  секции, открыта максимум одна) — внутри те же `PeriodPanel`/`OptionsPanel`;
  плюс квадратная play-кнопка 32×32 с `title={runLabel}`.
- Новые пропсы тулбара: `dataRange: { from: number; to: number } | null`
  (для «Данные проекта»); `runLabel` default становится «Запустить».
- `runDisabled` сохраняется (обе run-кнопки).

## Страница `/projects/next/+page.svelte`

- `let customPeriod = $state<PeriodPreset | null>(null)`.
- `selectedPeriod`: если `selectedPeriodId === "custom"` → `customPeriod`,
  иначе поиск в `periodPresets` (как сейчас).
- `onSelectPeriod: (preset) => { if (preset.id === "custom") customPeriod =
  preset; selectedPeriodId = preset.id; }`.
- Сброс `customPeriod = null` при смене проекта.
- В `toolbar`-бэг добавить `dataRange: railState.dataRange`.
- `runLabel` не передавать (новый дефолт «Запустить»).

## Тестирование

- `PeriodPanel.test.ts` (jsdom): «Данные проекта» с форматированным
  диапазоном; пресеты с поддиапазонами и ✓; выбор пресета → `onSelect`;
  кастомный диапазон: заполнить оба date-инпута → «Применить» эмитит
  `{id:"custom", from: начало дня, to: конец дня}`; кнопка заблокирована при
  пустых/перевёрнутых датах.
- `OptionsPanel.test.ts` (jsdom): поиск фильтрует по label/description/mono;
  группы-заголовки; точки; описание/моно-строка; ✓; «Ничего не найдено»;
  `onSelect`.
- `ProjectToolbar.test.ts` (обновить): eyebrow + title; триггеры БЕЗ
  префиксов «Период:»/«Промпт:»; play-кнопка «Запустить»; `?raw` — container
  query и narrow-разметка (`@container`, «Параметры»").
- Живая проверка в Tauri: open-подсветка триггеров; поповеры по макету;
  кастомный диапазон применяется и попадает в запуск анализа
  (periodFrom/periodTo); адаптив — сузить окно/открыть инспектор до <600px
  контейнера → «Параметры» со стопкой секций, квадратный play; выбор в
  narrow-режиме синхронен с wide (одно состояние).

## Границы/изоляция

Импорты — только `extractum-ui` (Popover/Command уже реэкспортированы);
панели презентационные; состояние периода/промпта/модели — на странице (как
сейчас); контракт import-boundary не затронут.

## Не в скоупе

Загрузка списка моделей (options остаются пустыми — отдельная итерация;
структура групп/точек уже поддержана типом); описания промптов с бэкенда
(поле появится — отобразится автоматически через `description`); клавиатурная
навигация по спискам.
