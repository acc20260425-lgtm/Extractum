import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, waitFor } from "@testing-library/svelte";
import { tick, type ComponentProps } from "svelte";
import ReportCanvas from "$lib/components/analysis/report-canvas.svelte";

const api = vi.hoisted(() => ({
  askAnalysisRunQuestion: vi.fn(),
  cancelAnalysisRun: vi.fn(),
  cancelLlmRequest: vi.fn(),
  cancelSourceJob: vi.fn(),
  cancelTakeoutSourceImport: vi.fn(),
  clearAnalysisChatMessages: vi.fn(),
  createAnalysisPromptTemplate: vi.fn(),
  createAnalysisSourceGroup: vi.fn(),
  deleteAnalysisPromptTemplate: vi.fn(),
  deleteAnalysisRun: vi.fn(),
  deleteAnalysisSourceGroup: vi.fn(),
  deleteSource: vi.fn(),
  exportSourceToNotebookLm: vi.fn(),
  getAnalysisRun: vi.fn().mockResolvedValue(null),
  getAnalysisRunTrace: vi.fn().mockResolvedValue({ refs: [] }),
  getLlmProfiles: vi.fn().mockResolvedValue({ active_profile: "", profiles: [] }),
  getWorkspaceAccountStatuses: vi.fn().mockResolvedValue([]),
  getYoutubePlaylistDetail: vi.fn().mockResolvedValue(null),
  getYoutubeRuntimeStatus: vi.fn().mockResolvedValue({
    ytdlpAvailable: false,
    ytdlpVersion: null,
    message: "YouTube runtime unavailable in component smoke",
  }),
  getYoutubeVideoDetail: vi.fn().mockResolvedValue(null),
  listActiveAnalysisRuns: vi.fn().mockResolvedValue([]),
  listAnalysisChatMessages: vi.fn().mockResolvedValue([]),
  listAnalysisPromptTemplates: vi.fn().mockResolvedValue([]),
  listAnalysisRunMessages: vi.fn().mockResolvedValue({
    messages: [],
    next_cursor: null,
    has_more: false,
  }),
  listAnalysisRuns: vi.fn().mockResolvedValue([]),
  listAnalysisSourceGroups: vi.fn().mockResolvedValue([]),
  listAnalysisSources: vi.fn().mockResolvedValue([]),
  listLlmProviderModels: vi.fn().mockResolvedValue([]),
  listSourceForumTopics: vi.fn().mockResolvedValue({
    topics: [],
    topicResolutionState: {
      status: "never_run",
      resolverVersion: 0,
      unresolvedCount: 0,
      pendingItemCount: 0,
      membershipsRefreshedAt: null,
    },
  }),
  listSourceItems: vi.fn().mockResolvedValue([]),
  listSourceJobs: vi.fn().mockResolvedValue([]),
  listSources: vi.fn().mockResolvedValue([]),
  listTakeoutImportRecoveryStates: vi.fn().mockResolvedValue([]),
  listTakeoutSourceImportJobs: vi.fn().mockResolvedValue([]),
  listWorkspaceAccounts: vi.fn().mockResolvedValue([]),
  listYoutubeSourceSummaries: vi.fn().mockResolvedValue([]),
  listYoutubeTranscriptSegments: vi.fn().mockResolvedValue({
    segments: [],
    nextCursor: null,
    hasMore: false,
  }),
  listenToAnalysisChatEvents: vi.fn(),
  listenToAnalysisRunEvents: vi.fn(),
  listenToNotebookLmExportEvents: vi.fn(),
  listenToSourceJobEvents: vi.fn(),
  listenToTakeoutImportEvents: vi.fn(),
  resolveAnalysisTraceRefs: vi.fn().mockResolvedValue([]),
  retryFailedYoutubePlaylistVideos: vi.fn(),
  startAnalysisReport: vi.fn(),
  startTakeoutMigratedHistoryImport: vi.fn(),
  startTakeoutSourceImport: vi.fn(),
  syncSource: vi.fn(),
  syncYoutubePlaylistVideo: vi.fn(),
  syncYoutubeSource: vi.fn(),
  unlistenAnalysisChat: vi.fn(),
  unlistenAnalysisRuns: vi.fn(),
  unlistenNotebookLmExport: vi.fn(),
  unlistenSourceJobs: vi.fn(),
  unlistenTakeoutImport: vi.fn(),
  updateAnalysisPromptTemplate: vi.fn(),
  updateAnalysisSourceGroup: vi.fn(),
}));

vi.mock("$lib/api/analysis-runs", () => ({
  cancelAnalysisRun: api.cancelAnalysisRun,
  deleteAnalysisRun: api.deleteAnalysisRun,
  getAnalysisRun: api.getAnalysisRun,
  listActiveAnalysisRuns: api.listActiveAnalysisRuns,
  listAnalysisRunMessages: api.listAnalysisRunMessages,
  listAnalysisRuns: api.listAnalysisRuns,
  listenToAnalysisRunEvents: api.listenToAnalysisRunEvents,
  startAnalysisReport: api.startAnalysisReport,
}));

