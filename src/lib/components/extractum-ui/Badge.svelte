<script lang="ts">
  import { Badge } from "$lib/components/ui/badge/index.js";
  import { cn } from "$lib/utils.js";
  import type { ComponentProps } from "svelte";

  let { class: className, ...rest }: ComponentProps<typeof Badge> = $props();
</script>

<Badge
  class={cn("extractum-badge rounded-[4px] px-1.5 py-0 text-[11px] font-medium", className)}
  {...rest}
/>
