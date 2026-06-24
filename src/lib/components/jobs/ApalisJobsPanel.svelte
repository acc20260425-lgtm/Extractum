<script lang="ts">
  import { RefreshCw, Trash2 } from "@lucide/svelte";
  import { onDestroy, onMount } from "svelte";
  import { loadApalisJobs, pruneOldTerminalApalisJobs } from "$lib/api/apalis-jobs";
  import {
    ExtractumDataGrid,
    type ExtractumDataGridColumn,
  } from "$lib/components/extractum-ui";
  import { formatDataGridDateTimeValue } from "$lib/components/extractum-ui/data-grid-date-format";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import SurfaceCard from "$lib/components/ui/SurfaceCard.svelte";
  import type {
    ApalisJobRow,
    ApalisJobStatusCount,
    ApalisJobsListRequest,
    ApalisJobsListResponse,
    ApalisJsonValue,
  } from "$lib/types/apalis-jobs";

  const baseStatusOptions = ["Pending", "Queued", "Running", "Done", "Failed", "Killed"];
  const limitOptions = [50, 100, 200, 500];
  const columns: ExtractumDataGridColumn[] = [
    { id: "status", header: "Status", width: 110 },
    { id: "jobType", header: "Job type", width: 170 },
    { id: "key", header: "Key", width: 220 },
    { id: "attemptsLabel", header: "Attempts", width: 95 },
    { id: "lastActivityAt", header: "Activity", width: 165, dateTimeFormat: "datetime" },
  ];

  type ApalisJobGridRow = {
    id: string;
    status: string;
    jobType: string;
    key: string;
    attemptsLabel: string;
    lastActivityAt: string | null;
  };

  let response = $state<ApalisJobsListResponse | null>(null);
  let loading = $state(true);
  let refreshing = $state(false);
  let pruning = $state(false);
  let error = $state<string | null>(null);
  let pruneMessage = $state<string | null>(null);
  let statusFilter = $state("");
  let jobTypeFilter = $state("");
  let search = $state("");
  let limit = $state(100);
  let selectedJobId = $state<string | null>(null);
  let searchDebounce: ReturnType<typeof setTimeout> | null = null;
  let refreshSequence = 0;

  let selectedJob = $derived(
    selectedJobId ? response?.jobs.find((job) => job.id === selectedJobId) ?? null : null,
  );
  let statusOptions = $derived(statusFilterOptions(response?.statusCounts ?? [], statusFilter));
  let gridRows = $derived((response?.jobs ?? []).map(jobToGridRow));
  let selectedRowIds = $derived(selectedJobId ? [selectedJobId] : []);

  onMount(() => {
    void refreshJobs(true);
  });

  onDestroy(() => {
    clearSearchDebounce();
    refreshSequence += 1;
  });

  function request(): ApalisJobsListRequest {
    return {
      limit,
      status: statusFilter || null,
      jobType: jobTypeFilter || null,
      search: search.trim() || null,
    };
  }

  function clearSearchDebounce() {
    if (searchDebounce) {
      clearTimeout(searchDebounce);
      searchDebounce = null;
    }
  }

  async function refreshJobs(initial: boolean) {
    clearSearchDebounce();
    const sequence = ++refreshSequence;
    const currentRequest = request();
    if (initial) {
      loading = true;
    } else {
      refreshing = true;
    }
    error = null;

    try {
      const next = await loadApalisJobs(currentRequest);
      if (sequence !== refreshSequence) return;
      response = next;
      if (selectedJobId && !next.jobs.some((job) => job.id === selectedJobId)) {
        selectedJobId = next.jobs[0]?.id ?? null;
      } else if (initial && !selectedJobId) {
        selectedJobId = next.jobs[0]?.id ?? null;
      }
    } catch (caught) {
      if (sequence !== refreshSequence) return;
      error = caught instanceof Error ? caught.message : String(caught);
      if (initial) response = null;
    } finally {
      if (sequence === refreshSequence) {
        loading = false;
        refreshing = false;
      }
    }
  }

  async function pruneOldTerminalJobs() {
    const confirmed = confirm(
      "Delete finished Apalis jobs older than 24 hours? This includes Done, Killed, and Failed jobs with no retries left. This cannot be undone.",
    );
    if (!confirmed) return;

    pruning = true;
    error = null;
    pruneMessage = null;

    try {
      const result = await pruneOldTerminalApalisJobs();
      pruneMessage =
        result.deletedCount === 1
          ? "Deleted 1 old finished job."
          : `Deleted ${result.deletedCount} old finished jobs.`;
      await refreshJobs(false);
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      pruning = false;
    }
  }

  function handleFilterChange(options: { debounce?: boolean } = {}) {
    clearSearchDebounce();
    if (options.debounce) {
      searchDebounce = setTimeout(() => {
        searchDebounce = null;
        void refreshJobs(false);
      }, 250);
      return;
    }
    void refreshJobs(false);
  }

  function formatTime(value: string | null) {
    return String(formatDataGridDateTimeValue(value, "datetime") ?? "Never");
  }

  function statusFilterOptions(counts: ApalisJobStatusCount[], selectedStatus: string) {
    const seen = new Set(baseStatusOptions);
    const unknownStatuses = counts
      .map((row) => row.status)
      .filter((status) => status && !seen.has(status))
      .sort();
    if (selectedStatus && !seen.has(selectedStatus) && !unknownStatuses.includes(selectedStatus)) {
      unknownStatuses.push(selectedStatus);
      unknownStatuses.sort();
    }
    return ["", ...baseStatusOptions, ...unknownStatuses];
  }

  function jobToGridRow(job: ApalisJobRow): ApalisJobGridRow {
    return {
      id: job.id,
      status: job.status,
      jobType: job.jobType || "unknown",
      key: job.idempotencyKey ?? job.id,
      attemptsLabel: `${job.attempts}/${job.maxAttempts ?? "-"}`,
      lastActivityAt: job.lastActivityAt,
    };
  }

  function handleGridSelection(ids: string[]) {
    selectedJobId = ids[0] ?? null;
  }

  function statusTone(status: string) {
    if (status === "Done") return "success";
    if (status === "Failed" || status === "Killed") return "danger";
    if (status === "Running") return "info";
    return "default";
  }

  function countForStatus(status: string) {
    return response?.statusCounts.find((row) => row.status === status)?.count ?? 0;
  }

  function jsonPreview(value: ApalisJsonValue | null, fallback: string | null) {
    if (value !== null) return JSON.stringify(value, null, 2);
    return fallback ?? "No data";
  }
