# Analysis Chat Wrapper And Controller Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Centralize Analysis chat frontend command/event access and extract route-level chat orchestration from `/analysis`.

**Architecture:** Add a narrow `$lib/api/analysis-chat.ts` wrapper first, mirroring the existing `analysis-runs`, `takeout-import`, and `notebooklm-export` wrappers. Then add `$lib/analysis-chat-workflow.ts` as a dependency-injected controller that reuses existing pure chat reducers and lets the route remain the Svelte state owner.

**Tech Stack:** Svelte 5, SvelteKit, TypeScript, Tauri v2 API, Vitest.

---

## Context

Current route:

```text
src/routes/analysis/+page.svelte
```

Current pure chat reducer:

```text
src/lib/analysis-chat-state.ts
src/lib/analysis-chat-state.test.ts
```

Current related wrappers:

```text
src/lib/api/analysis-runs.ts
src/lib/api/llm.ts
src/lib/api/notebooklm-export.ts
src/lib/api/takeout-import.ts
```

Existing chat types:

```text
src/lib/types/analysis.ts
```

Relevant backend command signatures:

```text
list_analysis_chat_messages(runId) -> AnalysisChatMessage[]
ask_analysis_run_question(runId, question, modelOverride, profileId) -> string request id
clear_analysis_chat_messages(runId) -> void
cancel_llm_request(requestId) -> void
```

Relevant event:

```text
analysis://chat
```

This work must not change Rust code, command names, event names, DTO field
names, UI layout, component props, or the existing chat reducer semantics.

## File Structure

- Create `src/lib/api/analysis-chat.ts`: typed Tauri wrapper for Analysis chat commands and event listener.
- Create `src/lib/api/analysis-chat.test.ts`: wrapper contract tests.
- Create `src/lib/analysis-chat-workflow.ts`: route-independent chat workflow controller.
- Create `src/lib/analysis-chat-workflow.test.ts`: controller behavior tests.
- Modify `src/routes/analysis/+page.svelte`: replace raw chat Tauri calls with wrappers and delegate chat orchestration to the controller.
- Read existing `src/lib/api/llm.ts`: use `cancelLlmRequest(...)` instead of raw `cancel_llm_request`.

## Task 1: Documentation Baseline

**Files:**

- Create: `docs/superpowers/specs/2026-05-05-analysis-chat-wrapper-controller-design.md`
- Create: `docs/superpowers/plans/2026-05-05-analysis-chat-wrapper-controller.md`

- [ ] **Step 1: Save the design document**

Create `docs/superpowers/specs/2026-05-05-analysis-chat-wrapper-controller-design.md` with the approved scope, API contract, workflow contract, route migration notes, and verification commands.

- [ ] **Step 2: Save this implementation plan**

Create `docs/superpowers/plans/2026-05-05-analysis-chat-wrapper-controller.md` with all tasks in this file.

- [ ] **Step 3: Verify documentation formatting**

Run:

```powershell
git diff --check
```

Expected:

```text
no output
```

- [ ] **Step 4: Commit**

Run:

```powershell
git add docs/superpowers/specs/2026-05-05-analysis-chat-wrapper-controller-design.md docs/superpowers/plans/2026-05-05-analysis-chat-wrapper-controller.md
git commit -m "docs(analysis): add chat wrapper controller plan"
```

If git writes fail with `.git/index.lock` permission errors in the Windows
sandbox, rerun the git command outside the sandbox after approval.

## Task 2: Analysis Chat API Wrapper

**Files:**

- Create: `src/lib/api/analysis-chat.test.ts`
- Create: `src/lib/api/analysis-chat.ts`
- Modify: `src/routes/analysis/+page.svelte`
- Read: `src/lib/api/llm.ts`

- [ ] **Step 1: Write the failing wrapper tests**

Create `src/lib/api/analysis-chat.test.ts` with:

```ts
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
```

- [ ] **Step 2: Run the focused test and verify it fails**

Run:

```powershell
npm.cmd test -- analysis-chat
```

