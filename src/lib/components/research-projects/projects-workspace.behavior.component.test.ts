// @vitest-environment jsdom
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { tick, type ComponentProps } from "svelte";
import { defaultDateOffset, endOfDayUnix, startOfDayUnix } from "$lib/analysis-utils";
import type { AnalysisPromptTemplate, AnalysisRunSummary } from "$lib/types/analysis";
import type { LibraryCatalogRecord } from "$lib/types/library-sources";
import type { ProjectRecord, ProjectSourceRecord, ProjectSummary } from "$lib/types/projects";
import type { PromptPackRunListItem } from "$lib/types/prompt-packs";
import type { YoutubePreview } from "$lib/types/sources";
import type { YoutubeContentStatus, YoutubePlaylistDetail } from "$lib/types/youtube";
import type { ResearchProjectsWorkflowState } from "$lib/ui/research-projects-workflow";
import { projectSourceGridColumns } from "$lib/ui/research-projects-project-source-grid";
import type {
  LibrarySourceView,
  ProjectSourceLinkView,
  ResearchProjectView,
} from "$lib/ui/research-projects-model";
import ProjectsShell from "./ProjectsShell.svelte";
import ProjectRail from "./ProjectRail.svelte";
import ProjectInspector from "./ProjectInspector.svelte";
import ProjectRunsTab from "./ProjectRunsTab.svelte";
import ProjectRunsScreen from "./ProjectRunsScreen.svelte";
import ProjectRunDialog from "./ProjectRunDialog.svelte";
import ConnectFromLibrary from "./ConnectFromLibrary.svelte";
import ProjectSourceSummary from "./ProjectSourceSummary.svelte";
import TopCommandBar from "./TopCommandBar.svelte";
import ProjectWorkspace from "./ProjectWorkspace.svelte";
import YoutubeSummaryRunsPanel from "./YoutubeSummaryRunsPanel.svelte";

const api = vi.hoisted(() => ({
  addProjectSources: vi.fn(),
  addYoutubeSource: vi.fn(),
  cancelPromptPackRun: vi.fn(),
  createProject: vi.fn(),
  deleteAnalysisRun: vi.fn(),
  deleteProject: vi.fn(),
  deleteProjectYoutubeVideoSourceFromLibrary: vi.fn(),
  deletePromptPackRun: vi.fn(),
  getProjectDataRange: vi.fn(),
  getYoutubePlaylistDetail: vi.fn(),
  getLlmProfiles: vi.fn(),
  listActivePromptPackRuns: vi.fn(),
  listAnalysisPromptTemplates: vi.fn(),
  listLibraryCatalog: vi.fn(),
  listProjectRuns: vi.fn(),
  listProjectSources: vi.fn(),
  listProjects: vi.fn(),
  listResearchProjects: vi.fn(),
  listPromptPackRuns: vi.fn(),
  listSourceJobs: vi.fn(),
  listAnalysisSourceGroups: vi.fn(),
  listenToAnalysisRunEvents: vi.fn(),
  listenToPromptPackRunEvents: vi.fn(),
  listenToSourceJobEvents: vi.fn(),
  openUrl: vi.fn(),
  previewYoutubeSource: vi.fn(),
  removeProjectSources: vi.fn(),
  startProjectAnalysis: vi.fn(),
  setProjectArchived: vi.fn(),
  setProjectPinned: vi.fn(),
  syncYoutubeSource: vi.fn(),
  unlistenAnalysisRuns: vi.fn(),
  unlistenPromptPackRuns: vi.fn(),
  unlistenSourceJobs: vi.fn(),
  updateProject: vi.fn(),
  updateAnalysisSourceGroup: vi.fn(),
  updatePromptPackRun: vi.fn(),
}));

vi.mock("$lib/api/projects", () => ({
  addProjectSources: api.addProjectSources,
  createProject: api.createProject,
  deleteProject: api.deleteProject,
  deleteProjectYoutubeVideoSourceFromLibrary: api.deleteProjectYoutubeVideoSourceFromLibrary,
  getProjectDataRange: api.getProjectDataRange,
  listProjectRuns: api.listProjectRuns,
  listProjectSources: api.listProjectSources,
  listProjects: api.listProjects,
  listResearchProjects: api.listResearchProjects,
  removeProjectSources: api.removeProjectSources,
  setProjectArchived: api.setProjectArchived,
  setProjectPinned: api.setProjectPinned,
  startProjectAnalysis: api.startProjectAnalysis,
  syncYoutubeSource: api.syncYoutubeSource,
  updateProject: api.updateProject,
}));

vi.mock("$lib/api/library-sources", () => ({
  listLibraryCatalog: api.listLibraryCatalog,
}));

vi.mock("$lib/api/sources", () => ({
  addYoutubeSource: api.addYoutubeSource,
  previewYoutubeSource: api.previewYoutubeSource,
}));

vi.mock("$lib/api/source-jobs", () => ({
  listSourceJobs: api.listSourceJobs,
  listenToSourceJobEvents: api.listenToSourceJobEvents,
  syncYoutubeSource: api.syncYoutubeSource,
}));

vi.mock("$lib/api/youtube-detail", () => ({
  getYoutubePlaylistDetail: api.getYoutubePlaylistDetail,
}));

vi.mock("$lib/api/analysis-runs", () => ({
  deleteAnalysisRun: api.deleteAnalysisRun,
  listenToAnalysisRunEvents: api.listenToAnalysisRunEvents,
}));

vi.mock("$lib/api/analysis-source-groups", () => ({
  listAnalysisSourceGroups: api.listAnalysisSourceGroups,
  listAnalysisPromptTemplates: api.listAnalysisPromptTemplates,
  updateAnalysisSourceGroup: api.updateAnalysisSourceGroup,
}));

vi.mock("$lib/components/research-projects/ResearchProjectsShell.svelte", async () => {
  const receiver = await import("$lib/testing/ProjectsRouteReceiver.svelte");
  return { default: receiver.default };
});

vi.mock("$lib/api/llm", () => ({
  getLlmProfiles: api.getLlmProfiles,
}));

vi.mock("$lib/api/prompt-packs", () => ({
  cancelPromptPackRun: api.cancelPromptPackRun,
  deletePromptPackRun: api.deletePromptPackRun,
  listActivePromptPackRuns: api.listActivePromptPackRuns,
  listPromptPackRuns: api.listPromptPackRuns,
  listenToPromptPackRunEvents: api.listenToPromptPackRunEvents,
  updatePromptPackRun: api.updatePromptPackRun,
}));

vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: api.openUrl }));

type ProjectsShellProps = ComponentProps<typeof ProjectsShell>;
type ProjectRailProps = ComponentProps<typeof ProjectRail>;
type ProjectInspectorProps = ComponentProps<typeof ProjectInspector>;
type ProjectRunsTabProps = ComponentProps<typeof ProjectRunsTab>;
type ProjectRunsScreenProps = ComponentProps<typeof ProjectRunsScreen>;
type ProjectRunDialogProps = ComponentProps<typeof ProjectRunDialog>;
type ConnectFromLibraryProps = ComponentProps<typeof ConnectFromLibrary>;
type ProjectSourceSummaryProps = ComponentProps<typeof ProjectSourceSummary>;
type TopCommandBarProps = ComponentProps<typeof TopCommandBar>;
type ProjectWorkspaceProps = ComponentProps<typeof ProjectWorkspace>;
type YoutubeSummaryRunsPanelProps = ComponentProps<typeof YoutubeSummaryRunsPanel>;

function projectRecord(overrides: Partial<ProjectRecord> = {}): ProjectRecord {
  return {
    id: 1,
    name: "Smoke project",
    description: null,
    created_at: 1,
    updated_at: 1,
    ...overrides,
  };
}

