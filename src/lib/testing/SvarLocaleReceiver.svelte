<script lang="ts">
  let { words, children } = $props<{ words: Record<string, unknown>; children?: import("svelte").Snippet }>();
</script>

<div data-testid="svar-locale" data-words={JSON.stringify(words)}>
  {@render children?.()}
</div>
