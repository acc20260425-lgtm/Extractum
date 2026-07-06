<script lang="ts">
  import { ExtractumButton, ExtractumDialog } from "$lib/components/extractum-ui";
  import { PROJECT_YOUTUBE_VIDEO_LIBRARY_DELETE_CONFIRM } from "$lib/ui/research-projects-model";

  let {
    count,
    syncDisabled = false,
    syncTitle = "",
    libraryDeleteDisabled = true,
    libraryDeleteTitle = "",
    onClear = () => {},
    onSync = () => {},
    onDeleteFromLibrary = () => {},
    onDelete = () => {},
  }: {
    count: number;
    syncDisabled?: boolean;
    syncTitle?: string;
    libraryDeleteDisabled?: boolean;
    libraryDeleteTitle?: string;
    onClear?: () => void;
    onSync?: () => void;
    onDeleteFromLibrary?: () => void;
    onDelete?: () => void;
  } = $props();

  let confirmOpen = $state(false);
  let libraryDeleteConfirmOpen = $state(false);

  function confirmDelete() {
    confirmOpen = false;
    onDelete();
  }

  function confirmDeleteFromLibrary() {
    libraryDeleteConfirmOpen = false;
    onDeleteFromLibrary();
  }
</script>

<div class="sources-bulk-bar" role="region" aria-label="Массовые действия">
  <span class="sources-bulk-bar__count">Выбрано: {count}</span>
  <button type="button" class="sources-bulk-bar__clear" onclick={() => onClear()}>
    Снять выделение
  </button>
  <div class="sources-bulk-bar__spacer"></div>
  <ExtractumButton
    variant="outline"
    disabled={syncDisabled}
    title={syncDisabled ? syncTitle : ""}
    onclick={() => onSync()}
  >
    Синхронизировать
  </ExtractumButton>
  <ExtractumButton
    variant="destructive"
    disabled={libraryDeleteDisabled}
    title={libraryDeleteDisabled ? libraryDeleteTitle : ""}
    onclick={() => (libraryDeleteConfirmOpen = true)}
  >
    Delete from Library
  </ExtractumButton>
  <ExtractumButton variant="destructive" onclick={() => (confirmOpen = true)}>
    Удалить
  </ExtractumButton>
</div>

<ExtractumDialog bind:open={confirmOpen} title="Удалить источники">
  <div class="sources-bulk-bar__confirm">
    <p>
      Удалить {count} источник(ов) из проекта? Материалы останутся в библиотеке —
      удаляется только связь с проектом.
    </p>
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

<ExtractumDialog bind:open={libraryDeleteConfirmOpen} title="Delete from Library">
  <div class="sources-bulk-bar__confirm">
    <p>
      {PROJECT_YOUTUBE_VIDEO_LIBRARY_DELETE_CONFIRM}
    </p>
    <footer>
      <ExtractumButton
        type="button"
        variant="outline"
        onclick={() => (libraryDeleteConfirmOpen = false)}
      >
        Cancel Library deletion
      </ExtractumButton>
      <ExtractumButton type="button" variant="destructive" onclick={confirmDeleteFromLibrary}>
        Delete from Library permanently
      </ExtractumButton>
    </footer>
  </div>
</ExtractumDialog>

<style>
  .sources-bulk-bar {
    flex-shrink: 0;
    min-height: 42px;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 14px;
    background: color-mix(in srgb, var(--extractum-primary) 10%, var(--extractum-surface-raised));
    border-bottom: 1px solid color-mix(in srgb, var(--extractum-primary) 28%, transparent);
    font: 400 12.5px/1.35 var(--extractum-font);
    color: var(--extractum-text);
  }

  .sources-bulk-bar__count {
    font-weight: 700;
    color: var(--extractum-primary);
  }

  .sources-bulk-bar__clear {
    background: none;
    border: none;
    padding: 0;
    color: var(--extractum-primary);
    cursor: pointer;
    font: 600 12px/1 var(--extractum-font);
    text-decoration: underline;
    text-underline-offset: 2px;
  }

  .sources-bulk-bar__spacer {
    flex: 1;
  }

  .sources-bulk-bar__confirm {
    display: flex;
    min-width: min(420px, calc(100vw - 96px));
    flex-direction: column;
    gap: 16px;
    padding: 16px;
  }

  .sources-bulk-bar__confirm footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
