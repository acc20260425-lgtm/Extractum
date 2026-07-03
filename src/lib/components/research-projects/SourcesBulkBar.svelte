<script lang="ts">
  import { ExtractumButton, ExtractumDialog } from "$lib/components/extractum-ui";

  let {
    count,
    syncDisabled = false,
    syncTitle = "",
    onClear = () => {},
    onSync = () => {},
    onDelete = () => {},
  }: {
    count: number;
    syncDisabled?: boolean;
    syncTitle?: string;
    onClear?: () => void;
    onSync?: () => void;
    onDelete?: () => void;
  } = $props();

  let confirmOpen = $state(false);

  function confirmDelete() {
    confirmOpen = false;
    onDelete();
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

<style>
  .sources-bulk-bar {
    position: absolute;
    inset: 0;
    z-index: 5;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 14px;
    background: color-mix(in srgb, var(--extractum-primary) 8%, var(--extractum-surface));
    border-bottom: 1px solid color-mix(in srgb, var(--extractum-primary) 24%, transparent);
    font: 400 13px/1.4 var(--extractum-font);
    color: var(--extractum-text);
  }

  .sources-bulk-bar__count {
    font-weight: 600;
    color: var(--extractum-primary);
  }

  .sources-bulk-bar__clear {
    background: none;
    border: none;
    padding: 0;
    color: var(--extractum-primary);
    cursor: pointer;
    font: inherit;
    text-decoration: underline;
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
