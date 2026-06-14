import { describe, expect, it } from "vitest";
import pageSource from "../routes/projects/+page.svelte?raw";
import shellSource from "$lib/components/research-projects/ProjectsShell.svelte?raw";
import railSource from "$lib/components/research-projects/ProjectRail.svelte?raw";
import inspectorSource from "$lib/components/research-projects/ProjectInspector.svelte?raw";
import topCommandBarSource from "$lib/components/research-projects/TopCommandBar.svelte?raw";

describe("projects mvp route contract", () => {
  it("uses real project APIs instead of analysis source group APIs", () => {
    expect(pageSource).toContain("listProjects");
    expect(pageSource).toContain("listProjectSources");
    expect(pageSource).toContain("listLibrarySources");
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
