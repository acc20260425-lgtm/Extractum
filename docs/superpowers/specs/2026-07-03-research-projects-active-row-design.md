# Активная строка vs чекбокс-выделение (/projects/next) — design

**Дата:** 2026-07-03
**Статус:** согласовано (brainstorming), готово к плану реализации
**Область:** таблица источников `/projects/next`; обёртка `ExtractumDataGrid`.
**Эталон:** `Research Projects v10.dc.html` — клик по строке делает её активной
(инспектор, фон `rgba(15,102,216,.07)` + полоска `inset 2px 0 0 #0f66d8`),
выделение для массовых действий — только чекбоксами.
**svar-основа:** официальный паттерн «Selecting rows with checkboxes»
(`docs/guides/configuration/select_rows.md`): `select={false}` отключает
выделение кликом по строке, `api.exec("select-row")` из чекбокс-ячеек
продолжает работать.

## Проблема

Сейчас клик по строке идёт в svar-selection: ставится галка, всплывает
bulk-бар «Выбрано: 1», и тот же массив питает инспектор
(`selectedSourceIds[0]`). В v10 это два независимых состояния.

## Решение

### `ExtractumDataGrid` (обёртка, `src/lib/components/extractum-ui/DataGrid.svelte`)

Новые пропсы:
- `selectOnClick?: boolean = true` → svar `select={selectOnClick}`.
  `SourcesGrid` передаёт `false`; `ConnectFromLibrary` не меняется
  (дефолт `true`, прежнее поведение).
- `activeRowId?: string | null = null`;
- `onRowClick?: (id: string) => void`.

Клик — делегирование на host-элементе обёртки:
- если `event.target.closest('[data-action="ignore-click"]')` — игнор
  (чекбоксы и прочие служебные зоны);
- иначе `closest('.wx-row')` → `data-id` (живая проверка показала формат
  `":430"` — префикс `:` перед нашим строковым id; точное правило снятия
  префикса фиксируется в реализации по живой проверке) → `onRowClick(id)`.

Подсветка активной строки — **динамическим CSS-правилом**, не классом на
DOM-узле: host получает уникальный `data-grid-uid`, компонент рендерит через
`{@html}` тег `<style>` с правилом

```
[data-grid-uid="…"] .wx-row[data-id="<активный id с префиксом>"] {
  background: color-mix(in srgb, var(--extractum-primary) 7%, var(--extractum-surface));
  box-shadow: inset 2px 0 0 var(--extractum-primary);
}
```

CSS-селектор переживает любые пере-рендеры svar (сортировка, обновление
данных) и не трогает svar-состояние — сортировка в безопасности (урок
итерации сортировки: любое изменение реактивных пропсов svar сбрасывает
sortMarks; сюда реактивные пропсы svar не добавляем).
Fallback (если `data-id` окажется ненадёжным): класс через MutationObserver —
только при провале основного пути.

### `SourcesGrid`

Прокидывает `selectOnClick={false}`, `activeRowId`, `onRowClick` в
`ExtractumDataGrid`. Пропсы компонента: `activeSourceId?: string | null`,
`onActivateSource?: (id: string) => void`.

### `ResearchProjectsShell`

Сквозные пропсы `activeSourceId` / `onActivateSource` → `SourcesGrid`.

### Страница `/projects/next/+page.svelte`

- `let activeSourceId = $state<string | null>(null)`.
- `selectedSourceRow` (питает инспектор) читает `activeSourceId` вместо
  `selectedSourceIds[0]`.
- `onActivateSource: (id) => (activeSourceId = id)`.
- Сброс `activeSourceId = null` при смене проекта (в `selectProject`).
- Активность СОХРАНЯЕТСЯ, если строку скрыл фильтр (инспектор продолжает
  показывать источник — как v10); чекбоксы не влияют на инспектор; клик по
  строке не влияет на bulk-бар.
- Повторный клик по активной строке оставляет её активной (v10; деактивации
  нет).

## Тестирование

- `SourcesGrid.test.ts` (`?raw`): `selectOnClick={false}`, `{activeRowId}`
  (или эквивалентная прокидка), `onRowClick`.
- Shell `?raw`: сквозные пропсы.
- Страница: юнит-логика инспектора уже покрыта (`buildSourceRow`); проводка —
  вживую.
- Живая проверка в Tauri (решающая):
  1) клик по строке → инспектор + подсветка (фон+полоска), bulk-бар НЕ
     появляется, чекбокс НЕ ставится;
  2) чекбокс → bulk-бар, инспектор не меняется;
  3) отсортировать → кликнуть строку → сортировка и маркер живы;
  4) select-all + активная строка сосуществуют;
  5) ConnectFromLibrary — прежнее поведение (клик по строке выделяет).

## Границы/изоляция

Вся svar-механика — в обёртке `ExtractumDataGrid`; фичевые файлы получают
только новые колбэки/пропсы (контракт import-boundary не затронут).

## Не в скоупе

Постоянный сворачиваемый инспектор (45px-рейл, v10) — отдельная итерация №2;
клавиатурная навигация по строкам (стрелки).