function projectSummary(overrides: Partial<ProjectSummary> = {}): ProjectSummary {
  return {
    id: 1,
    name: "Smoke project",
    description: null,
    source_count: 1,
    material_count: 4,
    status: "ready",
    last_run_at: null,
    pinned: false,
    archived: false,
    updated_at: 1,
    ...overrides,
  };
}

function researchProjectView(
  overrides: Partial<ResearchProjectView> = {},
): ResearchProjectView {
  return {
    id: "project:1",
    projectId: 1,
    title: "Smoke project",
    description: "Project description",
    periodLabel: "All time",
    sourceCount: 1,
    evidenceCount: 4,
    materialCount: 4,
    lastRunLabel: null,
    status: "ready",
    backing: { kind: "project", projectId: 1 },
    ...overrides,
  };
}

function projectSourceRecord(
  overrides: Partial<ProjectSourceRecord> = {},
): ProjectSourceRecord {
  return {
    project_id: 1,
    source_id: 11,
    provider: "youtube",
    source_subtype: "video",
    title: "Evidence video",
    subtitle: "Research channel",
    item_count: 4,
    added_at: 1_700_000_000,
    last_synced_at: 1_700_000_100,
    sync_status: "active",
    handle: null,
    ...overrides,
  };
}

function projectSourceLink(
  overrides: Partial<ProjectSourceLinkView> = {},
): ProjectSourceLinkView {
  return {
    projectId: "project:1",
    sourceId: "source:11",
    sourceNumericId: 11,
    provider: "youtube",
    subtype: "video",
    typeLabel: "YouTube / Video",
    title: "Evidence video",
    subtitle: "Research channel",
    itemCount: 4,
    localCopyLabel: "4 materials",
    addedAt: 1_700_000_000,
    addedAtLabel: "14/11/2023, 22:13",
    connectionStatus: "connected",
    filterSummary: "Research channel",
    ...overrides,
  };
}

function librarySourceView(overrides: Partial<LibrarySourceView> = {}): LibrarySourceView {
  return {
    id: "source:11",
    sourceId: 11,
    provider: "youtube",
    typeLabel: "YouTube / Video",
    title: "Evidence video",
    subtitle: "Research channel",
    projectCount: 0,
    lastCollectedAt: 1_700_000_100,
    lastCollectedLabel: "14/11/2023, 22:15",
    localCopyLabel: "4 materials",
    status: "active",
    disabledReason: null,
    alreadyConnected: false,
    connectable: true,
    ...overrides,
  };
}

function analysisRun(overrides: Partial<AnalysisRunSummary> = {}): AnalysisRunSummary {
  return {
    id: 71,
    run_type: "project",
    scope_type: "project",
    source_id: null,
    source_title: null,
    source_group_id: null,
    source_group_name: null,
    project_id: 1,
    project_name: "Smoke project",
    scope_label: "Smoke project",
    period_from: 0,
    period_to: 86_399,
    output_language: "en",
    prompt_template_id: 5,
    prompt_template_name: "Evidence brief",
    prompt_template_version: 2,
    provider_profile: "default",
    provider: "openai",
    model: "gpt-test",
    youtube_corpus_mode: "transcript_description",
    telegram_history_scope: "current",
    status: "completed",
    error: null,
    has_trace_data: false,
    snapshot_state: "captured",
    snapshot_captured_at: "2026-08-05T00:00:00Z",
    snapshot_error: null,
    created_at: 1_700_000_000,
    completed_at: 1_700_000_100,
    ...overrides,
  };
}

function promptTemplate(
  overrides: Partial<AnalysisPromptTemplate> = {},
): AnalysisPromptTemplate {
  return {
    id: 5,
    name: "Evidence brief",
    template_kind: "report",
    body: "Summarize evidence",
    version: 2,
    is_builtin: true,
    created_at: 1,
    updated_at: 1,
    ...overrides,
  };
}

function promptPackRun(
  overrides: Partial<PromptPackRunListItem> = {},
): PromptPackRunListItem {
  return {
    runId: 91,
    projectId: 1,
    runLabel: "Prompt detail",
    runtimeProvider: "api",
    packId: "youtube-summary",
    packVersion: "1",
    runStatus: "complete",
    resultStatus: "complete",
    latestMessage: "Prompt pack complete",
    ...overrides,
  };
}

function libraryCatalogRecord(
  overrides: Partial<LibraryCatalogRecord> = {},
): LibraryCatalogRecord {
  return {
    source: {
      source_id: 11,
      provider: "youtube",
      source_subtype: "video",
      account_id: null,
      external_id: "video-11",
      title: "Evidence video",
      subtitle: "Research channel",
      canonical_url: "https://www.youtube.com/watch?v=video-11",
      created_at: 1_700_000_000,
      last_synced_at: 1_700_000_100,
      item_count: 4,
      project_count: 0,
      youtube: {
        video_form: "watch",
        duration_seconds: 120,
        playlist_video_count: null,
        channel_title: "Research channel",
        availability_status: "available",
      },
      telegram: null,
    },
    latest_job: null,
    status: "active",
    status_detail: null,
    capabilities: {
      can_refresh_source: true,
      can_delete: true,
      can_edit: true,
      can_connect_to_project: true,
    },
    disabled_reasons: {
      refresh_source: null,
      delete: null,
      edit: null,
      connect_to_project: null,
    },
    ...overrides,
  };
}

function playlistCatalogRecord(): LibraryCatalogRecord {
  const record = libraryCatalogRecord();
  return {
    ...record,
    source: {
      ...record.source,
      source_id: 31,
      source_subtype: "playlist",
      external_id: "playlist-31",
      title: "Evidence playlist",
      canonical_url: "https://www.youtube.com/playlist?list=playlist-31",
      item_count: 2,
      youtube: {
        ...record.source.youtube!,
        video_form: "playlist",
        duration_seconds: null,
        playlist_video_count: 2,
      },
    },
  };
}

function youtubeContentStatus(): YoutubeContentStatus {
  return {
    state: "not_synced",
    itemCount: 0,
    segmentCount: 0,
    lastSyncedAt: null,
    label: "Not synced",
  };
}

function youtubePlaylistDetail(): YoutubePlaylistDetail {
  return {
    summary: {
      sourceId: 31,
      sourceSubtype: "playlist",
      title: "Evidence playlist",
      channelTitle: "Research channel",
      channelHandle: "@research",
      canonicalUrl: "https://www.youtube.com/playlist?list=playlist-31",
      thumbnailUrl: null,
      durationSeconds: null,
      publishedAt: null,
      availabilityStatus: "available",
      videoCount: 2,
      linkedVideoCount: 0,
      unavailableCount: 0,
      captions: youtubeContentStatus(),
      comments: youtubeContentStatus(),
    },
    items: [
      {
        position: 1,
        videoId: "playlist-video-1",
        videoSourceId: null,
        title: "First playlist video",
        canonicalUrl: "https://www.youtube.com/watch?v=playlist-video-1",
        thumbnailUrl: null,
        durationSeconds: 90,
        publishedAt: 1_700_000_000,
        availabilityStatus: "available",
        isRemovedFromPlaylist: false,
        captions: youtubeContentStatus(),
        comments: youtubeContentStatus(),
      },
      {
        position: 2,
        videoId: "playlist-video-2",
        videoSourceId: null,
        title: "Second playlist video",
        canonicalUrl: "https://www.youtube.com/watch?v=playlist-video-2",
        thumbnailUrl: null,
        durationSeconds: 120,
        publishedAt: 1_700_000_100,
        availabilityStatus: "available",
        isRemovedFromPlaylist: false,
        captions: youtubeContentStatus(),
        comments: youtubeContentStatus(),
      },
    ],
  };
}

