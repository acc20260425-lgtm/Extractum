import type { ApalisJobsPruneTerminalResponse } from "$lib/types/apalis-jobs";

export function apalisJobsNavigationItem(_mode: "legacy" | "projects") {
  return {
    href: "/jobs",
    label: "Jobs",
    caption: "Apalis queue",
    active: (pathname: string) => pathname.startsWith("/jobs"),
  };
}

export const APALIS_PRUNE_CONFIRMATION =
  "Delete finished Apalis jobs older than 24 hours? This includes Done, Killed, and Failed jobs with no retries left. This cannot be undone.";

export function createApalisJobsRouteOrchestration({
  confirmPrune,
  schedule,
}: {
  confirmPrune: (message: string) => boolean;
  schedule: (callback: () => void, delayMs: number) => ReturnType<typeof setTimeout>;
}) {
  return {
    manualRefresh(refresh: () => Promise<void>) {
      return refresh();
    },
    async guardedPrune(
      prune: () => Promise<ApalisJobsPruneTerminalResponse>,
      refresh: () => Promise<void>,
      onConfirmed: () => void = () => {},
    ) {
      if (!confirmPrune(APALIS_PRUNE_CONFIRMATION)) return null;
      onConfirmed();
      const result = await prune();
      await refresh();
      return result;
    },
    scheduleSearch(refresh: () => void) {
      return schedule(refresh, 250);
    },
  };
}
