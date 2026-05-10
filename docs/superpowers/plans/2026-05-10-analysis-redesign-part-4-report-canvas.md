# Analysis Result-First Redesign Part 4 Report Canvas Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current mixed central `/analysis` surface with a `ReportCanvas` that has explicit `Report | Source` modes, shows report setup only when no run is open, and makes run snapshot versus live source browsing visible and honest.

**Architecture:** Build a new center-surface component family around `ReportCanvas`, add small state helpers for source-basis decisions, wire it to Part 1/2 `workspaceUiState.canvasMode` and `workspaceUiState.sourceViewBasis`, and keep `CompactSourceRail` plus the legacy right `WorkspaceInspector` in place until later parts. Reuse existing report, live source, YouTube, template, group, and chat components only as transitional internals.

**Tech Stack:** SvelteKit 2, Svelte 5 runes, TypeScript, Vitest raw-source tests, existing Extractum UI components, lucide Svelte icons, Part 1 snapshot-only `listAnalysisRunMessages`, Part 1 `analysis-workspace-state`, Part 2 persisted `workspaceUiState`, Part 3 `CompactSourceRail`.

---

## Prerequisites

Implement this part only after Parts 1, 2, and 3 are implemented and committed, not merely planned.

This plan assumes these files already exist from earlier parts:

- `src/lib/analysis-workspace-state.ts`
- `src/lib/analysis-workspace-persistence.ts`
- `src/lib/components/analysis/compact-source-rail.svelte`
- `src/lib/components/analysis/source-switcher-panel.svelte`
- snapshot-only frontend API `listAnalysisRunMessages(...)` in `src/lib/api/analysis-runs.ts`
- snapshot-message frontend types in `src/lib/types/analysis.ts`
- route-owned `workspaceUiState` in `src/routes/analysis/+page.svelte`

This plan also assumes `src/routes/analysis/+page.svelte` already:

- applies `selectSourceWorkspace(...)` when selecting a source;
- applies `selectSourceGroupWorkspace(...)` when selecting a group;
- applies `openRunWorkspaceState(...)` when a run opens;
- persists `canvasMode`, `sourceViewBasis`, `companionTab`, `historyScope`, and `runFilter`;
- renders `CompactSourceRail` instead of `WorkspaceRail`.

If any prerequisite is missing, stop and implement the earlier part first.

This is **Part 4 of 7**. Stop after this part is implemented, verified, and committed. Continue to Part 5 only after explicit user approval.

## Part Boundary

Part 4 may:

- create `ReportCanvas`;
- create focused `ReportCanvas` subcomponents;
- move the central report setup, report output, live source preview, and snapshot source state out of `WorkspaceMain`;
- add a basic paged run-snapshot message panel using `listAnalysisRunMessages`;
- use existing `SourceContextPanel`, `SourceMessagesPanel`, `YoutubeSourceDetail`, and `YoutubePlaylistDetail` for live source browsing;
- keep a temporary legacy follow-up chat block inside `ReportCanvas` until Part 6 creates `RunCompanionTabs`;
- keep `WorkspaceInspector` as the right panel;
- leave `WorkspaceMain` as an unused legacy component for rollback during the staged migration.

Part 4 must not:

- create or wire `RunCompanionTabs`;
- implement the final Telegram TDesktop-like source reader;
- implement the final YouTube transcript/comment reader;
- move evidence or chat into a new companion tab system;
- silently substitute live source data when an opened run snapshot is unavailable;
- infer snapshot availability from run status alone;
- close an opened run when switching `Report | Source`;
- close an opened run when locally filtering a run snapshot;
- show template editing or report setup as part of the primary report-reading surface when any run is open;
- make failed or cancelled runs look completed because snapshot source material is visible;
- put source ingest jobs in analysis run history.

## File Structure

- Create: `src/lib/analysis-report-canvas-state.ts`
  - Responsibility: pure helpers for source-basis labels, snapshot availability, source-mode decisions, YouTube corpus labels, and run header copy.
- Create: `src/lib/analysis-report-canvas-state.test.ts`
  - Responsibility: state-helper coverage for no-run setup/source, active pending snapshot, terminal unavailable snapshot, available snapshot, explicit live source, and no status-only inference.
- Create: `src/lib/analysis-report-canvas.test.ts`
  - Responsibility: raw-source coverage for `ReportCanvas`, subcomponent contracts, run metadata, mode switching, setup placement, and absence of `RunCompanionTabs`.
- Create: `src/lib/analysis-report-canvas-route.test.ts`
  - Responsibility: raw-source coverage for route wiring, snapshot loading, `workspaceUiState` callbacks, and replacement of `WorkspaceMain`.
- Create: `src/lib/components/analysis/report-run-header.svelte`
  - Responsibility: opened-run metadata strip and source-basis status.
- Create: `src/lib/components/analysis/report-setup-panel.svelte`
  - Responsibility: no-run report setup controls, template editor drawer, group editor drawer, and primary `Run report` action.
- Create: `src/lib/components/analysis/run-snapshot-messages-panel.svelte`
  - Responsibility: minimal bounded snapshot-message reader backed by Part 1 snapshot-only pages.
- Create: `src/lib/components/analysis/report-source-surface.svelte`
  - Responsibility: `Source` mode basis shell, snapshot pending/unavailable/live indicators, `View live source`, `Back to run snapshot`, and transitional live-source component reuse.
- Create: `src/lib/components/analysis/report-canvas.svelte`
  - Responsibility: central `Report | Source` canvas, mode tabs, report/setup/source switching, and transitional NotebookLM/template/group/chat wiring.
- Modify: `src/routes/analysis/+page.svelte`
  - Responsibility: import/render `ReportCanvas`, pass `workspaceUiState.canvasMode`, pass `workspaceUiState.sourceViewBasis`, load run snapshot first pages, and update `workspaceUiState` on mode/basis actions.
- Keep: `src/lib/components/analysis/workspace-main.svelte`
  - Responsibility: legacy component kept unused for rollback in this staged migration.

## Task 1: Add Report Canvas Contract Tests

**Files:**
- Create: `src/lib/analysis-report-canvas-state.test.ts`
- Create: `src/lib/analysis-report-canvas.test.ts`
- Create: `src/lib/analysis-report-canvas-route.test.ts`

- [ ] **Step 1: Write failing state-helper tests**

