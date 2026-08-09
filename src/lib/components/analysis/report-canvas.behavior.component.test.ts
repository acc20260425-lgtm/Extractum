import { cleanup, fireEvent, render, screen, within } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { ComponentProps } from "svelte";
import type { AnalysisRunDetail } from "$lib/types/analysis";
import ReportCanvas from "./report-canvas.svelte";

afterEach(cleanup);

type Props = ComponentProps<typeof ReportCanvas>;

function run(): AnalysisRunDetail {
  return {
    id: 30, run_type: "report", scope_type: "single_source", source_id: 7,
    source_title: "Research channel", source_group_id: null, source_group_name: null,
    project_id: null, project_name: null, scope_label: "Research channel",
    period_from: 1, period_to: 2, output_language: "en", prompt_template_id: 4,
    prompt_template_name: "Daily brief", prompt_template_version: 2,
    provider_profile: "research", provider: "openai", model: "gpt-research",
    youtube_corpus_mode: "transcript_only", telegram_history_scope: "current",
    status: "completed", error: null, has_trace_data: true, snapshot_state: "captured",
    snapshot_captured_at: "2026-08-09T10:00:00Z", snapshot_error: null,
    created_at: 3, completed_at: 4, result_markdown: "Saved report body",
  };
}

function props(overrides: Partial<Props> = {}): Props {
  return {
    workspaceSelection: { kind: "none" }, currentSource: null, takeoutRecovery: null,
    currentGroup: null, currentSourceMetric: null, currentScopeTitle: "Analysis workspace",
    currentScopeSummary: "Choose source material for a report.", canvasMode: "report",
    sourceViewBasis: "live_source", runSnapshotAvailability: "unknown", snapshotProbeState: "unknown",
    runSnapshotMessages: [], loadingRunSnapshotMessages: false, runSnapshotError: "",
    hasMoreRunSnapshotMessages: false, youtubeTranscriptSegments: [], loadingYoutubeTranscriptSegments: false,
    youtubeTranscriptHasMore: false, youtubeTranscriptSearch: "", groupLiveItemsBySource: {},
    groupLiveTranscriptSegmentsBySource: {}, groupLiveHasMoreBySource: {}, selectedGroupSourceId: null,
    selectedSnapshotSourceId: null, periodFrom: "2026-08-01", periodTo: "2026-08-09",
    selectedTemplateId: "", loadingTemplates: false, templates: [], outputLanguage: "English",
    youtubeCorpusMode: "transcript_only", includeMigratedHistory: false, canIncludeMigratedHistory: false,
    llmProfiles: [], activeLlmProfile: "", selectedLlmProfileId: "", selectedLlmModel: "__profile_default__",
    customModelOverride: "", llmProviderModels: [], loadingLlmProviderModels: false, llmModelStatus: "",
    startingReport: false, selectedGroupEditorId: "", currentScopeHasSavedRuns: false, currentRun: null,
    chatAvailability: { enabled: false, reason: "no_run", title: "No report selected", description: "Open a report." },
    loadingRunDetail: false, selectedRunIsActive: false, activeProgress: "", activePhase: "",
    focusedStreamedOutput: "", canCancelCurrentRun: false, sourceItems: [], sourceItemsError: null,
    sourceItemsHasMore: false, loadingItems: false, sourceTopics: [], loadingSourceTopics: false,
    selectedTopicKey: "__all_topics__", showTopicSelector: false, telegramHistoryScope: "current",
    selectedTraceRef: null, highlightToken: null, sourceReturnContext: null, traceRefCount: 0,
    selectedTemplate: null, templateName: "", templateBody: "", savingTemplate: false, deletingTemplate: false,
    groups: [], groupName: "", groupSourceType: "telegram", groupMemberSourceIds: [], selectedGroup: null,
    savingGroup: false, deletingGroup: false, sourceMetricsList: [], syncingIds: {}, sourceJobs: [],
    youtubeVideoDetail: null, youtubePlaylistDetail: null, youtubeDetailError: null, loadingYoutubeDetail: false,
    formatTimestamp: (value) => `time:${value}`, formatPeriod: () => "Aug 1-Aug 9",
    runTargetLabel: (value) => value.scope_label, statusTone: () => "neutral",
    reportLines: (value) => [{ key: "line", segments: [{ type: "text", value, key: "text" }] }],
    phaseLabel: (value) => value, accountLabel: () => "Research account", sourceSyncDisabledReason: () => null,
    reportLaunchDisabledReason: "Choose source material before running a report.", startOfDayUnix: () => 1,
    endOfDayUnix: () => 2, isGroupSourceSelected: () => false, onChangeCanvasMode: vi.fn(),
    onViewLiveSource: vi.fn(), onBackToRunSnapshot: vi.fn(), onReturnToEvidenceReview: vi.fn(),
    onLoadMoreRunSnapshotMessages: vi.fn(), onLoadMoreSourceItems: vi.fn(), onChangeTelegramHistoryScope: vi.fn(),
    onChangeTranscriptSearch: vi.fn(), onLoadMoreYoutubeTranscriptSegments: vi.fn(), onLoadLiveGroupSourcePage: vi.fn(),
    onChangeSelectedGroupSourceId: vi.fn(), onChangeSelectedSnapshotSourceId: vi.fn(), onChangeSelectedTopicKey: vi.fn(),
    onChangePeriodFrom: vi.fn(), onChangePeriodTo: vi.fn(), onChangeSelectedTemplateId: vi.fn(),
    onChangeOutputLanguage: vi.fn(), onChangeYoutubeCorpusMode: vi.fn(), onChangeIncludeMigratedHistory: vi.fn(),
    onChangeLlmProfile: vi.fn(), onChangeLlmModel: vi.fn(), onChangeCustomModelOverride: vi.fn(), onRunReport: vi.fn(),
    onSyncCurrentSource: vi.fn(), onStartTakeoutImport: vi.fn(), onStartMigratedHistoryImport: vi.fn(),
    onSyncYoutubeMetadata: vi.fn(), onSyncYoutubeTranscript: vi.fn(), onSyncYoutubeComments: vi.fn(),
    onSyncYoutubePlaylist: vi.fn(), onRetryFailedYoutubePlaylistVideos: vi.fn(), onSyncYoutubePlaylistVideo: vi.fn(),
    onRetryYoutubePlaylistVideo: vi.fn(), onCancelSourceJob: vi.fn(), onOpenSource: vi.fn(), exportDialogOpen: false,
    notebookLmExportForm: { outputDir: "", range: "entire_history", fromDate: "", toDate: "",
      includeMediaPlaceholders: true, includeMigratedHistory: false, minMessageLength: 3,
      maxWordsPerFile: 300_000, maxBytesPerFile: 50_000_000, overwriteExisting: false },
    notebookLmExportResult: null, notebookLmExportProgress: null, exportingNotebookLm: false,
    onOpenNotebookLmExport: vi.fn(), onCloseNotebookLmExport: vi.fn(), onChooseNotebookLmOutputDir: vi.fn(),
    onChangeNotebookLmExportForm: vi.fn(), onExportNotebookLm: vi.fn(), onFocusTraceRef: vi.fn(),
    onCancelCurrentRun: vi.fn(), onSaveTemplateCopy: vi.fn(), onSaveTemplateChanges: vi.fn(), onDeleteTemplate: vi.fn(),
    onChangeSelectedGroupId: vi.fn(), onChangeGroupName: vi.fn(), onChangeGroupSourceType: vi.fn(),
    onToggleGroupSource: vi.fn(), onStartNewGroup: vi.fn(), onSaveGroupCopy: vi.fn(), onSaveGroupChanges: vi.fn(),
    onDeleteGroup: vi.fn(), ...overrides,
  };
}

