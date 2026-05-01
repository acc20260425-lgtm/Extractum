<script lang="ts">
  let {
    value = "",
    rows,
    placeholder,
    disabled = false,
    readonly = false,
    name,
    spellcheck,
    className = "",
    oninput,
    onchange,
    onkeydown,
  }: {
    value?: string;
    rows?: number;
    placeholder?: string;
    disabled?: boolean;
    readonly?: boolean;
    name?: string;
    spellcheck?: boolean;
    className?: string;
    oninput?: (event: Event) => void;
    onchange?: (event: Event) => void;
    onkeydown?: (event: KeyboardEvent) => void;
  } = $props();
</script>

<textarea
  {rows}
  {placeholder}
  {disabled}
  {readonly}
  {name}
  {spellcheck}
  class={`ui-textarea ${className}`.trim()}
  {oninput}
  {onchange}
  {onkeydown}
>{value}</textarea>

<style>
  .ui-textarea {
    width: 100%;
    min-width: 0;
    max-width: 100%;
    box-sizing: border-box;
    resize: vertical;
    min-height: 8rem;
    background: var(--panel-strong);
    border: 1px solid var(--border);
    color: var(--text);
    padding: 0.8rem;
    border-radius: 8px;
    font: inherit;
  }

  .ui-textarea::placeholder {
    color: var(--muted);
  }

  .ui-textarea:focus {
    border-color: var(--primary);
    outline: none;
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 18%, transparent);
  }
</style>
