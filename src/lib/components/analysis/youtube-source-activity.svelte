<script lang="ts">
  import { Square } from "@lucide/svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import type { BadgeVariant } from "$lib/components/ui/types";
  import type { SourceJobRecord } from "$lib/types/sources";

  let {
    jobs,
    formatTimestamp,
    onCancelJob,
    onSyncSource = null,
    title = "Source activity",
  }: {
    jobs: SourceJobRecord[];
    formatTimestamp: (value: number | null) => string;
    onCancelJob: (jobId: string) => void | Promise<void>;
    onSyncSource?: (() => void | Promise<void>) | null;
    title?: string;
  } = $props();

  const visibleJobs = $derived(
    [...jobs].sort((left, right) => right.started_at - left.started_at).slice(0, 8),
  );

  function isActiveJob(job: SourceJobRecord) {
    return job.status === "queued" || job.status === "running" || job.status === "cancel_requested";
  }

  function statusVariant(status: SourceJobRecord["status"]): BadgeVariant {
    if (status === "failed" || status === "cancelled") return "danger";
    if (status === "succeeded") return "success";
    if (status === "cancel_requested") return "warning";
    return "info";
  }

  function jobLabel(job: SourceJobRecord) {
    return job.job_type.replaceAll("_", " ");
  }

  function statusLabel(job: SourceJobRecord) {
    return job.status.replaceAll("_", " ");
  }

  function progressLabel(job: SourceJobRecord) {
    if (job.progress_current === null || job.progress_total === null) return null;
    return `${job.progress_current}/${job.progress_total}`;
  }
</script>

{#if visibleJobs.length > 0}
  <section class="youtube-source-activity" aria-label={title}>
    <div class="activity-heading">
      <span class="eyebrow">{title}</span>
      <Badge variant="neutral">{visibleJobs.length} recent</Badge>
    </div>

    {#if onSyncSource}
      <div class="activity-action-grid">
        <Button type="button" variant="secondary" onclick={onSyncSource}>
          Sync source
        </Button>
      </div>
    {/if}

    <div class="activity-list">
      {#each visibleJobs as job (job.job_id)}
        {@const progress = progressLabel(job)}
        <article class="activity-row">
          <div class="activity-copy">
            <strong>{jobLabel(job)}</strong>
            <span>{job.message ?? job.error ?? statusLabel(job)}</span>
            <small>
              Started {formatTimestamp(job.started_at)}
              {#if job.finished_at !== null}
                - Finished {formatTimestamp(job.finished_at)}
              {/if}
            </small>
            {#if progress}
              <small>Progress {progress}</small>
            {/if}
            {#if job.warnings.length > 0}
              <ul class="warning-list" aria-label="Job warnings">
                {#each job.warnings as warning, index (`${job.job_id}-warning-${index}`)}
                  <li>{warning}</li>
                {/each}
              </ul>
            {/if}
            {#if job.error}
              <small class="job-error">{job.error}</small>
            {/if}
          </div>

          <div class="activity-actions">
            <Badge variant={statusVariant(job.status)}>{statusLabel(job)}</Badge>
            {#if isActiveJob(job)}
              <Button
                type="button"
                size="sm"
                variant="secondary"
                disabled={job.status === "cancel_requested"}
                onclick={() => onCancelJob(job.job_id)}
              >
                <Square size={13} aria-hidden="true" /> Cancel
              </Button>
            {/if}
          </div>
        </article>
      {/each}
    </div>
  </section>
{/if}

<style>
  .youtube-source-activity {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    padding: 0.75rem;
    border: 1px solid color-mix(in srgb, var(--border) 82%, transparent);
    border-radius: 8px;
    background: color-mix(in srgb, var(--panel-strong) 58%, transparent);
  }

  .activity-heading,
  .activity-row,
  .activity-action-grid,
  .activity-actions {
    display: flex;
    align-items: flex-start;
    gap: 0.55rem;
  }

  .activity-heading {
    justify-content: space-between;
    align-items: center;
  }

  .activity-action-grid {
    flex-wrap: wrap;
    align-items: center;
    padding: 0.65rem;
    border: 1px solid color-mix(in srgb, var(--primary) 18%, transparent);
    border-radius: 8px;
    background: color-mix(in srgb, var(--primary) 6%, var(--panel));
  }

  .eyebrow {
    display: inline-block;
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .activity-list {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }

  .activity-row {
    justify-content: space-between;
    padding: 0.6rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
  }

  .activity-copy {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.18rem;
  }

  .activity-copy span,
  .activity-copy small,
  .warning-list {
    color: var(--muted);
    font-size: 0.78rem;
    overflow-wrap: anywhere;
  }

  .warning-list {
    margin: 0.15rem 0 0;
    padding-left: 1rem;
  }

  .job-error {
    color: var(--danger);
  }

  .activity-actions {
    align-items: center;
    justify-content: flex-end;
    flex-wrap: wrap;
  }

  @media (max-width: 760px) {
    .activity-row {
      flex-direction: column;
    }

    .activity-actions {
      justify-content: flex-start;
    }
  }
</style>
