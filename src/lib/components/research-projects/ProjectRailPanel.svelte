<script lang="ts">
  import { List, Plus, RefreshCw, Search, X } from "@lucide/svelte";
  import {
    ExtractumButton,
    ExtractumDialog,
    ExtractumDropdownMenu,
    ExtractumDropdownMenuTrigger,
    ExtractumDropdownMenuContent,
    ExtractumDropdownMenuItem,
    ExtractumDropdownMenuSeparator,
  } from "$lib/components/extractum-ui";
  import ProjectRow from "./ProjectRow.svelte";
  import {
    buildProjectRailRow,
    filterProjectRail,
    groupProjectRail,
    projectRailRowMatches,
  } from "$lib/ui/research-projects-rail";
  import type { ProjectSummary } from "$lib/types/projects";

  let {
    summaries,
    selectedProjectId,
    now,
    onSelect,
    onCreate,
    onEdit,
    onTogglePin,
    onToggleArchive,
    onDelete,
  }: {
    summaries: ProjectSummary[];
    selectedProjectId: number | null;
    now: number;
    onSelect?: (id: number) => void;
    onCreate?: () => void;
    onEdit?: (id: number) => void;
    onTogglePin?: (id: number, pinned: boolean) => void;
    onToggleArchive?: (id: number, archived: boolean) => void;
    onDelete?: (id: number) => void;
  } = $props();

  let query = $state("");
  let compact = $state(false);
  let archiveOpen = $state(false);
  let headerMenuOpen = $state(false);
  let confirmOpen = $state(false);
  let pendingDelete = $state<{ id: number; name: string } | null>(null);

  let selected = $derived(summaries.find((s) => s.id === selectedProjectId) ?? null);
  let activeRow = $derived(selected ? buildProjectRailRow(selected, now) : null);
  let sections = $derived(
    groupProjectRail(
      summaries.filter((s) => s.id !== selectedProjectId),
      now,
    ),
  );
  let filtered = $derived(filterProjectRail(sections, query));
  let activeVisible = $derived(activeRow !== null && projectRailRowMatches(activeRow, query));
  let archiveCount = $derived(sections.archived.length);
  let noProjects = $derived(
    query.trim() !== "" &&
      !activeVisible &&
      filtered.pinned.length === 0 &&
      filtered.normal.length === 0 &&
      filtered.archived.length === 0,
  );

  function requestDelete(id: number, name: string) {
    pendingDelete = { id, name };
    confirmOpen = true;
  }

  function confirmDelete() {
    const target = pendingDelete;
    confirmOpen = false;
    pendingDelete = null;
    if (target) onDelete?.(target.id);
  }
</script>