Create `src/lib/analysis-report-canvas-state.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  runSnapshotAvailabilityFromPage,
  sourceBasisLabel,
  sourceCanvasSurface,
  youtubeCorpusModeLabel,
  type RunSnapshotAvailability,
} from "./analysis-report-canvas-state";
import type { AnalysisRunDetail, AnalysisRunMessagesPage } from "./types/analysis";

function run(overrides: Partial<AnalysisRunDetail> = {}): AnalysisRunDetail {
  return {
    id: 42,
    run_type: "report",
    scope_type: "single_source",
    source_id: 7,
    source_title: "Source A",
    source_group_id: null,
    source_group_name: null,
    scope_label: "Source A",
    period_from: 1704067200,
    period_to: 1706659200,
    output_language: "Russian",
    prompt_template_id: 3,
    prompt_template_name: "Weekly",
    prompt_template_version: 5,
    provider_profile: "default",
    provider: "openai",
    model: "gpt-5.4",
    youtube_corpus_mode: "transcript_description_comments",
    status: "completed",
    error: null,
    has_trace_data: true,
    created_at: 1706660000,
    completed_at: 1706660400,
    result_markdown: "Report",
    ...overrides,
  };
}

function page(messageCount: number): AnalysisRunMessagesPage {
  return {
    messages: Array.from({ length: messageCount }, (_, index) => ({
      item_id: index + 1,
      source_id: 7,
      external_id: `m-${index + 1}`,
      author: null,
      published_at: 1704067200 + index,
      ref: `S7-${index + 1}`,
      content: `Message ${index + 1}`,
      item_kind: "message",
      source_type: "telegram",
      source_subtype: null,
      metadata_json: null,
    })),
    next_cursor: null,
    has_more: false,
  };
}

describe("report canvas state", () => {
  it("uses report setup and live source when no run is open", () => {
    expect(sourceCanvasSurface({
      currentRun: null,
      sourceViewBasis: "live_source",
      snapshotAvailability: "unknown",
    })).toBe("live_source");

    expect(sourceBasisLabel({
      currentRun: null,
      sourceViewBasis: "live_source",
      snapshotAvailability: "unknown",
    })).toBe("Live source");
  });

  it("marks active empty snapshot pages as capturing only after probing snapshot data", () => {
    const availability = runSnapshotAvailabilityFromPage({
      currentRun: run({ status: "running", completed_at: null }),
      page: page(0),
      loading: false,
      errorMessage: "",
    });

    expect(availability).toBe("capturing");
  });

  it("marks failed and cancelled empty snapshot pages as unavailable after probing snapshot data", () => {
    expect(runSnapshotAvailabilityFromPage({
      currentRun: run({ status: "failed", error: "Provider failed" }),
      page: page(0),
      loading: false,
      errorMessage: "",
    })).toBe("unavailable");

    expect(runSnapshotAvailabilityFromPage({
      currentRun: run({ status: "cancelled" }),
      page: page(0),
      loading: false,
      errorMessage: "",
    })).toBe("unavailable");
  });

  it("treats any non-empty snapshot page as available regardless of terminal status", () => {
    expect(runSnapshotAvailabilityFromPage({
      currentRun: run({ status: "failed", error: "Failed after capture" }),
      page: page(1),
      loading: false,
      errorMessage: "",
    })).toBe("available");
  });

  it("does not infer snapshot availability from run status alone", () => {
    expect(runSnapshotAvailabilityFromPage({
      currentRun: run({ status: "completed" }),
      page: null,
      loading: false,
      errorMessage: "",
    })).toBe("unknown");
  });

  it("keeps explicit live source visible while an opened run remains bound", () => {
    expect(sourceCanvasSurface({
      currentRun: run(),
      sourceViewBasis: "live_source",
      snapshotAvailability: "available",
    })).toBe("live_source");

    expect(sourceBasisLabel({
      currentRun: run(),
      sourceViewBasis: "live_source",
      snapshotAvailability: "available",
    })).toBe("Live source");
  });

  it.each([
    ["unknown", "Snapshot status unknown"],
    ["capturing", "Snapshot pending"],
    ["available", "Run snapshot"],
    ["unavailable", "Snapshot unavailable"],
  ] satisfies Array<[RunSnapshotAvailability, string]>)("labels %s snapshot basis", (availability, label) => {
    expect(sourceBasisLabel({
      currentRun: run(),
      sourceViewBasis: "run_snapshot",
      snapshotAvailability: availability,
    })).toBe(label);
  });

  it("labels YouTube corpus modes for run headers", () => {
    expect(youtubeCorpusModeLabel("transcript_only")).toBe("Transcript");
    expect(youtubeCorpusModeLabel("transcript_description")).toBe("Transcript + description");
    expect(youtubeCorpusModeLabel("transcript_description_comments")).toBe("Transcript + description + comments");
    expect(youtubeCorpusModeLabel(null)).toBe("Not recorded");
  });
});
```

- [ ] **Step 2: Write failing raw-source component tests**

Create `src/lib/analysis-report-canvas.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import reportCanvasSource from "./components/analysis/report-canvas.svelte?raw";
import reportRunHeaderSource from "./components/analysis/report-run-header.svelte?raw";
import reportSetupPanelSource from "./components/analysis/report-setup-panel.svelte?raw";
import reportSourceSurfaceSource from "./components/analysis/report-source-surface.svelte?raw";
import runSnapshotMessagesPanelSource from "./components/analysis/run-snapshot-messages-panel.svelte?raw";

describe("report canvas component contract", () => {
  it("owns the central Report and Source modes", () => {
    expect(reportCanvasSource).toContain('class="report-canvas"');
    expect(reportCanvasSource).toContain('role="tablist"');
    expect(reportCanvasSource).toContain('aria-label="Report canvas mode"');
    expect(reportCanvasSource).toContain('onChangeCanvasMode("report")');
    expect(reportCanvasSource).toContain('onChangeCanvasMode("source")');
    expect(reportCanvasSource).toContain('{#if canvasMode === "report"}');
    expect(reportCanvasSource).toContain("<ReportSourceSurface");
    expect(reportCanvasSource).not.toContain("<RunCompanionTabs");
  });

  it("shows setup only when no run is open and report mode is selected", () => {
    expect(reportCanvasSource).toContain('{#if currentRun}');
    expect(reportCanvasSource).toContain("<ReportRunHeader");
    expect(reportCanvasSource).toContain("<ReportViewer");
    expect(reportCanvasSource).toContain("<ReportSetupPanel");
    expect(reportCanvasSource).toContain('class="temporary-follow-up"');
    expect(reportSetupPanelSource).toContain("TemplateEditor");
    expect(reportSetupPanelSource).toContain("SourceGroupEditor");
    expect(reportSetupPanelSource).toContain('class="template-editor-drawer"');
    expect(reportSetupPanelSource).toContain('class="group-editor-drawer"');
  });

  it("renders required opened-run header metadata", () => {
    expect(reportRunHeaderSource).toContain("Run #");
    expect(reportRunHeaderSource).toContain("runTargetLabel(currentRun)");
    expect(reportRunHeaderSource).toContain("currentRun.status");
    expect(reportRunHeaderSource).toContain("currentRun.created_at");
    expect(reportRunHeaderSource).toContain("currentRun.completed_at");
    expect(reportRunHeaderSource).toContain("currentRun.prompt_template_name");
    expect(reportRunHeaderSource).toContain("currentRun.prompt_template_version");
    expect(reportRunHeaderSource).toContain("currentRun.provider_profile");
    expect(reportRunHeaderSource).toContain("currentRun.provider");
    expect(reportRunHeaderSource).toContain("currentRun.model");
    expect(reportRunHeaderSource).toContain("sourceBasisLabel");
    expect(reportRunHeaderSource).toContain("youtubeCorpusModeLabel");
  });

  it("keeps snapshot and live source basis explicit", () => {
    expect(reportSourceSurfaceSource).toContain("sourceViewBasis === \"live_source\"");
    expect(reportSourceSurfaceSource).toContain("sourceViewBasis === \"run_snapshot\"");
    expect(reportSourceSurfaceSource).toContain("Live source");
    expect(reportSourceSurfaceSource).toContain("View live source");
    expect(reportSourceSurfaceSource).toContain("Back to run snapshot");
    expect(reportSourceSurfaceSource).toContain("Snapshot pending");
    expect(reportSourceSurfaceSource).toContain("Snapshot unavailable");
    expect(reportSourceSurfaceSource).toContain("<RunSnapshotMessagesPanel");
    expect(reportSourceSurfaceSource).toContain("<SourceContextPanel");
    expect(reportSourceSurfaceSource).toContain("<YoutubeSourceDetail");
    expect(reportSourceSurfaceSource).toContain("<YoutubePlaylistDetail");
  });

  it("keeps run snapshot reading bounded and snapshot-only", () => {
    expect(runSnapshotMessagesPanelSource).toContain("AnalysisRunMessage");
    expect(runSnapshotMessagesPanelSource).toContain("Load older snapshot messages");
    expect(runSnapshotMessagesPanelSource).toContain("hasMoreRunSnapshotMessages");
    expect(runSnapshotMessagesPanelSource).not.toContain("listSourceItems");
    expect(runSnapshotMessagesPanelSource).not.toContain("SourceMessagesPanel");
  });
});
```

- [ ] **Step 3: Write failing route wiring tests**

