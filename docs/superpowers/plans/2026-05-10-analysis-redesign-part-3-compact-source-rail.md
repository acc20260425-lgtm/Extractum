# Analysis Result-First Redesign Part 3 Compact Source Rail Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current always-expanded `/analysis` source rail with a compact source context rail and an expanded source switcher layer, without introducing `ReportCanvas` or `RunCompanionTabs` yet.

**Architecture:** Build a new collapsed `CompactSourceRail` component plus a focused `SourceSwitcherPanel` for the full source/group list, search, statuses, and management actions. Wire the new rail into the existing `/analysis` route after Parts 1-2 have implemented `workspaceUiState`, while keeping `WorkspaceMain`, `WorkspaceInspector`, current source readers, and current run history behavior intact.

**Tech Stack:** SvelteKit 2, Svelte 5 runes, TypeScript, Vitest raw-source tests, existing lucide Svelte icons, existing Extractum UI components, Part 1 `analysis-workspace-state`, Part 2 workspace persistence wiring.

---

## Prerequisites

Implement this part only after Part 1 and Part 2 are implemented and committed, not merely planned.

This plan assumes these files already exist from earlier parts:

- `src/lib/analysis-workspace-state.ts`
- `src/lib/analysis-workspace-persistence.ts`
- `src/lib/analysis-route-workspace-state.test.ts`

This plan also assumes `src/routes/analysis/+page.svelte` already owns:

- `workspaceUiState`
- `selectSource(...)` wired through `selectSourceWorkspace(...)`
- `selectGroup(...)` wired through `selectSourceGroupWorkspace(...)`
- `alignWorkspaceToOpenedRun(...)`
- persistence that does not save open popovers/drawers

If any prerequisite is missing, stop and implement the earlier part first.

This is **Part 3 of 7**. Stop after this part is implemented, verified, and committed. Continue to Part 4 only after explicit user approval.

## Part Boundary

Part 3 may:

- create `CompactSourceRail`;
- create `SourceSwitcherPanel`;
- keep `workspace-rail.svelte` as a legacy component for now;
- replace the `/analysis` route usage of `WorkspaceRail` with `CompactSourceRail`;
- reduce the left analysis column to a compact rail width;
- keep source/group selection callbacks flowing through Part 1/2 route state helpers;
- move detailed source-list actions out of the collapsed rail into the expanded switcher layer;
- expose compact source status in the rail and detailed source status in the expanded panel;
- add focused raw-source tests for the rail contract and source access placement.

Part 3 must not:

- create or wire `ReportCanvas`;
- create or wire `RunCompanionTabs`;
- move source readers out of `WorkspaceMain`;
- redesign `WorkspaceMain` or `WorkspaceInspector`;
- add the central `Report | Source` mode switch;
- persist the expanded source panel open state;
- put source ingest jobs into saved/active analysis runs;
- change saved-run immutability or snapshot behavior;
- make completed-run evidence/chat fall back to live source data.

## File Structure

- Create: `src/lib/components/analysis/source-switcher-panel.svelte`
  - Responsibility: full temporary source/group list with search, detailed statuses, provider availability details, management actions, sync/retry/takeout actions, and keyboard-accessible selection.
- Create: `src/lib/components/analysis/compact-source-rail.svelte`
  - Responsibility: narrow collapsed analysis rail with current context, source/group quick switching, selected state, compact warning/running indicators, and one contextual primary action slot.
- Keep: `src/lib/components/analysis/workspace-rail.svelte`
  - Responsibility: legacy wide rail kept for reference and rollback during this staged migration.
- Modify: `src/routes/analysis/+page.svelte`
  - Responsibility: import and render `CompactSourceRail`, pass `workspaceUiState.workspaceSelection`, keep existing selection/sync/source-management callbacks, and update the layout grid width.
- Create: `src/lib/analysis-compact-source-rail.test.ts`
  - Responsibility: raw-source coverage for collapsed rail quietness, expanded-panel capability, callback wiring, accessibility, and no global nav duplication.
- Create: `src/lib/analysis-source-access-placement.test.ts`
  - Responsibility: raw-source coverage that `/analysis` uses the compact rail and that source ingest jobs stay out of analysis run history surfaces.

## Task 1: Add Compact Rail Contract Tests

**Files:**
- Create: `src/lib/analysis-compact-source-rail.test.ts`
- Create: `src/lib/analysis-source-access-placement.test.ts`

- [ ] **Step 1: Write failing raw-source tests for the rail contract**

Create `src/lib/analysis-compact-source-rail.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import compactRailSource from "./components/analysis/compact-source-rail.svelte?raw";
import sourceSwitcherPanelSource from "./components/analysis/source-switcher-panel.svelte?raw";

describe("compact analysis source rail", () => {
  it("keeps the collapsed rail compact and source-scoped", () => {
    expect(compactRailSource).toContain('class="compact-source-rail"');
    expect(compactRailSource).toContain("workspaceSelection: WorkspaceSelection");
    expect(compactRailSource).toContain("sourceSwitcherOpen");
    expect(compactRailSource).toContain('ariaLabel="Open source switcher"');
    expect(compactRailSource).toContain('title="Open source switcher"');
    expect(compactRailSource).toContain("context-primary-action");
    expect(compactRailSource).toContain("criticalSourceStatus");
    expect(compactRailSource).toContain("selected={isSelectedSource(source.id)}");
    expect(compactRailSource).toContain("selected={isSelectedGroup(group.id)}");
    expect(compactRailSource).not.toContain("<h1>Workspace</h1>");
    expect(compactRailSource).not.toContain("Research context");
    expect(compactRailSource).not.toContain("Manage sources");
    expect(compactRailSource).not.toContain("Transcript unavailable");
    expect(compactRailSource).not.toContain("Comments unavailable");
  });

  it("puts full list, search, management, and detailed status in the expanded source panel", () => {
    expect(sourceSwitcherPanelSource).toContain('class="source-switcher-panel"');
    expect(sourceSwitcherPanelSource).toContain('aria-label="Source switcher panel"');
    expect(sourceSwitcherPanelSource).toContain("Search sources or groups");
    expect(sourceSwitcherPanelSource).toContain("Manage sources");
    expect(sourceSwitcherPanelSource).toContain("New source");
    expect(sourceSwitcherPanelSource).toContain("filteredSourceCatalog");
    expect(sourceSwitcherPanelSource).toContain("filteredGroups");
    expect(sourceSwitcherPanelSource).toContain("youtubeSummary.captions.label");
    expect(sourceSwitcherPanelSource).toContain("youtubeSummary.comments.label");
    expect(sourceSwitcherPanelSource).toContain("takeout-status");
    expect(sourceSwitcherPanelSource).toContain("sourceJobsBySource");
    expect(sourceSwitcherPanelSource).toContain("onSyncSource(source.id)");
    expect(sourceSwitcherPanelSource).toContain("onStartTakeoutImport(source.id)");
  });

  it("keeps source and group switching callback-based", () => {
    expect(compactRailSource).toContain("onclick={() => onSelectSource(source.id)}");
    expect(compactRailSource).toContain("onclick={() => onSelectGroup(group.id)}");
    expect(sourceSwitcherPanelSource).toContain("onclick={() => onSelectSource(source.id)}");
    expect(sourceSwitcherPanelSource).toContain("onclick={() => onSelectGroup(group.id)}");
  });

  it("keeps icon-only controls accessible without hover-only status", () => {
    expect(compactRailSource).toContain("ariaLabel={sourceButtonLabel(source)}");
    expect(compactRailSource).toContain("title={sourceButtonLabel(source)}");
    expect(compactRailSource).toContain("ariaLabel={groupButtonLabel(group)}");
    expect(compactRailSource).toContain("title={groupButtonLabel(group)}");
    expect(compactRailSource).toContain("title={criticalStatusLabel}");
    expect(sourceSwitcherPanelSource).toContain("aria-pressed={isSelectedSource(source.id)}");
    expect(sourceSwitcherPanelSource).toContain("aria-pressed={isSelectedGroup(group.id)}");
  });
});
```

