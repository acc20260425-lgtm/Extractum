<script lang="ts" module>
  export type ComboOption = {
    value: string;
    label: string;
    description?: string;
    mono?: string;
    dot?: string;
    group?: string;
  };
</script>

<script lang="ts">
  import {
    ExtractumPopover,
    ExtractumPopoverTrigger,
    ExtractumPopoverContent,
  } from "$lib/components/extractum-ui";
  import OptionsPanel from "./OptionsPanel.svelte";

  let {
    options,
    selectedValue,
    placeholder,
    triggerIcon = "lines",
    triggerFallback,
    emptyLabel = "Ничего не найдено",
    open = $bindable(false),
    onSelect,
  }: {
    options: ComboOption[];
    selectedValue?: string;
    placeholder: string;
    triggerIcon?: "lines" | "dot";
    triggerFallback?: string;
    emptyLabel?: string;
    open?: boolean;
    onSelect?: (option: ComboOption) => void;
  } = $props();

  let selectedOption = $derived(options.find((option) => option.value === selectedValue));
  let triggerLabel = $derived(selectedOption?.label ?? triggerFallback ?? "—");

  function pick(option: ComboOption) {
    onSelect?.(option);
    open = false;
  }
</script>

<ExtractumPopover bind:open>
  <ExtractumPopoverTrigger class="tb-trigger combo-select__trigger">
    {#if triggerIcon === "dot"}
      <span
        class="combo-select__dot"
        style:background={selectedOption?.dot ?? "var(--extractum-muted)"}
      ></span>
    {:else}
      <svg
        width="13"
        height="13"
        viewBox="0 0 16 16"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
      >
        <path d="M3 3.5h10M3 7h10M3 10.5h6" />
      </svg>
    {/if}
    {triggerLabel}
    <span class="tb-caret">▾</span>
  </ExtractumPopoverTrigger>
  <ExtractumPopoverContent class="combo-select__content" align="end">
    <OptionsPanel {options} {selectedValue} {placeholder} {emptyLabel} onSelect={pick} />
  </ExtractumPopoverContent>
</ExtractumPopover>

<style>
  .combo-select__dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  :global(.combo-select__content) {
    width: 288px;
    padding: 6px;
  }
</style>
