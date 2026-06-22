import type {
  GeminiBrowserProviderStatus,
  GeminiBrowserRun,
  GeminiBrowserRunLogSummary,
} from "./types/gemini-browser";

export interface GeminiBrowserRefreshSchedulerDeps {
  loadStatus: () => Promise<GeminiBrowserProviderStatus>;
  loadRuns: () => Promise<GeminiBrowserRunLogSummary>;
  applyStatus: (status: GeminiBrowserProviderStatus) => void;
  applyRuns: (runs: GeminiBrowserRun[]) => void;
  applyStatusError: (message: string | null) => void;
  applyRunsError: (message: string | null) => void;
  applyMessage: (message: string) => void;
  syncActivePromptResult: (runs: GeminiBrowserRun[]) => void;
  formatError: (context: string, error: unknown) => string;
}

export interface GeminiBrowserRefreshScheduler {
  scheduleRefresh: () => Promise<void>;
}

// Each caller gets a promise for the refresh requested by that call.
// Later trailing refreshes must not resolve or reject an earlier caller.
export function createGeminiBrowserRefreshScheduler(
  deps: GeminiBrowserRefreshSchedulerDeps,
): GeminiBrowserRefreshScheduler {
  let activeRefresh: Promise<void> | null = null;
  let trailingRequested = false;
  let trailingPromise: Promise<void> | null = null;
  let resolveTrailing: (() => void) | null = null;
  let rejectTrailing: ((error: unknown) => void) | null = null;

  async function runRefreshOnce() {
    const [statusResult, runsResult] = await Promise.allSettled([
      deps.loadStatus(),
      deps.loadRuns(),
    ]);

    if (statusResult.status === "fulfilled") {
      deps.applyStatus(statusResult.value);
      deps.applyStatusError(null);
      deps.applyMessage(statusResult.value.latest_message ?? "");
    } else {
      const formatted = deps.formatError(
        "loading Gemini browser provider status",
        statusResult.reason,
      );
      deps.applyStatusError(formatted);
      deps.applyMessage(formatted);
    }

    if (runsResult.status === "fulfilled") {
      deps.applyRuns(runsResult.value.runs);
      deps.applyRunsError(null);
      deps.syncActivePromptResult(runsResult.value.runs);
    } else {
      deps.applyRunsError(
        deps.formatError("loading Gemini browser run history", runsResult.reason),
      );
    }
  }

  function takeTrailingRequest() {
    const resolve = resolveTrailing;
    const reject = rejectTrailing;
    trailingRequested = false;
    trailingPromise = null;
    resolveTrailing = null;
    rejectTrailing = null;
    return { resolve, reject };
  }

  function finishRefresh(refresh: Promise<void>) {
    if (activeRefresh === refresh) {
      activeRefresh = null;
    }
    if (trailingRequested) {
      const trailing = takeTrailingRequest();
      const trailingRefresh = startRefreshForCall();
      void trailingRefresh.then(
        () => trailing.resolve?.(),
        (error) => trailing.reject?.(error),
      );
    }
  }

  function startRefreshForCall(): Promise<void> {
    const refresh = runRefreshOnce();
    activeRefresh = refresh;

    void refresh.then(
      () => finishRefresh(refresh),
      () => finishRefresh(refresh),
    );

    return refresh;
  }

  function ensureTrailingPromise() {
    if (!trailingPromise) {
      trailingPromise = new Promise<void>((resolve, reject) => {
        resolveTrailing = resolve;
        rejectTrailing = reject;
      });
    }
    return trailingPromise;
  }

  function scheduleRefresh() {
    if (activeRefresh) {
      trailingRequested = true;
      return ensureTrailingPromise();
    }
    return startRefreshForCall();
  }

  return { scheduleRefresh };
}
