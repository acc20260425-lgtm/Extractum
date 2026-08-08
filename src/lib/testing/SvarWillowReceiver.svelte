<script lang="ts">
  let { fonts, children } = $props<{ fonts?: boolean; children?: import("svelte").Snippet }>();
</script>

<div data-testid="svar-willow" data-fonts={String(fonts)}>
  {@render children?.()}
</div>
