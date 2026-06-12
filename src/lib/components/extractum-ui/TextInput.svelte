<script lang="ts">
  import { Input } from "$lib/components/ui/input/index.js";
  import { cn } from "$lib/utils.js";
  import type { ComponentProps } from "svelte";

  let { class: className, ...rest }: ComponentProps<typeof Input> = $props();
</script>

<Input
  class={cn("extractum-input h-[32px] rounded-[var(--extractum-radius)] text-[13px]", className)}
  {...rest}
/>
