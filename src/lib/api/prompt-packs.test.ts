import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  getPromptPackLibrary,
  getPromptPackResult,
  getPromptPackStageArtifact,
  getPromptPackValidationFindings,
  deletePromptPackRun,
  listenToPromptPackRunEvents,
  listActivePromptPackRuns,
  listPromptPackAuditEvents,
  listPromptPackRunStages,
  listPromptPackStageArtifacts,
  listPromptPackRuns,
  PROMPT_PACK_RUN_EVENT,
  startYoutubeSummaryRun,
  updatePromptPackRun,
} from "./prompt-packs";

const invokeMock = vi.hoisted(() => vi.fn());
const listenMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: listenMock,
}));

describe("prompt pack api wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
  });

  it("loads prompt pack library with the registered command name", async () => {
    invokeMock.mockResolvedValueOnce({ packs: [] });

    await expect(getPromptPackLibrary()).resolves.toEqual({ packs: [] });

    expect(invokeMock).toHaveBeenCalledWith("get_prompt_pack_library");
  });

  it("starts youtube summary run", async () => {
    invokeMock.mockResolvedValueOnce({
      kind: "started",
      run: { runId: 42, runStatus: "queued", latestMessage: "Queued" },
    });

    await startYoutubeSummaryRun({
      clientRequestId: "req-ui-start-1",
      projectId: null,
      sourceIds: [901],
      profileId: null,
      modelOverride: null,
      outputLanguage: "en",
      controlPreset: "standard",
      evidenceMode: "standard",
      includeComments: false,
    });

    expect(invokeMock).toHaveBeenCalledWith("start_youtube_summary_run", {
      clientRequestId: "req-ui-start-1",
      projectId: null,
      sourceIds: [901],
      profileId: null,
      modelOverride: null,
      outputLanguage: "en",
      controlPreset: "standard",
      evidenceMode: "standard",
      includeComments: false,
    });
  });

  it("returns blocked start outcome without hiding fresh preflight failures", async () => {
    invokeMock.mockResolvedValueOnce({
      kind: "blocked",
      preflight: {
        packId: "youtube_summary",
        packVersion: "1.0.0",
        includedVideos: [],
        skippedVideos: [],
        blockingFailures: [{ sourceId: 10, reason: "no_included_videos" }],
        estimatedInputTokens: 0,
        selectedModelInputLimit: 32000,
      },
    });

    const outcome = await startYoutubeSummaryRun({
      clientRequestId: "req-ui-blocked-1",
      projectId: null,
      sourceIds: [10],
      profileId: null,
      modelOverride: null,
      outputLanguage: "en",
      controlPreset: "standard",
      evidenceMode: "standard",
      includeComments: false,
    });

    if (outcome.kind !== "blocked") {
      throw new Error(`expected blocked outcome, got ${outcome.kind}`);
    }

    expect(outcome.preflight.blockingFailures).toHaveLength(1);
  });

  it("listens to prompt pack run events", async () => {
    const handler = vi.fn();

    await listenToPromptPackRunEvents(handler);

    expect(PROMPT_PACK_RUN_EVENT).toBe("prompt-pack-run-event");
    expect(listenMock).toHaveBeenCalledWith("prompt-pack-run-event", expect.any(Function));
  });

  it("lists recent prompt pack runs", async () => {
    await listPromptPackRuns({ projectId: 7, limit: 20 });

    expect(invokeMock).toHaveBeenCalledWith("list_prompt_pack_runs", {
      projectId: 7,
      limit: 20,
    });
  });

  it("wraps project run update and delete commands", async () => {
    invokeMock.mockResolvedValueOnce({
      runId: 42,
      projectId: 7,
      packId: "youtube_summary",
      packVersion: "1.0.0",
      runStatus: "complete",
      resultStatus: "complete",
      runLabel: "June summary",
      createdAt: "2026-06-14T00:00:00Z",
    });

    await expect(updatePromptPackRun({ runId: 42, runLabel: "June summary" })).resolves.toMatchObject({
      runId: 42,
      runLabel: "June summary",
    });
    expect(invokeMock).toHaveBeenLastCalledWith("update_prompt_pack_run", {
      runId: 42,
      runLabel: "June summary",
    });

    invokeMock.mockResolvedValueOnce(undefined);
    await expect(deletePromptPackRun(42)).resolves.toBeUndefined();
    expect(invokeMock).toHaveBeenLastCalledWith("delete_prompt_pack_run", { runId: 42 });
  });

  it("keeps execution result artifact and audit wrappers available", async () => {
    await listActivePromptPackRuns();
    expect(invokeMock).toHaveBeenCalledWith("list_active_prompt_pack_runs");

    await listPromptPackRunStages(42);
    expect(invokeMock).toHaveBeenCalledWith("list_prompt_pack_run_stages", { runId: 42 });

    await getPromptPackResult(42);
    expect(invokeMock).toHaveBeenCalledWith("get_prompt_pack_result", { runId: 42 });

    await getPromptPackValidationFindings(42);
    expect(invokeMock).toHaveBeenCalledWith("get_prompt_pack_validation_findings", { runId: 42 });

    await listPromptPackStageArtifacts(1001);
    expect(invokeMock).toHaveBeenCalledWith("list_prompt_pack_stage_artifacts", {
      stageRunId: 1001,
    });

    await getPromptPackStageArtifact({
      stageRunId: 1001,
      artifactKind: "raw_output",
      attemptNumber: 1,
      artifactIndex: 2,
    });
    expect(invokeMock).toHaveBeenCalledWith("get_prompt_pack_stage_artifact", {
      stageRunId: 1001,
      artifactKind: "raw_output",
      attemptNumber: 1,
      artifactIndex: 2,
    });

    await listPromptPackAuditEvents(42);
    expect(invokeMock).toHaveBeenCalledWith("list_prompt_pack_audit_events", {
      runId: 42,
    });
  });
});
