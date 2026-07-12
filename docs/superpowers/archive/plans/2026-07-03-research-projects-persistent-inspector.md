# Постоянный инспектор + рабочий футер — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Колонка инспектора видна всегда при выбранном проекте (сворачивается в 44px-рейл), кнопки футера «Синхронизировать»/«Отключить» работают.

**Architecture:** Меняется только `src/routes/projects/next/+page.svelte`: `inspector`-бэг передаётся при выбранном проекте (не только при активном источнике), добавляются `syncDisabled`/`onSync`/`onDisconnect` и диалог подтверждения отключения. `Inspector.svelte` уже реализует v10 полностью.

**Tech Stack:** Svelte 5 runes, ExtractumDialog, Tauri MCP.

**Source spec:** `docs/superpowers/specs/2026-07-03-research-projects-persistent-inspector-design.md`

## Global Constraints

- **`Inspector.svelte`, shell, обёртки — НЕ менять.**
- **API (verbatim):** `syncYoutubeSource(sourceId: number, { metadata: true, transcripts: true, comments: false })` из `$lib/api/source-jobs` (уже импортирован на странице); `removeProjectSources({ projectId, sourceIds: number[] })` из `$lib/api/projects` (уже импортирован).
- **Syncable-предикат:** `provider === "youtube" && source_subtype ∈ {"video","playlist"}`.
- **Ошибки** → `railState.status` формата `Не удалось … (${String(error)})`; `railState.saving` на время операций.
- **Гейты:** `node scripts/run-vitest.mjs run src/lib/research-projects-import-boundary.test.ts` + `npm.cmd run check` (baseline 0 ошибок, 2 warning в ProjectsShell.svelte).
- **Живая проверка:** Tauri MCP 9223.
- **Коммит на задачу; push не делать.** Ветка `feat/research-projects-persistent-inspector` от main; план — первым коммитом.

---

### Task 1: Проводка страницы

**Files:**
- Modify: `src/routes/projects/next/+page.svelte`

**Interfaces:**
- Consumes: пропсы `Inspector` (`open`, `selected: InspectorSource | null`, `periodLabel`, `promptLabel`, `modelLabel`, `syncDisabled?`, `onToggle?`, `onSync?`, `onDisconnect?`); `ExtractumButton`, `ExtractumDialog` из extractum-ui.

- [ ] **Step 1: Derived activeSyncable**

После `sectionPlaceholder` добавить:

```ts
  let activeSyncable = $derived.by(() => {
    if (!activeSourceId) return false;
    const record = sources.find((source) => String(source.source_id) === activeSourceId);
    return (
      !!record &&
      record.provider === "youtube" &&
      (record.source_subtype === "video" || record.source_subtype === "playlist")
    );
  });
```

- [ ] **Step 2: Состояние диалога отключения**

Рядом с `connectOpen`:

```ts
  let disconnectOpen = $state(false);
```

- [ ] **Step 3: Обработчики**

После `connectSelectedLibrarySources`:

```ts
  async function syncActiveSource() {
    if (activeSourceId === null || selectedProjectId === null || !activeSyncable) return;
    railState = { ...railState, saving: true, status: "" };
    try {
      await syncYoutubeSource(Number(activeSourceId), {
        metadata: true,
        transcripts: true,
        comments: false,
      });
      sources = await listProjectSources(selectedProjectId);
    } catch (error) {
      railState = { ...railState, status: `Не удалось синхронизировать источник (${String(error)})` };
    } finally {
      railState = { ...railState, saving: false };
    }
  }

  async function disconnectActiveSource() {
    if (activeSourceId === null || selectedProjectId === null) return;
    const id = activeSourceId;
    disconnectOpen = false;
    railState = { ...railState, saving: true, status: "" };
    try {
      await removeProjectSources({ projectId: selectedProjectId, sourceIds: [Number(id)] });
      activeSourceId = null;
      selectedSourceIds = selectedSourceIds.filter((selected) => selected !== id);
      sources = await listProjectSources(selectedProjectId);
      await workflow.reload();
    } catch (error) {
      railState = { ...railState, status: `Не удалось отключить источник (${String(error)})` };
    } finally {
      railState = { ...railState, saving: false };
    }
  }
```

- [ ] **Step 4: inspector-бэг — всегда при выбранном проекте**

