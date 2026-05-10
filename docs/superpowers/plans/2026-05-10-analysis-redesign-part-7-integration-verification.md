# Analysis Result-First Redesign Part 7 Integration Verification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the final cross-part integration verification layer for the `/analysis` result-first redesign after Parts 1 through 6 are implemented.

**Architecture:** This part does not introduce a new product surface. It adds high-signal integration tests, raw-source contract tests, snapshot-safety checks, and a repeatable browser verification record that proves `CompactSourceRail | ReportCanvas | RunCompanionTabs` behaves as one workspace.

**Tech Stack:** SvelteKit 2, Svelte 5 runes, TypeScript, Vitest, raw-source tests, Tauri 2/Rust test commands, Chrome DevTools or Playwright-style manual browser verification, existing Extractum analysis modules from Parts 1-6.

---

## Prerequisites

Implement this part only after Parts 1 through 6 are implemented, verified, and committed.

This plan assumes these files already exist from earlier parts:

- `src/lib/analysis-workspace-state.ts`
- `src/lib/analysis-workspace-persistence.ts`
- `src/lib/analysis-report-canvas-state.ts`
- `src/lib/analysis-run-companion-state.ts`
- `src/lib/source-reader-model.ts`
- `src/lib/components/analysis/compact-source-rail.svelte`
- `src/lib/components/analysis/source-switcher-panel.svelte`
- `src/lib/components/analysis/report-canvas.svelte`
- `src/lib/components/analysis/report-source-surface.svelte`
- `src/lib/components/analysis/report-run-header.svelte`
- `src/lib/components/analysis/report-setup-panel.svelte`
- `src/lib/components/analysis/telegram-timeline-reader.svelte`
- `src/lib/components/analysis/youtube-transcript-reader.svelte`
- `src/lib/components/analysis/youtube-playlist-reader.svelte`
- `src/lib/components/analysis/source-group-reader.svelte`
- `src/lib/components/analysis/run-companion-tabs.svelte`
- `src/lib/components/analysis/run-evidence-tab.svelte`
- `src/lib/components/analysis/run-chat-tab.svelte`
- `src/lib/components/analysis/run-companion-runs-tab.svelte`
- `src/routes/analysis/+page.svelte`

If any prerequisite is missing, stop and implement the earlier part first.

This is **Part 7 of 7**. Stop after this part is implemented, verified, and committed. Do not start any redesign follow-up work without a new explicit request.

## Part Boundary

Part 7 may:

- add final integration tests that compose helpers from Parts 1, 2, 4, and 6;
- add raw-source route/component tests that prove the old wide workspace has been replaced by the approved three-zone layout;
- add raw-source safety tests for completed-run snapshot behavior, source ingest placement, and no live-source fallback for completed evidence/chat;
- add a verification checklist artifact under `docs/superpowers/verification/`;
- run focused, full frontend, full backend, Svelte check, and browser smoke verification;
- make small fixes only when they are required for a Part 1-6 contract to pass.

Part 7 must not:

- add new analysis product features;
- redesign the global app shell or global app sidebar;
- add a fourth `/analysis` workspace zone;
- reopen the old `WorkspaceRail`, `WorkspaceMain`, or `WorkspaceInspector` as active route surfaces;
- change the saved-run immutability model;
- persist `OpenRunState`, selected trace refs, chat drafts, open popovers, source filters, or scroll positions;
- make completed-run evidence or chat use live source data as replacement context;
- place source ingest jobs in `RunCompanionTabs.Runs`;
- introduce a Playwright dependency unless the project already added one in a previous part.

## File Structure

- Create: `src/lib/analysis-redesign-workflow-scenarios.test.ts`
  - Responsibility: integration-style helper tests for completed, active, failed/cancelled, workspace switch, source-basis, chat, evidence, runs filter, and persistence normalization workflows.
- Create: `src/lib/analysis-redesign-route-contract.test.ts`
  - Responsibility: raw-source coverage that `/analysis/+page.svelte` renders `CompactSourceRail`, `ReportCanvas`, and `RunCompanionTabs`, and no longer renders the legacy route surfaces.
- Create: `src/lib/analysis-redesign-safety-contract.test.ts`
  - Responsibility: raw-source coverage for snapshot/live-source safety, source ingest placement, source reader media limits, companion tab constraints, and deleted/missing scope labeling hooks.
- Create: `docs/superpowers/verification/2026-05-10-analysis-redesign.md`
  - Responsibility: a durable verification record with exact automated commands, browser viewport checks, observed status, and remaining risks.
- Modify only if a Part 7 test exposes a real gap: the smallest responsible file from Parts 1-6.

## Task 1: Add Final Workflow Scenario Tests

**Files:**
- Create: `src/lib/analysis-redesign-workflow-scenarios.test.ts`

- [x] **Step 1: Create the final workflow scenario test file**

