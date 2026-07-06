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
  let wideOpen = $state<"period" | "prompt" | "model" | null>(null);

  function setWideOpen(selector: "period" | "prompt" | "model", open: boolean) {
    if (open) {
      wideOpen = selector;
    } else if (wideOpen === selector) {
      wideOpen = null;
    }
  }

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
      ariaLabel="Период"
      {dataRange}
      bind:open={() => wideOpen === "period", (open) => setWideOpen("period", open)}
      onSelect={onSelectPeriod}
    />
    <ComboSelect
      options={promptOptions}
      selectedValue={selectedPromptValue}
      placeholder="Поиск шаблона…"
      ariaLabel="Промпт"
      triggerFallback="Промпт"
      bind:open={() => wideOpen === "prompt", (open) => setWideOpen("prompt", open)}
      onSelect={onSelectPrompt}
    />
    <ComboSelect
      options={modelOptions}
      selectedValue={selectedModelValue}
      placeholder="Поиск модели…"
      ariaLabel="Модель"
      triggerIcon="dot"
      triggerFallback="Модель"
      bind:open={() => wideOpen === "model", (open) => setWideOpen("model", open)}
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
        <svg
          width="14"
          height="14"
          viewBox="0 0 16 16"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
        >
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
          <svg
            width="14"
            height="14"
            viewBox="0 0 16 16"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
          >
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
          <svg
            width="14"
            height="14"
            viewBox="0 0 16 16"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
          >
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
     open-состояние по data-state поповер-триггера. */
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
