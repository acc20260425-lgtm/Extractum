# ProjectToolbar v10 — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Привести тулбар `/projects/next` к макету v10: eyebrow-заголовок, триггеры с иконками/caret/open-подсветкой, поповеры с поиском/группами/кастомным диапазоном, адаптив «Параметры».

**Architecture:** Контент поповеров — в переиспользуемых панелях `PeriodPanel`/`OptionsPanel` (без bits-ui Command — свой инпут и фильтрация, полностью jsdom-тестируемо); `PeriodPopover`/`ComboSelect` — тонкие обёртки Popover+панель; ProjectToolbar — wide-ряд + narrow-режим «Параметры» через container query; open-подсветка триггеров — CSS по `data-state` bits-ui.

**Tech Stack:** Svelte 5 runes, bits-ui Popover (extractum-ui), CSS container queries, vitest + jsdom, Tauri MCP.

**Source spec:** `docs/superpowers/specs/2026-07-04-research-projects-toolbar-v10-design.md`

## Global Constraints

- **Import boundary**: фичевые файлы — только `extractum-ui`/`$lib/*`.
- **bits-ui Command НЕ используется** (не рендерится в jsdom): OptionsPanel — свой `<input>` + фильтрация.
- **CSS-гочтча:** глобальное `button:not([data-slot="button"]) { background: var(--extractum-primary); color:#fff; }` — все нейтральные кнопки/триггеры требуют scoped-override.
- **Open-состояние триггеров:** CSS `.project-toolbar :global([data-slot="popover-trigger"][data-state="open"])` → `border-color: var(--extractum-primary); box-shadow: 0 0 0 3px color-mix(in srgb, var(--extractum-primary) 12%, transparent)`; caret внутри — `rotate(180deg)`.
- **Тексты (verbatim):** eyebrow «Research project»; run «Запустить»; поповеры: «Данные проекта: …», «Произвольный диапазон», «Применить диапазон», «Поиск шаблона…», «Поиск модели…», «Ничего не найдено», narrow: «Параметры», «Параметры запуска», секции «Период»/«Промпт»/«Модель».
- **Формат дат:** `DD.MM.YY` (unix-секунды, локальное время).
- **Кастомный период:** синтетический `PeriodPreset { id: "custom", label: "DD.MM.YY–DD.MM.YY", from: начало дня, to: конец дня }`.
- **jsdom:** bits-ui Popover рендерится; container query НЕ вычисляется (narrow-разметка присутствует в DOM всегда — тесты учитывают дубликаты кнопок; сам адаптив — `?raw` + живая проверка).
- **Тестовый раннер:** `node scripts/run-vitest.mjs run <files>`; гейт `npm.cmd run check` (baseline 0 ошибок, 2 warning в ProjectsShell.svelte).
- **Коммит на задачу; push не делать.** Ветка `feat/research-projects-toolbar-v10` от main; план — первым коммитом.

---

### Task 1: Хелперы формата дат периода

**Files:**
- Modify: `src/lib/ui/research-projects-period.ts`
- Test: `src/lib/ui/research-projects-period.test.ts` (добавить describe)

**Interfaces:**
- Produces:
  - `formatPeriodDate(unix: number): string` — `DD.MM.YY` локально;
  - `periodRangeLabel(from: number, to: number): string` — `DD.MM.YY – DD.MM.YY`.

- [ ] **Step 1: Write the failing tests**

Добавить в конец `src/lib/ui/research-projects-period.test.ts` (импорт дополнить `formatPeriodDate, periodRangeLabel`):

```ts
describe("period date formatting", () => {
  const unix = (y: number, m: number, d: number) => new Date(y, m - 1, d, 12).getTime() / 1000;

  it("formats unix seconds as DD.MM.YY in local time", () => {
    expect(formatPeriodDate(unix(2025, 5, 31))).toBe("31.05.25");
    expect(formatPeriodDate(unix(2024, 1, 3))).toBe("03.01.24");
  });

  it("builds a range label", () => {
    expect(periodRangeLabel(unix(2024, 3, 14), unix(2025, 5, 31))).toBe("14.03.24 – 31.05.25");
  });
});
```

- [ ] **Step 2: Run to verify FAIL**

Run: `node scripts/run-vitest.mjs run src/lib/ui/research-projects-period.test.ts` — FAIL (нет экспортов).

- [ ] **Step 3: Implement**

В конец `research-projects-period.ts`:

```ts
export function formatPeriodDate(unix: number): string {
  const date = new Date(unix * 1000);
  const dd = String(date.getDate()).padStart(2, "0");
  const mm = String(date.getMonth() + 1).padStart(2, "0");
  const yy = String(date.getFullYear()).slice(2);
  return `${dd}.${mm}.${yy}`;
}

export function periodRangeLabel(from: number, to: number): string {
  return `${formatPeriodDate(from)} – ${formatPeriodDate(to)}`;
}
```

- [ ] **Step 4: Run to verify PASS**, **Step 5: Commit** (сначала `git checkout -b feat/research-projects-toolbar-v10`; план — отдельным коммитом `docs: add toolbar v10 implementation plan`)

```bash
git add src/lib/ui/research-projects-period.ts src/lib/ui/research-projects-period.test.ts
git commit -m "feat(research-projects): add period date format helpers"
```

---

### Task 2: OptionsPanel + расширенный ComboOption + новый ComboSelect

**Files:**
- Create: `src/lib/components/research-projects/OptionsPanel.svelte`
- Rewrite: `src/lib/components/research-projects/ComboSelect.svelte`
- Test: create `src/lib/components/research-projects/OptionsPanel.test.ts`; rewrite `src/lib/components/research-projects/ComboSelect.test.ts`

**Interfaces:**
- Produces:
  - `ComboOption = { value: string; label: string; description?: string; mono?: string; dot?: string; group?: string }` (module-script ComboSelect, как раньше);
  - `OptionsPanel` пропсы `{ options: ComboOption[]; selectedValue?: string; placeholder: string; emptyLabel?: string; onSelect?: (option: ComboOption) => void }`;
  - `ComboSelect` пропсы `{ options; selectedValue?; placeholder: string; triggerIcon?: "lines" | "dot"; triggerFallback?: string; emptyLabel?; open?; onSelect? }` — триггер: иконка/точка + label выбранного (или `triggerFallback`) + caret, класс `tb-trigger`.

- [ ] **Step 1: Failing tests — OptionsPanel**

`src/lib/components/research-projects/OptionsPanel.test.ts`:

