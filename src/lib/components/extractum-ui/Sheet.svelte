<script lang="ts">
  import {
    Sheet,
    SheetContent,
    SheetDescription,
    SheetHeader,
    SheetTitle,
  } from "$lib/components/ui/sheet/index.js";
  import { cn } from "$lib/utils.js";
  import type { Snippet } from "svelte";
  import type { Side } from "$lib/components/ui/sheet/sheet-content.svelte";

  let {
    open = $bindable(false),
    title = "",
    description = "",
    side = "right",
    class: className,
    contentClass = "",
    children,
    ...rest
  }: {
    open?: boolean;
    title?: string;
    description?: string;
    side?: Side;
    class?: string;
    contentClass?: string;
    children?: Snippet;
  } = $props();
</script>

<Sheet bind:open {...rest}>
  <SheetContent
    {side}
    class={cn(
      "extractum-sheet w-[min(1180px,calc(100vw-96px))] data-[side=left]:w-[min(1180px,calc(100vw-96px))] data-[side=right]:w-[min(1180px,calc(100vw-96px))] max-w-none data-[side=left]:sm:max-w-none data-[side=right]:sm:max-w-none border-[var(--extractum-border)] bg-[var(--extractum-surface)] text-[var(--extractum-text)]",
      className,
      contentClass,
    )}
  >
    {#if title || description}
      <SheetHeader class="border-b border-[var(--extractum-border)]">
        {#if title}
          <SheetTitle>{title}</SheetTitle>
        {/if}
        {#if description}
          <SheetDescription>{description}</SheetDescription>
        {/if}
      </SheetHeader>
    {/if}

    <div class="extractum-sheet-body">
      {@render children?.()}
    </div>
  </SheetContent>
</Sheet>

<style>
  .extractum-sheet-body {
    min-height: 0;
    overflow: auto;
    padding: 0 16px 16px;
  }
</style>
