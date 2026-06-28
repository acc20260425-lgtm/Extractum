<script lang="ts">
  import * as Popover from "$lib/components/ui/popover";
  import type { PeriodPreset } from "$lib/ui/research-projects-period";

  let {
    presets,
    selectedId,
    triggerLabel,
    open = $bindable(false),
    onSelect,
  }: {
    presets: PeriodPreset[];
    selectedId?: string;
    triggerLabel: string;
    open?: boolean;
    onSelect?: (preset: PeriodPreset) => void;
  } = $props();

  function pick(preset: PeriodPreset) {
    onSelect?.(preset);
    open = false;
  }
</script>

<Popover.Root bind:open>
  <Popover.Trigger class="period-popover__trigger">Период: {triggerLabel}</Popover.Trigger>
  <Popover.Content class="period-popover__content" align="start">
    <ul class="period-popover__list">
      {#each presets as preset (preset.id)}
        <li>
          <button
            type="button"
            class="period-popover__item"
            data-selected={preset.id === selectedId}
            onclick={() => pick(preset)}
          >
            {preset.label}
          </button>
        </li>
      {/each}
    </ul>
  </Popover.Content>
</Popover.Root>

<style>
  .period-popover__list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
  }

  :global(.period-popover__item) {
    width: 100%;
    padding: 6px 10px;
    border: none;
    border-radius: 5px;
    background: transparent;
    text-align: left;
    font: 400 12.5px/1.2 var(--extractum-font);
    color: var(--extractum-text);
    cursor: pointer;
  }

  :global(.period-popover__item:hover) {
    background: var(--extractum-surface-subtle);
  }

  :global(.period-popover__item[data-selected="true"]) {
    color: var(--extractum-primary);
    font-weight: 600;
  }
</style>
