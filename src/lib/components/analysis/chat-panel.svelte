<script lang="ts">
  import type { AnalysisRunDetail, AnalysisChatTurn } from "$lib/types/analysis";

  let {
    currentRun,
    loadingChat,
    chatMessages,
    chatQuestion,
    chatting,
    canCancelChat,
    clearingChat,
    selectedTraceRef,
    reportLines,
    onFocusTraceRef,
    onAskQuestion,
    onCancelChat,
    onClearChat,
    onChangeChatQuestion,
  }: {
    currentRun: AnalysisRunDetail | null;
    loadingChat: boolean;
    chatMessages: AnalysisChatTurn[];
    chatQuestion: string;
    chatting: boolean;
    canCancelChat: boolean;
    clearingChat: boolean;
    selectedTraceRef: string | null;
    reportLines: (text: string) => Array<{
      key: string;
      segments: Array<{ type: "text" | "ref"; value: string; key: string }>;
    }>;
    onFocusTraceRef: (ref: string) => void | Promise<void>;
    onAskQuestion: () => void | Promise<void>;
    onCancelChat: () => void | Promise<void>;
    onClearChat: () => void | Promise<void>;
    onChangeChatQuestion: (value: string) => void;
  } = $props();

  let chatThreadElement = $state<HTMLDivElement | null>(null);

  $effect(() => {
    const scrollKey = chatMessages.map((message) => `${message.role}:${message.content.length}`).join("|");
    scrollKey;
    chatting;
    if (typeof window === "undefined" || !chatThreadElement) return;
    requestAnimationFrame(() => {
      chatThreadElement?.scrollTo({
        top: chatThreadElement.scrollHeight,
        behavior: "smooth",
      });
    });
  });
</script>

<section class="card chat">
  <div class="panel-header">
    <div>
      <h3>Report Chat</h3>
      <p class="sub">Ask follow-up questions grounded in the saved report and matching synced messages from the same analysis scope.</p>
    </div>
    {#if currentRun && currentRun.status === "completed"}
      <div class="chat-actions">
        {#if canCancelChat}
          <button class="danger-soft" type="button" onclick={onCancelChat}>Cancel answer</button>
        {/if}
        <button class="secondary" onclick={onClearChat} disabled={chatting || clearingChat}>
          {clearingChat ? "Clearing..." : "Clear chat"}
        </button>
      </div>
    {/if}
  </div>

  {#if !currentRun}
    <p class="empty">Open a saved run to start a grounded chat.</p>
  {:else if currentRun.status !== "completed"}
    <p class="empty">Chat is available only for completed runs.</p>
  {:else}
    <div class="chat-thread" bind:this={chatThreadElement}>
      {#if loadingChat}
        <p class="empty">Loading saved chat history...</p>
      {:else if chatMessages.length === 0}
        <p class="empty">No saved chat turns yet. Ask a follow-up question about this report.</p>
      {:else}
        {#each chatMessages as message, index (`${message.role}-${index}`)}
          <div class={`chat-bubble chat-${message.role}`}>
            <div class="chat-role">{message.role === "user" ? "You" : "Assistant"}</div>
            <div class="chat-content">
              {#if message.role === "assistant" && message.content}
                {#each reportLines(message.content) as line (line.key)}
                  <div class="report-line">
                    {#each line.segments as segment (segment.key)}
                      {#if segment.type === "ref"}
                        <button
                          class="ref-chip"
                          class:active={segment.value === selectedTraceRef}
                          type="button"
                          onclick={() => void onFocusTraceRef(segment.value)}
                        >
                          [{segment.value}]
                        </button>
                      {:else}
                        <span>{segment.value}</span>
                      {/if}
                    {/each}
                  </div>
                {/each}
              {:else}
                {message.content || (chatting && message.role === "assistant" ? "..." : "")}
              {/if}
            </div>
          </div>
        {/each}
      {/if}
    </div>

    <div class="chat-compose">
      <label>Question
        <textarea
          rows="4"
          placeholder="Ask a grounded follow-up question about this report."
          oninput={(event) => onChangeChatQuestion((event.currentTarget as HTMLTextAreaElement).value)}
        >{chatQuestion}</textarea>
      </label>
      <button onclick={onAskQuestion} disabled={chatting || loadingChat || !chatQuestion.trim() || currentRun.status !== "completed"}>
        {chatting ? "Answering..." : "Ask"}
      </button>
    </div>
  {/if}
</section>

<style>
  .card {
    background: var(--panel);
    border: 1px solid var(--border);
    box-shadow: var(--shadow);
    border-radius: 12px;
    padding: 1.5rem;
  }

  .chat {
    margin-top: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .sub,
  .empty {
    margin: 0;
    color: var(--muted);
    font-size: 0.9rem;
  }

  .chat-thread {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    padding: 1rem;
    background: var(--panel-strong);
    border: 1px solid var(--border);
    border-radius: 10px;
    min-height: 10rem;
  }

  .chat-bubble {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    max-width: min(52rem, 100%);
    padding: 0.9rem 1rem;
    border-radius: 12px;
    border: 1px solid var(--border);
    white-space: pre-wrap;
    word-break: break-word;
  }

  .chat-user {
    align-self: flex-end;
    background: color-mix(in srgb, var(--primary) 10%, var(--panel));
    border-color: color-mix(in srgb, var(--primary) 24%, transparent);
  }

  .chat-assistant {
    align-self: flex-start;
    background: var(--panel);
  }

  .chat-role {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--muted);
  }

  .chat-content {
    color: var(--text);
    line-height: 1.6;
  }

  .chat-compose {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }

  .chat-actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    flex-wrap: wrap;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.9rem;
    color: var(--muted);
  }

  textarea {
    width: 100%;
    resize: vertical;
    min-height: 10rem;
    background: var(--panel-strong);
    border: 1px solid var(--border);
    color: var(--text);
    padding: 0.8rem;
    border-radius: 8px;
    font: inherit;
  }

  textarea:focus {
    border-color: var(--primary);
    outline: none;
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 18%, transparent);
  }

  .report-line {
    white-space: pre-wrap;
    word-break: break-word;
  }

  .ref-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.08rem 0.45rem;
    margin: 0 0.08rem;
    border-radius: 999px;
    background: color-mix(in srgb, var(--primary) 14%, var(--panel));
    color: var(--primary);
    border: 1px solid color-mix(in srgb, var(--primary) 24%, transparent);
    font-size: 0.82rem;
    font-weight: 600;
  }

  .ref-chip:hover,
  .ref-chip.active {
    background: color-mix(in srgb, var(--primary) 22%, var(--panel));
  }

  .danger-soft {
    background: color-mix(in srgb, var(--danger) 14%, var(--panel));
    color: var(--danger);
    border: 1px solid color-mix(in srgb, var(--danger) 28%, transparent);
  }

  .danger-soft:hover {
    background: color-mix(in srgb, var(--danger) 22%, var(--panel));
  }
</style>