Create `src/lib/analysis-redesign-workflow-scenarios.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  defaultAnalysisWorkspaceUiState,
  openRunWorkspaceState,
  selectSourceGroupWorkspace,
  selectSourceWorkspace,
  type WorkspaceSelection,
} from "./analysis-workspace-state";
import {
  restoredUiStateFromPersisted,
  type PersistedAnalysisWorkspaceState,
} from "./analysis-workspace-persistence";
import {
  runSnapshotAvailabilityFromPage,
  sourceBasisLabel,
  sourceCanvasSurface,
  type RunSnapshotAvailability,
} from "./analysis-report-canvas-state";
import {
  chatAvailabilityForRun,
  evidenceSourceActionDecision,
  filterCompanionRuns,
  runsFilterDefaults,
} from "./analysis-run-companion-state";
import type {
  AnalysisRunDetail,
  AnalysisRunMessagesPage,
  AnalysisRunSummary,
  AnalysisTraceRef,
} from "./types/analysis";

function run(overrides: Partial<AnalysisRunDetail> = {}): AnalysisRunDetail {
  return {
    id: 42,
    run_type: "report",
    scope_type: "single_source",
    source_id: 7,
    source_title: "Telegram A",
    source_group_id: null,
    source_group_name: null,
    scope_label: "Telegram A",
    scope_label_snapshot: "Telegram A at run time",
    period_from: 1710000000,
    period_to: 1710100000,
    output_language: "Russian",
    prompt_template_id: 3,
    prompt_template_name: "Weekly",
    prompt_template_version: 5,
    provider_profile: "default",
    provider: "openai",
    model: "gpt-5.4",
    youtube_corpus_mode: "transcript_description_comments",
    status: "completed",
    result_markdown: "Saved report",
    error: null,
    has_trace_data: true,
    created_at: 1710100010,
    completed_at: 1710100100,
    ...overrides,
  };
}

function summary(overrides: Partial<AnalysisRunSummary> = {}): AnalysisRunSummary {
  return {
    id: 42,
    run_type: "report",
    scope_type: "single_source",
    source_id: 7,
    source_title: "Telegram A",
    source_group_id: null,
    source_group_name: null,
    scope_label: "Telegram A",
    period_from: 1710000000,
    period_to: 1710100000,
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
    created_at: 1710100010,
    completed_at: 1710100100,
    ...overrides,
  };
}

function trace(overrides: Partial<AnalysisTraceRef> = {}): AnalysisTraceRef {
  return {
    ref: "s7-i11",
    item_id: 11,
    source_id: 7,
    external_id: "11",
    published_at: 1710000020,
    excerpt: "Saved excerpt",
    youtube_url: null,
    youtube_timestamp_seconds: null,
    youtube_display_label: null,
    is_synthetic: false,
    ...overrides,
  };
}

function snapshotPage(messageCount: number): AnalysisRunMessagesPage {
  return {
    messages: Array.from({ length: messageCount }, (_, index) => ({
      item_id: index + 1,
      source_id: 7,
      external_id: `m-${index + 1}`,
      author: "Alice",
      published_at: 1710000000 + index,
      ref: `s7-i${index + 1}`,
      content: `Message ${index + 1}`,
      item_kind: "telegram_message",
      source_type: "telegram",
      source_subtype: "channel",
      metadata_json: null,
    })),
    next_cursor: null,
    has_more: false,
  };
}

describe("analysis redesign final workflow scenarios", () => {
  it("opens a completed run as report-first, evidence-first, and snapshot-backed", () => {
    const opened = openRunWorkspaceState(defaultAnalysisWorkspaceUiState(), {
      runId: 42,
      status: "completed",
      sourceId: 7,
      sourceGroupId: null,
      liveScopeExists: true,
    });
    const snapshotAvailability = runSnapshotAvailabilityFromPage({
      currentRun: run(),
      page: snapshotPage(2),
      loading: false,
      errorMessage: "",
    });

    expect(opened.workspaceSelection).toEqual({ kind: "source", sourceId: 7 });
    expect(opened.openRunState).toEqual({ kind: "saved", runId: 42 });
    expect(opened.canvasMode).toBe("report");
    expect(opened.sourceViewBasis).toBe("run_snapshot");
    expect(opened.companionTab).toBe("evidence");
    expect(snapshotAvailability).toBe("available");
    expect(sourceCanvasSurface({
      currentRun: run(),
      sourceViewBasis: opened.sourceViewBasis,
      snapshotAvailability,
    })).toBe("run_snapshot");
    expect(chatAvailabilityForRun({
      currentRun: run(),
      snapshotAvailability,
    })).toMatchObject({ enabled: true, reason: "enabled" });
    expect(evidenceSourceActionDecision({
      currentRun: run(),
      selectedTrace: trace(),
      snapshotAvailability,
    })).toMatchObject({
      kind: "run_snapshot",
      canvasMode: "source",
      sourceViewBasis: "run_snapshot",
      highlightedRef: "s7-i11",
    });
  });

  it("selecting a source clears run-bound state and makes Runs current-scope filtering meaningful", () => {
    const opened = openRunWorkspaceState(defaultAnalysisWorkspaceUiState(), {
      runId: 42,
      status: "completed",
      sourceId: 7,
      sourceGroupId: null,
      liveScopeExists: true,
    });
    const selected = selectSourceWorkspace(
      {
        ...opened,
        companionTab: "chat",
        selectedTraceRef: "s7-i11",
      },
      8,
    );
    const filtered = filterCompanionRuns({
      activeRuns: [
        summary({ id: 1, status: "running", source_id: 8, source_title: "Telegram B" }),
        summary({ id: 2, status: "running", source_id: 7, source_title: "Telegram A" }),
      ],
      savedRuns: [
        summary({ id: 3, status: "completed", source_id: 8, source_title: "Telegram B" }),
        summary({ id: 4, status: "completed", source_id: 9, source_title: "Telegram C" }),
      ],
      filter: {
        ...runsFilterDefaults(),
        scope: "current",
      },
      workspaceSelection: selected.workspaceSelection,
    });

    expect(selected).toEqual({
      workspaceSelection: { kind: "source", sourceId: 8 },
      openRunState: { kind: "none" },
      canvasMode: "source",
      sourceViewBasis: "live_source",
      companionTab: "runs",
      selectedTraceRef: null,
    });
    expect(filtered.map((entry) => entry.run.id)).toEqual([1, 3]);
  });

  it("selecting a source group clears run-bound state without opening a pseudo-chat merge", () => {
    const selected = selectSourceGroupWorkspace(
      {
        ...defaultAnalysisWorkspaceUiState(),
        openRunState: { kind: "saved", runId: 42 },
        canvasMode: "report",
        sourceViewBasis: "run_snapshot",
        companionTab: "evidence",
        selectedTraceRef: "s7-i11",
      },
      12,
    );

    expect(selected.workspaceSelection).toEqual({ kind: "source_group", sourceGroupId: 12 });
    expect(selected.openRunState).toEqual({ kind: "none" });
    expect(selected.canvasMode).toBe("source");
    expect(selected.sourceViewBasis).toBe("live_source");
    expect(selected.companionTab).toBe("runs");
    expect(selected.selectedTraceRef).toBeNull();
  });

  it("keeps active runs honest while snapshot capture is pending", () => {
    const currentRun = run({
      status: "running",
      completed_at: null,
      result_markdown: null,
    });
    const opened = openRunWorkspaceState(defaultAnalysisWorkspaceUiState(), {
      runId: currentRun.id,
      status: currentRun.status,
      sourceId: currentRun.source_id,
      sourceGroupId: currentRun.source_group_id,
      liveScopeExists: true,
    });
    const snapshotAvailability = runSnapshotAvailabilityFromPage({
      currentRun,
      page: snapshotPage(0),
      loading: false,
      errorMessage: "",
    });

    expect(opened.openRunState).toEqual({ kind: "active", runId: currentRun.id });
    expect(opened.canvasMode).toBe("report");
    expect(opened.companionTab).toBe("runs");
    expect(snapshotAvailability).toBe("capturing");
    expect(sourceCanvasSurface({
      currentRun,
      sourceViewBasis: "run_snapshot",
      snapshotAvailability,
    })).toBe("snapshot_pending");
    expect(chatAvailabilityForRun({
      currentRun,
      snapshotAvailability,
    })).toMatchObject({ enabled: false, reason: "pending_completion" });
  });

  it("does not resolve completed-run evidence or chat against live source when snapshot rows are missing", () => {
    const currentRun = run({ status: "completed" });
    const snapshotAvailability: RunSnapshotAvailability = "unavailable";

    expect(sourceCanvasSurface({
      currentRun,
      sourceViewBasis: "run_snapshot",
      snapshotAvailability,
    })).toBe("snapshot_unavailable");
    expect(sourceBasisLabel({
      currentRun,
      sourceViewBasis: "live_source",
      snapshotAvailability,
    })).toContain("Live source");
    expect(chatAvailabilityForRun({
      currentRun,
      snapshotAvailability,
    })).toMatchObject({ enabled: false, reason: "missing_snapshot" });
    expect(evidenceSourceActionDecision({
      currentRun,
      selectedTrace: trace(),
      snapshotAvailability,
    })).toMatchObject({
      kind: "unavailable",
      reason: expect.stringContaining("completed run has no saved snapshot rows"),
    });
  });

  it("keeps saved runs with deleted live scope openable without faking rail selection", () => {
    const opened = openRunWorkspaceState(
      {
        ...defaultAnalysisWorkspaceUiState(),
        workspaceSelection: { kind: "source", sourceId: 99 },
      },
      {
        runId: 42,
        status: "completed",
        sourceId: 7,
        sourceGroupId: null,
        liveScopeExists: false,
      },
    );

    expect(opened.openRunState).toEqual({ kind: "saved", runId: 42 });
    expect(opened.workspaceSelection).toEqual({ kind: "none" });
    expect(opened.canvasMode).toBe("report");
    expect(opened.companionTab).toBe("evidence");
  });

  it("normalizes persisted run-bound UI because OpenRunState is never restored", () => {
    const persisted: PersistedAnalysisWorkspaceState = {
      version: 1,
      workspaceSelection: { kind: "source", sourceId: 7 } satisfies WorkspaceSelection,
      canvasMode: "report",
      sourceViewBasis: "run_snapshot",
      companionTab: "chat",
      runs: {
        historyScope: "current",
        runFilter: "completed",
        runsFilter: {
          ...runsFilterDefaults(),
          query: "weekly",
          scope: "current",
          status: "completed",
        },
      },
    };

    const restored = restoredUiStateFromPersisted(persisted, {
      sources: [{ id: 7 }],
      groups: [],
    });

    expect(restored.workspaceSelection).toEqual({ kind: "source", sourceId: 7 });
    expect(restored.openRunState).toEqual({ kind: "none" });
    expect(restored.canvasMode).toBe("report");
    expect(restored.sourceViewBasis).toBe("live_source");
    expect(restored.companionTab).toBe("runs");
    expect(restored.selectedTraceRef).toBeNull();
  });
});
```

