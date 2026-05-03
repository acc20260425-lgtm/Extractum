import { describe, expect, it } from "vitest";
import {
  appendAssistantChatDelta,
  appendPendingChatExchange,
  applyAnalysisChatEvent,
  chatTurnsFromMessages,
  dropPendingChatExchange,
  matchesActiveAnalysisChatEvent,
  type AnalysisChatState,
} from "./analysis-chat-state";
import type { AnalysisChatEvent, AnalysisChatMessage, AnalysisChatTurn } from "./types/analysis";

function turn(role: AnalysisChatTurn["role"], content: string): AnalysisChatTurn {
  return { role, content };
}

function chatMessage(overrides: Partial<AnalysisChatMessage>): AnalysisChatMessage {
  return {
    id: 1,
    run_id: 10,
    role: "user",
    content: "question",
    created_at: 100,
    ...overrides,
  };
}

function chatEvent(overrides: Partial<AnalysisChatEvent> = {}): AnalysisChatEvent {
  return {
    request_id: "request-1",
    run_id: 10,
    kind: "delta",
    queue_position: null,
    delta: null,
    message: null,
    error: null,
    ...overrides,
  };
}

function chatState(overrides: Partial<AnalysisChatState> = {}): AnalysisChatState {
  return {
    messages: [
      turn("user", "new question"),
      turn("assistant", ""),
    ],
    chatting: true,
    activeRequestId: "request-1",
    activeRunId: 10,
    ...overrides,
  };
}

describe("analysis-chat-state", () => {
  it("drops only a pending user and assistant exchange from the end", () => {
    const existing = [
      turn("user", "earlier question"),
      turn("assistant", "earlier answer"),
      turn("user", "new question"),
      turn("assistant", "partial answer"),
    ];

    expect(dropPendingChatExchange(existing)).toEqual([
      turn("user", "earlier question"),
      turn("assistant", "earlier answer"),
    ]);
    expect(dropPendingChatExchange([turn("user", "question")])).toEqual([
      turn("user", "question"),
    ]);
    expect(dropPendingChatExchange([turn("assistant", "answer")])).toEqual([
      turn("assistant", "answer"),
    ]);
  });

  it("adds a pending user question and assistant placeholder", () => {
    expect(appendPendingChatExchange([turn("assistant", "ready")], "next question")).toEqual([
      turn("assistant", "ready"),
      turn("user", "next question"),
      turn("assistant", ""),
    ]);
  });

  it("maps persisted chat messages into chat turns", () => {
    expect(chatTurnsFromMessages([
      chatMessage({ id: 1, role: "user", content: "saved question" }),
      chatMessage({ id: 2, role: "assistant", content: "saved answer" }),
    ])).toEqual([
      turn("user", "saved question"),
      turn("assistant", "saved answer"),
    ]);
  });

  it("appends streamed deltas only to a trailing assistant turn", () => {
    expect(appendAssistantChatDelta([
      turn("user", "question"),
      turn("assistant", "partial"),
    ], " answer")).toEqual([
      turn("user", "question"),
      turn("assistant", "partial answer"),
    ]);
    expect(appendAssistantChatDelta([turn("user", "question")], "ignored")).toEqual([
      turn("user", "question"),
    ]);
  });

  it("matches chat events by active run and request", () => {
    expect(matchesActiveAnalysisChatEvent(chatEvent(), 10, "request-1")).toBe(true);
    expect(matchesActiveAnalysisChatEvent(chatEvent(), 10, null)).toBe(true);
    expect(matchesActiveAnalysisChatEvent(chatEvent({ run_id: 11 }), 10, "request-1")).toBe(false);
    expect(matchesActiveAnalysisChatEvent(chatEvent({ request_id: "other" }), 10, "request-1"))
      .toBe(false);
    expect(matchesActiveAnalysisChatEvent(chatEvent(), null, "request-1")).toBe(false);
  });

  it("reduces chat lifecycle events to state, status, and reload hints", () => {
    expect(applyAnalysisChatEvent(chatState(), chatEvent({
      kind: "started",
      message: "Starting answer...",
    }))).toEqual({
      state: chatState(),
      status: "Starting answer...",
      reloadRunId: null,
    });

    expect(applyAnalysisChatEvent(chatState(), chatEvent({
      delta: "hello",
    }))).toEqual({
      state: chatState({
        messages: [
          turn("user", "new question"),
          turn("assistant", "hello"),
        ],
      }),
      status: null,
      reloadRunId: null,
    });

    expect(applyAnalysisChatEvent(chatState(), chatEvent({
      kind: "completed",
      message: "Answer saved.",
    }))).toEqual({
      state: chatState({
        chatting: false,
        activeRequestId: null,
        activeRunId: null,
      }),
      status: "Answer saved.",
      reloadRunId: 10,
    });

    expect(applyAnalysisChatEvent(chatState(), chatEvent({
      kind: "failed",
      error: "model unavailable",
    }))).toEqual({
      state: chatState({
        messages: [],
        chatting: false,
        activeRequestId: null,
        activeRunId: null,
      }),
      status: "Analysis chat failed: model unavailable",
      reloadRunId: null,
    });

    expect(applyAnalysisChatEvent(chatState(), chatEvent({
      kind: "cancelled",
      message: null,
    })).status).toBe("Answer cancelled.");
  });

  it("does not replace status for empty informational chat messages", () => {
    expect(applyAnalysisChatEvent(chatState(), chatEvent({
      kind: "started",
      message: "",
    })).status).toBeNull();

    expect(applyAnalysisChatEvent(chatState(), chatEvent({
      kind: "completed",
      message: "",
    })).status).toBeNull();
  });
});