```ts
// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import OptionsPanel from "./OptionsPanel.svelte";
import type { ComboOption } from "./ComboSelect.svelte";

afterEach(cleanup);

const options: ComboOption[] = [
  { value: "gpt-4.1", label: "GPT-4.1", mono: "gpt-4.1", dot: "#10a37f", group: "OpenAI" },
  { value: "gpt-4o", label: "GPT-4o", mono: "gpt-4o", dot: "#10a37f", group: "OpenAI" },
  { value: "sonnet", label: "Claude Sonnet 4", mono: "claude-sonnet-4", dot: "#d97757", group: "Anthropic" },
  { value: "evidence", label: "Evidence brief", description: "Сводка с цитатами" },
];

describe("OptionsPanel", () => {
  it("renders options with group headings, dots, second lines and a check on the selected one", () => {
    render(OptionsPanel, {
      props: { options, selectedValue: "gpt-4o", placeholder: "Поиск модели…" },
    });
    expect(screen.getByText("OpenAI")).toBeTruthy();
    expect(screen.getByText("Anthropic")).toBeTruthy();
    expect(screen.getByText("claude-sonnet-4")).toBeTruthy();
    expect(screen.getByText("Сводка с цитатами")).toBeTruthy();
    const selected = screen.getByRole("option", { selected: true });
    expect(selected.textContent).toContain("GPT-4o");
    expect(selected.textContent).toContain("✓");
    expect(document.querySelectorAll(".options-panel__dot")).toHaveLength(3);
  });

  it("filters by label, description and mono; shows empty state", async () => {
    render(OptionsPanel, { props: { options, placeholder: "Поиск шаблона…" } });
    const input = screen.getByPlaceholderText("Поиск шаблона…");

    await fireEvent.input(input, { target: { value: "цитатами" } });
    expect(screen.getByText("Evidence brief")).toBeTruthy();
    expect(screen.queryByText("GPT-4.1")).toBeNull();

    await fireEvent.input(input, { target: { value: "claude-sonnet" } });
    expect(screen.getByText("Claude Sonnet 4")).toBeTruthy();

    await fireEvent.input(input, { target: { value: "нет-такого" } });
    expect(screen.getByText("Ничего не найдено")).toBeTruthy();
  });

  it("forwards selection", async () => {
    const onSelect = vi.fn();
    render(OptionsPanel, { props: { options, placeholder: "Поиск…", onSelect } });
    await fireEvent.click(screen.getByText("Evidence brief"));
    expect(onSelect).toHaveBeenCalledWith(options[3]);
  });
});
```

Run: FAIL (module not found).

- [ ] **Step 2: Implement OptionsPanel.svelte**

```svelte
<script lang="ts">
  import type { ComboOption } from "./ComboSelect.svelte";

  let {
    options,
    selectedValue,
    placeholder,
    emptyLabel = "Ничего не найдено",
    onSelect,
  }: {
    options: ComboOption[];
    selectedValue?: string;
    placeholder: string;
    emptyLabel?: string;
    onSelect?: (option: ComboOption) => void;
  } = $props();

  let query = $state("");

  let filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return options;
    return options.filter(
      (option) =>
        option.label.toLowerCase().includes(q) ||
        (option.description ?? "").toLowerCase().includes(q) ||
        (option.mono ?? "").toLowerCase().includes(q),
    );
  });
</script>

<div class="options-panel">
  <div class="options-panel__search">
    <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
      <circle cx="7" cy="7" r="4.5" />
      <path d="M10.5 10.5L14 14" />
    </svg>
    <input bind:value={query} {placeholder} aria-label={placeholder} />
  </div>
  <div class="options-panel__list" role="listbox">
    {#each filtered as option, index (option.value)}
      {#if option.group && (index === 0 || filtered[index - 1].group !== option.group)}
        <div class="options-panel__group" role="presentation">{option.group}</div>
      {/if}
      <button
        type="button"
        role="option"
        class="options-panel__item"
        aria-selected={option.value === selectedValue}
        onclick={() => onSelect?.(option)}
      >
        {#if option.dot}
          <span class="options-panel__dot" style:background={option.dot}></span>
        {/if}
        <span class="options-panel__body">
          <span class="options-panel__label">{option.label}</span>
          {#if option.description}
            <span class="options-panel__desc">{option.description}</span>
          {:else if option.mono}
            <span class="options-panel__mono">{option.mono}</span>
          {/if}
        </span>
        {#if option.value === selectedValue}
          <span class="options-panel__check">✓</span>
        {/if}
      </button>
    {/each}
    {#if filtered.length === 0}
      <div class="options-panel__empty">{emptyLabel}</div>
    {/if}
  </div>
</div>

<style>
  .options-panel {
    display: flex;
    flex-direction: column;
    width: 100%;
  }

  .options-panel__search {
    display: flex;
    align-items: center;
    gap: 7px;
    height: 32px;
    padding: 0 8px;
    border: 1px solid var(--extractum-border);
    border-radius: 6px;
    margin-bottom: 5px;
    background: var(--extractum-surface);
    color: var(--extractum-muted-2);
  }

  .options-panel__search input {
    border: none;
    outline: none;
    background: transparent;
    flex: 1;
    min-width: 0;
    font: 500 12.5px/1 var(--extractum-font);
    color: var(--extractum-text);
  }

  .options-panel__list {
    max-height: 248px;
    overflow-y: auto;
    overflow-x: hidden;
    display: flex;
    flex-direction: column;
  }

  .options-panel__group {
    font: 600 10px/1 var(--extractum-font);
    letter-spacing: 0.04em;
    color: var(--extractum-muted-2);
    text-transform: uppercase;
    padding: 8px 8px 4px;
  }

  .options-panel__list .options-panel__item {
    display: flex;
    align-items: flex-start;
    gap: 9px;
    padding: 8px;
    border: none;
    border-radius: 6px;
    background: transparent;
    text-align: left;
    cursor: pointer;
    color: var(--extractum-text);
  }

  .options-panel__list .options-panel__item:hover {
    background: var(--extractum-surface-subtle);
  }

  .options-panel__list .options-panel__item[aria-selected="true"] {
    background: color-mix(in srgb, var(--extractum-primary) 6%, transparent);
  }

  .options-panel__dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
    margin-top: 3px;
  }

  .options-panel__body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .options-panel__label {
    font: 600 12.5px/1.3 var(--extractum-font);
  }

  .options-panel__desc {
    font: 400 11px/1.3 var(--extractum-font);
    color: var(--extractum-muted-2);
  }

  .options-panel__mono {
    font: 500 10px/1.2 "SF Mono", Menlo, Consolas, monospace;
    color: var(--extractum-muted-2);
  }

  .options-panel__check {
    color: var(--extractum-primary);
    font-size: 12px;
    margin-top: 1px;
  }

  .options-panel__empty {
    padding: 18px 8px;
    text-align: center;
    font: 500 12px/1 var(--extractum-font);
    color: var(--extractum-muted-2);
  }
</style>
```

