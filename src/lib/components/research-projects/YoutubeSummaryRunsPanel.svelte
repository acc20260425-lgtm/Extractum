<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { RefreshCw, XCircle } from "@lucide/svelte";
  import { ExtractumBadge, ExtractumButton } from "$lib/components/extractum-ui";
  import {
    cancelPromptPackRun,
    listenToPromptPackRunEvents,
    listActivePromptPackRuns,
    listPromptPackRuns,
  } from "$lib/api/prompt-packs";
  import { statusLabel, updateRunListFromEvent } from "$lib/ui/youtube-summary-workflow";
  import type { PromptPackRunListItem } from "$lib/types/prompt-packs";

  let { projectId = null }: { projectId?: number | null } = $props();

  let runs = $state<PromptPackRunListItem[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let unlisten: (() => void) | null = null;

  let activeRuns = $derived(runs.filter((run) => run.runStatus === "queued" || run.runStatus === "running"));
  let recentRuns = $derived(runs.filter((run) => run.runStatus !== "queued" && run.runStatus !== "running"));

  onMount(() => {
    void refreshRuns();
    void listenToPromptPackRunEvents((event) => {
      runs = updateRunListFromEvent(runs, event.payload);
    }).then((stop) => {
      unlisten = stop;
    });
  });

  onDestroy(() => {
    unlisten?.();
  });

  async function refreshRuns() {
    loading = true;
    error = null;
    try {
      const [recent, active] = await Promise.all([
        listPromptPackRuns({ projectId, limit: 20 }),
        listActivePromptPackRuns(),
      ]);
      runs = [...recent, ...active]
        .filter((run, index, allRuns) => allRuns.findIndex((candidate) => candidate.runId === run.runId) === index)
        .sort((left, right) => right.runId - left.runId);
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      loading = false;
    }
  }

  async function cancelRun(runId: number) {
    await cancelPromptPackRun(runId);
    await refreshRuns();
  }
</script>

<section class="prompt-pack-runs" aria-label="Prompt Pack runs">
  <div class="runs-toolbar">
    <div>
      <span>Prompt Pack runs</span>
      <strong>{runs.length}</strong>
    </div>
    <ExtractumButton variant="outline" disabled={loading} onclick={() => void refreshRuns()}>
      <RefreshCw size={14} aria-hidden="true" />
      Refresh
    </ExtractumButton>
  </div>

  {#if error}
    <p class="run-error">{error}</p>
  {/if}

  <div class="run-groups">
    <section>
      <h3>Active</h3>
      {#if activeRuns.length === 0}
        <p class="empty-runs">No active prompt pack runs.</p>
      {:else}
        <ul>
          {#each activeRuns as run (run.runId)}
            <li>
              <div>
                <strong>Run #{run.runId}</strong>
                <ExtractumBadge>{statusLabel(run.runStatus)}</ExtractumBadge>
                <p>{run.latestMessage ?? "Waiting for progress."}</p>
              </div>
              <ExtractumButton variant="outline" onclick={() => void cancelRun(run.runId)}>
                <XCircle size={14} aria-hidden="true" />
                Cancel
              </ExtractumButton>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <section>
      <h3>Recent</h3>
      {#if recentRuns.length === 0}
        <p class="empty-runs">No prompt pack runs yet.</p>
      {:else}
        <ul>
          {#each recentRuns as run (run.runId)}
            <li>
              <div>
                <strong>Run #{run.runId}</strong>
                <ExtractumBadge>{statusLabel(run.runStatus)}</ExtractumBadge>
                <p>{run.latestMessage ?? run.resultStatus ?? "Completed"}</p>
              </div>
            </li>
          {/each}
        </ul>
      {/if}
    </section>
  </div>
</section>

<style>
  .prompt-pack-runs,
  .run-groups,
  section,
  ul {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 10px;
  }

  .runs-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .runs-toolbar div {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .runs-toolbar span,
  h3 {
    color: var(--extractum-muted);
    font-size: 11px;
    text-transform: uppercase;
  }

  h3,
  p,
  ul {
    margin: 0;
  }

  ul {
    padding: 0;
    list-style: none;
  }

  li,
  .empty-runs,
  .run-error {
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface-raised);
    padding: 10px;
  }

  li {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 10px;
  }

  li div {
    display: flex;
    min-width: 0;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }

  li p {
    flex-basis: 100%;
    color: var(--extractum-muted);
    font-size: 12px;
  }

  .empty-runs {
    color: var(--extractum-muted);
  }

  .run-error {
    color: var(--extractum-danger);
  }
</style>