Create `src/lib/analysis-report-canvas-route.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";

describe("analysis route report canvas wiring", () => {
  it("renders ReportCanvas instead of the legacy WorkspaceMain", () => {
    expect(analysisPageSource).toContain(
      'import ReportCanvas from "$lib/components/analysis/report-canvas.svelte";',
    );
    expect(analysisPageSource).not.toContain(
      'import WorkspaceMain from "$lib/components/analysis/workspace-main.svelte";',
    );
    expect(analysisPageSource).toContain("<ReportCanvas");
    expect(analysisPageSource).not.toContain("<WorkspaceMain");
    expect(analysisPageSource).toContain("<WorkspaceInspector");
    expect(analysisPageSource).not.toContain("<RunCompanionTabs");
  });

  it("passes persisted canvas mode and source basis from workspace UI state", () => {
    expect(analysisPageSource).toContain("canvasMode={workspaceUiState.canvasMode}");
    expect(analysisPageSource).toContain("sourceViewBasis={workspaceUiState.sourceViewBasis}");
    expect(analysisPageSource).toContain("onChangeCanvasMode={(mode) =>");
    expect(analysisPageSource).toContain('canvasMode: mode');
    expect(analysisPageSource).toContain("onViewLiveSource={() =>");
    expect(analysisPageSource).toContain('sourceViewBasis: "live_source"');
    expect(analysisPageSource).toContain("onBackToRunSnapshot={() =>");
    expect(analysisPageSource).toContain('sourceViewBasis: "run_snapshot"');
  });

  it("loads run snapshot messages through the snapshot-only API", () => {
    expect(analysisPageSource).toContain("listAnalysisRunMessages");
    expect(analysisPageSource).toContain("loadRunSnapshotFirstPage");
    expect(analysisPageSource).toContain("loadMoreRunSnapshotMessages");
    expect(analysisPageSource).toContain("runSnapshotAvailability");
    expect(analysisPageSource).toContain("runSnapshotMessages");
    expect(analysisPageSource).toContain("runSnapshotCursor");
    expect(analysisPageSource).toContain("runSnapshotError");
    expect(analysisPageSource).not.toContain("listSourceItems({ runId");
  });

  it("does not switch back to snapshot automatically when the user explicitly views live source", () => {
    expect(analysisPageSource).toContain("lastSnapshotLoadKey");
    expect(analysisPageSource).toContain("workspaceUiState.sourceViewBasis === \"run_snapshot\"");
    expect(analysisPageSource).toContain("void loadRunSnapshotFirstPage(currentRun.id)");
    expect(analysisPageSource).not.toContain('sourceViewBasis: "run_snapshot", // automatic');
  });
});
```

- [ ] **Step 4: Run the focused tests and verify they fail**

Run:

```powershell
npm.cmd test -- src/lib/analysis-report-canvas-state.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-report-canvas-route.test.ts
```

Expected: FAIL because the new helper module and components do not exist and the route still renders `WorkspaceMain`.

- [ ] **Step 5: Commit the failing tests**

Run:

```powershell
git add src/lib/analysis-report-canvas-state.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-report-canvas-route.test.ts
git commit -m "test: define report canvas contract"
```

## Task 2: Add Report Canvas State Helpers

**Files:**
- Create: `src/lib/analysis-report-canvas-state.ts`

- [ ] **Step 1: Add the state helper module**

Create `src/lib/analysis-report-canvas-state.ts`:

```ts
import type {
  AnalysisRunDetail,
  AnalysisRunMessagesPage,
  YoutubeCorpusMode,
} from "$lib/types/analysis";
import type { SourceViewBasis } from "$lib/analysis-workspace-state";

export type RunSnapshotAvailability =
  | "unknown"
  | "capturing"
  | "available"
  | "unavailable";

export type SourceCanvasSurface =
  | "live_source"
  | "run_snapshot_unknown"
  | "run_snapshot_pending"
  | "run_snapshot_available"
  | "run_snapshot_unavailable";

export interface SnapshotAvailabilityInput {
  currentRun: Pick<AnalysisRunDetail, "status"> | null;
  page: Pick<AnalysisRunMessagesPage, "messages"> | null;
  loading: boolean;
  errorMessage: string;
}

export interface SourceBasisInput {
  currentRun: Pick<AnalysisRunDetail, "status"> | null;
  sourceViewBasis: SourceViewBasis;
  snapshotAvailability: RunSnapshotAvailability;
}

export function isActiveRunStatus(status: string) {
  return status === "queued" || status === "running";
}

export function runSnapshotAvailabilityFromPage({
  currentRun,
  page,
  loading,
  errorMessage,
}: SnapshotAvailabilityInput): RunSnapshotAvailability {
  if (!currentRun) return "unknown";
  if (errorMessage.trim()) return "unavailable";
  if (loading) return "unknown";
  if (page === null) return "unknown";
  if (page.messages.length > 0) return "available";
  return isActiveRunStatus(currentRun.status) ? "capturing" : "unavailable";
}

export function sourceCanvasSurface({
  currentRun,
  sourceViewBasis,
  snapshotAvailability,
}: SourceBasisInput): SourceCanvasSurface {
  if (!currentRun || sourceViewBasis === "live_source") {
    return "live_source";
  }

  if (snapshotAvailability === "available") return "run_snapshot_available";
  if (snapshotAvailability === "capturing") return "run_snapshot_pending";
  if (snapshotAvailability === "unavailable") return "run_snapshot_unavailable";
  return "run_snapshot_unknown";
}

export function sourceBasisLabel({
  currentRun,
  sourceViewBasis,
  snapshotAvailability,
}: SourceBasisInput) {
  if (!currentRun || sourceViewBasis === "live_source") {
    return "Live source";
  }

  if (snapshotAvailability === "available") return "Run snapshot";
  if (snapshotAvailability === "capturing") return "Snapshot pending";
  if (snapshotAvailability === "unavailable") return "Snapshot unavailable";
  return "Snapshot status unknown";
}

export function sourceBasisDescription({
  currentRun,
  sourceViewBasis,
  snapshotAvailability,
}: SourceBasisInput) {
  if (!currentRun) {
    return "Browsing the currently selected live source context.";
  }

  if (sourceViewBasis === "live_source") {
    return "Browsing live source data while the opened run remains bound to its saved report context.";
  }

  if (snapshotAvailability === "available") {
    return "Browsing the frozen source material captured for this run.";
  }

  if (snapshotAvailability === "capturing") {
    return "The run snapshot is not browsable yet. It becomes available after corpus capture is saved.";
  }

  if (snapshotAvailability === "unavailable") {
    return "Extractum cannot show a frozen source snapshot for this run.";
  }

  return "Checking whether a frozen run snapshot exists.";
}

export function canReturnToRunSnapshot(availability: RunSnapshotAvailability) {
  return availability === "available";
}

export function youtubeCorpusModeLabel(value: YoutubeCorpusMode | null | undefined) {
  if (value === "transcript_only") return "Transcript";
  if (value === "transcript_description") return "Transcript + description";
  if (value === "transcript_description_comments") return "Transcript + description + comments";
  return "Not recorded";
}
```

If Part 1 placed `SourceViewBasis` in `src/lib/analysis-workspace-state.ts` instead of `src/lib/types/analysis.ts`, import it from that module and update the tests to match the actual Part 1 file.