Run OptionsPanel tests: PASS.

- [ ] **Step 3: Rewrite ComboSelect.svelte**

```svelte
<script lang="ts" module>
  export type ComboOption = {
    value: string;
    label: string;
    description?: string;
    mono?: string;
    dot?: string;
    group?: string;
  };
</script>

<script lang="ts">
  import {
    ExtractumPopover,
    ExtractumPopoverTrigger,
    ExtractumPopoverContent,
  } from "$lib/components/extractum-ui";
  import OptionsPanel from "./OptionsPanel.svelte";

  let {
    options,
    selectedValue,
    placeholder,
    triggerIcon = "lines",
    triggerFallback,
    emptyLabel = "Ничего не найдено",
    open = $bindable(false),
    onSelect,
  }: {
    options: ComboOption[];
    selectedValue?: string;
    placeholder: string;
    triggerIcon?: "lines" | "dot";
    triggerFallback?: string;
    emptyLabel?: string;
    open?: boolean;
    onSelect?: (option: ComboOption) => void;
  } = $props();

  let selectedOption = $derived(options.find((option) => option.value === selectedValue));
  let triggerLabel = $derived(selectedOption?.label ?? triggerFallback ?? "—");

  function pick(option: ComboOption) {
    onSelect?.(option);
    open = false;
  }
</script>

<ExtractumPopover bind:open>
  <ExtractumPopoverTrigger class="tb-trigger combo-select__trigger">
    {#if triggerIcon === "dot"}
      <span
        class="combo-select__dot"
        style:background={selectedOption?.dot ?? "var(--extractum-muted)"}
      ></span>
    {:else}
      <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M3 3.5h10M3 7h10M3 10.5h6" />
      </svg>
    {/if}
    {triggerLabel}
    <span class="tb-caret">▾</span>
  </ExtractumPopoverTrigger>
  <ExtractumPopoverContent class="combo-select__content" align="end">
    <OptionsPanel {options} {selectedValue} {placeholder} {emptyLabel} onSelect={pick} />
  </ExtractumPopoverContent>
</ExtractumPopover>

<style>
  .combo-select__dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  :global(.combo-select__content) {
    width: 288px;
    padding: 6px;
  }
</style>
```

- [ ] **Step 4: Rewrite ComboSelect.test.ts** (Command больше нет — рендер-тесты)

```ts
// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import ComboSelect, { type ComboOption } from "./ComboSelect.svelte";

afterEach(cleanup);

const options: ComboOption[] = [
  { value: "p1", label: "Evidence brief", description: "Сводка с цитатами" },
  { value: "p2", label: "Risk monitor" },
];

describe("ComboSelect", () => {
  it("shows the selected label without a prefix and a caret on the trigger", () => {
    render(ComboSelect, {
      props: { options, selectedValue: "p1", placeholder: "Поиск шаблона…" },
    });
    const trigger = document.querySelector(".combo-select__trigger");
    expect(trigger?.textContent).toContain("Evidence brief");
    expect(trigger?.textContent).not.toContain("Промпт:");
    expect(trigger?.textContent).toContain("▾");
  });

  it("falls back to the placeholder label when nothing is selected", () => {
    render(ComboSelect, {
      props: { options, placeholder: "Поиск…", triggerFallback: "Промпт" },
    });
    expect(document.querySelector(".combo-select__trigger")?.textContent).toContain("Промпт");
  });

  it("opens the options panel and forwards selection", async () => {
    const onSelect = vi.fn();
    render(ComboSelect, {
      props: { options, selectedValue: "p1", placeholder: "Поиск шаблона…", open: true, onSelect },
    });
    await fireEvent.click(await screen.findByText("Risk monitor"));
    expect(onSelect).toHaveBeenCalledWith(options[1]);
  });
});
```

- [ ] **Step 5: Run + gates + commit**

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/OptionsPanel.test.ts src/lib/components/research-projects/ComboSelect.test.ts && npm.cmd run check`
Expected: PASS; check покажет ошибки в ProjectToolbar (старые пропсы `triggerPrefix`) — это чинится в Task 4; если check блокирует, временно допустимо передать `placeholder` из ProjectToolbar (минимальная правка `triggerPrefix`→`placeholder`+`triggerFallback`) — но НЕ коммитить till zero errors: правильный порядок — выполнить Task 4 Step 1 (минимальную правку вызовов) в этом же коммите, если check требует.

```bash
git add src/lib/components/research-projects/OptionsPanel.svelte src/lib/components/research-projects/OptionsPanel.test.ts src/lib/components/research-projects/ComboSelect.svelte src/lib/components/research-projects/ComboSelect.test.ts
git commit -m "feat(research-projects): OptionsPanel with search/groups and thin ComboSelect"
```

(Если пришлось поправить вызовы в ProjectToolbar — добавить его в коммит.)

---

### Task 3: PeriodPanel + новый PeriodPopover

**Files:**
- Create: `src/lib/components/research-projects/PeriodPanel.svelte`
- Rewrite: `src/lib/components/research-projects/PeriodPopover.svelte`
- Test: create `src/lib/components/research-projects/PeriodPanel.test.ts`; rewrite `src/lib/components/research-projects/PeriodPopover.test.ts`

**Interfaces:**
- Consumes: `formatPeriodDate`, `periodRangeLabel` (Task 1); `PeriodPreset`.
- Produces:
  - `PeriodPanel` пропсы `{ presets: PeriodPreset[]; selectedId?: string; dataRange: { from: number; to: number } | null; onSelect?: (preset: PeriodPreset) => void }`;
  - `PeriodPopover` пропсы `{ presets; selectedId?; triggerLabel: string; dataRange; open?; onSelect? }` — триггер: календарь-svg + label + caret, класс `tb-trigger`.

- [ ] **Step 1: Failing tests — PeriodPanel**

`src/lib/components/research-projects/PeriodPanel.test.ts`:

```ts
// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import PeriodPanel from "./PeriodPanel.svelte";
import type { PeriodPreset } from "$lib/ui/research-projects-period";

afterEach(cleanup);

const unix = (y: number, m: number, d: number) => new Date(y, m - 1, d, 12).getTime() / 1000;

