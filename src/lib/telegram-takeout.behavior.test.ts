import { beforeEach, expect, it, vi } from "vitest";
import {
  TAKEOUT_IMPORT_EVENT,
  cancelTakeoutSourceImport,
  listTakeoutImportRecoveryStates,
  listTakeoutSourceImportJobs,
  listenToTakeoutImportEvents,
  startTakeoutMigratedHistoryImport,
  startTakeoutSourceImport,
} from "$lib/api/takeout-import";
import { TAKEOUT_IMPORT_PHASES, type TakeoutImportEvent } from "$lib/types/sources";

const invokeMock = vi.hoisted(() => vi.fn());
const listenMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/api/event", () => ({ listen: listenMock }));

beforeEach(() => {
  invokeMock.mockReset();
  listenMock.mockReset();
});

it("freezes Takeout behavior through Checkpoint 6 and hands CP7 paths to Rust behavior tests", async () => {
  const unlisten = vi.fn();
  const eventHandler = vi.fn();
  const event: TakeoutImportEvent = {
    job_id: "takeout-8",
    source_id: 9,
    account_id: 7,
    batch_id: 14,
    history_scope: "migrated_small_group_history",
    status: "completed",
    phase: "completed",
    message: "Completed",
    inserted: 10,
    skipped: 2,
    progress_current: 12,
    progress_total: 12,
    started_at: 100,
    finished_at: 101,
    warnings: [],
    error: null,
  };
  listenMock.mockResolvedValueOnce(unlisten);
  invokeMock
    .mockResolvedValueOnce([])
    .mockResolvedValueOnce([])
    .mockResolvedValueOnce({ job_id: "takeout-current" })
    .mockResolvedValueOnce({ job_id: "takeout-migrated" })
    .mockResolvedValueOnce({ cancelled: true });

  await expect(listTakeoutSourceImportJobs()).resolves.toEqual([]);
  await expect(listTakeoutImportRecoveryStates()).resolves.toEqual([]);
  await expect(startTakeoutSourceImport(9)).resolves.toEqual({ job_id: "takeout-current" });
  await expect(startTakeoutMigratedHistoryImport(9)).resolves.toEqual({ job_id: "takeout-migrated" });
  await expect(cancelTakeoutSourceImport("takeout-migrated")).resolves.toEqual({ cancelled: true });
  await expect(listenToTakeoutImportEvents(eventHandler)).resolves.toBe(unlisten);
  listenMock.mock.calls[0][1]({ payload: event });

  expect(invokeMock.mock.calls).toEqual([
    ["list_takeout_source_import_jobs"],
    ["list_takeout_import_recovery_states"],
    ["start_takeout_source_import", { sourceId: 9 }],
    ["start_takeout_migrated_history_import", { sourceId: 9 }],
    ["cancel_takeout_source_import", { jobId: "takeout-migrated" }],
  ]);
  expect(listenMock).toHaveBeenCalledWith(TAKEOUT_IMPORT_EVENT, expect.any(Function));
  expect(eventHandler).toHaveBeenCalledWith({ payload: event });
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
