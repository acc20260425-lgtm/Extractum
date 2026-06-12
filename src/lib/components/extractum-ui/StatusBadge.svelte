<script lang="ts">
  import ExtractumBadge from "./Badge.svelte";
  import { cn } from "$lib/utils.js";
  import type { LibrarySourceStatus } from "$lib/ui/research-projects-model";
  import type { ComponentProps } from "svelte";

  type Status = LibrarySourceStatus | "connected" | "pending" | "failed" | "already_connected";

  const labels: Record<Status, string> = {
    active: "Active",
    needs_account: "Needs account",
    syncing: "Syncing",
    error: "Error",
    unavailable: "Unavailable",
    connected: "Connected",
    pending: "Pending",
    failed: "Failed",
    already_connected: "Connected",
  };

  const statusClasses: Record<Status, string> = {
    active: "border-emerald-200 bg-emerald-50 text-emerald-700",
    needs_account: "border-amber-200 bg-amber-50 text-amber-700",
    syncing: "border-blue-200 bg-blue-50 text-blue-700",
    error: "border-red-200 bg-red-50 text-red-700",
    unavailable: "border-zinc-200 bg-zinc-50 text-zinc-600",
    connected: "border-emerald-200 bg-emerald-50 text-emerald-700",
    pending: "border-blue-200 bg-blue-50 text-blue-700",
    failed: "border-red-200 bg-red-50 text-red-700",
    already_connected: "border-emerald-200 bg-emerald-50 text-emerald-700",
  };

  let {
    status,
    label = labels[status] ?? labels.unavailable,
    class: className,
    ...rest
  }: Omit<ComponentProps<typeof ExtractumBadge>, "children"> & {
    status: Status;
    label?: string;
  } = $props();
</script>

<ExtractumBadge
  variant="outline"
  class={cn("extractum-status-badge border", statusClasses[status] ?? statusClasses.unavailable, className)}
  {...rest}
>
  {label}
</ExtractumBadge>
