import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  TAKEOUT_IMPORT_EVENT,
  cancelTakeoutSourceImport,
  listTakeoutImportRecoveryStates,
  listTakeoutSourceImportJobs,
  listenToTakeoutImportEvents,
  startTakeoutSourceImport,
} from "./takeout-import";
import type { TakeoutImportEvent } from "$lib/types/sources";
import { TAKEOUT_IMPORT_PHASES } from "$lib/types/sources";

const invokeMock = vi.hoisted(() => vi.fn());
const listenMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: listenMock,
}));

describe("takeout import api wrapper", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
  });

  it("lists takeout import jobs with the existing command", async () => {
    invokeMock.mockResolvedValueOnce([]);

    await expect(listTakeoutSourceImportJobs()).resolves.toEqual([]);

    expect(invokeMock).toHaveBeenLastCalledWith("list_takeout_source_import_jobs");
  });

  it("lists takeout import recovery states with the read-only command", async () => {
    invokeMock.mockResolvedValueOnce([]);

    await expect(listTakeoutImportRecoveryStates()).resolves.toEqual([]);

    expect(invokeMock).toHaveBeenLastCalledWith("list_takeout_import_recovery_states");
  });

  it("starts a takeout import for a source", async () => {
    invokeMock.mockResolvedValueOnce({ job_id: "takeout-1" });

    await expect(startTakeoutSourceImport(7)).resolves.toEqual({
      job_id: "takeout-1",
    });

    expect(invokeMock).toHaveBeenLastCalledWith("start_takeout_source_import", {
      sourceId: 7,
    });
  });

  it("cancels a takeout import job", async () => {
    invokeMock.mockResolvedValueOnce({ cancelled: true });

    await expect(cancelTakeoutSourceImport("takeout-1")).resolves.toEqual({
      cancelled: true,
    });

    expect(invokeMock).toHaveBeenLastCalledWith("cancel_takeout_source_import", {
      jobId: "takeout-1",
    });
  });

  it("listens on the shared takeout import event name", async () => {
    const unlisten = vi.fn();
    const handler = vi.fn();
    listenMock.mockResolvedValueOnce(unlisten);

    await expect(listenToTakeoutImportEvents(handler)).resolves.toBe(unlisten);
    expect(TAKEOUT_IMPORT_EVENT).toBe("sources://takeout-import");
    expect(listenMock).toHaveBeenCalledWith(TAKEOUT_IMPORT_EVENT, expect.any(Function));

    const payload: TakeoutImportEvent = {
      job_id: "takeout-1",
      source_id: 7,
      account_id: 2,
      batch_id: 100,
      status: "running",
      phase: "importing_history",
      message: "Importing",
      inserted: 12,
      skipped: 1,
      progress_current: 12,
      progress_total: 40,
      started_at: 1_700_000,
      finished_at: null,
      warnings: [],
      error: null,
    };
    const event = { payload };

    listenMock.mock.calls[0][1](event);

    expect(handler).toHaveBeenCalledWith(event);
  });

  it("pins frontend takeout import phases to the Rust emitted phases", () => {
    expect(TAKEOUT_IMPORT_PHASES).toEqual([
      "queued",
      "resolving_source",
      "starting_takeout",
      "validating_peer",
      "loading_splits",
      "counting",
      "importing_history",
      "finishing_takeout",
      "completed",
      "failed",
      "cancelled",
    ]);
  });
});
