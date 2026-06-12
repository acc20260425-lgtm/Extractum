<script lang="ts">
  import type { AnalysisRunSummary } from "$lib/types/analysis";
  import type { SourceJobRecord } from "$lib/types/sources";

  let {
    loading,
    saving,
    status,
    sourceJobs,
    runs,
  }: {
    loading: boolean;
    saving: boolean;
    status: string;
    sourceJobs: SourceJobRecord[];
    runs: AnalysisRunSummary[];
  } = $props();

  let activeSourceJobs = $derived(
    sourceJobs.filter((job) => job.status === "queued" || job.status === "running" || job.status === "failed"),
  );
  let activeRuns = $derived(runs.filter((run) => run.status === "queued" || run.status === "running"));

  function progressLabel(job: SourceJobRecord) {
    if (job.progress_current === null || job.progress_total === null) return job.message ?? job.status;
    return `${job.progress_current}/${job.progress_total}`;
  }
</script>

<section class="bottom-queue" data-ui-region="bottom-queue" aria-label="Background queue">
  <div class="queue-state">
    {#if loading}
      <span>Loading workspace</span>
    {/if}
    {#if saving}
      <span>Saving changes</span>
    {/if}
    {#if status}
      <span>{status}</span>
    {/if}
    {#if !loading && !saving && !status && activeSourceJobs.length === 0 && activeRuns.length === 0}
      <span>No active jobs</span>
    {/if}
  </div>

  <div class="queue-items">
    {#each activeSourceJobs as job (job.job_id)}
      <span class:failed={job.status === "failed"}>
        Source #{job.source_id}: {job.error ?? progressLabel(job)}
      </span>
    {/each}
    {#each activeRuns as run (run.id)}
      <span>{run.scope_label}: {run.status}</span>
    {/each}
  </div>
</section>

<style>
  .bottom-queue {
    display: flex;
    min-height: 38px;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    border-top: 1px solid var(--extractum-border);
    background: var(--extractum-surface-subtle);
    padding: 6px 12px;
    color: var(--extractum-muted);
    font-size: 12px;
  }

  .queue-state,
  .queue-items {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: 8px;
    overflow: hidden;
  }

  .queue-items {
    justify-content: flex-end;
  }

  .queue-items span,
  .queue-state span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .queue-items span {
    border: 1px solid var(--extractum-border);
    border-radius: 999px;
    background: var(--extractum-surface);
    padding: 2px 8px;
  }

  .queue-items span.failed {
    border-color: color-mix(in srgb, var(--extractum-danger) 35%, var(--extractum-border));
    color: var(--extractum-danger);
  }
</style>
