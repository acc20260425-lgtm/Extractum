<script lang="ts" module>
  export type ComboOption = { value: string; label: string };
</script>

<script lang="ts">
  import {
    ExtractumPopover,
    ExtractumPopoverTrigger,
    ExtractumPopoverContent,
    ExtractumCommand,
    ExtractumCommandInput,
    ExtractumCommandList,
    ExtractumCommandItem,
    ExtractumCommandEmpty,
  } from "$lib/components/extractum-ui";

  let {
    options,
    selectedValue,
    triggerPrefix,
    placeholder = "Поиск",
    emptyLabel = "Ничего не найдено",
    open = $bindable(false),
    onSelect,
  }: {
    options: ComboOption[];
    selectedValue?: string;
    triggerPrefix: string;
    placeholder?: string;
    emptyLabel?: string;
    open?: boolean;
    onSelect?: (option: ComboOption) => void;
  } = $props();

  let selectedLabel = $derived(
    options.find((option) => option.value === selectedValue)?.label ?? "—",
  );

  function pick(option: ComboOption) {
    onSelect?.(option);
    open = false;
  }
</script>

<ExtractumPopover bind:open>
  <ExtractumPopoverTrigger class="combo-select__trigger">{triggerPrefix}: {selectedLabel}</ExtractumPopoverTrigger>
  <ExtractumPopoverContent class="combo-select__content" align="start">
    <ExtractumCommand value={selectedValue}>
      <ExtractumCommandInput {placeholder} />
      <ExtractumCommandList>
        <ExtractumCommandEmpty>{emptyLabel}</ExtractumCommandEmpty>
        {#each options as option (option.value)}
          <ExtractumCommandItem
            value={option.value}
            keywords={[option.label]}
            onSelect={() => pick(option)}
          >
            {option.label}
          </ExtractumCommandItem>
        {/each}
      </ExtractumCommandList>
    </ExtractumCommand>
  </ExtractumPopoverContent>
</ExtractumPopover>