describe("analysis priority UX contract", () => {
  it("keeps the report canvas top chrome compact and action-oriented", async () => {
    const onChangeCanvasMode = vi.fn();
    const view = render(ReportCanvas, { props: props({ onChangeCanvasMode }) });
    const context = screen.getByLabelText("Analysis context");

    expect(context).toBeTruthy();
    expect(within(context).getByRole("heading", { name: "Analysis workspace" })).toBeTruthy();
    expect(within(context).getByText("Choose source material for a report.")).toBeTruthy();
    expect(within(context).getByRole("tablist", { name: "Report canvas mode" })).toBeTruthy();
    expect(within(context).getByRole("tab", { name: "Report" })).toBeTruthy();
    expect(within(context).getByRole("tab", { name: "Source" })).toBeTruthy();
    expect(within(context).getByRole("tab", { name: "Report" }).getAttribute("aria-selected")).toBe("true");
    expect(within(context).getByRole("tab", { name: "Source" }).getAttribute("aria-selected")).toBe("false");
    expect(within(context).getByRole("region", { name: "Workspace actions" })).toBeTruthy();
    expect(within(context).getByRole("button", { name: "Edit templates" })).toBeTruthy();
    expect(within(context).getByRole("button", { name: "Edit groups" })).toBeTruthy();
    expect(within(context).queryByRole("button", { name: "Export for NotebookLM" })).toBeNull();
    expect(within(context).queryByText("Workspace tools")).toBeNull();
    expect(screen.getByRole("region", { name: "Report setup" })).toBeTruthy();
    expect(screen.queryByRole("region", { name: "Opened run metadata" })).toBeNull();
    await fireEvent.click(within(context).getByRole("tab", { name: "Source" }));
    expect(onChangeCanvasMode).toHaveBeenCalledWith("source");
    await fireEvent.click(within(context).getByRole("button", { name: "Edit templates" }));
    expect(screen.getByLabelText("Template editor drawer")).toBeTruthy();
    await fireEvent.click(within(context).getByRole("button", { name: "Edit groups" }));
    expect(screen.getByLabelText("Source group editor drawer")).toBeTruthy();
    await view.rerender(props({ canvasMode: "source" }));
    expect(within(screen.getByLabelText("Analysis context")).getByRole("tab", { name: "Source" }).getAttribute("aria-selected")).toBe("true");
  });
});

