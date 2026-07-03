<script lang="ts">
  import {
    ExtractumDropdownMenu,
    ExtractumDropdownMenuTrigger,
    ExtractumDropdownMenuContent,
    ExtractumDropdownMenuItem,
    ExtractumDropdownMenuSeparator,
  } from "$lib/components/extractum-ui";
  import type { ProjectRailRow } from "$lib/ui/research-projects-rail";

  let {
    row,
    variant = "normal",
    compact = false,
    onSelect,
    onEdit,
    onTogglePin,
    onToggleArchive,
    onRequestDelete,
  }: {
    row: ProjectRailRow;
    variant?: "active" | "normal" | "archived";
    compact?: boolean;
    onSelect?: (id: number) => void;
    onEdit?: (id: number) => void;
    onTogglePin?: (id: number, pinned: boolean) => void;
    onToggleArchive?: (id: number, archived: boolean) => void;
    onRequestDelete?: (id: number, name: string) => void;
  } = $props();

  let menuOpen = $state(false);

  function openMenu(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    menuOpen = true;
  }
</script>

<div
  class="project-row"
  data-variant={variant}
  data-status={row.status}
  role="button"
  tabindex="0"
  title={compact ? `${row.name} — ${row.meta}` : undefined}
  onclick={() => onSelect?.(row.id)}
  onkeydown={(e) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onSelect?.(row.id);
    }
  }}
  oncontextmenu={openMenu}
>
  {#if variant === "active"}
    <span class="project-row__active-bar"></span>
  {/if}
  <span class="project-row__dot" data-testid="project-row-status-dot" data-status={row.status}
  ></span>
  <span class="project-row__body">
    <span class="project-row__name">{row.name}</span>
    {#if !compact}
      <span class="project-row__meta">{row.meta}</span>
    {/if}
  </span>
  <span class="project-row__actions" data-pinned={row.pinned || variant === "active"}>
    {#if row.pinned || variant === "active"}
      <span class="project-row__pin" title="Закреплён" aria-label="Закреплён">
        <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor">
          <path
            d="M9.5 1.5l5 5-2 .6-2.3 2.3.5 3.2-1.8-1.8-3.3 3.3-.5.5.5-3.8L1.9 8.7l3.2.5L7.4 6.9 8 4.9z"
          />
        </svg>
      </span>
    {/if}
    <ExtractumDropdownMenu bind:open={menuOpen}>
      <ExtractumDropdownMenuTrigger
        class="project-row__menu-trigger"
        title="Действия"
        onclick={(e: MouseEvent) => e.stopPropagation()}
      >
        ⋯
      </ExtractumDropdownMenuTrigger>
      <ExtractumDropdownMenuContent class="project-row-menu" align="end">
        {#if variant === "archived"}
          <ExtractumDropdownMenuItem onclick={() => onToggleArchive?.(row.id, false)}>
            Из архива
          </ExtractumDropdownMenuItem>
        {:else}
          <ExtractumDropdownMenuItem onclick={() => onEdit?.(row.id)}>
            Редактировать
          </ExtractumDropdownMenuItem>
          <ExtractumDropdownMenuItem onclick={() => onTogglePin?.(row.id, !row.pinned)}>
            {row.pinned ? "Открепить" : "Закрепить"}
          </ExtractumDropdownMenuItem>
          <ExtractumDropdownMenuItem disabled>
            <span title="Скоро">Синхронизировать</span>
          </ExtractumDropdownMenuItem>
          <ExtractumDropdownMenuItem disabled>
            <span title="Скоро">Экспорт</span>
          </ExtractumDropdownMenuItem>
          <ExtractumDropdownMenuItem onclick={() => onToggleArchive?.(row.id, true)}>
            В архив
          </ExtractumDropdownMenuItem>
        {/if}
        <ExtractumDropdownMenuSeparator />
        <ExtractumDropdownMenuItem
          class="project-row-menu__danger"
          onclick={() => onRequestDelete?.(row.id, row.name)}
        >
          Удалить
        </ExtractumDropdownMenuItem>
      </ExtractumDropdownMenuContent>
    </ExtractumDropdownMenu>
  </span>
</div>

<style>
  .project-row {
    position: relative;
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 7px 10px;
    border-radius: 6px;
    background: transparent;
    text-align: left;
    cursor: pointer;
  }

  .project-row[data-variant="archived"] {
    padding: 6px 10px;
  }

  .project-row:hover {
    background: var(--extractum-surface-subtle);
  }

  .project-row[data-variant="active"],
  .project-row[data-variant="active"]:hover {
    background: color-mix(in srgb, var(--extractum-primary) 10%, transparent);
    cursor: default;
  }

  .project-row__active-bar {
    position: absolute;
    left: 3px;
    top: 50%;
    transform: translateY(-50%);
    width: 3px;
    height: 22px;
    border-radius: 3px;
    background: var(--extractum-primary);
  }

  .project-row__dot {
    flex-shrink: 0;
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--extractum-muted);
  }

  .project-row__dot[data-status="ready"] {
    background: var(--extractum-success);
  }

  .project-row__dot[data-status="running"] {
    background: var(--extractum-primary);
  }

  .project-row__dot[data-status="needs_attention"] {
    background: var(--extractum-danger);
  }

  .project-row__dot[data-status="empty"] {
    background: var(--extractum-muted-2);
  }

  .project-row__body {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }

  .project-row__name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font: 400 13px/1.3 var(--extractum-font);
    color: var(--extractum-text);
  }

  .project-row[data-variant="active"] .project-row__name {
    font-weight: 600;
    color: var(--extractum-primary);
  }

  .project-row[data-variant="archived"] .project-row__name {
    color: var(--extractum-muted);
  }

  .project-row__meta {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font: 400 11px/1.3 var(--extractum-font);
    color: var(--extractum-muted);
  }

  .project-row[data-variant="archived"] .project-row__meta {
    color: var(--extractum-muted-2);
  }

  .project-row__actions {
    position: relative;
    flex-shrink: 0;
    width: 22px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .project-row__pin {
    position: absolute;
    display: inline-flex;
    color: var(--extractum-muted-2);
    transition: opacity 0.12s ease;
  }

  .project-row[data-variant="active"] .project-row__pin {
    color: var(--extractum-primary);
  }

  /* hover-своп: пин прячется, «⋯» появляется */
  .project-row:hover .project-row__pin {
    opacity: 0;
  }

  .project-row__actions :global(.project-row__menu-trigger) {
    position: absolute;
    width: 22px;
    height: 22px;
    border: none;
    border-radius: 5px;
    background: transparent;
    color: var(--extractum-muted);
    cursor: pointer;
    font-size: 14px;
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    transition: opacity 0.12s ease;
    padding: 0;
  }

  .project-row:hover .project-row__actions :global(.project-row__menu-trigger),
  .project-row__actions :global(.project-row__menu-trigger[data-state="open"]) {
    opacity: 1;
  }

  .project-row__actions :global(.project-row__menu-trigger:hover) {
    background: var(--extractum-surface-subtle);
  }

  :global(.project-row-menu__danger) {
    color: var(--extractum-danger) !important;
  }
</style>
