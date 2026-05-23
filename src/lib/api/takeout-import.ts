import { invoke } from "@tauri-apps/api/core";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  CancelTakeoutImportResponse,
  StartTakeoutImportResponse,
  TakeoutImportEvent,
  TakeoutImportJobRecord,
  TakeoutImportRecoveryState,
} from "$lib/types/sources";

export const TAKEOUT_IMPORT_EVENT = "sources://takeout-import";

export function listTakeoutSourceImportJobs() {
  return invoke<TakeoutImportJobRecord[]>("list_takeout_source_import_jobs");
}

export function listTakeoutImportRecoveryStates() {
  return invoke<TakeoutImportRecoveryState[]>("list_takeout_import_recovery_states");
}

export function startTakeoutSourceImport(sourceId: number) {
  return invoke<StartTakeoutImportResponse>("start_takeout_source_import", { sourceId });
}

export function cancelTakeoutSourceImport(jobId: string) {
  return invoke<CancelTakeoutImportResponse>("cancel_takeout_source_import", { jobId });
}

export function listenToTakeoutImportEvents(
  handler: (event: Event<TakeoutImportEvent>) => void,
): Promise<UnlistenFn> {
  return listen<TakeoutImportEvent>(TAKEOUT_IMPORT_EVENT, handler);
}
