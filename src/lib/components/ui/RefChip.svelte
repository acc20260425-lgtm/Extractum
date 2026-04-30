<script lang="ts">
  let {
    refValue,
    active = false,
    title,
    className = "",
    onclick,
  }: {
    refValue: string;
    active?: boolean;
    title?: string;
    className?: string;
    onclick?: (event: MouseEvent) => void | Promise<void>;
  } = $props();
</script>

<button
  type="button"
  {title}
  class={`ui-ref-chip ${active ? "active" : ""} ${className}`.trim()}
  onclick={onclick}
>
  [{refValue}]
</button>

<style>
  .ui-ref-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.08rem 0.45rem;
    margin: 0 0.08rem;
    border-radius: 999px;
    border: 1px solid color-mix(in srgb, var(--primary) 24%, transparent);
    background: color-mix(in srgb, var(--primary) 14%, var(--panel));
    color: var(--primary);
    font-size: 0.82rem;
    font-weight: 600;
  }

  .ui-ref-chip:hover,
  .ui-ref-chip.active {
    background: color-mix(in srgb, var(--primary) 22%, var(--panel));
  }
 </style>
