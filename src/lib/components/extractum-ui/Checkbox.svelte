<script lang="ts">
  import { Checkbox } from "$lib/components/ui/checkbox/index.js";
  import { cn } from "$lib/utils.js";
  import type { ComponentProps } from "svelte";

  let {
    ref = $bindable(null),
    checked = $bindable(false),
    indeterminate = $bindable(false),
    class: className,
    ...rest
  }: ComponentProps<typeof Checkbox> = $props();
</script>

<Checkbox bind:ref bind:checked bind:indeterminate class={cn("extractum-checkbox", className)} {...rest} />