- [ ] **Step 2: Run the state-helper tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-report-canvas-state.test.ts
```

Expected: PASS.

- [ ] **Step 3: Commit the state helpers**

Run:

```powershell
git add src/lib/analysis-report-canvas-state.ts src/lib/analysis-report-canvas-state.test.ts
git commit -m "feat: add report canvas state helpers"
```

## Task 3: Add Opened Run Header Component

**Files:**
- Create: `src/lib/components/analysis/report-run-header.svelte`

- [ ] **Step 1: Create the run header component**

Create `src/lib/components/analysis/report-run-header.svelte`:

```svelte
<script lang="ts">
  import { Square } from "@lucide/svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import MetaCell from "$lib/components/ui/MetaCell.svelte";
  import {
    sourceBasisLabel,
    sourceBasisDescription,
    youtubeCorpusModeLabel,
    type RunSnapshotAvailability,
  } from "$lib/analysis-report-canvas-state";
  import type { BadgeVariant } from "$lib/components/ui/types";
  import type { SourceViewBasis } from "$lib/analysis-workspace-state";
  import type { AnalysisRunDetail } from "$lib/types/analysis";

  let {
    currentRun,
    sourceViewBasis,
    snapshotAvailability,
    traceRefCount,
    activePhase,
    activeProgress,
    canCancelCurrentRun,
    formatTimestamp,
    formatPeriod,
    runTargetLabel,
    statusTone,
    onCancelCurrentRun,
  }: {
    currentRun: AnalysisRunDetail;
    sourceViewBasis: SourceViewBasis;
    snapshotAvailability: RunSnapshotAvailability;
    traceRefCount: number;
    activePhase: string;
    activeProgress: string;
    canCancelCurrentRun: boolean;
    formatTimestamp: (value: number | null) => string;
    formatPeriod: (from: number, to: number) => string;
    runTargetLabel: (run: Pick<AnalysisRunDetail, "scope_type" | "source_id" | "source_title" | "source_group_id" | "source_group_name" | "scope_label">) => string;
    statusTone: (value: string) => BadgeVariant;
    onCancelCurrentRun: () => void | Promise<void>;
  } = $props();

  const basisLabel = $derived(sourceBasisLabel({
    currentRun,
    sourceViewBasis,
    snapshotAvailability,
  }));
  const basisDescription = $derived(sourceBasisDescription({
    currentRun,
    sourceViewBasis,
    snapshotAvailability,
  }));
  const hasSnapshotWarning = $derived(
    currentRun.status === "completed" && snapshotAvailability === "unavailable",
  );
</script>

