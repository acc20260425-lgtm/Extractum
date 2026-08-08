import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, expect, it, vi } from "vitest";
import type { PromptPackRunListItem } from "$lib/types/prompt-packs";

const api = vi.hoisted(() => ({
  cancelPromptPackRun: vi.fn(), deletePromptPackRun: vi.fn(), listActivePromptPackRuns: vi.fn(),
  listPromptPackRuns: vi.fn(), listenToPromptPackRunEvents: vi.fn(), openConfirmModal: vi.fn(),
}));
vi.mock("$lib/api/prompt-packs", () => ({
  cancelPromptPackRun: api.cancelPromptPackRun, deletePromptPackRun: api.deletePromptPackRun,
  listActivePromptPackRuns: api.listActivePromptPackRuns, listPromptPackRuns: api.listPromptPackRuns,
  listenToPromptPackRunEvents: api.listenToPromptPackRunEvents,
}));
vi.mock("$lib/modals", () => ({ openConfirmModal: api.openConfirmModal }));

import YoutubeSummaryRunsPanel from "./YoutubeSummaryRunsPanel.svelte";

afterEach(cleanup);
afterEach(() => vi.clearAllMocks());

it("project runs tab delete controls > wires per-row delete controls for prompt pack runs", async () => {
  const run: PromptPackRunListItem = {
    runId: 91, projectId: 1, runLabel: "Evidence summary", runtimeProvider: "api",
    packId: "youtube-summary", packVersion: "1", runStatus: "complete",
    resultStatus: "complete", latestMessage: "Prompt pack complete",
  };
  api.listPromptPackRuns.mockResolvedValue([run]);
  api.listActivePromptPackRuns.mockResolvedValue([]);
  api.listenToPromptPackRunEvents.mockResolvedValue(vi.fn());
  api.openConfirmModal.mockResolvedValue(true);
  api.deletePromptPackRun.mockResolvedValue(undefined);

  render(YoutubeSummaryRunsPanel, { projectId: 1 });
  const button = await screen.findByRole("button", { name: "Delete Prompt Pack run 91" });
  expect((button as HTMLButtonElement).disabled).toBe(false);
  await fireEvent.click(button);
  await waitFor(() => expect(api.deletePromptPackRun).toHaveBeenCalledWith(91));
  expect(api.openConfirmModal).toHaveBeenCalledWith(expect.objectContaining({ tone: "danger" }));
  await waitFor(() => expect(screen.queryByText("Run #91")).toBeNull());
});
