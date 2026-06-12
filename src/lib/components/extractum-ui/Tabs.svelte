<script lang="ts">
  import { Tabs } from "$lib/components/ui/tabs/index.js";
  import { cn } from "$lib/utils.js";
  import type { ComponentProps } from "svelte";

  let { class: className, ...rest }: ComponentProps<typeof Tabs> = $props();
</script>

<Tabs class={cn("extractum-tabs", className)} {...rest} />