<section class="report-run-header" aria-label="Opened run metadata">
  <div class="run-header-top">
    <div>
      <span class="eyebrow">Opened run</span>
      <h2>Run #{currentRun.id}</h2>
      <p>{runTargetLabel(currentRun)}</p>
    </div>
    <div class="run-header-actions">
      <Badge variant={statusTone(currentRun.status)}>{currentRun.status}</Badge>
      <Badge variant={snapshotAvailability === "unavailable" ? "warning" : "neutral"}>{basisLabel}</Badge>
      {#if canCancelCurrentRun}
        <Button variant="danger-soft" type="button" onclick={onCancelCurrentRun}>
          <Square size={15} aria-hidden="true" /> Cancel run
        </Button>
      {/if}
    </div>
  </div>

  {#if hasSnapshotWarning}
    <p class="snapshot-warning">
      Frozen source snapshot is missing. The saved report can still be read, but exact source browsing is degraded.
    </p>
  {/if}

  <div class="run-meta-grid">
    <MetaCell label="Scope">{runTargetLabel(currentRun)}</MetaCell>
    <MetaCell label="Status">{currentRun.status}</MetaCell>
    <MetaCell label="Created">{formatTimestamp(currentRun.created_at)}</MetaCell>
    <MetaCell label="Completed">{formatTimestamp(currentRun.completed_at)}</MetaCell>
    <MetaCell label="Period">{formatPeriod(currentRun.period_from, currentRun.period_to)}</MetaCell>
    <MetaCell label="Template">{currentRun.prompt_template_name ?? "Unknown"} v{currentRun.prompt_template_version}</MetaCell>
    <MetaCell label="Provider profile">{currentRun.provider_profile}</MetaCell>
    <MetaCell label="Provider/model">{currentRun.provider}/{currentRun.model}</MetaCell>
    <MetaCell label="Source basis">{basisDescription}</MetaCell>
    <MetaCell label="YouTube corpus">{youtubeCorpusModeLabel(currentRun.youtube_corpus_mode)}</MetaCell>
    <MetaCell label="Trace refs">{traceRefCount}</MetaCell>
    <MetaCell label="Live phase">{activePhase || currentRun.status}</MetaCell>
    <MetaCell label="Live progress">{activeProgress || "n/a"}</MetaCell>
  </div>

  {#if currentRun.error}
    <p class="run-error">{currentRun.error}</p>
  {/if}
</section>
```

Add scoped CSS matching existing `ReportViewer` density:

```css
.report-run-header {
  display: flex;
  flex-direction: column;
  gap: 0.9rem;
  padding: 1rem;
  border: 1px solid color-mix(in srgb, var(--primary) 16%, var(--border));
  border-radius: 8px;
  background: var(--panel);
  box-shadow: var(--shadow-soft);
}

.run-header-top {
  display: flex;
  justify-content: space-between;
  gap: 0.75rem;
  align-items: flex-start;
}

.eyebrow {
  display: inline-block;
  font-size: 0.68rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--muted);
  margin-bottom: 0.2rem;
}

h2,
p {
  margin: 0;
}

.run-header-top p,
.snapshot-warning {
  color: var(--muted);
  line-height: 1.45;
}

.run-header-actions {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.45rem;
}

.run-meta-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.7rem;
}

.snapshot-warning,
.run-error {
  padding: 0.7rem 0.85rem;
  border-radius: 8px;
}

.snapshot-warning {
  background: var(--status-warning-bg);
  color: var(--status-warning-text);
}

.run-error {
  background: var(--status-error-bg);
  color: var(--status-error-text);
}

@media (max-width: 960px) {
  .run-header-top {
    flex-direction: column;
  }

  .run-meta-grid {
    grid-template-columns: 1fr;
  }
}
```

- [ ] **Step 2: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: PASS.

- [ ] **Step 3: Commit the run header**

Run:

```powershell
git add src/lib/components/analysis/report-run-header.svelte
git commit -m "feat: add report run header"
```

## Task 4: Add Report Setup Panel

**Files:**
- Create: `src/lib/components/analysis/report-setup-panel.svelte`

- [ ] **Step 1: Create the no-run setup component**

Create `src/lib/components/analysis/report-setup-panel.svelte` by moving the existing run setup controls from `WorkspaceMain` into this component:

- period from/to;
- prompt template select;
- output language;
- YouTube corpus mode for YouTube scope;
- LLM profile select;
- model select;
- custom model input;
- model status;
- primary `Run report`;
- NotebookLM export action for single-source scope;
- source sync action for single-source live browsing;
- compact live strip when a run is starting.

Add these local states in the component:

```ts
let templateEditorOpen = $state(false);
let groupEditorOpen = $state(false);
```

Render template and group editors behind explicit drawer buttons:

```svelte
<div class="setup-secondary-actions">
  <Button type="button" variant="secondary" onclick={() => (templateEditorOpen = !templateEditorOpen)}>
    {templateEditorOpen ? "Hide template editor" : "Edit templates"}
  </Button>
  <Button type="button" variant="secondary" onclick={() => (groupEditorOpen = !groupEditorOpen)}>
    {groupEditorOpen ? "Hide group editor" : "Edit groups"}
  </Button>
</div>

{#if templateEditorOpen}
  <div class="template-editor-drawer" aria-label="Template editor drawer">
    <TemplateEditor
      compact={true}
      {selectedTemplate}
      {templateName}
      {templateBody}
      {savingTemplate}
      {deletingTemplate}
      onSaveTemplateCopy={onSaveTemplateCopy}
      onSaveTemplateChanges={onSaveTemplateChanges}
      onDeleteTemplate={onDeleteTemplate}
    />
  </div>
{/if}

{#if groupEditorOpen}
  <div class="group-editor-drawer" aria-label="Source group editor drawer">
    <SourceGroupEditor
      compact={true}
      {groups}
      selectedGroupId={selectedGroupId}
      {selectedGroup}
      {groupName}
      {groupSourceType}
      {groupMemberSourceIds}
      sources={sourceMetricsList}
      {savingGroup}
      {deletingGroup}
      {formatTimestamp}
      {isGroupSourceSelected}
      onChangeSelectedGroupId={onChangeSelectedGroupId}
      onChangeGroupName={onChangeGroupName}
      onChangeGroupSourceType={onChangeGroupSourceType}
      onToggleSource={onToggleGroupSource}
      onStartNewGroup={onStartNewGroup}
      onSaveGroupCopy={onSaveGroupCopy}
      onSaveGroupChanges={onSaveGroupChanges}
      onDeleteGroup={onDeleteGroup}
    />
  </div>
{/if}
```

Use `Button`, `Input`, `Select`, `Badge`, `TemplateEditor`, and `SourceGroupEditor` imports from existing `WorkspaceMain`.

Do not render `ReportSetupPanel` when `currentRun` is non-null. That condition belongs in `ReportCanvas`.

- [ ] **Step 2: Run raw-source tests and Svelte check**

Run:

```powershell
npm.cmd test -- src/lib/analysis-report-canvas.test.ts
npm.cmd run check
```

Expected: `analysis-report-canvas.test.ts` still FAILS because `ReportCanvas` and source components are not finished. `npm.cmd run check` should PASS for the setup component.

- [ ] **Step 3: Commit the setup panel**

Run:

```powershell
git add src/lib/components/analysis/report-setup-panel.svelte
git commit -m "feat: add report setup panel"
```

## Task 5: Add Basic Run Snapshot Source Surface

**Files:**
- Create: `src/lib/components/analysis/run-snapshot-messages-panel.svelte`
- Create: `src/lib/components/analysis/report-source-surface.svelte`

- [ ] **Step 1: Create the snapshot-message panel**

Create `src/lib/components/analysis/run-snapshot-messages-panel.svelte`:

```svelte
<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import PanelHeader from "$lib/components/ui/PanelHeader.svelte";
  import type { AnalysisRunMessage } from "$lib/types/analysis";

  let {
    messages,
    loadingRunSnapshotMessages,
    hasMoreRunSnapshotMessages,
    formatTimestamp,
    onLoadMoreRunSnapshotMessages,
  }: {
    messages: AnalysisRunMessage[];
    loadingRunSnapshotMessages: boolean;
    hasMoreRunSnapshotMessages: boolean;
    formatTimestamp: (value: number | null) => string;
    onLoadMoreRunSnapshotMessages: () => void | Promise<void>;
  } = $props();

  function sourceLabel(message: AnalysisRunMessage) {
    const type = message.source_type ?? "source";
    const subtype = message.source_subtype ? `/${message.source_subtype}` : "";
    return `${type}${subtype} #${message.source_id}`;
  }
</script>

<section class="run-snapshot-messages">
  <PanelHeader title="Run snapshot">
    {#if loadingRunSnapshotMessages}
      <span class="subtle">Loading snapshot...</span>
    {:else}
      <span class="subtle">{messages.length} snapshot messages loaded</span>
    {/if}
  </PanelHeader>

  {#if !loadingRunSnapshotMessages && messages.length === 0}
    <EmptyState description="No frozen source messages were returned for this run snapshot." />
  {:else}
    <ul class="snapshot-list">
      {#each messages as message (message.ref)}
        <li>
          <div class="snapshot-meta">
            <Badge variant="neutral">{message.ref}</Badge>
            <span>{sourceLabel(message)}</span>
            <span>{formatTimestamp(message.published_at)}</span>
            {#if message.author}<span>{message.author}</span>{/if}
          </div>
          <p>{message.content || "No text content captured for this snapshot row."}</p>
        </li>
      {/each}
    </ul>
  {/if}

  {#if hasMoreRunSnapshotMessages}
    <div class="snapshot-footer">
      <Button
        type="button"
        variant="secondary"
        disabled={loadingRunSnapshotMessages}
        onclick={onLoadMoreRunSnapshotMessages}
      >
        {loadingRunSnapshotMessages ? "Loading..." : "Load older snapshot messages"}
      </Button>
    </div>
  {/if}
</section>
```

Add CSS with bounded rows:

```css
.run-snapshot-messages {
  display: flex;
  flex-direction: column;
  gap: 0.85rem;
  min-width: 0;
}

.snapshot-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.55rem;
}

.snapshot-list li {
  padding: 0.9rem 1rem;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--panel-strong);
}

.snapshot-list p {
  margin: 0;
  white-space: pre-wrap;
  line-height: 1.5;
}

.snapshot-meta {
  display: flex;
  gap: 0.55rem;
  flex-wrap: wrap;
  color: var(--muted);
  font-size: 0.78rem;
  margin-bottom: 0.45rem;
}

.snapshot-footer {
  display: flex;
  justify-content: center;
}

.subtle {
  color: var(--muted);
  font-size: 0.78rem;
}
```

- [ ] **Step 2: Create the source surface component**

Create `src/lib/components/analysis/report-source-surface.svelte`.

The component must accept:

- `currentRun`
- `sourceViewBasis`
- `snapshotAvailability`
- `runSnapshotMessages`
- `loadingRunSnapshotMessages`
- `runSnapshotError`
- `hasMoreRunSnapshotMessages`
- live source props currently used by `SourceContextPanel`, `YoutubeSourceDetail`, and `YoutubePlaylistDetail`
- source ingest callbacks currently passed to those components
- `onViewLiveSource`
- `onBackToRunSnapshot`
- `onLoadMoreRunSnapshotMessages`

Implement this top-level structure:

```svelte
<section class="report-source-surface">
  <div class="source-basis-header">
    <div>
      <span class="eyebrow">Source</span>
      <h2>{sourceBasisLabel({ currentRun, sourceViewBasis, snapshotAvailability })}</h2>
      <p>{sourceBasisDescription({ currentRun, sourceViewBasis, snapshotAvailability })}</p>
    </div>
    <div class="source-basis-actions">
      {#if currentRun && sourceViewBasis === "run_snapshot" && snapshotAvailability !== "available"}
        <Button type="button" variant="secondary" onclick={onViewLiveSource}>View live source</Button>
      {/if}
      {#if currentRun && sourceViewBasis === "live_source" && canReturnToRunSnapshot(snapshotAvailability)}
        <Button type="button" variant="secondary" onclick={onBackToRunSnapshot}>Back to run snapshot</Button>
      {/if}
      {#if sourceViewBasis === "live_source"}
        <Badge variant="warning">Live source</Badge>
      {:else if snapshotAvailability === "available"}
        <Badge variant="success">Run snapshot</Badge>
      {:else if snapshotAvailability === "capturing"}
        <Badge variant="info">Snapshot pending</Badge>
      {:else if snapshotAvailability === "unavailable"}
        <Badge variant="warning">Snapshot unavailable</Badge>
      {:else}
        <Badge variant="neutral">Snapshot status unknown</Badge>
      {/if}
    </div>
  </div>

  {#if currentRun && sourceViewBasis === "run_snapshot"}
    {#if snapshotAvailability === "available"}
      <RunSnapshotMessagesPanel
        messages={runSnapshotMessages}
        {loadingRunSnapshotMessages}
        {hasMoreRunSnapshotMessages}
        {formatTimestamp}
        {onLoadMoreRunSnapshotMessages}
      />
    {:else if snapshotAvailability === "capturing"}
      <StatusMessage tone="muted">Snapshot pending. The frozen source corpus is not browsable yet.</StatusMessage>
    {:else if snapshotAvailability === "unavailable"}
      <StatusMessage tone="warning">
        Snapshot unavailable. This run ended before Extractum could expose a frozen source snapshot.
      </StatusMessage>
      {#if runSnapshotError}
        <StatusMessage tone="error">{runSnapshotError}</StatusMessage>
      {/if}
    {:else}
      <StatusMessage tone="muted">Checking run snapshot availability...</StatusMessage>
    {/if}
  {:else}
    {@render liveSourceSurface()}
  {/if}
</section>
```

Define the `liveSourceSurface` snippet inside the component. It must reuse existing live source components:

```svelte
{#snippet liveSourceSurface()}
  {#if analysisScope === "single_source" && currentSource}
    {#key `${analysisScope}:${currentSource.id}:${currentRun?.id ?? "idle"}:live`}
      {#if currentSource.sourceType === "youtube" && currentSource.sourceSubtype === "video"}
        <YoutubeSourceDetail
          source={currentSource}
          detail={youtubeVideoDetail}
          jobs={sourceJobs}
          {loadingYoutubeDetail}
          {formatTimestamp}
          onSyncMetadata={onSyncYoutubeMetadata}
          onSyncTranscript={onSyncYoutubeTranscript}
          onSyncComments={onSyncYoutubeComments}
          onCancelJob={onCancelSourceJob}
        />
      {:else if currentSource.sourceType === "youtube" && currentSource.sourceSubtype === "playlist"}
        <YoutubePlaylistDetail
          source={currentSource}
          detail={youtubePlaylistDetail}
          jobs={sourceJobs}
          {loadingYoutubeDetail}
          {formatTimestamp}
          onOpenSource={onOpenSource}
          onSyncPlaylist={onSyncYoutubePlaylist}
          onRetryFailed={onRetryFailedYoutubePlaylistVideos}
          onSyncPlaylistVideo={onSyncYoutubePlaylistVideo}
          onRetryPlaylistVideo={onRetryYoutubePlaylistVideo}
          onCancelJob={onCancelSourceJob}
        />
      {:else}
        <SourceContextPanel
          currentRunOpen={!!currentRun}
          {currentSourceMetric}
          {sourceItems}
          {loadingItems}
          {sourceTopics}
          {loadingSourceTopics}
          {selectedTopicKey}
          {showTopicSelector}
          contentLabel={currentSourceContentLabel}
          {formatTimestamp}
          onChangeSelectedTopicKey={onChangeSelectedTopicKey}
        />
      {/if}
    {/key}
  {:else if analysisScope === "source_group" && currentGroup}
    <StatusMessage tone="muted" surface={false}>
      Source group live browsing remains summarized in this part. Full group readers are implemented in Part 5.
    </StatusMessage>
  {:else}
    <StatusMessage tone="muted" surface={false}>Select a source or source group to browse source material.</StatusMessage>
  {/if}
{/snippet}
```

This group live summary is acceptable in Part 4 because Part 5 owns final source readers. It must be visibly labeled as live browsing and must not be used as a silent run-snapshot fallback.

- [ ] **Step 3: Run raw-source tests and Svelte check**

Run:

```powershell
npm.cmd test -- src/lib/analysis-report-canvas.test.ts
npm.cmd run check
```

Expected: `analysis-report-canvas.test.ts` still FAILS until `ReportCanvas` exists. `npm.cmd run check` should PASS for these two components.

- [ ] **Step 4: Commit source surface components**

Run:

```powershell
git add src/lib/components/analysis/run-snapshot-messages-panel.svelte src/lib/components/analysis/report-source-surface.svelte
git commit -m "feat: add report canvas source surface"
```

## Task 6: Add ReportCanvas

**Files:**
- Create: `src/lib/components/analysis/report-canvas.svelte`

- [ ] **Step 1: Create the central canvas component**

Create `src/lib/components/analysis/report-canvas.svelte`.

Start from the current `WorkspaceMain` prop signature so route wiring stays mechanical. Add these explicit props:

```ts
canvasMode: CanvasMode;
sourceViewBasis: SourceViewBasis;
runSnapshotAvailability: RunSnapshotAvailability;
runSnapshotMessages: AnalysisRunMessage[];
loadingRunSnapshotMessages: boolean;
runSnapshotError: string;
hasMoreRunSnapshotMessages: boolean;
onChangeCanvasMode: (mode: CanvasMode) => void;
onViewLiveSource: () => void;
onBackToRunSnapshot: () => void;
onLoadMoreRunSnapshotMessages: () => void | Promise<void>;
```

Import the types from the actual Part 1 modules:

```ts
import type { CanvasMode, SourceViewBasis } from "$lib/analysis-workspace-state";
import type { RunSnapshotAvailability } from "$lib/analysis-report-canvas-state";
import type { AnalysisRunMessage } from "$lib/types/analysis";
```

If Part 1 exports `CanvasMode` or `SourceViewBasis` from `src/lib/types/analysis.ts`, import them from there instead.

Use this shell:

```svelte
<section class="report-canvas">
  <div class="canvas-toolbar">
    <div class="canvas-title">
      <span class="eyebrow">{currentRun ? "Run workspace" : "Analysis setup"}</span>
      <h2>{currentRun ? runTargetLabel(currentRun) : currentScopeTitle}</h2>
      <p>{currentRun ? "Read the report or inspect the source basis for this run." : currentScopeSummary}</p>
    </div>
    <div class="canvas-tabs" role="tablist" aria-label="Report canvas mode">
      <Button
        type="button"
        role="tab"
        variant="secondary"
        selected={canvasMode === "report"}
        ariaSelected={canvasMode === "report"}
        onclick={() => onChangeCanvasMode("report")}
      >
        Report
      </Button>
      <Button
        type="button"
        role="tab"
        variant="secondary"
        selected={canvasMode === "source"}
        ariaSelected={canvasMode === "source"}
        onclick={() => onChangeCanvasMode("source")}
      >
        Source
      </Button>
    </div>
  </div>

  {#if canvasMode === "report"}
    {#if currentRun}
      <ReportRunHeader
        {currentRun}
        {sourceViewBasis}
        snapshotAvailability={runSnapshotAvailability}
        traceRefCount={traceRefCount}
        {activePhase}
        {activeProgress}
        {canCancelCurrentRun}
        {formatTimestamp}
        {formatPeriod}
        {runTargetLabel}
        {statusTone}
        {onCancelCurrentRun}
      />
      <ReportViewer
        {currentRun}
        {loadingRunDetail}
        streamedOutput={focusedStreamedOutput}
        traceRefCount={traceRefCount}
        {selectedTraceRef}
        livePhase={activePhase}
        liveProgress={activeProgress}
        {canCancelCurrentRun}
        {formatTimestamp}
        {formatPeriod}
        {runTargetLabel}
        {statusTone}
        {reportLines}
        {onFocusTraceRef}
        {onCancelCurrentRun}
      />
      <div class="temporary-follow-up" aria-label="Temporary follow-up chat until companion tabs ship">
        <ChatPanel
          {currentRun}
          {loadingChat}
          {chatMessages}
          {chatQuestion}
          {chatting}
          {canCancelChat}
          {clearingChat}
          {selectedTraceRef}
          {reportLines}
          onFocusTraceRef={onFocusTraceRef}
          onAskQuestion={onAskRunQuestion}
          onCancelChat={onCancelChat}
          onClearChat={onClearChat}
          onChangeChatQuestion={onChangeChatQuestion}
        />
      </div>
    {:else}
      <ReportSetupPanel
        {analysisScope}
        {currentSource}
        {currentGroup}
        {currentSourceMetric}
        {periodFrom}
        {periodTo}
        {selectedTemplateId}
        {loadingTemplates}
        {templates}
        {outputLanguage}
        {youtubeCorpusMode}
        {llmProfiles}
        {activeLlmProfile}
        {selectedLlmProfileId}
        {selectedLlmModel}
        {customModelOverride}
        {llmProviderModels}
        {loadingLlmProviderModels}
        {llmModelStatus}
        {startingReport}
        {selectedSourceId}
        {selectedGroupId}
        {selectedTemplate}
        {templateName}
        {templateBody}
        {savingTemplate}
        {deletingTemplate}
        {groups}
        {groupName}
        {groupSourceType}
        {groupMemberSourceIds}
        {selectedGroup}
        {savingGroup}
        {deletingGroup}
        {sourceMetricsList}
        {syncingIds}
        {formatTimestamp}
        {formatPeriod}
        {phaseLabel}
        {accountLabel}
        {sourceSyncDisabledReason}
        {startOfDayUnix}
        {endOfDayUnix}
        {isGroupSourceSelected}
        onChangePeriodFrom={onChangePeriodFrom}
        onChangePeriodTo={onChangePeriodTo}
        onChangeSelectedTemplateId={onChangeSelectedTemplateId}
        onChangeOutputLanguage={onChangeOutputLanguage}
        onChangeYoutubeCorpusMode={onChangeYoutubeCorpusMode}
        onChangeLlmProfile={onChangeLlmProfile}
        onChangeLlmModel={onChangeLlmModel}
        onChangeCustomModelOverride={onChangeCustomModelOverride}
        onRunReport={onRunReport}
        onSyncCurrentSource={onSyncCurrentSource}
        onOpenNotebookLmExport={onOpenNotebookLmExport}
        onSaveTemplateCopy={onSaveTemplateCopy}
        onSaveTemplateChanges={onSaveTemplateChanges}
        onDeleteTemplate={onDeleteTemplate}
        onChangeSelectedGroupId={onChangeSelectedGroupId}
        onChangeGroupName={onChangeGroupName}
        onChangeGroupSourceType={onChangeGroupSourceType}
        onToggleGroupSource={onToggleGroupSource}
        onStartNewGroup={onStartNewGroup}
        onSaveGroupCopy={onSaveGroupCopy}
        onSaveGroupChanges={onSaveGroupChanges}
        onDeleteGroup={onDeleteGroup}
      />
    {/if}
  {:else}
    <ReportSourceSurface
      {analysisScope}
      {currentRun}
      {sourceViewBasis}
      snapshotAvailability={runSnapshotAvailability}
      {runSnapshotMessages}
      {loadingRunSnapshotMessages}
      {runSnapshotError}
      {hasMoreRunSnapshotMessages}
      {currentSource}
      {currentGroup}
      {currentSourceMetric}
      {sourceItems}
      {loadingItems}
      {sourceTopics}
      {loadingSourceTopics}
      {selectedTopicKey}
      {showTopicSelector}
      {currentSourceContentLabel}
      {sourceJobs}
      {youtubeVideoDetail}
      {youtubePlaylistDetail}
      {loadingYoutubeDetail}
      {formatTimestamp}
      onViewLiveSource={onViewLiveSource}
      onBackToRunSnapshot={onBackToRunSnapshot}
      onLoadMoreRunSnapshotMessages={onLoadMoreRunSnapshotMessages}
      onChangeSelectedTopicKey={onChangeSelectedTopicKey}
      onSyncYoutubeMetadata={onSyncYoutubeMetadata}
      onSyncYoutubeTranscript={onSyncYoutubeTranscript}
      onSyncYoutubeComments={onSyncYoutubeComments}
      onSyncYoutubePlaylist={onSyncYoutubePlaylist}
      onRetryFailedYoutubePlaylistVideos={onRetryFailedYoutubePlaylistVideos}
      onSyncYoutubePlaylistVideo={onSyncYoutubePlaylistVideo}
      onRetryYoutubePlaylistVideo={onRetryYoutubePlaylistVideo}
      onCancelSourceJob={onCancelSourceJob}
      onOpenSource={onOpenSource}
    />
  {/if}

  <NotebookLmExportDialog
    open={exportDialogOpen}
    source={currentSource}
    form={notebookLmExportForm}
    exporting={exportingNotebookLm}
    result={notebookLmExportResult}
    progress={notebookLmExportProgress}
    onClose={onCloseNotebookLmExport}
    onChooseFolder={onChooseNotebookLmOutputDir}
    onExport={onExportNotebookLm}
    onChangeForm={onChangeNotebookLmExportForm}
  />
</section>
```

The `ReportCanvas` CSS must avoid nested card stacks. Use full-width canvas bands and 8px radii for repeated framed controls:

```css
.report-canvas {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.9rem;
}

.canvas-toolbar {
  display: flex;
  justify-content: space-between;
  gap: 0.8rem;
  align-items: flex-start;
  padding: 1rem;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--panel);
  box-shadow: var(--shadow);
}

.canvas-title {
  min-width: 0;
}

.canvas-title h2,
.canvas-title p {
  margin: 0;
}

.canvas-title p {
  margin-top: 0.3rem;
  color: var(--muted);
  line-height: 1.45;
}

.canvas-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  padding: 0.2rem;
  border-radius: 8px;
  background: color-mix(in srgb, var(--panel-strong) 70%, transparent);
}

.temporary-follow-up {
  border: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
  border-radius: 8px;
  background: var(--panel);
}

@media (max-width: 720px) {
  .canvas-toolbar {
    flex-direction: column;
  }
}
```

- [ ] **Step 2: Run component tests and Svelte check**

Run:

```powershell
npm.cmd test -- src/lib/analysis-report-canvas.test.ts
npm.cmd run check
```

Expected: `analysis-report-canvas.test.ts` PASS. `npm.cmd run check` PASS.

- [ ] **Step 3: Commit ReportCanvas**

Run:

```powershell
git add src/lib/components/analysis/report-canvas.svelte src/lib/analysis-report-canvas.test.ts
git commit -m "feat: add analysis report canvas"
```

## Task 7: Wire ReportCanvas Into `/analysis`

**Files:**
- Modify: `src/routes/analysis/+page.svelte`

- [ ] **Step 1: Replace the legacy central component import**

In `src/routes/analysis/+page.svelte`, replace:

```ts
import WorkspaceMain from "$lib/components/analysis/workspace-main.svelte";
```

with:

```ts
import ReportCanvas from "$lib/components/analysis/report-canvas.svelte";
```

Add imports:

```ts
import { listAnalysisRunMessages } from "$lib/api/analysis-runs";
import {
  runSnapshotAvailabilityFromPage,
  type RunSnapshotAvailability,
} from "$lib/analysis-report-canvas-state";
import type {
  AnalysisRunMessage,
  AnalysisRunMessageCursor,
  AnalysisRunMessagesPage,
} from "$lib/types/analysis";
import type { CanvasMode } from "$lib/analysis-workspace-state";
```

If the route already imports from these modules, extend the existing import lists.

- [ ] **Step 2: Add route state for run snapshot pages**

Near the run/chat state declarations, add:

```ts
let runSnapshotAvailability = $state<RunSnapshotAvailability>("unknown");
let runSnapshotMessages = $state<AnalysisRunMessage[]>([]);
let runSnapshotCursor = $state<AnalysisRunMessageCursor | null>(null);
let runSnapshotHasMore = $state(false);
let loadingRunSnapshotMessages = $state(false);
let runSnapshotError = $state("");
let runSnapshotPage: AnalysisRunMessagesPage | null = null;
let lastSnapshotLoadKey = "";
```

- [ ] **Step 3: Add route helpers for mode and basis transitions**

Add these helpers near `applyWorkspaceUiState(...)` from Part 2:

```ts
function changeCanvasMode(mode: CanvasMode) {
  applyWorkspaceUiState({
    ...workspaceUiState,
    canvasMode: mode,
  });
}

function viewLiveSourceForOpenedRun() {
  applyWorkspaceUiState({
    ...workspaceUiState,
    canvasMode: "source",
    sourceViewBasis: "live_source",
  });
}

function backToRunSnapshot() {
  applyWorkspaceUiState({
    ...workspaceUiState,
    canvasMode: "source",
    sourceViewBasis: "run_snapshot",
  });
}

function resetRunSnapshotState() {
  runSnapshotAvailability = "unknown";
  runSnapshotMessages = [];
  runSnapshotCursor = null;
  runSnapshotHasMore = false;
  loadingRunSnapshotMessages = false;
  runSnapshotError = "";
  runSnapshotPage = null;
  lastSnapshotLoadKey = "";
}

function applySnapshotPage(run: AnalysisRunDetail, page: AnalysisRunMessagesPage, append: boolean) {
  runSnapshotPage = page;
  runSnapshotMessages = append ? [...runSnapshotMessages, ...page.messages] : page.messages;
  runSnapshotCursor = page.next_cursor;
  runSnapshotHasMore = page.has_more;
  runSnapshotAvailability = runSnapshotAvailabilityFromPage({
    currentRun: run,
    page,
    loading: false,
    errorMessage: "",
  });
}
```

- [ ] **Step 4: Add snapshot-only loaders**

Add:

```ts
async function loadRunSnapshotFirstPage(runId: number) {
  const run = currentRun;
  if (!run || run.id !== runId) {
    return;
  }

  const loadKey = `${runId}:first`;
  if (lastSnapshotLoadKey === loadKey && (loadingRunSnapshotMessages || runSnapshotPage !== null)) {
    return;
  }

  lastSnapshotLoadKey = loadKey;
  loadingRunSnapshotMessages = true;
  runSnapshotError = "";
  try {
    const page = await listAnalysisRunMessages({
      runId,
      after: null,
      limit: 50,
    });
    if (!currentRun || currentRun.id !== runId) {
      return;
    }
    applySnapshotPage(currentRun, page, false);
  } catch (error) {
    if (!currentRun || currentRun.id !== runId) {
      return;
    }
    runSnapshotMessages = [];
    runSnapshotCursor = null;
    runSnapshotHasMore = false;
    runSnapshotPage = null;
    runSnapshotError = formatAppError("loading run snapshot", error);
    runSnapshotAvailability = runSnapshotAvailabilityFromPage({
      currentRun,
      page: null,
      loading: false,
      errorMessage: runSnapshotError,
    });
  } finally {
    if (currentRun?.id === runId) {
      loadingRunSnapshotMessages = false;
    }
  }
}

async function loadMoreRunSnapshotMessages() {
  const run = currentRun;
  if (!run || !runSnapshotCursor || loadingRunSnapshotMessages) {
    return;
  }

  const runId = run.id;
  loadingRunSnapshotMessages = true;
  runSnapshotError = "";
  try {
    const page = await listAnalysisRunMessages({
      runId,
      after: runSnapshotCursor,
      limit: 50,
    });
    if (!currentRun || currentRun.id !== runId) {
      return;
    }
    applySnapshotPage(currentRun, page, true);
  } catch (error) {
    if (!currentRun || currentRun.id !== runId) {
      return;
    }
    runSnapshotError = formatAppError("loading more run snapshot messages", error);
  } finally {
    if (currentRun?.id === runId) {
      loadingRunSnapshotMessages = false;
    }
  }
}
```

This loader derives availability from the snapshot-only API response plus run state. It does not use run status alone.

- [ ] **Step 5: Reset snapshot state when the opened run changes**

Add an effect:

```ts
$effect(() => {
  const runId = currentRun?.id ?? null;
  if (runId === null) {
    resetRunSnapshotState();
    return;
  }

  const key = `run:${runId}`;
  if (!lastSnapshotLoadKey.startsWith(`${runId}:`) && lastSnapshotLoadKey !== "") {
    resetRunSnapshotState();
  }
});
```

If this effect conflicts with Svelte's dependency model after implementation, replace `lastSnapshotLoadKey` with a separate `snapshotStateRunId: number | null` and reset when it differs from `currentRun?.id`.

- [ ] **Step 6: Load the first snapshot page only when needed**

Add an effect:

```ts
$effect(() => {
  if (
    currentRun &&
    workspaceUiState.canvasMode === "source" &&
    workspaceUiState.sourceViewBasis === "run_snapshot"
  ) {
    void loadRunSnapshotFirstPage(currentRun.id);
  }
});
```

Do not add any effect that changes `sourceViewBasis` from `"live_source"` to `"run_snapshot"` after a snapshot becomes available. The user may explicitly browse live source while an opened run remains bound.

- [ ] **Step 7: Replace `<WorkspaceMain ... />` with `<ReportCanvas ... />`**

Replace the existing `<WorkspaceMain ... />` block with `<ReportCanvas ... />`.

Keep all current central props and callbacks from `WorkspaceMain`, then add these props and callbacks:

```svelte
    canvasMode={workspaceUiState.canvasMode}
    sourceViewBasis={workspaceUiState.sourceViewBasis}
    {runSnapshotAvailability}
    {runSnapshotMessages}
    {loadingRunSnapshotMessages}
    {runSnapshotError}
    hasMoreRunSnapshotMessages={runSnapshotHasMore}
    onChangeCanvasMode={(mode) => changeCanvasMode(mode)}
    onViewLiveSource={() => viewLiveSourceForOpenedRun()}
    onBackToRunSnapshot={() => backToRunSnapshot()}
    onLoadMoreRunSnapshotMessages={() => void loadMoreRunSnapshotMessages()}
```

Do not change `WorkspaceInspector` in this part.

- [ ] **Step 8: Run route tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-report-canvas-route.test.ts src/lib/analysis-route-workspace-state.test.ts src/lib/analysis-report-canvas-state.test.ts
```

Expected: PASS.

- [ ] **Step 9: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: PASS.

- [ ] **Step 10: Commit route wiring**

Run:

```powershell
git add src/routes/analysis/+page.svelte src/lib/analysis-report-canvas-route.test.ts
git commit -m "feat: wire report canvas into analysis"
```

## Task 8: Validate Behavior Boundaries

**Files:**
- Verify files changed in Tasks 1-7.

- [ ] **Step 1: Run focused report canvas tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-report-canvas-state.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-report-canvas-route.test.ts
```

Expected: PASS.

- [ ] **Step 2: Run workspace state and persistence tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-workspace-state.test.ts src/lib/analysis-workspace-persistence.test.ts src/lib/analysis-route-workspace-state.test.ts
```

Expected: PASS.

- [ ] **Step 3: Run existing analysis workflow tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-state.test.ts src/lib/analysis-run-workflow.test.ts src/lib/analysis-chat-workflow.test.ts src/lib/analysis-trace-workflow.test.ts src/lib/analysis-source-groups-workflow.test.ts
```

Expected: PASS.

- [ ] **Step 4: Run Svelte and TypeScript checks**

Run:

```powershell
npm.cmd run check
```

Expected: PASS.

- [ ] **Step 5: Run the full frontend test suite**

Run:

```powershell
npm.cmd test
```

Expected: PASS.

- [ ] **Step 6: Check whitespace**

Run:

```powershell
git diff --check
```

Expected: no output and exit code 0.

- [ ] **Step 7: Check staged redesign boundaries**

Run:

```powershell
rg -n "RunCompanionTabs|<WorkspaceMain|import WorkspaceMain" src/routes/analysis src/lib/components/analysis src/lib/analysis-report-canvas*.test.ts
```

Expected:

- no `RunCompanionTabs`;
- no `<WorkspaceMain` or `import WorkspaceMain` in `src/routes/analysis/+page.svelte`;
- `workspace-main.svelte` may still exist as an unused legacy file.

Run:

```powershell
rg -n "sourceViewBasis: \"run_snapshot\", // automatic|listSourceItems\\(\\{ runId|WorkspaceRail" src/routes/analysis/+page.svelte src/lib/components/analysis
```

Expected: no output. The route must not auto-switch live source back to snapshot and must not use live source APIs for run snapshot browsing.

- [ ] **Step 8: Commit final fixes if needed**

If verification required fixes, commit them:

```powershell
git add src
git commit -m "test: verify analysis report canvas"
```

Skip this commit if Tasks 1-7 already have a clean verified tree.

- [ ] **Step 9: Stop for review**

Run:

```powershell
git status --short
```

Expected: clean working tree.

Report:

```text
Part 4 report canvas is implemented and verified. Stopping before Part 5.
```

Do not begin Part 5 until the user explicitly approves continuing.

## Self-Review

- Spec coverage: this plan covers the approved `ReportCanvas` responsibilities, explicit `Report | Source` switching, no-run setup behavior, opened-run metadata, snapshot basis labels, pending/unavailable snapshot states, explicit live source browsing, and bounded snapshot-message paging.
- Boundary check: this plan intentionally does not create `RunCompanionTabs`, final Telegram source readers, final YouTube readers, or companion evidence/chat migration. It keeps `WorkspaceInspector` as the right panel and keeps `WorkspaceMain` only as an unused legacy rollback file.
- Snapshot trust: this plan uses Part 1 `listAnalysisRunMessages` for snapshot-only browsing. It derives availability from an actual snapshot API response, loading state, and errors, not from status alone.
- Transition safety: selecting `Report | Source` only changes `workspaceUiState.canvasMode`. Choosing `View live source` only changes `sourceViewBasis` explicitly. No effect silently substitutes live source data for a missing run snapshot.
- Test coverage: focused tests cover helper decisions, component structure, route wiring, setup placement, metadata requirements, snapshot/live basis controls, and absence of `RunCompanionTabs`.
