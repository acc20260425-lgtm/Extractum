import { beforeEach, describe, expect, it, vi } from "vitest";
import { createGeminiBrowserRefreshScheduler } from "./gemini-browser-refresh-scheduler";
import { createGeminiBrowserPollingController } from "./gemini-browser-polling";
import type {
  GeminiBrowserProviderStatus,
  GeminiBrowserRun,
} from "./types/gemini-browser";

describe("gemini browser polling controller", () => {
  let now = 0;

  beforeEach(() => {
    vi.useFakeTimers();
    now = 0;
  });

  it("uses idle cadence by default and active cadence when local pending work is fresh", async () => {
    const scheduleRefresh = vi.fn(async () => ({ allFailed: false }));
    const controller = createGeminiBrowserPollingController({
      scheduleRefresh,
      getActivitySnapshot: () => ({ runLogSignals: [], statusSignal: null }),
      now: () => now,
      idleMs: 5000,
      activeMs: 1000,
    });

    controller.start();
    await vi.advanceTimersByTimeAsync(5000);
    expect(scheduleRefresh).toHaveBeenCalledWith({ mode: "light" });

    controller.setLocalPendingRun("run-1");
    await vi.advanceTimersByTimeAsync(1000);
    expect(scheduleRefresh).toHaveBeenCalledTimes(2);

    controller.stop();
  });

  it("does not schedule a new polling refresh while one is in flight", async () => {
    let resolve!: () => void;
    const scheduleRefresh = vi.fn(
      () =>
        new Promise<{ allFailed: boolean }>((done) => {
          resolve = () => done({ allFailed: false });
        }),
    );
    const controller = createGeminiBrowserPollingController({
      scheduleRefresh,
      getActivitySnapshot: () => ({
        runLogSignals: [{ key: "run:active", updatedAt: "2026-06-22T00:00:00Z" }],
        statusSignal: null,
      }),
      now: () => Date.parse("2026-06-22T00:00:01Z"),
      idleMs: 5000,
      activeMs: 1000,
    });

    controller.start();
    await vi.advanceTimersToNextTimerAsync();
    await vi.advanceTimersByTimeAsync(5000);
    expect(scheduleRefresh).toHaveBeenCalledTimes(1);

    resolve();
    await Promise.resolve();
    controller.stop();
  });

  it("degrades active polling to idle cadence after three full polling failures", async () => {
    const scheduleRefresh = vi.fn(async () => ({ allFailed: true }));
    const controller = createGeminiBrowserPollingController({
      scheduleRefresh,
      getActivitySnapshot: () => ({
        runLogSignals: [{ key: "run:active", updatedAt: "2026-06-22T00:00:00Z" }],
        statusSignal: null,
      }),
      now: () => Date.parse("2026-06-22T00:00:01Z"),
      idleMs: 5000,
      activeMs: 1000,
      maxConsecutiveFailures: 3,
    });

    controller.start();
    for (let index = 0; index < 3; index += 1) {
      await vi.advanceTimersToNextTimerAsync();
      await Promise.resolve();
    }
    expect(scheduleRefresh).toHaveBeenCalledTimes(3);

    await vi.advanceTimersByTimeAsync(4999);
    expect(scheduleRefresh).toHaveBeenCalledTimes(3);
    await vi.advanceTimersByTimeAsync(1);
    expect(scheduleRefresh).toHaveBeenCalledTimes(4);
    controller.stop();
  });

  it("restores active cadence after a successful idle refresh", async () => {
    const scheduleRefresh = vi
      .fn()
      .mockResolvedValueOnce({ allFailed: true })
      .mockResolvedValueOnce({ allFailed: true })
      .mockResolvedValueOnce({ allFailed: true })
      .mockResolvedValue({ allFailed: false });
    const controller = createGeminiBrowserPollingController({
      scheduleRefresh,
      getActivitySnapshot: () => ({
        runLogSignals: [{ key: "run:active", updatedAt: "2026-06-22T00:00:00Z" }],
        statusSignal: null,
      }),
      now: () => Date.parse("2026-06-22T00:00:01Z"),
      idleMs: 5000,
      activeMs: 1000,
      maxConsecutiveFailures: 3,
    });

    controller.start();
    for (let index = 0; index < 3; index += 1) {
      await vi.advanceTimersToNextTimerAsync();
      await Promise.resolve();
    }
    await vi.advanceTimersByTimeAsync(5000);
    expect(scheduleRefresh).toHaveBeenCalledTimes(4);

    await vi.advanceTimersByTimeAsync(1000);
    expect(scheduleRefresh).toHaveBeenCalledTimes(5);
    controller.stop();
  });

  it("manual successful refresh outcome clears degraded polling state", async () => {
    const scheduleRefresh = vi.fn(async () => ({ allFailed: true }));
    const controller = createGeminiBrowserPollingController({
      scheduleRefresh,
      getActivitySnapshot: () => ({
        runLogSignals: [{ key: "run:active", updatedAt: "2026-06-22T00:00:00Z" }],
        statusSignal: null,
      }),
      now: () => Date.parse("2026-06-22T00:00:01Z"),
      idleMs: 5000,
      activeMs: 1000,
      maxConsecutiveFailures: 3,
    });

    controller.start();
    for (let index = 0; index < 3; index += 1) {
      await vi.advanceTimersToNextTimerAsync();
      await Promise.resolve();
    }

    controller.recordRefreshOutcome({ allFailed: false });
    await vi.advanceTimersByTimeAsync(1000);

    expect(scheduleRefresh).toHaveBeenCalledTimes(4);
    controller.stop();
  });

  it("clears local pending run only after confirmed terminal state", () => {
    const controller = createGeminiBrowserPollingController({
      scheduleRefresh: vi.fn(async () => ({ allFailed: false })),
      getActivitySnapshot: () => ({ runLogSignals: [], statusSignal: null }),
      now: () => now,
      idleMs: 5000,
      activeMs: 1000,
    });

    controller.setLocalPendingRun("run-1");
    expect(controller.hasLocalPendingRun()).toBe(true);
    controller.confirmPendingRunTerminal("run-1");
    expect(controller.hasLocalPendingRun()).toBe(false);
  });

  it("expires local pending runs after the grace window", async () => {
    const scheduleRefresh = vi.fn(async () => ({ allFailed: false }));
    const controller = createGeminiBrowserPollingController({
      scheduleRefresh,
      getActivitySnapshot: () => ({ runLogSignals: [], statusSignal: null }),
      now: () => now,
      idleMs: 5000,
      activeMs: 1000,
      activeGraceMs: 30 * 60 * 1000,
    });

    controller.setLocalPendingRun("run-1");
    now = 30 * 60 * 1000 + 1;
    controller.start();
    await vi.advanceTimersByTimeAsync(1000);
    expect(scheduleRefresh).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(4000);
    expect(scheduleRefresh).toHaveBeenCalledTimes(1);
    expect(controller.hasLocalPendingRun()).toBe(false);
    controller.stop();
  });

  it("treats stale run-log activity as idle after the grace window", async () => {
    const scheduleRefresh = vi.fn(async () => ({ allFailed: false }));
    const controller = createGeminiBrowserPollingController({
      scheduleRefresh,
      getActivitySnapshot: () => ({
        runLogSignals: [{ key: "run:old", updatedAt: "2026-06-22T00:00:00Z" }],
        statusSignal: null,
      }),
      now: () => Date.parse("2026-06-22T00:31:00Z"),
      idleMs: 5000,
      activeMs: 1000,
      activeGraceMs: 30 * 60 * 1000,
    });

    controller.start();
    await vi.advanceTimersByTimeAsync(1000);
    expect(scheduleRefresh).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(4000);
    expect(scheduleRefresh).toHaveBeenCalledTimes(1);
    controller.stop();
  });

  it("treats stale status-derived activity as idle after the grace window", async () => {
    const scheduleRefresh = vi.fn(async () => ({ allFailed: false }));
    const controller = createGeminiBrowserPollingController({
      scheduleRefresh,
      getActivitySnapshot: () => ({ runLogSignals: [], statusSignal: "status:running" }),
      now: () => now,
      idleMs: 5000,
      activeMs: 1000,
      activeGraceMs: 30 * 60 * 1000,
    });

    controller.start();
    await vi.advanceTimersByTimeAsync(1000);
    expect(scheduleRefresh).toHaveBeenCalledTimes(1);

    now = 30 * 60 * 1000 + 1;
    await vi.advanceTimersByTimeAsync(1000);
    expect(scheduleRefresh).toHaveBeenCalledTimes(1);
    await vi.advanceTimersByTimeAsync(5000);
    expect(scheduleRefresh).toHaveBeenCalledTimes(2);
    controller.stop();
  });

  it("requires two not-found confirmations after rejected pending run", () => {
    const controller = createGeminiBrowserPollingController({
      scheduleRefresh: vi.fn(async () => ({ allFailed: false })),
      getActivitySnapshot: () => ({ runLogSignals: [], statusSignal: null }),
      now: () => now,
    });

    controller.setLocalPendingRun("run-1");
    expect(controller.hasLocalPendingRun("run-1")).toBe(true);
    expect(controller.hasLocalPendingRun("other-run")).toBe(false);
    controller.markLocalPendingRunRejected("run-1");
    controller.confirmPendingRunNotFound("run-1");
    expect(controller.hasLocalPendingRun()).toBe(true);
    controller.confirmPendingRunNotFound("run-1");
    expect(controller.hasLocalPendingRun()).toBe(false);
  });

  it("idle polling discovers prompt-pack runs through scheduler light refresh without live status", async () => {
    const cachedStatus: GeminiBrowserProviderStatus = {
      status: "ready",
      manual_action: null,
      active_run_id: null,
      queue_depth: 0,
      browser_profile_dir: "profile-dir",
      latest_message: "Cached",
    };
    const promptPackRun: GeminiBrowserRun = {
      run_id: "prompt-pack-run",
      source: "prompt_pack",
      status: "ok",
      prompt_preview: "Summarize",
      created_at: "2026-06-22T00:00:00Z",
      updated_at: "2026-06-22T00:00:01Z",
      result: null,
    };
    const loadStatus = vi.fn(async () => cachedStatus);
    const loadStatusSnapshot = vi.fn(async () => cachedStatus);
    const applyRuns = vi.fn();
    const scheduler = createGeminiBrowserRefreshScheduler({
      loadStatus,
      loadStatusSnapshot,
      loadRuns: vi.fn(async () => ({ runs: [promptPackRun] })),
      loadRun: vi.fn(async () => promptPackRun),
      getSelectedRunId: vi.fn(() => null),
      getSelectedDetailToken: vi.fn(() => 0),
      applyStatus: vi.fn(),
      applyRuns,
      applySelectedRun: vi.fn(),
      applySelectedRunUnavailable: vi.fn(),
      applySelectedRunError: vi.fn(),
      applyStatusError: vi.fn(),
      applyRunsError: vi.fn(),
      applyMessage: vi.fn(),
      syncActivePromptResult: vi.fn(),
      formatError: (_context, error) => String(error),
      isRunNotFoundError: vi.fn(() => false),
    });
    const controller = createGeminiBrowserPollingController({
      scheduleRefresh: scheduler.scheduleRefresh,
      getActivitySnapshot: () => ({ runLogSignals: [], statusSignal: null }),
      idleMs: 5000,
      activeMs: 1000,
    });

    controller.start();
    await vi.advanceTimersByTimeAsync(5000);

    expect(loadStatusSnapshot).toHaveBeenCalledTimes(1);
    expect(loadStatus).not.toHaveBeenCalled();
    expect(applyRuns).toHaveBeenCalledWith([promptPackRun]);
    controller.stop();
  });
});