- [ ] **Step 2: Write failing source access placement tests**

Create `src/lib/analysis-source-access-placement.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";
import activeRunListSource from "./components/analysis/active-run-list.svelte?raw";
import runHistorySource from "./components/analysis/run-history.svelte?raw";
import workspaceInspectorSource from "./components/analysis/workspace-inspector.svelte?raw";

describe("analysis source access placement", () => {
  it("uses the compact source rail inside the analysis route", () => {
    expect(analysisPageSource).toContain(
      'import CompactSourceRail from "$lib/components/analysis/compact-source-rail.svelte";',
    );
    expect(analysisPageSource).not.toContain(
      'import WorkspaceRail from "$lib/components/analysis/workspace-rail.svelte";',
    );
    expect(analysisPageSource).toContain("<CompactSourceRail");
    expect(analysisPageSource).toContain("workspaceSelection={workspaceUiState.workspaceSelection}");
    expect(analysisPageSource).toContain("onSelectSource={(sourceId) => void selectSource(sourceId)}");
    expect(analysisPageSource).toContain("onSelectGroup={selectGroup}");
    expect(analysisPageSource).toContain("sourceJobsBySource");
  });

  it("does not place source ingest jobs in the analysis runs surfaces", () => {
    expect(workspaceInspectorSource).not.toContain("SourceJobRecord");
    expect(workspaceInspectorSource).not.toContain("sourceJobs");
    expect(workspaceInspectorSource).not.toContain("takeoutJobsBySource");
    expect(runHistorySource).not.toContain("SourceJobRecord");
    expect(runHistorySource).not.toContain("sourceJobs");
    expect(runHistorySource).not.toContain("Takeout");
    expect(activeRunListSource).not.toContain("SourceJobRecord");
    expect(activeRunListSource).not.toContain("sourceJobs");
    expect(activeRunListSource).not.toContain("Takeout");
  });

  it("keeps the left analysis column compact without changing the center and inspector components", () => {
    expect(analysisPageSource).toContain(
      "grid-template-columns: minmax(4.25rem, 4.75rem) minmax(0, 1.6fr) minmax(320px, 430px);",
    );
    expect(analysisPageSource).toContain("<WorkspaceMain");
    expect(analysisPageSource).toContain("<WorkspaceInspector");
    expect(analysisPageSource).not.toContain("<ReportCanvas");
    expect(analysisPageSource).not.toContain("<RunCompanionTabs");
  });
});
```

- [ ] **Step 3: Run the focused tests and verify they fail**

Run:

```powershell
npm.cmd test -- src/lib/analysis-compact-source-rail.test.ts src/lib/analysis-source-access-placement.test.ts
```

Expected: FAIL because `compact-source-rail.svelte` and `source-switcher-panel.svelte` do not exist and `/analysis/+page.svelte` still imports `WorkspaceRail`.

- [ ] **Step 4: Commit the failing tests**

Run:

```powershell
git add src/lib/analysis-compact-source-rail.test.ts src/lib/analysis-source-access-placement.test.ts
git commit -m "test: define compact analysis source rail contract"
```

## Task 2: Add Expanded Source Switcher Panel

**Files:**
- Create: `src/lib/components/analysis/source-switcher-panel.svelte`

- [ ] **Step 1: Create the expanded source panel component**

Create `src/lib/components/analysis/source-switcher-panel.svelte`:

