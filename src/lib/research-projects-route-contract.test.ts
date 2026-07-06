import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import layoutSource from "../routes/+layout.svelte?raw";
import pageSource from "../routes/projects/+page.svelte?raw";
import dataGridSource from "$lib/components/extractum-ui/DataGrid.svelte?raw";
import shellSource from "$lib/components/research-projects/ProjectsShell.svelte?raw";
import railSource from "$lib/components/research-projects/ProjectRail.svelte?raw";
import inspectorSource from "$lib/components/research-projects/ProjectInspector.svelte?raw";
import runsTabSource from "$lib/components/research-projects/ProjectRunsTab.svelte?raw";
import runsScreenSource from "$lib/components/research-projects/ProjectRunsScreen.svelte?raw";
import runDialogSource from "$lib/components/research-projects/ProjectRunDialog.svelte?raw";
import sourcesTabSource from "$lib/components/research-projects/SourcesTab.svelte?raw";
import connectFromLibrarySource from "$lib/components/research-projects/ConnectFromLibrary.svelte?raw";
import projectSourceSummarySource from "$lib/components/research-projects/ProjectSourceSummary.svelte?raw";
import topCommandBarSource from "$lib/components/research-projects/TopCommandBar.svelte?raw";
import workspaceSource from "$lib/components/research-projects/ProjectWorkspace.svelte?raw";
import youtubeSummaryRunsPanelSource from "$lib/components/research-projects/YoutubeSummaryRunsPanel.svelte?raw";

const baseStylesSource = readFileSync(resolve(process.cwd(), "src/lib/styles/base.css"), "utf8");

