<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import { cn } from "$lib/utils.js";
  import type { ComponentProps } from "svelte";

  let {
    class: className,
    density = "compact",
    ...rest
  }: ComponentProps<typeof Button> & { density?: "compact" | "normal" } = $props();
</script>

<Button
  class={cn(
    "extractum-button",
    density === "compact" &&
      "h-[var(--extractum-density-control-height)] rounded-[var(--extractum-radius)] px-2 text-[13px]",
    className,
  )}
  {...rest}
/>