```svelte
<script lang="ts">
  import { Archive, ExternalLink, Plus, RefreshCw, Search, Square, Trash2 } from "@lucide/svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import { membershipLabel, sourceCapabilities, sourceKindLabel } from "$lib/source-capabilities";
  import type { AccountRuntimeStatus } from "$lib/types/accounts";
  import type { AnalysisSourceGroup, AnalysisSourceOption } from "$lib/types/analysis";
  import type { Source, SourceJobRecord, TakeoutImportJobRecord } from "$lib/types/sources";
  import type { YoutubeRuntimeStatus, YoutubeSourceSummary } from "$lib/types/youtube";
  import type { WorkspaceSelection } from "$lib/analysis-workspace-state";
  import type { BadgeVariant } from "$lib/components/ui/types";

  let {
    sourceCatalog,
    groups,
    sourceMetrics,
    loadingSourceCatalog,
    loadingGroups,
    railQuery,
    filteredSourceCatalog,
    filteredGroups,
    workspaceSelection,
    syncingIds,
    deletingSourceIds,
    startingTakeoutSourceIds,
    takeoutJobsBySource,
    sourceJobsBySource,
    youtubeSummaries,
    youtubeRuntimeStatus,
    formatTimestamp,
    accountLabel,
    sourceInitial,
    runtimeStatus,
    runtimeBadge,
    sourceSyncDisabledReason,
    onChangeRailQuery,
    onSelectSource,
    onSelectGroup,
    onSyncSource,
    onStartTakeoutImport,
    onCancelTakeoutImport,
    onCancelSourceJob,
    onOpenSourceManager,
    onDeleteSource,
    onClose,
  }: {
    sourceCatalog: Source[];
    groups: AnalysisSourceGroup[];
    sourceMetrics: Record<number, AnalysisSourceOption>;
    loadingSourceCatalog: boolean;
    loadingGroups: boolean;
    railQuery: string;
    filteredSourceCatalog: Source[];
    filteredGroups: AnalysisSourceGroup[];
    workspaceSelection: WorkspaceSelection;
    syncingIds: Record<number, boolean>;
    deletingSourceIds: Record<number, boolean>;
    startingTakeoutSourceIds: Record<number, boolean>;
    takeoutJobsBySource: Record<number, TakeoutImportJobRecord>;
    sourceJobsBySource: Record<number, SourceJobRecord[]>;
    youtubeSummaries: Record<number, YoutubeSourceSummary>;
    youtubeRuntimeStatus: YoutubeRuntimeStatus | null;
    formatTimestamp: (value: number | null) => string;
    accountLabel: (accountId: number | null) => string;
    sourceInitial: (source: Source) => string;
    runtimeStatus: (accountId: number | null) => AccountRuntimeStatus | null;
    runtimeBadge: (runtime: AccountRuntimeStatus | null) => string;
    sourceSyncDisabledReason: (source: Source) => string | null;
    onChangeRailQuery: (value: string) => void;
    onSelectSource: (sourceId: number) => void;
    onSelectGroup: (groupId: number) => void;
    onSyncSource: (sourceId: number) => void;
    onStartTakeoutImport: (sourceId: number) => void;
    onCancelTakeoutImport: (jobId: string) => void;
    onCancelSourceJob: (jobId: string) => void;
    onOpenSourceManager: () => void;
    onDeleteSource: (source: Source) => void;
    onClose: () => void;
  } = $props();

  function isSelectedSource(sourceId: number) {
    return workspaceSelection.kind === "source" && workspaceSelection.sourceId === sourceId;
  }

  function isSelectedGroup(groupId: number) {
    return workspaceSelection.kind === "source_group" && workspaceSelection.sourceGroupId === groupId;
  }

  function isActiveTakeoutJob(job: TakeoutImportJobRecord | undefined) {
    return (
      job?.status === "queued" ||
      job?.status === "running" ||
      job?.status === "cancel_requested"
    );
  }

  function isActiveSourceJob(job: SourceJobRecord) {
    return job.status === "queued" || job.status === "running" || job.status === "cancel_requested";
  }

  function takeoutPhaseName(phase: TakeoutImportJobRecord["phase"]) {
    return String(phase).replaceAll("_", " ");
  }

  function takeoutPhaseLabel(job: TakeoutImportJobRecord) {
    if (job.status === "failed") return "Takeout failed";
    if (job.status === "completed") return "Takeout complete";
    if (job.status === "cancelled") return "Takeout cancelled";
    if (job.status === "cancel_requested") return "Cancelling Takeout";
    return `Takeout ${takeoutPhaseName(job.phase)}`;
  }

  function takeoutBadgeVariant(job: TakeoutImportJobRecord): BadgeVariant {
    if (job.status === "completed") return "success";
    if (job.status === "failed") return "danger";
    if (job.status === "cancelled") return "neutral";
    if (job.warnings.length > 0) return "warning";
    return "info";
  }

  function availabilityLabel(value: string | null) {
    return value ? value.replaceAll("_", " ") : null;
  }

  function youtubeMetaLine(summary: YoutubeSourceSummary | null) {
    if (!summary) return null;
    return [summary.channelHandle ?? summary.channelTitle, summary.videoCount !== null ? `${summary.videoCount} videos` : null]
      .filter(Boolean)
      .join(" - ") || null;
  }
</script>

<section class="source-switcher-panel" aria-label="Source switcher panel">
  <div class="panel-head">
    <div>
      <span class="eyebrow">Sources</span>
      <h2>Switch source context</h2>
    </div>
    <div class="panel-actions">
      <Button size="sm" variant="secondary" onclick={onOpenSourceManager}>
        <Plus size={14} aria-hidden="true" /> New source
      </Button>
      <Button size="sm" variant="ghost" onclick={onOpenSourceManager}>Manage sources</Button>
      <Button size="sm" variant="ghost" onclick={onClose}>Close</Button>
    </div>
  </div>

  <label class="search-field">
    <span>Search sources or groups</span>
    <div class="search-shell">
      <Search size={15} aria-hidden="true" />
      <Input
        type="search"
        value={railQuery}
        placeholder="Search sources or groups"
        ariaLabel="Search sources or groups"
        oninput={(event) => onChangeRailQuery((event.currentTarget as HTMLInputElement).value)}
      />
    </div>
  </label>

  <div class="panel-section">
    <div class="section-title">
      <span>Sources</span>
      <Badge>{loadingSourceCatalog ? "loading" : `${filteredSourceCatalog.length}/${sourceCatalog.length}`}</Badge>
    </div>

    {#if loadingSourceCatalog}
      <div class="panel-empty">Loading sources...</div>
    {:else if filteredSourceCatalog.length === 0}
      <div class="panel-empty">No sources match the current search.</div>
    {:else}
      <div class="source-list">
        {#each filteredSourceCatalog as source (source.id)}
          {@const metrics = sourceMetrics[source.id]}
          {@const capabilities = sourceCapabilities(source)}
          {@const kindLabel = sourceKindLabel(source)}
          {@const sourceMembershipLabel = membershipLabel(source)}
          {@const runtime = runtimeStatus(source.accountId)}
          {@const runtimeStateBadge = runtimeBadge(runtime)}
          {@const syncReason = sourceSyncDisabledReason(source)}
          {@const youtubeSummary = source.sourceType === "youtube" ? youtubeSummaries[source.id] ?? null : null}
          {@const sourceJobs = sourceJobsBySource[source.id] ?? []}
          {@const takeoutJob = takeoutJobsBySource[source.id]}
          {@const takeoutActive = isActiveTakeoutJob(takeoutJob)}
          {@const deleting = !!deletingSourceIds[source.id]}
          <article class:selected={isSelectedSource(source.id)} class="source-row">
            <button
              class="source-main"
              type="button"
              aria-pressed={isSelectedSource(source.id)}
              onclick={() => onSelectSource(source.id)}
            >
              <div class="source-avatar" aria-hidden="true">
                {#if youtubeSummary?.thumbnailUrl ?? source.avatarDataUrl}
                  <img src={youtubeSummary?.thumbnailUrl ?? source.avatarDataUrl ?? ""} alt="" loading="lazy" />
                {:else}
                  <span>{sourceInitial(source)}</span>
                {/if}
              </div>
              <div class="source-copy">
                <strong>{youtubeSummary?.title ?? source.title ?? source.externalId}</strong>
                <div class="source-meta">
                  <span>{kindLabel}</span>
                  <span>{youtubeMetaLine(youtubeSummary) ?? accountLabel(source.accountId)}</span>
                  {#if metrics}
                    <span>{metrics.item_count} {capabilities.contentLabel}</span>
                  {/if}
                </div>
              </div>
            </button>

            <div class="detail-badges">
              {#if runtimeStateBadge}
                <Badge variant="warning" title={runtime?.message ?? undefined}>{runtimeStateBadge}</Badge>
              {/if}
              {#if capabilities.hasMembershipState && sourceMembershipLabel}
                <Badge>{sourceMembershipLabel}</Badge>
              {/if}
              {#if source.sourceType === "youtube" && youtubeRuntimeStatus && !youtubeRuntimeStatus.ytdlpAvailable}
                <Badge variant="warning" title={youtubeRuntimeStatus.message}>yt-dlp unavailable</Badge>
              {/if}
              {#if youtubeSummary}
                <Badge variant={youtubeSummary.captions.state === "synced" ? "success" : youtubeSummary.captions.state === "unavailable" ? "warning" : "neutral"}>
                  {youtubeSummary.captions.label}
                </Badge>
                <Badge variant={youtubeSummary.comments.state === "synced" ? "success" : "neutral"}>
                  {youtubeSummary.comments.label}
                </Badge>
                {#if availabilityLabel(youtubeSummary.availabilityStatus)}
                  <Badge variant={youtubeSummary.availabilityStatus === "available" ? "neutral" : "warning"}>
                    {availabilityLabel(youtubeSummary.availabilityStatus)}
                  </Badge>
                {/if}
                {#if youtubeSummary.canonicalUrl}
                  <a class="panel-link" href={youtubeSummary.canonicalUrl} target="_blank" rel="noreferrer">
                    <ExternalLink size={13} aria-hidden="true" /> YouTube
                  </a>
                {/if}
              {/if}
              {#if takeoutJob}
                <Badge variant={takeoutBadgeVariant(takeoutJob)} title={takeoutJob.error ?? takeoutJob.message ?? undefined}>
                  {takeoutPhaseLabel(takeoutJob)}
                </Badge>
              {/if}
            </div>

            <div class="row-actions">
              {#if capabilities.canSync}
                <Button
                  size="sm"
                  variant="secondary"
                  onclick={() => onSyncSource(source.id)}
                  disabled={!!syncingIds[source.id] || deleting || takeoutActive || syncReason !== null}
                  title={takeoutActive ? "Takeout import is active." : syncReason ?? undefined}
                >
                  <RefreshCw size={13} aria-hidden="true" />
                  {syncingIds[source.id] ? "Syncing..." : "Sync"}
                </Button>
              {/if}
              {#if capabilities.canImportArchive}
                {#if takeoutActive && takeoutJob}
                  <Button
                    size="sm"
                    variant="secondary"
                    onclick={() => onCancelTakeoutImport(takeoutJob.job_id)}
                    disabled={takeoutJob.status === "cancel_requested"}
                  >
                    <Square size={13} aria-hidden="true" />
                    {takeoutJob.status === "cancel_requested" ? "Cancelling..." : "Cancel"}
                  </Button>
                {:else}
                  <Button
                    size="sm"
                    variant="secondary"
                    onclick={() => onStartTakeoutImport(source.id)}
                    disabled={!!startingTakeoutSourceIds[source.id] || deleting || !!syncingIds[source.id] || syncReason !== null}
                    title={syncReason ?? undefined}
                  >
                    <Archive size={13} aria-hidden="true" />
                    {startingTakeoutSourceIds[source.id] ? "Starting..." : "Takeout"}
                  </Button>
                {/if}
              {/if}
              <Button
                size="sm"
                variant="danger-soft"
                onclick={() => onDeleteSource(source)}
                disabled={deleting || !!syncingIds[source.id] || takeoutActive}
                title={takeoutActive ? "Takeout import is active." : undefined}
              >
                <Trash2 size={13} aria-hidden="true" />
                {deleting ? "Deleting..." : "Delete"}
              </Button>
            </div>

            {#if takeoutJob}
              <div class="takeout-status">
                <span>{takeoutPhaseName(takeoutJob.phase)}</span>
                <span>{takeoutJob.message ?? takeoutJob.error ?? `${takeoutJob.inserted} inserted, ${takeoutJob.skipped} skipped`}</span>
              </div>
            {/if}

            {#if sourceJobs.length > 0}
              <div class="source-job-list">
                {#each sourceJobs.slice(0, 3) as job (job.job_id)}
                  <div class="source-job-row">
                    <span>{job.job_type.replaceAll("_", " ")}</span>
                    <Badge variant={job.status === "failed" ? "danger" : job.status === "succeeded" ? "success" : "info"}>
                      {job.status.replaceAll("_", " ")}
                    </Badge>
                    {#if isActiveSourceJob(job)}
                      <Button size="sm" variant="ghost" onclick={() => onCancelSourceJob(job.job_id)} disabled={job.status === "cancel_requested"}>
                        <Square size={13} aria-hidden="true" /> Cancel
                      </Button>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}
          </article>
        {/each}
      </div>
    {/if}
  </div>

  <div class="panel-section">
    <div class="section-title">
      <span>Groups</span>
      <Badge>{loadingGroups ? "loading" : `${filteredGroups.length}/${groups.length}`}</Badge>
    </div>

    {#if loadingGroups}
      <div class="panel-empty">Loading groups...</div>
    {:else if filteredGroups.length === 0}
      <div class="panel-empty">No groups match the current search.</div>
    {:else}
      <div class="group-list">
        {#each filteredGroups as group (group.id)}
          <button
            class:selected={isSelectedGroup(group.id)}
            class="group-row"
            type="button"
            aria-pressed={isSelectedGroup(group.id)}
            onclick={() => onSelectGroup(group.id)}
          >
            <span class="group-avatar" aria-hidden="true">{group.name.trim().charAt(0).toUpperCase() || "G"}</span>
            <span class="group-copy">
              <strong>{group.name}</strong>
              <small>{group.members.length} sources - updated {formatTimestamp(group.updated_at)}</small>
            </span>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</section>

<style>
  .source-switcher-panel {
    position: absolute;
    z-index: 20;
    left: calc(100% + 0.55rem);
    top: 0;
    width: min(31rem, calc(100vw - 7rem));
    max-height: calc(100vh - 6rem);
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    padding: 0.85rem;
    overflow: auto;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
    box-shadow: var(--shadow);
  }

  .panel-head,
  .section-title,
  .source-main,
  .row-actions,
  .detail-badges,
  .source-meta,
  .panel-actions,
  .source-job-row {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .panel-head,
  .section-title {
    justify-content: space-between;
  }

  .eyebrow,
  .search-field span,
  .section-title span {
    color: var(--muted);
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  h2 {
    margin: 0.15rem 0 0;
    font-size: 1rem;
  }

  .search-field {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .search-shell {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 0.4rem;
    align-items: center;
  }

  .panel-section,
  .source-list,
  .group-list,
  .source-job-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .source-row {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.65rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel-strong);
  }

  .source-row.selected,
  .group-row.selected {
    border-color: var(--primary);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--primary) 12%, transparent);
  }

  .source-main,
  .group-row {
    width: 100%;
    min-width: 0;
    border: 0;
    background: transparent;
    color: inherit;
    text-align: left;
    cursor: pointer;
  }

  .source-avatar,
  .group-avatar {
    width: 2.25rem;
    height: 2.25rem;
    flex: 0 0 2.25rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    border-radius: 8px;
    background: color-mix(in srgb, var(--primary) 12%, var(--panel));
    color: var(--primary);
    font-weight: 700;
  }

  .source-avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .source-copy,
  .group-copy {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .source-copy strong,
  .group-copy strong {
    overflow-wrap: anywhere;
  }

  .source-meta,
  .group-copy small,
  .takeout-status,
  .source-job-row {
    color: var(--muted);
    font-size: 0.78rem;
  }

  .panel-link {
    display: inline-flex;
    gap: 0.25rem;
    align-items: center;
    color: var(--text);
    text-decoration: none;
    font-size: 0.78rem;
  }

  .takeout-status,
  .source-job-row,
  .panel-empty {
    padding: 0.55rem;
    border: 1px solid color-mix(in srgb, var(--border) 78%, transparent);
    border-radius: 8px;
    background: color-mix(in srgb, var(--panel-strong) 70%, transparent);
  }

  .group-row {
    display: flex;
    gap: 0.6rem;
    align-items: center;
    padding: 0.65rem;
    border: 1px solid var(--border);
    border-radius: 8px;
  }

  @media (max-width: 1180px) {
    .source-switcher-panel {
      position: fixed;
      left: 0.75rem;
      right: 0.75rem;
      top: 4.5rem;
      width: auto;
      max-height: calc(100vh - 5.5rem);
    }
  }
</style>
```

