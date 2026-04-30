<script lang="ts">
  let {
    title = "",
    description,
    centered = false,
    className = "",
  }: {
    title?: string;
    description: string;
    centered?: boolean;
    className?: string;
  } = $props();
</script>

<div class={`ui-empty-state ${centered ? "centered" : ""} ${className}`.trim()}>
  {#if title}
    <h3>{title}</h3>
  {/if}
  <p>{description}</p>
</div>

<style>
  .ui-empty-state {
    color: var(--muted);
  }

  .ui-empty-state.centered {
    display: flex;
    flex-direction: column;
    justify-content: center;
    min-height: 18rem;
  }

  .ui-empty-state h3 {
    margin: 0 0 0.5rem 0;
    color: var(--text);
    font-size: 1.05rem;
  }

  .ui-empty-state p {
    margin: 0;
    line-height: 1.5;
  }
</style>
