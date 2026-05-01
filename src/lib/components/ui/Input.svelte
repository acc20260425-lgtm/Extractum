<script lang="ts">
  type InputAutocomplete = HTMLInputElement["autocomplete"];

  let {
    type = "text",
    value = "",
    placeholder,
    disabled = false,
    readonly = false,
    autocomplete,
    spellcheck,
    min,
    max,
    step,
    name,
    className = "",
    oninput,
    onchange,
    onkeydown,
  }: {
    type?: string;
    value?: string | number;
    placeholder?: string;
    disabled?: boolean;
    readonly?: boolean;
    autocomplete?: InputAutocomplete;
    spellcheck?: boolean;
    min?: number | string;
    max?: number | string;
    step?: number | string;
    name?: string;
    className?: string;
    oninput?: (event: Event) => void;
    onchange?: (event: Event) => void;
    onkeydown?: (event: KeyboardEvent) => void;
  } = $props();
</script>

<input
  {type}
  {value}
  {placeholder}
  {disabled}
  {readonly}
  {autocomplete}
  {spellcheck}
  {min}
  {max}
  {step}
  {name}
  class={`ui-input ${className}`.trim()}
  {oninput}
  {onchange}
  {onkeydown}
/>

<style>
  .ui-input {
    width: 100%;
    min-width: 0;
    max-width: 100%;
    box-sizing: border-box;
    background: var(--panel-strong);
    border: 1px solid var(--border);
    color: var(--text);
    padding: 0.6rem 0.8rem;
    border-radius: 6px;
    font-size: 1rem;
  }

  .ui-input::placeholder {
    color: var(--muted);
  }

  .ui-input:focus {
    border-color: var(--primary);
    outline: none;
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 18%, transparent);
  }
</style>