const presets: PeriodPreset[] = [
  { id: "all", label: "Весь период", from: unix(2024, 3, 14), to: unix(2025, 5, 31) },
  { id: "year:2025", label: "2025", from: unix(2025, 1, 1), to: unix(2025, 5, 31) },
];

const dataRange = { from: unix(2024, 3, 14), to: unix(2025, 5, 31) };

describe("PeriodPanel", () => {
  it("shows the project data span and preset sub-ranges with a check on the selected one", () => {
    render(PeriodPanel, { props: { presets, selectedId: "all", dataRange } });
    expect(screen.getByText(/Данные проекта: 14\.03\.24 – 31\.05\.25/)).toBeTruthy();
    expect(screen.getByText("14.03.24 – 31.05.25")).toBeTruthy();
    expect(screen.getByText("01.01.25 – 31.05.25")).toBeTruthy();
    const selected = screen.getByRole("option", { selected: true });
    expect(selected.textContent).toContain("Весь период");
    expect(selected.textContent).toContain("✓");
  });

  it("selects a preset", async () => {
    const onSelect = vi.fn();
    render(PeriodPanel, { props: { presets, selectedId: "all", dataRange, onSelect } });
    await fireEvent.click(screen.getByText("2025"));
    expect(onSelect).toHaveBeenCalledWith(presets[1]);
  });

  it("applies a custom range as a synthetic preset (day bounds)", async () => {
    const onSelect = vi.fn();
    render(PeriodPanel, { props: { presets, selectedId: "all", dataRange, onSelect } });
    await fireEvent.input(screen.getByLabelText("Дата начала"), {
      target: { value: "2025-02-01" },
    });
    await fireEvent.input(screen.getByLabelText("Дата конца"), {
      target: { value: "2025-02-28" },
    });
    await fireEvent.click(screen.getByText("Применить диапазон"));
    const preset = onSelect.mock.calls[0][0] as PeriodPreset;
    expect(preset.id).toBe("custom");
    expect(preset.label).toBe("01.02.25–28.02.25");
    expect(preset.from).toBe(new Date("2025-02-01T00:00:00").getTime() / 1000);
    expect(preset.to).toBe(new Date("2025-02-28T00:00:00").getTime() / 1000 + 86_399);
  });

  it("disables apply on missing or inverted dates", async () => {
    render(PeriodPanel, { props: { presets, dataRange } });
    const apply = () => screen.getByText("Применить диапазон") as HTMLButtonElement;
    expect(apply().disabled).toBe(true);

    await fireEvent.input(screen.getByLabelText("Дата начала"), {
      target: { value: "2025-03-01" },
    });
    await fireEvent.input(screen.getByLabelText("Дата конца"), {
      target: { value: "2025-02-01" },
    });
    expect(apply().disabled).toBe(true);
  });
});
```

Run: FAIL (module not found).

- [ ] **Step 2: Implement PeriodPanel.svelte**

```svelte
<script lang="ts">
  import {
    formatPeriodDate,
    periodRangeLabel,
    type PeriodPreset,
  } from "$lib/ui/research-projects-period";

  let {
    presets,
    selectedId,
    dataRange,
    onSelect,
  }: {
    presets: PeriodPreset[];
    selectedId?: string;
    dataRange: { from: number; to: number } | null;
    onSelect?: (preset: PeriodPreset) => void;
  } = $props();

  let customFrom = $state("");
  let customTo = $state("");

  let applyDisabled = $derived(!customFrom || !customTo || customFrom > customTo);

  function dayStart(iso: string): number {
    return new Date(`${iso}T00:00:00`).getTime() / 1000;
  }

  function applyCustom() {
    if (applyDisabled) return;
    const from = dayStart(customFrom);
    const to = dayStart(customTo) + 86_399;
    onSelect?.({
      id: "custom",
      label: `${formatPeriodDate(from)}–${formatPeriodDate(to)}`,
      from,
      to,
    });
  }
</script>

