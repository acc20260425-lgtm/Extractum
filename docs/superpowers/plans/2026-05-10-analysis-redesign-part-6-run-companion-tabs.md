# Analysis Result-First Redesign Part 6 Run Companion Tabs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the temporary right-side inspector and temporary central follow-up chat with `RunCompanionTabs` for Evidence, Chat, and Runs.

**Architecture:** Add pure companion helpers for tab activation, chat availability, evidence-to-source decisions, and run filtering, then build focused companion tab components. Wire the route so completed runs default to Evidence, trace clicks activate Evidence, chat activates only by explicit tab selection or question submit, and Runs contains only analysis report runs.

**Tech Stack:** SvelteKit 2, Svelte 5 runes, TypeScript, Vitest raw-source and helper tests, existing Extractum UI components, lucide Svelte icons, Tauri 2/Rust analysis chat guard, Part 1 `analysis-workspace-state`, Part 4 `ReportCanvas`, Part 5 source readers.

---

## Prerequisites

Implement this part only after Parts 1 through 5 are implemented and committed, not merely planned.

This plan assumes these files already exist from earlier parts:

- `src/lib/analysis-workspace-state.ts`
- `src/lib/analysis-workspace-persistence.ts`
- `src/lib/analysis-report-canvas-state.ts`
- `src/lib/components/analysis/report-canvas.svelte`
- `src/lib/components/analysis/report-source-surface.svelte`
- `src/lib/components/analysis/source-reader-header.svelte`
- `src/lib/components/analysis/telegram-timeline-reader.svelte`
- `src/lib/components/analysis/youtube-transcript-reader.svelte`
- `src/lib/components/analysis/source-group-reader.svelte`
- `src/lib/api/analysis-runs.ts` with `listAnalysisRunMessages(...)`

This plan also assumes `src/routes/analysis/+page.svelte` already:

- owns `workspaceUiState`;
- persists `workspaceUiState.companionTab`;
- renders `CompactSourceRail`;
- renders `ReportCanvas`;
- no longer renders `WorkspaceMain`;
- loads run snapshot pages for `ReportSourceSurface`;
- passes `selectedTraceRef` into source readers for highlighting.

If any prerequisite is missing, stop and implement the earlier part first.

This is **Part 6 of 7**. Stop after this part is implemented, verified, and committed. Continue to Part 7 only after explicit user approval.

## Part Boundary

Part 6 may:

- create `RunCompanionTabs`;
- create focused Evidence, Chat, and Runs tab components;
- move trace/evidence behavior from `WorkspaceInspector` into the Evidence tab;
- move follow-up chat out of the temporary `ReportCanvas` block into the Chat tab;
- replace `WorkspaceInspector` in `/analysis/+page.svelte` with `RunCompanionTabs`;
- keep `workspace-inspector.svelte`, `active-run-list.svelte`, `run-history.svelte`, and `chunk-summaries.svelte` as legacy or reusable components;
- add run search and filters for active/saved report runs;
- persist durable Runs filters/search;
- make `Show in source` switch `ReportCanvas` to Source and prefer run snapshot;
- add backend and frontend guards so completed-run chat cannot use live source as replacement context.

Part 6 must not:

- redesign the global app shell or app sidebar;
- change `CompactSourceRail` behavior except necessary callback wiring;
- change Part 5 source readers except selected-ref/source-basis wiring needed for `Show in source`;
- put source ingest jobs in Runs;
- make completed-run evidence or chat fall back to live source when snapshot rows are missing;
- auto-open a restored run;
- persist `OpenRunState`, selected trace refs, draft chat text, open popovers, scroll positions, or temporary source filters;
- close an opened run because of local evidence, chat, or Runs filtering;
- perform the final cross-part integration sweep. Leave that for Part 7.

## File Structure

- Create: `src/lib/analysis-run-companion-state.ts`
  - Responsibility: pure companion tab helpers, chat availability matrix, evidence `Show in source` decisions, and combined active/saved report-run filtering.
- Create: `src/lib/analysis-run-companion-state.test.ts`
  - Responsibility: focused helper coverage for Evidence defaults, Chat activation, snapshot-safe evidence source actions, chat availability, and Runs filters.
- Create: `src/lib/analysis-run-companion-tabs.test.ts`
  - Responsibility: raw-source coverage for `RunCompanionTabs` and tab components.
- Create: `src/lib/analysis-run-companion-route.test.ts`
  - Responsibility: raw-source coverage for route wiring, removal of temporary chat from `ReportCanvas`, replacement of `WorkspaceInspector`, and snapshot-safe tab activation.
- Modify: `src/lib/analysis-workspace-persistence.ts`
  - Responsibility: persist durable Runs filter/search state while still accepting the Part 2 persisted shape.
- Modify: `src/lib/analysis-workspace-persistence.test.ts`
  - Responsibility: verify new Runs filters are persisted and transient run-bound state remains excluded.
- Modify: `src/lib/analysis-trace-workflow.ts`
  - Responsibility: activate the Evidence companion tab when a trace ref is focused.
- Modify: `src/lib/analysis-trace-workflow.test.ts`
  - Responsibility: verify trace focus activates Evidence without touching Chat.
- Modify: `src/lib/analysis-chat-workflow.ts`
  - Responsibility: keep workflow validation focused on run/question state while route-level companion helpers gate snapshot availability before submit.
- Modify: `src/lib/analysis-chat-workflow.test.ts`
  - Responsibility: verify focus is not a chat activation path and question submission remains explicit.
- Modify: `src-tauri/src/analysis/chat.rs`
  - Responsibility: load completed-run chat context from saved snapshot rows only and reject completed runs whose snapshot context is missing.
- Modify: `src-tauri/src/analysis/mod.rs`
  - Responsibility: keep trace ref resolution from resolving completed-run refs against live source when snapshot rows are missing.
- Create: `src/lib/components/analysis/run-evidence-tab.svelte`
  - Responsibility: trace refs, selected evidence details, and `Show in source`.
- Create: `src/lib/components/analysis/run-chat-tab.svelte`
  - Responsibility: follow-up chat surface with disabled/warning-bound states from the chat availability matrix.
- Create: `src/lib/components/analysis/run-companion-runs-tab.svelte`
  - Responsibility: combined active queued/running analysis report runs and saved analysis run history with search/filter/scope controls.
- Create: `src/lib/components/analysis/run-companion-tabs.svelte`
  - Responsibility: accessible right-side `Evidence | Chat | Runs` tab shell.
- Modify: `src/lib/components/analysis/report-canvas.svelte`
  - Responsibility: remove the temporary follow-up chat block after Chat moves to `RunCompanionTabs`.
- Modify: `src/lib/components/analysis/trace-panel.svelte`
  - Responsibility: expose optional `Show in source` action for selected evidence when used by `RunEvidenceTab`.
- Modify: `src/routes/analysis/+page.svelte`
  - Responsibility: render `RunCompanionTabs`, remove `WorkspaceInspector`, route tab changes through `workspaceUiState.companionTab`, make trace clicks activate Evidence, make question submit activate Chat, and keep Runs filters local/durable.

## Task 1: Add Companion Contract Tests

**Files:**
- Create: `src/lib/analysis-run-companion-state.test.ts`
- Create: `src/lib/analysis-run-companion-tabs.test.ts`
- Create: `src/lib/analysis-run-companion-route.test.ts`

- [ ] **Step 1: Write failing companion state tests**