- [x] **Step 2: Run the workflow tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-redesign-workflow-scenarios.test.ts
```

Expected: PASS after Parts 1-6 are correctly implemented. If the test fails, repair the named helper contract in the smallest responsible Part 1-6 file, then rerun this exact command.

- [x] **Step 3: Commit the workflow tests**

Run:

```powershell
git add src/lib/analysis-redesign-workflow-scenarios.test.ts
git commit -m "test: add analysis redesign workflow scenarios"
```

## Task 2: Add Final Route And Layout Contract Tests

**Files:**
- Create: `src/lib/analysis-redesign-route-contract.test.ts`

- [x] **Step 1: Create the raw-source route contract test file**

Create `src/lib/analysis-redesign-route-contract.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";
import compactRailSource from "./components/analysis/compact-source-rail.svelte?raw";
import reportCanvasSource from "./components/analysis/report-canvas.svelte?raw";
import reportSetupSource from "./components/analysis/report-setup-panel.svelte?raw";
import reportRunHeaderSource from "./components/analysis/report-run-header.svelte?raw";
import runCompanionSource from "./components/analysis/run-companion-tabs.svelte?raw";
import runsTabSource from "./components/analysis/run-companion-runs-tab.svelte?raw";

describe("analysis redesign final route contract", () => {
  it("renders the approved three-zone analysis workspace", () => {
    expect(analysisPageSource).toContain(
      'import CompactSourceRail from "$lib/components/analysis/compact-source-rail.svelte";',
    );
    expect(analysisPageSource).toContain(
      'import ReportCanvas from "$lib/components/analysis/report-canvas.svelte";',
    );
    expect(analysisPageSource).toContain(
      'import RunCompanionTabs from "$lib/components/analysis/run-companion-tabs.svelte";',
    );
    expect(analysisPageSource).toContain("<CompactSourceRail");
    expect(analysisPageSource).toContain("<ReportCanvas");
    expect(analysisPageSource).toContain("<RunCompanionTabs");
    expect(analysisPageSource).toContain("analysis-workspace-grid");
    expect(analysisPageSource).toContain("rail-slot");
    expect(analysisPageSource).toContain("canvas-slot");
    expect(analysisPageSource).toContain("companion-slot");
  });

  it("does not render the legacy wide analysis workspace surfaces", () => {
    expect(analysisPageSource).not.toContain(
      'import WorkspaceRail from "$lib/components/analysis/workspace-rail.svelte";',
    );
    expect(analysisPageSource).not.toContain(
      'import WorkspaceMain from "$lib/components/analysis/workspace-main.svelte";',
    );
    expect(analysisPageSource).not.toContain(
      'import WorkspaceInspector from "$lib/components/analysis/workspace-inspector.svelte";',
    );
    expect(analysisPageSource).not.toContain("<WorkspaceRail");
    expect(analysisPageSource).not.toContain("<WorkspaceMain");
    expect(analysisPageSource).not.toContain("<WorkspaceInspector");
    expect(analysisPageSource).not.toContain("inspectorMode");
  });

  it("keeps source switching, run opening, and canvas switching on separate state paths", () => {
    expect(analysisPageSource).toContain("selectSourceWorkspace");
    expect(analysisPageSource).toContain("selectSourceGroupWorkspace");
    expect(analysisPageSource).toContain("openRunWorkspaceState");
    expect(analysisPageSource).toContain("workspaceUiState.canvasMode");
    expect(analysisPageSource).toContain("workspaceUiState.sourceViewBasis");
    expect(analysisPageSource).toContain("workspaceUiState.companionTab");
    expect(analysisPageSource).toContain("function changeCanvasMode");
    expect(analysisPageSource).toContain("function viewLiveSource");
    expect(analysisPageSource).toContain("function backToRunSnapshot");
    expect(analysisPageSource).not.toContain("clearCurrentRunForCanvasSwitch");
    expect(analysisPageSource).not.toContain("clearCurrentRunForSourceFilter");
  });

  it("keeps the collapsed rail source-scoped and quiet", () => {
    expect(compactRailSource).toContain('class="compact-source-rail"');
    expect(compactRailSource).toContain("workspaceSelection: WorkspaceSelection");
    expect(compactRailSource).toContain('ariaLabel="Open source switcher"');
    expect(compactRailSource).toContain("context-primary-action");
    expect(compactRailSource).toContain("criticalSourceStatus");
    expect(compactRailSource).not.toContain("AppSidebar");
    expect(compactRailSource).not.toContain("Settings");
    expect(compactRailSource).not.toContain("Accounts");
    expect(compactRailSource).not.toContain("Manage sources</Button>");
    expect(compactRailSource).not.toContain("Transcript unavailable");
    expect(compactRailSource).not.toContain("Comments unavailable");
  });

  it("keeps ReportCanvas as the report/source mode owner", () => {
    expect(reportCanvasSource).toContain('class="report-canvas"');
    expect(reportCanvasSource).toContain('role="tablist"');
    expect(reportCanvasSource).toContain('onChangeCanvasMode("report")');
    expect(reportCanvasSource).toContain('onChangeCanvasMode("source")');
    expect(reportCanvasSource).toContain("<ReportRunHeader");
    expect(reportCanvasSource).toContain("<ReportSetupPanel");
    expect(reportCanvasSource).toContain("<ReportSourceSurface");
    expect(reportCanvasSource).not.toContain("<RunCompanionTabs");
    expect(reportCanvasSource).not.toContain("<WorkspaceInspector");
  });

  it("keeps report setup out of the primary opened-run reading surface", () => {
    expect(reportCanvasSource).toContain("currentRun");
    expect(reportCanvasSource).toContain("!currentRun");
    expect(reportSetupSource).toContain("template");
    expect(reportSetupSource).toContain("Run report");
    expect(reportSetupSource).toContain("dialog");
    expect(reportRunHeaderSource).toContain("prompt_template_name");
    expect(reportRunHeaderSource).toContain("prompt_template_version");
    expect(reportRunHeaderSource).toContain("provider_profile");
    expect(reportRunHeaderSource).toContain("youtube_corpus_mode");
    expect(reportRunHeaderSource).not.toContain("Template editor");
  });

  it("keeps companion tabs as Evidence, Chat, and Runs only", () => {
    expect(runCompanionSource).toContain('role="tablist"');
    expect(runCompanionSource).toContain('aria-label="Run companion tabs"');
    expect(runCompanionSource).toContain('onChangeCompanionTab("evidence")');
    expect(runCompanionSource).toContain('onChangeCompanionTab("chat")');
    expect(runCompanionSource).toContain('onChangeCompanionTab("runs")');
    expect(runCompanionSource).toContain("<RunEvidenceTab");
    expect(runCompanionSource).toContain("<RunChatTab");
    expect(runCompanionSource).toContain("<RunCompanionRunsTab");
    expect(runCompanionSource).not.toContain("Source activity");
  });

  it("keeps Runs focused on analysis report runs and durable filters", () => {
    expect(runsTabSource).toContain("filterCompanionRuns");
    expect(runsTabSource).toContain("Search runs");
    expect(runsTabSource).toContain("Current scope");
    expect(runsTabSource).toContain("Date range");
    expect(runsTabSource).toContain("Provider filter");
    expect(runsTabSource).toContain("Template filter");
    expect(runsTabSource).not.toContain("SourceJobRecord");
    expect(runsTabSource).not.toContain("takeoutJobs");
    expect(runsTabSource).not.toContain("sourceJobs");
  });
});
```

- [x] **Step 2: Run the route contract tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-redesign-route-contract.test.ts
```

