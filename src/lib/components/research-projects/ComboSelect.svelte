<script lang="ts" module>
  export type ComboOption = { value: string; label: string };
</script>

<script lang="ts">
  import * as Popover from "$lib/components/ui/popover";
  import * as Command from "$lib/components/ui/command";

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

<Popover.Root bind:open>
  <Popover.Trigger class="combo-select__trigger">{triggerPrefix}: {selectedLabel}</Popover.Trigger>
  <Popover.Content class="combo-select__content" align="start">
    <Command.Root value={selectedValue}>
      <Command.Input {placeholder} />
      <Command.List>
        <Command.Empty>{emptyLabel}</Command.Empty>
        {#each options as option (option.value)}
          <Command.Item value={option.value} keywords={[option.label]} onSelect={() => pick(option)}>
            {option.label}
          </Command.Item>
        {/each}
      </Command.List>
    </Command.Root>
  </Popover.Content>
</Popover.Root>
