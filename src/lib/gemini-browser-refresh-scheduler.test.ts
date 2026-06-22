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
    loadStatus: vi.fn(async () => status()),
    loadRuns: vi.fn(async () => ({ runs: [run("run-1")] })),
    applyStatus: vi.fn(),
    applyRuns: vi.fn(),
    applyStatusError: vi.fn(),
    applyRunsError: vi.fn(),
    applyMessage: vi.fn(),
    syncActivePromptResult: vi.fn(),
    formatError: (context, error) => `${context}: ${String(error)}`,
    ...overrides,
  };
}

describe("gemini browser refresh scheduler", () => {
  it("applies status and run history independently", async () => {
    const deps = schedulerDeps({
      loadStatus: vi.fn(async () => {
        throw new Error("status down");
      }),
      loadRuns: vi.fn(async (): Promise<GeminiBrowserRunLogSummary> => ({ runs: [run("run-ok")] })),
    });

    await createGeminiBrowserRefreshScheduler(deps).scheduleRefresh();

    expect(deps.applyRuns).toHaveBeenCalledWith([run("run-ok")]);
    expect(deps.syncActivePromptResult).toHaveBeenCalledWith([run("run-ok")]);
    expect(deps.applyStatus).not.toHaveBeenCalled();
    expect(deps.applyStatusError).toHaveBeenCalledWith(
      "loading Gemini browser provider status: Error: status down",
    );
  });

  it("applies provider status independently when run history fails", async () => {
    const ready = status({ latest_message: "Ready" });
    const deps = schedulerDeps({
      loadStatus: vi.fn(async () => ready),
      loadRuns: vi.fn(async () => {
        throw new Error("runs down");
      }),
    });

    await createGeminiBrowserRefreshScheduler(deps).scheduleRefresh();

    expect(deps.applyStatus).toHaveBeenCalledWith(ready);
    expect(deps.applyStatusError).toHaveBeenCalledWith(null);
    expect(deps.applyMessage).toHaveBeenCalledWith("Ready");
    expect(deps.applyRuns).not.toHaveBeenCalled();
    expect(deps.syncActivePromptResult).not.toHaveBeenCalled();
    expect(deps.applyRunsError).toHaveBeenCalledWith(
      "loading Gemini browser run history: Error: runs down",
    );
  });

  it("preserves previous state and records both errors when both requests fail", async () => {
    const deps = schedulerDeps({
      loadStatus: vi.fn(async () => {
        throw new Error("status down");
      }),
      loadRuns: vi.fn(async () => {
        throw new Error("runs down");
      }),
    });

    await expect(createGeminiBrowserRefreshScheduler(deps).scheduleRefresh()).resolves.toBeUndefined();

    expect(deps.applyStatus).not.toHaveBeenCalled();
    expect(deps.applyRuns).not.toHaveBeenCalled();
    expect(deps.applyStatusError).toHaveBeenCalledWith(
      "loading Gemini browser provider status: Error: status down",
    );
    expect(deps.applyRunsError).toHaveBeenCalledWith(
      "loading Gemini browser run history: Error: runs down",
    );
    expect(deps.applyMessage).toHaveBeenCalledWith(
      "loading Gemini browser provider status: Error: status down",
    );
  });

  it("shares one trailing promise for callers during an active refresh", async () => {
    const firstStatus = deferred<GeminiBrowserProviderStatus>();
    const firstRuns = deferred<GeminiBrowserRunLogSummary>();
    const deps = schedulerDeps({
      loadStatus: vi
        .fn()
        .mockReturnValueOnce(firstStatus.promise)
        .mockResolvedValue(status({ latest_message: "Second" })),
      loadRuns: vi
        .fn()
        .mockReturnValueOnce(firstRuns.promise)
        .mockResolvedValue({ runs: [run("run-2")] }),
    });
    const scheduler = createGeminiBrowserRefreshScheduler(deps);

    const active = scheduler.scheduleRefresh();
    const trailingA = scheduler.scheduleRefresh();
    const trailingB = scheduler.scheduleRefresh();

    expect(trailingA).toBe(trailingB);

    firstStatus.resolve(status({ latest_message: "First" }));
    firstRuns.resolve({ runs: [run("run-1")] });

    await active;
    await trailingA;

    expect(deps.loadStatus).toHaveBeenCalledTimes(2);
    expect(deps.loadRuns).toHaveBeenCalledTimes(2);
  });

  it("resolves the first caller and rejects only the trailing promise when trailing refresh throws unexpectedly", async () => {
    const firstStatus = deferred<GeminiBrowserProviderStatus>();
    const firstRuns = deferred<GeminiBrowserRunLogSummary>();
    const unexpected = new Error("apply exploded");
    const deps = schedulerDeps({
      loadStatus: vi
        .fn()
        .mockReturnValueOnce(firstStatus.promise)
        .mockResolvedValue(status({ latest_message: "Second" })),
      loadRuns: vi
        .fn()
        .mockReturnValueOnce(firstRuns.promise)
        .mockResolvedValue({ runs: [run("run-2")] }),
      applyRuns: vi
        .fn()
        .mockImplementationOnce(() => {})
        .mockImplementationOnce(() => {
          throw unexpected;
        }),
    });
    const scheduler = createGeminiBrowserRefreshScheduler(deps);

    const active = scheduler.scheduleRefresh();
    const trailingA = scheduler.scheduleRefresh();
    const trailingB = scheduler.scheduleRefresh();

    expect(trailingA).toBe(trailingB);
    const activeResolution = expect(active).resolves.toBeUndefined();
    const trailingRejection = expect(trailingA).rejects.toBe(unexpected);

    firstStatus.resolve(status({ latest_message: "First" }));
    firstRuns.resolve({ runs: [run("run-1")] });

    await activeResolution;
    await trailingRejection;
  });
});