- [ ] **Step 2: Run Svelte check for the new panel**

Run:

```powershell
npm.cmd run check
```

Expected: PASS. If it fails, fix the Svelte or TypeScript errors in `source-switcher-panel.svelte` and rerun the same command before continuing.

- [ ] **Step 3: Commit the expanded panel**

Run:

```powershell
git add src/lib/components/analysis/source-switcher-panel.svelte
git commit -m "feat: add analysis source switcher panel"
```

## Task 3: Add Collapsed Compact Source Rail

**Files:**
- Create: `src/lib/components/analysis/compact-source-rail.svelte`

- [ ] **Step 1: Create the compact rail component**

Create `src/lib/components/analysis/compact-source-rail.svelte`:

```svelte
<script lang="ts">
  import { AlertTriangle, Folder, Loader2, Plus, RefreshCw, Search, Send, Video } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import SourceSwitcherPanel from "$lib/components/analysis/source-switcher-panel.svelte";
  import { sourceCapabilities } from "$lib/source-capabilities";
  import type { AccountRuntimeStatus } from "$lib/types/accounts";
  import type { AnalysisSourceGroup, AnalysisSourceOption } from "$lib/types/analysis";
  import type { Source, SourceJobRecord, TakeoutImportJobRecord } from "$lib/types/sources";
  import type { YoutubeRuntimeStatus, YoutubeSourceSummary } from "$lib/types/youtube";
  import type { WorkspaceSelection } from "$lib/analysis-workspace-state";

  let {
    sourceCatalog,
    groups,
    sourceMetrics,
    loadingSourceCatalog,
    loadingGroups,
    railQuery,
    filteredSourceCatalog,
    filteredGroups,
    workspaceSelection,
    syncingIds,
    deletingSourceIds,
    startingTakeoutSourceIds,
    takeoutJobsBySource,
    sourceJobsBySource,
    youtubeSummaries,
    youtubeRuntimeStatus,
    formatTimestamp,
    accountLabel,
    sourceInitial,
    runtimeStatus,
    runtimeBadge,
    sourceSyncDisabledReason,
    onChangeRailQuery,
    onSelectSource,
    onSelectGroup,
    onSyncSource,
    onStartTakeoutImport,
    onCancelTakeoutImport,
    onCancelSourceJob,
    onOpenSourceManager,
    onDeleteSource,
  }: {
    sourceCatalog: Source[];
    groups: AnalysisSourceGroup[];
    sourceMetrics: Record<number, AnalysisSourceOption>;
    loadingSourceCatalog: boolean;
    loadingGroups: boolean;
    railQuery: string;
    filteredSourceCatalog: Source[];
    filteredGroups: AnalysisSourceGroup[];
    workspaceSelection: WorkspaceSelection;
    syncingIds: Record<number, boolean>;
    deletingSourceIds: Record<number, boolean>;
    startingTakeoutSourceIds: Record<number, boolean>;
    takeoutJobsBySource: Record<number, TakeoutImportJobRecord>;
    sourceJobsBySource: Record<number, SourceJobRecord[]>;
    youtubeSummaries: Record<number, YoutubeSourceSummary>;
    youtubeRuntimeStatus: YoutubeRuntimeStatus | null;
    formatTimestamp: (value: number | null) => string;
    accountLabel: (accountId: number | null) => string;
    sourceInitial: (source: Source) => string;
    runtimeStatus: (accountId: number | null) => AccountRuntimeStatus | null;
    runtimeBadge: (runtime: AccountRuntimeStatus | null) => string;
    sourceSyncDisabledReason: (source: Source) => string | null;
    onChangeRailQuery: (value: string) => void;
    onSelectSource: (sourceId: number) => void;
    onSelectGroup: (groupId: number) => void;
    onSyncSource: (sourceId: number) => void;
    onStartTakeoutImport: (sourceId: number) => void;
    onCancelTakeoutImport: (jobId: string) => void;
    onCancelSourceJob: (jobId: string) => void;
    onOpenSourceManager: () => void;
    onDeleteSource: (source: Source) => void;
  } = $props();

  let sourceSwitcherOpen = $state(false);

  const currentSource = $derived.by(() =>
    workspaceSelection.kind === "source"
      ? sourceCatalog.find((source) => source.id === workspaceSelection.sourceId) ?? null
      : null,
  );
  const currentGroup = $derived.by(() =>
    workspaceSelection.kind === "source_group"
      ? groups.find((group) => group.id === workspaceSelection.sourceGroupId) ?? null
      : null,
  );
  const visibleSources = $derived(filteredSourceCatalog.slice(0, 8));
  const visibleGroups = $derived(filteredGroups.slice(0, 4));
  const criticalStatusLabel = $derived(criticalSourceStatus());

  function isSelectedSource(sourceId: number) {
    return workspaceSelection.kind === "source" && workspaceSelection.sourceId === sourceId;
  }

  function isSelectedGroup(groupId: number) {
    return workspaceSelection.kind === "source_group" && workspaceSelection.sourceGroupId === groupId;
  }

  function sourceButtonLabel(source: Source) {
    const name = youtubeSummaries[source.id]?.title ?? source.title ?? source.externalId;
    const status = compactSourceStatus(source);
    return status ? `${name}. ${status}` : name;
  }

  function groupButtonLabel(group: AnalysisSourceGroup) {
    return `${group.name}. ${group.members.length} sources`;
  }

  function compactSourceStatus(source: Source) {
    const runtime = runtimeBadge(runtimeStatus(source.accountId));
    if (runtime) return runtime;
    const syncReason = sourceSyncDisabledReason(source);
    if (syncReason) return syncReason;
    if (syncingIds[source.id]) return "Syncing";
    const activeJob = (sourceJobsBySource[source.id] ?? []).find(isActiveSourceJob);
    if (activeJob) return activeJob.status.replaceAll("_", " ");
    const takeoutJob = takeoutJobsBySource[source.id];
    if (takeoutJob && isActiveTakeoutJob(takeoutJob)) return takeoutJob.phase.replaceAll("_", " ");
    return "";
  }

  function criticalSourceStatus() {
    const source = currentSource;
    if (!source) return "";
    return compactSourceStatus(source);
  }

  function isActiveSourceJob(job: SourceJobRecord) {
    return job.status === "queued" || job.status === "running" || job.status === "cancel_requested";
  }

  function isActiveTakeoutJob(job: TakeoutImportJobRecord | undefined) {
    return (
      job?.status === "queued" ||
      job?.status === "running" ||
      job?.status === "cancel_requested"
    );
  }

  function canUsePrimarySync(source: Source | null) {
    if (!source) return false;
    return sourceCapabilities(source).canSync && sourceSyncDisabledReason(source) === null;
  }

  function providerMark(source: Source) {
    if (source.sourceType === "telegram") return Send;
    if (source.sourceType === "youtube") return Video;
    return Folder;
  }
</script>

<aside class="compact-source-rail">
  <div class="rail-top">
    <Button
      iconOnly
      variant="secondary"
      ariaLabel="Open source switcher"
      title="Open source switcher"
      ariaExpanded={sourceSwitcherOpen}
      onclick={() => (sourceSwitcherOpen = !sourceSwitcherOpen)}
    >
      <Search size={16} aria-hidden="true" />
    </Button>

    <button
      class:active={workspaceSelection.kind !== "none"}
      class="current-context-button"
      type="button"
      title={currentSource ? sourceButtonLabel(currentSource) : currentGroup ? groupButtonLabel(currentGroup) : "No source selected"}
      aria-label={currentSource ? sourceButtonLabel(currentSource) : currentGroup ? groupButtonLabel(currentGroup) : "No source selected"}
      onclick={() => (sourceSwitcherOpen = true)}
    >
      {#if currentSource}
        {@const Mark = providerMark(currentSource)}
        <span class="context-avatar">
          {#if youtubeSummaries[currentSource.id]?.thumbnailUrl ?? currentSource.avatarDataUrl}
            <img src={youtubeSummaries[currentSource.id]?.thumbnailUrl ?? currentSource.avatarDataUrl ?? ""} alt="" loading="lazy" />
          {:else}
            {sourceInitial(currentSource)}
          {/if}
        </span>
        <Mark size={13} aria-hidden="true" />
      {:else if currentGroup}
        <span class="context-avatar group">{currentGroup.name.trim().charAt(0).toUpperCase() || "G"}</span>
        <Folder size={13} aria-hidden="true" />
      {:else}
        <span class="context-avatar empty">-</span>
      {/if}
    </button>

    {#if criticalStatusLabel}
      <span class="status-dot" title={criticalStatusLabel} aria-label={criticalStatusLabel}>
        {#if criticalStatusLabel.toLocaleLowerCase().includes("sync") || criticalStatusLabel.toLocaleLowerCase().includes("running")}
          <Loader2 size={13} aria-hidden="true" />
        {:else}
          <AlertTriangle size={13} aria-hidden="true" />
        {/if}
      </span>
    {/if}
  </div>

  <div class="quick-list" aria-label="Quick source choices">
    {#each visibleSources as source (source.id)}
      {@const Mark = providerMark(source)}
      <Button
        iconOnly
        size="sm"
        variant="ghost"
        selected={isSelectedSource(source.id)}
        ariaLabel={sourceButtonLabel(source)}
        title={sourceButtonLabel(source)}
        ariaPressed={isSelectedSource(source.id)}
        onclick={() => onSelectSource(source.id)}
      >
        <span class="mini-avatar">
          {#if youtubeSummaries[source.id]?.thumbnailUrl ?? source.avatarDataUrl}
            <img src={youtubeSummaries[source.id]?.thumbnailUrl ?? source.avatarDataUrl ?? ""} alt="" loading="lazy" />
          {:else}
            {sourceInitial(source)}
          {/if}
        </span>
        <Mark size={10} aria-hidden="true" />
      </Button>
    {/each}

    {#each visibleGroups as group (group.id)}
      <Button
        iconOnly
        size="sm"
        variant="ghost"
        selected={isSelectedGroup(group.id)}
        ariaLabel={groupButtonLabel(group)}
        title={groupButtonLabel(group)}
        ariaPressed={isSelectedGroup(group.id)}
        onclick={() => onSelectGroup(group.id)}
      >
        <span class="mini-avatar group">{group.name.trim().charAt(0).toUpperCase() || "G"}</span>
      </Button>
    {/each}
  </div>

  <div class="context-primary-action">
    {#if sourceCatalog.length === 0 && groups.length === 0}
      <Button iconOnly size="sm" variant="primary" ariaLabel="New source" title="New source" onclick={onOpenSourceManager}>
        <Plus size={14} aria-hidden="true" />
      </Button>
    {:else if canUsePrimarySync(currentSource)}
      <Button
        iconOnly
        size="sm"
        variant="secondary"
        ariaLabel={`Sync ${currentSource?.title ?? currentSource?.externalId ?? "source"}`}
        title={`Sync ${currentSource?.title ?? currentSource?.externalId ?? "source"}`}
        disabled={currentSource ? !!syncingIds[currentSource.id] : true}
        onclick={() => currentSource && onSyncSource(currentSource.id)}
      >
        <RefreshCw size={14} aria-hidden="true" />
      </Button>
    {:else}
      <Button iconOnly size="sm" variant="secondary" ariaLabel="Open source switcher" title="Open source switcher" onclick={() => (sourceSwitcherOpen = true)}>
        <Search size={14} aria-hidden="true" />
      </Button>
    {/if}
  </div>

  {#if sourceSwitcherOpen}
    <SourceSwitcherPanel
      {sourceCatalog}
      {groups}
      {sourceMetrics}
      {loadingSourceCatalog}
      {loadingGroups}
      {railQuery}
      {filteredSourceCatalog}
      {filteredGroups}
      {workspaceSelection}
      {syncingIds}
      {deletingSourceIds}
      {startingTakeoutSourceIds}
      {takeoutJobsBySource}
      {sourceJobsBySource}
      {youtubeSummaries}
      {youtubeRuntimeStatus}
      {formatTimestamp}
      {accountLabel}
      {sourceInitial}
      {runtimeStatus}
      {runtimeBadge}
      {sourceSyncDisabledReason}
      {onChangeRailQuery}
      {onSelectSource}
      {onSelectGroup}
      {onSyncSource}
      {onStartTakeoutImport}
      {onCancelTakeoutImport}
      {onCancelSourceJob}
      {onOpenSourceManager}
      {onDeleteSource}
      onClose={() => (sourceSwitcherOpen = false)}
    />
  {/if}
</aside>

<style>
  .compact-source-rail {
    position: sticky;
    top: 0;
    width: 100%;
    min-width: 0;
    max-height: calc(100vh - 6rem);
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    align-items: center;
    padding: 0.55rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
    box-shadow: var(--shadow);
  }

  .rail-top,
  .quick-list,
  .context-primary-action {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    align-items: center;
  }

  .current-context-button {
    width: 2.75rem;
    min-height: 3.2rem;
    display: inline-flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.2rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel-strong);
    color: var(--text);
    cursor: pointer;
  }

  .current-context-button.active {
    border-color: var(--primary);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--primary) 12%, transparent);
  }

  .context-avatar,
  .mini-avatar {
    width: 1.75rem;
    height: 1.75rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    border-radius: 7px;
    background: color-mix(in srgb, var(--primary) 12%, var(--panel));
    color: var(--primary);
    font-size: 0.75rem;
    font-weight: 700;
  }

  .context-avatar.group,
  .mini-avatar.group {
    background: color-mix(in srgb, var(--accent) 12%, var(--panel));
  }

  .context-avatar.empty {
    color: var(--muted);
  }

  .context-avatar img,
  .mini-avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .quick-list {
    width: 100%;
    padding-top: 0.45rem;
    border-top: 1px solid color-mix(in srgb, var(--border) 76%, transparent);
  }

  .status-dot {
    width: 1.55rem;
    height: 1.55rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 999px;
    color: var(--warning);
    background: color-mix(in srgb, var(--warning) 14%, var(--panel));
  }

  @media (max-width: 1180px) {
    .compact-source-rail {
      position: relative;
      top: auto;
      max-height: none;
      flex-direction: row;
      justify-content: flex-start;
      overflow-x: auto;
    }

    .rail-top,
    .quick-list,
    .context-primary-action {
      flex-direction: row;
    }

    .quick-list {
      width: auto;
      padding-top: 0;
      padding-left: 0.45rem;
      border-top: 0;
      border-left: 1px solid color-mix(in srgb, var(--border) 76%, transparent);
    }
  }
</style>
```

