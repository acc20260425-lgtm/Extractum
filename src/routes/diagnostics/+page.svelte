<script lang="ts">
  import { RefreshCw } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { loadDiagnosticSummary } from "$lib/api/diagnostics";
  import DiagnosticCountTable from "$lib/components/diagnostics/DiagnosticCountTable.svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import MetaCell from "$lib/components/ui/MetaCell.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import SurfaceCard from "$lib/components/ui/SurfaceCard.svelte";
  import {
    availabilityLabel,
    availabilityTone,
    buildModeTone,
    diagnosticRowHasIssue,
    filterDiagnosticIssueRows,
    formatDiagnosticError,
    formatSummaryGeneratedAt,
    labelFromKey,
    privacyExcludedDataClasses,
    privacyFallbackNote,
    sortCountRows,
    statusTone,
    yesNo,
  } from "$lib/diagnostics-view-model";
  import type { BadgeVariant } from "$lib/components/ui/types";
  import type { DiagnosticRuntimeCheck, DiagnosticSummaryDto } from "$lib/types/diagnostics";

  type StatusStripItem = {
    label: string;
    value: string;
    tone: BadgeVariant;
    meta: string;
  };
  type DiagnosticTableRow = Record<string, string | number>;
  type DiagnosticTableColumn = {
    key: string;
    label: string;
    align?: "start" | "end";
  };
  type DiagnosticTableSection = {
    title: string;
    description: string;
    columns: DiagnosticTableColumn[];
    rows: DiagnosticTableRow[];
  };
  type VisibleDiagnosticTableSection = DiagnosticTableSection & {
    visibleRows: DiagnosticTableRow[];
  };

  const sourceColumns = [
    { key: "sourceType", label: "Source" },
    { key: "sourceSubtype", label: "Subtype" },
    { key: "active", label: "Active" },
    { key: "syncState", label: "Sync" },
    { key: "count", label: "Count", align: "end" as const },
  ];

  const itemColumns = [
    { key: "sourceType", label: "Source" },
    { key: "sourceSubtype", label: "Subtype" },
    { key: "itemKind", label: "Item kind" },
    { key: "contentKind", label: "Content" },
    { key: "hasContent", label: "Has content" },
    { key: "hasMedia", label: "Has media" },
    { key: "mediaKind", label: "Media" },
    { key: "count", label: "Count", align: "end" as const },
  ];

  const runColumns = [
    { key: "provider", label: "Provider" },
    { key: "runType", label: "Run" },
    { key: "scopeType", label: "Scope" },
    { key: "status", label: "Status" },
    { key: "snapshotState", label: "Snapshot" },
    { key: "errorKind", label: "Error" },
    { key: "count", label: "Count", align: "end" as const },
  ];

  const llmColumns = [
    { key: "provider", label: "Provider" },
    { key: "kind", label: "Kind" },
    { key: "state", label: "State" },
    { key: "count", label: "Count", align: "end" as const },
  ];

  const youtubeJobColumns = [
    { key: "jobType", label: "Job" },
    { key: "status", label: "Status" },
    { key: "warningState", label: "Warning" },
    { key: "errorKind", label: "Error" },
    { key: "count", label: "Count", align: "end" as const },
  ];

  const ingestBatchColumns = [
    { key: "provider", label: "Provider" },
    { key: "ingestKind", label: "Kind" },
    { key: "status", label: "Status" },
    { key: "completeness", label: "Completeness" },
    { key: "errorKind", label: "Error" },
    { key: "count", label: "Count", align: "end" as const },
  ];

  const ingestWarningColumns = [
    { key: "provider", label: "Provider" },
    { key: "ingestKind", label: "Kind" },
    { key: "status", label: "Status" },
    { key: "warningCode", label: "Warning" },
    { key: "count", label: "Count", align: "end" as const },
  ];

  const providerColumns = [
    { key: "provider", label: "Provider" },
    { key: "configuredCount", label: "Configured", align: "end" as const },
    { key: "missingKeyCount", label: "Missing keys", align: "end" as const },
  ];

  const telegramColumns = [
    { key: "status", label: "Runtime status" },
    { key: "count", label: "Count", align: "end" as const },
  ];

  let summary = $state<DiagnosticSummaryDto | null>(null);
  let loading = $state(true);
  let refreshing = $state(false);
  let status = $state("");
  let error = $state<string | null>(null);
  let diagnosticsTableMode = $state<"issues" | "all">("issues");

  async function refreshDiagnostics(initial: boolean) {
    if (initial) {
      loading = true;
      status = "";
    } else {
      refreshing = true;
      status = "Refreshing...";
    }
    error = null;

    try {
      summary = await loadDiagnosticSummary();
      status = "";
    } catch (caught) {
      error = formatDiagnosticError("loading diagnostics", caught);
      status = "";
      if (initial) summary = null;
    } finally {
      if (initial) {
        loading = false;
      } else {
        refreshing = false;
      }
    }
  }

  onMount(() => {
    void refreshDiagnostics(true);
  });

  function runtimeMeta(runtime: DiagnosticRuntimeCheck) {
    return runtime.version ?? runtime.summary ?? labelFromKey(runtime.status);
  }

  function hasDiagnosticIssue(rows: Record<string, string | number | undefined>[]) {
    return rows.some(diagnosticRowHasIssue);
  }

  function visibleDiagnosticRows<T extends Record<string, string | number | undefined>>(rows: T[]) {
    return diagnosticsTableMode === "issues" ? filterDiagnosticIssueRows(rows) : rows;
  }

  function visibleDiagnosticsTableSections(sections: DiagnosticTableSection[]): VisibleDiagnosticTableSection[] {
    return sections
      .map((section) => ({
        ...section,
        visibleRows: visibleDiagnosticRows(section.rows),
      }))
      .filter((section) => diagnosticsTableMode === "all" || section.visibleRows.length > 0);
  }

  function diagnosticsTableSections(current: DiagnosticSummaryDto): DiagnosticTableSection[] {
    return [
      { title: "Provider profiles", description: "Configured profile counts by provider", columns: providerColumns, rows: providerRows(current) },
      { title: "Telegram runtimes", description: "Account runtime statuses by coarse state", columns: telegramColumns, rows: telegramRows(current) },
      { title: "Sources", description: "Source counts by type, subtype, active state, and sync state", columns: sourceColumns, rows: sourceRows(current) },
      { title: "Items", description: "Item counts by coarse source and content fields", columns: itemColumns, rows: itemRows(current) },
      { title: "Analysis runs", description: "Run counts by provider, scope, status, snapshot state, and error kind", columns: runColumns, rows: runRows(current) },
      { title: "LLM requests", description: "Request counts by provider, kind, and state", columns: llmColumns, rows: llmRows(current) },
      { title: "YouTube jobs", description: "Job aggregates by type, status, warning state, and error kind", columns: youtubeJobColumns, rows: youtubeRows(current) },
      { title: "Ingest batches", description: "Batch aggregates by provider, kind, status, completeness, and error kind", columns: ingestBatchColumns, rows: ingestBatchRows(current) },
      { title: "Ingest warnings", description: "Warning aggregates by provider, kind, status, and warning code", columns: ingestWarningColumns, rows: ingestWarningRows(current) },
    ];
  }

  function statusStripItems(current: DiagnosticSummaryDto): StatusStripItem[] {
    return [
      {
        label: "SQLite",
        value: availabilityLabel(current.database.sqliteAvailable),
        tone: availabilityTone(current.database.sqliteAvailable),
        meta: `${current.database.accountCount} accounts`,
      },
      {
        label: "Migrations",
        value: labelFromKey(current.database.migrations.status),
        tone: statusTone(current.database.migrations.status),
        meta: `${current.database.migrations.appliedVersions.length}/${current.database.migrations.expectedVersions.length} applied`,
      },
      {
        label: "Secure storage",
        value: labelFromKey(current.runtimes.secureStorage.status),
        tone: statusTone(current.runtimes.secureStorage.status),
        meta: availabilityLabel(current.runtimes.secureStorage.available),
      },
      {
        label: "yt-dlp",
        value: labelFromKey(current.runtimes.ytdlp.status),
        tone: statusTone(current.runtimes.ytdlp.status),
        meta: runtimeMeta(current.runtimes.ytdlp),
      },
    ];
  }

  function providerRows(current: DiagnosticSummaryDto) {
    return sortCountRows(current.providers.profilesByProvider, ["provider"]).map((row) => ({
      provider: labelFromKey(row.provider),
      configuredCount: row.configuredCount,
      missingKeyCount: row.missingKeyCount,
    }));
  }

  function telegramRows(current: DiagnosticSummaryDto) {
    return sortCountRows(current.telegram.runtimeStatuses, ["status"]).map((row) => ({
      status: labelFromKey(row.status),
      count: row.count,
    }));
  }

  function sourceRows(current: DiagnosticSummaryDto) {
    return sortCountRows(current.sources.counts, ["sourceType", "sourceSubtype", "active", "syncState"]).map((row) => ({
      sourceType: labelFromKey(row.sourceType),
      sourceSubtype: labelFromKey(row.sourceSubtype),
      active: yesNo(row.active),
      syncState: labelFromKey(row.syncState),
      count: row.count,
    }));
  }

  function itemRows(current: DiagnosticSummaryDto) {
    return sortCountRows(current.items.counts, [
      "sourceType",
      "sourceSubtype",
      "itemKind",
      "contentKind",
      "hasContent",
      "hasMedia",
      "mediaKind",
    ]).map((row) => ({
      sourceType: labelFromKey(row.sourceType),
      sourceSubtype: labelFromKey(row.sourceSubtype),
      itemKind: labelFromKey(row.itemKind),
      contentKind: labelFromKey(row.contentKind),
      hasContent: yesNo(row.hasContent),
      hasMedia: yesNo(row.hasMedia),
      mediaKind: labelFromKey(row.mediaKind),
      count: row.count,
    }));
  }

  function runRows(current: DiagnosticSummaryDto) {
    return sortCountRows(current.analysisRuns.counts, [
      "provider",
      "runType",
      "scopeType",
      "status",
      "snapshotState",
      "errorKind",
    ]).map((row) => ({
      provider: labelFromKey(row.provider),
      runType: labelFromKey(row.runType),
      scopeType: labelFromKey(row.scopeType),
      status: labelFromKey(row.status),
      snapshotState: labelFromKey(row.snapshotState),
      errorKind: labelFromKey(row.errorKind),
      count: row.count,
    }));
  }

  function llmRows(current: DiagnosticSummaryDto) {
    return sortCountRows(current.llmRequests.counts, ["provider", "kind", "state"]).map((row) => ({
      provider: labelFromKey(row.provider),
      kind: labelFromKey(row.kind),
      state: labelFromKey(row.state),
      count: row.count,
    }));
  }

  function youtubeRows(current: DiagnosticSummaryDto) {
    return sortCountRows(current.youtubeJobs.counts, ["jobType", "status", "warningState", "errorKind"]).map((row) => ({
      jobType: labelFromKey(row.jobType),
      status: labelFromKey(row.status),
      warningState: labelFromKey(row.warningState),
      errorKind: labelFromKey(row.errorKind),
      count: row.count,
    }));
  }

  function ingestBatchRows(current: DiagnosticSummaryDto) {
    return sortCountRows(current.ingest.batches, ["provider", "ingestKind", "status", "completeness", "errorKind"]).map((row) => ({
      provider: labelFromKey(row.provider),
      ingestKind: labelFromKey(row.ingestKind),
      status: labelFromKey(row.status),
      completeness: labelFromKey(row.completeness),
      errorKind: labelFromKey(row.errorKind),
      count: row.count,
    }));
  }

  function ingestWarningRows(current: DiagnosticSummaryDto) {
    return sortCountRows(current.ingest.warnings, ["provider", "ingestKind", "status", "warningCode"]).map((row) => ({
      provider: labelFromKey(row.provider),
      ingestKind: labelFromKey(row.ingestKind),
      status: labelFromKey(row.status),
      warningCode: labelFromKey(row.warningCode),
      count: row.count,
    }));
  }

  function privacyLabels(current: DiagnosticSummaryDto) {
    return privacyExcludedDataClasses(
      current === summary ? summary.privacy?.excludedDataClasses : current.privacy?.excludedDataClasses,
    );
  }

  function privacyNote(current: DiagnosticSummaryDto) {
    return privacyFallbackNote(
      current === summary ? summary.privacy?.excludedDataClasses : current.privacy?.excludedDataClasses,
    );
  }