Expected:

```text
FAIL src/lib/api/analysis-chat.test.ts
Cannot find module './analysis-chat'
```

- [ ] **Step 3: Create the wrapper module**

Create `src/lib/api/analysis-chat.ts` with:

```ts
import { invoke } from "@tauri-apps/api/core";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AnalysisChatEvent,
  AnalysisChatMessage,
  EventEnvelope,
} from "$lib/types/analysis";

export const ANALYSIS_CHAT_EVENT = "analysis://chat";

export interface AskAnalysisRunQuestionInput {
  runId: number;
  question: string;
  modelOverride: string | null;
  profileId: string | null;
}

export function listAnalysisChatMessages(runId: number) {
  return invoke<AnalysisChatMessage[]>("list_analysis_chat_messages", { runId });
}

export function askAnalysisRunQuestion(input: AskAnalysisRunQuestionInput) {
  return invoke<string>("ask_analysis_run_question", { ...input });
}

export function clearAnalysisChatMessages(runId: number) {
  return invoke<void>("clear_analysis_chat_messages", { runId });
}

export function listenToAnalysisChatEvents(
  handler: (event: Event<AnalysisChatEvent>) => void,
): Promise<UnlistenFn> {
  return listen<AnalysisChatEvent>(
    ANALYSIS_CHAT_EVENT,
    (event: EventEnvelope<AnalysisChatEvent> & Event<AnalysisChatEvent>) => handler(event),
  );
}
```

- [ ] **Step 4: Run the focused wrapper test**

Run:

```powershell
npm.cmd test -- analysis-chat
```

Expected:

```text
PASS src/lib/api/analysis-chat.test.ts
```

- [ ] **Step 5: Import the wrapper and LLM cancellation in the route**

In `src/routes/analysis/+page.svelte`, add near the other `$lib/api/*` imports:

```ts
import {
  askAnalysisRunQuestion,
  clearAnalysisChatMessages,
  listAnalysisChatMessages,
  listenToAnalysisChatEvents,
} from "$lib/api/analysis-chat";
import { cancelLlmRequest } from "$lib/api/llm";
```

Remove this import because the route should no longer own a raw event listener:

```ts
import { listen } from "@tauri-apps/api/event";
```

Remove unused type imports after the migration:

```ts
AnalysisChatEvent,
AnalysisChatMessage,
EventEnvelope,
```

- [ ] **Step 6: Replace direct cancellation**

Change:

```ts
await invoke("cancel_llm_request", { requestId });
```

to:

```ts
await cancelLlmRequest(requestId);
```

- [ ] **Step 7: Replace chat message loading**

Change:

```ts
const messages = await invoke<AnalysisChatMessage[]>("list_analysis_chat_messages", { runId });
```

to:

```ts
const messages = await listAnalysisChatMessages(runId);
```

- [ ] **Step 8: Replace chat question start**

Change:

```ts
const requestId = await invoke<string>("ask_analysis_run_question", {
  runId: currentRun.id,
  question,
  modelOverride: modelOverride.trim() ? modelOverride.trim() : null,
  profileId: null,
});
```

to:

```ts
const requestId = await askAnalysisRunQuestion({
  runId: currentRun.id,
  question,
  modelOverride: modelOverride.trim() ? modelOverride.trim() : null,
  profileId: null,
});
```

- [ ] **Step 9: Replace clear-chat command**

Change:

```ts
await invoke("clear_analysis_chat_messages", { runId: currentRun.id });
```

to:

```ts
await clearAnalysisChatMessages(currentRun.id);
```

- [ ] **Step 10: Replace the chat event listener**

Change:

```ts
void listen<AnalysisChatEvent>("analysis://chat", ({ payload }: EventEnvelope<AnalysisChatEvent>) => {
  if (
    disposed ||
    !matchesActiveAnalysisChatEvent(payload, activeChatRunId, activeChatRequestId)
  ) {
    return;
  }

  const reduction = applyAnalysisChatEvent(currentChatState(), payload);
  assignChatState(reduction.state);
  if (reduction.reloadRunId !== null) {
    void loadChatMessages(reduction.reloadRunId);
  }
  if (reduction.status !== null) {
    status = reduction.status;
  }
}).then((unlisten) => {
  if (disposed) {
    unlisten();
    return;
  }
  detachChatListener = unlisten;
});
```

