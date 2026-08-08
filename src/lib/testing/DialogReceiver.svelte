<script lang="ts">
  let {
    open = false,
    title,
    onClose,
    children,
  } = $props<{
    open?: boolean;
    title: string;
    onClose?: () => void;
    children?: import("svelte").Snippet;
  }>();
</script>

{#if open}
  <div role="dialog" aria-label={title}>
    <h2>{title}</h2>
    {@render children?.()}
    <button type="button" onclick={() => onClose?.()}>Close dialog</button>
  </div>
{/if}
