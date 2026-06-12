<script lang="ts">
  import { Checkbox } from "$lib/components/ui/checkbox/index.js";
  import { cn } from "$lib/utils.js";
  import type { ComponentProps } from "svelte";

  let { class: className, ...rest }: ComponentProps<typeof Checkbox> = $props();
</script>

<Checkbox class={cn("extractum-checkbox", className)} {...rest} />