- [ ] **Step 2: Run the compact rail raw-source tests and Svelte check**

Run:

```powershell
npm.cmd test -- src/lib/analysis-compact-source-rail.test.ts
```

Expected: PASS.

Run:

```powershell
npm.cmd run check
```

Expected: PASS.

- [ ] **Step 3: Commit the compact rail component**

Run:

```powershell
git add src/lib/components/analysis/compact-source-rail.svelte
git commit -m "feat: add compact analysis source rail"
```

## Task 4: Wire Compact Rail Into `/analysis`

**Files:**
- Modify: `src/routes/analysis/+page.svelte`

- [ ] **Step 1: Replace the legacy rail import**

In `src/routes/analysis/+page.svelte`, replace:

```ts
  import WorkspaceRail from "$lib/components/analysis/workspace-rail.svelte";
```

with:

```ts
  import CompactSourceRail from "$lib/components/analysis/compact-source-rail.svelte";
```

- [ ] **Step 2: Replace the route rail markup**

Replace the `<WorkspaceRail ... />` block with:

```svelte
  <CompactSourceRail
    {sourceCatalog}
    {groups}
    {sourceMetrics}
    {loadingSourceCatalog}
    {loadingGroups}
    {railQuery}
    {filteredSourceCatalog}
    {filteredGroups}
    workspaceSelection={workspaceUiState.workspaceSelection}
    {syncingIds}
    {deletingSourceIds}
    {startingTakeoutSourceIds}
    {takeoutJobsBySource}
    {sourceJobsBySource}
    {youtubeSummaries}
    {youtubeRuntimeStatus}
    {formatTimestamp}
    {accountLabel}
    {sourceInitial}
    {runtimeStatus}
    {runtimeBadge}
    {sourceSyncDisabledReason}
    onChangeRailQuery={(value) => (railQuery = value)}
    onSelectSource={(sourceId) => void selectSource(sourceId)}
    onSelectGroup={selectGroup}
    onSyncSource={(sourceId) => void syncSelectedSource(sourceId)}
    onStartTakeoutImport={(sourceId) => void startTakeoutImport(sourceId)}
    onCancelTakeoutImport={(jobId) => void cancelTakeoutImport(jobId)}
    onCancelSourceJob={(jobId) => void cancelYoutubeSourceJob(jobId)}
    onOpenSourceManager={() => (sourceManagerOpen = true)}
    onDeleteSource={(source) => void deleteSource(source)}
  />
```