function youtubePreview(overrides: Partial<YoutubePreview> = {}): YoutubePreview {
  return {
    kind: "video",
    externalId: "video-11",
    canonicalUrl: "https://www.youtube.com/watch?v=video-11",
    title: "Evidence video",
    channelTitle: "Research channel",
    channelId: "channel-1",
    channelHandle: "@research",
    channelUrl: "https://www.youtube.com/@research",
    thumbnailUrl: null,
    durationSeconds: 120,
    publishedAt: "2026-08-05T00:00:00Z",
    playlistVideoCount: null,
    captionsEstimate: { hasManual: true, hasAuto: false, languages: ["en"] },
    availabilityStatus: "available",
    warnings: [],
    ...overrides,
  };
}

function emptyProjectsState(
  overrides: Partial<ResearchProjectsWorkflowState> = {},
): ResearchProjectsWorkflowState {
  return {
    projectsRaw: [],
    projectSources: [],
    runs: [],
    libraryCatalogRecords: [],
    sourceJobs: [],
    promptTemplates: [],
    projects: [],
    librarySources: [],
    projectSourceLinks: [],
    selectedProjectId: null,
    selectedLibrarySourceIds: new Set(),
    loading: false,
    saving: false,
    status: "",
    ...overrides,
  };
}

function projectsShellProps(overrides: Partial<ProjectsShellProps> = {}): ProjectsShellProps {
  return {
    state: emptyProjectsState(),
    onSelectProject: vi.fn(),
    onCreateProject: vi.fn(),
    onUpdateProject: vi.fn(),
    onDeleteProject: vi.fn(),
    onRemoveProjectSource: vi.fn(),
    onRunProject: vi.fn(),
    onConnectSelectedSources: vi.fn(),
    onConnectAddedProjectSource: vi.fn(),
    onConnectAddedProjectSources: vi.fn(),
    onConnectExistingProjectSource: vi.fn(),
    onSelectedLibrarySourceIdsChange: vi.fn(),
    onRefreshProjectRuns: vi.fn(),
    onSyncSelectedSources: vi.fn(),
    onSetStatus: vi.fn(),
    ...overrides,
  };
}

function projectRailProps(overrides: Partial<ProjectRailProps> = {}): ProjectRailProps {
  return {
    projects: [],
    selectedProjectId: null,
    onSelectProject: vi.fn(),
    onCreateProject: vi.fn(),
    ...overrides,
  };
}

function projectInspectorProps(overrides: Partial<ProjectInspectorProps> = {}): ProjectInspectorProps {
  return {
    project: null,
    sources: [],
    selectedSource: null,
    runs: [],
    onEditProject: vi.fn(),
    onDeleteProject: vi.fn(),
    onRunProject: vi.fn(),
    onRemoveSource: vi.fn(),
    ...overrides,
  };
}

function projectRunsTabProps(overrides: Partial<ProjectRunsTabProps> = {}): ProjectRunsTabProps {
  return { runs: [], onRefreshProjectRuns: vi.fn(), ...overrides };
}

function projectRunsScreenProps(): ProjectRunsScreenProps {
  return {};
}

function projectRunDialogProps(overrides: Partial<ProjectRunDialogProps> = {}): ProjectRunDialogProps {
  return { project: null, templates: [], onSubmit: vi.fn(), ...overrides };
}

function connectFromLibraryProps(
  overrides: Partial<ConnectFromLibraryProps> = {},
): ConnectFromLibraryProps {
  return {
    open: false,
    project: null,
    librarySources: [],
    selectedSourceIds: new Set(),
    saving: false,
    status: "",
    onOpenChange: vi.fn(),
    onSelectedSourceIdsChange: vi.fn(),
    onConnectSelectedSources: vi.fn(),
    ...overrides,
  };
}

function projectSourceSummaryProps(
  overrides: Partial<ProjectSourceSummaryProps> = {},
): ProjectSourceSummaryProps {
  return { project: null, connectedCount: 0, materialCount: 0, libraryCount: 0, ...overrides };
}

function topCommandBarProps(overrides: Partial<TopCommandBarProps> = {}): TopCommandBarProps {
  return { project: null, sources: [], onRunProject: vi.fn(), ...overrides };
}

function projectWorkspaceProps(overrides: Partial<ProjectWorkspaceProps> = {}): ProjectWorkspaceProps {
  return {
    project: null,
    projectSourceLinks: [],
    librarySources: [],
    runs: [],
    selectedSourceIds: [],
    onSelectedSourceIdsChange: vi.fn(),
    onOpenAddSource: vi.fn(),
    onOpenConnectLibrary: vi.fn(),
    onRefreshProjectRuns: vi.fn(),
    onRemoveSource: vi.fn(),
    onSyncSelectedSources: vi.fn(),
    ...overrides,
  };
}

function youtubeSummaryRunsPanelProps(
  overrides: Partial<YoutubeSummaryRunsPanelProps> = {},
): YoutubeSummaryRunsPanelProps {
  return { projectId: null, ...overrides };
}

function arrangeCurrentProjectsRoute(
  sources: ProjectSourceRecord[] = [],
  catalogRecords: LibraryCatalogRecord[] = [libraryCatalogRecord()],
) {
  api.listProjects.mockResolvedValue([projectRecord()]);
  api.listProjectSources.mockResolvedValue(sources);
  api.listLibraryCatalog.mockResolvedValue({
    sources: catalogRecords,
    filter_counts: [],
  });
}

async function renderCurrentProjectsRoute(route: "main" | "list") {
  const page =
    route === "main"
      ? await import("../../../routes/projects/+page.svelte")
      : await import("../../../routes/projects/list/+page.svelte");
  const view = render(page.default);

  await waitFor(() => {
    expect(api.listProjects).toHaveBeenCalled();
    expect(api.listProjectSources).toHaveBeenCalledWith(1);
  });
  await screen.findAllByRole("heading", { name: "Smoke project" });
  return view;
}

async function selectCurrentProjectSource(title = "Evidence video") {
  const grid = await screen.findByRole("region", { name: "Project sources" });
  await fireEvent.click(await within(grid).findByText(title));
  await screen.findByRole("button", {
    name: "Delete selected YouTube video from Library",
  });
}

async function openProjectAddSourceDialog() {
  await fireEvent.click(await screen.findByRole("button", { name: "Add source to project" }));
  return screen.findByRole("dialog", { name: "Add source" });
}

async function addNewVideoThroughOpenDialog(options: {
  sourceId: number;
  title: string;
  url: string;
}) {
  api.previewYoutubeSource.mockResolvedValueOnce(youtubePreview({
    externalId: `video-${options.sourceId}`,
    canonicalUrl: options.url,
    title: options.title,
  }));
  api.addYoutubeSource.mockResolvedValueOnce({
    id: options.sourceId,
    title: options.title,
    externalId: `video-${options.sourceId}`,
  });

  const dialog = screen.getByRole("dialog", { name: "Add source" });
  await fireEvent.input(within(dialog).getByLabelText("YouTube URL"), {
    target: { value: options.url },
  });
  await fireEvent.click(within(dialog).getByRole("button", { name: "Preview" }));
  await within(dialog).findByText(options.title);
  await fireEvent.click(within(dialog).getByRole("button", { name: "Add source" }));
  await waitFor(() => {
    expect(api.addYoutubeSource).toHaveBeenCalledOnce();
    expect(api.addYoutubeSource).toHaveBeenCalledWith(options.url, {
      materializePlaylistVideos: true,
    });
  });
}

