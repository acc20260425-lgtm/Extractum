import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import "$lib/testing/dom-assertions";
const api = vi.hoisted(() => ({ listPromptPackRuns: vi.fn(), listActivePromptPackRuns: vi.fn(), listenToPromptPackRunEvents: vi.fn(), updatePromptPackRun: vi.fn(), deletePromptPackRun: vi.fn(), cancelPromptPackRun: vi.fn(), openConfirmModal: vi.fn() }));
vi.mock("$lib/api/prompt-packs", () => ({ listPromptPackRuns: api.listPromptPackRuns, listActivePromptPackRuns: api.listActivePromptPackRuns, listenToPromptPackRunEvents: api.listenToPromptPackRunEvents, updatePromptPackRun: api.updatePromptPackRun, deletePromptPackRun: api.deletePromptPackRun, cancelPromptPackRun: api.cancelPromptPackRun, getPromptPackResult: vi.fn(), listPromptPackRunStages: vi.fn().mockResolvedValue([]), getPromptPackValidationFindings: vi.fn().mockResolvedValue([]), listPromptPackAuditEvents: vi.fn().mockResolvedValue([]), listPromptPackStageArtifacts: vi.fn().mockResolvedValue([]), getPromptPackStageArtifact: vi.fn() }));
vi.mock("$lib/modals", () => ({ openConfirmModal: api.openConfirmModal }));
vi.mock("@svar-ui/svelte-core", async () => ({ Locale: (await import("$lib/testing/SvarLocaleReceiver.svelte")).default }));
vi.mock("@svar-ui/svelte-grid", async () => ({ Grid: (await import("$lib/testing/SvarGridReceiver.svelte")).default, Willow: (await import("$lib/testing/SvarWillowReceiver.svelte")).default }));
vi.mock("@svar-ui/core-locales", () => ({ ru: {} })); vi.mock("@svar-ui/grid-locales", () => ({ en: {} }));
import ProjectRunsScreen from "./ProjectRunsScreen.svelte";
const run = (overrides = {}) => ({ runId: 91, projectId: 5, runLabel: "Evidence report", runtimeProvider: "api", packId: "youtube-summary", packVersion: "1", runStatus: "complete", resultStatus: "complete", latestMessage: "Complete", progressCurrent: 2, progressTotal: 2, queuePosition: null, createdAt: "2026-08-09T10:00:00Z", completedAt: "2026-08-09T10:01:00Z", ...overrides });
beforeEach(() => { api.listPromptPackRuns.mockResolvedValue([run()]); api.listActivePromptPackRuns.mockResolvedValue([]); api.listenToPromptPackRunEvents.mockResolvedValue(() => {}); api.updatePromptPackRun.mockResolvedValue(run({ runLabel: "Updated" })); api.deletePromptPackRun.mockResolvedValue(undefined); api.cancelPromptPackRun.mockResolvedValue(undefined); api.openConfirmModal.mockResolvedValue(true); });
afterEach(() => { cleanup(); vi.clearAllMocks(); });
describe("project runs screen", () => {
  it("uses the Extractum SVAR grid for prompt-pack project runs with update and delete actions", async () => {
    render(ProjectRunsScreen); const grid = await screen.findByRole("grid", { name: "Prompt Pack runs" });
    expect(grid).toBeTruthy(); expect(JSON.parse(grid.dataset.rowIds ?? "[]")).toEqual(["91"]); expect(screen.getByText("Evidence report")).toBeTruthy(); expect(screen.getByRole("textbox", { name: "Run label" })).toHaveValue("Evidence report");
    await fireEvent.input(screen.getByRole("textbox", { name: "Run label" }), { target: { value: "Updated" } }); await fireEvent.click(screen.getByRole("button", { name: "Update selected prompt pack run label" }));
    await waitFor(() => expect(api.updatePromptPackRun).toHaveBeenCalledWith({ runId: 91, runLabel: "Updated" })); expect(screen.getByText("Run label updated.")).toBeTruthy(); expect(screen.getByRole("button", { name: "Delete selected prompt pack run 91" })).toBeEnabled();
  });
  it("marks run date columns for locale-aware datetime formatting", async () => {
    render(ProjectRunsScreen); const grid = await screen.findByRole("grid", { name: "Prompt Pack runs" }); const templateColumnIds = JSON.parse(grid.dataset.templateColumnIds ?? "[]");
    expect(templateColumnIds).toContain("createdAt"); expect(templateColumnIds).toContain("completedAt");
  });
  it("uses the shared confirm modal before deleting or cancelling prompt-pack runs", async () => {
    render(ProjectRunsScreen); await screen.findByText("Evidence report"); await fireEvent.click(screen.getByRole("button", { name: "Delete selected prompt pack run 91" }));
    await waitFor(() => expect(api.openConfirmModal).toHaveBeenCalledWith(expect.objectContaining({ title: "Delete Prompt Pack run?", confirmLabel: "Delete", tone: "danger" })));
    expect(api.deletePromptPackRun).toHaveBeenCalledWith(91); expect(screen.getByText("Run deleted.")).toBeTruthy();
    cleanup();
    api.listPromptPackRuns.mockResolvedValue([run({ runStatus: "running", resultStatus: "none" })]);
    render(ProjectRunsScreen); await screen.findByText("Evidence report"); await fireEvent.click(screen.getByRole("button", { name: "Cancel selected prompt pack run" }));
    await waitFor(() => expect(api.openConfirmModal).toHaveBeenLastCalledWith(expect.objectContaining({ title: "Cancel active Prompt Pack run?", confirmLabel: "Cancel run", tone: "danger" })));
    expect(api.cancelPromptPackRun).toHaveBeenCalledWith(91); expect(api.openConfirmModal).toHaveBeenCalledTimes(2);
  });
});
