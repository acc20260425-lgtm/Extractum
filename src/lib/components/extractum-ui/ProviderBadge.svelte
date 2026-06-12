<script lang="ts">
  import ExtractumBadge from "./Badge.svelte";
  import { cn } from "$lib/utils.js";
  import type { LibrarySourceProvider } from "$lib/ui/research-projects-model";
  import type { ComponentProps } from "svelte";

  type Provider = LibrarySourceProvider | "telegram" | "youtube" | "rss" | "forum" | "web" | "other";

  const labels: Record<Provider, string> = {
    telegram: "Telegram",
    youtube: "YouTube",
    rss: "RSS",
    forum: "Forum",
    web: "Web",
    other: "Other",
  };

  const providerClasses: Record<Provider, string> = {
    telegram: "border-sky-200 bg-sky-50 text-sky-700",
    youtube: "border-red-200 bg-red-50 text-red-700",
    rss: "border-amber-200 bg-amber-50 text-amber-700",
    forum: "border-emerald-200 bg-emerald-50 text-emerald-700",
    web: "border-indigo-200 bg-indigo-50 text-indigo-700",
    other: "border-zinc-200 bg-zinc-50 text-zinc-700",
  };

  let {
    provider,
    label = labels[provider] ?? labels.other,
    class: className,
    ...rest
  }: Omit<ComponentProps<typeof ExtractumBadge>, "children"> & {
    provider: Provider;
    label?: string;
  } = $props();
</script>

<ExtractumBadge
  variant="outline"
  class={cn("extractum-provider-badge border", providerClasses[provider] ?? providerClasses.other, className)}
  {...rest}
>
  {label}
</ExtractumBadge>
