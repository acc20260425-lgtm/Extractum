import type {
  GeminiBrowserProviderStatus,
  GeminiBrowserRun,
  GeminiBrowserRunLogSummary,
} from "./types/gemini-browser";

export type GeminiBrowserRefreshMode = "light" | "full";

export interface GeminiBrowserRefreshOptions {
  mode?: GeminiBrowserRefreshMode;
  forceTrailing?: boolean;
}

export interface GeminiBrowserRefreshOutcome {
  allFailed: boolean;
}

export interface GeminiBrowserRefreshSchedulerDeps {
  loadStatus: () => Promise<GeminiBrowserProviderStatus>;
  loadStatusSnapshot: () => Promise<GeminiBrowserProviderStatus>;
  loadRuns: () => Promise<GeminiBrowserRunLogSummary>;
  loadRun: (runId: string) => Promise<GeminiBrowserRun>;
  getSelectedRunId: () => string | null;
  getSelectedDetailToken: () => number;
  applyStatus: (status: GeminiBrowserProviderStatus) => void;
  applyRuns: (runs: GeminiBrowserRun[]) => void;
  applySelectedRun: (run: GeminiBrowserRun) => void;
  applySelectedRunUnavailable: (runId: string, message: string) => void;
  applySelectedRunError: (runId: string, message: string) => void;
  applyStatusError: (message: string | null) => void;
  applyRunsError: (message: string | null) => void;
  applyMessage: (message: string) => void;
  syncActivePromptResult: (runs: GeminiBrowserRun[]) => void;
  formatError: (context: string, error: unknown) => string;
  isRunNotFoundError: (error: unknown) => boolean;
  isDisposed?: () => boolean;
}

export interface GeminiBrowserRefreshScheduler {
  scheduleRefresh: (
    options?: GeminiBrowserRefreshOptions,
  ) => Promise<GeminiBrowserRefreshOutcome>;
  dispose: () => void;
}

function modeRank(mode: GeminiBrowserRefreshMode) {
  return mode === "full" ? 2 : 1;
}

function strongestMode(
  left: GeminiBrowserRefreshMode,
  right: GeminiBrowserRefreshMode,
): GeminiBrowserRefreshMode {
  return modeRank(right) > modeRank(left) ? right : left;
}

function compareUpdatedAt(left: string, right: string) {
  const leftMs = Date.parse(left);
  const rightMs = Date.parse(right);
  if (Number.isNaN(leftMs) || Number.isNaN(rightMs)) return null;
  return leftMs - rightMs;
}

function selectedVersionKey(runId: string, token: number) {
  return `${token}:${runId}`;
}

