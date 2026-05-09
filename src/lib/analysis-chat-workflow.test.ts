import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  createAnalysisChatWorkflow,
  type AnalysisChatWorkflowPatch,
  type AnalysisChatWorkflowState,
} from "./analysis-chat-workflow";
import type {
  AnalysisChatEvent,
  AnalysisChatMessage,
  AnalysisChatTurn,
  AnalysisRunDetail,
  AnalysisRunSummary,
} from "./types/analysis";

function runSummary(overrides: Partial<AnalysisRunSummary> = {}): AnalysisRunSummary {
  return {
    id: 7,
    run_type: "report",
    scope_type: "single_source",
    source_id: 2,
    source_title: "Source",
    source_group_id: null,
    source_group_name: null,
    scope_label: "Source",
    period_from: 100,
    period_to: 200,
    output_language: "Russian",
    prompt_template_id: 3,
    prompt_template_name: "Template",
    prompt_template_version: 1,
    provider_profile: "default",
    provider: "gemini",
    model: "gemini-2.5-flash",
    status: "completed",
    error: null,
    has_trace_data: false,
    created_at: 100,
    completed_at: 200,
    ...overrides,
  };
}

function runDetail(overrides: Partial<AnalysisRunDetail> = {}): AnalysisRunDetail {
  return {
    ...runSummary(overrides),
    result_markdown: "Saved report",
    ...overrides,
  };
}

function turn(role: AnalysisChatTurn["role"], content: string): AnalysisChatTurn {
  return { role, content };
}

function chatMessage(overrides: Partial<AnalysisChatMessage> = {}): AnalysisChatMessage {
  return {
    id: 1,
    run_id: 7,
    role: "user",
    content: "Saved question",
    created_at: 1_700_000,
    ...overrides,
  };
}

function chatEvent(overrides: Partial<AnalysisChatEvent> = {}): AnalysisChatEvent {
  return {
    request_id: "request-1",
    run_id: 7,
    kind: "delta",
    queue_position: null,
    delta: null,
    message: null,
    error: null,
    ...overrides,
  };
}

type HarnessState = AnalysisChatWorkflowState & {
  loadingChat: boolean;
  clearingChat: boolean;
  status: string;
};

function createHarness(initial: Partial<HarnessState> = {}) {
  const state: HarnessState = {
    currentRun: runDetail(),
    chatQuestion: "",
    chatMessages: [],
    chatting: false,
    activeChatRequestId: null,
    activeChatRunId: null,
    profileId: null,
    modelOverride: "",
    loadingChat: false,
    clearingChat: false,
    status: "",
    ...initial,
  };

  const deps = {
    getState: () => state,
    patch: vi.fn((patch: AnalysisChatWorkflowPatch) => Object.assign(state, patch)),
    listMessages: vi.fn(),
    askQuestion: vi.fn(),
    clearMessages: vi.fn(),
    cancelRequest: vi.fn(),
    confirmClearChat: vi.fn(),
    formatError: vi.fn((action: string, error: unknown) => `Error ${action}: ${String(error)}`),
  };

  const workflow = createAnalysisChatWorkflow(deps);
  return { state, deps, workflow };
}