describe("projects mvp route contract", () => {
  it("uses real project APIs instead of analysis source group APIs", () => {
    expect(pageSource).toContain("listProjects");
    expect(pageSource).toContain("listProjectSources");
    expect(pageSource).toContain("listLibraryCatalog");
    expect(pageSource).not.toContain("listLibrarySources");
    expect(pageSource).toContain("listenToAnalysisRunEvents");
    expect(pageSource).toContain("formatAppError");
    expect(pageSource).not.toContain("String(error)");
    expect(pageSource).not.toContain("listAnalysisSourceGroups");
    expect(pageSource).not.toContain("updateAnalysisSourceGroup");
  });

  it("renders three-zone projects workspace", () => {
    expect(shellSource).toContain('data-ui-region="project-rail"');
    expect(shellSource).toContain('data-ui-region="project-workspace"');
    expect(shellSource).toContain('data-ui-region="project-inspector"');
  });

  it("exposes create/edit/delete and run eligibility UI", () => {
    expect(railSource).toContain("Create project");
    expect(inspectorSource).toContain("Run project analysis");
    expect(inspectorSource).toContain("Mixed-provider project analysis runs are not supported yet.");
  });

  it("defaults project run dates to all synced history instead of today only", () => {
    expect(runDialogSource).toContain('PROJECT_RUN_DEFAULT_FROM_DATE = "1970-01-01"');
    expect(runDialogSource).toContain("defaultDateOffset(0)");
    expect(runDialogSource).not.toContain("new Date().toISOString().slice(0, 10)");
  });

  it("shows project runs in the central Runs tab", () => {
    expect(shellSource).toContain("runs={currentRuns}");
    expect(shellSource).toContain("onRefreshProjectRuns={onRefreshProjectRuns}");
    expect(workspaceSource).toContain("ProjectRunsTab");
    expect(workspaceSource).not.toContain("Runs will surface active LLM jobs");
    expect(runsTabSource).toContain("analysisRunHref");
    expect(runsTabSource).toContain("`/analysis?runId=${run.id}`");
    expect(runsTabSource).toContain("Open report");
    expect(runsTabSource).toContain("formatPeriod");
  });

  it("keeps prompt-pack run details in the Runs tab instead of duplicating them in the inspector", () => {
    expect(runsTabSource).toContain("YoutubeSummaryRunsPanel");
    expect(inspectorSource).not.toContain("YoutubeSummaryRunsPanel");
  });

  it("matches the Library type column in Workspace project sources", () => {
    expect(workspaceSource).toContain("SourcesTab");
    expect(sourcesTabSource).toContain('id: "typeLabel", header: "Type"');
    expect(sourcesTabSource).not.toContain('header: "Provider"');
    expect(sourcesTabSource).not.toContain('header: "Subtype"');
  });

  it("shows full source type labels when connecting sources from Library", () => {
    expect(connectFromLibrarySource).toContain('id: "typeLabel", header: "Тип"');
    expect(connectFromLibrarySource).not.toContain('id: "provider", header: "Тип"');
  });

  it("places Add source immediately before Connect from Library in the project sources toolbar", () => {
    expect(sourcesTabSource).toContain("onOpenAddSource");
    expect(sourcesTabSource).toContain('data-ui-action="add-source"');
    expect(sourcesTabSource).toContain('data-ui-action="connect-library"');
    expect(sourcesTabSource.indexOf('data-ui-action="add-source"')).toBeLessThan(
      sourcesTabSource.indexOf('data-ui-action="connect-library"'),
    );
    expect(sourcesTabSource).toContain("Plus");
    expect(sourcesTabSource).toContain("Add source");
  });

  it("wires the project Add source dialog through the current ProjectsShell", () => {
    expect(workspaceSource).toContain("onOpenAddSource");
    expect(shellSource).toContain("LibraryAddSourceDialog");
    expect(shellSource).toContain("addSourceOpen");
    expect(shellSource).toContain("projectAddSourceContext");
    expect(shellSource).toContain("let connectedSourceIds = $derived(");
    expect(shellSource).toContain("connectedSourceIdsForProject(workflowState.projectSources");
    expect(shellSource).toContain("let projectAddSourceContext = $derived<ProjectAddSourceContext | undefined>");
    expect(shellSource).toContain("buildLibraryCatalogSourcesView");
    expect(shellSource).toContain("connectedSourceIdsForProject");
    expect(shellSource).toContain("onConnectAddedProjectSource");
    expect(shellSource).toContain("onConnectAddedProjectSources");
    expect(shellSource).toContain("onConnectExistingProjectSource");
  });

  it("passes project add-source workflow callbacks from both current project routes", () => {
    expect(pageSource).toContain("onConnectAddedProjectSource={workflow.connectAddedProjectSource}");
    expect(pageSource).toContain("onConnectAddedProjectSources={workflow.connectAddedProjectSources}");
    expect(pageSource).toContain("onConnectExistingProjectSource={workflow.connectExistingProjectSource}");
    expect(pageSource).toContain("onSetStatus={workflow.setStatus}");
    expect(pageSource).not.toContain("onSourcesChanged={(ids)");

    const listPageSource = readFileSync(resolve(process.cwd(), "src/routes/projects/list/+page.svelte"), "utf8");
    expect(listPageSource).toContain("onConnectAddedProjectSource={workflow.connectAddedProjectSource}");
    expect(listPageSource).toContain("onConnectAddedProjectSources={workflow.connectAddedProjectSources}");
    expect(listPageSource).toContain("onConnectExistingProjectSource={workflow.connectExistingProjectSource}");
    expect(listPageSource).toContain("onSetStatus={workflow.setStatus}");
  });

  it("wires the project Add source dialog through the next Projects route", () => {
    const nextPageSource = readFileSync(resolve(process.cwd(), "src/routes/projects/next/+page.svelte"), "utf8");
    expect(nextPageSource).toContain("LibraryAddSourceDialog");
    expect(nextPageSource).toContain("connectAddedProjectSource");
    expect(nextPageSource).toContain("connectAddedProjectSources");
    expect(nextPageSource).toContain("connectExistingProjectSource");
    expect(nextPageSource).toContain("projectAddSourceContext");
    expect(nextPageSource).toContain("let connectedSourceIds = $derived(connectedSourceIdsForProject(sources, selectedProjectId))");
    expect(nextPageSource).toContain("formatError: formatAppError");
    expect(nextPageSource).toContain("onConnectFromLibrary: () => void openConnectSources()");
    expect(nextPageSource).toContain("onAddSource: () => (addSourceOpen = true)");
  });

  it("wires selected Workspace source syncs to the YouTube source job command", () => {
    expect(pageSource).toContain("syncYoutubeSource");
    expect(pageSource).toContain("syncYoutubeSource,");
    expect(shellSource).toContain("onSyncSelectedSources");
    expect(workspaceSource).toContain("onSyncSelectedSources");
    expect(sourcesTabSource).toContain("selectedProjectSourcesSyncDisabledReason");
    expect(sourcesTabSource).toContain("handleSyncSelected");
    expect(sourcesTabSource).toContain("handleSyncAll");
    expect(sourcesTabSource).toContain("syncAllDisabledReason");
    expect(sourcesTabSource).toContain("onclick={handleSyncAll}");
    expect(sourcesTabSource).not.toContain('disabled={true} title="Sync selected sources (not implemented)"');
    expect(sourcesTabSource).not.toContain("Sync all sources (not implemented)");
    const nextPageSource = readFileSync(resolve(process.cwd(), "src/routes/projects/next/+page.svelte"), "utf8");
    expect(nextPageSource).not.toContain("comments: false");
  });

  it("refreshes Workspace source content when source sync jobs finish", () => {
    expect(pageSource).toContain("listenToSourceJobEvents");
    expect(pageSource).toContain("shouldRefreshForSourceJobStatus");
    expect(pageSource).toContain('status === "succeeded"');
    expect(pageSource).toContain('status === "failed"');
    expect(pageSource).toContain('status === "cancelled"');
  });

  it("keeps top command actions honest while project export is out of scope", () => {
    expect(shellSource).toContain("sources={currentProjectSources}");
    expect(shellSource).toContain("onRunProject={() => (runOpen = true)}");
    expect(topCommandBarSource).toContain("projectRunDisabledReason(project, sources)");
    expect(topCommandBarSource).toContain("disabled={loading || runDisabledReason !== null}");
    expect(topCommandBarSource).toContain("onclick={onRunProject}");
    expect(topCommandBarSource).toContain("PROJECT_EXPORT_DISABLED_REASON");
    expect(topCommandBarSource).toContain("disabled={true}");
    expect(topCommandBarSource).toContain("title={PROJECT_EXPORT_DISABLED_REASON}");
  });

  it("keeps project action hierarchy consistent across Workspace, Projects, and Runs", () => {
    expect(layoutSource).toContain(":global(button:not([data-slot=\"button\"]))");
    expect(inspectorSource).toMatch(/variant="destructive"[\s\S]*Delete project/);
    expect(inspectorSource).toMatch(/variant="destructive"[\s\S]*Remove from project/);
    expect(runsScreenSource).toMatch(/variant="destructive"[\s\S]*Delete/);
    expect(shellSource).toContain("aria-label={`Edit project ${project.title}`}");
    expect(shellSource).toContain("aria-label={`Delete project ${project.title}`}");
  });

  it("keeps project navigation rows visually neutral until selected", () => {
    expect(railSource).toContain("data-selected={project.id === selectedProjectId}");
    expect(railSource).toContain("extractum-project-row");
    expect(baseStylesSource).toContain(".extractum-project-row:not(.is-selected):hover");
    expect(baseStylesSource).toContain("box-shadow: inset 3px 0 0 var(--extractum-primary)");
  });

  it("shares the project row selected-state recipe across project navigation variants", () => {
    expect(baseStylesSource).toContain(".extractum-project-row");
    expect(baseStylesSource).toContain(".extractum-project-row.is-selected");
    expect(railSource).toContain("extractum-project-row");
    expect(railSource).toContain('aria-current={project.id === selectedProjectId ? "page" : undefined}');
    expect(shellSource).toContain("tree-project-item-row extractum-project-row");
    expect(shellSource).toContain("data-selected={workflowState.selectedProjectId === project.id}");
    expect(shellSource).toContain("class:is-selected={workflowState.selectedProjectId === project.id}");
    expect(shellSource).toContain('aria-current={workflowState.selectedProjectId === project.id ? "page" : undefined}');
  });

  it("labels project data grids for assistive technology", () => {
    expect(dataGridSource).toContain("ariaLabel");
    expect(dataGridSource).toContain("aria-label={ariaLabel}");
    expect(sourcesTabSource).toContain('{ id: "title", header: "Title", width: 260, flexgrow: 1');
    expect(sourcesTabSource).toContain('ariaLabel="Project sources"');
    expect(connectFromLibrarySource).toContain('ariaLabel="Library sources available to connect"');
    expect(runsScreenSource).toContain('<section class="project-runs-screen" aria-label="Prompt Pack runs">');
    expect(runsScreenSource).toContain('<section class="runs-grid-panel" aria-label="Prompt Pack runs grid">');
    expect(runsScreenSource).toContain('ariaLabel="Prompt Pack runs"');
    expect(runsScreenSource).toContain('overlay={loading ? "Loading Prompt Pack runs..." : "No Prompt Pack runs yet."}');
  });

  it("scopes repeated project refresh controls", () => {
    expect(runsTabSource).toContain('aria-label="Refresh project analysis runs"');
    expect(youtubeSummaryRunsPanelSource).toContain('aria-label="Refresh prompt pack runs"');
  });

  it("clarifies the Workspace Runs taxonomy", () => {
    expect(runsTabSource).toContain('<section class="project-runs-tab" aria-label="Project analysis runs">');
    expect(runsTabSource).toContain("<span>Project analysis runs</span>");
    expect(runsTabSource).toContain("No project analysis runs yet.");
    expect(inspectorSource).toContain("Recent project analysis runs");
    expect(inspectorSource).toContain("No project analysis runs");
    expect(inspectorSource).not.toContain("Recent project runs");
    expect(runsTabSource).toContain("YoutubeSummaryRunsPanel");
    expect(youtubeSummaryRunsPanelSource).toContain('aria-label="Prompt Pack runs"');
  });

  it("uses shared density primitives for repeated New Projects shells", () => {
    expect(baseStylesSource).toContain(".extractum-panel-shell");
    expect(baseStylesSource).toContain(".extractum-grid-frame");
    expect(baseStylesSource).toContain(".extractum-toolbar-row");
    expect(baseStylesSource).toContain(".extractum-stat-card");
    expect(inspectorSource).toContain('class="extractum-panel-shell"');
    expect(sourcesTabSource).toContain('class="sources-grid-region extractum-grid-frame"');
    expect(projectSourceSummarySource).toContain('class="extractum-stat-card"');
    expect(runsTabSource).toContain('class="runs-toolbar extractum-toolbar-row"');
    expect(youtubeSummaryRunsPanelSource).toContain('class="runs-toolbar extractum-toolbar-row"');
  });
});