Create `src/lib/analysis-run-companion-state.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  chatAvailabilityForRun,
  defaultCompanionTabForOpenedRun,
  evidenceSourceActionDecision,
  filterCompanionRuns,
  runsFilterDefaults,
  type CompanionRunsFilterState,
} from "./analysis-run-companion-state";
import type { RunSnapshotAvailability } from "./analysis-report-canvas-state";
import type { WorkspaceSelection } from "./analysis-workspace-state";
import type { AnalysisRunDetail, AnalysisRunSummary, AnalysisTraceRef } from "./types/analysis";

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
    scope_label_snapshot: "Telegram A",
    period_from: 1710000000,
    period_to: 1710100000,
    output_language: "Russian",
    prompt_template_id: 1,
    prompt_template_name: "Weekly",
    prompt_template_version: 3,
    provider_profile: "default",
    provider: "openai",
    model: "gpt-5.4",
    youtube_corpus_mode: "transcript_description",
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
    id: 1,
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
    prompt_template_id: 1,
    prompt_template_name: "Weekly",
    prompt_template_version: 3,
    provider_profile: "default",
    provider: "openai",
    model: "gpt-5.4",
    youtube_corpus_mode: "transcript_description",
    status: "completed",
    error: null,
    has_trace_data: true,
    created_at: 1710100010,
    completed_at: 1710100100,
    ...overrides,
  };
}

function traceRef(overrides: Partial<AnalysisTraceRef> = {}): AnalysisTraceRef {
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

describe("analysis run companion state", () => {
  it("defaults completed runs to Evidence and active runs to Runs", () => {
    expect(defaultCompanionTabForOpenedRun(run({ status: "completed" }))).toBe("evidence");
    expect(defaultCompanionTabForOpenedRun(run({ status: "queued", completed_at: null }))).toBe("runs");
    expect(defaultCompanionTabForOpenedRun(run({ status: "running", completed_at: null }))).toBe("runs");
    expect(defaultCompanionTabForOpenedRun(run({ status: "failed", error: "failed" }))).toBe("runs");
    expect(defaultCompanionTabForOpenedRun(null)).toBe("runs");
  });

  it.each([
    ["unknown", false, "checking_snapshot"],
    ["capturing", false, "checking_snapshot"],
    ["available", true, "enabled"],
    ["unavailable", false, "missing_snapshot"],
  ] satisfies Array<[RunSnapshotAvailability, boolean, string]>)(
    "maps completed run chat availability for %s snapshot",
    (snapshotAvailability, enabled, reason) => {
      expect(chatAvailabilityForRun({
        currentRun: run({ status: "completed" }),
        snapshotAvailability,
      })).toMatchObject({ enabled, reason });
    },
  );

  it("disables chat for no run, active runs, failed runs, and cancelled runs", () => {
    expect(chatAvailabilityForRun({
      currentRun: null,
      snapshotAvailability: "unknown",
    })).toMatchObject({ enabled: false, reason: "no_run" });
    expect(chatAvailabilityForRun({
      currentRun: run({ status: "running", completed_at: null }),
      snapshotAvailability: "capturing",
    })).toMatchObject({ enabled: false, reason: "pending_completion" });
    expect(chatAvailabilityForRun({
      currentRun: run({ status: "failed", error: "provider failed" }),
      snapshotAvailability: "available",
    })).toMatchObject({ enabled: false, reason: "terminal_run" });
    expect(chatAvailabilityForRun({
      currentRun: run({ status: "cancelled" }),
      snapshotAvailability: "available",
    })).toMatchObject({ enabled: false, reason: "terminal_run" });
  });

  it("prefers run snapshot for Show in source when snapshot is available", () => {
    expect(evidenceSourceActionDecision({
      currentRun: run({ status: "completed" }),
      selectedTrace: traceRef(),
      snapshotAvailability: "available",
    })).toEqual({
      kind: "run_snapshot",
      canvasMode: "source",
      sourceViewBasis: "run_snapshot",
      highlightedRef: "s7-i11",
    });
  });

  it("does not resolve completed-run evidence against live source when snapshot is missing", () => {
    expect(evidenceSourceActionDecision({
      currentRun: run({ status: "completed" }),
      selectedTrace: traceRef(),
      snapshotAvailability: "unavailable",
    })).toEqual({
      kind: "unavailable",
      reason: "Exact source resolution is unavailable because this completed run has no saved snapshot rows.",
    });
  });

  it("allows explicit live source bridge for non-completed runs without labeling it as snapshot", () => {
    expect(evidenceSourceActionDecision({
      currentRun: run({ status: "failed", error: "failed" }),
      selectedTrace: traceRef(),
      snapshotAvailability: "unavailable",
    })).toMatchObject({
      kind: "live_source",
      canvasMode: "source",
      sourceViewBasis: "live_source",
      highlightedRef: "s7-i11",
    });
  });

  it("filters companion runs across active and saved report runs", () => {
    const filter: CompanionRunsFilterState = {
      ...runsFilterDefaults(),
      query: "weekly openai",
      status: "queued_running",
      scope: "current",
      dateFrom: "",
      dateTo: "",
      provider: "",
      model: "",
      template: "",
    };
    const workspaceSelection: WorkspaceSelection = { kind: "source", sourceId: 7 };

    const result = filterCompanionRuns({
      activeRuns: [
        summary({ id: 10, status: "running", provider: "openai", model: "gpt-5.4" }),
        summary({ id: 11, status: "queued", source_id: 8, source_title: "Other" }),
      ],
      savedRuns: [
        summary({ id: 12, status: "completed" }),
      ],
      filter,
      workspaceSelection,
    });

    expect(result.map((entry) => [entry.kind, entry.run.id])).toEqual([["active", 10]]);
  });

  it("supports cancelled status and optional provider/model/template filters", () => {
    const result = filterCompanionRuns({
      activeRuns: [],
      savedRuns: [
        summary({ id: 20, status: "cancelled", provider: "openai", model: "gpt-5.4", prompt_template_name: "Weekly" }),
        summary({ id: 21, status: "failed", provider: "gemini", model: "flash", prompt_template_name: "Daily" }),
      ],
      filter: {
        query: "",
        status: "cancelled",
        scope: "all",
        dateFrom: "2024-03-01",
        dateTo: "2024-03-31",
        provider: "openai",
        model: "gpt-5.4",
        template: "weekly",
      },
      workspaceSelection: { kind: "none" },
    });

    expect(result.map((entry) => entry.run.id)).toEqual([20]);
  });
});
```

- [ ] **Step 2: Write failing raw-source tests for companion components**

Create `src/lib/analysis-run-companion-tabs.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import companionTabsSource from "./components/analysis/run-companion-tabs.svelte?raw";
import evidenceTabSource from "./components/analysis/run-evidence-tab.svelte?raw";
import chatTabSource from "./components/analysis/run-chat-tab.svelte?raw";
import runsTabSource from "./components/analysis/run-companion-runs-tab.svelte?raw";
import reportCanvasSource from "./components/analysis/report-canvas.svelte?raw";

describe("run companion tabs", () => {
  it("uses accessible Evidence, Chat, and Runs tabs", () => {
    expect(companionTabsSource).toContain('class="run-companion-tabs"');
    expect(companionTabsSource).toContain('role="tablist"');
    expect(companionTabsSource).toContain('aria-label="Run companion tabs"');
    expect(companionTabsSource).toContain('onChangeCompanionTab("evidence")');
    expect(companionTabsSource).toContain('onChangeCompanionTab("chat")');
    expect(companionTabsSource).toContain('onChangeCompanionTab("runs")');
    expect(companionTabsSource).toContain("<RunEvidenceTab");
    expect(companionTabsSource).toContain("<RunChatTab");
    expect(companionTabsSource).toContain("<RunCompanionRunsTab");
  });

  it("keeps Evidence focused on trace refs and Show in source", () => {
    expect(evidenceTabSource).toContain("<TracePanel");
    expect(evidenceTabSource).toContain("Show in source");
    expect(evidenceTabSource).toContain("onShowSelectedTraceInSource");
    expect(evidenceTabSource).toContain("evidenceSourceActionDecision");
    expect(evidenceTabSource).toContain("Snapshot unavailable");
  });

  it("keeps Chat explicit and availability-gated", () => {
    expect(chatTabSource).toContain("<ChatPanel");
    expect(chatTabSource).toContain("chatAvailability");
    expect(chatTabSource).not.toContain("onfocus");
    expect(chatTabSource).not.toContain("onFocus");
  });

  it("contains only analysis report runs in the Runs tab", () => {
    expect(runsTabSource).toContain("filterCompanionRuns");
    expect(runsTabSource).toContain("queued/running");
    expect(runsTabSource).toContain("Search runs");
    expect(runsTabSource).toContain("Current scope");
    expect(runsTabSource).toContain("Date range");
    expect(runsTabSource).toContain("Provider filter");
    expect(runsTabSource).toContain("Template filter");
    expect(runsTabSource).not.toContain("SourceJobRecord");
    expect(runsTabSource).not.toContain("takeoutJobs");
    expect(runsTabSource).not.toContain("sourceJobs");
  });

  it("removes temporary chat from ReportCanvas once companion Chat exists", () => {
    expect(reportCanvasSource).not.toContain("temporary-follow-up");
    expect(reportCanvasSource).not.toContain("<ChatPanel");
    expect(reportCanvasSource).not.toContain("onAskRunQuestion");
    expect(reportCanvasSource).not.toContain("onChangeChatQuestion");
  });
});
```

- [ ] **Step 3: Write failing raw-source tests for route wiring**

Create `src/lib/analysis-run-companion-route.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";
import traceWorkflowSource from "./analysis-trace-workflow.ts?raw";
import chatWorkflowSource from "./analysis-chat-workflow.ts?raw";

describe("analysis route run companion wiring", () => {
  it("renders RunCompanionTabs instead of WorkspaceInspector", () => {
    expect(analysisPageSource).toContain(
      'import RunCompanionTabs from "$lib/components/analysis/run-companion-tabs.svelte";',
    );
    expect(analysisPageSource).not.toContain(
      'import WorkspaceInspector from "$lib/components/analysis/workspace-inspector.svelte";',
    );
    expect(analysisPageSource).toContain("<RunCompanionTabs");
    expect(analysisPageSource).not.toContain("<WorkspaceInspector");
  });

  it("uses workspaceUiState.companionTab as the only companion tab source", () => {
    expect(analysisPageSource).toContain("companionTab={workspaceUiState.companionTab}");
    expect(analysisPageSource).toContain("function changeCompanionTab");
    expect(analysisPageSource).toContain("companionTab: nextTab");
    expect(analysisPageSource).not.toContain("let inspectorMode");
    expect(analysisPageSource).not.toContain("onChangeInspectorMode");
  });

  it("activates Evidence for trace clicks and Show in source prefers snapshot", () => {
    expect(analysisPageSource).toContain("async function focusTraceRef");
    expect(analysisPageSource).toContain('companionTab: "evidence"');
    expect(analysisPageSource).toContain("showSelectedTraceInSource");
    expect(analysisPageSource).toContain("evidenceSourceActionDecision");
    expect(analysisPageSource).toContain('sourceViewBasis: "run_snapshot"');
    expect(analysisPageSource).toContain('sourceViewBasis: "live_source"');
  });

  it("activates Chat only through tab selection or question submission", () => {
    expect(analysisPageSource).toContain("submitRunQuestionFromCompanion");
    expect(analysisPageSource).toContain("chatAvailabilityForRun");
    expect(analysisPageSource).toContain('companionTab: "chat"');
    expect(analysisPageSource).not.toContain("onFocusChat");
    expect(chatWorkflowSource).not.toContain("companionTab");
  });

  it("keeps Runs filters durable and source ingest jobs out of Runs", () => {
    expect(analysisPageSource).toContain("runsFilter");
    expect(analysisPageSource).toContain("persistableAnalysisWorkspaceState(workspaceUiState");
    expect(analysisPageSource).toContain("runsFilter");
  });

  it("updates trace workflow patches from inspector mode to evidence companion", () => {
    expect(traceWorkflowSource).toContain('companionTab: "evidence"');
    expect(traceWorkflowSource).not.toContain("inspectorMode");
  });
});
```

