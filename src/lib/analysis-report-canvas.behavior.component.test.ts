import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { tick, type ComponentProps } from "svelte";
import ReportCanvas from "$lib/components/analysis/report-canvas.svelte";
import type {
  AnalysisRunDetail,
  AnalysisRunMessage,
  AnalysisSourceGroup,
} from "$lib/types/analysis";
import type {
  Source,
  SourceForumTopic,
  SourceItem,
  SourceJobRecord,
} from "$lib/types/sources";

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
const originalAnimate = Object.getOwnPropertyDescriptor(Element.prototype, "animate");

beforeAll(() => {
  Object.defineProperty(Element.prototype, "animate", {
    configurable: true,
    value: () => {
      const animation = {
        cancel: () => {},
        currentTime: 0,
        effect: null,
        onfinish: null,
        playState: "finished",
      } as unknown as Animation;
      setTimeout(() => animation.onfinish?.(new Event("finish") as AnimationPlaybackEvent), 0);
      return animation;
    },
  });
});

afterAll(() => {
  if (originalAnimate) {
    Object.defineProperty(Element.prototype, "animate", originalAnimate);
  } else {
    delete (Element.prototype as Partial<Element>).animate;
  }
});

function source(overrides: Partial<Source> = {}): Source {
  return {
    id: 1,
    sourceType: "telegram",
    sourceSubtype: "supergroup",
    accountId: 7,
    externalId: "source-1",
    title: "Research channel",
    lastSyncState: 1,
    lastSyncedAt: 1_700_000_000,
    isMember: true,
    isActive: true,
    createdAt: 1_699_000_000,
    telegramUsername: "research",
    avatarDataUrl: null,
    migratedHistoryStatus: "none",
    migratedHistoryDetectedAt: null,
    migratedHistoryRefreshedAt: null,
    migratedHistoryRowCount: 0,
    migratedHistoryImportCompleted: false,
    ...overrides,
  };
}

function sourceItem(overrides: Partial<SourceItem> = {}): SourceItem {
  return {
    id: 1,
    sourceId: 1,
    externalId: "live-item-1",
    itemKind: "rss_post",
    author: "Live author",
    publishedAt: 1_700_000_000,
    content: "Live-only source row",
    contentKind: "text",
    hasMedia: false,
    mediaKind: null,
    mediaSummary: null,
    mediaFileName: null,
    mediaMimeType: null,
    hasRawData: false,
    forumTopicId: null,
    forumTopicTitle: null,
    forumTopicTopMessageId: null,
    replyToMessageId: null,
    replyToPeerKind: null,
    replyToPeerId: null,
    replyToTopMessageId: null,
    reactionCount: null,
    historyScope: "current",
    isMigratedHistory: false,
    migrationDomain: null,
    historyScopeLabel: "Current supergroup history",
    pageCursor: "live-cursor-1",
    ...overrides,
  };
}

function group(overrides: Partial<AnalysisSourceGroup> = {}): AnalysisSourceGroup {
  return {
    id: 20,
    name: "Research group",
    source_type: "telegram",
    members: [
      { source_id: 1, source_title: "Research channel", item_count: 2 },
      { source_id: 2, source_title: "Research archive", item_count: 1 },
    ],
    created_at: 1_699_000_000,
    updated_at: 1_700_000_000,
    ...overrides,
  };
}

function run(overrides: Partial<AnalysisRunDetail> = {}): AnalysisRunDetail {
  return {
    id: 30,
    run_type: "report",
    scope_type: "project",
    source_id: null,
    source_title: null,
    source_group_id: null,
    source_group_name: null,
    project_id: 40,
    project_name: "Project Alpha",
    scope_label: "Project sources",
    period_from: 1_699_000_000,
    period_to: 1_700_000_000,
    output_language: "en",
    prompt_template_id: 1,
    prompt_template_name: "Research report",
    prompt_template_version: 3,
    provider_profile: "Default",
    provider: "openai",
    model: "model-a",
    youtube_corpus_mode: "transcript_only",
    telegram_history_scope: "current",
    status: "completed",
    error: null,
    has_trace_data: true,
    snapshot_state: "captured",
    snapshot_captured_at: "2026-08-03T10:00:00Z",
    snapshot_error: null,
    created_at: 1_700_000_000,
    completed_at: 1_700_000_100,
    result_markdown: "# Saved report",
    ...overrides,
  };
}

function snapshotMessage(overrides: Partial<AnalysisRunMessage> = {}): AnalysisRunMessage {
  return {
    item_id: 101,
    source_id: 1,
    external_id: "snapshot-item-1",
    author: "Snapshot author",
    published_at: 1_699_999_000,
    ref: "snapshot:1",
    content: "Frozen snapshot row",
    item_kind: "rss_post",
    source_type: "rss",
    source_subtype: "feed",
    metadata_json: null,
    ...overrides,
  };
}