to:

```ts
void listenToAnalysisChatEvents(({ payload }) => {
  if (
    disposed ||
    !matchesActiveAnalysisChatEvent(payload, activeChatRunId, activeChatRequestId)
  ) {
    return;
  }

  const reduction = applyAnalysisChatEvent(currentChatState(), payload);
  assignChatState(reduction.state);
  if (reduction.reloadRunId !== null) {
    void loadChatMessages(reduction.reloadRunId);
  }
  if (reduction.status !== null) {
    status = reduction.status;
  }
}).then((unlisten) => {
  if (disposed) {
    unlisten();
    return;
  }
  detachChatListener = unlisten;
});
```

- [ ] **Step 11: Verify the route no longer owns raw chat Tauri strings**

Run:

```powershell
rg -n "analysis://chat|list_analysis_chat_messages|ask_analysis_run_question|clear_analysis_chat_messages|cancel_llm_request" src\routes\analysis\+page.svelte
```

Expected:

```text
no matches
```

- [ ] **Step 12: Run nearby wrapper tests**

Run:

```powershell
npm.cmd test -- analysis-chat analysis-runs llm
```

Expected:

```text
3 test files passed
```

- [ ] **Step 13: Commit**

Run:

```powershell
git add src/lib/api/analysis-chat.ts src/lib/api/analysis-chat.test.ts src/routes/analysis/+page.svelte
git commit -m "refactor(analysis): add chat api wrapper"
```

## Task 3: Analysis Chat Workflow Controller

**Files:**

- Create: `src/lib/analysis-chat-workflow.test.ts`
- Create: `src/lib/analysis-chat-workflow.ts`
- Modify: `src/routes/analysis/+page.svelte`
- Read: `src/lib/analysis-chat-state.ts`
- Read: `src/lib/analysis-run-workflow.ts`

- [ ] **Step 1: Write the failing workflow tests**

Create `src/lib/analysis-chat-workflow.test.ts` with:

```ts
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
      profileId: null,
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
```

- [ ] **Step 2: Run the focused workflow test and verify it fails**

Run:

```powershell
npm.cmd test -- analysis-chat-workflow
```

Expected:

```text
FAIL src/lib/analysis-chat-workflow.test.ts
Cannot find module './analysis-chat-workflow'
```

- [ ] **Step 3: Create the workflow controller**

Create `src/lib/analysis-chat-workflow.ts` with:

```ts
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
    if (!matchesActiveAnalysisChatEvent(
      payload,
      state.activeChatRunId,
      state.activeChatRequestId,
    )) {
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
```

- [ ] **Step 4: Run the focused workflow test**

Run:

```powershell
npm.cmd test -- analysis-chat-workflow
```

Expected:

```text
PASS src/lib/analysis-chat-workflow.test.ts
```

- [ ] **Step 5: Update route imports**

In `src/routes/analysis/+page.svelte`, add:

```ts
import {
  createAnalysisChatWorkflow,
  type AnalysisChatWorkflowPatch,
} from "$lib/analysis-chat-workflow";
```

Remove these imports from `$lib/analysis-chat-state` because the route should no longer call them directly:

```ts
appendPendingChatExchange,
applyAnalysisChatEvent,
chatTurnsFromMessages,
dropPendingChatExchange,
matchesActiveAnalysisChatEvent,
type AnalysisChatState,
```

Keep no imports from `$lib/analysis-chat-state` in the route after this task.

- [ ] **Step 6: Add the chat workflow patch helper**

Replace the route-local `currentChatState()` and `assignChatState(...)`
functions with:

