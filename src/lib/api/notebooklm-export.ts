import { invoke } from "@tauri-apps/api/core";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  NotebookLmExportEvent,
  NotebookLmExportRequest,
  NotebookLmExportResult,
} from "$lib/types/sources";

export const NOTEBOOKLM_EXPORT_EVENT = "notebooklm://export";

export function exportSourceToNotebookLm(request: NotebookLmExportRequest) {
  return invoke<NotebookLmExportResult>("export_source_to_notebooklm", { request });
}

export function listenToNotebookLmExportEvents(
  handler: (event: Event<NotebookLmExportEvent>) => void,
): Promise<UnlistenFn> {
  return listen<NotebookLmExportEvent>(NOTEBOOKLM_EXPORT_EVENT, handler);
}