describe("analysis redesign final route contract", () => {
  it("keeps ReportCanvas as the report/source mode owner", async () => {
    const onChangeCanvasMode = vi.fn();
    const view = render(ReportCanvas, { props: props({ onChangeCanvasMode }) });

    expect(screen.getByRole("tablist", { name: "Report canvas mode" })).toBeTruthy();
    expect(screen.getByRole("tab", { name: "Report" })).toBeTruthy();
    expect(screen.getByRole("tab", { name: "Source" })).toBeTruthy();
    expect(screen.getByRole("tab", { name: "Report" }).getAttribute("aria-selected")).toBe("true");
    expect(screen.getByRole("region", { name: "Report setup" })).toBeTruthy();
    await fireEvent.click(screen.getByRole("tab", { name: "Source" }));
    expect(onChangeCanvasMode).toHaveBeenCalledWith("source");
    await view.rerender(props({ canvasMode: "source" }));
    expect(screen.getByRole("tab", { name: "Source" }).getAttribute("aria-selected")).toBe("true");
    expect(screen.getByText("Select a source or source group to browse source material.")).toBeTruthy();
    expect(screen.queryByRole("region", { name: "Report setup" })).toBeNull();
  });

  it("keeps report setup out of the primary opened-run reading surface", () => {
    render(ReportCanvas, { props: props({ currentRun: run(), currentScopeTitle: "Research channel", focusedStreamedOutput: "Saved report body" }) });

    expect(screen.getByText("Run workspace")).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Research channel" })).toBeTruthy();
    expect(screen.getByText("Report and source basis stay side by side.")).toBeTruthy();
    expect(screen.getByRole("region", { name: "Opened run metadata" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Run #30" })).toBeTruthy();
    expect(screen.getByText("Daily brief v2")).toBeTruthy();
    expect(screen.getByText("openai/gpt-research")).toBeTruthy();
    expect(screen.getByText("Saved report body")).toBeTruthy();
    expect(screen.queryByRole("region", { name: "Report setup" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Run report" })).toBeNull();
    expect(screen.queryByRole("combobox", { name: "Prompt template" })).toBeNull();
    expect(screen.queryByRole("combobox", { name: "LLM profile" })).toBeNull();
    expect(screen.getByRole("tab", { name: "Report" }).getAttribute("aria-selected")).toBe("true");
    expect(screen.getByRole("tab", { name: "Source" }).getAttribute("aria-selected")).toBe("false");
    expect(screen.getByRole("region", { name: "Workspace actions" })).toBeTruthy();
  });
});
