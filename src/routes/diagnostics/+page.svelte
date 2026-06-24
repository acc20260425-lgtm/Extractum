<script lang="ts">
  import { RefreshCw } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { loadDiagnosticSummary } from "$lib/api/diagnostics";
  import DiagnosticCountTable from "$lib/components/diagnostics/DiagnosticCountTable.svelte";
  import {
    ExtractumBadge,
    ExtractumButton,
    ExtractumStatusMessage,
  } from "$lib/components/extractum-ui";
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
      <div class="extractum-toolbar-row gap-2" role="group" aria-label="Diagnostics page actions">
        {#if summary}
          <span class="hero-pill">Build mode</span>
          <ExtractumBadge>{labelFromKey(summary.app.buildMode)}</ExtractumBadge>
          <span class="hero-pill">Application</span>
          <ExtractumBadge>{summary.app.appName}</ExtractumBadge>
        {/if}
        <ExtractumButton
          size="sm"
          variant="outline"
          aria-label="Refresh diagnostics"
          title="Refresh diagnostics summary"
          disabled={loading || refreshing}
          onclick={() => void refreshDiagnostics(false)}
        >
          <RefreshCw size={14} aria-hidden="true" />
          Refresh
        </ExtractumButton>
      </div>
    </div>
  </header>

  {#if status}
    <div role="status" aria-live="polite">
      <ExtractumStatusMessage tone="info" className="page-status">{status}</ExtractumStatusMessage>
    </div>
  {/if}

  {#if error}
    <div role="alert" aria-live="assertive" aria-atomic="true">
      <ExtractumStatusMessage tone="error" className="page-status">{error}</ExtractumStatusMessage>
    </div>
  {/if}

  {#if summary}
    {@const tableSections = diagnosticsTableSections(summary)}

    <div class="diagnostics-table-controls extractum-toolbar-row" aria-label="Diagnostics table display">
      <ExtractumButton
        size="sm"
        variant={diagnosticsTableMode === "issues" ? "default" : "outline"}
        aria-label="Show diagnostics issues only"
        title="Show diagnostics issues only"
        aria-pressed={diagnosticsTableMode === "issues"}
        onclick={() => (diagnosticsTableMode = "issues")}
      >
        Only issues
      </ExtractumButton>
      <ExtractumButton
        size="sm"
        variant={diagnosticsTableMode === "all" ? "default" : "outline"}
        aria-label="Show all diagnostics tables"
        title="Show all diagnostics tables"
        aria-pressed={diagnosticsTableMode === "all"}
        onclick={() => (diagnosticsTableMode = "all")}
      >
        All tables
      </ExtractumButton>
    </div>

    {#if diagnosticsTableMode === "issues"}
      {@render diagnosticsTableArea(tableSections)}
      {@render diagnosticsOverviewArea(summary)}
    {:else}
      {@render diagnosticsOverviewArea(summary)}
      {@render diagnosticsTableArea(tableSections)}
    {/if}
  {:else if loading}
    <ExtractumStatusMessage tone="muted" className="page-status">Loading diagnostics...</ExtractumStatusMessage>
  {/if}
</section>

{#snippet diagnosticsOverviewArea(current: DiagnosticSummaryDto)}
  <div class="diagnostics-overview-area">
    <div class="status-strip" aria-label="Diagnostics health overview">
      {#each statusStripItems(current) as item (item.label)}
        <div
          role="group"
          aria-label={`Diagnostics status ${item.label}: ${item.value}. ${item.meta}`}
          class="status-tile extractum-stat-card"
          title={`${item.label}: ${item.value} (${item.meta})`}
        >
          <span>{item.label}</span>
          <strong>{item.value}</strong>
          <ExtractumBadge>{item.meta}</ExtractumBadge>
        </div>
      {/each}
    </div>

    <div class="diagnostics-grid">
      <section class="extractum-panel-shell diagnostics-meta-card">
        <header class="panel-header">
          <div class="panel-header-copy">
            <h2>App and build</h2>
            <p>Factual diagnostic summary metadata</p>
          </div>
        </header>
        <div class="meta-grid">
          <div><span class="muted-copy">App</span><strong>{current.app.appName}</strong></div>
          <div><span class="muted-copy">Version</span><strong>{current.app.appVersion}</strong></div>
          <div><span class="muted-copy">Build</span><strong>{labelFromKey(current.app.buildMode)}</strong></div>
          <div><span class="muted-copy">Generated</span><strong>{formatSummaryGeneratedAt(current.app.generatedAtUnix).replace("Summary generated ", "")}</strong></div>
        </div>
      </section>

      <section class="extractum-panel-shell diagnostics-meta-card">
        <header class="panel-header">
          <div class="panel-header-copy">
            <h2>Database</h2>
            <p>SQLite availability and migration state</p>
          </div>
        </header>
        <div class="meta-grid">
          <div><span class="muted-copy">SQLite</span><strong>{availabilityLabel(current.database.sqliteAvailable)}</strong></div>
          <div><span class="muted-copy">Migrations</span><strong>{labelFromKey(current.database.migrations.status)}</strong></div>
          <div><span class="muted-copy">Accounts</span><strong>{current.database.accountCount}</strong></div>
          <div><span class="muted-copy">Pending versions</span><strong>{current.database.migrations.pendingVersions.length}</strong></div>
          <div><span class="muted-copy">Failed versions</span><strong>{current.database.migrations.failedVersions.length}</strong></div>
        </div>
      </section>

      <section class="extractum-panel-shell diagnostics-meta-card">
        <header class="panel-header">
          <div class="panel-header-copy">
            <h2>Runtimes</h2>
            <p>Backend-reported runtime checks</p>
          </div>
        </header>
        <div class="meta-grid">
          <div><span class="muted-copy">Secure storage</span><strong>{labelFromKey(current.runtimes.secureStorage.status)}</strong></div>
          <div><span class="muted-copy">Secure storage available</span><strong>{availabilityLabel(current.runtimes.secureStorage.available)}</strong></div>
          <div><span class="muted-copy">yt-dlp</span><strong>{labelFromKey(current.runtimes.ytdlp.status)}</strong></div>
          <div><span class="muted-copy">yt-dlp available</span><strong>{availabilityLabel(current.runtimes.ytdlp.available)}</strong></div>
          <div><span class="muted-copy">yt-dlp version</span><strong>{current.runtimes.ytdlp.version ?? "Unknown"}</strong></div>
        </div>
      </section>

      <section class="extractum-panel-shell diagnostics-meta-card">
        <header class="panel-header">
          <div class="panel-header-copy">
            <h2>Privacy boundary</h2>
            <p>Data classes intentionally excluded by backend diagnostics</p>
          </div>
        </header>
        {#if privacyLabels(current).length > 0}
          <div class="privacy-chips">
            {#each privacyLabels(current) as item (item)}
              <ExtractumBadge>{item}</ExtractumBadge>
            {/each}
          </div>
        {:else}
          <ExtractumStatusMessage tone="muted" surface={false}>{privacyNote(current)}</ExtractumStatusMessage>
        {/if}
      </section>
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
      <ExtractumStatusMessage tone="muted" className="diagnostics-empty-state">
        No diagnostic issue rows match this view.
      </ExtractumStatusMessage>
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

  .diagnostics-meta-card {
    display: flex;
    flex-direction: column;
    gap: 0.7rem;
  }

  .diagnostics-grid,
  .diagnostics-tables {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.9rem;
    align-items: start;
  }

  :global(.diagnostics-empty-state.extractum-status-message) {
    grid-column: 1 / -1;
  }

  .meta-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.65rem;
  }

  .meta-grid div {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    min-width: 0;
  }

  .meta-grid strong {
    color: var(--text);
    font-size: 0.9rem;
    min-width: 0;
    overflow-wrap: anywhere;
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
