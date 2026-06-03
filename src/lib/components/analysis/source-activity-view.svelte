<script lang="ts">
  import { Square } from "@lucide/svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import TakeoutRecoveryNotice from "$lib/components/analysis/takeout-recovery-notice.svelte";
  import type { BadgeVariant } from "$lib/components/ui/types";
  import type {
    Source,
    SourceJobRecord,
    TakeoutImportRecoveryState,
  } from "$lib/types/sources";

  let {
    source,
    jobs,
    takeoutRecovery,
    sourceSyncDisabledReason,
    formatTimestamp,
    onSyncSource,
    onSyncMetadata,
    onSyncTranscript,
    onSyncComments,
    onStartTakeoutImport,
    onStartMigratedHistoryImport,
    onCancelSourceJob,
  }: {
    source: Source;
    jobs: SourceJobRecord[];
    takeoutRecovery: TakeoutImportRecoveryState | null;
    sourceSyncDisabledReason: (source: Source) => string | null;
    formatTimestamp: (value: number | null) => string;
    onSyncSource: (sourceId: number) => void | Promise<void>;
    onSyncMetadata: (sourceId: number) => void | Promise<void>;
    onSyncTranscript: (sourceId: number) => void | Promise<void>;
    onSyncComments: (sourceId: number) => void | Promise<void>;
    onStartTakeoutImport: (sourceId: number) => void | Promise<void>;
    onStartMigratedHistoryImport: (sourceId: number) => void | Promise<void>;
    onCancelSourceJob: (jobId: string) => void | Promise<void>;
  } = $props();

  const visibleJobs = $derived(
    [...jobs].sort((left, right) => right.started_at - left.started_at).slice(0, 8),
  );
  const disabledReason = $derived(sourceSyncDisabledReason(source));
  const canImportMigratedHistory = $derived(
    source.sourceType === "telegram"
      && source.migratedHistoryStatus === "available"
      && !source.migratedHistoryImportCompleted,
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

<section class="source-activity-view" aria-label="Source activity">
  <div class="activity-action-grid">
    <Button
      type="button"
      variant="secondary"
      disabled={disabledReason !== null}
      title={disabledReason ?? undefined}
      onclick={() => onSyncSource(source.id)}
    >
      Sync source
    </Button>
    {#if source.sourceType === "telegram"}
      <Button type="button" variant="secondary" onclick={() => onStartTakeoutImport(source.id)}>
        Start Takeout import
      </Button>
    {/if}
  </div>

  <section class="activity-section" aria-label="Source sync">
    <div class="section-heading">
      <span class="eyebrow">Source sync</span>
      <Badge variant={source.lastSyncedAt === null ? "neutral" : "success"}>
        {source.lastSyncedAt === null ? "not synced" : "synced"}
      </Badge>
    </div>
    <p>Last synced {formatTimestamp(source.lastSyncedAt)}</p>
    {#if source.lastSyncState !== null}
      <p>Sync state #{source.lastSyncState}</p>
    {/if}
    {#if disabledReason}
      <StatusMessage tone="error">{disabledReason}</StatusMessage>
    {/if}
    <div class="activity-actions">
      {#if source.sourceType === "youtube" && source.sourceSubtype === "video"}
        <Button type="button" variant="secondary" onclick={() => onSyncMetadata(source.id)}>Sync metadata</Button>
        <Button type="button" variant="secondary" onclick={() => onSyncTranscript(source.id)}>Sync transcript</Button>
        <Button type="button" variant="secondary" onclick={() => onSyncComments(source.id)}>Sync comments</Button>
      {/if}
    </div>
  </section>

  {#if source.sourceType === "telegram"}
    <section class="activity-section" aria-label="Telegram recovery">
      <div class="section-heading">
        <span class="eyebrow">Takeout</span>
        <Badge variant={takeoutRecovery ? "warning" : "neutral"}>
          {takeoutRecovery ? takeoutRecovery.recovery_kind.replaceAll("_", " ") : "no recovery"}
        </Badge>
      </div>
      {#if takeoutRecovery}
        <TakeoutRecoveryNotice recovery={takeoutRecovery} compact={true} />
      {:else}
        <p>No Takeout recovery is currently visible for this source.</p>
      {/if}
    </section>

    <section class="activity-section" aria-label="Migrated history">
      <div class="section-heading">
        <span class="eyebrow">Migrated history</span>
        <Badge variant={source.migratedHistoryImportCompleted ? "success" : canImportMigratedHistory ? "warning" : "neutral"}>
          {source.migratedHistoryStatus.replaceAll("_", " ")}
        </Badge>
      </div>
      <p>
        {source.migratedHistoryRowCount} imported migrated rows. This Activity tab shows route-owned Telegram recovery
        state while detailed Telegram job history remains a follow-up.
      </p>
      {#if canImportMigratedHistory}
        <div class="activity-actions">
          <Button type="button" variant="secondary" onclick={() => onStartMigratedHistoryImport(source.id)}>
            Start migrated history import
          </Button>
        </div>
      {/if}
    </section>
  {/if}

  <section class="activity-section" aria-label="Detailed source jobs">
    <div class="section-heading">
      <span class="eyebrow">Detailed jobs</span>
      <Badge variant="neutral">{visibleJobs.length} recent</Badge>
    </div>

    {#if visibleJobs.length === 0}
      <StatusMessage tone="muted">No detailed source jobs are loaded for this source.</StatusMessage>
    {:else}
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
                <div class="warning-block">
                  <small>Warnings</small>
                  <ul aria-label="Job warnings">
                    {#each job.warnings as warning, index (`${job.job_id}-warning-${index}`)}
                      <li>{warning}</li>
                    {/each}
                  </ul>
                </div>
              {/if}
              {#if job.error}
                <small class="job-error">Error {job.error}</small>
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
                  onclick={() => onCancelSourceJob(job.job_id)}
                >
                  <Square size={13} aria-hidden="true" /> Cancel
                </Button>
              {/if}
            </div>
          </article>
        {/each}
      </div>
    {/if}
  </section>
</section>

<style>
  .source-activity-view,
  .activity-section,
  .activity-list,
  .activity-copy,
  .warning-block {
    display: flex;
    flex-direction: column;
  }

  .source-activity-view {
    gap: 0.75rem;
    min-width: 0;
  }

  .activity-section {
    gap: 0.55rem;
    padding: 0.75rem;
    border: 1px solid color-mix(in srgb, var(--border) 82%, transparent);
    border-radius: 8px;
    background: color-mix(in srgb, var(--panel-strong) 58%, transparent);
  }

  .section-heading,
  .activity-row,
  .activity-action-grid,
  .activity-actions {
    display: flex;
    align-items: flex-start;
    gap: 0.55rem;
  }

  .section-heading {
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

  .activity-section p {
    margin: 0;
    color: var(--muted);
    font-size: 0.84rem;
    line-height: 1.45;
  }

  .activity-list {
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
    gap: 0.18rem;
  }

  .activity-copy span,
  .activity-copy small,
  .warning-block ul {
    color: var(--muted);
    font-size: 0.78rem;
    overflow-wrap: anywhere;
  }

  .warning-block {
    gap: 0.1rem;
  }

  .warning-block ul {
    margin: 0;
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
