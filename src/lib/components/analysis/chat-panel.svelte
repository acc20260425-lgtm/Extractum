<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import RefChip from "$lib/components/ui/RefChip.svelte";
  import Textarea from "$lib/components/ui/Textarea.svelte";
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

<Card>
  <div class="chat">
    <div class="panel-header">
      <div>
        <h3>Report Chat</h3>
        <p class="sub">Ask follow-up questions grounded in the saved report and matching synced messages from the same analysis scope.</p>
      </div>
      {#if currentRun && currentRun.status === "completed"}
        <div class="chat-actions">
          {#if canCancelChat}
            <Button variant="danger-soft" type="button" onclick={onCancelChat}>Cancel answer</Button>
          {/if}
          <Button variant="secondary" onclick={onClearChat} disabled={chatting || clearingChat}>
            {clearingChat ? "Clearing..." : "Clear chat"}
          </Button>
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
                          <RefChip
                            refValue={segment.value}
                            active={segment.value === selectedTraceRef}
                            onclick={() => void onFocusTraceRef(segment.value)}
                          />
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
          <Textarea
            value={chatQuestion}
            rows="4"
            placeholder="Ask a grounded follow-up question about this report."
            oninput={(event) => onChangeChatQuestion((event.currentTarget as HTMLTextAreaElement).value)}
            className="chat-question-field"
          />
        </label>
        <Button onclick={onAskQuestion} disabled={chatting || loadingChat || !chatQuestion.trim() || currentRun.status !== "completed"}>
          {chatting ? "Answering..." : "Ask"}
        </Button>
      </div>
    {/if}
  </div>
</Card>

<style>
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

  .chat-compose :global(.ui-textarea.chat-question-field) {
    min-height: 10rem;
  }

  .report-line {
    white-space: pre-wrap;
    word-break: break-word;
  }

</style>