</script>

<section class="page-shell jobs-page">
  <div class="page-hero jobs-hero">
    <div class="jobs-hero-content">
      <span class="page-eyebrow">Apalis queue</span>
      <h1>Jobs</h1>
      <p>Inspector and maintenance tools for local Apalis jobs.</p>
    </div>
    <div class="jobs-actions">
      <Button variant="secondary" onclick={() => refreshJobs(false)} disabled={loading || refreshing || pruning}>
        <RefreshCw size={15} aria-hidden="true" />
        Refresh
      </Button>
      <Button
        variant="danger-soft"
        onclick={pruneOldTerminalJobs}
        disabled={loading || refreshing || pruning}
        title="Delete finished jobs older than 24 hours: Done, Killed, and Failed with no retries left"
      >
        <Trash2 size={15} aria-hidden="true" />
        {pruning ? "Deleting..." : "Delete old finished jobs"}
      </Button>
    </div>
  </div>

  <div class="jobs-layout">
    <SurfaceCard className="jobs-list-panel">
      <div class="jobs-toolbar">
        <label>
          <span>Status</span>
          <select bind:value={statusFilter} onchange={() => handleFilterChange()}>
            {#each statusOptions as status (status)}
              <option value={status}>{status || "All statuses"}</option>
            {/each}
          </select>
        </label>
        <label>
          <span>Job type</span>
          <select bind:value={jobTypeFilter} onchange={() => handleFilterChange()}>
            <option value="">All job types</option>
            {#each response?.jobTypeCounts ?? [] as row (row.jobType)}
              <option value={row.jobType}>{row.jobType || "unknown"} ({row.count})</option>
            {/each}
          </select>
        </label>
        <label class="search-control">
          <span>Search</span>
          <input
            bind:value={search}
            placeholder="id or idempotency key"
            oninput={() => handleFilterChange({ debounce: true })}
          />
        </label>
        <label>
          <span>Limit</span>
          <select bind:value={limit} onchange={() => handleFilterChange()}>
            {#each limitOptions as option (option)}
              <option value={option}>{option}</option>
            {/each}
          </select>
        </label>
      </div>

      {#if error}
        <StatusMessage tone="error">{error}</StatusMessage>
      {/if}
      {#if pruneMessage}
        <StatusMessage tone="info">{pruneMessage}</StatusMessage>
      {/if}

      <div class="status-strip">
        {#each statusOptions.filter(Boolean) as status (status)}
          <Badge variant={statusTone(status)}>{status} {countForStatus(status)}</Badge>
        {/each}
      </div>

      <div class="jobs-grid">
        <ExtractumDataGrid
          rows={gridRows}
          {columns}
          {selectedRowIds}
          overlay={loading ? "Loading Apalis jobs..." : "No Apalis jobs match these filters."}
          onSelectedRowIdsChange={handleGridSelection}
        />
      </div>

      <div class="jobs-summary">
        <span>{response?.totalMatching ?? 0} matching</span>
        <span>Limit {response?.limit ?? limit}</span>
        <span>Refreshed {formatTime(response?.refreshedAt ?? null)}</span>
      </div>
    </SurfaceCard>

    <SurfaceCard className="jobs-detail-panel">
      {#if selectedJob}
        <div class="detail-header">
          <div>
            <span class="detail-kicker">Selected job</span>
            <h2>{selectedJob.idempotencyKey ?? selectedJob.id}</h2>
          </div>
          <Badge variant={statusTone(selectedJob.status)}>{selectedJob.status}</Badge>
        </div>
        <dl class="detail-list">
          <div><dt>Job type</dt><dd>{selectedJob.jobType || "unknown"}</dd></div>
          <div><dt>Attempts</dt><dd>{selectedJob.attempts}/{selectedJob.maxAttempts ?? "-"}</dd></div>
          <div><dt>Priority</dt><dd>{selectedJob.priority ?? "-"}</dd></div>
          <div><dt>Run at</dt><dd>{formatTime(selectedJob.runAt)}</dd></div>
          <div><dt>Lock at</dt><dd>{formatTime(selectedJob.lockAt)}</dd></div>
          <div><dt>Done at</dt><dd>{formatTime(selectedJob.doneAt)}</dd></div>
          <div><dt>Lock by</dt><dd>{selectedJob.lockBy ?? "-"}</dd></div>
        </dl>

        {@render payloadSection("Job payload", selectedJob.jobJson, selectedJob.jobPreview, selectedJob.jobTruncated)}
        {@render payloadSection("Last result", selectedJob.lastResult, null, selectedJob.lastResultTruncated)}
        {@render payloadSection("Metadata", selectedJob.metadata, null, selectedJob.metadataTruncated)}
      {:else}
        <div class="empty-detail">
          <h2>Select a job</h2>
          <p>No Apalis jobs match these filters.</p>
        </div>
      {/if}
    </SurfaceCard>
  </div>
</section>

{#snippet payloadSection(
  title: string,
  value: ApalisJsonValue | null,
  fallback: string | null,
  truncated: boolean,
)}
  <section class="payload-section">
    <div class="payload-title">
      <h3>{title}</h3>
      <div class="payload-badges">
        {#if truncated}
          <Badge variant="warning">truncated</Badge>
        {/if}
        {#if value !== null || fallback}
          <Badge variant="neutral">redacted</Badge>
        {/if}
      </div>
    </div>
    <pre>{jsonPreview(value, fallback)}</pre>
  </section>
{/snippet}

<style>
  .jobs-page {
    min-height: calc(100vh - 68px);
    overflow: hidden;
  }

  .jobs-hero {
    align-items: center;
  }

  .jobs-hero-content {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    min-width: 0;
  }

  .jobs-hero-content h1,
  .jobs-hero-content p {
    margin: 0;
  }

  .jobs-hero-content p {
    color: var(--muted);
  }

  .jobs-actions {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
    flex-wrap: wrap;
  }

  .jobs-layout {
    display: grid;
    grid-template-columns: minmax(0, 1.35fr) minmax(340px, 0.9fr);
    gap: 0.9rem;
    min-height: min(720px, calc(100vh - 180px));
    min-width: 0;
  }

  :global(.jobs-list-panel),
  :global(.jobs-detail-panel) {
    min-height: 0;
    overflow: hidden;
  }

  .jobs-toolbar {
    display: grid;
    grid-template-columns: minmax(120px, 0.8fr) minmax(160px, 1fr) minmax(220px, 1.4fr) 90px;
    gap: 0.65rem;
    align-items: end;
  }

  .jobs-toolbar label {
    display: grid;
    gap: 0.25rem;
    min-width: 0;
  }

  .jobs-toolbar span,
  .detail-kicker,
  .jobs-summary {
    color: var(--muted);
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .jobs-toolbar input,
  .jobs-toolbar select {
    min-height: 2.25rem;
    border-radius: 6px;
    font-size: 0.86rem;
  }

  .status-strip {
    display: flex;
    gap: 0.4rem;
    flex-wrap: wrap;
    min-height: 1.45rem;
  }

  .jobs-grid {
    min-width: 0;
    min-height: 0;
    flex: 1;
  }

  .jobs-summary {
    display: flex;
    justify-content: space-between;
    gap: 0.7rem;
    flex-wrap: wrap;
  }

  .detail-header,
  .payload-title,
  .payload-badges {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .detail-header,
  .payload-title {
    justify-content: space-between;
  }

  .detail-header h2 {
    margin: 0.15rem 0 0;
    font-size: 1rem;
    line-height: 1.25;
    overflow-wrap: anywhere;
  }

  .detail-list {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.45rem;
    margin: 0;
  }

  .detail-list div {
    display: grid;
    gap: 0.1rem;
    min-width: 0;
    padding: 0.45rem 0.5rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--panel);
  }

  .detail-list dt {
    color: var(--muted);
    font-size: 0.72rem;
  }

  .detail-list dd {
    margin: 0;
    min-width: 0;
    overflow-wrap: anywhere;
    font-size: 0.86rem;
  }

  .payload-section {
    display: grid;
    gap: 0.4rem;
    min-width: 0;
  }

  .payload-title h3 {
    margin: 0;
    font-size: 0.9rem;
  }

  .payload-section pre {
    max-height: 160px;
    min-height: 54px;
    overflow: auto;
    margin: 0;
    padding: 0.65rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--panel);
    color: var(--text);
    font-size: 0.78rem;
    line-height: 1.45;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }

  .empty-detail {
    display: grid;
    place-content: center;
    gap: 0.35rem;
    min-height: 280px;
    color: var(--muted);
    text-align: center;
  }

  .empty-detail h2,
  .empty-detail p {
    margin: 0;
  }

  @media (max-width: 980px) {
    .jobs-layout {
      grid-template-columns: 1fr;
      min-height: auto;
    }

    .jobs-toolbar {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .jobs-grid {
      min-height: 360px;
    }
  }

  @media (max-width: 640px) {
    .jobs-actions {
      justify-content: stretch;
    }

    .jobs-toolbar,
    .detail-list {
      grid-template-columns: 1fr;
    }
  }
</style>