vi.mock("$lib/api/analysis-chat", () => ({
  askAnalysisRunQuestion: api.askAnalysisRunQuestion,
  clearAnalysisChatMessages: api.clearAnalysisChatMessages,
  listAnalysisChatMessages: api.listAnalysisChatMessages,
  listenToAnalysisChatEvents: api.listenToAnalysisChatEvents,
}));

vi.mock("$lib/api/analysis-trace", () => ({
  getAnalysisRunTrace: api.getAnalysisRunTrace,
  resolveAnalysisTraceRefs: api.resolveAnalysisTraceRefs,
}));

vi.mock("$lib/api/analysis-workspace", () => ({
  getWorkspaceAccountStatuses: api.getWorkspaceAccountStatuses,
  listAnalysisSources: api.listAnalysisSources,
  listWorkspaceAccounts: api.listWorkspaceAccounts,
}));

vi.mock("$lib/api/analysis-source-groups", () => ({
  createAnalysisPromptTemplate: api.createAnalysisPromptTemplate,
  createAnalysisSourceGroup: api.createAnalysisSourceGroup,
  deleteAnalysisPromptTemplate: api.deleteAnalysisPromptTemplate,
  deleteAnalysisSourceGroup: api.deleteAnalysisSourceGroup,
  listAnalysisPromptTemplates: api.listAnalysisPromptTemplates,
  listAnalysisSourceGroups: api.listAnalysisSourceGroups,
  updateAnalysisPromptTemplate: api.updateAnalysisPromptTemplate,
  updateAnalysisSourceGroup: api.updateAnalysisSourceGroup,
}));

vi.mock("$lib/api/llm", () => ({
  cancelLlmRequest: api.cancelLlmRequest,
  getLlmProfiles: api.getLlmProfiles,
  listLlmProviderModels: api.listLlmProviderModels,
}));

vi.mock("$lib/api/takeout-import", () => ({
  cancelTakeoutSourceImport: api.cancelTakeoutSourceImport,
  listTakeoutImportRecoveryStates: api.listTakeoutImportRecoveryStates,
  listTakeoutSourceImportJobs: api.listTakeoutSourceImportJobs,
  listenToTakeoutImportEvents: api.listenToTakeoutImportEvents,
  startTakeoutMigratedHistoryImport: api.startTakeoutMigratedHistoryImport,
  startTakeoutSourceImport: api.startTakeoutSourceImport,
}));

vi.mock("$lib/api/source-jobs", () => ({
  cancelSourceJob: api.cancelSourceJob,
  listSourceJobs: api.listSourceJobs,
  listenToSourceJobEvents: api.listenToSourceJobEvents,
  retryFailedYoutubePlaylistVideos: api.retryFailedYoutubePlaylistVideos,
  syncYoutubePlaylistVideo: api.syncYoutubePlaylistVideo,
  syncYoutubeSource: api.syncYoutubeSource,
}));

vi.mock("$lib/api/youtube-detail", () => ({
  getYoutubePlaylistDetail: api.getYoutubePlaylistDetail,
  getYoutubeRuntimeStatus: api.getYoutubeRuntimeStatus,
  getYoutubeVideoDetail: api.getYoutubeVideoDetail,
  listYoutubeSourceSummaries: api.listYoutubeSourceSummaries,
}));

vi.mock("$lib/api/notebooklm-export", () => ({
  exportSourceToNotebookLm: api.exportSourceToNotebookLm,
  listenToNotebookLmExportEvents: api.listenToNotebookLmExportEvents,
}));

vi.mock("$lib/api/sources", () => ({
  deleteSource: api.deleteSource,
  listSourceForumTopics: api.listSourceForumTopics,
  listSourceItems: api.listSourceItems,
  listSources: api.listSources,
  listYoutubeTranscriptSegments: api.listYoutubeTranscriptSegments,
  syncSource: api.syncSource,
}));

type ReportCanvasProps = ComponentProps<typeof ReportCanvas>;