- [ ] **Step 4: Run the focused tests and verify they fail**

Run:

```powershell
npm.cmd test -- src/lib/analysis-run-companion-state.test.ts src/lib/analysis-run-companion-tabs.test.ts src/lib/analysis-run-companion-route.test.ts
```

Expected: FAIL because `analysis-run-companion-state.ts` and the companion components do not exist, `ReportCanvas` still contains temporary chat, and `/analysis` still renders the old right panel.

- [ ] **Step 5: Commit the failing contract tests**

Run:

```powershell
git add src/lib/analysis-run-companion-state.test.ts src/lib/analysis-run-companion-tabs.test.ts src/lib/analysis-run-companion-route.test.ts
git commit -m "test: define analysis run companion contract"
```

## Task 2: Add Companion State Helpers

**Files:**
- Create: `src/lib/analysis-run-companion-state.ts`

- [ ] **Step 1: Add the helper module**

Create `src/lib/analysis-run-companion-state.ts`:

```ts
import type { RunSnapshotAvailability } from "$lib/analysis-report-canvas-state";
import type {
  CanvasMode,
  CompanionTab,
  SourceViewBasis,
  WorkspaceSelection,
} from "$lib/analysis-workspace-state";
import type { AnalysisRunDetail, AnalysisRunSummary, AnalysisTraceRef } from "$lib/types/analysis";

export type ChatAvailabilityReason =
  | "enabled"
  | "no_run"
  | "pending_completion"
  | "terminal_run"
  | "checking_snapshot"
  | "missing_snapshot"
  | "missing_report";

export interface ChatAvailability {
  enabled: boolean;
  reason: ChatAvailabilityReason;
  title: string;
  description: string;
}

export type EvidenceSourceActionDecision =
  | {
      kind: "run_snapshot";
      canvasMode: CanvasMode;
      sourceViewBasis: Extract<SourceViewBasis, "run_snapshot">;
      highlightedRef: string;
    }
  | {
      kind: "live_source";
      canvasMode: CanvasMode;
      sourceViewBasis: Extract<SourceViewBasis, "live_source">;
      highlightedRef: string;
      warning: string;
    }
  | {
      kind: "unavailable";
      reason: string;
    };

export type CompanionRunStatusFilter =
  | "all"
  | "completed"
  | "failed"
  | "cancelled"
  | "queued_running";

export interface CompanionRunsFilterState {
  query: string;
  status: CompanionRunStatusFilter;
  scope: "all" | "current";
  dateFrom: string;
  dateTo: string;
  provider: string;
  model: string;
  template: string;
}

export interface CompanionRunEntry {
  kind: "active" | "saved";
  run: AnalysisRunSummary;
}

export function runsFilterDefaults(): CompanionRunsFilterState {
  return {
    query: "",
    status: "all",
    scope: "all",
    dateFrom: "",
    dateTo: "",
    provider: "",
    model: "",
    template: "",
  };
}

export function defaultCompanionTabForOpenedRun(run: AnalysisRunDetail | null): CompanionTab {
  return run?.status === "completed" ? "evidence" : "runs";
}

export function chatAvailabilityForRun({
  currentRun,
  snapshotAvailability,
}: {
  currentRun: AnalysisRunDetail | null;
  snapshotAvailability: RunSnapshotAvailability;
}): ChatAvailability {
  if (!currentRun) {
    return {
      enabled: false,
      reason: "no_run",
      title: "Open a completed run",
      description: "Follow-up chat is available after a saved report is open.",
    };
  }

  if (currentRun.status === "queued" || currentRun.status === "running") {
    return {
      enabled: false,
      reason: "pending_completion",
      title: "Run still in progress",
      description: "Chat becomes available after the report completes and saved context is available.",
    };
  }

  if (currentRun.status === "failed" || currentRun.status === "cancelled") {
    return {
      enabled: false,
      reason: "terminal_run",
      title: "Chat is disabled for this run",
      description: "For this MVP, follow-up chat is available only for completed reports.",
    };
  }

  if (!currentRun.result_markdown?.trim()) {
    return {
      enabled: false,
      reason: "missing_report",
      title: "No saved report",
      description: "This completed run has no saved report output for follow-up chat.",
    };
  }

  if (snapshotAvailability === "available") {
    return {
      enabled: true,
      reason: "enabled",
      title: "Chat ready",
      description: "Questions use the saved report and saved run snapshot context.",
    };
  }

  if (snapshotAvailability === "unavailable") {
    return {
      enabled: false,
      reason: "missing_snapshot",
      title: "Saved context unavailable",
      description: "This completed run has no saved snapshot rows, so chat will not use live source as replacement context.",
    };
  }

  return {
    enabled: false,
    reason: "checking_snapshot",
    title: "Checking saved context",
    description: "Chat becomes available when the saved run snapshot has been checked.",
  };
}

export function evidenceSourceActionDecision({
  currentRun,
  selectedTrace,
  snapshotAvailability,
}: {
  currentRun: AnalysisRunDetail | null;
  selectedTrace: AnalysisTraceRef | null;
  snapshotAvailability: RunSnapshotAvailability;
}): EvidenceSourceActionDecision {
  if (!currentRun || !selectedTrace) {
    return {
      kind: "unavailable",
      reason: "Select evidence from an opened run before showing it in source.",
    };
  }

  if (snapshotAvailability === "available") {
    return {
      kind: "run_snapshot",
      canvasMode: "source",
      sourceViewBasis: "run_snapshot",
      highlightedRef: selectedTrace.ref,
    };
  }

  if (currentRun.status === "completed") {
    return {
      kind: "unavailable",
      reason: "Exact source resolution is unavailable because this completed run has no saved snapshot rows.",
    };
  }

  return {
    kind: "live_source",
    canvasMode: "source",
    sourceViewBasis: "live_source",
    highlightedRef: selectedTrace.ref,
    warning: "Showing live source for a non-completed run. This is not the frozen run snapshot.",
  };
}

function normalizedText(value: string | null | undefined) {
  return (value ?? "").trim().toLocaleLowerCase();
}

function runSearchText(run: AnalysisRunSummary) {
  return [
    run.scope_label,
    run.source_title,
    run.source_group_name,
    run.prompt_template_name,
    run.provider_profile,
    run.provider,
    run.model,
    run.error,
  ].map(normalizedText).join(" ");
}

function runMatchesStatus(run: AnalysisRunSummary, status: CompanionRunStatusFilter) {
  if (status === "all") return true;
  if (status === "queued_running") return run.status === "queued" || run.status === "running";
  return run.status === status;
}

function runMatchesWorkspace(run: AnalysisRunSummary, selection: WorkspaceSelection) {
  if (selection.kind === "none") return false;
  if (selection.kind === "source") return run.source_id === selection.sourceId;
  return run.source_group_id === selection.sourceGroupId;
}

function parseDateStart(value: string) {
  const trimmed = value.trim();
  if (!trimmed) return null;
  const time = Date.parse(`${trimmed}T00:00:00Z`);
  return Number.isFinite(time) ? Math.floor(time / 1000) : null;
}

function parseDateEnd(value: string) {
  const trimmed = value.trim();
  if (!trimmed) return null;
  const time = Date.parse(`${trimmed}T23:59:59Z`);
  return Number.isFinite(time) ? Math.floor(time / 1000) : null;
}

export function filterCompanionRuns({
  activeRuns,
  savedRuns,
  filter,
  workspaceSelection,
}: {
  activeRuns: AnalysisRunSummary[];
  savedRuns: AnalysisRunSummary[];
  filter: CompanionRunsFilterState;
  workspaceSelection: WorkspaceSelection;
}): CompanionRunEntry[] {
  const queryTerms = normalizedText(filter.query).split(/\s+/).filter(Boolean);
  const provider = normalizedText(filter.provider);
  const model = normalizedText(filter.model);
  const template = normalizedText(filter.template);
  const from = parseDateStart(filter.dateFrom);
  const to = parseDateEnd(filter.dateTo);

  return [
    ...activeRuns.map((run): CompanionRunEntry => ({ kind: "active", run })),
    ...savedRuns.map((run): CompanionRunEntry => ({ kind: "saved", run })),
  ].filter(({ run }) => {
    if (filter.scope === "current" && !runMatchesWorkspace(run, workspaceSelection)) {
      return false;
    }
    if (!runMatchesStatus(run, filter.status)) {
      return false;
    }
    if (from !== null && run.created_at < from) {
      return false;
    }
    if (to !== null && run.created_at > to) {
      return false;
    }
    if (provider && !normalizedText(run.provider).includes(provider)) {
      return false;
    }
    if (model && !normalizedText(run.model).includes(model)) {
      return false;
    }
    if (template && !normalizedText(run.prompt_template_name).includes(template)) {
      return false;
    }

    const haystack = runSearchText(run);
    return queryTerms.every((term) => haystack.includes(term));
  }).sort((left, right) => right.run.created_at - left.run.created_at);
}
```

