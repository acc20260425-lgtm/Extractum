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
    ariaLabel,
    dataRange = null,
    open = $bindable(false),
    onSelect,
  }: {
    presets: PeriodPreset[];
    selectedId?: string;
    triggerLabel: string;
    ariaLabel?: string;
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
  <ExtractumPopoverTrigger
    class="tb-trigger period-popover__trigger"
    role="combobox"
    aria-label={ariaLabel}
    aria-expanded={open}
  >
    <svg
      width="13"
      height="13"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
    >
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