function reportCanvasProps(
  overrides: Partial<ReportCanvasProps> = {},
): ReportCanvasProps {
  return {
    workspaceSelection: { kind: "none" },
    currentSource: null,
    takeoutRecovery: null,
    currentGroup: null,
    currentSourceMetric: null,
    currentScopeTitle: "Analysis workspace",
    currentScopeSummary: "Choose source material for a report.",
    canvasMode: "report",
    sourceViewBasis: "live_source",
    runSnapshotAvailability: "unknown",
    snapshotProbeState: "unknown",
    runSnapshotMessages: [],
    loadingRunSnapshotMessages: false,
    runSnapshotError: "",
    hasMoreRunSnapshotMessages: false,
    youtubeTranscriptSegments: [],
    loadingYoutubeTranscriptSegments: false,
    youtubeTranscriptHasMore: false,
    youtubeTranscriptSearch: "",
    groupLiveItemsBySource: {},
    groupLiveTranscriptSegmentsBySource: {},
    groupLiveHasMoreBySource: {},
    selectedGroupSourceId: null,
    selectedSnapshotSourceId: null,
    periodFrom: "2026-08-01",
    periodTo: "2026-08-05",
    selectedTemplateId: "",
    loadingTemplates: false,
    templates: [],
    outputLanguage: "en",
    youtubeCorpusMode: "transcript_only",
    includeMigratedHistory: false,
    canIncludeMigratedHistory: false,
    llmProfiles: [],
    activeLlmProfile: "",
    selectedLlmProfileId: "",
    selectedLlmModel: "__profile_default__",
    customModelOverride: "",
    llmProviderModels: [],
    loadingLlmProviderModels: false,
    llmModelStatus: "",
    startingReport: false,
    selectedGroupEditorId: "",
    currentScopeHasSavedRuns: false,
    currentRun: null,
    chatAvailability: {
      enabled: false,
      reason: "no_run",
      title: "No report selected",
      description: "Open a report to use follow-up chat.",
    },
    loadingRunDetail: false,
    selectedRunIsActive: false,
    activeProgress: "",
    activePhase: "",
    focusedStreamedOutput: "",
    canCancelCurrentRun: false,
    sourceItems: [],
    sourceItemsError: null,
    sourceItemsHasMore: false,
    loadingItems: false,
    sourceTopics: [],
    loadingSourceTopics: false,
    selectedTopicKey: "__all_topics__",
    showTopicSelector: false,
    telegramHistoryScope: "current",
    selectedTraceRef: null,
    highlightToken: null,
    sourceReturnContext: null,
    traceRefCount: 0,
    selectedTemplate: null,
    templateName: "",
    templateBody: "",
    savingTemplate: false,
    deletingTemplate: false,
    groups: [],
    groupName: "",
    groupSourceType: "telegram",
    groupMemberSourceIds: [],
    selectedGroup: null,
    savingGroup: false,
    deletingGroup: false,
    sourceMetricsList: [],
    syncingIds: {},
    sourceJobs: [],
    youtubeVideoDetail: null,
    youtubePlaylistDetail: null,
    youtubeDetailError: null,
    loadingYoutubeDetail: false,
    formatTimestamp: (value) => value === null ? "Never" : `time:${value}`,
    formatPeriod: (from, to) => `${from}-${to}`,
    runTargetLabel: () => "Analysis report",
    statusTone: () => "neutral",
    reportLines: () => [],
    phaseLabel: (value) => value,
    accountLabel: (accountId) => accountId === null ? "No account" : `Account ${accountId}`,
    sourceSyncDisabledReason: () => null,
    reportLaunchDisabledReason: "Choose source material before running a report.",
    startOfDayUnix: () => 0,
    endOfDayUnix: () => 0,
    isGroupSourceSelected: () => false,
    onChangeCanvasMode: vi.fn(),
    onViewLiveSource: vi.fn(),
    onBackToRunSnapshot: vi.fn(),
    onReturnToEvidenceReview: vi.fn(),
    onLoadMoreRunSnapshotMessages: vi.fn(),
    onLoadMoreSourceItems: vi.fn(),
    onChangeTelegramHistoryScope: vi.fn(),
    onChangeTranscriptSearch: vi.fn(),
    onLoadMoreYoutubeTranscriptSegments: vi.fn(),
    onLoadLiveGroupSourcePage: vi.fn(),
    onChangeSelectedGroupSourceId: vi.fn(),
    onChangeSelectedSnapshotSourceId: vi.fn(),
    onChangeSelectedTopicKey: vi.fn(),
    onChangePeriodFrom: vi.fn(),
    onChangePeriodTo: vi.fn(),
    onChangeSelectedTemplateId: vi.fn(),
    onChangeOutputLanguage: vi.fn(),
    onChangeYoutubeCorpusMode: vi.fn(),
    onChangeIncludeMigratedHistory: vi.fn(),
    onChangeLlmProfile: vi.fn(),
    onChangeLlmModel: vi.fn(),
    onChangeCustomModelOverride: vi.fn(),
    onRunReport: vi.fn(),
    onSyncCurrentSource: vi.fn(),
    onStartTakeoutImport: vi.fn(),
    onStartMigratedHistoryImport: vi.fn(),
    onSyncYoutubeMetadata: vi.fn(),
    onSyncYoutubeTranscript: vi.fn(),
    onSyncYoutubeComments: vi.fn(),
    onSyncYoutubePlaylist: vi.fn(),
    onRetryFailedYoutubePlaylistVideos: vi.fn(),
    onSyncYoutubePlaylistVideo: vi.fn(),
    onRetryYoutubePlaylistVideo: vi.fn(),
    onCancelSourceJob: vi.fn(),
    onOpenSource: vi.fn(),
    exportDialogOpen: false,
    notebookLmExportForm: {
      outputDir: "",
      range: "entire_history",
      fromDate: "",
      toDate: "",
      includeMediaPlaceholders: true,
      includeMigratedHistory: false,
      minMessageLength: 3,
      maxWordsPerFile: 300_000,
      maxBytesPerFile: 50_000_000,
      overwriteExisting: false,
    },
    notebookLmExportResult: null,
    notebookLmExportProgress: null,
    exportingNotebookLm: false,
    onOpenNotebookLmExport: vi.fn(),
    onCloseNotebookLmExport: vi.fn(),
    onChooseNotebookLmOutputDir: vi.fn(),
    onChangeNotebookLmExportForm: vi.fn(),
    onExportNotebookLm: vi.fn(),
    onFocusTraceRef: vi.fn(),
    onCancelCurrentRun: vi.fn(),
    onSaveTemplateCopy: vi.fn(),
    onSaveTemplateChanges: vi.fn(),
    onDeleteTemplate: vi.fn(),
    onChangeSelectedGroupId: vi.fn(),
    onChangeGroupName: vi.fn(),
    onChangeGroupSourceType: vi.fn(),
    onToggleGroupSource: vi.fn(),
    onStartNewGroup: vi.fn(),
    onSaveGroupCopy: vi.fn(),
    onSaveGroupChanges: vi.fn(),
    onDeleteGroup: vi.fn(),
    ...overrides,
  };
}