<div class="rail-panel">
  <div class="rail-panel__header">
    <span class="rail-panel__title">Проекты</span>
    <div class="rail-panel__actions">
      <button
        type="button"
        class="rail-panel__icon-btn"
        data-ui-action="toggle-project-compact"
        title={compact ? "Комфортный вид" : "Компактный вид"}
        aria-label={compact ? "Комфортный вид" : "Компактный вид"}
        aria-pressed={compact}
        onclick={() => (compact = !compact)}
      >
        <List size={13} aria-hidden="true" />
      </button>
      <button
        type="button"
        class="rail-panel__icon-btn"
        data-ui-action="create-project"
        title="Создать проект"
        aria-label="Создать проект"
        onclick={() => onCreate?.()}
      >
        <Plus size={14} aria-hidden="true" />
      </button>
      <button
        type="button"
        class="rail-panel__icon-btn"
        data-ui-action="sync-projects"
        title="Скоро"
        aria-label="Синхронизация скоро"
        aria-disabled="true"
        disabled
      >
        <RefreshCw size={13} aria-hidden="true" />
      </button>
      {#if selected}
        <ExtractumDropdownMenu bind:open={headerMenuOpen}>
          <ExtractumDropdownMenuTrigger
            class="rail-panel__menu-trigger"
            data-ui-action="selected-project-actions"
            title="Действия с проектом"
            aria-label="Действия выбранного проекта"
          >
            <span class="rail-panel__more-dots" aria-hidden="true">⋯</span>
          </ExtractumDropdownMenuTrigger>
          <ExtractumDropdownMenuContent align="end">
            <ExtractumDropdownMenuItem onclick={() => selected && onEdit?.(selected.id)}>
              Редактировать
            </ExtractumDropdownMenuItem>
            <ExtractumDropdownMenuItem disabled>
              <span title="Скоро">Экспорт</span>
            </ExtractumDropdownMenuItem>
            <ExtractumDropdownMenuSeparator />
            <ExtractumDropdownMenuItem
              class="project-row-menu__danger"
              onclick={() => selected && requestDelete(selected.id, selected.name)}
            >
              Удалить
            </ExtractumDropdownMenuItem>
          </ExtractumDropdownMenuContent>
        </ExtractumDropdownMenu>
      {/if}
    </div>
  </div>

  <div class="rail-panel__search">
    <Search size={13} aria-hidden="true" />
    <input bind:value={query} placeholder="Поиск проектов" aria-label="Поиск проектов" />
    {#if query.length > 0}
      <button
        type="button"
        class="rail-panel__clear"
        data-ui-action="clear-project-search"
        title="Очистить"
        aria-label="Очистить поиск проектов"
        onclick={() => (query = "")}
      >
        <X size={13} aria-hidden="true" />
      </button>
    {/if}
  </div>

  <div class="rail-panel__list" role="listbox" aria-label="Проекты">
    {#if activeVisible || filtered.pinned.length > 0}
      <div class="rail-panel__section-header">
        <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor">
          <path
            d="M9.5 1.5l5 5-2 .6-2.3 2.3.5 3.2-1.8-1.8-3.3 3.3-.5.5.5-3.8L1.9 8.7l3.2.5L7.4 6.9 8 4.9z"
          />
        </svg>
        <span>Закреплённые</span>
      </div>
    {/if}
    {#if activeRow && activeVisible}
      <ProjectRow
        row={activeRow}
        variant="active"
        {compact}
        {onSelect}
        {onEdit}
        {onTogglePin}
        {onToggleArchive}
        onRequestDelete={requestDelete}
      />
    {/if}
    {#each filtered.pinned as row (row.id)}
      <ProjectRow
        {row}
        {compact}
        {onSelect}
        {onEdit}
        {onTogglePin}
        {onToggleArchive}
        onRequestDelete={requestDelete}
      />
    {/each}

    {#if filtered.normal.length > 0}
      <div class="rail-panel__section-header rail-panel__section-header--plain">Проекты</div>
    {/if}
    {#each filtered.normal as row (row.id)}
      <ProjectRow
        {row}
        {compact}
        {onSelect}
        {onEdit}
        {onTogglePin}
        {onToggleArchive}
        onRequestDelete={requestDelete}
      />
    {/each}

    {#if noProjects}
      <div class="rail-panel__empty">Проекты не найдены</div>
    {/if}

    {#if filtered.archived.length > 0}
      <button
        type="button"
        class="rail-panel__archive-toggle"
        data-ui-action="toggle-project-archive"
        aria-expanded={archiveOpen}
        onclick={() => (archiveOpen = !archiveOpen)}
      >
        <svg
          width="13"
          height="13"
          viewBox="0 0 16 16"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          style:transform={archiveOpen ? "rotate(90deg)" : "rotate(0deg)"}
        >
          <path d="M6 3.5L10.5 8 6 12.5" />
        </svg>
        <span class="rail-panel__archive-label">Архив</span>
        <span class="rail-panel__archive-count">{archiveCount}</span>
      </button>
      {#if archiveOpen}
        {#each filtered.archived as row (row.id)}
          <ProjectRow
            {row}
            variant="archived"
            {compact}
            {onSelect}
            {onEdit}
            {onTogglePin}
            {onToggleArchive}
            onRequestDelete={requestDelete}
          />
        {/each}
      {/if}
    {/if}
  </div>
</div>

<ExtractumDialog bind:open={confirmOpen} title="Удалить проект">
  <div class="rail-panel__confirm">
    <p>Удалить проект «{pendingDelete?.name ?? ""}»? Действие необратимо.</p>
    <footer>
      <ExtractumButton type="button" variant="outline" onclick={() => (confirmOpen = false)}>
        Отмена
      </ExtractumButton>
      <ExtractumButton type="button" variant="destructive" onclick={confirmDelete}>
        Да, удалить
      </ExtractumButton>
    </footer>
  </div>
</ExtractumDialog>

<style>
  .rail-panel {
    display: flex;
    flex-direction: column;
    min-height: 0;
    height: 100%;
  }

  .rail-panel__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 12px 8px;
  }

  .rail-panel__title {
    font: 700 11px/1 var(--extractum-font);
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--extractum-muted);
  }

  .rail-panel__actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  /* Перебивает глобальное button:not([data-slot="button"]) */
  .rail-panel__actions .rail-panel__icon-btn,
  .rail-panel__actions :global(.rail-panel__menu-trigger) {
    width: 24px;
    height: 24px;
    padding: 0;
    border: 1px solid var(--extractum-border);
    border-radius: 6px;
    background: var(--extractum-surface);
    color: var(--extractum-muted);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 15px;
    line-height: 1;
    cursor: pointer;
    flex-shrink: 0;
  }

  .rail-panel__actions .rail-panel__icon-btn:hover:not(:disabled),
  .rail-panel__actions :global(.rail-panel__menu-trigger:hover) {
    background: var(--extractum-surface-subtle);
  }

  .rail-panel__actions .rail-panel__icon-btn[aria-pressed="true"],
  .rail-panel__actions :global(.rail-panel__menu-trigger[data-state="open"]) {
    border-color: var(--extractum-primary);
    background: color-mix(in srgb, var(--extractum-primary) 10%, var(--extractum-surface));
    color: var(--extractum-primary);
  }

  .rail-panel__actions .rail-panel__icon-btn:focus-visible,
  .rail-panel__actions :global(.rail-panel__menu-trigger:focus-visible),
  .rail-panel__search .rail-panel__clear:focus-visible,
  .rail-panel__list .rail-panel__archive-toggle:focus-visible {
    outline: 2px solid var(--extractum-primary);
    outline-offset: 2px;
  }

  .rail-panel__actions .rail-panel__icon-btn:disabled {
    opacity: 0.45;
    cursor: default;
  }

  .rail-panel__actions :global(.rail-panel__more-dots) {
    display: inline-block;
    transform: translateY(-1px);
    font-size: 16px;
    font-weight: 700;
    line-height: 1;
  }

  .rail-panel__search {
    display: flex;
    align-items: center;
    gap: 7px;
    height: 30px;
    margin: 0 10px 8px;
    border: 1px solid var(--extractum-border);
    border-radius: 6px;
    background: var(--extractum-surface);
    padding: 0 9px;
    color: var(--extractum-muted-2);
    flex-shrink: 0;
  }

  .rail-panel__search input {
    flex: 1;
    min-width: 0;
    border: none;
    outline: none;
    background: transparent;
    font: 400 12.5px/1 var(--extractum-font);
    color: var(--extractum-text);
  }

  .rail-panel__search .rail-panel__clear {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border: none;
    border-radius: 4px;
    background: transparent;
    padding: 0;
    color: var(--extractum-muted-2);
    line-height: 1;
    cursor: pointer;
  }

  .rail-panel__search .rail-panel__clear:hover {
    background: var(--extractum-surface-subtle);
    color: var(--extractum-text);
  }

  .rail-panel__list {
    flex: 1;
    overflow: auto;
    padding: 2px 8px 8px;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .rail-panel__section-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 8px 4px;
    font: 700 10px/1 var(--extractum-font);
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--extractum-muted-2);
  }

  .rail-panel__section-header--plain {
    padding-top: 12px;
  }

  .rail-panel__empty {
    padding: 22px 12px;
    text-align: center;
    font: 400 12px/1.5 var(--extractum-font);
    color: var(--extractum-muted-2);
  }

  .rail-panel__list .rail-panel__archive-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 9px 10px;
    margin-top: 6px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--extractum-muted-2);
    cursor: pointer;
    text-align: left;
  }

  .rail-panel__list .rail-panel__archive-toggle:hover {
    background: var(--extractum-surface-subtle);
  }

  .rail-panel__archive-toggle svg {
    transition: transform 0.15s ease;
  }

  .rail-panel__archive-label {
    flex: 1;
    font: 600 11px/1 var(--extractum-font);
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .rail-panel__archive-count {
    font: 600 11px/1 var(--extractum-font);
  }

  .rail-panel__confirm {
    display: flex;
    min-width: min(420px, calc(100vw - 96px));
    flex-direction: column;
    gap: 16px;
    padding: 16px;
  }

  .rail-panel__confirm footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
