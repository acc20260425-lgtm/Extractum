<script lang="ts">
  let {
    checked = false,
    disabled = false,
    title,
    description = "",
    className = "",
    onchange,
  }: {
    checked?: boolean;
    disabled?: boolean;
    title: string;
    description?: string;
    className?: string;
    onchange?: (event: Event) => void | Promise<void>;
  } = $props();
</script>

<label class={`ui-checkbox-row ${className}`.trim()}>
  <input
    type="checkbox"
    {checked}
    {disabled}
    {onchange}
  />
  <div class="copy">
    <strong>{title}</strong>
    {#if description}
      <span>{description}</span>
    {/if}
  </div>
</label>

<style>
  .ui-checkbox-row {
    display: flex;
    flex-direction: row;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.85rem 0.95rem;
    background: var(--panel-strong);
    border: 1px solid var(--border);
    border-radius: 10px;
    cursor: pointer;
  }

  .ui-checkbox-row:hover {
    background: var(--panel-hover);
  }

  .ui-checkbox-row input {
    margin-top: 0.2rem;
    width: auto;
    flex: 0 0 auto;
  }

  .copy {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    min-width: 0;
  }

  .copy span {
    color: var(--muted);
    font-size: 0.82rem;
  }
</style>
