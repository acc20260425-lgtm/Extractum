<script lang="ts">
  import { onMount } from "svelte";
  import { resolveYoutubeThumbnail } from "$lib/youtube-thumbnail";

  let {
    url,
    fallbackSrc = null,
    alt = "",
    class: className = "",
  }: {
    url: string | null;
    fallbackSrc?: string | null;
    alt?: string;
    class?: string;
  } = $props();

  let src = $state<string | null>(null);
  let target = $state<HTMLSpanElement>();

  onMount(() => {
    src = fallbackSrc;
    if (!url) return;
    if (!target) return;
    const observer = new IntersectionObserver(async ([entry]) => {
      if (!entry?.isIntersecting) return;
      observer.disconnect();
      src = (await resolveYoutubeThumbnail(url)) ?? fallbackSrc;
    });
    observer.observe(target);
    return () => observer.disconnect();
  });
</script>

<span data-youtube-thumbnail={url ?? ""} bind:this={target}>
  {#if src}
    <img src={src} {alt} class={className} loading="lazy" />
  {/if}
</span>