async function addPlaylistVideosThroughOpenDialog(sourceIds: [number, number]) {
  api.getYoutubePlaylistDetail.mockResolvedValueOnce(youtubePlaylistDetail());
  api.addYoutubeSource
    .mockResolvedValueOnce({
      id: sourceIds[0],
      title: "First playlist video",
      externalId: "playlist-video-1",
    })
    .mockResolvedValueOnce({
      id: sourceIds[1],
      title: "Second playlist video",
      externalId: "playlist-video-2",
    });

  const dialog = screen.getByRole("dialog", { name: "Add source" });
  await fireEvent.click(within(dialog).getByRole("tab", { name: "From existing data" }));
  await fireEvent.click(await within(dialog).findByRole("button", { name: /Evidence playlist/ }));
  await waitFor(() => expect(api.getYoutubePlaylistDetail).toHaveBeenCalledWith(31));
  await fireEvent.click(await within(dialog).findByRole("checkbox", { name: /First playlist video/ }));
  await fireEvent.click(within(dialog).getByRole("checkbox", { name: /Second playlist video/ }));
  await fireEvent.click(within(dialog).getByRole("button", { name: "Add selected" }));
  await within(dialog).findByText("Added 2, skipped 0, failed 0.");

  expect(api.addYoutubeSource).toHaveBeenCalledTimes(2);
  expect(api.addYoutubeSource).toHaveBeenNthCalledWith(
    1,
    "https://www.youtube.com/watch?v=playlist-video-1",
  );
  expect(api.addYoutubeSource).toHaveBeenNthCalledWith(
    2,
    "https://www.youtube.com/watch?v=playlist-video-2",
  );
}

beforeEach(() => {
  class ResizeObserverStub {
    constructor(private readonly callback: ResizeObserverCallback) {}
    observe(target: Element) {
      this.callback(
        [
          {
            target,
            contentRect: { width: 1_200, height: 720 },
          } as ResizeObserverEntry,
        ],
        this as unknown as ResizeObserver,
      );
    }
    unobserve() {}
    disconnect() {}
  }

  vi.stubGlobal("ResizeObserver", ResizeObserverStub);
  api.addProjectSources.mockResolvedValue({ added_count: 0, already_present_count: 0 });
  api.addYoutubeSource.mockResolvedValue({ id: 11, title: "Evidence video", externalId: "video-11" });
  api.cancelPromptPackRun.mockResolvedValue(undefined);
  api.createProject.mockResolvedValue(projectRecord());
  api.deleteAnalysisRun.mockResolvedValue(undefined);
  api.deleteProject.mockResolvedValue(undefined);
  api.deleteProjectYoutubeVideoSourceFromLibrary.mockResolvedValue({
    status: "deleted",
    blocking_projects: [],
    remaining_blocking_project_count: 0,
  });
  api.deletePromptPackRun.mockResolvedValue(undefined);
  api.getProjectDataRange.mockResolvedValue({ from: null, to: null });
  api.getYoutubePlaylistDetail.mockResolvedValue(null);
  api.getLlmProfiles.mockResolvedValue({ active_profile: "", profiles: [] });
  api.listActivePromptPackRuns.mockResolvedValue([]);
  api.listAnalysisPromptTemplates.mockResolvedValue([]);
  api.listLibraryCatalog.mockResolvedValue({ sources: [], filter_counts: [] });
  api.listProjectRuns.mockResolvedValue([]);
  api.listProjectSources.mockResolvedValue([]);
  api.listProjects.mockResolvedValue([]);
  api.listResearchProjects.mockResolvedValue([]);
  api.listPromptPackRuns.mockResolvedValue([]);
  api.listSourceJobs.mockResolvedValue([]);
  api.listenToAnalysisRunEvents.mockResolvedValue(api.unlistenAnalysisRuns);
  api.listenToPromptPackRunEvents.mockResolvedValue(api.unlistenPromptPackRuns);
  api.listenToSourceJobEvents.mockResolvedValue(api.unlistenSourceJobs);
  api.openUrl.mockResolvedValue(undefined);
  api.previewYoutubeSource.mockResolvedValue(youtubePreview());
  api.removeProjectSources.mockResolvedValue(undefined);
  api.startProjectAnalysis.mockResolvedValue(1);
  api.setProjectArchived.mockResolvedValue(undefined);
  api.setProjectPinned.mockResolvedValue(undefined);
  api.syncYoutubeSource.mockResolvedValue({});
  api.updateProject.mockResolvedValue(projectRecord());
  api.updatePromptPackRun.mockResolvedValue({ runId: 1, runStatus: "complete" });
});

afterEach(cleanup);

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
  vi.resetAllMocks();
});

it("smoke renders ProjectsShell", async () => {
  render(ProjectsShell, { props: projectsShellProps() });

  await waitFor(() => expect(api.getLlmProfiles).toHaveBeenCalledOnce());

  expect(screen.getByText("Research Projects")).toBeTruthy();
});

it("smoke renders ProjectRail", () => {
  render(ProjectRail, { props: projectRailProps() });

  expect(screen.getByRole("listbox", { name: "Research projects" })).toBeTruthy();
});

it("smoke renders ProjectInspector", () => {
  render(ProjectInspector, { props: projectInspectorProps() });

  expect(screen.getByText("Inspector")).toBeTruthy();
});

it("smoke renders ProjectRunsTab", () => {
  render(ProjectRunsTab, { props: projectRunsTabProps() });

  expect(screen.getByRole("region", { name: "Project analysis runs" })).toBeTruthy();
});

it("smoke renders ProjectRunsScreen", async () => {
  render(ProjectRunsScreen, { props: projectRunsScreenProps() });

  await waitFor(() => expect(api.listPromptPackRuns).toHaveBeenCalledOnce());

  expect(screen.getByRole("heading", { name: "Prompt Pack runs" })).toBeTruthy();
});

it("smoke renders ProjectRunDialog", () => {
  render(ProjectRunDialog, { props: projectRunDialogProps({ open: true }) });

  expect(screen.getByText("Run project analysis")).toBeTruthy();
});

it("smoke renders ConnectFromLibrary", () => {
  render(ConnectFromLibrary, { props: connectFromLibraryProps({ open: true }) });

  expect(screen.getByText("Connect from Library")).toBeTruthy();
});

it("smoke renders ProjectSourceSummary", () => {
  render(ProjectSourceSummary, { props: projectSourceSummaryProps() });

  expect(screen.getByRole("region", { name: "Project source summary" })).toBeTruthy();
});

it("smoke renders TopCommandBar", () => {
  render(TopCommandBar, { props: topCommandBarProps() });

  expect(screen.getByText("Research Projects")).toBeTruthy();
});

it("smoke renders ProjectWorkspace", () => {
  render(ProjectWorkspace, { props: projectWorkspaceProps() });

  expect(screen.getByRole("heading", { name: "Research project" })).toBeTruthy();
});

it("smoke renders YoutubeSummaryRunsPanel", async () => {
  render(YoutubeSummaryRunsPanel, { props: youtubeSummaryRunsPanelProps() });

  await waitFor(() => expect(api.listPromptPackRuns).toHaveBeenCalledOnce());

  expect(screen.getByRole("region", { name: "Prompt Pack runs" })).toBeTruthy();
});

it("smoke renders main Projects route", async () => {
  const { default: ProjectsPage } = await import("../../../routes/projects/+page.svelte");
  const view = render(ProjectsPage);

  await waitFor(() => {
    expect(api.listProjects).toHaveBeenCalledOnce();
    expect(api.listLibraryCatalog).toHaveBeenCalledOnce();
    expect(api.listSourceJobs).toHaveBeenCalledOnce();
    expect(api.listAnalysisPromptTemplates).toHaveBeenCalledOnce();
  });
  await waitFor(() => expect(api.listenToAnalysisRunEvents).toHaveBeenCalledOnce());
  await waitFor(() => expect(api.listenToSourceJobEvents).toHaveBeenCalledOnce());

  expect(view.container.querySelector('[data-ui-route="research-projects"]')).toBeTruthy();

  view.unmount();

  expect(api.unlistenAnalysisRuns).toHaveBeenCalledOnce();
  expect(api.unlistenSourceJobs).toHaveBeenCalledOnce();
});