Expected: PASS. If this fails because a raw-source marker has a different final name, keep the assertion intent and update only the string to the actual implemented public contract.

- [x] **Step 3: Commit the route contract tests**

Run:

```powershell
git add src/lib/analysis-redesign-route-contract.test.ts
git commit -m "test: add analysis redesign route contract"
```

## Task 3: Add Final Snapshot, Source, And Ingest Safety Tests

**Files:**
- Create: `src/lib/analysis-redesign-safety-contract.test.ts`

- [x] **Step 1: Create the raw-source safety contract test file**

Create `src/lib/analysis-redesign-safety-contract.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";
import reportSourceSurfaceSource from "./components/analysis/report-source-surface.svelte?raw";
import sourceReaderHeaderSource from "./components/analysis/source-reader-header.svelte?raw";
import telegramMediaCardSource from "./components/analysis/telegram-media-card.svelte?raw";
import telegramTimelineSource from "./components/analysis/telegram-timeline-reader.svelte?raw";
import youtubeTranscriptSource from "./components/analysis/youtube-transcript-reader.svelte?raw";
import youtubePlaylistSource from "./components/analysis/youtube-playlist-reader.svelte?raw";
import sourceGroupReaderSource from "./components/analysis/source-group-reader.svelte?raw";
import evidenceTabSource from "./components/analysis/run-evidence-tab.svelte?raw";
import chatTabSource from "./components/analysis/run-chat-tab.svelte?raw";
import runsTabSource from "./components/analysis/run-companion-runs-tab.svelte?raw";
import reportRunHeaderSource from "./components/analysis/report-run-header.svelte?raw";
import chatBackendSource from "../../src-tauri/src/analysis/chat.rs?raw";
import corpusBackendSource from "../../src-tauri/src/analysis/corpus.rs?raw";

describe("analysis redesign final safety contract", () => {
  it("keeps run snapshot and live source basis explicit in Source mode", () => {
    expect(reportSourceSurfaceSource).toContain("sourceViewBasis");
    expect(reportSourceSurfaceSource).toContain("run_snapshot");
    expect(reportSourceSurfaceSource).toContain("live_source");
    expect(sourceReaderHeaderSource).toContain("Live source");
    expect(sourceReaderHeaderSource).toContain("Run snapshot");
    expect(sourceReaderHeaderSource).toContain("View live source");
    expect(sourceReaderHeaderSource).toContain("Back to run snapshot");
    expect(reportSourceSurfaceSource).not.toContain("sourceViewBasis = \"run_snapshot\"");
    expect(analysisPageSource).not.toContain("autoSwitchToRunSnapshot");
  });

  it("does not resolve completed-run evidence through live source fallback", () => {
    expect(evidenceTabSource).toContain("evidenceSourceActionDecision");
    expect(evidenceTabSource).toContain("Show in source");
    expect(evidenceTabSource).toContain("Snapshot unavailable");
    expect(evidenceTabSource).not.toContain("listSourceItems");
    expect(corpusBackendSource).toContain("list_analysis_run_messages");
    expect(corpusBackendSource).toContain("analysis_run_messages");
    expect(corpusBackendSource).not.toContain("completed_run_live_source_fallback");
  });

  it("gates completed-run chat on saved run context instead of live source context", () => {
    expect(chatTabSource).toContain("chatAvailability");
    expect(chatTabSource).toContain("completed");
    expect(chatTabSource).toContain("snapshot");
    expect(chatTabSource).not.toContain("onfocus");
    expect(chatTabSource).not.toContain("onFocus");
    expect(chatBackendSource).toContain("analysis_run_messages");
    expect(chatBackendSource).toContain("completed");
    expect(chatBackendSource).not.toContain("load_run_corpus_messages(&pool, &run)");
  });

  it("keeps source ingest activity out of analysis Runs", () => {
    expect(runsTabSource).not.toContain("SourceJobRecord");
    expect(runsTabSource).not.toContain("sourceJobs");
    expect(runsTabSource).not.toContain("takeoutJobs");
    expect(runsTabSource).not.toContain("Sync transcript");
    expect(runsTabSource).not.toContain("Takeout import");
    expect(reportSourceSurfaceSource).toContain("Source activity");
    expect(reportSourceSurfaceSource).toContain("Live source actions");
  });

  it("renders Telegram source material as metadata-first timeline without binary previews", () => {
    expect(telegramTimelineSource).toContain('class="telegram-timeline-reader"');
    expect(telegramTimelineSource).toContain("topicLabel");
    expect(telegramTimelineSource).toContain("replyLabel");
    expect(telegramTimelineSource).toContain("reactionLabel");
    expect(telegramTimelineSource).toContain("<TelegramMediaCard");
    expect(telegramMediaCardSource).toContain("media.fileName");
    expect(telegramMediaCardSource).toContain("media.mimeType");
    expect(telegramMediaCardSource).not.toContain("<img");
    expect(telegramMediaCardSource).not.toContain("<video");
    expect(telegramMediaCardSource).not.toContain("<audio");
  });

  it("renders YouTube source material as transcript and playlist readers without an embedded player", () => {
    expect(youtubeTranscriptSource).toContain('class="youtube-transcript-reader"');
    expect(youtubeTranscriptSource).toContain("Search transcript");
    expect(youtubeTranscriptSource).toContain("Copy timestamp link");
    expect(youtubeTranscriptSource).toContain("youtubeTimestampUrl");
    expect(youtubeTranscriptSource).not.toContain("<iframe");
    expect(youtubeTranscriptSource).not.toContain("<video");
    expect(youtubePlaylistSource).toContain('class="youtube-playlist-reader"');
    expect(youtubePlaylistSource).toContain("playlist.items");
    expect(youtubePlaylistSource).toContain("onOpenSource");
  });

  it("keeps source groups grouped by source instead of merged into one pseudo-chat", () => {
    expect(sourceGroupReaderSource).toContain('class="source-group-reader"');
    expect(sourceGroupReaderSource).toContain("groupReaderItemsBySource");
    expect(sourceGroupReaderSource).toContain("source-heading");
    expect(sourceGroupReaderSource).toContain("selectedGroupSourceId");
    expect(sourceGroupReaderSource).not.toContain("mergedTimeline");
    expect(sourceGroupReaderSource).not.toContain("pseudoChat");
  });

  it("keeps missing or deleted run scope labeling visible in the run header", () => {
    expect(reportRunHeaderSource).toContain("scope_label_snapshot");
    expect(reportRunHeaderSource).toContain("missing");
    expect(reportRunHeaderSource).toContain("deleted");
    expect(reportRunHeaderSource).toContain("source basis");
    expect(reportRunHeaderSource).toContain("youtube_corpus_mode");
  });
});
```