```ts
function applyChatWorkflowPatch(patch: AnalysisChatWorkflowPatch) {
  if ("chatMessages" in patch) chatMessages = patch.chatMessages ?? [];
  if ("chatQuestion" in patch) chatQuestion = patch.chatQuestion ?? "";
  if ("chatting" in patch) chatting = patch.chatting ?? false;
  if ("activeChatRequestId" in patch) activeChatRequestId = patch.activeChatRequestId ?? null;
  if ("activeChatRunId" in patch) activeChatRunId = patch.activeChatRunId ?? null;
  if ("loadingChat" in patch) loadingChat = patch.loadingChat ?? false;
  if ("clearingChat" in patch) clearingChat = patch.clearingChat ?? false;
  if ("status" in patch && patch.status !== undefined) status = patch.status;
}
```

- [ ] **Step 7: Instantiate the chat workflow**

Add before `createAnalysisRunWorkflow(...)`:

```ts
const chatWorkflow = createAnalysisChatWorkflow({
  getState: () => ({
    currentRun,
    chatQuestion,
    chatMessages,
    chatting,
    activeChatRequestId,
    activeChatRunId,
    modelOverride,
  }),
  patch: applyChatWorkflowPatch,
  listMessages: listAnalysisChatMessages,
  askQuestion: askAnalysisRunQuestion,
  clearMessages: clearAnalysisChatMessages,
  cancelRequest: cancelLlmRequest,
  confirmClearChat: () => openConfirmModal({
    title: "Clear chat history?",
    message: "This will remove all saved follow-up messages for the currently opened run.",
    confirmLabel: "Clear history",
    cancelLabel: "Cancel",
    tone: "danger",
  }),
  formatError: formatAppError,
});
```

- [ ] **Step 8: Delegate route chat helpers to the workflow**

Change:

```ts
function clearChatState() {
  chatMessages = [];
  chatQuestion = "";
  chatting = false;
  activeChatRequestId = null;
  activeChatRunId = null;
}
```

to:

```ts
function clearChatState() {
  chatWorkflow.clearState();
}
```

Change `cancelChat(...)` to:

```ts
async function cancelChat({ silent = false }: { silent?: boolean } = {}) {
  await chatWorkflow.cancelChat({ silent });
}
```

Change `loadChatMessages(...)` to:

```ts
async function loadChatMessages(runId: number, guard?: AnalysisRunRequestGuard) {
  await chatWorkflow.loadMessages(runId, guard);
}
```

Change `askRunQuestion()` to:

```ts
async function askRunQuestion() {
  await chatWorkflow.askRunQuestion();
}
```

Change `clearChatMessages()` to:

```ts
async function clearChatMessages() {
  await chatWorkflow.clearMessages();
}
```

- [ ] **Step 9: Delegate event handling to the workflow**

Change the chat listener body to:

```ts
void listenToAnalysisChatEvents(({ payload }) => {
  if (disposed) {
    return;
  }

  chatWorkflow.handleEvent(payload);
}).then((unlisten) => {
  if (disposed) {
    unlisten();
    return;
  }
  detachChatListener = unlisten;
});
```

- [ ] **Step 10: Run focused workflow and reducer tests**

Run:

```powershell
npm.cmd test -- analysis-chat-workflow analysis-chat-state
```

Expected:

```text
2 test files passed
```

- [ ] **Step 11: Run nearby frontend tests**

Run:

```powershell
npm.cmd test -- analysis-chat analysis-chat-workflow analysis-chat-state analysis-runs llm
```

Expected:

```text
5 test files passed
```

- [ ] **Step 12: Run Svelte and TypeScript checks**

Run:

```powershell
npm.cmd run check
```

Expected:

```text
svelte-check found 0 errors and 0 warnings
```

If this fails with `spawn EPERM` in the default Windows sandbox, rerun outside
the sandbox after approval.

- [ ] **Step 13: Commit**

Run:

```powershell
git add src/lib/analysis-chat-workflow.ts src/lib/analysis-chat-workflow.test.ts src/routes/analysis/+page.svelte
git commit -m "refactor(analysis): extract chat workflow controller"
```

