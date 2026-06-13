<script lang="ts">
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import { cn } from "$lib/utils.js";
  import type { Snippet } from "svelte";

  let {
    tone = "default",
    size = "md",
    surface = true,
    className = "",
    children,
  }: {
    tone?: "default" | "error" | "info" | "muted";
    size?: "sm" | "md";
    surface?: boolean;
    className?: string;
    children?: Snippet;
  } = $props();
</script>

<StatusMessage {tone} {size} {surface} className={cn("extractum-status-message", className)}>
  {@render children?.()}
</StatusMessage>
