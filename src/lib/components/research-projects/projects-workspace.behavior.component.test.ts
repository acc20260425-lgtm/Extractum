// @vitest-environment jsdom
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/svelte";
import { tick, type ComponentProps } from "svelte";
import type { AnalysisRunSummary } from "$lib/types/analysis";
import type { ProjectRecord, ProjectSourceRecord } from "$lib/types/projects";
import type { ResearchProjectsWorkflowState } from "$lib/ui/research-projects-workflow";
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
  cancelPromptPackRun: vi.fn(),
  createProject: vi.fn(),
  deleteAnalysisRun: vi.fn(),
  deleteProject: vi.fn(),
  deleteProjectYoutubeVideoSourceFromLibrary: vi.fn(),
  deletePromptPackRun: vi.fn(),
  getProjectDataRange: vi.fn(),
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
  listenToAnalysisRunEvents: vi.fn(),
  listenToPromptPackRunEvents: vi.fn(),
  listenToSourceJobEvents: vi.fn(),
  openUrl: vi.fn(),
  removeProjectSources: vi.fn(),
  startProjectAnalysis: vi.fn(),
  setProjectArchived: vi.fn(),
  setProjectPinned: vi.fn(),
  syncYoutubeSource: vi.fn(),
  unlistenAnalysisRuns: vi.fn(),
  unlistenPromptPackRuns: vi.fn(),
  unlistenSourceJobs: vi.fn(),
  updateProject: vi.fn(),
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

vi.mock("$lib/api/source-jobs", () => ({
  listSourceJobs: api.listSourceJobs,
  listenToSourceJobEvents: api.listenToSourceJobEvents,
  syncYoutubeSource: api.syncYoutubeSource,
}));

vi.mock("$lib/api/analysis-runs", () => ({
  deleteAnalysisRun: api.deleteAnalysisRun,
  listenToAnalysisRunEvents: api.listenToAnalysisRunEvents,
}));

vi.mock("$lib/api/analysis-source-groups", () => ({
  listAnalysisPromptTemplates: api.listAnalysisPromptTemplates,
}));

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

function projectSource(overrides: Partial<ProjectSourceRecord> = {}): ProjectSourceRecord {
  return {
    project_id: 1,
    source_id: 10,
    provider: "youtube",
    source_subtype: "video",
    title: "Smoke source",
    subtitle: null,
    item_count: 1,
    added_at: 1,
    last_synced_at: null,
    sync_status: "active",
    handle: "smoke-video",
    ...overrides,
  };
}

function analysisRun(overrides: Partial<AnalysisRunSummary> = {}): AnalysisRunSummary {
  return {
    id: 1,
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
    period_to: 1,
    output_language: "en",
    prompt_template_id: 1,
    prompt_template_name: "Smoke template",
    prompt_template_version: 1,
    provider_profile: "default",
    provider: "openai",
    model: "smoke-model",
    youtube_corpus_mode: "transcript_description",
    telegram_history_scope: "current",
    status: "completed",
    error: null,
    has_trace_data: false,
    snapshot_state: null,
    snapshot_captured_at: null,
    snapshot_error: null,
    created_at: 1,
    completed_at: null,
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

function projectRunsScreenProps(
  overrides: Partial<ProjectRunsScreenProps> = {},
): ProjectRunsScreenProps {
  return { ...overrides };
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

beforeEach(() => {
  class ResizeObserverStub {
    observe() {}
    unobserve() {}
    disconnect() {}
  }

  vi.stubGlobal("ResizeObserver", ResizeObserverStub);
  api.addProjectSources.mockResolvedValue({ added_count: 0, already_present_count: 0 });
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
  api.getLlmProfiles.mockResolvedValue({ active_profile: "", profiles: [] });
  api.listActivePromptPackRuns.mockResolvedValue([]);
  api.listAnalysisPromptTemplates.mockResolvedValue([]);
  api.listLibraryCatalog.mockResolvedValue({ sources: [], filter_counts: [] });
  api.listProjectRuns.mockResolvedValue([analysisRun()]);
  api.listProjectSources.mockResolvedValue([projectSource()]);
  api.listProjects.mockResolvedValue([]);
  api.listResearchProjects.mockResolvedValue([]);
  api.listPromptPackRuns.mockResolvedValue([]);
  api.listSourceJobs.mockResolvedValue([]);
  api.listenToAnalysisRunEvents.mockResolvedValue(api.unlistenAnalysisRuns);
  api.listenToPromptPackRunEvents.mockResolvedValue(api.unlistenPromptPackRuns);
  api.listenToSourceJobEvents.mockResolvedValue(api.unlistenSourceJobs);
  api.openUrl.mockResolvedValue(undefined);
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

  await waitFor(() => expect(api.listProjects).toHaveBeenCalledOnce());
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

  await waitFor(() => expect(api.listProjects).toHaveBeenCalledOnce());
  await waitFor(() => expect(api.listenToAnalysisRunEvents).toHaveBeenCalledOnce());
  await waitFor(() => expect(api.listenToSourceJobEvents).toHaveBeenCalledOnce());

  expect(view.container.querySelector('[data-ui-route="research-projects"]')).toBeTruthy();

  view.unmount();

  expect(api.unlistenAnalysisRuns).toHaveBeenCalledOnce();
  expect(api.unlistenSourceJobs).toHaveBeenCalledOnce();
});

it("smoke renders next Projects route", async () => {
  const { default: ProjectsNextPage } = await import("../../../routes/projects/next/+page.svelte");
  const view = render(ProjectsNextPage);

  await waitFor(() => expect(api.listResearchProjects).toHaveBeenCalledOnce());

  expect(screen.getByRole("main")).toBeTruthy();

  view.unmount();
  await tick();
});