- [x] **Step 2: Run the safety tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-redesign-safety-contract.test.ts
```

Expected: PASS. If a backend raw-source assertion fails because the final Rust helper uses a different function name, preserve the safety property and update the assertion to the exact function or error string in the implemented file.

- [x] **Step 3: Commit the safety tests**

Run:

```powershell
git add src/lib/analysis-redesign-safety-contract.test.ts
git commit -m "test: add analysis redesign safety contract"
```

## Task 4: Add The Verification Record

**Files:**
- Create: `docs/superpowers/verification/2026-05-10-analysis-redesign.md`

- [x] **Step 1: Create the verification record**

Create `docs/superpowers/verification/2026-05-10-analysis-redesign.md`:

```markdown
# Analysis Result-First Redesign Verification

Date: 2026-05-10
Scope: `/analysis` result-first redesign, Parts 1-7

## Automated Verification

| Check | Command | Result | Notes |
| --- | --- | --- | --- |
| Final workflow scenarios | `npm.cmd test -- src/lib/analysis-redesign-workflow-scenarios.test.ts` | Pending automated execution | Result recorded in Task 5 |
| Final route contract | `npm.cmd test -- src/lib/analysis-redesign-route-contract.test.ts` | Pending automated execution | Result recorded in Task 5 |
| Final safety contract | `npm.cmd test -- src/lib/analysis-redesign-safety-contract.test.ts` | Pending automated execution | Result recorded in Task 5 |
| Focused redesign frontend tests | `npm.cmd test -- src/lib/analysis-workspace-state.test.ts src/lib/analysis-workspace-persistence.test.ts src/lib/analysis-route-workspace-state.test.ts src/lib/analysis-compact-source-rail.test.ts src/lib/analysis-source-access-placement.test.ts src/lib/analysis-report-canvas-state.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-report-canvas-route.test.ts src/lib/source-reader-model.test.ts src/lib/analysis-source-readers.test.ts src/lib/analysis-source-readers-route.test.ts src/lib/analysis-run-companion-state.test.ts src/lib/analysis-run-companion-tabs.test.ts src/lib/analysis-run-companion-route.test.ts src/lib/analysis-redesign-workflow-scenarios.test.ts src/lib/analysis-redesign-route-contract.test.ts src/lib/analysis-redesign-safety-contract.test.ts` | Pending automated execution | Result recorded in Task 5 |
| Svelte and TypeScript | `npm.cmd run check` | Pending automated execution | Result recorded in Task 5 |
| Full frontend suite | `npm.cmd test` | Pending automated execution | Result recorded in Task 5 |
| Full backend suite | `cargo test --manifest-path src-tauri/Cargo.toml` | Pending automated execution | Result recorded in Task 5 |
| Whitespace | `git diff --check` | Pending automated execution | Result recorded in Task 5 |