Do not remove `SourceManagementDialog`. The expanded source panel continues to open the existing dialog through `onOpenSourceManager`.

- [ ] **Step 3: Update the desktop layout grid**

In the route `<style>` block, replace:

```css
  .analysis-workspace {
    display: grid;
    grid-template-columns: minmax(260px, 320px) minmax(0, 1.6fr) minmax(320px, 430px);
    gap: 0.9rem;
    align-items: start;
    min-width: 0;
  }
```

with:

```css
  .analysis-workspace {
    display: grid;
    grid-template-columns: minmax(4.25rem, 4.75rem) minmax(0, 1.6fr) minmax(320px, 430px);
    gap: 0.9rem;
    align-items: start;
    min-width: 0;
  }
```

Keep the existing medium and narrow breakpoint structure. In the `@media (max-width: 1500px)` rule, replace:

```css
    .analysis-workspace {
      grid-template-columns: minmax(250px, 300px) minmax(0, 1fr);
    }
```

with:

```css
    .analysis-workspace {
      grid-template-columns: minmax(4.25rem, 4.75rem) minmax(0, 1fr);
    }
```

- [ ] **Step 4: Run route placement tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-source-access-placement.test.ts src/lib/analysis-compact-source-rail.test.ts
```

Expected: PASS.

- [ ] **Step 5: Run route workspace state tests from Part 2**

Run:

```powershell
npm.cmd test -- src/lib/analysis-route-workspace-state.test.ts
```

Expected: PASS. This confirms the compact rail is using the route selection contract instead of reviving legacy rail-owned state.

- [ ] **Step 6: Commit route wiring**

Run:

```powershell
git add src/routes/analysis/+page.svelte
git commit -m "feat: wire compact source rail into analysis"
```

## Task 5: Run Part 3 Verification

**Files:**
- Verify all Part 3 files changed in Tasks 1-4.

- [ ] **Step 1: Run focused frontend tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-compact-source-rail.test.ts src/lib/analysis-source-access-placement.test.ts src/lib/analysis-route-workspace-state.test.ts
```

