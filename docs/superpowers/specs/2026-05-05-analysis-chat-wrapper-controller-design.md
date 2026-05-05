# Analysis Chat Wrapper And Controller Design

## Purpose

Continue the `/analysis` route cleanup by moving the Analysis chat frontend
boundary and chat orchestration into focused modules.

The route currently owns raw Tauri chat command names, the `analysis://chat`
event listener, and several chat workflow functions. This work centralizes the
Tauri command/event boundary in `$lib/api/analysis-chat.ts`, then extracts the
route-level chat workflow into `$lib/analysis-chat-workflow.ts`.

Behavior must stay unchanged. The UI should still optimistically append the
user question and an empty assistant turn before the backend request returns,
stream deltas into that assistant turn, reload saved chat messages after a
completed chat event, use the same clear-chat confirmation copy, and support
silent cancellation when another workflow needs to stop an active answer.

## Scope

Included:

- Create `$lib/api/analysis-chat.ts`.
- Add Vitest coverage for the chat API wrapper.
- Replace Analysis chat raw `invoke(...)` and `listen(...)` calls in
  `src/routes/analysis/+page.svelte`.
- Use the existing `$lib/api/llm.ts` `cancelLlmRequest(...)` wrapper instead
  of calling `cancel_llm_request` directly from the route.
- Create `$lib/analysis-chat-workflow.ts`.
- Add Vitest coverage for the chat workflow controller.
- Keep existing pure reducer helpers in `$lib/analysis-chat-state.ts`.

Excluded:

- Rust backend changes.
- Tauri command or event name changes.
- DTO field renames or camelCase migration.
- Chat UI redesign or component prop redesign.
- Template, source group, trace, report-run, accounts, sources, Takeout, or
  NotebookLM refactors.
- Folder picker, source management, or LLM profile settings changes.

## Frontend API Contract

`src/lib/api/analysis-chat.ts` exposes:

```ts
export const ANALYSIS_CHAT_EVENT = "analysis://chat";

export interface AskAnalysisRunQuestionInput {
  runId: number;
  question: string;
  modelOverride: string | null;
  profileId: string | null;
}

export function listAnalysisChatMessages(
  runId: number,
): Promise<AnalysisChatMessage[]>;

export function askAnalysisRunQuestion(
  input: AskAnalysisRunQuestionInput,
): Promise<string>;

export function clearAnalysisChatMessages(runId: number): Promise<void>;

export function listenToAnalysisChatEvents(
  handler: (event: Event<AnalysisChatEvent>) => void,
): Promise<UnlistenFn>;
```

Wrapped backend command names:

```text
list_analysis_chat_messages
ask_analysis_run_question
clear_analysis_chat_messages
```

Wrapped event name:

```text
analysis://chat
```

Cancellation stays in `$lib/api/llm.ts`:

```ts
cancelLlmRequest(requestId: string): Promise<void>;
```

## Workflow Controller Contract

`src/lib/analysis-chat-workflow.ts` exposes:

```ts
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

export function createAnalysisChatWorkflow(deps: AnalysisChatWorkflowDeps): {
  loadMessages(runId: number, guard?: AnalysisChatRequestGuard): Promise<void>;
  askRunQuestion(): Promise<void>;
  cancelChat(options?: { silent?: boolean }): Promise<void>;
  clearMessages(): Promise<void>;
  clearState(): void;
  handleEvent(payload: AnalysisChatEvent): void;
};
```

The workflow controller must be dependency-injected. It must not import Svelte,
modal helpers, Tauri APIs, or route-local state directly.

## Route Migration

The route keeps the state variables and passes them into
`createAnalysisChatWorkflow(...)` through `getState()` and `patch(...)`.

The route should delegate:

- `cancelChat(...)` to `chatWorkflow.cancelChat(...)`.
- `loadChatMessages(...)` to `chatWorkflow.loadMessages(...)`.
- `askRunQuestion()` to `chatWorkflow.askRunQuestion()`.
- `clearChatMessages()` to `chatWorkflow.clearMessages()`.
- `clearChatState()` to `chatWorkflow.clearState()`.
- the `analysis://chat` listener body to `chatWorkflow.handleEvent(payload)`.

`createAnalysisRunWorkflow(...)` continues to receive chat dependencies, but
those dependencies should call the chat workflow methods.

## Testing

Wrapper tests verify:

- command names;
- payload shapes;
- event constant;
- listener forwarding.

Workflow tests verify:

- guarded saved-message loading ignores stale responses;
- loading failures clear chat messages and report formatted status;
- asking requires a completed run and a non-empty question;
- asking appends the pending exchange and stores the returned request id;
- ask failure drops the pending exchange and clears active chat state;
- cancellation supports silent and visible modes;
- clear-chat requires a current run, respects confirmation, and toggles
  clearing state;
- event handling ignores foreign events, streams deltas, handles terminal
  states, and reloads saved messages on completion.

Required verification:

```powershell
npm.cmd test -- analysis-chat
npm.cmd test -- analysis-chat analysis-chat-state analysis-runs llm
npm.cmd test
npm.cmd run check
git diff --check
```

Route cleanup check:

```powershell
rg -n "analysis://chat|list_analysis_chat_messages|ask_analysis_run_question|clear_analysis_chat_messages|cancel_llm_request" src\routes\analysis\+page.svelte
```

Expected result:

```text
no matches
```

If Vite, esbuild, or Svelte preprocessing fails with `spawn EPERM` in the
default Windows sandbox, rerun frontend verification outside the sandbox after
approval, matching the existing repository notes.
