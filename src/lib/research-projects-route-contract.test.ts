import { describe, expect, it } from "vitest";
import pageSource from "../routes/projects/+page.svelte?raw";
import shellSource from "$lib/components/research-projects/ProjectsShell.svelte?raw";
import railSource from "$lib/components/research-projects/ProjectRail.svelte?raw";
import inspectorSource from "$lib/components/research-projects/ProjectInspector.svelte?raw";
import runsTabSource from "$lib/components/research-projects/ProjectRunsTab.svelte?raw";
import runDialogSource from "$lib/components/research-projects/ProjectRunDialog.svelte?raw";
import sourcesTabSource from "$lib/components/research-projects/SourcesTab.svelte?raw";
import connectFromLibrarySource from "$lib/components/research-projects/ConnectFromLibrary.svelte?raw";
import topCommandBarSource from "$lib/components/research-projects/TopCommandBar.svelte?raw";
import workspaceSource from "$lib/components/research-projects/ProjectWorkspace.svelte?raw";

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
    expect(inspectorSource).toContain("Mixed-provider project runs are not supported yet.");
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

  it("wires selected Workspace source syncs to the YouTube source job command", () => {
    expect(pageSource).toContain("syncYoutubeSource");
    expect(pageSource).toContain("syncYoutubeSource,");
    expect(shellSource).toContain("onSyncSelectedSources");
    expect(workspaceSource).toContain("onSyncSelectedSources");
    expect(sourcesTabSource).toContain("selectedProjectSourcesSyncDisabledReason");
    expect(sourcesTabSource).toContain("handleSyncSelected");
    expect(sourcesTabSource).not.toContain('disabled={true} title="Sync selected sources (not implemented)"');
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
});