it("smoke renders list Projects route", async () => {
  const { default: ProjectsListPage } = await import("../../../routes/projects/list/+page.svelte");
  const view = render(ProjectsListPage);

  await waitFor(() => {
    expect(api.listProjects).toHaveBeenCalledOnce();
    expect(api.listLibraryCatalog).toHaveBeenCalledOnce();
    expect(api.listSourceJobs).toHaveBeenCalledOnce();
    expect(api.listAnalysisPromptTemplates).toHaveBeenCalledOnce();
  });
  await waitFor(() => expect(api.listenToAnalysisRunEvents).toHaveBeenCalledOnce());
  await waitFor(() => expect(api.listenToSourceJobEvents).toHaveBeenCalledOnce());

  expect(view.container.querySelector('[data-ui-route="research-projects"]')).toBeTruthy();

  view.unmount();

  expect(api.unlistenAnalysisRuns).toHaveBeenCalledOnce();
  expect(api.unlistenSourceJobs).toHaveBeenCalledOnce();
});

it("smoke renders next Projects route", async () => {
  const page = await import("../../../routes/projects/next/+page.svelte");
  const view = render(page.default);

  await waitFor(() => {
    expect(api.listResearchProjects).toHaveBeenCalledOnce();
    expect(api.listAnalysisPromptTemplates).toHaveBeenCalledOnce();
  });

  expect(screen.getByRole("main")).toBeTruthy();

  view.unmount();
  await tick();
});

it("uses real project APIs instead of analysis source group APIs", async () => {
  arrangeCurrentProjectsRoute([projectSourceRecord()]);
  api.listProjectRuns.mockResolvedValue([analysisRun()]);

  const view = await renderCurrentProjectsRoute("main");

  expect(api.listProjects).toHaveBeenCalledOnce();
  expect(api.listProjectSources).toHaveBeenCalledWith(1);
  expect(api.listProjectRuns).toHaveBeenCalledWith(1);
  expect(api.listLibraryCatalog).toHaveBeenCalledOnce();
  expect(api.listAnalysisPromptTemplates).toHaveBeenCalledWith("report");
  expect(api.listAnalysisSourceGroups).not.toHaveBeenCalled();
  expect(api.updateAnalysisSourceGroup).not.toHaveBeenCalled();

  view.unmount();
  await tick();
});

it("passes project add-source workflow callbacks from both current project routes", async () => {
  for (const route of ["main", "list"] as const) {
    arrangeCurrentProjectsRoute([], []);
    let view = await renderCurrentProjectsRoute(route);
    await openProjectAddSourceDialog();
    await addNewVideoThroughOpenDialog({
      sourceId: 21,
      title: "New route video",
      url: "https://www.youtube.com/watch?v=new-route-video",
    });

    await waitFor(() => {
      expect(api.addProjectSources).toHaveBeenCalledOnce();
      expect(api.addProjectSources).toHaveBeenCalledWith({ projectId: 1, sourceIds: [21] });
    });
    view.unmount();
    await tick();
    vi.clearAllMocks();

    arrangeCurrentProjectsRoute([], [playlistCatalogRecord()]);
    view = await renderCurrentProjectsRoute(route);
    await openProjectAddSourceDialog();
    await addPlaylistVideosThroughOpenDialog([22, 23]);

    await waitFor(() => {
      expect(api.addProjectSources).toHaveBeenCalledOnce();
      expect(api.addProjectSources).toHaveBeenCalledWith({
        projectId: 1,
        sourceIds: [22, 23],
      });
    });
    view.unmount();
    await tick();
    vi.clearAllMocks();
  }
});

it("wires project source Library delete through the main projects route", async () => {
  arrangeCurrentProjectsRoute([projectSourceRecord()]);
  const view = await renderCurrentProjectsRoute("main");

  await selectCurrentProjectSource();
  await fireEvent.click(
    screen.getByRole("button", { name: "Delete selected YouTube video from Library" }),
  );
  await fireEvent.click(screen.getByRole("button", { name: "Delete from Library permanently" }));

  await waitFor(() => {
    expect(api.deleteProjectYoutubeVideoSourceFromLibrary).toHaveBeenCalledWith({
      projectId: 1,
      sourceId: 11,
    });
  });

  view.unmount();
  await tick();
});

it("wires project source Library delete through the list projects route", async () => {
  arrangeCurrentProjectsRoute([projectSourceRecord()]);
  const view = await renderCurrentProjectsRoute("list");

  await selectCurrentProjectSource();
  await fireEvent.click(
    screen.getByRole("button", { name: "Delete selected YouTube video from Library" }),
  );
  await fireEvent.click(screen.getByRole("button", { name: "Delete from Library permanently" }));

  await waitFor(() => {
    expect(api.deleteProjectYoutubeVideoSourceFromLibrary).toHaveBeenCalledWith({
      projectId: 1,
      sourceId: 11,
    });
  });

  view.unmount();
  await tick();
});

it("keeps Remove membership-only and adds a separate Delete from Library action", async () => {
  arrangeCurrentProjectsRoute([projectSourceRecord()]);
  vi.stubGlobal("confirm", vi.fn(() => true));
  const view = await renderCurrentProjectsRoute("main");

  await selectCurrentProjectSource();
  await fireEvent.click(screen.getByRole("button", { name: "Remove 1 selected source" }));

  await waitFor(() => {
    expect(api.removeProjectSources).toHaveBeenCalledWith({ projectId: 1, sourceIds: [11] });
    expect(api.deleteProjectYoutubeVideoSourceFromLibrary).not.toHaveBeenCalled();
    expect(screen.queryByRole("button", { name: "Remove 1 selected source" })).toBeNull();
  });

  await selectCurrentProjectSource();
  await fireEvent.click(
    screen.getByRole("button", { name: "Delete selected YouTube video from Library" }),
  );
  await fireEvent.click(screen.getByRole("button", { name: "Delete from Library permanently" }));

  await waitFor(() => {
    expect(api.deleteProjectYoutubeVideoSourceFromLibrary).toHaveBeenCalledWith({
      projectId: 1,
      sourceId: 11,
    });
  });
  expect(api.removeProjectSources).toHaveBeenCalledOnce();

  view.unmount();
  await tick();
});

it("wires the project Add source dialog through the next Projects route", async () => {
  api.listResearchProjects.mockResolvedValue([projectSummary()]);
  api.listProjectSources.mockResolvedValue([]);
  const page = await import("../../../routes/projects/next/+page.svelte");
  const view = render(page.default);

  await fireEvent.click(screen.getByRole("button", { name: "Select route project" }));
  await waitFor(() => expect(api.listProjectSources).toHaveBeenCalledWith(1));
  await fireEvent.click(screen.getByRole("button", { name: "Open route add source" }));

  const dialog = await screen.findByRole("dialog", { name: "Add source" });
  await fireEvent.input(within(dialog).getByLabelText("YouTube URL"), {
    target: { value: "https://www.youtube.com/watch?v=video-11" },
  });
  await fireEvent.click(within(dialog).getByRole("button", { name: "Preview" }));
  await within(dialog).findByText("Evidence video");
  await fireEvent.click(within(dialog).getByRole("button", { name: "Add source" }));

  await waitFor(() => {
    expect(api.addYoutubeSource).toHaveBeenCalledOnce();
    expect(api.addProjectSources).toHaveBeenCalledWith({ projectId: 1, sourceIds: [11] });
  });

  view.unmount();
  await tick();
});

