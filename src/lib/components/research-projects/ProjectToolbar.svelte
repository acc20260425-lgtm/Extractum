<script lang="ts">
  import PeriodPopover from "./PeriodPopover.svelte";
  import ComboSelect, { type ComboOption } from "./ComboSelect.svelte";
  import type { PeriodPreset } from "$lib/ui/research-projects-period";

  let {
    title,
    runLabel = "Запустить анализ",
    runDisabled = false,
    onRun,
    periodPresets,
    selectedPeriodId,
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
    onSelectPeriod?: (preset: PeriodPreset) => void;
    promptOptions: ComboOption[];
    selectedPromptValue?: string;
    onSelectPrompt?: (option: ComboOption) => void;
    modelOptions: ComboOption[];
    selectedModelValue?: string;
    onSelectModel?: (option: ComboOption) => void;
  } = $props();

  let periodLabel = $derived(
    periodPresets.find((preset) => preset.id === selectedPeriodId)?.label ?? "Период",
  );
</script>

<div class="project-toolbar">
  <span class="project-toolbar__title">{title}</span>

  <div class="project-toolbar__selectors">
    <PeriodPopover
      presets={periodPresets}
      selectedId={selectedPeriodId}
      triggerLabel={periodLabel}
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

  /* Neutral outline triggers for the popover/combobox selectors. Scoped under
     .project-toolbar to outrank the app's global `button:not([data-slot=button])`
     primary styling. */
  .project-toolbar :global(.period-popover__trigger),
  .project-toolbar :global(.combo-select__trigger) {
    height: 32px;
    padding: 0 11px;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface-raised);
    color: var(--extractum-text);
    font: 500 12px/1 var(--extractum-font);
    cursor: pointer;
  }

  .project-toolbar :global(.period-popover__trigger:hover),
  .project-toolbar :global(.combo-select__trigger:hover) {
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
