<script lang="ts">
  import { Play, Square, Terminal } from "@lucide/svelte";
  import { browser } from "$app/environment";
  import DesktopDialog from "$lib/components/desktop-dialog.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import MetaPill from "$lib/components/ui/MetaPill.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import SurfaceCard from "$lib/components/ui/SurfaceCard.svelte";
  import Textarea from "$lib/components/ui/Textarea.svelte";

  let {
    open,
    prompt,
    providerLabel,
    providerModelLine,
    testing,
    canRun,
    status,
    output,
    usage,
    onOpen,
    onClose,
    onPromptChange,
    onRun,
    onCancel,
  }: {
    open: boolean;
    prompt: string;
    providerLabel: string;
    providerModelLine: string;
    testing: boolean;
    canRun: boolean;
    status: string;
    output: string;
    usage: string;
    onOpen: () => void;
    onClose: () => void;
    onPromptChange: (value: string) => void;
    onRun: () => void | Promise<void>;
    onCancel: () => void | Promise<void>;
  } = $props();
</script>

<Button
  variant="secondary"
  type="button"
  onclick={onOpen}
  ariaLabel="Open provider test console"
  title="Open provider test console"
>
  <Terminal size={15} aria-hidden="true" />
  Open test
</Button>

{#snippet consoleContent()}
  <div class="test-dialog">
    <label>Prompt
      <Textarea
        value={prompt}
        rows={8}
        placeholder={`Ask ${providerLabel} something simple...`}
        oninput={(event) => onPromptChange((event.currentTarget as HTMLTextAreaElement).value)}
      />
    </label>

    <div class="modal-actions">
      <Button
        onclick={onRun}
        disabled={testing || !prompt.trim() || !canRun}
        ariaLabel="Run provider test request"
        title="Run provider test request"
      >
        <Play size={15} aria-hidden="true" />
        {testing ? "Streaming..." : "Run test"}
      </Button>
      {#if testing}
        <Button
          variant="danger-soft"
          type="button"
          onclick={onCancel}
          ariaLabel="Cancel provider test request"
          title="Cancel provider test request"
        >
          <Square size={15} aria-hidden="true" /> Cancel
        </Button>
      {/if}
      {#if providerModelLine}
        <MetaPill>{providerModelLine}</MetaPill>
      {/if}
    </div>

    {#if status}
      <StatusMessage
        tone={status.startsWith("Provider test failed") || status.startsWith("Error") ? "error" : "default"}
      >
        {status}
      </StatusMessage>
    {/if}

    <SurfaceCard title="Streaming output" meta={usage} className="output-surface">
      {#if output}
        <pre>{output}</pre>
      {:else}
        <EmptyState description="No output yet." />
      {/if}
    </SurfaceCard>
  </div>
{/snippet}

{#if browser}
  <DesktopDialog
    {open}
    title="Provider Test Console"
    description="Run a live request with the profile currently open in settings before using it in reports."
    labelledBy="provider-test-title"
    width="52rem"
    {onClose}
  >
    {@render consoleContent()}
  </DesktopDialog>
{:else if open}
  <div role="dialog" aria-labelledby="provider-test-title" aria-describedby="provider-test-description">
    <h2 id="provider-test-title">Provider Test Console</h2>
    <p id="provider-test-description">
      Run a live request with the profile currently open in settings before using it in reports.
    </p>
    {@render consoleContent()}
  </div>
{/if}

<style>
  .test-dialog {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.9rem;
    color: var(--muted);
  }

  .modal-actions {
    display: flex;
    gap: 0.6rem;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
  }

  :global(.ui-surface-card.output-surface) {
    min-height: 14rem;
  }

  pre {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
    font: inherit;
    line-height: 1.6;
  }

  @media (max-width: 720px) {
    .modal-actions {
      flex-direction: column;
      align-items: stretch;
    }
  }
</style>