describe("analysis-chat-workflow", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("loads persisted messages and maps them into chat turns", async () => {
    const { state, deps, workflow } = createHarness();
    deps.listMessages.mockResolvedValueOnce([
      chatMessage({ role: "user", content: "Saved question" }),
      chatMessage({ id: 2, role: "assistant", content: "Saved answer" }),
    ]);

    await workflow.loadMessages(7);

    expect(deps.listMessages).toHaveBeenCalledWith(7);
    expect(state.chatMessages).toEqual([
      turn("user", "Saved question"),
      turn("assistant", "Saved answer"),
    ]);
    expect(state.loadingChat).toBe(false);
  });

  it("ignores stale guarded message loads", async () => {
    const { state, deps, workflow } = createHarness({
      chatMessages: [turn("assistant", "existing")],
    });
    deps.listMessages.mockResolvedValueOnce([chatMessage()]);

    await workflow.loadMessages(7, { isCurrent: () => false });

    expect(state.chatMessages).toEqual([turn("assistant", "existing")]);
    expect(state.loadingChat).toBe(false);
  });

  it("clears messages and reports status when loading fails", async () => {
    const { state, deps, workflow } = createHarness({
      chatMessages: [turn("assistant", "existing")],
    });
    deps.listMessages.mockRejectedValueOnce("db down");

    await workflow.loadMessages(7);

    expect(state.chatMessages).toEqual([]);
    expect(state.status).toBe("Error loading analysis chat: db down");
    expect(state.loadingChat).toBe(false);
  });

  it("requires a completed run and a non-empty question before asking", async () => {
    const noRun = createHarness({ currentRun: null, chatQuestion: "Hello" });
    await noRun.workflow.askRunQuestion();
    expect(noRun.state.status).toBe("Open a completed report first.");
    expect(noRun.deps.askQuestion).not.toHaveBeenCalled();

    const running = createHarness({
      currentRun: runDetail({ status: "running" }),
      chatQuestion: "Hello",
    });
    await running.workflow.askRunQuestion();
    expect(running.state.status).toBe("Open a completed report first.");
    expect(running.deps.askQuestion).not.toHaveBeenCalled();

    const empty = createHarness({ chatQuestion: "   " });
    await empty.workflow.askRunQuestion();
    expect(empty.state.status).toBe("Question cannot be empty.");
    expect(empty.deps.askQuestion).not.toHaveBeenCalled();
  });

  it("optimistically appends a pending exchange and stores the request id", async () => {
    const { state, deps, workflow } = createHarness({
      chatQuestion: "  What changed?  ",
      chatMessages: [turn("assistant", "Ready")],
      profileId: "research",
      modelOverride: " gemini-2.5-flash ",
    });
    deps.askQuestion.mockResolvedValueOnce("request-1");

    await workflow.askRunQuestion();

    expect(state.chatQuestion).toBe("");
    expect(state.chatting).toBe(true);
    expect(state.activeChatRunId).toBe(7);
    expect(state.activeChatRequestId).toBe("request-1");
    expect(state.chatMessages).toEqual([
      turn("assistant", "Ready"),
      turn("user", "What changed?"),
      turn("assistant", ""),
    ]);
    expect(deps.askQuestion).toHaveBeenCalledWith({
      runId: 7,
      question: "What changed?",
      modelOverride: "gemini-2.5-flash",
      profileId: "research",
    });
  });

  it("rolls back the pending exchange when asking fails", async () => {
    const { state, deps, workflow } = createHarness({
      chatQuestion: "What changed?",
    });
    deps.askQuestion.mockRejectedValueOnce("model unavailable");

    await workflow.askRunQuestion();

    expect(state.chatMessages).toEqual([]);
    expect(state.chatting).toBe(false);
    expect(state.activeChatRunId).toBeNull();
    expect(state.activeChatRequestId).toBeNull();
    expect(state.status).toBe("Error starting the chat answer: model unavailable");
  });

  it("cancels active chat requests with visible and silent status handling", async () => {
    const visible = createHarness({ activeChatRequestId: "request-1" });
    await visible.workflow.cancelChat();
    expect(visible.deps.cancelRequest).toHaveBeenCalledWith("request-1");
    expect(visible.state.status).toBe("Cancelling answer...");

    const silent = createHarness({ activeChatRequestId: "request-1" });
    silent.deps.cancelRequest.mockRejectedValueOnce("already finished");
    await silent.workflow.cancelChat({ silent: true });
    expect(silent.state.status).toBe("");

    const none = createHarness({ activeChatRequestId: null });
    await none.workflow.cancelChat();
    expect(none.deps.cancelRequest).not.toHaveBeenCalled();
  });

  it("clears persisted chat after confirmation", async () => {
    const { state, deps, workflow } = createHarness({
      chatMessages: [turn("user", "Saved question")],
    });
    deps.confirmClearChat.mockResolvedValueOnce(true);
    deps.clearMessages.mockResolvedValueOnce(undefined);

    await workflow.clearMessages();

    expect(deps.clearMessages).toHaveBeenCalledWith(7);
    expect(state.chatMessages).toEqual([]);
    expect(state.status).toBe("Saved chat history cleared.");
    expect(state.clearingChat).toBe(false);
  });

  it("does not clear chat when there is no run or confirmation is declined", async () => {
    const noRun = createHarness({ currentRun: null });
    await noRun.workflow.clearMessages();
    expect(noRun.state.status).toBe("Open a run first.");
    expect(noRun.deps.clearMessages).not.toHaveBeenCalled();

    const declined = createHarness({ chatMessages: [turn("user", "Saved question")] });
    declined.deps.confirmClearChat.mockResolvedValueOnce(false);
    await declined.workflow.clearMessages();
    expect(declined.state.chatMessages).toEqual([turn("user", "Saved question")]);
    expect(declined.deps.clearMessages).not.toHaveBeenCalled();
  });

  it("applies matching chat events and reloads persisted messages on completion", () => {
    const { state, deps, workflow } = createHarness({
      chatMessages: [turn("user", "Question"), turn("assistant", "")],
      chatting: true,
      activeChatRequestId: "request-1",
      activeChatRunId: 7,
    });
    deps.listMessages.mockResolvedValue([]);

    workflow.handleEvent(chatEvent({ delta: "Answer" }));
    expect(state.chatMessages).toEqual([
      turn("user", "Question"),
      turn("assistant", "Answer"),
    ]);

    workflow.handleEvent(chatEvent({ kind: "completed", message: "Answer saved." }));
    expect(state.chatting).toBe(false);
    expect(state.activeChatRequestId).toBeNull();
    expect(state.activeChatRunId).toBeNull();
    expect(state.status).toBe("Answer saved.");
    expect(deps.listMessages).toHaveBeenCalledWith(7);
  });

  it("ignores events for another run or request", () => {
    const { state, deps, workflow } = createHarness({
      chatMessages: [turn("user", "Question"), turn("assistant", "")],
      chatting: true,
      activeChatRequestId: "request-1",
      activeChatRunId: 7,
    });

    workflow.handleEvent(chatEvent({ run_id: 8, delta: "Ignored" }));
    workflow.handleEvent(chatEvent({ request_id: "request-2", delta: "Ignored" }));

    expect(state.chatMessages).toEqual([
      turn("user", "Question"),
      turn("assistant", ""),
    ]);
    expect(deps.listMessages).not.toHaveBeenCalled();
  });

  it("clears chat state to the route default values", () => {
    const { state, workflow } = createHarness({
      chatQuestion: "Question",
      chatMessages: [turn("assistant", "Answer")],
      chatting: true,
      activeChatRequestId: "request-1",
      activeChatRunId: 7,
    });

    workflow.clearState();

    expect(state.chatMessages).toEqual([]);
    expect(state.chatQuestion).toBe("");
    expect(state.chatting).toBe(false);
    expect(state.activeChatRequestId).toBeNull();
    expect(state.activeChatRunId).toBeNull();
  });
});
