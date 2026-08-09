import { cleanup, fireEvent, render, screen, within } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { AnalysisRunDetail } from "$lib/types/analysis";
import ReportRunHeader from "./report-run-header.svelte";

afterEach(cleanup);

function run(overrides: Partial<AnalysisRunDetail> = {}): AnalysisRunDetail {
  return {
    id: 30,
    run_type: "report",
    scope_type: "single_source",
    source_id: 7,
    source_title: null,
    source_group_id: null,
    source_group_name: null,
    project_id: null,
    project_name: null,
    scope_label: "Deleted research channel",
    period_from: 1_700_000_000,
    period_to: 1_700_086_400,
    output_language: "en",
    prompt_template_id: 4,
    prompt_template_name: "Daily brief",
    prompt_template_version: 2,
    provider_profile: "research",
    provider: "openai",
    model: "gpt-research",
    youtube_corpus_mode: "transcript_description",
    telegram_history_scope: "current",
    status: "completed",
    error: null,
    has_trace_data: true,
    snapshot_state: "captured",
    snapshot_captured_at: "2026-08-09T10:00:00Z",
    snapshot_error: null,
    created_at: 1_700_090_000,
    completed_at: 1_700_090_100,
    result_markdown: "# Report",
    ...overrides,
  };
}

function props(currentRun: AnalysisRunDetail, overrides: Record<string, unknown> = {}) {
  return {
    currentRun,
    sourceViewBasis: "run_snapshot" as const,
    snapshotAvailability: "available" as const,
    snapshotProbeState: "available" as const,
    traceRefCount: 6,
    activePhase: "",
    activeProgress: "",
    canCancelCurrentRun: false,
    formatTimestamp: (value: number | null) => `time:${value}`,
    formatPeriod: () => "Aug 1-Aug 2",
    runTargetLabel: (value: AnalysisRunDetail) => value.scope_label,
    statusTone: () => "success" as const,
    onCancelCurrentRun: vi.fn(),
    ...overrides,
  };
}

describe("analysis redesign final route contract", () => {
  it("keeps report setup out of the primary opened-run reading surface", async () => {
    const onCancelCurrentRun = vi.fn();
    render(ReportRunHeader, { props: props(run(), { canCancelCurrentRun: true, onCancelCurrentRun }) });
    const header = screen.getByRole("region", { name: "Opened run metadata" });

    expect(within(header).getByRole("heading", { name: "Run #30" })).toBeTruthy();
    expect(within(header).getAllByText("Deleted research channel")).toHaveLength(2);
    expect(within(header).getAllByText("completed")).toHaveLength(3);
    expect(within(header).getAllByText("Snapshot available")).toHaveLength(2);
    expect(within(header).getByText("Aug 1-Aug 2")).toBeTruthy();
    expect(within(header).getByText("Daily brief v2")).toBeTruthy();
    expect(within(header).getByText("openai/gpt-research")).toBeTruthy();
    expect(within(header).getByText("6")).toBeTruthy();
    expect(within(header).getByText("Run details")).toBeTruthy();
    expect(within(header).queryByRole("button", { name: "Run report" })).toBeNull();
    expect(within(header).queryByLabelText("Prompt template")).toBeNull();
    expect(within(header).queryByLabelText("LLM profile")).toBeNull();
    expect(within(header).queryByText("Template editor")).toBeNull();
    expect(within(header).getByRole("button", { name: "Cancel run" })).toBeTruthy();
    await fireEvent.click(within(header).getByRole("button", { name: "Cancel run" }));
    expect(onCancelCurrentRun).toHaveBeenCalledOnce();
  });
});

describe("analysis redesign final safety contract", () => {
  it("surfaces saved Telegram historical scope instead of treating it as ordinary current history", async () => {
    const view = render(ReportRunHeader, {
      props: props(run({ telegram_history_scope: "current_plus_migrated" })),
    });

    await fireEvent.click(screen.getByText("Run details"));
    expect(screen.getByText("Telegram history")).toBeTruthy();
    expect(screen.getByText("Current + migrated historical scope")).toBeTruthy();
    await view.rerender(props(run({ telegram_history_scope: "current" })));
    expect(screen.queryByText("Telegram history")).toBeNull();
  });
});
