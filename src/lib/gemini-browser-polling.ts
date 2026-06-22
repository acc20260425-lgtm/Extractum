import type {
  GeminiBrowserRefreshOptions,
  GeminiBrowserRefreshOutcome,
} from "./gemini-browser-refresh-scheduler";

export interface GeminiBrowserPollingSignal {
  key: string;
  updatedAt?: string | null;
}

export interface GeminiBrowserPollingActivitySnapshot {
  runLogSignals: GeminiBrowserPollingSignal[];
  statusSignal: string | null;
}

export interface GeminiBrowserPollingControllerDeps {
  scheduleRefresh: (options: GeminiBrowserRefreshOptions) => Promise<GeminiBrowserRefreshOutcome>;
  getActivitySnapshot: () => GeminiBrowserPollingActivitySnapshot;
  now?: () => number;
  idleMs?: number;
  activeMs?: number;
  activeGraceMs?: number;
  pendingNotFoundRetryMs?: number;
  maxConsecutiveFailures?: number;
}

export interface GeminiBrowserPollingController {
  start: () => void;
  stop: () => void;
  setLocalPendingRun: (runId: string) => void;
  markLocalPendingRunRejected: (runId: string) => void;
  confirmPendingRunNotFound: (runId: string) => void;
  clearLocalPendingRun: (runId: string) => void;
  confirmPendingRunTerminal: (runId: string) => void;
  recordRefreshOutcome: (outcome: GeminiBrowserRefreshOutcome) => void;
  hasLocalPendingRun: (runId?: string) => boolean;
}

export function createGeminiBrowserPollingController(
  deps: GeminiBrowserPollingControllerDeps,
): GeminiBrowserPollingController {
  const idleMs = deps.idleMs ?? 5000;
  const activeMs = deps.activeMs ?? 1000;
  const activeGraceMs = deps.activeGraceMs ?? 30 * 60 * 1000;
  const pendingNotFoundRetryMs = deps.pendingNotFoundRetryMs ?? 2000;
  const maxConsecutiveFailures = deps.maxConsecutiveFailures ?? 3;
  let timer: ReturnType<typeof setTimeout> | null = null;
  let running = false;
  let inFlight = false;
  let degraded = false;
  let consecutiveFailures = 0;
  const localPendingRuns = new Map<
    string,
    { startedAt: number; rejectedAt: number | null; notFoundCount: number }
  >();
  const firstSeenActivity = new Map<string, number>();

  function now() {
    return deps.now?.() ?? Date.now();
  }

  function clearTimer() {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
  }

  function isFreshByTimestamp(updatedAt: string | null | undefined) {
    if (!updatedAt) return false;
    const updatedAtMs = Date.parse(updatedAt);
    if (Number.isNaN(updatedAtMs)) return false;
    return now() - updatedAtMs <= activeGraceMs;
  }

  function isFreshByFirstSeen(key: string) {
    const seenAt = firstSeenActivity.get(key);
    if (seenAt === undefined) {
      firstSeenActivity.set(key, now());
      return true;
    }
    return now() - seenAt <= activeGraceMs;
  }

  function isFreshSignal(signal: GeminiBrowserPollingSignal) {
    if (signal.updatedAt) return isFreshByTimestamp(signal.updatedAt);
    return isFreshByFirstSeen(`run:${signal.key}`);
  }

  function pruneExpiredPendingRuns() {
    for (const [runId, pending] of localPendingRuns) {
      const age = now() - pending.startedAt;
      const rejectedAge = pending.rejectedAt === null ? 0 : now() - pending.rejectedAt;
      if (age > activeGraceMs) {
        localPendingRuns.delete(runId);
      } else if (pending.notFoundCount >= 2) {
        localPendingRuns.delete(runId);
      } else if (
        pending.rejectedAt !== null &&
        pending.notFoundCount > 0 &&
        rejectedAge >= pendingNotFoundRetryMs
      ) {
        localPendingRuns.delete(runId);
      }
    }
  }

  function hasFreshStatusSignal(
    snapshot: GeminiBrowserPollingActivitySnapshot,
    hasRunActivity: boolean,
  ) {
    if (!snapshot.statusSignal) return false;
    if (hasRunActivity) return true;
    return isFreshByFirstSeen(`status:${snapshot.statusSignal}`);
  }

  function active() {
    pruneExpiredPendingRuns();
    if (degraded) return false;
    const snapshot = deps.getActivitySnapshot();
    const hasRunActivity =
      localPendingRuns.size > 0 || snapshot.runLogSignals.some((signal) => isFreshSignal(signal));
    return hasRunActivity || hasFreshStatusSignal(snapshot, hasRunActivity);
  }

  function applyRefreshOutcome(outcome: GeminiBrowserRefreshOutcome) {
    if (outcome.allFailed) {
      consecutiveFailures += 1;
      if (consecutiveFailures >= maxConsecutiveFailures) {
        degraded = true;
      }
    } else {
      consecutiveFailures = 0;
      degraded = false;
    }
  }

  function scheduleNext() {
    if (!running) return;
    clearTimer();
    const wasActive = active();
    timer = setTimeout(() => {
      void tick(wasActive);
    }, wasActive ? activeMs : idleMs);
  }

  async function tick(wasActive: boolean) {
    if (!running) return;
    if (inFlight) {
      scheduleNext();
      return;
    }
    if (wasActive && !active()) {
      scheduleNext();
      return;
    }
    inFlight = true;
    try {
      const outcome = await deps.scheduleRefresh({ mode: "light" });
      applyRefreshOutcome(outcome);
    } catch (_error) {
      consecutiveFailures += 1;
      if (consecutiveFailures >= maxConsecutiveFailures) {
        degraded = true;
      }
    } finally {
      inFlight = false;
      scheduleNext();
    }
  }

  return {
    start() {
      if (running) return;
      running = true;
      scheduleNext();
    },
    stop() {
      running = false;
      clearTimer();
    },
    setLocalPendingRun(runId) {
      localPendingRuns.set(runId, { startedAt: now(), rejectedAt: null, notFoundCount: 0 });
      if (running) scheduleNext();
    },
    markLocalPendingRunRejected(runId) {
      const pending = localPendingRuns.get(runId);
      if (!pending) return;
      pending.rejectedAt = now();
    },
    confirmPendingRunNotFound(runId) {
      const pending = localPendingRuns.get(runId);
      if (!pending) return;
      pending.notFoundCount += 1;
      if (pending.rejectedAt === null) pending.rejectedAt = now();
      pruneExpiredPendingRuns();
    },
    clearLocalPendingRun(runId) {
      localPendingRuns.delete(runId);
    },
    confirmPendingRunTerminal(runId) {
      localPendingRuns.delete(runId);
    },
    recordRefreshOutcome(outcome) {
      applyRefreshOutcome(outcome);
      if (running) scheduleNext();
    },
    hasLocalPendingRun(runId) {
      pruneExpiredPendingRuns();
      return runId ? localPendingRuns.has(runId) : localPendingRuns.size > 0;
    },
  };
}
