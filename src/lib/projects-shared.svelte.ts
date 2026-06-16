import { browser } from "$app/environment";
import { listProjects, listProjectSources } from "$lib/api/projects";
import { buildResearchProjectsView, type ResearchProjectView } from "$lib/ui/research-projects-model";

let projectsList = $state<ResearchProjectView[]>([]);
let activeProjectId = $state<string | null>(null);
let initialized = $state(false);
let showCreateDialog = $state(false);

export const projectsSharedState = {
  get projects() {
    return projectsList;
  },
  set projects(value: ResearchProjectView[]) {
    projectsList = value;
  },
  get selectedProjectId() {
    return activeProjectId;
  },
  set selectedProjectId(value: string | null) {
    activeProjectId = value;
  },
  get initialized() {
    return initialized;
  },
  set initialized(value: boolean) {
    initialized = value;
  },
  get showCreateDialog() {
    return showCreateDialog;
  },
  set showCreateDialog(value: boolean) {
    showCreateDialog = value;
  },

  async load() {
    if (!browser) return;
    try {
      const projectsRaw = await listProjects();
      const allProjectSources = (
        await Promise.all(projectsRaw.map((project) => listProjectSources(project.id)))
      ).flat();
      projectsList = buildResearchProjectsView(projectsRaw, allProjectSources, []);
      initialized = true;

      // Select first project if none is active
      if (!activeProjectId && projectsList.length > 0) {
        activeProjectId = projectsList[0].id;
      }
    } catch (err) {
      console.error("Failed to load projects in projectsSharedState:", err);
    }
  }
};