## Browser Smoke Verification

Run a dev server:

```powershell
npm.cmd run dev -- --host 127.0.0.1 --port 5173
```

Then verify `http://127.0.0.1:5173/analysis` in desktop, narrow desktop, and mobile-width viewports.

| Viewport | Result | Notes |
| --- | --- | --- |
| Desktop 1440x900 | Pending browser execution | Check three-zone layout, canvas dominance, no overlap |
| Wide desktop 1920x1080 | Pending browser execution | Check report readability and companion width |
| Narrow 1180x900 | Pending browser execution | Check companion fallback and compact rail usability |
| Mobile 390x844 | Pending browser execution | Check stacked canvas/companion and source switching access |

## Browser Scenarios

| Scenario | Result | Notes |
| --- | --- | --- |
| No source state shows central onboarding for Telegram and YouTube | Pending browser execution | No global-sidebar duplication in compact rail |
| Selecting source clears opened run and shows live source + Runs | Pending browser execution | Rail selection, canvas, companion aligned |
| Completed saved run opens Report + Evidence and aligns rail if live scope exists | Pending browser execution | Header metadata visible |
| Completed saved run with missing snapshot does not resolve evidence/chat against live source | Pending browser execution | Source mode shows unavailable snapshot state |
| Running run opens Report and Source shows pending snapshot | Pending browser execution | Chat disabled until completion |
| Failed/cancelled run shows snapshot if available, otherwise explicit live source option | Pending browser execution | Not visually styled as completed |
| Trace ref click activates Evidence | Pending browser execution | Focus/selection visible |
| Show in source prefers run snapshot and highlights message/segment | Pending browser execution | Live source clearly labeled when allowed for non-completed states |
| Chat tab activates only on explicit tab selection or question submit | Pending browser execution | Textarea focus alone does not switch tab |
| Runs search/status/scope filters work and exclude source ingest jobs | Pending browser execution | Current-scope filter updates after workspace switch |
| Telegram timeline shows groups, metadata, and media placeholders only | Pending browser execution | No binary previews |
| YouTube video reader shows transcript timestamps and copy/open actions | Pending browser execution | No embedded player |
| YouTube playlist reader shows playlist item list before transcript reading | Pending browser execution | Per-video source navigation reachable |
| Source group reader groups by source with counts | Pending browser execution | No pseudo-chat merge |
| Workspace persistence restores source/group and UI context without opening a run | Pending browser execution | Run-bound tabs normalize to Runs |

