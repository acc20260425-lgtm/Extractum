import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  NOTEBOOKLM_EXPORT_EVENT,
  exportSourceToNotebookLm,
  listenToNotebookLmExportEvents,
} from "./notebooklm-export";
import type {
  NotebookLmExportEvent,
  NotebookLmExportRequest,
  NotebookLmExportResult,
} from "$lib/types/sources";

const invokeMock = vi.hoisted(() => vi.fn());
const listenMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: listenMock,
}));

function notebookLmExportRequest(
  overrides: Partial<NotebookLmExportRequest> = {},
): NotebookLmExportRequest {
  return {
    export_id: "export-a",
    source_id: 7,
    source_group_id: null,
    output_dir: "C:/Exports",
    period_from: 1_700_000,
    period_to: 1_786_000,
    include_media_placeholders: true,
    include_migrated_history: false,
    min_message_length: 5,
    max_words_per_file: 1000,
    max_bytes_per_file: 5000,
    overwrite_existing: false,
    ...overrides,
  };
}

function notebookLmExportResult(
  overrides: Partial<NotebookLmExportResult> = {},
): NotebookLmExportResult {
  return {
    output_dir: "C:/Exports",
    files: [
      {
        path: "C:/Exports/source.md",
        message_count: 12,
        byte_size: 1024,
        approximate_word_count: 300,
      },
    ],
    glossary_file: null,
    exported_message_count: 12,
    skipped_message_count: 2,
    warning_count: 0,
    warnings: [],
    ...overrides,
  };
}

describe("notebooklm export api wrapper", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
  });

  it("exports a source for NotebookLM with the existing command and request payload", async () => {
    const request = notebookLmExportRequest();
    const result = notebookLmExportResult();
    invokeMock.mockResolvedValueOnce(result);

    await expect(exportSourceToNotebookLm(request)).resolves.toBe(result);

    expect(invokeMock).toHaveBeenLastCalledWith("export_source_to_notebooklm", {
      request,
    });
  });

  it("listens on the shared NotebookLM export event name", async () => {
    const unlisten = vi.fn();
    const handler = vi.fn();
    listenMock.mockResolvedValueOnce(unlisten);

    await expect(listenToNotebookLmExportEvents(handler)).resolves.toBe(unlisten);
    expect(NOTEBOOKLM_EXPORT_EVENT).toBe("notebooklm://export");
    expect(listenMock).toHaveBeenCalledWith(NOTEBOOKLM_EXPORT_EVENT, expect.any(Function));

    const payload: NotebookLmExportEvent = {
      export_id: "export-a",
      source_id: 7,
      kind: "progress",
      phase: "writing",
      message: "Writing files",
      progress_current: 2,
      progress_total: 5,
      file_path: "C:/Exports/source.md",
      error: null,
    };
    const event = { payload };

    listenMock.mock.calls[0][1](event);

    expect(handler).toHaveBeenCalledWith(event);
  });
});
