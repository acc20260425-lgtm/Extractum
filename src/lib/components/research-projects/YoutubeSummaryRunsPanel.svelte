<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { RefreshCw, Trash2, XCircle } from "@lucide/svelte";
  import { ExtractumBadge, ExtractumButton } from "$lib/components/extractum-ui";
  import {
    cancelPromptPackRun,
    deletePromptPackRun,
    listenToPromptPackRunEvents,
    listActivePromptPackRuns,
    listPromptPackRuns,
  } from "$lib/api/prompt-packs";
  import { openConfirmModal } from "$lib/modals";
  import {
    filterDeletedRunIds,
    retainSelectedRunId,
    shouldApplyRunEventToRunsPanel,
    statusLabel,
    updateRunListFromEvent,
  } from "$lib/ui/youtube-summary-workflow";
  import type { PromptPackRunListItem } from "$lib/types/prompt-packs";
  import YoutubeSummaryResultView from "./YoutubeSummaryResultView.svelte";

  let { projectId = null }: { projectId?: number | null } = $props();

  let runs = $state<PromptPackRunListItem[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let selectedRunId = $state<number | null>(null);
  let deletingRunIds = $state<Record<number, boolean>>({});
  let deletedRunIds = $state<Record<number, boolean>>({});
  let unlisten: (() => void) | null = null;

  let activeRuns = $derived(runs.filter((run) => run.runStatus === "queued" || run.runStatus === "running"));
  let recentRuns = $derived(runs.filter((run) => run.runStatus !== "queued" && run.runStatus !== "running"));
  let selectedRun = $derived(runs.find((run) => run.runId === selectedRunId) ?? null);

  onMount(() => {
    void refreshRuns();
    void listenToPromptPackRunEvents((event) => {
      if (deletedRunIds[event.payload.runId]) return;
      if (!shouldApplyRunEventToRunsPanel(runs, event.payload, projectId)) return;
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
      const scopedActive = projectId === null ? active : active.filter((run) => run.projectId === projectId);
      const nextRuns = [...recent, ...scopedActive]
        .filter((run, index, allRuns) => allRuns.findIndex((candidate) => candidate.runId === run.runId) === index)
        .sort((left, right) => right.runId - left.runId);
      const visibleRuns = filterDeletedRunIds(nextRuns, deletedRunIds);
      runs = visibleRuns;
      selectedRunId = retainSelectedRunId(selectedRunId, visibleRuns);
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

  function isPromptPackRunActive(run: PromptPackRunListItem) {
    return run.runStatus === "queued" || run.runStatus === "running";
  }

  function runtimeLabel(runtimeProvider: PromptPackRunListItem["runtimeProvider"]) {
    return runtimeProvider === "gemini_browser" ? "Gemini Browser" : "API profile";
  }

  async function deleteRun(run: PromptPackRunListItem) {
    if (isPromptPackRunActive(run) || deletingRunIds[run.runId]) return;
    const confirmed = await openConfirmModal({
      title: "Delete Prompt Pack run?",
      message: `Run ${run.runId} will be removed with its result, stages, and artifacts.`,
      confirmLabel: "Delete",
      cancelLabel: "Cancel",
      tone: "danger",
    });
    if (!confirmed) return;
    deletingRunIds = { ...deletingRunIds, [run.runId]: true };
    error = null;
    try {
      await deletePromptPackRun(run.runId);
      deletedRunIds = { ...deletedRunIds, [run.runId]: true };
      if (selectedRunId === run.runId) selectedRunId = null;
      runs = runs.filter((candidate) => candidate.runId !== run.runId);
      void refreshRuns();
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      const next = { ...deletingRunIds };
      delete next[run.runId];
      deletingRunIds = next;
    }
  }
</script>

<section class="prompt-pack-runs" aria-label="Prompt Pack runs">
  <div class="runs-toolbar extractum-toolbar-row">
    <div>
      <span>Prompt Pack runs</span>
      <strong>{runs.length}</strong>
    </div>
    <ExtractumButton
      variant="outline"
      disabled={loading}
      onclick={() => void refreshRuns()}
      aria-label="Refresh prompt pack runs"
      title="Refresh prompt pack runs"
    >
      <RefreshCw size={14} aria-hidden="true" />
      Refresh
    </ExtractumButton>
  </div>

  {#if error}
    <p class="run-error extractum-panel-shell compact">{error}</p>
  {/if}

  <div class="run-groups">
    <section>
      <h3>Active</h3>
      {#if activeRuns.length === 0}
        <p class="empty-runs extractum-panel-shell compact">No active prompt pack runs.</p>
      {:else}
        <ul>
          {#each activeRuns as run (run.runId)}
            <li class="extractum-panel-shell compact">
              <div>
                <strong>Run #{run.runId}</strong>
                <ExtractumBadge>{statusLabel(run.runStatus)}</ExtractumBadge>
                <ExtractumBadge>{runtimeLabel(run.runtimeProvider)}</ExtractumBadge>
                <p>{run.latestMessage ?? "Waiting for progress."}</p>
              </div>
              <div class="run-actions">
                <ExtractumButton
                  variant="outline"
                  onclick={() => void cancelRun(run.runId)}
                  aria-label={`Cancel prompt pack run ${run.runId}`}
                  title={`Cancel prompt pack run ${run.runId}`}
                >
                  <XCircle size={14} aria-hidden="true" />
                  Cancel
                </ExtractumButton>
                <ExtractumButton
                  class="icon-button danger"
                  variant="destructive"
                  aria-label={`Delete Prompt Pack run ${run.runId}`}
                  title="Delete Prompt Pack run"
                  disabled={isPromptPackRunActive(run) || deletingRunIds[run.runId]}
                  onclick={() => void deleteRun(run)}
                >
                  <Trash2 size={14} aria-hidden="true" />
                </ExtractumButton>
              </div>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <section>
      <h3>Recent</h3>
      {#if recentRuns.length === 0}
        <p class="empty-runs extractum-panel-shell compact">No prompt pack runs yet.</p>
      {:else}
        <ul>
          {#each recentRuns as run (run.runId)}
            <li class="extractum-panel-shell compact" class:selected={run.runId === selectedRunId}>
              <div>
                <strong>Run #{run.runId}</strong>
                <ExtractumBadge>{statusLabel(run.runStatus)}</ExtractumBadge>
                <ExtractumBadge>{runtimeLabel(run.runtimeProvider)}</ExtractumBadge>
                <p>{run.latestMessage ?? run.resultStatus ?? "Completed"}</p>
              </div>
              <div class="run-actions">
                <ExtractumButton
                  variant="outline"
                  onclick={() => (selectedRunId = run.runId)}
                  aria-label={`View prompt pack run ${run.runId} result`}
                  title={`View prompt pack run ${run.runId} result`}
                >
                  View result
                </ExtractumButton>
                <ExtractumButton
                  class="icon-button danger"
                  variant="destructive"
                  aria-label={`Delete Prompt Pack run ${run.runId}`}
                  title="Delete Prompt Pack run"
                  disabled={isPromptPackRunActive(run) || deletingRunIds[run.runId]}
                  onclick={() => void deleteRun(run)}
                >
                  <Trash2 size={14} aria-hidden="true" />
                </ExtractumButton>
              </div>
            </li>
          {/each}
        </ul>
      {/if}
    </section>
  </div>

  {#key selectedRunId}
    <YoutubeSummaryResultView run={selectedRun} runId={selectedRunId} />
  {/key}
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

  li {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 10px;
  }

  li.selected {
    border-color: var(--extractum-info);
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

  .run-actions {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    gap: 6px;
  }

  :global(.icon-button) {
    min-width: 32px;
    width: 32px;
    padding-inline: 0;
  }

  :global(.icon-button.danger) {
    color: var(--extractum-danger);
    border-color: color-mix(in srgb, var(--extractum-danger) 32%, transparent);
    background: color-mix(in srgb, var(--extractum-danger) 8%, transparent);
  }

  :global(.icon-button.danger:hover:enabled) {
    background: color-mix(in srgb, var(--extractum-danger) 14%, transparent);
  }

  :global(.icon-button.danger svg) {
    color: currentColor;
    stroke: currentColor;
  }

  .empty-runs {
    color: var(--extractum-muted);
  }

  .run-error {
    color: var(--extractum-danger);
  }
</style>
