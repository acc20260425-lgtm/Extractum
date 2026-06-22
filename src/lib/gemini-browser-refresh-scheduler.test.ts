import { describe, expect, it, vi } from "vitest";
import {
  createGeminiBrowserRefreshScheduler,
  type GeminiBrowserRefreshSchedulerDeps,
} from "./gemini-browser-refresh-scheduler";
import type {
  GeminiBrowserProviderStatus,
  GeminiBrowserRun,
  GeminiBrowserRunLogSummary,
} from "./types/gemini-browser";

function status(overrides: Partial<GeminiBrowserProviderStatus> = {}): GeminiBrowserProviderStatus {
  return {
    status: "ready",
    manual_action: null,
    active_run_id: null,
    queue_depth: 0,
    browser_profile_dir: "profile-dir",
    latest_message: "Ready",
    ...overrides,
  };
}

function run(run_id: string): GeminiBrowserRun {
  return {
    run_id,
    source: "settings_test",
    status: "ok",
    prompt_preview: "hello",
    created_at: "2026-06-22T00:00:00Z",
    updated_at: "2026-06-22T00:00:01Z",
    result: null,
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

function schedulerDeps(
  overrides: Partial<GeminiBrowserRefreshSchedulerDeps> = {},
): GeminiBrowserRefreshSchedulerDeps {
  return {
    loadStatus: vi.fn(async () => status({ latest_message: "Live" })),
    loadStatusSnapshot: vi.fn(async () => status({ latest_message: "Cached" })),
    loadRuns: vi.fn(async () => ({ runs: [run("run-1")] })),
    loadRun: vi.fn(async (runId: string) => run(runId)),
    getSelectedRunId: vi.fn(() => null),
    getSelectedDetailToken: vi.fn(() => 0),
    applyStatus: vi.fn(),
    applyRuns: vi.fn(),
    applySelectedRun: vi.fn(),
    applySelectedRunUnavailable: vi.fn(),
    applySelectedRunError: vi.fn(),
    applyStatusError: vi.fn(),
    applyRunsError: vi.fn(),
    applyMessage: vi.fn(),
    syncActivePromptResult: vi.fn(),
    formatError: (context, error) => `${context}: ${String(error)}`,
    isRunNotFoundError: (error: unknown) =>
      typeof error === "object" &&
      error !== null &&
      "kind" in error &&
      (error as { kind?: unknown }).kind === "not_found",
    isDisposed: vi.fn(() => false),
    ...overrides,
  };
}

describe("gemini browser refresh scheduler", () => {
  it("light refresh uses cached status and never calls live status", async () => {
    const deps = schedulerDeps();
    await createGeminiBrowserRefreshScheduler(deps).scheduleRefresh({ mode: "light" });

    expect(deps.loadStatusSnapshot).toHaveBeenCalledTimes(1);
    expect(deps.loadStatus).not.toHaveBeenCalled();
    expect(deps.loadRuns).toHaveBeenCalledTimes(1);
  });

  it("defaults to light refresh for safety", async () => {
    const deps = schedulerDeps();
    await createGeminiBrowserRefreshScheduler(deps).scheduleRefresh();

    expect(deps.loadStatusSnapshot).toHaveBeenCalledTimes(1);
    expect(deps.loadStatus).not.toHaveBeenCalled();
  });

  it("full refresh uses live status", async () => {
    const deps = schedulerDeps();
    await createGeminiBrowserRefreshScheduler(deps).scheduleRefresh({ mode: "full" });

    expect(deps.loadStatus).toHaveBeenCalledTimes(1);
    expect(deps.loadStatusSnapshot).not.toHaveBeenCalled();
    expect(deps.loadRuns).toHaveBeenCalledTimes(1);
  });

  it("applies status and run history independently", async () => {
    const deps = schedulerDeps({
      loadStatusSnapshot: vi.fn(async () => {
        throw new Error("status down");
      }),
      loadRuns: vi.fn(async (): Promise<GeminiBrowserRunLogSummary> => ({ runs: [run("run-ok")] })),
    });

    const outcome = await createGeminiBrowserRefreshScheduler(deps).scheduleRefresh();

    expect(outcome.allFailed).toBe(false);
    expect(deps.applyRuns).toHaveBeenCalledWith([run("run-ok")]);
    expect(deps.syncActivePromptResult).toHaveBeenCalledWith([run("run-ok")]);
    expect(deps.applyStatus).not.toHaveBeenCalled();
    expect(deps.applyStatusError).toHaveBeenCalledWith(
      "loading Gemini browser provider status: Error: status down",
    );
  });

  it("does not downgrade pending full refresh behind an active light refresh", async () => {
    const firstStatus = deferred<GeminiBrowserProviderStatus>();
    const firstRuns = deferred<GeminiBrowserRunLogSummary>();
    const deps = schedulerDeps({
      loadStatusSnapshot: vi.fn().mockReturnValueOnce(firstStatus.promise).mockResolvedValue(status()),
      loadRuns: vi.fn().mockReturnValueOnce(firstRuns.promise).mockResolvedValue({ runs: [] }),
    });
    const scheduler = createGeminiBrowserRefreshScheduler(deps);

    const active = scheduler.scheduleRefresh({ mode: "light" });
    const trailing = scheduler.scheduleRefresh({ mode: "full" });

    firstStatus.resolve(status());
    firstRuns.resolve({ runs: [] });
    await active;
    await trailing;

    expect(deps.loadStatus).toHaveBeenCalledTimes(1);
  });

  it("light request attaches to active full refresh without trailing light refresh", async () => {
    const firstStatus = deferred<GeminiBrowserProviderStatus>();
    const firstRuns = deferred<GeminiBrowserRunLogSummary>();
    const deps = schedulerDeps({
      loadStatus: vi.fn().mockReturnValueOnce(firstStatus.promise),
      loadRuns: vi.fn().mockReturnValueOnce(firstRuns.promise),
    });
    const scheduler = createGeminiBrowserRefreshScheduler(deps);

    const active = scheduler.scheduleRefresh({ mode: "full" });
    const attached = scheduler.scheduleRefresh({ mode: "light" });

    expect(attached).toBe(active);
    firstStatus.resolve(status());
    firstRuns.resolve({ runs: [] });
    await attached;

    expect(deps.loadStatus).toHaveBeenCalledTimes(1);
    expect(deps.loadStatusSnapshot).not.toHaveBeenCalled();
  });

  it("ignores selected detail response with stale updated_at", async () => {
    const latest = run("selected");
    latest.updated_at = "2026-06-22T00:00:02Z";
    const stale = run("selected");
    stale.status = "running";
    stale.updated_at = "2026-06-22T00:00:01Z";
    const deps = schedulerDeps({
      getSelectedRunId: vi.fn(() => "selected"),
      getSelectedDetailToken: vi.fn(() => 1),
      loadRuns: vi.fn().mockResolvedValueOnce({ runs: [latest] }).mockResolvedValueOnce({ runs: [] }),
      loadRun: vi.fn(async () => stale),
    });
    const scheduler = createGeminiBrowserRefreshScheduler(deps);

    await scheduler.scheduleRefresh({ mode: "light" });
    await scheduler.scheduleRefresh({ mode: "light" });

    expect(deps.applySelectedRun).toHaveBeenCalledWith(latest);
    expect(deps.applySelectedRun).not.toHaveBeenCalledWith(stale);
  });

  it("applies selected row from list_runs when it is visible", async () => {
    const selected = run("selected");
    const deps = schedulerDeps({
      getSelectedRunId: vi.fn(() => "selected"),
      getSelectedDetailToken: vi.fn(() => 1),
      loadRuns: vi.fn(async () => ({ runs: [selected] })),
    });

    await createGeminiBrowserRefreshScheduler(deps).scheduleRefresh({ mode: "light" });

    expect(deps.applySelectedRun).toHaveBeenCalledWith(selected);
    expect(deps.loadRun).not.toHaveBeenCalled();
  });

  it("loads selected detail even when list_runs fails", async () => {
    const selected = run("selected");
    const deps = schedulerDeps({
      getSelectedRunId: vi.fn(() => "selected"),
      getSelectedDetailToken: vi.fn(() => 1),
      loadRuns: vi.fn(async () => {
        throw new Error("history down");
      }),
      loadRun: vi.fn(async () => selected),
    });

    const outcome = await createGeminiBrowserRefreshScheduler(deps).scheduleRefresh({ mode: "light" });

    expect(outcome.allFailed).toBe(false);
    expect(deps.applyRunsError).toHaveBeenCalled();
    expect(deps.loadRun).toHaveBeenCalledWith("selected");
    expect(deps.applySelectedRun).toHaveBeenCalledWith(selected);
  });

  it("applies externally created prompt-pack runs from light refresh list_runs", async () => {
    const promptPackRun = run("prompt-pack-run");
    promptPackRun.source = "prompt_pack";
    const deps = schedulerDeps({
      loadRuns: vi.fn(async () => ({ runs: [promptPackRun] })),
    });

    await createGeminiBrowserRefreshScheduler(deps).scheduleRefresh({ mode: "light" });

    expect(deps.loadStatusSnapshot).toHaveBeenCalledTimes(1);
    expect(deps.loadStatus).not.toHaveBeenCalled();
    expect(deps.applyRuns).toHaveBeenCalledWith([promptPackRun]);
  });

  it("ignores selected detail response for an obsolete selection token", async () => {
    let selectedToken = 1;
    const selected = run("selected");
    const deps = schedulerDeps({
      getSelectedRunId: vi.fn(() => "selected"),
      getSelectedDetailToken: vi.fn(() => selectedToken),
      loadRuns: vi.fn(async () => ({ runs: [] })),
      loadRun: vi.fn(async () => {
        selectedToken = 2;
        return selected;
      }),
    });

    await createGeminiBrowserRefreshScheduler(deps).scheduleRefresh({ mode: "light" });

    expect(deps.applySelectedRun).not.toHaveBeenCalled();
  });

  it("does not reuse selected detail version guard across selection tokens", async () => {
    let selectedToken = 1;
    const first = run("selected");
    first.updated_at = "not-a-date";
    const second = run("selected");
    second.updated_at = "not-a-date";
    const deps = schedulerDeps({
      getSelectedRunId: vi.fn(() => "selected"),
      getSelectedDetailToken: vi.fn(() => selectedToken),
      loadRuns: vi.fn().mockResolvedValueOnce({ runs: [first] }).mockResolvedValueOnce({ runs: [second] }),
    });
    const scheduler = createGeminiBrowserRefreshScheduler(deps);

    await scheduler.scheduleRefresh({ mode: "light" });
    selectedToken = 2;
    await scheduler.scheduleRefresh({ mode: "light" });

    expect(deps.applySelectedRun).toHaveBeenCalledWith(first);
    expect(deps.applySelectedRun).toHaveBeenCalledWith(second);
  });

  it("does not apply callbacks after disposal", async () => {
    const deps = schedulerDeps({
      isDisposed: vi.fn(() => true),
    });

    await createGeminiBrowserRefreshScheduler(deps).scheduleRefresh({ mode: "light" });

    expect(deps.applyStatus).not.toHaveBeenCalled();
    expect(deps.applyRuns).not.toHaveBeenCalled();
  });

  it("resolves with allFailed when every requested light read model fails", async () => {
    const deps = schedulerDeps({
      loadStatusSnapshot: vi.fn(async () => {
        throw new Error("snapshot down");
      }),
      loadRuns: vi.fn(async () => {
        throw new Error("runs down");
      }),
      getSelectedRunId: vi.fn(() => "selected"),
      loadRun: vi.fn(async () => {
        throw new Error("detail down");
      }),
    });

    const outcome = await createGeminiBrowserRefreshScheduler(deps).scheduleRefresh({ mode: "light" });

    expect(outcome.allFailed).toBe(true);
    expect(deps.applyStatusError).toHaveBeenCalled();
    expect(deps.applyRunsError).toHaveBeenCalled();
    expect(deps.applySelectedRunError).toHaveBeenCalled();
  });
});