Заменить текущее

```svelte
    inspector={inspectorSource
      ? {
          open: inspectorOpen,
          selected: inspectorSource,
          periodLabel: selectedPeriod?.label ?? "—",
          promptLabel: selectedPromptLabel,
          modelLabel: selectedModelValue ?? "—",
          onToggle: () => (inspectorOpen = !inspectorOpen),
        }
      : undefined}
```

на

```svelte
    inspector={selectedProject
      ? {
          open: inspectorOpen,
          selected: inspectorSource,
          periodLabel: selectedPeriod?.label ?? "—",
          promptLabel: selectedPromptLabel,
          modelLabel: selectedModelValue ?? "—",
          syncDisabled: railState.saving || !activeSyncable,
          onToggle: () => (inspectorOpen = !inspectorOpen),
          onSync: () => void syncActiveSource(),
          onDisconnect: () => (disconnectOpen = true),
        }
      : undefined}
```

- [ ] **Step 5: Диалог подтверждения**

Импорт: `import { ExtractumButton, ExtractumDialog } from "$lib/components/extractum-ui";`

После `<ConnectFromLibrary ... />` добавить:

```svelte
  <ExtractumDialog bind:open={disconnectOpen} title="Отключить источник">
    <div class="disconnect-confirm">
      <p>
        Отключить источник «{selectedSourceRow?.title ?? ""}» от проекта? Материалы останутся в
        библиотеке.
      </p>
      <footer>
        <ExtractumButton type="button" variant="outline" onclick={() => (disconnectOpen = false)}>
          Отмена
        </ExtractumButton>
        <ExtractumButton
          type="button"
          variant="destructive"
          onclick={() => void disconnectActiveSource()}
        >
          Да, отключить
        </ExtractumButton>
      </footer>
    </div>
  </ExtractumDialog>
```

и в `<style>` страницы:

```css
  .disconnect-confirm {
    display: flex;
    min-width: min(420px, calc(100vw - 96px));
    flex-direction: column;
    gap: 16px;
    padding: 16px;
  }

  .disconnect-confirm footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
```

- [ ] **Step 6: Gates**

Run: `node scripts/run-vitest.mjs run src/lib/research-projects-import-boundary.test.ts && npm.cmd run check`
Expected: PASS; 0 ошибок.

- [ ] **Step 7: Commit** (сначала `git checkout -b feat/research-projects-persistent-inspector`; план — отдельным коммитом `docs: add persistent inspector implementation plan`)

```bash
git add src/routes/projects/next/+page.svelte
git commit -m "feat(research-projects): persistent inspector with working sync/disconnect footer"
```

---

### Task 2: Живая проверка

- [ ] **Step 1:** `/projects/next`, Project 2:
  1. Без активного источника → колонка инспектора видна («Конфигурация проекта», футера нет).
  2. «Свернуть» → 44px-рейл с вертикальной подписью «Инспектор»; «Развернуть» — обратно; свёрнутое состояние переживает клики по строкам.
  3. Клик по youtube-video строке → футер: «Синхронизировать» активна.
  4. «Отключить» → диалог «Отключить источник «…»?»; «Отмена» ничего не делает; «Да, отключить» → источник исчез из таблицы, инспектор без источника, счётчики рейла обновились. Восстановить источник через «Добавить источник» (ConnectFromLibrary).
  5. «Синхронизировать» (на восстановленном/другом источнике) — только проверить, что кнопка активна и обработчик привязан (реальный сетевой sync не запускать без необходимости — поведение API проверено в bulk-итерации; если запуск произошёл — это не ошибка).
  6. Чекбокс-выделение и сортировка не затронуты (bulk-бар и маркеры живы).
- [ ] **Step 2:** Скриншоты: колонка без источника; свёрнутый 44px-рейл; диалог отключения.

---

## Self-Review Notes

- **Spec coverage:** постоянный бэг (T1 Step 4), activeSyncable/syncDisabled (Step 1/4), onSync (Step 3), onDisconnect + диалог + очистка active/selected + reload (Steps 2/3/5), живая матрица (T2). Компонент/shell не тронуты.
- **Type consistency:** `activeSourceId: string | null` → `Number(...)` для API; `selectedSourceRow?.title` для текста диалога; `inspectorSource: InspectorSource | null` — тип пропа допускает null.
