<script lang="ts">
  let {
    title,
    runLabel = "Запустить анализ",
    periodLabel,
    promptLabel,
    modelLabel,
    runDisabled = false,
    onRun,
    onPeriodClick,
    onPromptClick,
    onModelClick,
  }: {
    title: string;
    runLabel?: string;
    periodLabel: string;
    promptLabel: string;
    modelLabel: string;
    runDisabled?: boolean;
    onRun?: () => void;
    onPeriodClick?: () => void;
    onPromptClick?: () => void;
    onModelClick?: () => void;
  } = $props();
</script>

<div class="project-toolbar">
  <span class="project-toolbar__title">{title}</span>

  <div class="project-toolbar__selectors">
    <button class="project-toolbar__trigger" type="button" onclick={() => onPeriodClick?.()}>
      Период: {periodLabel}
    </button>
    <button class="project-toolbar__trigger" type="button" onclick={() => onPromptClick?.()}>
      Промпт: {promptLabel}
    </button>
    <button class="project-toolbar__trigger" type="button" onclick={() => onModelClick?.()}>
      Модель: {modelLabel}
    </button>
  </div>

  <button
    class="project-toolbar__run"
    type="button"
    disabled={runDisabled}
    onclick={() => onRun?.()}
  >
    {runLabel}
  </button>
</div>

<style>
  .project-toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 54px;
    padding: 0 16px;
    background: var(--extractum-surface);
    border-bottom: 1px solid var(--extractum-border);
  }

  .project-toolbar__title {
    font: 600 14px/1.2 var(--extractum-font);
    color: var(--extractum-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .project-toolbar__selectors {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-left: auto;
  }

  .project-toolbar__trigger {
    height: 32px;
    padding: 0 11px;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface-raised);
    color: var(--extractum-text);
    font: 500 12px/1 var(--extractum-font);
    cursor: pointer;
  }

  .project-toolbar__trigger:hover {
    background: var(--extractum-surface-subtle);
  }

  .project-toolbar__run {
    height: 32px;
    padding: 0 14px;
    border: 1px solid var(--extractum-primary);
    border-radius: var(--extractum-radius);
    background: var(--extractum-primary);
    color: #ffffff;
    font: 600 12.5px/1 var(--extractum-font);
    cursor: pointer;
  }

  .project-toolbar__run:hover:not(:disabled) {
    background: var(--extractum-primary-hover);
    border-color: var(--extractum-primary-hover);
  }

  .project-toolbar__run:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