beforeEach(() => {
  vi.clearAllMocks();
  localStorage.clear();
  api.listenToAnalysisRunEvents.mockResolvedValue(api.unlistenAnalysisRuns);
  api.listenToAnalysisChatEvents.mockResolvedValue(api.unlistenAnalysisChat);
  api.listenToNotebookLmExportEvents.mockResolvedValue(api.unlistenNotebookLmExport);
  api.listenToTakeoutImportEvents.mockResolvedValue(api.unlistenTakeoutImport);
  api.listenToSourceJobEvents.mockResolvedValue(api.unlistenSourceJobs);
});

afterEach(cleanup);

describe("report canvas render smoke", () => {
  it("smoke renders report canvas", () => {
    const view = render(ReportCanvas, { props: reportCanvasProps() });

    expect(
      view.container.querySelector('[data-smoke-id="analysis-report-canvas"]'),
    ).toBeTruthy();

    view.unmount();
  });
});

it("smoke renders analysis route", async () => {
  const { default: AnalysisPage } = await import("../routes/analysis/+page.svelte");
  const view = render(AnalysisPage);

  await waitFor(() => {
    expect(api.listWorkspaceAccounts).toHaveBeenCalledOnce();
    expect(api.listSources).toHaveBeenCalledWith(null);
    expect(api.listAnalysisSources).toHaveBeenCalledOnce();
    expect(api.listAnalysisSourceGroups).toHaveBeenCalledOnce();
    expect(api.listAnalysisPromptTemplates).toHaveBeenCalledWith("report");
    expect(api.listActiveAnalysisRuns).toHaveBeenCalledOnce();
    expect(api.getLlmProfiles).toHaveBeenCalledOnce();
    expect(api.getYoutubeRuntimeStatus).toHaveBeenCalledOnce();
    expect(api.listTakeoutSourceImportJobs).toHaveBeenCalledOnce();
    expect(api.listTakeoutImportRecoveryStates).toHaveBeenCalledOnce();
    expect(api.listSourceJobs).toHaveBeenCalledWith({ limit: 100 });
    expect(api.listenToAnalysisRunEvents).toHaveBeenCalledOnce();
    expect(api.listenToAnalysisChatEvents).toHaveBeenCalledOnce();
    expect(api.listenToNotebookLmExportEvents).toHaveBeenCalledOnce();
    expect(api.listenToTakeoutImportEvents).toHaveBeenCalledOnce();
    expect(api.listenToSourceJobEvents).toHaveBeenCalledOnce();
  });
  await tick();
  await Promise.resolve();

  expect(view.container.querySelector("section.analysis-workspace")).toBeTruthy();

  view.unmount();

  expect(api.unlistenAnalysisRuns).toHaveBeenCalledOnce();
  expect(api.unlistenAnalysisChat).toHaveBeenCalledOnce();
  expect(api.unlistenNotebookLmExport).toHaveBeenCalledOnce();
  expect(api.unlistenTakeoutImport).toHaveBeenCalledOnce();
  expect(api.unlistenSourceJobs).toHaveBeenCalledOnce();
});