## Task 4: Final Verification And Handoff

**Files:**

- Verify: `src/lib/api/analysis-chat.ts`
- Verify: `src/lib/analysis-chat-workflow.ts`
- Verify: `src/routes/analysis/+page.svelte`
- Modify if needed: `docs/session-context-2026-05-03.md`
- Modify if needed: `docs/superpowers/plans/2026-05-05-analysis-chat-wrapper-controller.md`

- [ ] **Step 1: Verify no raw chat command or event strings remain in the route**

Run:

```powershell
rg -n "analysis://chat|list_analysis_chat_messages|ask_analysis_run_question|clear_analysis_chat_messages|cancel_llm_request" src\routes\analysis\+page.svelte
```

Expected:

```text
no matches
```

- [ ] **Step 2: Run focused tests**

Run:

```powershell
npm.cmd test -- analysis-chat analysis-chat-workflow analysis-chat-state
```

Expected:

```text
3 test files passed
```

- [ ] **Step 3: Run nearby API and workflow tests**

Run:

```powershell
npm.cmd test -- analysis-chat analysis-chat-workflow analysis-chat-state analysis-runs llm notebooklm-export takeout-import sources
```

Expected:

```text
8 test files passed
```

- [ ] **Step 4: Run full frontend tests**

Run:

```powershell
npm.cmd test
```

Expected:

```text
all test files passed
```

- [ ] **Step 5: Run Svelte and TypeScript checks**

Run:

```powershell
npm.cmd run check
```

Expected:

```text
svelte-check found 0 errors and 0 warnings
```

If this fails with `spawn EPERM` in the default Windows sandbox, rerun outside
the sandbox after approval.

- [ ] **Step 6: Run whitespace check**

Run:

```powershell
git diff --check
```

Expected:

```text
no output
```

- [ ] **Step 7: Refresh handoff docs if implementation changed repository state**

Update `docs/session-context-2026-05-03.md` with:

```text
Completed Analysis Chat Wrapper And Controller Work

Plan:
docs/superpowers/plans/2026-05-05-analysis-chat-wrapper-controller.md

Design/spec:
docs/superpowers/specs/2026-05-05-analysis-chat-wrapper-controller-design.md

Goal completed:
- Centralized Analysis chat frontend command/event access in `$lib/api/analysis-chat.ts`.
- Removed Analysis chat raw Tauri calls from `src/routes/analysis/+page.svelte`.
- Extracted route-level chat orchestration into `$lib/analysis-chat-workflow.ts`.

Scope intentionally preserved:
- No Rust backend command or event changes.
- No Analysis chat DTO camelCase migration.
- No chat UI redesign.
- No template, source group, trace, report-run, accounts, sources, Takeout, or NotebookLM refactors.
```

- [ ] **Step 8: Commit verification or docs**

If files changed, run:

```powershell
git add docs/session-context-2026-05-03.md docs/superpowers/plans/2026-05-05-analysis-chat-wrapper-controller.md
git commit -m "docs(analysis): record chat controller completion"
```

If no files changed but the one-commit-per-task rule still applies, create an
empty verification commit:

```powershell
git commit --allow-empty -m "test(analysis): verify chat controller integration"
```

## Self-Review Checklist

- The chat API wrapper is the only new Tauri chat API surface.
- The route no longer owns Analysis chat command names, the chat event name, or
  `cancel_llm_request`.
- The controller is dependency-injected and imports no Svelte, Tauri APIs, or
  modal helpers.
- Existing `analysis-chat-state.ts` reducer helpers remain the source of chat
  event reduction behavior.
- Existing backend DTO field names stay unchanged.
- No Rust files are modified.
- No template, source group, trace, report-run, accounts, sources, Takeout, or
  NotebookLM workflows are refactored in this workstream.

## Commit Messages

```text
docs(analysis): add chat wrapper controller plan
refactor(analysis): add chat api wrapper
refactor(analysis): extract chat workflow controller
test(analysis): verify chat controller integration
```