it("wires Delete from Library in the next projects bulk bar", async () => {
  api.listResearchProjects.mockResolvedValue([projectSummary()]);
  api.listProjectSources.mockResolvedValue([
    projectSourceRecord({
      source_id: 12,
      source_subtype: "playlist",
      title: "Ineligible playlist",
      handle: "playlist-12",
    }),
  ]);
  const page = await import("../../../routes/projects/next/+page.svelte");
  let view = render(page.default);

  await fireEvent.click(screen.getByRole("button", { name: "Select route project" }));
  await waitFor(() => expect(api.listProjectSources).toHaveBeenCalledWith(1));
  await fireEvent.click(screen.getByRole("button", { name: "Select ineligible route source" }));
  await tick();

  const ineligibleDelete = screen.getByRole("button", { name: "Run route Library delete" });
  expect((ineligibleDelete as HTMLButtonElement).disabled).toBe(true);
  expect(ineligibleDelete.getAttribute("title")).toBe(
    "Only YouTube videos can be deleted from Library here",
  );
  await fireEvent.click(ineligibleDelete);
  expect(api.deleteProjectYoutubeVideoSourceFromLibrary).not.toHaveBeenCalled();

  view.unmount();
  await tick();
  vi.clearAllMocks();

  api.listResearchProjects.mockResolvedValue([projectSummary()]);
  api.listProjectSources.mockResolvedValue([projectSourceRecord({ handle: "video-11" })]);
  view = render(page.default);

  await fireEvent.click(screen.getByRole("button", { name: "Select route project" }));
  await waitFor(() => expect(api.listProjectSources).toHaveBeenCalledWith(1));
  await fireEvent.click(screen.getByRole("button", { name: "Select route source" }));
  await tick();
  const eligibleDelete = screen.getByRole("button", { name: "Run route Library delete" });
  expect((eligibleDelete as HTMLButtonElement).disabled).toBe(false);
  await fireEvent.click(eligibleDelete);

  await waitFor(() => {
    expect(api.deleteProjectYoutubeVideoSourceFromLibrary).toHaveBeenCalledWith({
      projectId: 1,
      sourceId: 11,
    });
  });

  view.unmount();
  await tick();
});

it("wires selected Workspace source syncs to the YouTube source job command", async () => {
  arrangeCurrentProjectsRoute([projectSourceRecord()]);
  let view = await renderCurrentProjectsRoute("main");

  await selectCurrentProjectSource();
  await fireEvent.click(screen.getByRole("button", { name: "Sync selected 1 source" }));

  await waitFor(() => {
    expect(api.syncYoutubeSource).toHaveBeenCalledOnce();
    expect(api.syncYoutubeSource).toHaveBeenCalledWith(11, {
      metadata: true,
      transcripts: true,
      comments: true,
    });
  });

  view.unmount();
  await tick();

  vi.clearAllMocks();
  arrangeCurrentProjectsRoute([
    projectSourceRecord({ source_id: 11, title: "First syncable video" }),
    projectSourceRecord({ source_id: 12, title: "Second syncable video" }),
  ]);
  view = await renderCurrentProjectsRoute("main");

  await fireEvent.click(screen.getByRole("button", { name: "Sync all sources" }));
  await waitFor(() => expect(api.syncYoutubeSource).toHaveBeenCalledTimes(2));
  expect(api.syncYoutubeSource).toHaveBeenNthCalledWith(1, 11, {
    metadata: true,
    transcripts: true,
    comments: true,
  });
  expect(api.syncYoutubeSource).toHaveBeenNthCalledWith(2, 12, {
    metadata: true,
    transcripts: true,
    comments: true,
  });

  view.unmount();
  await tick();

  vi.clearAllMocks();
  arrangeCurrentProjectsRoute([
    projectSourceRecord({
      source_id: 12,
      source_subtype: "playlist",
      title: "Unsupported sync playlist",
    }),
  ]);
  view = await renderCurrentProjectsRoute("main");

  const ineligibleSyncAll = screen.getByRole("button", { name: "Sync all sources" });
  expect((ineligibleSyncAll as HTMLButtonElement).disabled).toBe(true);
  expect(ineligibleSyncAll.getAttribute("title")).toBe(
    "Selected sources include unsupported sync types",
  );
  await fireEvent.click(ineligibleSyncAll);
  expect(api.syncYoutubeSource).not.toHaveBeenCalled();

  await selectCurrentProjectSource("Unsupported sync playlist");
  const ineligibleSyncSelected = screen.getByRole("button", {
    name: "Sync selected 1 source",
  });
  expect((ineligibleSyncSelected as HTMLButtonElement).disabled).toBe(true);
  expect(ineligibleSyncSelected.getAttribute("title")).toBe(
    "Selected sources include unsupported sync types",
  );
  await fireEvent.click(ineligibleSyncSelected);
  expect(api.syncYoutubeSource).not.toHaveBeenCalled();

  view.unmount();
  await tick();

  vi.clearAllMocks();
  api.listResearchProjects.mockResolvedValue([projectSummary()]);
  api.listProjectSources.mockResolvedValue([
    projectSourceRecord({ source_id: 11, title: "Filtered sync video" }),
    projectSourceRecord({
      source_id: 12,
      provider: "telegram",
      source_subtype: "supergroup",
      title: "Filtered sync group",
    }),
  ]);
  const nextPage = await import("../../../routes/projects/next/+page.svelte");
  view = render(nextPage.default);

  await fireEvent.click(screen.getByRole("button", { name: "Select route project" }));
  await waitFor(() => expect(api.listProjectSources).toHaveBeenCalledWith(1));
  await fireEvent.click(screen.getByRole("button", { name: "Select mixed route sources" }));
  await tick();
  const filteredSync = screen.getByRole("button", { name: "Run route sync" });
  expect((filteredSync as HTMLButtonElement).disabled).toBe(false);
  await fireEvent.click(filteredSync);
  await waitFor(() => expect(api.syncYoutubeSource).toHaveBeenCalledOnce());
  expect(api.syncYoutubeSource).toHaveBeenCalledWith(11, {
    metadata: true,
    transcripts: true,
    comments: true,
  });

  await fireEvent.click(screen.getByRole("button", { name: "Select ineligible route source" }));
  await tick();
  const filteredIneligibleSync = screen.getByRole("button", { name: "Run route sync" });
  expect((filteredIneligibleSync as HTMLButtonElement).disabled).toBe(true);
  await fireEvent.click(filteredIneligibleSync);
  expect(api.syncYoutubeSource).toHaveBeenCalledOnce();

  view.unmount();
  await tick();
});

it("refreshes Workspace source content when source sync jobs finish", async () => {
  arrangeCurrentProjectsRoute([projectSourceRecord()]);
  const view = await renderCurrentProjectsRoute("main");
  await waitFor(() => expect(api.listenToSourceJobEvents).toHaveBeenCalledOnce());
  const onSourceJob = api.listenToSourceJobEvents.mock.calls[0]?.[0];
  expect(onSourceJob).toEqual(expect.any(Function));
  expect(api.listProjects).toHaveBeenCalledOnce();
  expect(api.listProjectSources).toHaveBeenCalledOnce();
  expect(api.listProjectSources).toHaveBeenCalledWith(1);

  vi.useFakeTimers();
  onSourceJob({ status: "running" });
  await vi.advanceTimersByTimeAsync(350);
  expect(api.listProjects).toHaveBeenCalledOnce();
  expect(api.listProjectSources).toHaveBeenCalledOnce();

  for (const [index, status] of ["succeeded", "failed", "cancelled"].entries()) {
    const callsBeforeRefresh = index + 1;
    onSourceJob({ status });
    await vi.advanceTimersByTimeAsync(349);
    expect(api.listProjects).toHaveBeenCalledTimes(callsBeforeRefresh);
    expect(api.listProjectSources).toHaveBeenCalledTimes(callsBeforeRefresh);

    await vi.advanceTimersByTimeAsync(1);
    await tick();
    await Promise.resolve();
    expect(api.listProjects).toHaveBeenCalledTimes(callsBeforeRefresh + 1);
    expect(api.listProjectSources).toHaveBeenCalledTimes(callsBeforeRefresh + 1);
    expect(api.listProjectSources).toHaveBeenNthCalledWith(callsBeforeRefresh + 1, 1);
  }

  view.unmount();
  expect(api.unlistenSourceJobs).toHaveBeenCalledOnce();
});

