import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";
import AnalysisPage from "../routes/analysis/+page.svelte";
import type {
  AnalysisRunDetail,
  AnalysisSourceGroup,
  AnalysisTraceRef,
} from "$lib/types/analysis";
import type {
  Source,
  SourceItem,
  YoutubeTranscriptSegment,
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
  getAnalysisRun: vi.fn(),
  getAnalysisRunTrace: vi.fn(),
  getLlmProfiles: vi.fn(),
  getWorkspaceAccountStatuses: vi.fn(),
  getYoutubePlaylistDetail: vi.fn(),
  getYoutubeRuntimeStatus: vi.fn(),
  getYoutubeVideoDetail: vi.fn(),
  listActiveAnalysisRuns: vi.fn(),
  listAnalysisChatMessages: vi.fn(),
  listAnalysisPromptTemplates: vi.fn(),
  listAnalysisRunMessages: vi.fn(),
  listAnalysisRuns: vi.fn(),
  listAnalysisSourceGroups: vi.fn(),
  listAnalysisSources: vi.fn(),
  listLlmProviderModels: vi.fn(),
  listSourceForumTopics: vi.fn(),
  listSourceItems: vi.fn(),
  listSourceJobs: vi.fn(),
  listSources: vi.fn(),
  listTakeoutImportRecoveryStates: vi.fn(),
  listTakeoutSourceImportJobs: vi.fn(),
  listWorkspaceAccounts: vi.fn(),
  listYoutubeSourceSummaries: vi.fn(),
  listYoutubeTranscriptSegments: vi.fn(),
  listenToAnalysisChatEvents: vi.fn(),
  listenToAnalysisRunEvents: vi.fn(),
  listenToNotebookLmExportEvents: vi.fn(),
  listenToSourceJobEvents: vi.fn(),
  listenToTakeoutImportEvents: vi.fn(),
  resolveAnalysisTraceRefs: vi.fn(),
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

vi.mock("$lib/components/analysis/report-canvas.svelte", async () => {
  const receiver = await import("$lib/testing/AnalysisRouteReceiver.svelte");
  return { default: receiver.default };
});

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

function trace(overrides: Partial<AnalysisTraceRef> = {}): AnalysisTraceRef {
  return {
    ref: "s1-i501@42000ms",
    item_id: 501,
    source_id: 1,
    external_id: "video-1:42000",
    published_at: 1_700_000_000,
    excerpt: "Focused route transcript excerpt",
    youtube_url: "https://www.youtube.com/watch?v=video-1&t=42s",
    youtube_timestamp_seconds: 42,
    youtube_display_label: "Research video at 00:42",
    is_synthetic: false,
    ...overrides,
  };
}

function transcriptSegment(
  overrides: Partial<YoutubeTranscriptSegment> = {},
): YoutubeTranscriptSegment {
  return {
    id: 91,
    sourceId: 1,
    itemId: 501,
    segmentIndex: 4,
    startMs: 42_000,
    endMs: 48_000,
    text: "Focused route transcript",
    captionLanguage: "en",
    captionTrackKind: "manual",
    isAutoGenerated: false,
    ...overrides,
  };
}

function persistWorkspaceSelection(
  workspaceSelection:
    | { kind: "source"; sourceId: number }
    | { kind: "source_group"; sourceGroupId: number },
) {
  localStorage.setItem("extractum.analysis.workspace.v1", JSON.stringify({
    version: 1,
    workspaceSelection,
    canvasMode: "report",
    sourceViewBasis: "live_source",
    companionTab: "evidence",
    runs: {
      historyScope: "all",
      runFilter: "all",
      runsFilter: {
        query: "",
        status: "all",
        scope: "all",
        dateFrom: "",
        dateTo: "",
        provider: "",
        model: "",
        template: "",
      },
    },
  }));
}

function arrangeAnalysisRoute(options: {
  sources?: Source[];
  groups?: AnalysisSourceGroup[];
} = {}) {
  api.getAnalysisRun.mockResolvedValue(null);
  api.getAnalysisRunTrace.mockResolvedValue({ refs: [] });
  api.listActiveAnalysisRuns.mockResolvedValue([]);
  api.listAnalysisRunMessages.mockResolvedValue({
    messages: [],
    next_cursor: null,
    has_more: false,
  });
  api.listAnalysisRuns.mockResolvedValue([]);
  api.listAnalysisSourceGroups.mockResolvedValue(options.groups ?? []);
  api.listAnalysisSources.mockResolvedValue([]);
  api.listSourceItems.mockResolvedValue([]);
  api.listSources.mockResolvedValue(options.sources ?? []);
  api.listYoutubeTranscriptSegments.mockResolvedValue({
    segments: [],
    nextCursor: null,
    hasMore: false,
  });
}

function arrangeActiveSourceEvidenceRoute() {
  const focusedTrace = trace({
    ref: "s1-i101",
    item_id: 101,
    external_id: "101",
    youtube_url: null,
    youtube_timestamp_seconds: null,
    youtube_display_label: null,
  });
  arrangeAnalysisRoute({ sources: [source()] });
  api.getAnalysisRun.mockResolvedValue(run({
    status: "running",
    scope_type: "source",
    source_id: 1,
    source_title: "Research channel",
    source_group_id: null,
    source_group_name: null,
    project_id: null,
    project_name: null,
    scope_label: "Research channel",
    snapshot_state: null,
    snapshot_captured_at: null,
    completed_at: null,
    result_markdown: null,
  }));
  api.getAnalysisRunTrace.mockResolvedValue({ refs: [focusedTrace] });
  api.listSourceItems.mockResolvedValue([sourceItem({
    id: 101,
    sourceId: 1,
    externalId: "101",
    content: "Focused route source item",
  })]);
  window.history.replaceState({}, "", "/analysis?runId=30");
}

function arrangeActiveGroupTranscriptRoute() {
  const focusedTrace = trace({
    ref: "s2-i202@754000ms",
    item_id: 202,
    source_id: 2,
    external_id: "video-2:754000",
    youtube_url: "https://www.youtube.com/watch?v=video-2&t=754s",
    youtube_timestamp_seconds: 754,
    youtube_display_label: "Group video at 12:34",
  });
  const focusedSegment = transcriptSegment({
    id: 92,
    sourceId: 2,
    itemId: 202,
    segmentIndex: 12,
    startMs: 754_000,
    endMs: 760_000,
    text: "Focused group route transcript",
  });
  arrangeAnalysisRoute({
    sources: [source({
      id: 2,
      sourceType: "youtube",
      sourceSubtype: "video",
      externalId: "video-2",
      title: "Group research video",
    })],
    groups: [group({
      source_type: "youtube",
      members: [{
        source_id: 2,
        source_title: "Group research video",
        item_count: 1,
      }],
    })],
  });
  api.getAnalysisRun.mockResolvedValue(run({
    status: "running",
    scope_type: "source_group",
    source_id: null,
    source_title: null,
    source_group_id: 20,
    source_group_name: "Research group",
    project_id: null,
    project_name: null,
    scope_label: "Research group",
    snapshot_state: null,
    snapshot_captured_at: null,
    completed_at: null,
    result_markdown: null,
  }));
  api.getAnalysisRunTrace.mockResolvedValue({ refs: [focusedTrace] });
  api.listYoutubeTranscriptSegments.mockResolvedValue({
    segments: [focusedSegment],
    nextCursor: null,
    hasMore: false,
  });
  window.history.replaceState({}, "", "/analysis?runId=30");
}

function renderAnalysisRouteWithReceiver() {
  return render(AnalysisPage);
}

async function expectRestoredRouteTarget(expected: string) {
  await waitFor(() => expect(api.listActiveAnalysisRuns).toHaveBeenCalled());
  await fireEvent.click(screen.getByRole("button", { name: "Read route target" }));
  expect(screen.getByLabelText("Received route target").textContent).toBe(expected);
}

async function submitRouteNotebookLmExport() {
  await fireEvent.click(screen.getByRole("button", { name: "Open route NotebookLM export" }));
  await fireEvent.click(screen.getByRole("button", { name: "Prepare route NotebookLM export" }));
  await tick();
  await fireEvent.click(screen.getByRole("button", { name: "Submit route NotebookLM export" }));
  await waitFor(() => expect(api.exportSourceToNotebookLm).toHaveBeenCalledOnce());
}

async function showSelectedTraceInSource(traceRef: string) {
  await waitFor(() => {
    expect(api.getAnalysisRun).toHaveBeenCalledWith(30);
    expect(api.getAnalysisRunTrace).toHaveBeenCalledWith(30);
  });
  await fireEvent.click(await screen.findByRole("tab", { name: "Evidence" }));
  await fireEvent.click(await screen.findByRole("button", { name: new RegExp(traceRef) }));
  await fireEvent.click(await screen.findByRole("button", { name: "Show in source" }));
}

beforeEach(() => {
  vi.resetAllMocks();
  localStorage.clear();
  api.getAnalysisRun.mockResolvedValue(null);
  api.getAnalysisRunTrace.mockResolvedValue({ refs: [] });
  api.getLlmProfiles.mockResolvedValue({ active_profile: "", profiles: [] });
  api.getWorkspaceAccountStatuses.mockResolvedValue([]);
  api.getYoutubePlaylistDetail.mockResolvedValue(null);
  api.getYoutubeRuntimeStatus.mockResolvedValue({
    ytdlpAvailable: false,
    ytdlpVersion: null,
    message: "YouTube runtime unavailable in component smoke",
  });
  api.getYoutubeVideoDetail.mockResolvedValue(null);
  api.listActiveAnalysisRuns.mockResolvedValue([]);
  api.listAnalysisChatMessages.mockResolvedValue([]);
  api.listAnalysisPromptTemplates.mockResolvedValue([]);
  api.listAnalysisRunMessages.mockResolvedValue({
    messages: [],
    next_cursor: null,
    has_more: false,
  });
  api.listAnalysisRuns.mockResolvedValue([]);
  api.listAnalysisSourceGroups.mockResolvedValue([]);
  api.listAnalysisSources.mockResolvedValue([]);
  api.listLlmProviderModels.mockResolvedValue([]);
  api.listSourceForumTopics.mockResolvedValue({
    topics: [],
    topicResolutionState: {
      status: "never_run",
      resolverVersion: 0,
      unresolvedCount: 0,
      pendingItemCount: 0,
      membershipsRefreshedAt: null,
    },
  });
  api.listSourceItems.mockResolvedValue([]);
  api.listSourceJobs.mockResolvedValue([]);
  api.listSources.mockResolvedValue([]);
  api.listTakeoutImportRecoveryStates.mockResolvedValue([]);
  api.listTakeoutSourceImportJobs.mockResolvedValue([]);
  api.listWorkspaceAccounts.mockResolvedValue([]);
  api.listYoutubeSourceSummaries.mockResolvedValue([]);
  api.listYoutubeTranscriptSegments.mockResolvedValue({
    segments: [],
    nextCursor: null,
    hasMore: false,
  });
  api.resolveAnalysisTraceRefs.mockResolvedValue([]);
  api.listenToAnalysisRunEvents.mockResolvedValue(api.unlistenAnalysisRuns);
  api.listenToAnalysisChatEvents.mockResolvedValue(api.unlistenAnalysisChat);
  api.listenToNotebookLmExportEvents.mockResolvedValue(api.unlistenNotebookLmExport);
  api.listenToTakeoutImportEvents.mockResolvedValue(api.unlistenTakeoutImport);
  api.listenToSourceJobEvents.mockResolvedValue(api.unlistenSourceJobs);
});

afterEach(cleanup);

afterEach(() => {
  window.history.replaceState({}, "", "/analysis");
});

describe("report canvas component contract", () => {
  it("submits NotebookLM export for either current source or current source group", async () => {
    api.exportSourceToNotebookLm.mockResolvedValue({
      output_dir: "C:\\NotebookLM",
      files: [],
      glossary_file: null,
      exported_message_count: 0,
      skipped_message_count: 0,
      warning_count: 0,
      warnings: [],
    });

    arrangeAnalysisRoute({ sources: [source()] });
    persistWorkspaceSelection({ kind: "source", sourceId: 1 });
    const sourceView = renderAnalysisRouteWithReceiver();
    await expectRestoredRouteTarget("source:1");
    await submitRouteNotebookLmExport();
    expect(api.exportSourceToNotebookLm).toHaveBeenCalledWith({
      export_id: expect.any(String),
      source_id: 1,
      source_group_id: null,
      output_dir: "C:\\NotebookLM",
      period_from: null,
      period_to: null,
      include_media_placeholders: true,
      include_migrated_history: false,
      min_message_length: 3,
      max_words_per_file: 300_000,
      max_bytes_per_file: 50_000_000,
      overwrite_existing: false,
    });

    sourceView.unmount();
    await tick();
    localStorage.clear();
    vi.clearAllMocks();

    arrangeAnalysisRoute({ groups: [group()] });
    persistWorkspaceSelection({ kind: "source_group", sourceGroupId: 20 });
    renderAnalysisRouteWithReceiver();
    await expectRestoredRouteTarget("group:20:telegram");
    await submitRouteNotebookLmExport();
    expect(api.exportSourceToNotebookLm).toHaveBeenCalledWith({
      export_id: expect.any(String),
      source_id: null,
      source_group_id: 20,
      output_dir: "C:\\NotebookLM",
      period_from: null,
      period_to: null,
      include_media_placeholders: true,
      include_migrated_history: false,
      min_message_length: 3,
      max_words_per_file: 300_000,
      max_bytes_per_file: 50_000_000,
      overwrite_existing: false,
    });
  });

  it("opens NotebookLM export for Telegram source groups without the old single-source guard", async () => {
    arrangeAnalysisRoute({ groups: [group()] });
    persistWorkspaceSelection({ kind: "source_group", sourceGroupId: 20 });
    renderAnalysisRouteWithReceiver();
    await expectRestoredRouteTarget("group:20:telegram");

    await fireEvent.click(screen.getByRole("button", {
      name: "Open route NotebookLM export",
    }));
    await tick();
    await fireEvent.click(screen.getByRole("button", {
      name: "Read route export dialog",
    }));

    expect(screen.getByLabelText("Received route export dialog").textContent).toBe("open:true");
    expect(screen.queryByText("Select a source or source group before exporting.")).toBeNull();
  });

  it("passes transient evidence highlight tokens from route to source surfaces", async () => {
    arrangeActiveSourceEvidenceRoute();
    renderAnalysisRouteWithReceiver();
    await showSelectedTraceInSource("s1-i101");
    await waitFor(() => {
      expect(api.listSourceItems).toHaveBeenCalledWith({
        sourceId: 1,
        limit: 120,
        beforePublishedAt: null,
        beforeCursor: null,
        historyScope: "current",
        topicFilter: null,
        aroundItemId: 101,
      });
    });
    await tick();

    await fireEvent.click(screen.getByRole("button", {
      name: "Read route highlight token",
    }));
    expect(screen.getByLabelText("Received route highlight token").textContent).toBe(
      "s1-i101|live_source|source:1",
    );
  });

  it("passes focused group transcript segments from route to source surfaces", async () => {
    arrangeActiveGroupTranscriptRoute();
    renderAnalysisRouteWithReceiver();
    await showSelectedTraceInSource("s2-i202@754000ms");
    await waitFor(() => {
      expect(api.listYoutubeTranscriptSegments).toHaveBeenCalledWith({
        sourceId: 2,
        after: null,
        limit: 80,
        searchQuery: null,
        aroundStartMs: 754_000,
      });
    });
    await tick();

    await fireEvent.click(screen.getByRole("button", {
      name: "Read route group transcripts",
    }));
    expect(screen.getByLabelText("Received route group transcripts").textContent).toBe(
      "Focused group route transcript",
    );
  });
});
