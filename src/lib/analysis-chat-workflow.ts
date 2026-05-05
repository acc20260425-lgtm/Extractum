import {
  appendPendingChatExchange,
  applyAnalysisChatEvent,
  chatTurnsFromMessages,
  dropPendingChatExchange,
  matchesActiveAnalysisChatEvent,
  type AnalysisChatState,
} from "$lib/analysis-chat-state";
import type { AskAnalysisRunQuestionInput } from "$lib/api/analysis-chat";
import type {
  AnalysisChatEvent,
  AnalysisChatMessage,
  AnalysisChatTurn,
  AnalysisRunDetail,
} from "$lib/types/analysis";

export interface AnalysisChatRequestGuard {
  isCurrent(): boolean;
}

export interface AnalysisChatWorkflowState {
  currentRun: AnalysisRunDetail | null;
  chatQuestion: string;
  chatMessages: AnalysisChatTurn[];
  chatting: boolean;
  activeChatRequestId: string | null;
  activeChatRunId: number | null;
  modelOverride: string;
}

export type AnalysisChatWorkflowPatch = Partial<{
  chatMessages: AnalysisChatTurn[];
  chatQuestion: string;
  chatting: boolean;
  activeChatRequestId: string | null;
  activeChatRunId: number | null;
  loadingChat: boolean;
  clearingChat: boolean;
  status: string;
}>;

export interface AnalysisChatWorkflowDeps {
  getState(): AnalysisChatWorkflowState;
  patch(patch: AnalysisChatWorkflowPatch): void;
  listMessages(runId: number): Promise<AnalysisChatMessage[]>;
  askQuestion(input: AskAnalysisRunQuestionInput): Promise<string>;
  clearMessages(runId: number): Promise<void>;
  cancelRequest(requestId: string): Promise<void>;
  confirmClearChat(): Promise<boolean>;
  formatError(action: string, error: unknown): string;
}

function routeStateToChatState(state: AnalysisChatWorkflowState): AnalysisChatState {
  return {
    messages: state.chatMessages,
    chatting: state.chatting,
    activeRequestId: state.activeChatRequestId,
    activeRunId: state.activeChatRunId,
  };
}

function chatStateToPatch(state: AnalysisChatState): AnalysisChatWorkflowPatch {
  return {
    chatMessages: state.messages,
    chatting: state.chatting,
    activeChatRequestId: state.activeRequestId,
    activeChatRunId: state.activeRunId,
  };
}

function guardIsCurrent(guard?: AnalysisChatRequestGuard) {
  return !guard || guard.isCurrent();
}

export function createAnalysisChatWorkflow(deps: AnalysisChatWorkflowDeps) {
  async function loadMessages(runId: number, guard?: AnalysisChatRequestGuard) {
    if (!guardIsCurrent(guard)) {
      return;
    }

    deps.patch({ loadingChat: true });
    try {
      const messages = await deps.listMessages(runId);
      if (!guardIsCurrent(guard)) {
        return;
      }
      deps.patch({ chatMessages: chatTurnsFromMessages(messages) });
    } catch (error) {
      if (!guardIsCurrent(guard)) {
        return;
      }
      deps.patch({
        chatMessages: [],
        status: deps.formatError("loading analysis chat", error),
      });
    } finally {
      if (guardIsCurrent(guard)) {
        deps.patch({ loadingChat: false });
      }
    }
  }

  async function cancelChat({ silent = false }: { silent?: boolean } = {}) {
    const requestId = deps.getState().activeChatRequestId;
    if (!requestId) {
      return;
    }

    try {
      await deps.cancelRequest(requestId);
      if (!silent) {
        deps.patch({ status: "Cancelling answer..." });
      }
    } catch (error) {
      if (!silent) {
        deps.patch({ status: deps.formatError("cancelling the chat answer", error) });
      }
    }
  }

  async function askRunQuestion() {
    const state = deps.getState();
    const run = state.currentRun;
    if (!run || run.status !== "completed") {
      deps.patch({ status: "Open a completed report first." });
      return;
    }

    const question = state.chatQuestion.trim();
    if (!question) {
      deps.patch({ status: "Question cannot be empty." });
      return;
    }

    deps.patch({
      chatMessages: appendPendingChatExchange(state.chatMessages, question),
      chatQuestion: "",
      chatting: true,
      activeChatRunId: run.id,
    });

    try {
      const requestId = await deps.askQuestion({
        runId: run.id,
        question,
        modelOverride: state.modelOverride.trim() ? state.modelOverride.trim() : null,
        profileId: null,
      });
      deps.patch({ activeChatRequestId: requestId });
    } catch (error) {
      const latest = deps.getState();
      deps.patch({
        chatMessages: dropPendingChatExchange(latest.chatMessages),
        chatting: false,
        activeChatRequestId: null,
        activeChatRunId: null,
        status: deps.formatError("starting the chat answer", error),
      });
    }
  }

  async function clearMessages() {
    const run = deps.getState().currentRun;
    if (!run) {
      deps.patch({ status: "Open a run first." });
      return;
    }

    const confirmed = await deps.confirmClearChat();
    if (!confirmed) {
      return;
    }

    deps.patch({ clearingChat: true });
    try {
      await deps.clearMessages(run.id);
      deps.patch({
        chatMessages: [],
        status: "Saved chat history cleared.",
      });
    } catch (error) {
      deps.patch({ status: deps.formatError("clearing analysis chat", error) });
    } finally {
      deps.patch({ clearingChat: false });
    }
  }

  function clearState() {
    deps.patch({
      chatMessages: [],
      chatQuestion: "",
      chatting: false,
      activeChatRequestId: null,
      activeChatRunId: null,
    });
  }

  function handleEvent(payload: AnalysisChatEvent) {
    const state = deps.getState();
    if (
      !matchesActiveAnalysisChatEvent(
        payload,
        state.activeChatRunId,
        state.activeChatRequestId,
      )
    ) {
      return;
    }

    const reduction = applyAnalysisChatEvent(routeStateToChatState(state), payload);
    deps.patch(chatStateToPatch(reduction.state));
    if (reduction.reloadRunId !== null) {
      void loadMessages(reduction.reloadRunId);
    }
    if (reduction.status !== null) {
      deps.patch({ status: reduction.status });
    }
  }

  return {
    loadMessages,
    askRunQuestion,
    cancelChat,
    clearMessages,
    clearState,
    handleEvent,
  };
}