it("renders three-zone projects workspace", () => {
  const project = researchProjectView();
  const view = render(ProjectsShell, {
    props: projectsShellProps({
      state: emptyProjectsState({
        projects: [project],
        selectedProjectId: project.id,
      }),
    }),
  });

  const rail = view.container.querySelector<HTMLElement>('[data-ui-region="project-rail"]');
  const workspace = view.container.querySelector<HTMLElement>('[data-ui-region="project-workspace"]');
  const inspector = view.container.querySelector<HTMLElement>('[data-ui-region="project-inspector"]');

  expect(rail).toBeTruthy();
  expect(workspace).toBeTruthy();
  expect(inspector).toBeTruthy();
  expect(within(rail!).getByRole("listbox", { name: "Research projects" })).toBeTruthy();
  expect(within(workspace!).getByRole("heading", { name: "Smoke project" })).toBeTruthy();
  expect(within(inspector!).getByText("Inspector")).toBeTruthy();
});

it("exposes create/edit/delete and run eligibility UI", async () => {
  const onCreateProject = vi.fn();
  const railView = render(ProjectRail, {
    props: projectRailProps({
      projects: [researchProjectView()],
      onCreateProject,
    }),
  });

  await fireEvent.click(screen.getByRole("button", { name: "Create project" }));
  expect(onCreateProject).toHaveBeenCalledOnce();
  railView.unmount();

  const onEditProject = vi.fn();
  const onDeleteProject = vi.fn();
  render(ProjectInspector, {
    props: projectInspectorProps({
      project: researchProjectView(),
      sources: [
        projectSourceRecord({ provider: "youtube" }),
        projectSourceRecord({ source_id: 12, provider: "telegram", source_subtype: "channel" }),
      ],
      onEditProject,
      onDeleteProject,
    }),
  });

  expect(screen.getByText("Mixed-provider project analysis runs are not supported yet.")).toBeTruthy();
  expect(
    (screen.getByRole("button", { name: "Run project analysis" }) as HTMLButtonElement).disabled,
  ).toBe(true);
  await fireEvent.click(screen.getByRole("button", { name: "Edit selected project" }));
  await fireEvent.click(screen.getByRole("button", { name: "Delete selected project" }));
  expect(onEditProject).toHaveBeenCalledOnce();
  expect(onDeleteProject).toHaveBeenCalledOnce();
});

it("defaults project run dates to all synced history instead of today only", async () => {
  const onSubmit = vi.fn();
  const today = defaultDateOffset(0);
  render(ProjectRunDialog, {
    props: projectRunDialogProps({
      open: true,
      project: researchProjectView(),
      templates: [promptTemplate()],
      onSubmit,
    }),
  });
  await tick();

  expect((screen.getByLabelText("From") as HTMLInputElement).value).toBe("1970-01-01");
  expect((screen.getByLabelText("To") as HTMLInputElement).value).toBe(today);

  await fireEvent.click(screen.getByRole("button", { name: "Run project analysis" }));
  await waitFor(() => expect(onSubmit).toHaveBeenCalledOnce());
  expect(onSubmit).toHaveBeenCalledWith({
    projectId: 1,
    periodFrom: startOfDayUnix("1970-01-01"),
    periodTo: endOfDayUnix(today),
    outputLanguage: "en",
    promptTemplateId: 5,
    modelOverride: null,
    profileId: null,
    youtubeCorpusMode: "transcript_description",
    includeMigratedHistory: false,
  });
});

it("shows project runs in the central Runs tab", async () => {
  const onRefreshProjectRuns = vi.fn();
  render(ProjectWorkspace, {
    props: projectWorkspaceProps({
      project: researchProjectView(),
      runs: [analysisRun()],
      onRefreshProjectRuns,
    }),
  });

  await fireEvent.click(screen.getByRole("tab", { name: "Runs" }));
  const runsRegion = screen.getByRole("region", { name: "Project analysis runs" });
  const reportLink = within(runsRegion).getByRole("link", { name: "Open report for run 71" });
  expect(reportLink.getAttribute("href")).toBe("/analysis?runId=71");
  expect(within(runsRegion).getByText(/Evidence brief v2/)).toBeTruthy();

  await fireEvent.click(
    within(runsRegion).getByRole("button", { name: "Refresh project analysis runs" }),
  );
  expect(onRefreshProjectRuns).toHaveBeenCalledOnce();
});

it("keeps prompt-pack run details in the Runs tab instead of duplicating them in the inspector", async () => {
  api.listPromptPackRuns.mockResolvedValue([promptPackRun()]);
  const workspaceView = render(ProjectWorkspace, {
    props: projectWorkspaceProps({
      project: researchProjectView(),
      runs: [analysisRun()],
    }),
  });

  await fireEvent.click(screen.getByRole("tab", { name: "Runs" }));
  await waitFor(() => expect(screen.getByText("Run #91")).toBeTruthy());
  expect(screen.getByRole("region", { name: "Prompt Pack runs" })).toBeTruthy();
  workspaceView.unmount();

  render(ProjectInspector, {
    props: projectInspectorProps({
      project: researchProjectView(),
      sources: [projectSourceRecord()],
      runs: [analysisRun()],
    }),
  });
  expect(screen.queryByRole("region", { name: "Prompt Pack runs" })).toBeNull();
  expect(screen.queryByText("Run #91")).toBeNull();
});

it("matches the Library type column in Workspace project sources", () => {
  const columns = projectSourceGridColumns(undefined);

  expect(columns.map((column) => column.id)).toEqual([
    "title",
    "typeLabel",
    "localCopyLabel",
    "addedAt",
  ]);
  expect(columns.map((column) => column.header)).toEqual([
    "Title",
    "Type",
    "Details",
    "Added to project at",
  ]);
  expect(columns.some((column) => column.id === "provider" || column.id === "subtype")).toBe(false);
});

it("shows full source type labels when connecting sources from Library", async () => {
  render(ConnectFromLibrary, {
    props: connectFromLibraryProps({
      open: true,
      project: researchProjectView(),
      librarySources: [librarySourceView({ typeLabel: "YouTube / Playlist" })],
    }),
  });

  const gridHost = screen.getByRole("region", { name: "Library sources available to connect" });
  await waitFor(() => expect(within(gridHost).getByText("YouTube / Playlist")).toBeTruthy());
  expect(within(gridHost).queryByText(/^youtube$/i)).toBeNull();
});