// Each caller gets a promise for the refresh requested by that call.
// Later trailing refreshes must not resolve or reject an earlier caller.
export function createGeminiBrowserRefreshScheduler(
  deps: GeminiBrowserRefreshSchedulerDeps,
): GeminiBrowserRefreshScheduler {
  let activeRefresh: Promise<GeminiBrowserRefreshOutcome> | null = null;
  let activeMode: GeminiBrowserRefreshMode | null = null;
  let trailingRequested = false;
  let trailingMode: GeminiBrowserRefreshMode = "light";
  let trailingPromise: Promise<GeminiBrowserRefreshOutcome> | null = null;
  let resolveTrailing: ((outcome: GeminiBrowserRefreshOutcome) => void) | null = null;
  let rejectTrailing: ((error: unknown) => void) | null = null;
  let disposed = false;
  const latestSelectedRunVersions = new Map<string, string>();

  function isDisposed() {
    return disposed || deps.isDisposed?.() === true;
  }

  function applyIfLive(callback: () => void) {
    if (isDisposed()) return false;
    callback();
    return true;
  }

  function applySelectedRunIfCurrent(
    run: GeminiBrowserRun,
    requestedRunId: string,
    requestedToken: number,
  ) {
    if (isDisposed()) return false;
    if (deps.getSelectedRunId() !== requestedRunId) return false;
    if (deps.getSelectedDetailToken() !== requestedToken) return false;
    const versionKey = selectedVersionKey(run.run_id, requestedToken);
    const latest = latestSelectedRunVersions.get(versionKey);
    if (latest) {
      const comparison = compareUpdatedAt(run.updated_at, latest);
      if (comparison !== null && comparison < 0) return false;
      if (comparison === null && !Number.isNaN(Date.parse(latest))) return false;
    }
    latestSelectedRunVersions.set(versionKey, run.updated_at);
    deps.applySelectedRun(run);
    return true;
  }

  async function runRefreshOnce(
    mode: GeminiBrowserRefreshMode,
  ): Promise<GeminiBrowserRefreshOutcome> {
    let successfulReadModels = 0;
    const selectedRunId = deps.getSelectedRunId();
    const selectedDetailToken = deps.getSelectedDetailToken();
    const statusPromise = mode === "full" ? deps.loadStatus() : deps.loadStatusSnapshot();
    const [statusResult, runsResult] = await Promise.allSettled([
      statusPromise,
      deps.loadRuns(),
    ]);

    if (statusResult.status === "fulfilled") {
      successfulReadModels += 1;
      applyIfLive(() => {
        deps.applyStatus(statusResult.value);
        deps.applyStatusError(null);
        deps.applyMessage(statusResult.value.latest_message ?? "");
      });
    } else {
      const formatted = deps.formatError(
        "loading Gemini browser provider status",
        statusResult.reason,
      );
      applyIfLive(() => {
        deps.applyStatusError(formatted);
        deps.applyMessage(formatted);
      });
    }

    let selectedRow: GeminiBrowserRun | null = null;
    if (runsResult.status === "fulfilled") {
      successfulReadModels += 1;
      const runs = runsResult.value.runs;
      selectedRow =
        selectedRunId === null ? null : runs.find((run) => run.run_id === selectedRunId) ?? null;
      applyIfLive(() => {
        deps.applyRuns(runs);
        deps.applyRunsError(null);
        deps.syncActivePromptResult(runs);
      });
      if (selectedRow && selectedRunId) {
        applySelectedRunIfCurrent(selectedRow, selectedRunId, selectedDetailToken);
      }
    } else {
      applyIfLive(() => {
        deps.applyRunsError(
          deps.formatError("loading Gemini browser run history", runsResult.reason),
        );
      });
    }

    const shouldLoadSelectedDetail =
      selectedRunId !== null && (runsResult.status === "rejected" || !selectedRow);
    if (shouldLoadSelectedDetail && selectedRunId) {
      try {
        const detailRun = await deps.loadRun(selectedRunId);
        successfulReadModels += 1;
        applySelectedRunIfCurrent(detailRun, selectedRunId, selectedDetailToken);
      } catch (error) {
        const formatted = deps.formatError("loading Gemini browser run detail", error);
        if (deps.isRunNotFoundError(error)) {
          applyIfLive(() => {
            deps.applySelectedRunUnavailable(selectedRunId, formatted);
          });
        } else {
          applyIfLive(() => {
            deps.applySelectedRunError(selectedRunId, formatted);
          });
        }
      }
    }

    return { allFailed: successfulReadModels === 0 };
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

  function finishRefresh(refresh: Promise<GeminiBrowserRefreshOutcome>) {
    if (activeRefresh === refresh) {
      activeRefresh = null;
      activeMode = null;
    }
    if (trailingRequested) {
      const nextMode = trailingMode;
      trailingMode = "light";
      const trailing = takeTrailingRequest();
      const trailingRefresh = startRefreshForCall(nextMode);
      void trailingRefresh.then(
        (outcome) => trailing.resolve?.(outcome),
        (error) => trailing.reject?.(error),
      );
    }
  }

  function startRefreshForCall(mode: GeminiBrowserRefreshMode) {
    const refresh = runRefreshOnce(mode);
    activeRefresh = refresh;
    activeMode = mode;

    void refresh.then(
      () => finishRefresh(refresh),
      () => finishRefresh(refresh),
    );

    return refresh;
  }

  function ensureTrailingPromise() {
    if (!trailingPromise) {
      trailingPromise = new Promise<GeminiBrowserRefreshOutcome>((resolve, reject) => {
        resolveTrailing = resolve;
        rejectTrailing = reject;
      });
    }
    return trailingPromise;
  }

  function scheduleRefresh(options: GeminiBrowserRefreshOptions = {}) {
    const requestedMode = options.mode ?? "light";
    if (
      activeRefresh &&
      !options.forceTrailing &&
      activeMode &&
      modeRank(activeMode) >= modeRank(requestedMode)
    ) {
      return activeRefresh;
    }
    if (activeRefresh) {
      trailingRequested = true;
      trailingMode = strongestMode(trailingMode, requestedMode);
      return ensureTrailingPromise();
    }
    return startRefreshForCall(requestedMode);
  }

  return {
    scheduleRefresh,
    dispose() {
      disposed = true;
    },
  };
}