</script>

<section class="page-shell diagnostics-page">
  <header class="page-hero">
    <div class="page-hero-copy">
      <span class="page-eyebrow">Operator diagnostics</span>
      <h1>Diagnostics</h1>
      <p>Sanitized local health summary for app, storage, runtimes, providers, sources, and ingest.</p>
      {#if summary}
        <p class="diagnostics-meta">
          v{summary.app.appVersion} - {labelFromKey(summary.app.buildMode)} - {formatSummaryGeneratedAt(summary.app.generatedAtUnix)}
        </p>
      {/if}
    </div>
    <div class="page-hero-meta">
      {#if summary}
        <Badge variant={buildModeTone(summary.app.buildMode)}>{labelFromKey(summary.app.buildMode)}</Badge>
        <Badge variant="neutral">{summary.app.appName}</Badge>
      {/if}
      <Button
        size="sm"
        variant="secondary"
        ariaLabel="Refresh diagnostics"
        disabled={loading || refreshing}
        onclick={() => void refreshDiagnostics(false)}
      >
        <RefreshCw size={14} aria-hidden="true" />
        Refresh
      </Button>
    </div>
  </header>

  {#if status}
    <StatusMessage tone="info" className="page-status">{status}</StatusMessage>
  {/if}

  {#if error}
    <StatusMessage tone="error" className="page-status">{error}</StatusMessage>
  {/if}

  {#if summary}
    {@const tableSections = diagnosticsTableSections(summary)}

    <div class="diagnostics-table-controls extractum-toolbar-row" aria-label="Diagnostics table display">
      <Button
        size="sm"
        variant="secondary"
        selected={diagnosticsTableMode === "issues"}
        ariaLabel="Show diagnostics issues only"
        onclick={() => (diagnosticsTableMode = "issues")}
      >
        Only issues
      </Button>
      <Button
        size="sm"
        variant="secondary"
        selected={diagnosticsTableMode === "all"}
        ariaLabel="Show all diagnostics tables"
        onclick={() => (diagnosticsTableMode = "all")}
      >
        All tables
      </Button>
    </div>

    {#if diagnosticsTableMode === "issues"}
      {@render diagnosticsTableArea(tableSections)}
      {@render diagnosticsOverviewArea(summary)}
    {:else}
      {@render diagnosticsOverviewArea(summary)}
      {@render diagnosticsTableArea(tableSections)}
    {/if}
  {:else if loading}
    <StatusMessage tone="muted" className="page-status">Loading diagnostics...</StatusMessage>
  {/if}
</section>

{#snippet diagnosticsOverviewArea(current: DiagnosticSummaryDto)}
  <div class="diagnostics-overview-area">
    <div class="status-strip" aria-label="Diagnostics health overview">
      {#each statusStripItems(current) as item (item.label)}
        <div class="status-tile extractum-panel-shell">
          <span>{item.label}</span>
          <strong>{item.value}</strong>
          <Badge variant={item.tone}>{item.meta}</Badge>
        </div>
      {/each}
    </div>

    <div class="diagnostics-grid">
      <SurfaceCard title="App and build" meta="Factual diagnostic summary metadata">
        <div class="meta-grid">
          <MetaCell label="App">{current.app.appName}</MetaCell>
          <MetaCell label="Version">{current.app.appVersion}</MetaCell>
          <MetaCell label="Build">{labelFromKey(current.app.buildMode)}</MetaCell>
          <MetaCell label="Generated">{formatSummaryGeneratedAt(current.app.generatedAtUnix).replace("Summary generated ", "")}</MetaCell>
        </div>
      </SurfaceCard>

      <SurfaceCard title="Database" meta="SQLite availability and migration state">
        <div class="meta-grid">
          <MetaCell label="SQLite">{availabilityLabel(current.database.sqliteAvailable)}</MetaCell>
          <MetaCell label="Migrations">{labelFromKey(current.database.migrations.status)}</MetaCell>
          <MetaCell label="Accounts">{current.database.accountCount}</MetaCell>
          <MetaCell label="Pending versions">{current.database.migrations.pendingVersions.length}</MetaCell>
          <MetaCell label="Failed versions">{current.database.migrations.failedVersions.length}</MetaCell>
        </div>
      </SurfaceCard>

      <SurfaceCard title="Runtimes" meta="Backend-reported runtime checks">
        <div class="meta-grid">
          <MetaCell label="Secure storage">{labelFromKey(current.runtimes.secureStorage.status)}</MetaCell>
          <MetaCell label="Secure storage available">{availabilityLabel(current.runtimes.secureStorage.available)}</MetaCell>
          <MetaCell label="yt-dlp">{labelFromKey(current.runtimes.ytdlp.status)}</MetaCell>
          <MetaCell label="yt-dlp available">{availabilityLabel(current.runtimes.ytdlp.available)}</MetaCell>
          <MetaCell label="yt-dlp version">{current.runtimes.ytdlp.version ?? "Unknown"}</MetaCell>
        </div>
      </SurfaceCard>

      <SurfaceCard title="Privacy boundary" meta="Data classes intentionally excluded by backend diagnostics">
        {#if privacyLabels(current).length > 0}
          <div class="privacy-chips">
            {#each privacyLabels(current) as item (item)}
              <Badge variant="neutral">{item}</Badge>
            {/each}
          </div>
        {:else}
          <StatusMessage tone="muted" surface={false}>{privacyNote(current)}</StatusMessage>
        {/if}
      </SurfaceCard>
    </div>
  </div>
{/snippet}

{#snippet diagnosticsTableArea(tableSections: DiagnosticTableSection[])}
  {@const visibleSections = visibleDiagnosticsTableSections(tableSections)}
  <div class="diagnostics-table-area diagnostics-tables">
    {#each visibleSections as section (section.title)}
      <DiagnosticCountTable
        title={section.title}
        description={section.description}
        columns={section.columns}
        rows={section.visibleRows}
        totalRows={section.rows.length}
        open={hasDiagnosticIssue(section.rows)}
      />
    {:else}
      <StatusMessage tone="muted" className="diagnostics-empty-state">
        No diagnostic issue rows match this view.
      </StatusMessage>
    {/each}
  </div>
{/snippet}

<style>
  .diagnostics-page {
    gap: 0.95rem;
  }

  .diagnostics-meta {
    font-size: 0.86rem;
  }

  .diagnostics-overview-area {
    display: flex;
    flex-direction: column;
    gap: 0.95rem;
  }

  .status-strip {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 0.7rem;
  }

  .status-tile {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 0.35rem;
  }

  .status-tile span {
    color: var(--muted);
    font-size: 0.76rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .status-tile strong {
    font-size: 0.98rem;
  }

  .diagnostics-grid,
  .diagnostics-tables {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.9rem;
    align-items: start;
  }

  :global(.diagnostics-empty-state.ui-status-message) {
    grid-column: 1 / -1;
  }

  .meta-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.65rem;
  }

  .privacy-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .diagnostics-table-controls {
    justify-content: flex-end;
  }

  @media (max-width: 980px) {
    .status-strip,
    .diagnostics-grid,
    .diagnostics-tables {
      grid-template-columns: 1fr;
    }

    .meta-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