it("wires the project Add source dialog through the current ProjectsShell", async () => {
  const onConnectExistingProjectSource = vi.fn();
  const onConnectAddedProjectSource = vi.fn();
  const onConnectAddedProjectSources = vi.fn();
  const project = researchProjectView();
  render(ProjectsShell, {
    props: projectsShellProps({
      state: emptyProjectsState({
        projectsRaw: [projectRecord()],
        projects: [project],
        selectedProjectId: project.id,
        libraryCatalogRecords: [playlistCatalogRecord()],
      }),
      onConnectExistingProjectSource,
      onConnectAddedProjectSource,
      onConnectAddedProjectSources,
    }),
  });

  await openProjectAddSourceDialog();
  await addNewVideoThroughOpenDialog({
    sourceId: 21,
    title: "New shell video",
    url: "https://www.youtube.com/watch?v=new-shell-video",
  });
  await waitFor(() => {
    expect(onConnectAddedProjectSource).toHaveBeenCalledOnce();
    expect(onConnectAddedProjectSource).toHaveBeenCalledWith(21);
  });
  expect(onConnectExistingProjectSource).not.toHaveBeenCalled();

  api.addYoutubeSource.mockClear();
  await addPlaylistVideosThroughOpenDialog([22, 23]);
  await waitFor(() => {
    expect(onConnectAddedProjectSources).toHaveBeenCalledOnce();
    expect(onConnectAddedProjectSources).toHaveBeenCalledWith([22, 23]);
  });
  expect(onConnectExistingProjectSource).not.toHaveBeenCalled();
});

it("keeps top command actions honest while project export is out of scope", async () => {
  const onRunProject = vi.fn();
  render(TopCommandBar, {
    props: topCommandBarProps({
      project: researchProjectView(),
      sources: [projectSourceRecord()],
      onRunProject,
    }),
  });

  const run = screen.getByRole("button", { name: "Run project analysis" });
  expect((run as HTMLButtonElement).disabled).toBe(false);
  await fireEvent.click(run);
  expect(onRunProject).toHaveBeenCalledOnce();

  const exportAction = screen.getByRole("button", {
    name: "Project export is not available yet.",
  });
  expect((exportAction as HTMLButtonElement).disabled).toBe(true);
  expect(exportAction.getAttribute("title")).toBe("Project export is not available yet.");
  expect(exportAction.getAttribute("data-disabled-reason")).toBe(
    "Project export is not available yet.",
  );
});

it("keeps project action hierarchy consistent across Workspace, Projects, and Runs", async () => {
  const inspectorView = render(ProjectInspector, {
    props: projectInspectorProps({
      project: researchProjectView(),
      sources: [projectSourceRecord()],
      selectedSource: projectSourceLink(),
    }),
  });
  expect(screen.getByRole("button", { name: "Delete selected project" }).className).toContain(
    "bg-destructive/10",
  );
  expect(
    screen.getByRole("button", { name: "Remove source Evidence video from project" }).className,
  ).toContain("bg-destructive/10");
  inspectorView.unmount();

  api.listPromptPackRuns.mockResolvedValue([promptPackRun()]);
  const runsView = render(ProjectRunsScreen, { props: projectRunsScreenProps() });
  const deleteRun = await screen.findByRole("button", {
    name: "Delete selected prompt pack run 91",
  });
  expect(deleteRun.className).toContain("bg-destructive/10");
  runsView.unmount();

  const project = researchProjectView();
  render(ProjectsShell, {
    props: projectsShellProps({
      showRail: false,
      state: emptyProjectsState({
        projects: [project],
        selectedProjectId: project.id,
      }),
    }),
  });
  expect(screen.getByRole("button", { name: "Edit project Smoke project" })).toBeTruthy();
  expect(screen.getByRole("button", { name: "Delete project Smoke project" })).toBeTruthy();
});

it("keeps project navigation rows visually neutral until selected", async () => {
  const onSelectProject = vi.fn();
  render(ProjectRail, {
    props: projectRailProps({
      projects: [
        researchProjectView({ id: "project:1", projectId: 1, title: "Selected project" }),
        researchProjectView({ id: "project:2", projectId: 2, title: "Neutral project" }),
      ],
      selectedProjectId: "project:1",
      onSelectProject,
    }),
  });

  const selected = screen.getByRole("button", { name: /Selected project/ });
  const neutral = screen.getByRole("button", { name: /Neutral project/ });
  expect(selected.classList.contains("extractum-project-row")).toBe(true);
  expect(selected.getAttribute("data-selected")).toBe("true");
  expect(selected.classList.contains("is-selected")).toBe(true);
  expect(neutral.classList.contains("extractum-project-row")).toBe(true);
  expect(neutral.getAttribute("data-selected")).toBe("false");
  expect(neutral.classList.contains("is-selected")).toBe(false);
  expect(neutral.hasAttribute("aria-current")).toBe(false);

  await fireEvent.click(neutral);
  expect(onSelectProject).toHaveBeenCalledOnce();
  expect(onSelectProject).toHaveBeenCalledWith("project:2");
});

it("labels project data grids for assistive technology", async () => {
  const columns = projectSourceGridColumns(undefined);
  expect(columns.find((column) => column.id === "title")?.header).toBe("Title");

  const workspaceView = render(ProjectWorkspace, {
    props: projectWorkspaceProps({
      project: researchProjectView(),
      projectSourceLinks: [projectSourceLink()],
      librarySources: [librarySourceView()],
    }),
  });
  expect(screen.getByRole("region", { name: "Project sources" })).toBeTruthy();
  workspaceView.unmount();

  const connectView = render(ConnectFromLibrary, {
    props: connectFromLibraryProps({
      open: true,
      project: researchProjectView(),
      librarySources: [librarySourceView()],
    }),
  });
  expect(
    screen.getByRole("region", { name: "Library sources available to connect" }),
  ).toBeTruthy();
  connectView.unmount();

  render(ProjectRunsScreen, { props: projectRunsScreenProps() });
  expect(screen.getByRole("region", { name: "Prompt Pack runs grid" })).toBeTruthy();
  const promptPackRegions = screen.getAllByRole("region", { name: "Prompt Pack runs" });
  expect(promptPackRegions.some((region) => region.classList.contains("project-runs-screen"))).toBe(
    true,
  );
  expect(promptPackRegions.some((region) => region.classList.contains("extractum-data-grid"))).toBe(
    true,
  );
});

it("scopes repeated project refresh controls", async () => {
  const onRefreshProjectRuns = vi.fn();
  render(ProjectRunsTab, {
    props: projectRunsTabProps({
      projectId: 1,
      onRefreshProjectRuns,
    }),
  });
  await waitFor(() => expect(api.listPromptPackRuns).toHaveBeenCalledOnce());
  api.listPromptPackRuns.mockClear();
  api.listActivePromptPackRuns.mockClear();

  await fireEvent.click(
    screen.getByRole("button", { name: "Refresh project analysis runs" }),
  );
  expect(onRefreshProjectRuns).toHaveBeenCalledOnce();

  await fireEvent.click(screen.getByRole("button", { name: "Refresh prompt pack runs" }));
  await waitFor(() => {
    expect(api.listPromptPackRuns).toHaveBeenCalledWith({ projectId: 1, limit: 20 });
    expect(api.listActivePromptPackRuns).toHaveBeenCalledOnce();
  });
});

it("clarifies the Workspace Runs taxonomy", async () => {
  const runsView = render(ProjectRunsTab, {
    props: projectRunsTabProps({ projectId: 1 }),
  });
  const analysisRuns = screen.getByRole("region", { name: "Project analysis runs" });
  expect(within(analysisRuns).getByText("Project analysis runs")).toBeTruthy();
  expect(within(analysisRuns).getByText("No project analysis runs yet.")).toBeTruthy();
  const promptPackRuns = within(analysisRuns).getByRole("region", { name: "Prompt Pack runs" });
  expect(within(promptPackRuns).getByText("Prompt Pack runs")).toBeTruthy();
  expect(within(promptPackRuns).getByText("No prompt pack runs yet.")).toBeTruthy();
  runsView.unmount();

  render(ProjectInspector, { props: projectInspectorProps() });
  expect(screen.getByRole("heading", { name: "Recent project analysis runs" })).toBeTruthy();
  expect(screen.getByText("No project analysis runs")).toBeTruthy();
  expect(screen.queryByRole("heading", { name: "Recent project runs" })).toBeNull();
});
