import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, expect, it, vi } from "vitest";
import type { AnalysisRunSummary } from "$lib/types/analysis";

const api = vi.hoisted(() => ({
  deleteAnalysisRun: vi.fn(), openConfirmModal: vi.fn(), listPromptPackRuns: vi.fn(),
  listActivePromptPackRuns: vi.fn(), listenToPromptPackRunEvents: vi.fn(),
}));
vi.mock("$lib/api/analysis-runs", () => ({ deleteAnalysisRun: api.deleteAnalysisRun }));
vi.mock("$lib/modals", () => ({ openConfirmModal: api.openConfirmModal }));
vi.mock("$lib/api/prompt-packs", () => ({
  cancelPromptPackRun: vi.fn(), deletePromptPackRun: vi.fn(),
  listPromptPackRuns: api.listPromptPackRuns, listActivePromptPackRuns: api.listActivePromptPackRuns,
  listenToPromptPackRunEvents: api.listenToPromptPackRunEvents,
}));

import ProjectRunsTab from "./ProjectRunsTab.svelte";

afterEach(cleanup);
afterEach(() => vi.clearAllMocks());

it("project runs tab delete controls > wires per-row delete controls for analysis project runs", async () => {
  api.openConfirmModal.mockResolvedValue(true);
  api.deleteAnalysisRun.mockResolvedValue(undefined);
  api.listPromptPackRuns.mockResolvedValue([]);
  api.listActivePromptPackRuns.mockResolvedValue([]);
  api.listenToPromptPackRunEvents.mockResolvedValue(vi.fn());
  const run: AnalysisRunSummary = {
    id: 71, run_type: "project", scope_type: "project", source_id: null, source_title: null,
    source_group_id: null, source_group_name: null, project_id: 1, project_name: "Smoke project",
    scope_label: "Smoke project", period_from: 0, period_to: 86_399, output_language: "en",
    prompt_template_id: 5, prompt_template_name: "Evidence brief", prompt_template_version: 2,
    provider_profile: "default", provider: "openai", model: "gpt-test",
    youtube_corpus_mode: "transcript_description", telegram_history_scope: "current",
    status: "completed", error: null, has_trace_data: false, snapshot_state: "captured",
    snapshot_captured_at: "2026-08-05T00:00:00Z", snapshot_error: null,
    created_at: 1_700_000_000, completed_at: 1_700_000_100,
  };
  const onRefreshProjectRuns = vi.fn();
  render(ProjectRunsTab, { runs: [run], projectId: 1, onRefreshProjectRuns });

  const button = screen.getByRole("button", { name: "Delete project analysis run 71" });
  expect((button as HTMLButtonElement).disabled).toBe(false);
  await fireEvent.click(button);
  await waitFor(() => expect(api.deleteAnalysisRun).toHaveBeenCalledWith(71));
  expect(api.openConfirmModal).toHaveBeenCalledWith(expect.objectContaining({ tone: "danger" }));
  expect(onRefreshProjectRuns).toHaveBeenCalledOnce();
});