- [ ] **Step 2: Run companion helper tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-run-companion-state.test.ts
```

Expected: PASS.

- [ ] **Step 3: Commit the helper module**

Run:

```powershell
git add src/lib/analysis-run-companion-state.ts src/lib/analysis-run-companion-state.test.ts
git commit -m "feat: add analysis run companion state helpers"
```

## Task 3: Persist Durable Runs Filters

**Files:**
- Modify: `src/lib/analysis-workspace-persistence.ts`
- Modify: `src/lib/analysis-workspace-persistence.test.ts`

- [ ] **Step 1: Add failing persistence coverage for extended Runs filters**

In `src/lib/analysis-workspace-persistence.test.ts`, import:

```ts
import { runsFilterDefaults } from "./analysis-run-companion-state";
```

Extend the existing durable-state serialization test so the second argument passed to `persistableAnalysisWorkspaceState(...)` is:

```ts
{
  historyScope: "current",
  runFilter: "completed",
  runsFilter: {
    ...runsFilterDefaults(),
    query: "weekly openai",
    status: "completed",
    scope: "current",
    dateFrom: "2024-03-01",
    dateTo: "2024-03-31",
    provider: "openai",
    model: "gpt-5.4",
    template: "weekly",
  },
}
```

Update the expected persisted `runs` object:

```ts
runs: {
  historyScope: "current",
  runFilter: "completed",
  runsFilter: {
    query: "weekly openai",
    status: "completed",
    scope: "current",
    dateFrom: "2024-03-01",
    dateTo: "2024-03-31",
    provider: "openai",
    model: "gpt-5.4",
    template: "weekly",
  },
},
```

Add this test:

```ts
it("loads older persisted runs state with default companion filters", () => {
  const storage = new MemoryStorage();
  storage.setItem(ANALYSIS_WORKSPACE_STATE_KEY, JSON.stringify({
    version: 1,
    workspaceSelection: { kind: "source", sourceId: 7 },
    canvasMode: "source",
    sourceViewBasis: "live_source",
    companionTab: "runs",
    runs: {
      historyScope: "all",
      runFilter: "all",
    },
  }));

  expect(loadPersistedAnalysisWorkspaceState(storage)?.runs.runsFilter)
    .toEqual(runsFilterDefaults());
});
```

- [ ] **Step 2: Run persistence tests and verify they fail**

Run:

```powershell
npm.cmd test -- src/lib/analysis-workspace-persistence.test.ts
```

Expected: FAIL because `PersistedAnalysisWorkspaceRunsState` does not include `runsFilter`.

- [ ] **Step 3: Extend persisted Runs state**

In `src/lib/analysis-workspace-persistence.ts`, import:

```ts
import {
  runsFilterDefaults,
  type CompanionRunStatusFilter,
  type CompanionRunsFilterState,
} from "$lib/analysis-run-companion-state";
```

Extend `PersistedAnalysisWorkspaceRunsState`:

```ts
export interface PersistedAnalysisWorkspaceRunsState {
  historyScope: AnalysisHistoryScope;
  runFilter: AnalysisRunFilter;
  runsFilter: CompanionRunsFilterState;
}
```

Add parsers:

```ts
function parseString(value: unknown) {
  return typeof value === "string" ? value : "";
}

function parseRunsStatus(value: unknown): CompanionRunStatusFilter {
  return value === "completed" ||
    value === "failed" ||
    value === "cancelled" ||
    value === "queued_running"
    ? value
    : "all";
}

function parseRunsFilter(value: unknown): CompanionRunsFilterState {
  if (!isObject(value)) {
    return runsFilterDefaults();
  }

  return {
    query: parseString(value.query),
    status: parseRunsStatus(value.status),
    scope: value.scope === "current" ? "current" : "all",
    dateFrom: parseString(value.dateFrom),
    dateTo: parseString(value.dateTo),
    provider: parseString(value.provider),
    model: parseString(value.model),
    template: parseString(value.template),
  };
}
```

In `parsePersistedAnalysisWorkspaceState(...)`, set:

```ts
const runsFilter = runs ? parseRunsFilter(runs.runsFilter) : runsFilterDefaults();
```

Return:

```ts
runs: {
  historyScope,
  runFilter,
  runsFilter,
},
```

- [ ] **Step 4: Run persistence tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-workspace-persistence.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit persistence changes**

Run:

```powershell
git add src/lib/analysis-workspace-persistence.ts src/lib/analysis-workspace-persistence.test.ts
git commit -m "feat: persist analysis runs companion filters"
```

## Task 4: Guard Completed-Run Chat And Evidence On The Backend

**Files:**
- Modify: `src-tauri/src/analysis/chat.rs`
- Modify: `src-tauri/src/analysis/mod.rs`
- Modify: `src-tauri/src/analysis/corpus.rs`

- [ ] **Step 1: Add failing chat guard tests**

In `src-tauri/src/analysis/chat.rs`, update imports:

```rust
use super::corpus::load_run_snapshot_messages;
```

Add this helper near `format_chat_context_messages(...)`:

```rust
fn ensure_completed_chat_context(
    run: &AnalysisRunDetail,
    snapshot: &[CorpusMessage],
) -> AppResult<()> {
    if run.status != ANALYSIS_STATUS_COMPLETED {
        return Err(AppError::validation(
            "Open a completed analysis run before asking follow-up questions",
        ));
    }

    if snapshot.is_empty() {
        return Err(AppError::conflict(
            "This completed analysis run has no saved snapshot context for follow-up chat",
        ));
    }

    Ok(())
}
```

In the `#[cfg(test)] mod tests`, import it:

```rust
use super::{build_chat_request, ensure_completed_chat_context, format_chat_context_messages, ChatRequestParams};
```

Add tests:

```rust
#[test]
fn completed_chat_context_requires_saved_snapshot_messages() {
    let error = ensure_completed_chat_context(&sample_run(), &[])
        .expect_err("missing snapshot rejects completed chat");

    assert_eq!(
        error.message,
        "This completed analysis run has no saved snapshot context for follow-up chat"
    );
}

#[test]
fn completed_chat_context_accepts_saved_snapshot_messages() {
    ensure_completed_chat_context(&sample_run(), &[sample_message()])
        .expect("snapshot context enables completed chat");
}
```

- [ ] **Step 2: Run the chat guard tests and verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::chat::tests::completed_chat_context_requires_saved_snapshot_messages analysis::chat::tests::completed_chat_context_accepts_saved_snapshot_messages
```

Expected: FAIL because the helper is not wired and imports are not updated.

- [ ] **Step 3: Make chat use saved snapshot only**

In `ask_analysis_run_question(...)`, replace:

```rust
let corpus = load_run_corpus_messages(&pool, &run).await?;
let context_messages = find_chat_context_messages(&question, &corpus);
```

with:

```rust
let corpus = load_run_snapshot_messages(&pool, run.id)
    .await
    .map_err(AppError::database)?;
ensure_completed_chat_context(&run, &corpus)?;
let context_messages = find_chat_context_messages(&question, &corpus);
```

This keeps completed-run chat bound to the saved run snapshot. It does not call `load_run_corpus_messages(...)` for chat.

- [ ] **Step 4: Add snapshot-safe trace resolution loader**

In `src-tauri/src/analysis/corpus.rs`, add this function near `load_run_corpus_messages(...)`:

```rust
pub(crate) async fn load_trace_resolution_messages(
    pool: &Pool<Sqlite>,
    run: &AnalysisRunDetail,
) -> Result<Vec<CorpusMessage>, String> {
    let snapshot = load_run_snapshot_messages(pool, run.id).await?;
    if !snapshot.is_empty() {
        return Ok(snapshot);
    }

    if run.status == "completed" {
        return Ok(Vec::new());
    }

    load_run_corpus_messages(pool, run).await
}
```

Add a unit test beside the snapshot tests:

```rust
#[tokio::test]
async fn trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot() {
    let pool = snapshot_pool().await;
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, content_zstd)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(11_i64)
    .bind(2_i64)
    .bind("100")
    .bind("telegram_message")
    .bind("Alice")
    .bind(1_710_000_000_i64)
    .bind(compress_text("Live source text").expect("compress live text"))
    .execute(&pool)
    .await
    .expect("insert live item");

    let messages = load_trace_resolution_messages(&pool, &sample_run())
        .await
        .expect("load trace resolution messages");

    assert!(messages.is_empty());
}
```

- [ ] **Step 5: Wire trace resolution to the safe loader**

In `src-tauri/src/analysis/mod.rs`, replace the corpus import:

```rust
use self::corpus::load_run_corpus_messages;
```

with:

```rust
use self::corpus::load_trace_resolution_messages;
```

In `resolve_analysis_trace_refs(...)`, replace:

```rust
let corpus = load_run_corpus_messages(&pool, &run).await?;
```

with:

```rust
let corpus = load_trace_resolution_messages(&pool, &run).await?;
```

- [ ] **Step 6: Run backend focused tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::chat::tests::completed_chat_context_requires_saved_snapshot_messages analysis::chat::tests::completed_chat_context_accepts_saved_snapshot_messages analysis::corpus::tests::trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot
```

Expected: PASS.

- [ ] **Step 7: Commit backend guard changes**

Run:

```powershell
git add src-tauri/src/analysis/chat.rs src-tauri/src/analysis/corpus.rs src-tauri/src/analysis/mod.rs
git commit -m "fix: guard analysis companion context against live fallback"
```

## Task 5: Build Companion Tab Components

**Files:**
- Create: `src/lib/components/analysis/run-evidence-tab.svelte`
- Create: `src/lib/components/analysis/run-chat-tab.svelte`
- Create: `src/lib/components/analysis/run-companion-runs-tab.svelte`
- Create: `src/lib/components/analysis/run-companion-tabs.svelte`
- Modify: `src/lib/components/analysis/trace-panel.svelte`

- [x] **Step 1: Extend `TracePanel` with optional Show in source**

In `src/lib/components/analysis/trace-panel.svelte`, add props:

```ts
    showInSourceDisabledReason?: string;
    onShowInSource?: () => void | Promise<void>;
```

Add defaults in destructuring:

```ts
    showInSourceDisabledReason = "",
    onShowInSource,
```

Inside the selected trace detail block, after the YouTube link, add:

```svelte
{#if onShowInSource}
  <Button
    variant="secondary"
    size="sm"
    type="button"
    disabled={!!showInSourceDisabledReason}
    title={showInSourceDisabledReason || "Show this evidence in Source mode"}
    onclick={() => void onShowInSource()}
  >
    Show in source
  </Button>
{/if}
```

Add the import:

```ts
import Button from "$lib/components/ui/Button.svelte";
```

- [x] **Step 2: Create Evidence tab**

Create `src/lib/components/analysis/run-evidence-tab.svelte`:

```svelte
<script lang="ts">
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import TracePanel from "$lib/components/analysis/trace-panel.svelte";
  import { evidenceSourceActionDecision } from "$lib/analysis-run-companion-state";
  import type { RunSnapshotAvailability } from "$lib/analysis-report-canvas-state";
  import type { AnalysisRunDetail, AnalysisTraceData, AnalysisTraceRef } from "$lib/types/analysis";

  let {
    currentRun,
    traceData,
    selectedTraceRef,
    selectedTrace,
    snapshotAvailability,
    formatTimestamp,
    traceRefOrigin,
    onSelectTraceRef,
    onShowSelectedTraceInSource,
  }: {
    currentRun: AnalysisRunDetail | null;
    traceData: AnalysisTraceData;
    selectedTraceRef: string | null;
    selectedTrace: AnalysisTraceRef | null;
    snapshotAvailability: RunSnapshotAvailability;
    formatTimestamp: (timestamp: number | null) => string;
    traceRefOrigin: (ref: string) => string;
    onSelectTraceRef: (ref: string) => void | Promise<void>;
    onShowSelectedTraceInSource: () => void | Promise<void>;
  } = $props();

  const sourceDecision = $derived(evidenceSourceActionDecision({
    currentRun,
    selectedTrace,
    snapshotAvailability,
  }));
</script>

<section class="run-evidence-tab">
  {#if !currentRun}
    <EmptyState
      title="No run open"
      description="Open a saved or active report to inspect trace evidence."
    />
  {:else}
    {#if sourceDecision.kind === "unavailable" && selectedTrace}
      <StatusMessage tone="default" className="evidence-warning">
        {sourceDecision.reason}
      </StatusMessage>
    {:else if sourceDecision.kind === "live_source"}
      <StatusMessage tone="default" className="evidence-warning">
        {sourceDecision.warning}
      </StatusMessage>
    {/if}

    <TracePanel
      traceRefs={traceData.refs}
      {selectedTraceRef}
      {selectedTrace}
      {formatTimestamp}
      {traceRefOrigin}
      showInSourceDisabledReason={sourceDecision.kind === "unavailable" ? sourceDecision.reason : ""}
      onSelectTraceRef={onSelectTraceRef}
      onShowInSource={onShowSelectedTraceInSource}
    />
  {/if}
</section>

<style>
  .run-evidence-tab {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
</style>
```

- [x] **Step 3: Create Chat tab**

Create `src/lib/components/analysis/run-chat-tab.svelte`:

```svelte
<script lang="ts">
  import ChatPanel from "$lib/components/analysis/chat-panel.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import type { ChatAvailability } from "$lib/analysis-run-companion-state";
  import type { AnalysisChatTurn, AnalysisRunDetail } from "$lib/types/analysis";

  let {
    currentRun,
    chatAvailability,
    loadingChat,
    chatMessages,
    chatQuestion,
    chatting,
    canCancelChat,
    clearingChat,
    selectedTraceRef,
    reportLines,
    onFocusTraceRef,
    onAskQuestion,
    onCancelChat,
    onClearChat,
    onChangeChatQuestion,
  }: {
    currentRun: AnalysisRunDetail | null;
    chatAvailability: ChatAvailability;
    loadingChat: boolean;
    chatMessages: AnalysisChatTurn[];
    chatQuestion: string;
    chatting: boolean;
    canCancelChat: boolean;
    clearingChat: boolean;
    selectedTraceRef: string | null;
    reportLines: (text: string) => Array<{
      key: string;
      segments: Array<{ type: "text" | "ref"; value: string; key: string }>;
    }>;
    onFocusTraceRef: (ref: string) => void | Promise<void>;
    onAskQuestion: () => void | Promise<void>;
    onCancelChat: () => void | Promise<void>;
    onClearChat: () => void | Promise<void>;
    onChangeChatQuestion: (value: string) => void;
  } = $props();
</script>

<section class="run-chat-tab">
  {#if !chatAvailability.enabled}
    <StatusMessage tone="default">
      {chatAvailability.title}: {chatAvailability.description}
    </StatusMessage>
    <EmptyState title={chatAvailability.title} description={chatAvailability.description} />
  {:else}
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
      {onFocusTraceRef}
      {onAskQuestion}
      {onCancelChat}
      {onClearChat}
      {onChangeChatQuestion}
    />
  {/if}
</section>

<style>
  .run-chat-tab {
    min-width: 0;
  }
</style>
```

- [x] **Step 4: Create Runs tab**

Create `src/lib/components/analysis/run-companion-runs-tab.svelte`:

```svelte
<script lang="ts">
  import { PanelRightOpen, RefreshCw, Square, Trash2 } from "@lucide/svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import {
    filterCompanionRuns,
    type CompanionRunsFilterState,
  } from "$lib/analysis-run-companion-state";
  import type { BadgeVariant } from "$lib/components/ui/types";
  import type { WorkspaceSelection } from "$lib/analysis-workspace-state";
  import type { AnalysisRunSummary } from "$lib/types/analysis";

  let {
    activeRuns,
    savedRuns,
    loadingActiveRuns,
    loadingRuns,
    activeRunId,
    deletingRunIds,
    workspaceSelection,
    runsFilter,
    formatTimestamp,
    formatPeriod,
    phaseLabel,
    livePhase,
    liveProgress,
    runTargetLabel,
    statusTone,
    onChangeRunsFilter,
    onRefreshActiveRuns,
    onRefreshRuns,
    onOpenRun,
    onCancelRun,
    onDeleteRun,
  }: {
    activeRuns: AnalysisRunSummary[];
    savedRuns: AnalysisRunSummary[];
    loadingActiveRuns: boolean;
    loadingRuns: boolean;
    activeRunId: number | null;
    deletingRunIds: Record<number, boolean>;
    workspaceSelection: WorkspaceSelection;
    runsFilter: CompanionRunsFilterState;
    formatTimestamp: (timestamp: number | null) => string;
    formatPeriod: (periodFromUnix: number, periodToUnix: number) => string;
    phaseLabel: (phase: string) => string;
    livePhase: (runId: number) => string;
    liveProgress: (runId: number) => string;
    runTargetLabel: (run: Pick<AnalysisRunSummary, "scope_type" | "source_id" | "source_title" | "source_group_id" | "source_group_name" | "scope_label">) => string;
    statusTone: (status: string) => BadgeVariant;
    onChangeRunsFilter: (filter: CompanionRunsFilterState) => void;
    onRefreshActiveRuns: () => void | Promise<void>;
    onRefreshRuns: () => void | Promise<void>;
    onOpenRun: (runId: number) => void | Promise<void>;
    onCancelRun: (runId: number) => void | Promise<void>;
    onDeleteRun: (run: AnalysisRunSummary) => void | Promise<void>;
  } = $props();

  const entries = $derived(filterCompanionRuns({
    activeRuns,
    savedRuns,
    filter: runsFilter,
    workspaceSelection,
  }));

  function updateFilter(patch: Partial<CompanionRunsFilterState>) {
    onChangeRunsFilter({ ...runsFilter, ...patch });
  }
</script>

<section class="run-companion-runs-tab">
  <div class="runs-toolbar">
    <label>
      <span>Search runs</span>
      <Input
        type="search"
        value={runsFilter.query}
        placeholder="Search target, template, provider, model, error"
        ariaLabel="Search runs"
        oninput={(event) => updateFilter({ query: (event.currentTarget as HTMLInputElement).value })}
      />
    </label>

    <div class="segmented" aria-label="Runs scope">
      <Button size="sm" variant="secondary" selected={runsFilter.scope === "current"} onclick={() => updateFilter({ scope: "current" })}>Current scope</Button>
      <Button size="sm" variant="secondary" selected={runsFilter.scope === "all"} onclick={() => updateFilter({ scope: "all" })}>All runs</Button>
    </div>

    <div class="segmented" aria-label="Runs status">
      <Button size="sm" variant="secondary" selected={runsFilter.status === "all"} onclick={() => updateFilter({ status: "all" })}>All</Button>
      <Button size="sm" variant="secondary" selected={runsFilter.status === "queued_running"} onclick={() => updateFilter({ status: "queued_running" })}>queued/running</Button>
      <Button size="sm" variant="secondary" selected={runsFilter.status === "completed"} onclick={() => updateFilter({ status: "completed" })}>Completed</Button>
      <Button size="sm" variant="secondary" selected={runsFilter.status === "failed"} onclick={() => updateFilter({ status: "failed" })}>Failed</Button>
      <Button size="sm" variant="secondary" selected={runsFilter.status === "cancelled"} onclick={() => updateFilter({ status: "cancelled" })}>Cancelled</Button>
    </div>

    <div class="date-row" aria-label="Date range">
      <label><span>From</span><Input type="date" value={runsFilter.dateFrom} ariaLabel="Runs from date" oninput={(event) => updateFilter({ dateFrom: (event.currentTarget as HTMLInputElement).value })} /></label>
      <label><span>To</span><Input type="date" value={runsFilter.dateTo} ariaLabel="Runs to date" oninput={(event) => updateFilter({ dateTo: (event.currentTarget as HTMLInputElement).value })} /></label>
    </div>

    <div class="meta-row">
      <Input value={runsFilter.provider} placeholder="Provider" ariaLabel="Provider filter" oninput={(event) => updateFilter({ provider: (event.currentTarget as HTMLInputElement).value })} />
      <Input value={runsFilter.model} placeholder="Model" ariaLabel="Model filter" oninput={(event) => updateFilter({ model: (event.currentTarget as HTMLInputElement).value })} />
      <Input value={runsFilter.template} placeholder="Template" ariaLabel="Template filter" oninput={(event) => updateFilter({ template: (event.currentTarget as HTMLInputElement).value })} />
    </div>

    <div class="refresh-row">
      <Button size="sm" variant="secondary" onclick={onRefreshActiveRuns}>
        <RefreshCw size={14} aria-hidden="true" /> Active
      </Button>
      <Button size="sm" variant="secondary" onclick={onRefreshRuns}>
        <RefreshCw size={14} aria-hidden="true" /> Saved
      </Button>
    </div>
  </div>

  {#if loadingActiveRuns || loadingRuns}
    <EmptyState description="Loading analysis report runs..." />
  {:else if entries.length === 0}
    <EmptyState description="No analysis report runs match these filters." />
  {:else}
    <ul class="runs-list">
      {#each entries as entry (`${entry.kind}-${entry.run.id}`)}
        {@const run = entry.run}
        <li class:selected={run.id === activeRunId}>
          <div class="run-copy">
            <div class="run-title">
              <strong>{runTargetLabel(run)}</strong>
              <Badge variant={statusTone(run.status)}>{run.status}</Badge>
              <Badge variant={entry.kind === "active" ? "info" : "neutral"}>{entry.kind}</Badge>
            </div>
            <p>{formatTimestamp(run.created_at)} - {run.provider}/{run.model} - {run.prompt_template_name ?? "Unknown template"} v{run.prompt_template_version}</p>
            <p>Period: {formatPeriod(run.period_from, run.period_to)}</p>
            {#if entry.kind === "active"}
              <p>Phase: {phaseLabel(livePhase(run.id) || run.status)} {liveProgress(run.id)}</p>
            {/if}
            {#if run.error}
              <p class="run-error">{run.error}</p>
            {/if}
          </div>
          <div class="run-actions">
            <Button size="sm" variant="secondary" onclick={() => onOpenRun(run.id)}>
              <PanelRightOpen size={14} aria-hidden="true" /> Open
            </Button>
            {#if entry.kind === "active"}
              <Button size="sm" variant="danger-soft" onclick={() => onCancelRun(run.id)}>
                <Square size={14} aria-hidden="true" /> Cancel
              </Button>
            {:else}
              <Button size="sm" variant="danger-soft" disabled={deletingRunIds[run.id]} onclick={() => onDeleteRun(run)}>
                <Trash2 size={14} aria-hidden="true" />
                {deletingRunIds[run.id] ? "Deleting..." : "Delete"}
              </Button>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</section>
```

Add CSS with stable dimensions and no nested cards:

```svelte
<style>
  .run-companion-runs-tab,
  .runs-toolbar,
  .runs-list {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    color: var(--muted);
    font-size: 0.82rem;
  }

  .segmented,
  .date-row,
  .meta-row,
  .refresh-row,
  .run-title,
  .run-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
    align-items: center;
  }

  .runs-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .runs-list li {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.85rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel-strong);
  }

  .runs-list li.selected {
    border-color: var(--primary);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 12%, transparent);
  }

  .run-copy {
    min-width: 0;
  }

  .run-copy p {
    margin: 0.25rem 0 0;
    color: var(--muted);
    font-size: 0.86rem;
    overflow-wrap: anywhere;
  }

  .run-error {
    color: var(--status-error-text);
  }

  @media (max-width: 720px) {
    .runs-list li {
      flex-direction: column;
      align-items: stretch;
    }
  }
</style>
```

- [x] **Step 5: Create tab shell**

Create `src/lib/components/analysis/run-companion-tabs.svelte`:

```svelte
<script lang="ts">
  import RunChatTab from "$lib/components/analysis/run-chat-tab.svelte";
  import RunCompanionRunsTab from "$lib/components/analysis/run-companion-runs-tab.svelte";
  import RunEvidenceTab from "$lib/components/analysis/run-evidence-tab.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import type { ChatAvailability, CompanionRunsFilterState } from "$lib/analysis-run-companion-state";
  import type { RunSnapshotAvailability } from "$lib/analysis-report-canvas-state";
  import type { CompanionTab, WorkspaceSelection } from "$lib/analysis-workspace-state";
  import type { BadgeVariant } from "$lib/components/ui/types";
  import type {
    AnalysisChatTurn,
    AnalysisRunDetail,
    AnalysisRunSummary,
    AnalysisTraceData,
    AnalysisTraceRef,
  } from "$lib/types/analysis";

  let {
    companionTab,
    currentRun,
    snapshotAvailability,
    chatAvailability,
    traceData,
    selectedTraceRef,
    selectedTrace,
    activeRuns,
    savedRuns,
    loadingActiveRuns,
    loadingRuns,
    activeRunId,
    deletingRunIds,
    workspaceSelection,
    runsFilter,
    loadingChat,
    chatMessages,
    chatQuestion,
    chatting,
    canCancelChat,
    clearingChat,
    formatTimestamp,
    formatPeriod,
    phaseLabel,
    livePhase,
    liveProgress,
    runTargetLabel,
    statusTone,
    traceRefOrigin,
    reportLines,
    onChangeCompanionTab,
    onSelectTraceRef,
    onShowSelectedTraceInSource,
    onFocusTraceRef,
    onAskQuestion,
    onCancelChat,
    onClearChat,
    onChangeChatQuestion,
    onChangeRunsFilter,
    onRefreshActiveRuns,
    onRefreshRuns,
    onOpenRun,
    onCancelRun,
    onDeleteRun,
  }: {
    companionTab: CompanionTab;
    currentRun: AnalysisRunDetail | null;
    snapshotAvailability: RunSnapshotAvailability;
    chatAvailability: ChatAvailability;
    traceData: AnalysisTraceData;
    selectedTraceRef: string | null;
    selectedTrace: AnalysisTraceRef | null;
    activeRuns: AnalysisRunSummary[];
    savedRuns: AnalysisRunSummary[];
    loadingActiveRuns: boolean;
    loadingRuns: boolean;
    activeRunId: number | null;
    deletingRunIds: Record<number, boolean>;
    workspaceSelection: WorkspaceSelection;
    runsFilter: CompanionRunsFilterState;
    loadingChat: boolean;
    chatMessages: AnalysisChatTurn[];
    chatQuestion: string;
    chatting: boolean;
    canCancelChat: boolean;
    clearingChat: boolean;
    formatTimestamp: (timestamp: number | null) => string;
    formatPeriod: (periodFromUnix: number, periodToUnix: number) => string;
    phaseLabel: (phase: string) => string;
    livePhase: (runId: number) => string;
    liveProgress: (runId: number) => string;
    runTargetLabel: (run: Pick<AnalysisRunSummary, "scope_type" | "source_id" | "source_title" | "source_group_id" | "source_group_name" | "scope_label">) => string;
    statusTone: (status: string) => BadgeVariant;
    traceRefOrigin: (ref: string) => string;
    reportLines: (text: string) => Array<{ key: string; segments: Array<{ type: "text" | "ref"; value: string; key: string }> }>;
    onChangeCompanionTab: (tab: CompanionTab) => void;
    onSelectTraceRef: (ref: string) => void | Promise<void>;
    onShowSelectedTraceInSource: () => void | Promise<void>;
    onFocusTraceRef: (ref: string) => void | Promise<void>;
    onAskQuestion: () => void | Promise<void>;
    onCancelChat: () => void | Promise<void>;
    onClearChat: () => void | Promise<void>;
    onChangeChatQuestion: (value: string) => void;
    onChangeRunsFilter: (filter: CompanionRunsFilterState) => void;
    onRefreshActiveRuns: () => void | Promise<void>;
    onRefreshRuns: () => void | Promise<void>;
    onOpenRun: (runId: number) => void | Promise<void>;
    onCancelRun: (runId: number) => void | Promise<void>;
    onDeleteRun: (run: AnalysisRunSummary) => void | Promise<void>;
  } = $props();

  function tabId(tab: CompanionTab) {
    return `run-companion-tab-${tab}`;
  }
</script>

<aside class="run-companion-tabs">
  <div class="companion-header">
    <div>
      <span class="eyebrow">Companion</span>
      <h3>{currentRun ? `Run #${currentRun.id}` : "Runs"}</h3>
    </div>
    <div class="companion-tab-list" role="tablist" aria-label="Run companion tabs">
      <Button id={tabId("evidence")} role="tab" size="sm" variant="secondary" selected={companionTab === "evidence"} ariaSelected={companionTab === "evidence"} ariaControls="run-companion-panel" onclick={() => onChangeCompanionTab("evidence")}>Evidence</Button>
      <Button id={tabId("chat")} role="tab" size="sm" variant="secondary" selected={companionTab === "chat"} ariaSelected={companionTab === "chat"} ariaControls="run-companion-panel" onclick={() => onChangeCompanionTab("chat")}>Chat</Button>
      <Button id={tabId("runs")} role="tab" size="sm" variant="secondary" selected={companionTab === "runs"} ariaSelected={companionTab === "runs"} ariaControls="run-companion-panel" onclick={() => onChangeCompanionTab("runs")}>Runs</Button>
    </div>
  </div>

  <div id="run-companion-panel" class="companion-panel" role="tabpanel" aria-labelledby={tabId(companionTab)}>
    {#if companionTab === "evidence"}
      <RunEvidenceTab
        {currentRun}
        {traceData}
        {selectedTraceRef}
        {selectedTrace}
        {snapshotAvailability}
        {formatTimestamp}
        {traceRefOrigin}
        onSelectTraceRef={onSelectTraceRef}
        onShowSelectedTraceInSource={onShowSelectedTraceInSource}
      />
    {:else if companionTab === "chat"}
      <RunChatTab
        {currentRun}
        {chatAvailability}
        {loadingChat}
        {chatMessages}
        {chatQuestion}
        {chatting}
        {canCancelChat}
        {clearingChat}
        {selectedTraceRef}
        {reportLines}
        onFocusTraceRef={onFocusTraceRef}
        onAskQuestion={onAskQuestion}
        onCancelChat={onCancelChat}
        onClearChat={onClearChat}
        onChangeChatQuestion={onChangeChatQuestion}
      />
    {:else}
      <RunCompanionRunsTab
        {activeRuns}
        savedRuns={savedRuns}
        {loadingActiveRuns}
        {loadingRuns}
        {activeRunId}
        {deletingRunIds}
        {workspaceSelection}
        {runsFilter}
        {formatTimestamp}
        {formatPeriod}
        {phaseLabel}
        {livePhase}
        {liveProgress}
        {runTargetLabel}
        {statusTone}
        onChangeRunsFilter={onChangeRunsFilter}
        onRefreshActiveRuns={onRefreshActiveRuns}
        onRefreshRuns={onRefreshRuns}
        onOpenRun={onOpenRun}
        onCancelRun={onCancelRun}
        onDeleteRun={onDeleteRun}
      />
    {/if}
  </div>
</aside>

<style>
  .run-companion-tabs {
    position: sticky;
    top: 0;
    min-width: 0;
    max-height: calc(100vh - 4.75rem);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
    box-shadow: var(--shadow);
  }

  .companion-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.9rem;
    border-bottom: 1px solid color-mix(in srgb, var(--border) 76%, transparent);
  }

  .companion-header h3 {
    margin: 0;
  }

  .eyebrow {
    display: inline-block;
    margin-bottom: 0.2rem;
    color: var(--muted);
    font-size: 0.68rem;
    text-transform: uppercase;
  }

  .companion-tab-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    justify-content: flex-end;
  }

  .companion-panel {
    min-width: 0;
    min-height: 18rem;
    overflow: auto;
    padding: 0.9rem;
  }

  @media (max-width: 1500px) {
    .run-companion-tabs {
      position: static;
      max-height: none;
    }

    .companion-panel {
      overflow: visible;
    }
  }

  @media (max-width: 720px) {
    .companion-header {
      flex-direction: column;
    }

    .companion-tab-list {
      justify-content: flex-start;
    }
  }