Expected: PASS.

- [ ] **Step 2: Run the relevant state and route tests from earlier parts**

Run:

```powershell
npm.cmd test -- src/lib/analysis-workspace-state.test.ts src/lib/analysis-workspace-persistence.test.ts src/lib/analysis-run-workflow.test.ts
```

Expected: PASS.

- [ ] **Step 3: Run Svelte and TypeScript checks**

Run:

```powershell
npm.cmd run check
```

Expected: PASS.

- [ ] **Step 4: Run the full frontend test suite**

Run:

```powershell
npm.cmd test
```

Expected: PASS.

- [ ] **Step 5: Check whitespace**

Run:

```powershell
git diff --check
```

Expected: no output and exit code 0.

- [ ] **Step 6: Commit any final fixes**

If verification required fixes, commit them:

```powershell
git add src
git commit -m "test: verify compact analysis source rail"
```

Skip this commit if there are no additional changes after Tasks 1-4.

- [ ] **Step 7: Stop for review**

Run:

```powershell
git status --short
```

Expected: clean working tree.

Report:

```text
Part 3 compact source rail is implemented and verified. Stopping before Part 4.
```

Do not begin Part 4 until the user explicitly approves continuing.

## Self-Review

- Spec coverage: this plan covers the approved `CompactSourceRail` responsibilities, the collapsed-vs-expanded source access split, source/group switching callbacks, compact status indicators, detailed expanded source statuses, accessible icon-only controls, and the rule that source ingest jobs stay out of analysis runs.
- Boundary check: this plan intentionally keeps `WorkspaceMain`, `WorkspaceInspector`, current source readers, and current saved/active run surfaces in place. It does not introduce `ReportCanvas`, `RunCompanionTabs`, source readers, central `Report | Source` switching, snapshot behavior changes, or chat/evidence changes.
- Placeholder scan: the plan uses concrete file paths, concrete test names, concrete Svelte component code, concrete route snippets, concrete commands, concrete expected outcomes, and concrete commit messages.
- Type consistency: the plan uses Part 1 `WorkspaceSelection`, existing `Source`, `AnalysisSourceGroup`, `AnalysisSourceOption`, `TakeoutImportJobRecord`, `SourceJobRecord`, `YoutubeSourceSummary`, and existing route callback names consistently.
