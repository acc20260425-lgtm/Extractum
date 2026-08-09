import { cleanup, fireEvent, render, screen, within } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import { runsFilterDefaults } from "$lib/analysis-run-companion-state";
import RunCompanionTabs from "./run-companion-tabs.svelte";

afterEach(cleanup);

function props(overrides: Record<string, unknown> = {}) {
  return {
    companionTab: "runs" as const,
    currentRun: null,
    snapshotAvailability: "unknown" as const,
    snapshotProbeState: "unknown" as const,
    chatAvailability: { enabled: false, reason: "no_run", title: "No report selected", description: "Open a report." } as const,
    traceData: { refs: [] },
    selectedTraceRef: null,
    selectedTrace: null,
    focusedChunkSummaries: [],
    selectedRunIsActive: false,
    activeRuns: [],
    savedRuns: [],
    loadingActiveRuns: false,
    loadingRuns: false,
    activeRunId: null,
    deletingRunIds: {},
    workspaceSelection: { kind: "none" } as const,
    runsFilter: runsFilterDefaults(),
    loadingChat: false,
    chatMessages: [],
    chatQuestion: "",
    chatting: false,
    canCancelChat: false,
    clearingChat: false,
    formatTimestamp: () => "Never",
    formatPeriod: () => "No period",
    phaseLabel: (value: string) => value,
    livePhase: () => "",
    liveProgress: () => "",
    runTargetLabel: () => "Analysis report",
    statusTone: () => "neutral" as const,
    traceRefOrigin: () => "snapshot",
    reportLines: () => [],
    onChangeCompanionTab: vi.fn(),
    onSelectTraceRef: vi.fn(),
    onShowSelectedTraceInSource: vi.fn(),
    onFocusTraceRef: vi.fn(),
    onAskQuestion: vi.fn(),
    onCancelChat: vi.fn(),
    onClearChat: vi.fn(),
    onChangeChatQuestion: vi.fn(),
    onChangeRunsFilter: vi.fn(),
    onRefreshActiveRuns: vi.fn(),
    onRefreshRuns: vi.fn(),
    onOpenRun: vi.fn(),
    onCancelRun: vi.fn(),
    onDeleteRun: vi.fn(),
    ...overrides,
  };
}

describe("analysis redesign final route contract", () => {
  it("keeps companion tabs as Evidence, Chat, and Runs only", async () => {
    const onChangeCompanionTab = vi.fn();
    render(RunCompanionTabs, { props: props({ onChangeCompanionTab }) });
    const tabs = screen.getByRole("tablist", { name: "Run companion tabs" });

    expect(within(tabs).getByRole("tab", { name: "Evidence" })).toBeTruthy();
    expect(within(tabs).getByRole("tab", { name: "Chat" })).toBeTruthy();
    expect(within(tabs).getByRole("tab", { name: "Runs" })).toBeTruthy();
    expect(within(tabs).queryByRole("tab", { name: "Source activity" })).toBeNull();
    expect(within(tabs).getByRole("tab", { name: "Runs" }).getAttribute("aria-selected")).toBe("true");
    expect(screen.getByRole("tabpanel").getAttribute("aria-labelledby")).toBe("run-companion-tab-runs");
    expect(screen.getByText("Run a report to create the first saved workspace.")).toBeTruthy();
    await fireEvent.click(within(tabs).getByRole("tab", { name: "Evidence" }));
    expect(onChangeCompanionTab).toHaveBeenCalledWith("evidence");
    await fireEvent.click(within(tabs).getByRole("tab", { name: "Chat" }));
    expect(onChangeCompanionTab).toHaveBeenCalledWith("chat");
  });
});
