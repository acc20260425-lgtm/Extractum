import type {
  AnalysisChatEvent,
  AnalysisChatMessage,
  AnalysisChatTurn,
} from "$lib/types/analysis";

export type AnalysisChatState = {
  messages: AnalysisChatTurn[];
  chatting: boolean;
  activeRequestId: string | null;
  activeRunId: number | null;
};

export type AnalysisChatEventReduction = {
  state: AnalysisChatState;
  status: string | null;
  reloadRunId: number | null;
};

export function appendPendingChatExchange(
  messages: AnalysisChatTurn[],
  question: string,
): AnalysisChatTurn[] {
  return [
    ...messages,
    { role: "user", content: question },
    { role: "assistant", content: "" },
  ];
}

export function chatTurnsFromMessages(messages: AnalysisChatMessage[]): AnalysisChatTurn[] {
  return messages.map((message) => ({
    role: message.role,
    content: message.content,
  }));
}

export function dropPendingChatExchange(messages: AnalysisChatTurn[]): AnalysisChatTurn[] {
  if (
    messages.length >= 2 &&
    messages[messages.length - 1]?.role === "assistant" &&
    messages[messages.length - 2]?.role === "user"
  ) {
    return messages.slice(0, -2);
  }

  return messages;
}

export function appendAssistantChatDelta(
  messages: AnalysisChatTurn[],
  delta: string | null,
): AnalysisChatTurn[] {
  const lastIndex = messages.length - 1;
  if (lastIndex < 0 || messages[lastIndex]?.role !== "assistant") {
    return messages;
  }

  const updated = [...messages];
  updated[lastIndex] = {
    role: "assistant",
    content: `${updated[lastIndex].content}${delta ?? ""}`,
  };
  return updated;
}

export function matchesActiveAnalysisChatEvent(
  payload: AnalysisChatEvent,
  activeRunId: number | null,
  activeRequestId: string | null,
): boolean {
  return (
    payload.run_id === activeRunId &&
    (activeRequestId === null || payload.request_id === activeRequestId)
  );
}

export function applyAnalysisChatEvent(
  state: AnalysisChatState,
  payload: AnalysisChatEvent,
): AnalysisChatEventReduction {
  if (payload.kind === "queued" || payload.kind === "started") {
    return {
      state,
      status: payload.message || null,
      reloadRunId: null,
    };
  }

  if (payload.kind === "delta") {
    return {
      state: {
        ...state,
        messages: appendAssistantChatDelta(state.messages, payload.delta),
      },
      status: null,
      reloadRunId: null,
    };
  }

  if (payload.kind === "completed") {
    return {
      state: {
        ...state,
        chatting: false,
        activeRequestId: null,
        activeRunId: null,
      },
      status: payload.message || null,
      reloadRunId: state.activeRunId,
    };
  }

  return {
    state: {
      messages: dropPendingChatExchange(state.messages),
      chatting: false,
      activeRequestId: null,
      activeRunId: null,
    },
    status:
      payload.kind === "cancelled"
        ? payload.message ?? "Answer cancelled."
        : payload.error
          ? `Analysis chat failed: ${payload.error}`
          : "Analysis chat failed.",
    reloadRunId: null,
  };
}