</style>
```

- [x] **Step 6: Run component raw-source tests and Svelte check**

Run:

```powershell
npm.cmd test -- src/lib/analysis-run-companion-tabs.test.ts
npm.cmd run check
```

Expected: PASS.

- [x] **Step 7: Commit components**

Run:

```powershell
git add src/lib/components/analysis/run-evidence-tab.svelte src/lib/components/analysis/run-chat-tab.svelte src/lib/components/analysis/run-companion-runs-tab.svelte src/lib/components/analysis/run-companion-tabs.svelte src/lib/components/analysis/trace-panel.svelte src/lib/analysis-run-companion-tabs.test.ts
git commit -m "feat: add analysis run companion tabs"
```

## Task 6: Wire Companion Tabs Into `/analysis`

**Files:**
- Modify: `src/routes/analysis/+page.svelte`
- Modify: `src/lib/analysis-trace-workflow.ts`
- Modify: `src/lib/analysis-trace-workflow.test.ts`
- Modify: `src/lib/analysis-chat-workflow.test.ts`
- Modify: `src/lib/components/analysis/report-canvas.svelte`

- [x] **Step 1: Update trace workflow patch contract**

In `src/lib/analysis-trace-workflow.ts`, replace:

```ts
export type AnalysisTraceWorkflowPatch = Partial<{
  inspectorMode: "trace";
```

with:

```ts
export type AnalysisTraceWorkflowPatch = Partial<{
  companionTab: "evidence";
```

In `focusTraceRef(...)`, replace:

```ts
deps.patch({ inspectorMode: "trace", selectedTraceRef: ref });
```

with:

```ts
deps.patch({ companionTab: "evidence", selectedTraceRef: ref });
```

In `src/lib/analysis-trace-workflow.test.ts`, update the trace focus test expectation:

```ts
expect(deps.patch).toHaveBeenCalledWith({
  companionTab: "evidence",
  selectedTraceRef: "s7-i1",
});
```

- [x] **Step 2: Remove temporary chat from ReportCanvas**

In `src/lib/components/analysis/report-canvas.svelte`, remove the `ChatPanel` import and the `.temporary-follow-up` block.

Remove these props from `ReportCanvas`:

```ts
chatMessages
chatQuestion
chatting
canCancelChat
clearingChat
loadingChat
onAskRunQuestion
onCancelChat
onClearChat
onChangeChatQuestion
```

Keep `onFocusTraceRef` because report refs still activate Evidence through the route.

- [x] **Step 3: Add route imports and state**

In `src/routes/analysis/+page.svelte`, replace:

```ts
import WorkspaceInspector from "$lib/components/analysis/workspace-inspector.svelte";
```

with:

```ts
import RunCompanionTabs from "$lib/components/analysis/run-companion-tabs.svelte";
```

Add imports:

```ts
import {
  chatAvailabilityForRun,
  evidenceSourceActionDecision,
  runsFilterDefaults,
  type CompanionRunsFilterState,
} from "$lib/analysis-run-companion-state";
import type { CompanionTab } from "$lib/analysis-workspace-state";
```

Remove the local `InspectorMode` type and `let inspectorMode = ...`.

Add route state near `runFilter` and `historyScope`:

```ts
let runsFilter = $state<CompanionRunsFilterState>(runsFilterDefaults());
```

Add derived chat availability:

```ts
const chatAvailability = $derived(chatAvailabilityForRun({
  currentRun,
  snapshotAvailability: runSnapshotAvailability,
}));
```

- [x] **Step 4: Update route patch handlers**

In `applyTraceWorkflowPatch(...)`, replace the inspector patch handling:

```ts
if ("inspectorMode" in patch && patch.inspectorMode) inspectorMode = patch.inspectorMode;
```

with:

```ts
if ("companionTab" in patch && patch.companionTab) {
  workspaceUiState = {
    ...workspaceUiState,
    companionTab: patch.companionTab,
  };
}
```

In `applyRunWorkflowPatch(...)`, remove `inspectorMode` assignments. When a run opens through `alignWorkspaceToOpenedRun(...)`, rely on `openRunWorkspaceState(...)` and Part 6 helper defaults.

In source/group selection handlers, keep the Part 1 rule that workspace switches set `companionTab = "runs"`.

- [x] **Step 5: Add companion route helpers**

Add these helpers near the other workspace state helpers:

```ts
function changeCompanionTab(nextTab: CompanionTab) {
  workspaceUiState = {
    ...workspaceUiState,
    companionTab: nextTab,
  };
}

async function focusTraceRef(ref: string) {
  changeCompanionTab("evidence");
  await traceWorkflow.focusTraceRef(ref);
}

function showSelectedTraceInSource() {
  const decision = evidenceSourceActionDecision({
    currentRun,
    selectedTrace,
    snapshotAvailability: runSnapshotAvailability,
  });

  if (decision.kind === "unavailable") {
    status = decision.reason;
    return;
  }

  selectedTraceRef = decision.highlightedRef;
  workspaceUiState = {
    ...workspaceUiState,
    canvasMode: decision.canvasMode,
    sourceViewBasis: decision.sourceViewBasis,
    companionTab: "evidence",
  };

  if (decision.kind === "live_source") {
    status = decision.warning;
  }
}

async function submitRunQuestionFromCompanion() {
  const availability = chatAvailabilityForRun({
    currentRun,
    snapshotAvailability: runSnapshotAvailability,
  });

  if (!availability.enabled) {
    status = availability.description;
    return;
  }

  if (!chatQuestion.trim()) {
    status = "Question cannot be empty.";
    return;
  }

  changeCompanionTab("chat");
  await chatWorkflow.askRunQuestion();
}

function changeRunsFilter(next: CompanionRunsFilterState) {
  runsFilter = next;
  historyScope = next.scope;
}
```

This route code activates Chat only from question submission or explicit tab selection through `changeCompanionTab("chat")`. The chat textarea focus path does not call this helper.

- [x] **Step 6: Persist Runs filters**

Where the route calls `persistableAnalysisWorkspaceState(...)`, pass:

```ts
{
  historyScope,
  runFilter,
  runsFilter,
}
```

When restoring persisted state, add:

```ts
runsFilter = persisted.runs.runsFilter;
historyScope = persisted.runs.runsFilter.scope;
```

If existing route logic still needs `runFilter` for compatibility, keep it synchronized:

```ts
runFilter =
  persisted.runs.runsFilter.status === "cancelled" ||
  persisted.runs.runsFilter.status === "queued_running"
    ? "all"
    : persisted.runs.runsFilter.status;
```

- [x] **Step 7: Replace right panel markup**

Replace the `<div class="inspector-slot">...</div>` block with:

```svelte
<div class="companion-slot">
  <RunCompanionTabs
    companionTab={workspaceUiState.companionTab}
    {currentRun}
    snapshotAvailability={runSnapshotAvailability}
    {chatAvailability}
    {traceData}
    {selectedTraceRef}
    {selectedTrace}
    {activeRuns}
    savedRuns={runs}
    {loadingActiveRuns}
    {loadingRuns}
    {activeRunId}
    {deletingRunIds}
    workspaceSelection={workspaceUiState.workspaceSelection}
    {runsFilter}
    {loadingChat}
    {chatMessages}
    {chatQuestion}
    {chatting}
    canCancelChat={chatting && activeChatRequestId !== null}
    {clearingChat}
    {formatTimestamp}
    {formatPeriod}
    {phaseLabel}
    {livePhase}
    {liveProgress}
    {runTargetLabel}
    {statusTone}
    {traceRefOrigin}
    {reportLines}
    onChangeCompanionTab={changeCompanionTab}
    onSelectTraceRef={(ref) => void focusTraceRef(ref)}
    onShowSelectedTraceInSource={showSelectedTraceInSource}
    onFocusTraceRef={(ref) => void focusTraceRef(ref)}
    onAskQuestion={() => void submitRunQuestionFromCompanion()}
    onCancelChat={() => void cancelChat()}
    onClearChat={() => void clearChatMessages()}
    onChangeChatQuestion={(value) => (chatQuestion = value)}
    onChangeRunsFilter={changeRunsFilter}
    onRefreshActiveRuns={() => void loadActiveRuns()}
    onRefreshRuns={() => void loadRuns()}
    onOpenRun={(runId) => void openRun(runId)}
    onCancelRun={(runId) => void cancelActiveRun(runId)}
    onDeleteRun={(run) => void deleteSavedRun(run)}
  />
</div>
```

Update route CSS:

```css
.companion-slot {
  min-width: 0;
}

@media (max-width: 1500px) {
  .companion-slot {
    grid-column: 2;
  }
}

@media (max-width: 1180px) {
  .companion-slot {
    grid-column: 1;
  }
}
```

Remove `.inspector-slot` rules.

- [x] **Step 8: Update `ReportCanvas` route props**

Remove these props and callbacks from `<ReportCanvas ... />`:

```svelte
{chatMessages}
{chatQuestion}
{chatting}
canCancelChat={chatting && activeChatRequestId !== null}
{clearingChat}
{loadingChat}
onAskRunQuestion={() => void askRunQuestion()}
onCancelChat={() => void cancelChat()}
onClearChat={() => void clearChatMessages()}
onChangeChatQuestion={(value) => (chatQuestion = value)}
```

Keep:

```svelte
onFocusTraceRef={focusTraceRef}
```

- [x] **Step 9: Run route and workflow tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-run-companion-route.test.ts src/lib/analysis-run-companion-tabs.test.ts src/lib/analysis-trace-workflow.test.ts src/lib/analysis-chat-workflow.test.ts src/lib/analysis-workspace-persistence.test.ts
```

Expected: PASS.

- [x] **Step 10: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: PASS.

- [x] **Step 11: Commit route wiring**

Run:

```powershell
git add src/routes/analysis/+page.svelte src/lib/components/analysis/report-canvas.svelte src/lib/analysis-trace-workflow.ts src/lib/analysis-trace-workflow.test.ts src/lib/analysis-chat-workflow.test.ts src/lib/analysis-run-companion-route.test.ts
git commit -m "feat: wire analysis run companion tabs"
```

## Task 7: Run Part 6 Verification

**Files:**
- Verify all Part 6 files changed in Tasks 1-6.

- [x] **Step 1: Run focused frontend tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-run-companion-state.test.ts src/lib/analysis-run-companion-tabs.test.ts src/lib/analysis-run-companion-route.test.ts src/lib/analysis-workspace-persistence.test.ts src/lib/analysis-trace-workflow.test.ts src/lib/analysis-chat-workflow.test.ts
```

Expected: PASS.

- [x] **Step 2: Run relevant route and canvas tests from earlier parts**

Run:

```powershell
npm.cmd test -- src/lib/analysis-report-canvas-state.test.ts src/lib/analysis-report-canvas.test.ts src/lib/analysis-report-canvas-route.test.ts src/lib/analysis-source-readers.test.ts src/lib/analysis-source-readers-route.test.ts src/lib/analysis-route-workspace-state.test.ts src/lib/analysis-workspace-state.test.ts
```

Expected: PASS.

- [x] **Step 3: Run backend context-safety tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::chat::tests::completed_chat_context_requires_saved_snapshot_messages analysis::chat::tests::completed_chat_context_accepts_saved_snapshot_messages analysis::corpus::tests::trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot
```

Expected: PASS.

- [x] **Step 4: Run Svelte and TypeScript checks**

Run:

```powershell
npm.cmd run check
```

Expected: PASS.

- [x] **Step 5: Run full frontend test suite**

Run:

```powershell
npm.cmd test
```

Expected: PASS.

- [x] **Step 6: Run full backend tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [x] **Step 7: Check staged redesign boundaries**

Run:

```powershell
rg -n "WorkspaceInspector|inspectorMode|temporary-follow-up|<ChatPanel|load_run_corpus_messages\\(&pool, &run\\)" src/routes/analysis/+page.svelte src/lib/components/analysis/report-canvas.svelte src/lib/analysis-*.ts src-tauri/src/analysis/chat.rs
```

Expected:

- no `WorkspaceInspector` in `src/routes/analysis/+page.svelte`;
- no route-owned `inspectorMode`;
- no `temporary-follow-up` in `ReportCanvas`;
- no direct `<ChatPanel` in `ReportCanvas`;
- no completed-run chat path using `load_run_corpus_messages(&pool, &run)`.

Run:

```powershell
rg -n "SourceJobRecord|takeoutJobsBySource=|sourceJobsBySource=" src/lib/components/analysis/run-companion*.svelte src/lib/components/analysis/run-evidence-tab.svelte src/lib/components/analysis/run-chat-tab.svelte
```

Expected: no source ingest job types or source ingest job props in companion tab components.

Run:

```powershell
rg -n "sourceViewBasis: \"run_snapshot\", // automatic|listSourceItems\\(\\{ runId|completed.*live source|live source.*completed" src/routes/analysis/+page.svelte src/lib/components/analysis src/lib/analysis-run-companion-state.ts src-tauri/src/analysis
```

Expected: no output for automatic snapshot switching, live-source snapshot fallback, or completed-run live fallback language.

- [x] **Step 8: Check whitespace**

Run:

```powershell
git diff --check
```

Expected: no output and exit code 0.

- [x] **Step 9: Commit final fixes if needed**

If verification required fixes, commit them:

```powershell
git add src src-tauri
git commit -m "test: verify analysis run companion tabs"
```

Skip this commit if Tasks 1-6 already have a clean verified tree.

- [x] **Step 10: Stop for review**

Run:

```powershell
git status --short
```

Expected: clean working tree.

Report:

```text
Part 6 run companion tabs are implemented and verified. Stopping before Part 7.
```

Do not begin Part 7 until the user explicitly approves continuing.

## Self-Review

- Spec coverage: this plan covers the approved `RunCompanionTabs` responsibilities: Evidence default for completed runs, trace click activation, explicit Chat activation, snapshot-safe chat availability, `Show in source`, and Runs search/filter/current-scope behavior.
- Boundary check: this plan intentionally does not redesign the global shell, does not change Part 5 readers beyond selected-ref/source-basis wiring, does not include source ingest jobs in Runs, and does not perform the final integration sweep.
- Snapshot trust: completed-run chat uses saved snapshot rows only. Completed-run evidence source resolution degrades when snapshot rows are missing instead of resolving against live source data.
- Persistence: durable Runs filters are persisted; opened runs, selected trace refs, draft chat questions, source filters, popovers, and scroll positions remain transient.
- Test coverage: helper tests cover companion decisions, component raw-source tests cover tab structure, route tests cover wiring and activation rules, and backend tests cover no-live-fallback context safety.