## Residual Risks

- Browser scenarios depend on local data fixtures. Record any missing fixture as a verification gap with the smallest reproducible setup.
- Raw-source tests intentionally protect architectural contracts. If they fail because implementation names changed, update the assertion string while preserving the tested behavior.
```

- [x] **Step 2: Commit the verification record template**

Run:

```powershell
git add docs/superpowers/verification/2026-05-10-analysis-redesign.md
git commit -m "docs: add analysis redesign verification record"
```

## Task 5: Run Automated Verification

**Files:**
- Verify all files touched by Parts 1-7.
- Update: `docs/superpowers/verification/2026-05-10-analysis-redesign.md`

- [x] **Step 1: Run the final Part 7 tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-redesign-workflow-scenarios.test.ts src/lib/analysis-redesign-route-contract.test.ts src/lib/analysis-redesign-safety-contract.test.ts
```

Expected: PASS.

- [x] **Step 2: Run focused redesign frontend tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-workspace-state.test.ts src/lib/analysis-workspace-persistence.test.ts src/lib/analysis-route-workspace-state.test.ts src/lib/analysis-compact-source-rail.test.ts src/lib/analysis-source-access-placement.test.ts src/lib/analysis-report-canvas-state.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-report-canvas-route.test.ts src/lib/source-reader-model.test.ts src/lib/analysis-source-readers.test.ts src/lib/analysis-source-readers-route.test.ts src/lib/analysis-run-companion-state.test.ts src/lib/analysis-run-companion-tabs.test.ts src/lib/analysis-run-companion-route.test.ts src/lib/analysis-redesign-workflow-scenarios.test.ts src/lib/analysis-redesign-route-contract.test.ts src/lib/analysis-redesign-safety-contract.test.ts
```

Expected: PASS.

- [x] **Step 3: Run Svelte and TypeScript checks**

Run:

```powershell
npm.cmd run check
```

Expected: PASS.

- [x] **Step 4: Run the full frontend test suite**

Run:

```powershell
npm.cmd test
```

Expected: PASS.

- [x] **Step 5: Run focused backend safety tests from earlier parts**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::chat::tests::completed_chat_context_requires_saved_snapshot_messages analysis::chat::tests::completed_chat_context_accepts_saved_snapshot_messages analysis::corpus::tests::trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot analysis::corpus::tests::list_run_snapshot_messages_page_reads_saved_snapshot_only youtube::transcript_reader::tests sources::items::query::tests
```

Expected: PASS.

- [x] **Step 6: Run the full backend test suite**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [x] **Step 7: Run final boundary searches**

Run:

```powershell
rg -n "WorkspaceRail|WorkspaceMain|WorkspaceInspector|inspectorMode|temporary-follow-up|clearCurrentRunForCanvasSwitch|autoSwitchToRunSnapshot|completed_run_live_source_fallback" src/routes/analysis/+page.svelte src/lib/components/analysis src/lib/analysis-*.ts src-tauri/src/analysis
```

Expected: no active route or active component usage of these legacy or unsafe markers. It is acceptable only if the match is in a test file whose assertion expects absence.

Run:

```powershell
rg -n "SourceJobRecord|sourceJobs|takeoutJobs|Takeout import|Sync transcript" src/lib/components/analysis/run-companion*.svelte src/lib/components/analysis/run-evidence-tab.svelte src/lib/components/analysis/run-chat-tab.svelte
```

Expected: no source ingest job types, props, or actions in companion tab components.

Run:

```powershell
rg -n "<iframe|<video|<audio|<img" src/lib/components/analysis/telegram-media-card.svelte src/lib/components/analysis/youtube-transcript-reader.svelte
```

Expected: no embedded players or binary media preview elements in Telegram media placeholders or YouTube transcript reader.

- [x] **Step 8: Check whitespace**

Run:

```powershell
git diff --check
```

Expected: no output and exit code 0.

- [x] **Step 9: Update the verification record with automated results**

In `docs/superpowers/verification/2026-05-10-analysis-redesign.md`, replace each `Pending automated execution` in the Automated Verification table with:

```text
PASS
```

