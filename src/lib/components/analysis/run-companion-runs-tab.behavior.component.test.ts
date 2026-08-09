import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import { runsFilterDefaults } from "$lib/analysis-run-companion-state";
import type { AnalysisRunSummary } from "$lib/types/analysis";
import RunCompanionRunsTab from "./run-companion-runs-tab.svelte";

afterEach(cleanup);

function run(overrides: Partial<AnalysisRunSummary> = {}): AnalysisRunSummary {
  return {
    id: 31,
    run_type: "report",
    scope_type: "single_source",
    source_id: 7,
    source_title: "Research channel",
    source_group_id: null,
    source_group_name: null,
    project_id: null,
    project_name: null,
    scope_label: "Research channel",
    period_from: 1_700_000_000,
    period_to: 1_700_086_400,
    output_language: "en",
    prompt_template_id: 4,
    prompt_template_name: "Daily brief",
    prompt_template_version: 2,
    provider_profile: "research",
    provider: "openai",
    model: "gpt-research",
    youtube_corpus_mode: "transcript_only",
    telegram_history_scope: "current",
    status: "completed",
    error: null,
    has_trace_data: true,
    snapshot_state: "captured",
    snapshot_captured_at: "2026-08-09T10:00:00Z",
    snapshot_error: null,
    created_at: 1_700_090_000,
    completed_at: 1_700_090_100,
    ...overrides,
  };
}

function props(overrides: Record<string, unknown> = {}) {
  return {
    activeRuns: [],
    savedRuns: [],
    loadingActiveRuns: false,
    loadingRuns: false,
    activeRunId: null,
    deletingRunIds: {},
    workspaceSelection: { kind: "source", sourceId: 7 } as const,
    runsFilter: runsFilterDefaults(),
    formatTimestamp: (value: number | null) => `created:${value}`,
    formatPeriod: (from: number, to: number) => `${from}-${to}`,
    phaseLabel: (value: string) => value,
    livePhase: () => "streaming",
    liveProgress: () => "2/4 chunks",
    runTargetLabel: () => "Research channel",
    statusTone: () => "neutral" as const,
    onChangeRunsFilter: vi.fn(),
    onRefreshActiveRuns: vi.fn(),
    onRefreshRuns: vi.fn(),
    onOpenRun: vi.fn(),
    onCancelRun: vi.fn(),
    onDeleteRun: vi.fn(),
    ...overrides,
  };
}

describe("analysis priority UX contract", () => {
  it("keeps run filters progressive when no runs exist", () => {
    render(RunCompanionRunsTab, { props: props() });

    expect(screen.queryByRole("searchbox", { name: "Search runs" })).toBeNull();
    expect(screen.queryByRole("group", { name: "Runs scope" })).toBeNull();
    expect(screen.queryByText("Advanced filters")).toBeNull();
    expect(screen.getByText(/Completed reports will appear here/)).toBeTruthy();
  });
});

describe("analysis redesign final safety contract", () => {
  it("keeps source ingest activity out of analysis Runs", async () => {
    const onChangeRunsFilter = vi.fn();
    const onOpenRun = vi.fn();
    const onDeleteRun = vi.fn();
    render(RunCompanionRunsTab, {
      props: props({ savedRuns: [run()], onChangeRunsFilter, onOpenRun, onDeleteRun }),
    });

    expect(screen.getByRole("searchbox", { name: "Search runs" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Current scope" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "All runs" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Completed" })).toBeTruthy();
    expect(screen.getByText("Research channel")).toBeTruthy();
    expect(screen.getByText(/openai\/gpt-research/)).toBeTruthy();
    expect(screen.getByText(/Daily brief v2/)).toBeTruthy();
    expect(screen.getByText(/Period:/)).toBeTruthy();
    expect(screen.queryByText(/Takeout import/i)).toBeNull();
    expect(screen.queryByText(/Sync transcript/i)).toBeNull();
    expect(screen.queryByText(/source job/i)).toBeNull();
    await fireEvent.input(screen.getByRole("searchbox", { name: "Search runs" }), { target: { value: "brief" } });
    expect(onChangeRunsFilter).toHaveBeenCalledWith(expect.objectContaining({ query: "brief" }));
    await fireEvent.click(screen.getByRole("button", { name: "Open" }));
    expect(onOpenRun).toHaveBeenCalledWith(31);
    await fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    expect(onDeleteRun).toHaveBeenCalledWith(expect.objectContaining({ id: 31 }));
  });
});
