<script lang="ts">
  import { Select } from "$lib/components/ui/select/index.js";
  import { cn } from "$lib/utils.js";
  import type { ComponentProps } from "svelte";

  let { class: className, ...rest }: ComponentProps<typeof Select> & { class?: string } = $props();
</script>

<div class={cn("extractum-select", className)}>
  <Select {...rest} />
</div>