For any failed command, write:

```text
FAIL: command exited 1; first failing assertion or compiler error is recorded in Notes
```

Do not mark a command `PASS` unless that exact command has been run in this task.

- [x] **Step 10: Commit automated verification record updates**

Run:

```powershell
git add docs/superpowers/verification/2026-05-10-analysis-redesign.md
git commit -m "docs: record analysis redesign automated verification"
```

## Task 6: Run Browser Verification

**Files:**
- Update: `docs/superpowers/verification/2026-05-10-analysis-redesign.md`

- [x] **Step 1: Start the local dev server**

Run:

```powershell
npm.cmd run dev -- --host 127.0.0.1 --port 5173
```

Expected: Vite prints a local URL containing `http://127.0.0.1:5173/`.

Keep this server running until the browser verification is complete.

- [x] **Step 2: Open `/analysis` at desktop width**

Use Chrome DevTools MCP or the available browser automation tool:

```text
Navigate to http://127.0.0.1:5173/analysis
Set viewport to 1440x900
Capture a screenshot or accessibility snapshot
```

Expected:

- `CompactSourceRail` is narrow and source-oriented;
- `ReportCanvas` is the dominant column;
- `RunCompanionTabs` is visible and usable;
- no text overlaps or incoherent clipping;
- global app sidebar remains the only app navigation.

- [x] **Step 3: Check wide desktop width**

Use Chrome DevTools MCP or the available browser automation tool:

```text
Set viewport to 1920x1080
Capture a screenshot or accessibility snapshot
```

Expected:

- report reading width is comfortable;
- companion panel is wide enough for evidence snippets and chat controls;
- compact rail does not expand into a full source manager;
- no page section is presented as a nested card inside another card.

- [x] **Step 4: Check narrow desktop width**

Use Chrome DevTools MCP or the available browser automation tool:

```text
Set viewport to 1180x900
Capture a screenshot or accessibility snapshot
```

Expected:

- compact rail remains accessible or moves to the intended responsive access point;
- canvas remains readable;
- companion is placed below or beside the canvas according to implemented CSS;
- `Report | Source` and `Evidence | Chat | Runs` controls remain reachable.

- [x] **Step 5: Check mobile width**

Use Chrome DevTools MCP or the available browser automation tool:

```text
Set viewport to 390x844
Capture a screenshot or accessibility snapshot
```

Expected:

- canvas and companion stack without overlap;
- source switching is reachable through compact top/rail/drawer access;
- `Report | Source` mode control is visible near the canvas title;
- tab labels do not overflow their buttons;
- report setup/onboarding text fits its containers.

- [x] **Step 6: Exercise the core browser scenarios**

Using the local data available in the development database, verify every scenario in the Browser Scenarios table of `docs/superpowers/verification/2026-05-10-analysis-redesign.md`.

For scenarios that cannot be exercised because the local database lacks that fixture, write:

```text
BLOCKED: missing local fixture; scenario name is the current table row
```

Do not mark a scenario `PASS` unless it was actually exercised in the browser.

- [x] **Step 7: Update the browser verification record**

In `docs/superpowers/verification/2026-05-10-analysis-redesign.md`, update the Browser Smoke Verification and Browser Scenarios tables with actual `PASS`, `FAIL`, or `BLOCKED` results from Steps 2-6.

- [x] **Step 8: Stop the dev server**

Stop the `npm.cmd run dev` process with `Ctrl+C`.

Expected: the dev server exits.

- [x] **Step 9: Commit browser verification results**

Run:

```powershell
git add docs/superpowers/verification/2026-05-10-analysis-redesign.md
git commit -m "docs: record analysis redesign browser verification"
```

## Task 7: Final Redesign Completion Gate

**Files:**
- Verify all Part 7 test and documentation files.

- [x] **Step 1: Run the final command set after browser verification updates**

Run:

```powershell
npm.cmd test -- src/lib/analysis-redesign-workflow-scenarios.test.ts src/lib/analysis-redesign-route-contract.test.ts src/lib/analysis-redesign-safety-contract.test.ts
npm.cmd run check
git diff --check
```

Expected: all commands pass, and `git diff --check` prints no output.

- [x] **Step 2: Verify the verification document has no unfilled markers**

Run:

```powershell
rg -n "Pending automated execution|Pending browser execution|Result recorded in Task 5" docs/superpowers/verification/2026-05-10-analysis-redesign.md
```

Expected: no output.

- [x] **Step 3: Check final working tree**

Run:

```powershell
git status --short
```

Expected: clean working tree.

- [x] **Step 4: Stop for review**

Report:

```text
Part 7 integration verification is implemented, recorded, and verified. The analysis result-first redesign plan series is complete.
```

Do not start any follow-up refactor, visual polish, or feature work until the user explicitly asks.

## Self-Review

- Spec coverage: this plan covers final verification for workspace/open-run separation, deleted scope behavior, report/source mode switching, no-run setup, run metadata, active/failed/cancelled source availability, evidence/chat snapshot safety, Runs filters, source reader behavior, persistence normalization, onboarding/error states, source ingest placement, and responsive browser checks.
- Boundary check: this plan adds verification and small contract fixes only. It does not add new `/analysis` features, does not redesign the global shell, and does not reintroduce legacy workspace surfaces.
- Snapshot trust: completed-run evidence and chat are verified to use saved snapshot context only; missing completed snapshots degrade instead of resolving against live source data.
- Persistence: tests verify that restored UI context does not restore `OpenRunState`, run snapshot basis, Evidence/Chat tabs, selected trace refs, or chat drafts.
- Browser verification: the verification record requires actual desktop, wide, narrow, and mobile checks before any PASS claim is written.
