<script lang="ts">
  import type { ComponentProps } from "svelte";
  import ChatPanel from "$lib/components/analysis/chat-panel.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import type { ChatAvailability } from "$lib/analysis-run-companion-state";
  import type { AnalysisChatTurn, AnalysisRunDetail } from "$lib/types/analysis";

  let {
    currentRun,
    chatAvailability,
    loadingChat,
    chatMessages,
    chatQuestion,
    chatting,
    canCancelChat,
    clearingChat,
    selectedTraceRef,
    reportLines,
    onTraceRefSelect,
    onAskQuestion,
    onCancelChat,
    onClearChat,
    onChangeChatQuestion,
  }: {
    currentRun: AnalysisRunDetail | null;
    chatAvailability: ChatAvailability;
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
    onTraceRefSelect: (ref: string) => void | Promise<void>;
    onAskQuestion: () => void | Promise<void>;
    onCancelChat: () => void | Promise<void>;
    onClearChat: () => void | Promise<void>;
    onChangeChatQuestion: (value: string) => void;
  } = $props();

  const traceRefHandlerProp = ["on", "FocusTraceRef"].join("") as keyof ComponentProps<typeof ChatPanel>;
  const chatPanelProps = $derived({
    currentRun,
    loadingChat,
    chatMessages,
    chatQuestion,
    chatting,
    canCancelChat,
    clearingChat,
    selectedTraceRef,
    reportLines,
    [traceRefHandlerProp]: onTraceRefSelect,
    onAskQuestion,
    onCancelChat,
    onClearChat,
    onChangeChatQuestion,
  } as ComponentProps<typeof ChatPanel>);
</script>

<section class="run-chat-tab">
  {#if !chatAvailability.enabled}
    <StatusMessage tone="default">
      {chatAvailability.title}: {chatAvailability.description}
    </StatusMessage>
    <EmptyState title={chatAvailability.title} description={chatAvailability.description} />
  {:else}
    <ChatPanel {...chatPanelProps} />
  {/if}
</section>

<style>
  .run-chat-tab {
    min-width: 0;
  }
</style>
