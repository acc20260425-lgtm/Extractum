<script lang="ts">
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
  } from "$lib/components/ui/dialog/index.js";
  import { cn } from "$lib/utils.js";
  import type { ComponentProps, Snippet } from "svelte";

  let {
    open = $bindable(false),
    title = "",
    description = "",
    class: className,
    contentClass = "",
    children,
    ...rest
  }: ComponentProps<typeof Dialog> & {
    title?: string;
    description?: string;
    class?: string;
    contentClass?: string;
    children?: Snippet;
  } = $props();
</script>

<Dialog bind:open {...rest}>
  <DialogContent
    class={cn(
      "extractum-dialog max-h-[min(760px,calc(100vh-48px))] w-[min(960px,calc(100vw-48px))] max-w-none overflow-hidden rounded-[var(--extractum-radius)] border border-[var(--extractum-border)] bg-[var(--extractum-surface)] p-0 text-[var(--extractum-text)] shadow-xl sm:max-w-none",
      className,
      contentClass,
    )}
  >
    {#if title || description}
      <DialogHeader class="border-b border-[var(--extractum-border)] px-4 py-3">
        {#if title}
          <DialogTitle>{title}</DialogTitle>
        {/if}
        {#if description}
          <DialogDescription>{description}</DialogDescription>
        {/if}
      </DialogHeader>
    {/if}

    <div class="extractum-dialog-body">
      {@render children?.()}
    </div>
  </DialogContent>
</Dialog>

<style>
  .extractum-dialog-body {
    min-height: 0;
    overflow: auto;
    padding: 16px;
  }
</style>
