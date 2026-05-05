import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  ANALYSIS_CHAT_EVENT,
  askAnalysisRunQuestion,
  clearAnalysisChatMessages,
  listAnalysisChatMessages,
  listenToAnalysisChatEvents,
} from "./analysis-chat";
import type { AnalysisChatEvent, AnalysisChatMessage } from "$lib/types/analysis";

const invokeMock = vi.hoisted(() => vi.fn());
const listenMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: listenMock,
}));

function chatMessage(overrides: Partial<AnalysisChatMessage> = {}): AnalysisChatMessage {
  return {
    id: 1,
    run_id: 7,
    role: "user",
    content: "Question",
    created_at: 1_700_000,
    ...overrides,
  };
}

function chatEvent(overrides: Partial<AnalysisChatEvent> = {}): AnalysisChatEvent {
  return {
    request_id: "chat-1",
    run_id: 7,
    kind: "delta",
    queue_position: null,
    delta: "Hello",
    message: null,
    error: null,
    ...overrides,
  };
}

describe("analysis chat api wrapper", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
  });

  it("loads persisted chat messages for a run", async () => {
    invokeMock.mockResolvedValueOnce([chatMessage()]);

    await expect(listAnalysisChatMessages(7)).resolves.toEqual([chatMessage()]);

    expect(invokeMock).toHaveBeenLastCalledWith("list_analysis_chat_messages", {
      runId: 7,
    });
  });

  it("starts an analysis chat answer with typed arguments", async () => {
    invokeMock.mockResolvedValueOnce("chat-1");

    await expect(askAnalysisRunQuestion({
      runId: 7,
      question: "What changed?",
      modelOverride: "gemini-2.5-flash",
      profileId: null,
    })).resolves.toBe("chat-1");

    expect(invokeMock).toHaveBeenLastCalledWith("ask_analysis_run_question", {
      runId: 7,
      question: "What changed?",
      modelOverride: "gemini-2.5-flash",
      profileId: null,
    });
  });

  it("clears persisted chat messages for a run", async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await expect(clearAnalysisChatMessages(7)).resolves.toBeUndefined();

    expect(invokeMock).toHaveBeenLastCalledWith("clear_analysis_chat_messages", {
      runId: 7,
    });
  });

  it("listens on the shared analysis chat event name", async () => {
    const unlisten = vi.fn();
    const handler = vi.fn();
    listenMock.mockResolvedValueOnce(unlisten);

    await expect(listenToAnalysisChatEvents(handler)).resolves.toBe(unlisten);
    expect(ANALYSIS_CHAT_EVENT).toBe("analysis://chat");
    expect(listenMock).toHaveBeenCalledWith(ANALYSIS_CHAT_EVENT, expect.any(Function));

    const event = { payload: chatEvent() };
    listenMock.mock.calls[0][1](event);

    expect(handler).toHaveBeenCalledWith(event);
  });
});