<div class="period-panel">
  {#if dataRange}
    <div class="period-panel__span">
      <svg width="11" height="11" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="8" cy="8" r="6" />
        <path d="M8 5v3.2l2 1.3" />
      </svg>
      Данные проекта: {periodRangeLabel(dataRange.from, dataRange.to)}
    </div>
  {/if}
  <div role="listbox" class="period-panel__list">
    {#each presets as preset (preset.id)}
      <button
        type="button"
        role="option"
        class="period-panel__item"
        aria-selected={preset.id === selectedId}
        onclick={() => onSelect?.(preset)}
      >
        <span class="period-panel__item-body">
          <span class="period-panel__label">{preset.label}</span>
          <span class="period-panel__sub">{periodRangeLabel(preset.from, preset.to)}</span>
        </span>
        {#if preset.id === selectedId}
          <span class="period-panel__check">✓</span>
        {/if}
      </button>
    {/each}
  </div>
  <div class="period-panel__divider"></div>
  <div class="period-panel__custom">
    <div class="period-panel__custom-title">Произвольный диапазон</div>
    <div class="period-panel__dates">
      <input type="date" aria-label="Дата начала" bind:value={customFrom} />
      <span class="period-panel__arrow">→</span>
      <input type="date" aria-label="Дата конца" bind:value={customTo} />
    </div>
    <button
      type="button"
      class="period-panel__apply"
      disabled={applyDisabled}
      onclick={applyCustom}
    >
      Применить диапазон
    </button>
  </div>
</div>

<style>
  .period-panel {
    display: flex;
    flex-direction: column;
    width: 100%;
  }

  .period-panel__span {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px 7px;
    font: 500 10.5px/1.3 var(--extractum-font);
    color: var(--extractum-muted-2);
  }

  .period-panel__list {
    display: flex;
    flex-direction: column;
  }

  .period-panel__list .period-panel__item {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 8px;
    border: none;
    border-radius: 6px;
    background: transparent;
    text-align: left;
    cursor: pointer;
    color: var(--extractum-text);
  }

  .period-panel__list .period-panel__item:hover {
    background: var(--extractum-surface-subtle);
  }

  .period-panel__list .period-panel__item[aria-selected="true"] {
    background: color-mix(in srgb, var(--extractum-primary) 6%, transparent);
  }

  .period-panel__item-body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .period-panel__label {
    font: 600 12.5px/1.2 var(--extractum-font);
  }

  .period-panel__sub {
    font: 500 10.5px/1.2 "SF Mono", Menlo, Consolas, monospace;
    color: var(--extractum-muted-2);
  }

  .period-panel__check {
    color: var(--extractum-primary);
    font-size: 12px;
  }

  .period-panel__divider {
    height: 1px;
    background: var(--extractum-border);
    margin: 5px 2px;
  }

  .period-panel__custom {
    padding: 4px 8px 6px;
  }

  .period-panel__custom-title {
    font: 600 10px/1 var(--extractum-font);
    letter-spacing: 0.04em;
    color: var(--extractum-muted-2);
    text-transform: uppercase;
    margin-bottom: 7px;
  }

  .period-panel__dates {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .period-panel__dates input {
    height: 29px;
    flex: 1;
    min-width: 0;
    border: 1px solid var(--extractum-border);
    border-radius: 5px;
    padding: 0 6px;
    font: 500 11.5px var(--extractum-font);
    color: var(--extractum-text);
    background: var(--extractum-surface-raised);
  }

  .period-panel__arrow {
    color: var(--extractum-muted-2);
    font-size: 11px;
  }

  .period-panel__custom .period-panel__apply {
    margin-top: 8px;
    width: 100%;
    height: 30px;
    border: none;
    border-radius: 6px;
    background: color-mix(in srgb, var(--extractum-primary) 10%, transparent);
    color: var(--extractum-primary);
    font: 600 12px/1 var(--extractum-font);
    cursor: pointer;
  }

  .period-panel__custom .period-panel__apply:hover:not(:disabled) {
    background: color-mix(in srgb, var(--extractum-primary) 18%, transparent);
  }

  .period-panel__custom .period-panel__apply:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
```

Run PeriodPanel tests: PASS.

- [ ] **Step 3: Rewrite PeriodPopover.svelte**

```svelte
<script lang="ts">
  import {
    ExtractumPopover,
    ExtractumPopoverTrigger,
    ExtractumPopoverContent,
  } from "$lib/components/extractum-ui";
  import PeriodPanel from "./PeriodPanel.svelte";
  import type { PeriodPreset } from "$lib/ui/research-projects-period";

  let {
    presets,
    selectedId,
    triggerLabel,
    dataRange = null,
    open = $bindable(false),
    onSelect,
  }: {
    presets: PeriodPreset[];
    selectedId?: string;
    triggerLabel: string;
    dataRange?: { from: number; to: number } | null;
    open?: boolean;
    onSelect?: (preset: PeriodPreset) => void;
  } = $props();

  function pick(preset: PeriodPreset) {
    onSelect?.(preset);
    open = false;
  }
</script>

<ExtractumPopover bind:open>
  <ExtractumPopoverTrigger class="tb-trigger period-popover__trigger">
    <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
      <rect x="2.5" y="3.5" width="11" height="10" rx="1.5" />
      <path d="M2.5 6.5h11M5.5 2v3M10.5 2v3" />
    </svg>
    {triggerLabel}
    <span class="tb-caret">▾</span>
  </ExtractumPopoverTrigger>
  <ExtractumPopoverContent class="period-popover__content" align="end">
    <PeriodPanel {presets} {selectedId} {dataRange} onSelect={pick} />
  </ExtractumPopoverContent>
</ExtractumPopover>

<style>
  :global(.period-popover__content) {
    width: 290px;
    padding: 6px;
  }
</style>
```

- [ ] **Step 4: Rewrite PeriodPopover.test.ts**

```ts
// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import PeriodPopover from "./PeriodPopover.svelte";
import type { PeriodPreset } from "$lib/ui/research-projects-period";

afterEach(cleanup);

const unix = (y: number, m: number, d: number) => new Date(y, m - 1, d, 12).getTime() / 1000;
const presets: PeriodPreset[] = [
  { id: "all", label: "Весь период", from: unix(2024, 3, 14), to: unix(2025, 5, 31) },
  { id: "year:2025", label: "2025", from: unix(2025, 1, 1), to: unix(2025, 5, 31) },
];

describe("PeriodPopover", () => {
  it("renders a prefix-free trigger with a caret", () => {
    render(PeriodPopover, {
      props: { presets, selectedId: "all", triggerLabel: "Весь период" },
    });
    const trigger = document.querySelector(".period-popover__trigger");
    expect(trigger?.textContent).toContain("Весь период");
    expect(trigger?.textContent).not.toContain("Период:");
    expect(trigger?.textContent).toContain("▾");
  });

  it("opens the panel and forwards preset selection", async () => {
    const onSelect = vi.fn();
    render(PeriodPopover, {
      props: {
        presets,
        selectedId: "all",
        triggerLabel: "Весь период",
        dataRange: { from: presets[0].from, to: presets[0].to },
        open: true,
        onSelect,
      },
    });
    expect(await screen.findByText(/Данные проекта:/)).toBeTruthy();
    await fireEvent.click(screen.getByText("2025"));
    expect(onSelect).toHaveBeenCalledWith(presets[1]);
  });
});
```

- [ ] **Step 5: Run + commit**

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/PeriodPanel.test.ts src/lib/components/research-projects/PeriodPopover.test.ts`
Expected: PASS. (`npm.cmd run check` может падать на ProjectToolbar до Task 4 — допустимо на этом коммите ТОЛЬКО если Task 2 уже сделал минимальную правку; иначе поправить вызов PeriodPopover в ProjectToolbar (`selectedId`/`dataRange` пропсы) в этом же коммите.)

```bash
git add src/lib/components/research-projects/PeriodPanel.svelte src/lib/components/research-projects/PeriodPanel.test.ts src/lib/components/research-projects/PeriodPopover.svelte src/lib/components/research-projects/PeriodPopover.test.ts
git commit -m "feat(research-projects): PeriodPanel with data span and custom range"
```

---

### Task 4: ProjectToolbar — заголовок, триггеры, wide/narrow

**Files:**
- Rewrite: `src/lib/components/research-projects/ProjectToolbar.svelte`
- Test: rewrite `src/lib/components/research-projects/ProjectToolbar.test.ts`

**Interfaces:**
- Consumes: `PeriodPopover` (Task 3), `ComboSelect` (Task 2), `PeriodPanel`, `OptionsPanel` (narrow-стопка).
- Produces: пропсы ProjectToolbar:

```ts
{
  title: string;
  runLabel?: string;               // default "Запустить"
  runDisabled?: boolean;
  onRun?: () => void;
  periodPresets: PeriodPreset[];
  selectedPeriodId?: string;
  selectedPeriodLabel?: string;    // НОВОЕ: label выбранного периода (включая custom)
  dataRange?: { from: number; to: number } | null;  // НОВОЕ
  onSelectPeriod?: (preset: PeriodPreset) => void;
  promptOptions: ComboOption[];
  selectedPromptValue?: string;
  onSelectPrompt?: (option: ComboOption) => void;
  modelOptions: ComboOption[];
  selectedModelValue?: string;
  onSelectModel?: (option: ComboOption) => void;
}
```

- [ ] **Step 1: Rewrite tests (failing)**

`src/lib/components/research-projects/ProjectToolbar.test.ts`:

```ts
// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import ProjectToolbar from "./ProjectToolbar.svelte";
import rawSource from "./ProjectToolbar.svelte?raw";
import type { PeriodPreset } from "$lib/ui/research-projects-period";
import type { ComboOption } from "./ComboSelect.svelte";

afterEach(cleanup);

const source = rawSource.replace(/\r\n/g, "\n");
const unix = (y: number, m: number, d: number) => new Date(y, m - 1, d, 12).getTime() / 1000;

const periodPresets: PeriodPreset[] = [
  { id: "all", label: "Весь период", from: unix(2024, 3, 14), to: unix(2025, 5, 31) },
  { id: "year:2025", label: "2025", from: unix(2025, 1, 1), to: unix(2025, 5, 31) },
];
const promptOptions: ComboOption[] = [{ value: "p1", label: "Evidence brief" }];
const modelOptions: ComboOption[] = [{ value: "m1", label: "GPT-4.1", mono: "gpt-4.1", dot: "#10a37f" }];

const base = {
  title: "Беларусь: медиаполе 2025",
  periodPresets,
  selectedPeriodId: "all",
  selectedPeriodLabel: "Весь период",
  promptOptions,
  selectedPromptValue: "p1",
  modelOptions,
  selectedModelValue: "m1",
};

describe("ProjectToolbar", () => {
  it("renders the eyebrow, title and prefix-free triggers", () => {
    render(ProjectToolbar, { props: { ...base } });

    expect(screen.getByText("Research project")).toBeTruthy();
    expect(screen.getByText("Беларусь: медиаполе 2025")).toBeTruthy();
    expect(document.querySelector(".period-popover__trigger")?.textContent).toContain(
      "Весь период",
    );
    expect(screen.queryByText(/Период:/)).toBeNull();
    expect(screen.queryByText(/Промпт:/)).toBeNull();
    expect(screen.queryByText(/Модель:/)).toBeNull();
  });

  it("runs from the wide run button (narrow duplicate exists in DOM)", async () => {
    const onRun = vi.fn();
    render(ProjectToolbar, { props: { ...base, onRun } });

    // container queries не вычисляются в jsdom — обе кнопки в DOM
    const buttons = screen.getAllByRole("button", { name: "Запустить" });
    expect(buttons.length).toBeGreaterThanOrEqual(1);
    await fireEvent.click(buttons[0]);
    expect(onRun).toHaveBeenCalledTimes(1);
  });

  it("disables both run buttons when runDisabled", () => {
    render(ProjectToolbar, { props: { ...base, runDisabled: true } });
    for (const button of screen.getAllByRole("button", { name: "Запустить" })) {
      expect((button as HTMLButtonElement).disabled).toBe(true);
    }
  });

  it("collapses selectors into «Параметры» below 600px via a container query", () => {
    expect(source).toContain("container-type: inline-size");
    expect(source).toContain("@container tb (max-width: 600px)");
    expect(source).toContain("Параметры");
    expect(source).toContain("Параметры запуска");
  });

  it("highlights open triggers via bits-ui data-state", () => {
    expect(source).toContain('[data-state="open"]');
  });
});
```

Run: FAIL.

- [ ] **Step 2: Rewrite ProjectToolbar.svelte**

```svelte
<script lang="ts">
  import {
    ExtractumPopover,
    ExtractumPopoverTrigger,
    ExtractumPopoverContent,
  } from "$lib/components/extractum-ui";
  import PeriodPopover from "./PeriodPopover.svelte";
  import PeriodPanel from "./PeriodPanel.svelte";
  import ComboSelect, { type ComboOption } from "./ComboSelect.svelte";
  import OptionsPanel from "./OptionsPanel.svelte";
  import type { PeriodPreset } from "$lib/ui/research-projects-period";

  let {
    title,
    runLabel = "Запустить",
    runDisabled = false,
    onRun,
    periodPresets,
    selectedPeriodId,
    selectedPeriodLabel,
    dataRange = null,
    onSelectPeriod,
    promptOptions,
    selectedPromptValue,
    onSelectPrompt,
    modelOptions,
    selectedModelValue,
    onSelectModel,
  }: {
    title: string;
    runLabel?: string;
    runDisabled?: boolean;
    onRun?: () => void;
    periodPresets: PeriodPreset[];
    selectedPeriodId?: string;
    selectedPeriodLabel?: string;
    dataRange?: { from: number; to: number } | null;
    onSelectPeriod?: (preset: PeriodPreset) => void;
    promptOptions: ComboOption[];
    selectedPromptValue?: string;
    onSelectPrompt?: (option: ComboOption) => void;
    modelOptions: ComboOption[];
    selectedModelValue?: string;
    onSelectModel?: (option: ComboOption) => void;
  } = $props();

  let periodLabel = $derived(selectedPeriodLabel ?? "Период");
  let promptLabel = $derived(
    promptOptions.find((option) => option.value === selectedPromptValue)?.label ?? "Промпт",
  );
  let selectedModel = $derived(modelOptions.find((option) => option.value === selectedModelValue));
  let modelLabel = $derived(selectedModel?.label ?? "Модель");

  let paramsOpen = $state(false);
  let narrowSection = $state<"period" | "prompt" | "model" | null>(null);

  function toggleSection(section: "period" | "prompt" | "model") {
    narrowSection = narrowSection === section ? null : section;
  }

  function narrowSelectPeriod(preset: PeriodPreset) {
    onSelectPeriod?.(preset);
    narrowSection = null;
  }

  function narrowSelectPrompt(option: ComboOption) {
    onSelectPrompt?.(option);
    narrowSection = null;
  }

  function narrowSelectModel(option: ComboOption) {
    onSelectModel?.(option);
    narrowSection = null;
  }
</script>

<div class="project-toolbar">
  <div class="project-toolbar__heading">
    <span class="project-toolbar__eyebrow">Research project</span>
    <strong class="project-toolbar__title">{title}</strong>
  </div>

  <div class="project-toolbar__wide">
    <PeriodPopover
      presets={periodPresets}
      selectedId={selectedPeriodId}
      triggerLabel={periodLabel}
      {dataRange}
      onSelect={onSelectPeriod}
    />
    <ComboSelect
      options={promptOptions}
      selectedValue={selectedPromptValue}
      placeholder="Поиск шаблона…"
      triggerFallback="Промпт"
      onSelect={onSelectPrompt}
    />
    <ComboSelect
      options={modelOptions}
      selectedValue={selectedModelValue}
      placeholder="Поиск модели…"
      triggerIcon="dot"
      triggerFallback="Модель"
      onSelect={onSelectModel}
    />
    <button
      class="project-toolbar__run"
      type="button"
      disabled={runDisabled}
      onclick={() => onRun?.()}
    >
      <svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor">
        <path d="M4 2.5v11l9-5.5z" />
      </svg>
      {runLabel}
    </button>
  </div>

  <div class="project-toolbar__narrow">
    <ExtractumPopover bind:open={paramsOpen}>
      <ExtractumPopoverTrigger class="tb-trigger project-toolbar__params-trigger">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M2 4.5h7M12 4.5h2M2 11.5h2M7 11.5h7" />
          <circle cx="10.5" cy="4.5" r="1.6" />
          <circle cx="5.5" cy="11.5" r="1.6" />
        </svg>
        Параметры
        <span class="tb-caret">▾</span>
      </ExtractumPopoverTrigger>
      <ExtractumPopoverContent class="project-toolbar__params" align="end">
        <div class="project-toolbar__params-title">Параметры запуска</div>

        <button
          type="button"
          class="project-toolbar__section"
          onclick={() => toggleSection("period")}
        >
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="2.5" y="3.5" width="11" height="10" rx="1.5" />
            <path d="M2.5 6.5h11M5.5 2v3M10.5 2v3" />
          </svg>
          <span class="project-toolbar__section-name">Период</span>
          <span class="project-toolbar__section-value">{periodLabel}</span>
          <span class="tb-caret" data-open={narrowSection === "period"}>▾</span>
        </button>
        {#if narrowSection === "period"}
          <div class="project-toolbar__section-body">
            <PeriodPanel
              presets={periodPresets}
              selectedId={selectedPeriodId}
              {dataRange}
              onSelect={narrowSelectPeriod}
            />
          </div>
        {/if}

        <div class="project-toolbar__params-divider"></div>

        <button
          type="button"
          class="project-toolbar__section"
          onclick={() => toggleSection("prompt")}
        >
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M3 3.5h10M3 7h10M3 10.5h6" />
          </svg>
          <span class="project-toolbar__section-name">Промпт</span>
          <span class="project-toolbar__section-value">{promptLabel}</span>
          <span class="tb-caret" data-open={narrowSection === "prompt"}>▾</span>
        </button>
        {#if narrowSection === "prompt"}
          <div class="project-toolbar__section-body">
            <OptionsPanel
              options={promptOptions}
              selectedValue={selectedPromptValue}
              placeholder="Поиск шаблона…"
              onSelect={narrowSelectPrompt}
            />
          </div>
        {/if}

        <div class="project-toolbar__params-divider"></div>

        <button
          type="button"
          class="project-toolbar__section"
          onclick={() => toggleSection("model")}
        >
          <span
            class="project-toolbar__section-dot"
            style:background={selectedModel?.dot ?? "var(--extractum-muted)"}
          ></span>
          <span class="project-toolbar__section-name">Модель</span>
          <span class="project-toolbar__section-value">{modelLabel}</span>
          <span class="tb-caret" data-open={narrowSection === "model"}>▾</span>
        </button>
        {#if narrowSection === "model"}
          <div class="project-toolbar__section-body">
            <OptionsPanel
              options={modelOptions}
              selectedValue={selectedModelValue}
              placeholder="Поиск модели…"
              onSelect={narrowSelectModel}
            />
          </div>
        {/if}
      </ExtractumPopoverContent>
    </ExtractumPopover>

    <button
      class="project-toolbar__run project-toolbar__run--square"
      type="button"
      title={runLabel}
      aria-label={runLabel}
      disabled={runDisabled}
      onclick={() => onRun?.()}
    >
      <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
        <path d="M4 2.5v11l9-5.5z" />
      </svg>
    </button>
  </div>
</div>

<style>
  .project-toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 54px;
    padding: 8px 14px;
    background: var(--extractum-surface-raised);
    border-bottom: 1px solid var(--extractum-border);
    container-type: inline-size;
    container-name: tb;
  }

  .project-toolbar__heading {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
    flex: 1;
  }

  .project-toolbar__eyebrow {
    font: 600 10px/1 var(--extractum-font);
    letter-spacing: 0.05em;
    color: var(--extractum-muted-2);
    text-transform: uppercase;
  }

  .project-toolbar__title {
    font: 600 15px/1.2 var(--extractum-font);
    color: var(--extractum-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .project-toolbar__wide {
    display: flex;
    align-items: center;
    gap: 7px;
    justify-content: flex-end;
    flex-shrink: 0;
  }

  .project-toolbar__narrow {
    display: none;
    align-items: center;
    gap: 7px;
    justify-content: flex-end;
    flex-shrink: 0;
  }

  @container tb (max-width: 600px) {
    .project-toolbar__wide {
      display: none;
    }
    .project-toolbar__narrow {
      display: flex;
    }
  }

  /* Общий вид триггеров (scoped override глобального button-правила) +
     open-состояние по bits-ui data-state. */
  .project-toolbar :global(.tb-trigger) {
    height: 32px;
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 0 10px;
    border: 1px solid var(--extractum-border);
    border-radius: 6px;
    background: var(--extractum-surface-raised);
    font: 500 12.5px/1 var(--extractum-font);
    color: var(--extractum-text);
    cursor: pointer;
    white-space: nowrap;
  }

  .project-toolbar :global(.tb-trigger:hover) {
    border-color: var(--extractum-border-strong, var(--extractum-border));
  }

  .project-toolbar :global(.tb-trigger svg) {
    color: var(--extractum-muted-2);
  }

  .project-toolbar :global(.tb-caret) {
    color: var(--extractum-muted-2);
    font-size: 10px;
    transition: transform 0.12s ease;
  }

  .project-toolbar :global([data-slot="popover-trigger"][data-state="open"]) {
    border-color: var(--extractum-primary);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--extractum-primary) 12%, transparent);
  }

  .project-toolbar :global([data-slot="popover-trigger"][data-state="open"] .tb-caret) {
    transform: rotate(180deg);
  }

  .project-toolbar :global(.tb-caret[data-open="true"]) {
    transform: rotate(180deg);
  }

  .project-toolbar__wide .project-toolbar__run,
  .project-toolbar__narrow .project-toolbar__run {
    height: 32px;
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 0 14px;
    border: none;
    border-radius: 6px;
    background: var(--extractum-primary);
    color: #fff;
    font: 600 13px/1 var(--extractum-font);
    cursor: pointer;
    box-shadow: 0 1px 2px color-mix(in srgb, var(--extractum-primary) 30%, transparent);
  }

  .project-toolbar__run--square {
    width: 32px;
    padding: 0;
    justify-content: center;
    flex-shrink: 0;
  }

  .project-toolbar__wide .project-toolbar__run:hover:not(:disabled),
  .project-toolbar__narrow .project-toolbar__run:hover:not(:disabled) {
    background: var(--extractum-primary-hover);
  }

  .project-toolbar__wide .project-toolbar__run:disabled,
  .project-toolbar__narrow .project-toolbar__run:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  :global(.project-toolbar__params) {
    width: 296px;
    padding: 6px;
  }

  .project-toolbar__params-title {
    font: 700 11px/1 var(--extractum-font);
    letter-spacing: 0.04em;
    color: var(--extractum-muted);
    text-transform: uppercase;
    padding: 6px 8px 8px;
  }

  :global(.project-toolbar__params) .project-toolbar__section {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    padding: 9px 8px;
    border: none;
    border-radius: 7px;
    background: transparent;
    cursor: pointer;
    text-align: left;
    color: var(--extractum-text);
  }

  :global(.project-toolbar__params) .project-toolbar__section:hover {
    background: var(--extractum-surface-subtle);
  }

  .project-toolbar__section-name {
    flex: 1;
    font: 600 12.5px/1 var(--extractum-font);
  }

  .project-toolbar__section-value {
    font: 500 11px/1 var(--extractum-font);
    color: var(--extractum-muted-2);
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .project-toolbar__section-dot {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    flex-shrink: 0;
    margin: 0 2px;
  }

  .project-toolbar__section-body {
    padding: 2px 4px 8px;
  }

  .project-toolbar__params-divider {
    height: 1px;
    background: var(--extractum-border);
    margin: 3px 6px;
  }
</style>
```

- [ ] **Step 3: Run tests + gates**

Run: `node scripts/run-vitest.mjs run src/lib/components/research-projects/ProjectToolbar.test.ts && npm.cmd run check`
Expected: PASS; check 0 ошибок (страница может ругаться на отсутствующие пропсы — они опциональные; если TS требует `selectedPeriodLabel` — он опционален, ок).

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/research-projects/ProjectToolbar.svelte src/lib/components/research-projects/ProjectToolbar.test.ts
git commit -m "feat(research-projects): v10 toolbar with iconed triggers and narrow params mode"
```

---

### Task 5: Проводка страницы + живая проверка

**Files:**
- Modify: `src/routes/projects/next/+page.svelte`

**Interfaces:**
- Consumes: новые пропсы ProjectToolbar (`selectedPeriodLabel`, `dataRange`), синтетический custom-пресет.

- [ ] **Step 1: Page wiring**

1. Состояние (рядом с `selectedPeriodId`):

```ts
  let customPeriod = $state<PeriodPreset | null>(null);
```

2. `selectedPeriod` — заменить на:

```ts
  let selectedPeriod = $derived<PeriodPreset | undefined>(
    selectedPeriodId === "custom"
      ? (customPeriod ?? undefined)
      : periodPresets.find((preset) => preset.id === selectedPeriodId),
  );
```

3. В `selectProject` добавить `customPeriod = null;`.

4. В `toolbar`-бэге:
- заменить `onSelectPeriod: (preset) => (selectedPeriodId = preset.id),` на

```ts
          onSelectPeriod: (preset) => {
            if (preset.id === "custom") customPeriod = preset;
            selectedPeriodId = preset.id;
          },
```

- добавить `selectedPeriodLabel: selectedPeriod?.label,` и `dataRange: railState.dataRange,`.

- [ ] **Step 2: Gates**

Run: `node scripts/run-vitest.mjs run src/lib/research-projects-import-boundary.test.ts && npm.cmd run check`
Expected: PASS; 0 ошибок.

- [ ] **Step 3: Live verification in Tauri**

`/projects/next`, Project 2:
1. Заголовок: eyebrow «RESEARCH PROJECT» + полное название (не «Proj…»).
2. Триггеры: календарь+«Весь период»+▾; строки+«Default report»+▾; точка+«Модель»+▾; при открытии — синяя рамка+ring, caret перевёрнут; открыт максимум один.
3. Период-поповер: «Данные проекта: …», поддиапазоны пресетов, ✓; произвольный диапазон: выбрать 2 даты → «Применить диапазон» → триггер показывает «DD.MM.YY–DD.MM.YY»; инспектор «Период» отражает кастом.
4. Промпт-поповер: поиск фильтрует; выбор меняет триггер.
5. Модель-поповер: пустой список → «Ничего не найдено» (моделей нет — ок).
6. Адаптив: уменьшить окно (или открыть инспектор при узком окне), пока main-колонка <600px: селекторы → «Параметры» + квадратный play; открыть «Параметры»: стопка секций, раскрытие Период/Промпт, выбор синхронен с wide-режимом (расширить окно — выбранное на месте).
7. Кнопка «Запустить»: play-иконка; runDisabled — работает.

Скриншоты: wide-тулбар с открытым периодом; narrow-«Параметры» со стопкой.

- [ ] **Step 4: Commit**

```bash
git add src/routes/projects/next/+page.svelte
git commit -m "feat(research-projects): wire custom period and data range into the toolbar"
```

---

## Self-Review Notes

- **Spec coverage:** хелперы дат (T1), OptionsPanel поиск/группы/точки/desc/mono/пусто (T2), ComboOption расширен + тонкий ComboSelect с иконкой/фолбэком (T2), PeriodPanel span/sub/custom/валидация + тонкий PeriodPopover (T3), тулбар: eyebrow/заголовок flex, wide-ряд, run с play, narrow «Параметры» с аккордеоном на тех же панелях, container query, open-подсветка по data-state (T4), страница: customPeriod/selectedPeriodLabel/dataRange (T5), живая матрица + адаптив (T5).
- **Type consistency:** `ComboOption` (T2) потребляется T4/T5; `PeriodPreset` c custom (T3→T5); `selectedPeriodLabel?: string`, `dataRange?: {from;to}|null` (T4→T5); `tb-trigger`/`tb-caret` классы разделяют стили (T2/T3/T4).
- **Известные допущения:** в jsdom narrow-разметка всегда в DOM (дубли run-кнопок — тесты используют `getAllByRole`); PeriodPopover в jsdom-тестах страниц не участвует; «Данные проекта» скрывается без dataRange.