function topic(overrides: Partial<SourceForumTopic> = {}): SourceForumTopic {
  return {
    kind: "topic",
    key: "topic:7",
    title: "Methods",
    messageCount: 9,
    topicId: 7,
    topMessageId: 70,
    iconColor: null,
    iconEmojiId: null,
    isClosed: false,
    isPinned: false,
    isHidden: false,
    isDeleted: false,
    sortOrder: 1,
    ...overrides,
  };
}

function sourceJob(overrides: Partial<SourceJobRecord> = {}): SourceJobRecord {
  return {
    job_id: "source-job-1",
    source_id: 1,
    related_source_id: null,
    job_type: "youtube_video_comments_sync",
    status: "running",
    message: "Syncing live comments",
    progress_current: 1,
    progress_total: 2,
    started_at: 1_700_000_000,
    finished_at: null,
    warnings: [],
    error: null,
    ...overrides,
  };
}

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

function renderReportCanvas(overrides: Partial<ReportCanvasProps> = {}) {
  return render(ReportCanvas, { props: reportCanvasProps(overrides) });
}

function exportForm(
  overrides: Partial<ReportCanvasProps["notebookLmExportForm"]> = {},
): ReportCanvasProps["notebookLmExportForm"] {
  return {
    ...reportCanvasProps().notebookLmExportForm,
    ...overrides,
  };
}

function expectBefore(first: Element, second: Element) {
  expect(
    first.compareDocumentPosition(second) & Node.DOCUMENT_POSITION_FOLLOWING,
  ).toBeTruthy();
}

