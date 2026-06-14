import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  getPromptPackLibrary,
  listenToPromptPackRunEvents,
  listPromptPackRuns,
  PROMPT_PACK_RUN_EVENT,
  startYoutubeSummaryRun,
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
});
