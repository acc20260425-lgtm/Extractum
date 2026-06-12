import { describe, expect, it } from "vitest";
import projectsRouteSource from "../routes/projects/+page.svelte?raw";
import shellSource from "./components/research-projects/ProjectsShell.svelte?raw";
import projectRailSource from "./components/research-projects/ProjectRail.svelte?raw";
import workspaceSource from "./components/research-projects/ProjectWorkspace.svelte?raw";
import sourcesTabSource from "./components/research-projects/SourcesTab.svelte?raw";

describe("research projects route contract", () => {
  it("adds the new route without redirecting through the old analysis workspace", () => {
    expect(projectsRouteSource).toContain('data-ui-route="research-projects"');
    expect(projectsRouteSource).toContain("createResearchProjectsWorkflow");
    expect(projectsRouteSource).toContain("listAnalysisSourceGroups");
    expect(projectsRouteSource).toContain("listAnalysisSources");
    expect(projectsRouteSource).toContain("listSourceJobs");
    expect(projectsRouteSource).not.toContain('goto("/analysis")');
  });

  it("renders the dense project control deck regions", () => {
    expect(shellSource).toContain('data-ui-region="icon-rail"');
    expect(shellSource).toContain('data-ui-region="project-rail"');
    expect(shellSource).toContain('data-ui-region="top-command-bar"');
    expect(shellSource).toContain('data-ui-region="project-workspace"');
    expect(shellSource).toContain("grid-template-columns: 56px 260px minmax(0, 1fr)");
  });

  it("keeps project rail and workspace in product language", () => {
    expect(projectRailSource).toContain("Проекты");
    expect(projectRailSource).not.toContain("source group");
    expect(workspaceSource).toContain("Overview");
    expect(workspaceSource).toContain("Sources");
    expect(workspaceSource).toContain("Evidence");
  });

  it("uses SVAR-backed product grid for project sources", () => {
    expect(sourcesTabSource).toContain("ExtractumDataGrid");
    expect(sourcesTabSource).toContain("ProviderBadge");
    expect(sourcesTabSource).toContain("StatusBadge");
    expect(sourcesTabSource).toContain('data-ui-action="connect-library"');
    expect(sourcesTabSource).not.toContain("@svar-ui/");
  });
});