function sourceBrowserBody() {
  const tabs = screen.getByRole("navigation", { name: "Source browser tabs" });
  const body = tabs.nextElementSibling;
  expect(body).toBeInstanceOf(HTMLElement);
  return body as HTMLElement;
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

describe("report canvas component contract", () => {
  it("owns the central Report and Source modes", async () => {
    const onChangeCanvasMode = vi.fn();
    const view = renderReportCanvas({ onChangeCanvasMode });

    const tablist = screen.getByRole("tablist", { name: "Report canvas mode" });
    const reportTab = within(tablist).getByRole("tab", { name: "Report" });
    const sourceTab = within(tablist).getByRole("tab", { name: "Source" });

    expect(reportTab.getAttribute("aria-selected")).toBe("true");
    expect(sourceTab.getAttribute("aria-selected")).toBe("false");
    expect(screen.getByRole("region", { name: "Report setup" })).toBeTruthy();
    expect(screen.queryByRole("tablist", { name: "Run companion tabs" })).toBeNull();
    expect(screen.queryByRole("heading", { name: "Follow-up chat" })).toBeNull();
    await fireEvent.click(sourceTab);
    await fireEvent.click(reportTab);

    expect(onChangeCanvasMode).toHaveBeenNthCalledWith(1, "source");
    expect(onChangeCanvasMode).toHaveBeenNthCalledWith(2, "report");

    await view.rerender(reportCanvasProps({
      canvasMode: "source",
      workspaceSelection: { kind: "source", sourceId: 1 },
      currentSource: source(),
      onChangeCanvasMode,
    }));
    expect(screen.getByRole("navigation", { name: "Source browser tabs" })).toBeTruthy();
    expect(screen.queryByRole("region", { name: "Report setup" })).toBeNull();
    expect(screen.queryByRole("tablist", { name: "Run companion tabs" })).toBeNull();

    await view.rerender(reportCanvasProps({
      canvasMode: "report",
      currentRun: run(),
      onChangeCanvasMode,
    }));
    expect(screen.getByRole("region", { name: "Opened run metadata" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Report output" })).toBeTruthy();
    expect(screen.queryByRole("navigation", { name: "Source browser tabs" })).toBeNull();
    expect(screen.queryByRole("tablist", { name: "Run companion tabs" })).toBeNull();
    expect(screen.queryByRole("heading", { name: "Follow-up chat" })).toBeNull();
  });

  it("shows setup only when no run is open and report mode is selected", async () => {
    const view = renderReportCanvas();

    const setup = screen.getByRole("region", { name: "Report setup" });
    expect(setup).toBeTruthy();
    expect(screen.getByRole("region", { name: "Workspace actions" })).toBeTruthy();
    expect(within(setup).getByRole("heading", { name: "Start the first report" })).toBeTruthy();
    expect(screen.queryByRole("region", { name: "Opened run metadata" })).toBeNull();
    expect(screen.queryByRole("tablist", { name: "Run companion tabs" })).toBeNull();
    expect(screen.queryByRole("heading", { name: "Follow-up chat" })).toBeNull();
    expect(within(setup).queryByRole("heading", { name: "Prompt Template" })).toBeNull();
    expect(within(setup).queryByRole("heading", { name: "Source Groups" })).toBeNull();
    expect(screen.queryByLabelText("Template editor drawer")).toBeNull();
    expect(screen.queryByLabelText("Source group editor drawer")).toBeNull();

    const disabledReason = "Choose source material before running a report.";
    const launch = within(setup).getByRole("button", { name: "Run report" });
    expect((launch as HTMLButtonElement).disabled).toBe(true);
    expect(launch.getAttribute("title")).toBe(disabledReason);
    expect(screen.getByRole("alert").textContent).toContain(disabledReason);

    await view.rerender(reportCanvasProps({ startingReport: true }));
    expect(screen.getByRole("region", { name: "Report setup" })).toBeTruthy();
    expect(screen.queryByRole("heading", { name: "Start the first report" })).toBeNull();
    expect(screen.getByRole("button", { name: "Starting..." })).toBeTruthy();

    await view.rerender(reportCanvasProps({
      selectedRunIsActive: true,
      activePhase: "gathering",
    }));
    expect(screen.getByRole("region", { name: "Report setup" })).toBeTruthy();
    expect(screen.queryByRole("heading", { name: "Start the first report" })).toBeNull();
    expect(screen.getByText("gathering")).toBeTruthy();

    await view.rerender(reportCanvasProps({ reportLaunchDisabledReason: null }));
    expect(screen.getByRole("heading", { name: "Start the first report" })).toBeTruthy();
    expect((screen.getByRole("button", { name: "Run report" }) as HTMLButtonElement).disabled).toBe(false);
    expect(screen.queryByRole("alert")).toBeNull();

    await view.rerender(reportCanvasProps({ currentRun: run() }));
    expect(screen.queryByRole("region", { name: "Report setup" })).toBeNull();
    expect(screen.getByRole("region", { name: "Opened run metadata" })).toBeTruthy();

    await view.rerender(reportCanvasProps({
      canvasMode: "source",
      workspaceSelection: { kind: "source", sourceId: 1 },
      currentSource: source(),
    }));
    expect(screen.queryByRole("region", { name: "Report setup" })).toBeNull();
    expect(screen.getByRole("navigation", { name: "Source browser tabs" })).toBeTruthy();
  });

  it("renders required opened-run header metadata", async () => {
    const view = renderReportCanvas({
      currentRun: run(),
      runSnapshotAvailability: "available",
      snapshotProbeState: "available",
      traceRefCount: 4,
      activePhase: "finalizing",
      activeProgress: "9/10",
      runTargetLabel: () => "Saved research scope",
    });

    const header = screen.getByRole("region", { name: "Opened run metadata" });
    expect(within(header).getByRole("heading", { name: "Run #30" })).toBeTruthy();
    expect(within(header).getAllByText("Saved research scope")).toHaveLength(2);
    expect(within(header).getAllByText("completed").length).toBeGreaterThan(0);
    expect(within(header).getByText("1699000000-1700000000")).toBeTruthy();
    expect(within(header).getByText("Research report v3")).toBeTruthy();
    expect(within(header).getByText("openai/model-a")).toBeTruthy();
    expect(within(header).getByText("4")).toBeTruthy();

    await fireEvent.click(within(header).getByText("Run details"));
    for (const label of [
      "Scope",
      "Status",
      "Created",
      "Completed",
      "Provider profile",
      "Source basis",
      "Snapshot status",
      "Snapshot captured",
      "Snapshot note",
      "YouTube corpus",
      "Live phase",
      "Live progress",
    ]) {
      expect(within(header).getByText(label, { selector: "span" })).toBeTruthy();
    }
    expect(within(header).getByText("time:1700000000")).toBeTruthy();
    expect(within(header).getByText("time:1700000100")).toBeTruthy();
    expect(within(header).getByText("Default")).toBeTruthy();
    expect(within(header).getByText("2026-08-03T10:00:00Z")).toBeTruthy();
    expect(within(header).getByText(
      "Browsing live source data while the opened run remains bound to its saved report context.",
    )).toBeTruthy();
    expect(within(header).getByText("Snapshot available")).toBeTruthy();
    expect(within(header).getByText(
      "Frozen source material captured for this run is available.",
    )).toBeTruthy();
    expect(within(header).getByText("Transcript")).toBeTruthy();
    expect(within(header).getByText("finalizing")).toBeTruthy();
    expect(within(header).getByText("9/10")).toBeTruthy();

    await view.rerender(reportCanvasProps({
      currentRun: run({
        snapshot_state: "capture_failed",
        snapshot_captured_at: null,
        snapshot_error: "  snapshot   disk full  ",
      }),
      runSnapshotAvailability: "unavailable",
      snapshotProbeState: "error",
      runTargetLabel: () => "Saved research scope",
    }));
    const failedHeader = screen.getByRole("region", { name: "Opened run metadata" });
    expect(within(failedHeader).getByText(
      "Saved report is readable, but Extractum could not save the frozen source context for this run.",
    )).toBeTruthy();
    expect(within(failedHeader).getByText("Snapshot capture failed")).toBeTruthy();
    expect(within(failedHeader).getByText(
      "Extractum could not save the frozen source context for this run. Exact source browsing, evidence source resolution, and follow-up chat stay unavailable.",
    )).toBeTruthy();
    expect(within(failedHeader).getByText("Snapshot error", { selector: "span" })).toBeTruthy();
    expect(within(failedHeader).getByText("snapshot disk full")).toBeTruthy();
  });

  it("keeps report setup copy aware of existing saved runs", async () => {
    const view = renderReportCanvas({ currentScopeHasSavedRuns: false });

    expect(screen.getByRole("heading", { name: "Start the first report" })).toBeTruthy();
    expect(screen.getByText(/Once the report is ready/)).toBeTruthy();
    expect(screen.queryByText("Build the first report for this workspace")).toBeNull();

    await view.rerender(reportCanvasProps({ currentScopeHasSavedRuns: true }));
    expect(screen.queryByRole("heading", { name: "Start the first report" })).toBeNull();
    expect(screen.getByRole("heading", { name: "Run another report" })).toBeTruthy();
    expect(screen.getByText(/Prior reports stay available in Runs/)).toBeTruthy();
    expect(screen.queryByText("Build the first report for this workspace")).toBeNull();
  });

  it("keeps snapshot and live source basis explicit", async () => {
    const onViewLiveSource = vi.fn();
    const onBackToRunSnapshot = vi.fn();
    const openedRun = run({
      scope_type: "source",
      source_id: 1,
      source_title: "Research channel",
      project_id: null,
      project_name: null,
      scope_label: "Research channel",
    });
    const view = renderReportCanvas({
      canvasMode: "source",
      workspaceSelection: { kind: "source", sourceId: 1 },
      currentSource: source(),
      currentRun: openedRun,
      sourceViewBasis: "run_snapshot",
      runSnapshotAvailability: "available",
      snapshotProbeState: "available",
      runSnapshotMessages: [snapshotMessage({
        item_kind: "telegram_message",
        source_type: "telegram",
        source_subtype: "supergroup",
      })],
      onViewLiveSource,
      onBackToRunSnapshot,
    });

    const snapshotHeader = screen.getByLabelText("Run snapshot");
    expect(within(snapshotHeader).getByText("Run snapshot")).toBeTruthy();
    expect(within(snapshotHeader).getByText("Frozen source material captured for the opened run.")).toBeTruthy();
    await fireEvent.click(within(snapshotHeader).getByRole("button", { name: "View live source" }));
    expect(onViewLiveSource).toHaveBeenCalledOnce();

    await view.rerender(reportCanvasProps({
      canvasMode: "source",
      workspaceSelection: { kind: "source", sourceId: 1 },
      currentSource: source(),
      currentRun: openedRun,
      sourceViewBasis: "live_source",
      runSnapshotAvailability: "available",
      snapshotProbeState: "available",
      onViewLiveSource,
      onBackToRunSnapshot,
    }));
    const liveHeader = screen.getByLabelText("Live source");
    expect(within(liveHeader).getByText("Live source")).toBeTruthy();
    expect(within(liveHeader).getByText(/Browsing live source data/)).toBeTruthy();
    await fireEvent.click(within(liveHeader).getByRole("button", { name: "Back to run snapshot" }));
    expect(onBackToRunSnapshot).toHaveBeenCalledOnce();

    await view.rerender(reportCanvasProps({
      canvasMode: "source",
      workspaceSelection: { kind: "source", sourceId: 1 },
      currentSource: source(),
      currentRun: run({
        ...openedRun,
        status: "running",
        snapshot_state: null,
        snapshot_captured_at: null,
        completed_at: null,
        result_markdown: null,
      }),
      sourceViewBasis: "run_snapshot",
      runSnapshotAvailability: "capturing",
      snapshotProbeState: "unknown",
      onViewLiveSource,
      onBackToRunSnapshot,
    }));
    const pendingHeader = screen.getByLabelText("Snapshot pending");
    expect(within(pendingHeader).getByText("Snapshot pending")).toBeTruthy();
    expect(screen.getAllByText("Snapshot capture is still pending for this active run.")).toHaveLength(2);

    await view.rerender(reportCanvasProps({
      canvasMode: "source",
      workspaceSelection: { kind: "source", sourceId: 1 },
      currentSource: source(),
      currentRun: openedRun,
      sourceViewBasis: "run_snapshot",
      runSnapshotAvailability: "unknown",
      snapshotProbeState: "unknown",
      onViewLiveSource,
      onBackToRunSnapshot,
    }));
    const checkingHeader = screen.getByLabelText("Checking saved snapshot");
    expect(within(checkingHeader).getByText("Checking snapshot")).toBeTruthy();
    expect(screen.getAllByText(
      "Extractum is checking whether frozen source material is available for this run.",
    )).toHaveLength(2);

    await view.rerender(reportCanvasProps({
      canvasMode: "source",
      workspaceSelection: { kind: "source", sourceId: 1 },
      currentSource: source(),
      currentRun: run({
        ...openedRun,
        snapshot_state: "capture_failed",
        snapshot_captured_at: null,
        snapshot_error: "  snapshot   failed  ",
      }),
      sourceViewBasis: "run_snapshot",
      runSnapshotAvailability: "unavailable",
      snapshotProbeState: "error",
      onViewLiveSource,
      onBackToRunSnapshot,
    }));
    expect(screen.getByLabelText("Snapshot capture failed")).toBeTruthy();
    expect(screen.getByRole("alert").textContent).toBe("snapshot failed");
    expect(screen.getByText(/This is live data, not the saved run snapshot/)).toBeTruthy();

    await view.rerender(reportCanvasProps({
      canvasMode: "source",
      workspaceSelection: { kind: "source", sourceId: 1 },
      currentSource: source({
        sourceType: "youtube",
        sourceSubtype: "playlist",
        title: "Research playlist",
      }),
      currentScopeTitle: "Research playlist",
      sourceViewBasis: "live_source",
      onViewLiveSource,
      onBackToRunSnapshot,
    }));
    const playlistTabs = screen.getByRole("navigation", { name: "Source browser tabs" });
    expect(within(playlistTabs).getByRole("button", { name: "Videos" }).getAttribute("aria-selected")).toBe("true");
    expect(within(playlistTabs).getByRole("button", { name: "Items" })).toBeTruthy();
    expect(within(playlistTabs).getByRole("button", { name: "Metadata" })).toBeTruthy();
    expect(within(playlistTabs).getByRole("button", { name: "Activity" })).toBeTruthy();
    expect(screen.getByRole("region", { name: "YouTube playlist videos" })).toBeTruthy();
    expect(screen.getByText("Research playlist", { selector: "h3" })).toBeTruthy();
  });

  it("passes YouTube comments and source activity callbacks through the report canvas", async () => {
    const onSyncYoutubeComments = vi.fn();
    const onCancelSourceJob = vi.fn();
    renderReportCanvas({
      canvasMode: "source",
      workspaceSelection: { kind: "source", sourceId: 1 },
      currentSource: source({
        sourceType: "youtube",
        sourceSubtype: "video",
        title: "Research video",
      }),
      sourceJobs: [sourceJob()],
      onSyncYoutubeComments,
      onCancelSourceJob,
    });

    await fireEvent.click(await screen.findByRole("button", { name: "Sync comments" }));
    expect(onSyncYoutubeComments).toHaveBeenCalledWith(1);

    await fireEvent.click(screen.getByRole("button", { name: "Activity" }));
    expect(screen.getByText("Syncing live comments")).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(onCancelSourceJob).toHaveBeenCalledWith("source-job-1");
  });

  it("passes Telegram topic state into the source surface", async () => {
    const onChangeSelectedTopicKey = vi.fn();
    const view = renderReportCanvas({
      canvasMode: "source",
      workspaceSelection: { kind: "source", sourceId: 1 },
      currentSource: source(),
      sourceTopics: [topic()],
      selectedTopicKey: "topic:7",
      showTopicSelector: true,
      onChangeSelectedTopicKey,
    });

    const topicSelect = await screen.findByLabelText("Topic view");
    expect((topicSelect as HTMLSelectElement).value).toBe("topic:7");
    expect(screen.getByRole("option", { name: "Methods (9)" })).toBeTruthy();
    await fireEvent.change(topicSelect, { target: { value: "__all_topics__" } });
    expect(onChangeSelectedTopicKey).toHaveBeenCalledWith("__all_topics__");

    await view.rerender(reportCanvasProps({
      canvasMode: "source",
      workspaceSelection: { kind: "source", sourceId: 1 },
      currentSource: source(),
      sourceTopics: [topic()],
      loadingSourceTopics: true,
      selectedTopicKey: "topic:7",
      showTopicSelector: true,
      onChangeSelectedTopicKey,
    }));
    expect((screen.getByLabelText("Topic view") as HTMLSelectElement).disabled).toBe(true);
  });

  it("labels source surfaces without repeating the selected workspace title", async () => {
    const view = renderReportCanvas({
      canvasMode: "source",
      workspaceSelection: { kind: "source", sourceId: 1 },
      currentSource: source(),
      currentScopeTitle: "Selected source workspace title",
    });

    expect(screen.getAllByText("Selected source workspace title")).toHaveLength(1);
    expect(screen.getByText("Source material")).toBeTruthy();

    await view.rerender(reportCanvasProps({
      canvasMode: "source",
      workspaceSelection: { kind: "source_group", sourceGroupId: 20 },
      currentGroup: group(),
      currentScopeTitle: "Selected group workspace title",
    }));
    expect(screen.getAllByText("Selected group workspace title")).toHaveLength(1);
    expect(screen.getByText("Group sources")).toBeTruthy();
  });

  it("keeps run snapshot reading bounded and snapshot-only", async () => {
    const onLoadMoreRunSnapshotMessages = vi.fn();
    api.listSourceItems.mockClear();
    renderReportCanvas({
      canvasMode: "source",
      workspaceSelection: { kind: "source", sourceId: 1 },
      currentSource: source({ sourceType: "rss", sourceSubtype: "feed" }),
      currentRun: run({
        scope_type: "source",
        source_id: 1,
        source_title: "Research feed",
        project_id: null,
        project_name: null,
        scope_label: "Research feed",
      }),
      sourceViewBasis: "run_snapshot",
      runSnapshotAvailability: "available",
      snapshotProbeState: "available",
      runSnapshotMessages: [snapshotMessage()],
      hasMoreRunSnapshotMessages: true,
      sourceItems: [sourceItem()],
      onLoadMoreRunSnapshotMessages,
    });
    await tick();

    expect(screen.getByRole("region", { name: "Run snapshot items" })).toBeTruthy();
    expect(screen.getByText("Frozen snapshot row")).toBeTruthy();
    expect(screen.getByText("snapshot:1")).toBeTruthy();
    expect(screen.queryByText("Live-only source row")).toBeNull();
    expect(screen.getByText(/Snapshot items are limited to frozen rows loaded for this run/)).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Load older snapshot messages" }));
    expect(onLoadMoreRunSnapshotMessages).toHaveBeenCalledOnce();
    expect(api.listSourceItems).not.toHaveBeenCalled();
  });

  it("keeps source-group run snapshots pageable through the snapshot browser", async () => {
    const onLoadMoreRunSnapshotMessages = vi.fn();
    renderReportCanvas({
      canvasMode: "source",
      workspaceSelection: { kind: "source_group", sourceGroupId: 20 },
      currentGroup: group(),
      currentRun: run({
        scope_type: "source_group",
        source_group_id: 20,
        source_group_name: "Research group",
        project_id: null,
        project_name: null,
        scope_label: "Research group",
      }),
      sourceViewBasis: "run_snapshot",
      runSnapshotAvailability: "available",
      snapshotProbeState: "available",
      runSnapshotMessages: [
        snapshotMessage({
          item_kind: "telegram_message",
          source_type: "telegram",
          source_subtype: "supergroup",
        }),
        snapshotMessage({
          item_id: 102,
          source_id: 2,
          external_id: "snapshot-item-2",
          ref: "snapshot:2",
          content: "Second frozen group row",
          item_kind: "telegram_message",
          source_type: "telegram",
          source_subtype: "supergroup",
        }),
      ],
      hasMoreRunSnapshotMessages: true,
      onLoadMoreRunSnapshotMessages,
    });

    expect(screen.getByRole("region", { name: "Run snapshot group sources" })).toBeTruthy();
    expect(screen.getByText("Frozen snapshot row")).toBeTruthy();
    expect(screen.getByText("Second frozen group row")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Load older messages" })).toBeNull();
    await fireEvent.click(screen.getByRole("button", { name: "Load older snapshot messages" }));
    expect(onLoadMoreRunSnapshotMessages).toHaveBeenCalledOnce();
  });

  it("uses real chat availability in the report toolbar", async () => {
    const view = renderReportCanvas({
      currentRun: run(),
      chatAvailability: {
        enabled: false,
        reason: "checking_snapshot",
        title: "Checking exact saved context",
        description: "Waiting for the saved snapshot check.",
      },
    });

    expect(screen.getByText("Checking exact saved context")).toBeTruthy();
    expect(screen.queryByText("Chat ready")).toBeNull();

    await view.rerender(reportCanvasProps({
      currentRun: run(),
      chatAvailability: {
        enabled: true,
        reason: "enabled",
        title: "Saved context ready",
        description: "Questions use saved context.",
      },
    }));
    expect(screen.getByText("Chat ready")).toBeTruthy();
    expect(screen.queryByText("Checking exact saved context")).toBeNull();
  });

  it("keeps workspace tools reachable before setup report and source bodies", async () => {
    const view = renderReportCanvas();

    const tools = screen.getByRole("region", { name: "Workspace actions" });
    const setup = screen.getByRole("region", { name: "Report setup" });
    expectBefore(tools, setup);
    await fireEvent.click(within(tools).getByRole("button", { name: "Edit templates" }));
    await fireEvent.click(within(tools).getByRole("button", { name: "Edit groups" }));
    expect(screen.getByLabelText("Template editor drawer")).toBeTruthy();
    expect(screen.getByLabelText("Source group editor drawer")).toBeTruthy();
    await fireEvent.click(within(tools).getByRole("button", { name: "Hide templates" }));
    await fireEvent.click(within(tools).getByRole("button", { name: "Hide groups" }));

    await view.rerender(reportCanvasProps({
      workspaceSelection: { kind: "source", sourceId: 1 },
      currentSource: source(),
      currentRun: run({
        scope_type: "source",
        source_id: 1,
        source_title: "Research channel",
        project_id: null,
        project_name: null,
        scope_label: "Research channel",
      }),
    }));
    const openedRunTools = screen.getByRole("region", { name: "Workspace actions" });
    expectBefore(
      openedRunTools,
      screen.getByRole("region", { name: "Opened run metadata" }),
    );
    expect(screen.queryByRole("region", { name: "Opened run management" })).toBeNull();
    expect(screen.getAllByRole("button", { name: "Export for NotebookLM" })).toHaveLength(1);
    expect(screen.getAllByRole("button", { name: "Edit templates" })).toHaveLength(1);
    expect(screen.getAllByRole("button", { name: "Edit groups" })).toHaveLength(1);
    expect(within(openedRunTools).getByRole("button", { name: "Export for NotebookLM" })).toBeTruthy();
    expect(within(openedRunTools).getByRole("button", { name: "Edit templates" })).toBeTruthy();
    expect(within(openedRunTools).getByRole("button", { name: "Edit groups" })).toBeTruthy();

    await view.rerender(reportCanvasProps({
      canvasMode: "source",
      workspaceSelection: { kind: "source", sourceId: 1 },
      currentSource: source(),
    }));
    expectBefore(
      screen.getByRole("region", { name: "Workspace actions" }),
      screen.getByRole("navigation", { name: "Source browser tabs" }),
    );
  });

  it("derives NotebookLM export availability from live canvas source or Telegram group", async () => {
    const onOpenNotebookLmExport = vi.fn();
    const view = renderReportCanvas({
      workspaceSelection: { kind: "source", sourceId: 1 },
      currentSource: source(),
      exportDialogOpen: true,
      notebookLmExportForm: exportForm({ outputDir: "C:\\NotebookLM" }),
      canIncludeMigratedHistory: true,
      onOpenNotebookLmExport,
    });

    const sourceExport = screen.getByRole("button", { name: "Export for NotebookLM" });
    expect((sourceExport as HTMLButtonElement).disabled).toBe(false);
    const sourceDialog = screen.getByRole("dialog", { name: "Export for NotebookLM" });
    expect(within(sourceDialog).getByText(
      "Prepare Markdown files for Research channel.",
    )).toBeTruthy();
    expect((within(sourceDialog).getByRole("button", { name: "Export" }) as HTMLButtonElement).disabled).toBe(false);
    expect(within(sourceDialog).getByRole("checkbox", {
      name: /Include migrated historical scope/,
    })).toBeTruthy();
    await fireEvent.click(sourceExport);
    expect(onOpenNotebookLmExport).toHaveBeenCalledTimes(1);

    await view.rerender(reportCanvasProps({
      workspaceSelection: { kind: "source_group", sourceGroupId: 20 },
      currentGroup: group({ source_type: "telegram" }),
      exportDialogOpen: true,
      notebookLmExportForm: exportForm({ outputDir: "C:\\NotebookLM" }),
      canIncludeMigratedHistory: false,
      onOpenNotebookLmExport,
    }));
    const telegramGroupExport = screen.getByRole("button", { name: "Export for NotebookLM" });
    expect((telegramGroupExport as HTMLButtonElement).disabled).toBe(false);
    const groupDialog = screen.getByRole("dialog", { name: "Export for NotebookLM" });
    expect(within(groupDialog).getByText(
      "Prepare Markdown files for Research group (2 sources).",
    )).toBeTruthy();
    expect((within(groupDialog).getByRole("button", { name: "Export" }) as HTMLButtonElement).disabled).toBe(false);
    expect(within(groupDialog).queryByRole("checkbox", {
      name: /Include migrated historical scope/,
    })).toBeNull();
    await fireEvent.click(telegramGroupExport);
    expect(onOpenNotebookLmExport).toHaveBeenCalledTimes(2);

    await view.rerender(reportCanvasProps({
      workspaceSelection: { kind: "source_group", sourceGroupId: 20 },
      currentGroup: group({ source_type: "youtube" }),
      exportDialogOpen: false,
      onOpenNotebookLmExport,
    }));
    const youtubeGroupExport = screen.getByRole("button", { name: "Export for NotebookLM" });
    expect((youtubeGroupExport as HTMLButtonElement).disabled).toBe(true);
    expect(youtubeGroupExport.getAttribute("title")).toBe(
      "YouTube source-group NotebookLM export is not implemented yet.",
    );

    await view.rerender(reportCanvasProps({ onOpenNotebookLmExport }));
    expect(screen.queryByRole("button", { name: "Export for NotebookLM" })).toBeNull();
  });

  it("renders the scoped evidence return affordance above source reader headers", async () => {
    const view = renderReportCanvas({
      canvasMode: "source",
      workspaceSelection: { kind: "source", sourceId: 1 },
      currentSource: source(),
      currentScopeTitle: "Evidence source workspace",
      selectedTraceRef: null,
      sourceReturnContext: {
        kind: "evidence",
        runId: 30,
        sourceScope: { kind: "source", sourceId: 1 },
        sourceViewBasis: "live_source",
        traceRef: "source:1:item:101",
      },
    });

    const returnButton = screen.getByRole("button", { name: "Back to evidence" });
    const readerHeader = screen.getByLabelText("Evidence source workspace");
    expectBefore(returnButton, readerHeader);

    await view.rerender(reportCanvasProps({
      canvasMode: "source",
      workspaceSelection: { kind: "source", sourceId: 1 },
      currentSource: source(),
      currentScopeTitle: "Evidence source workspace",
      sourceReturnContext: null,
    }));
    expect(screen.queryByRole("button", { name: "Back to evidence" })).toBeNull();
  });

  it("passes evidence return context and callback through the report canvas", async () => {
    const onReturnToEvidenceReview = vi.fn();
    renderReportCanvas({
      canvasMode: "source",
      workspaceSelection: { kind: "source_group", sourceGroupId: 20 },
      currentGroup: group(),
      sourceReturnContext: {
        kind: "evidence",
        runId: 30,
        sourceScope: { kind: "group_member", groupId: 20, sourceId: 2 },
        sourceViewBasis: "live_source",
        traceRef: "source:2:item:202",
      },
      onReturnToEvidenceReview,
    });

    await fireEvent.click(screen.getByRole("button", { name: "Back to evidence" }));
    expect(onReturnToEvidenceReview).toHaveBeenCalledOnce();
    expect(onReturnToEvidenceReview).toHaveBeenCalledWith(expect.any(MouseEvent));
  });

  it("passes bounded source browser mode only for live source canvas review", async () => {
    const openedRun = run({
      scope_type: "source",
      source_id: 1,
      source_title: "Research channel",
      project_id: null,
      project_name: null,
      scope_label: "Research channel",
    });
    const view = renderReportCanvas({
      canvasMode: "source",
      workspaceSelection: { kind: "source", sourceId: 1 },
      currentSource: source(),
      sourceViewBasis: "live_source",
    });

    expect(sourceBrowserBody().parentElement?.classList.contains("bounded")).toBe(true);

    await view.rerender(reportCanvasProps({
      canvasMode: "source",
      workspaceSelection: { kind: "source", sourceId: 1 },
      currentSource: source(),
      currentRun: openedRun,
      sourceViewBasis: "run_snapshot",
      runSnapshotAvailability: "available",
      snapshotProbeState: "available",
      runSnapshotMessages: [snapshotMessage({
        item_kind: "telegram_message",
        source_type: "telegram",
        source_subtype: "supergroup",
      })],
    }));
    expect(sourceBrowserBody().parentElement?.classList.contains("bounded")).toBe(false);
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
