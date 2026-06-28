import type {
  ProjectArchivedInput,
  ProjectDataRange,
  ProjectDataRangeInput,
  ProjectPinnedInput,
  ProjectSummary,
} from "$lib/types/projects";

export interface ProjectRailState {
  summaries: ProjectSummary[];
  dataRange: ProjectDataRange | null;
  saving: boolean;
  status: string;
}

export interface ProjectRailWorkflowDeps {
  getState(): ProjectRailState;
  patch(patch: Partial<ProjectRailState>): void;
  listResearchProjects(): Promise<ProjectSummary[]>;
  setProjectPinned(input: ProjectPinnedInput): Promise<void>;
  setProjectArchived(input: ProjectArchivedInput): Promise<void>;
  getProjectDataRange(input: ProjectDataRangeInput): Promise<ProjectDataRange>;
  formatError(action: string, error: unknown): string;
}

export function createProjectRailWorkflow(deps: ProjectRailWorkflowDeps) {
  async function reload() {
    const summaries = await deps.listResearchProjects();
    deps.patch({ summaries });
  }

  async function setPinned(projectId: number, pinned: boolean) {
    deps.patch({ saving: true });
    try {
      await deps.setProjectPinned({ projectId, pinned });
      await reload();
    } catch (error) {
      deps.patch({
        status: deps.formatError(pinned ? "pinning project" : "unpinning project", error),
      });
    } finally {
      deps.patch({ saving: false });
    }
  }

  async function setArchived(projectId: number, archived: boolean) {
    deps.patch({ saving: true });
    try {
      await deps.setProjectArchived({ projectId, archived });
      await reload();
    } catch (error) {
      deps.patch({
        status: deps.formatError(archived ? "archiving project" : "restoring project", error),
      });
    } finally {
      deps.patch({ saving: false });
    }
  }

  async function loadDataRange(input: ProjectDataRangeInput) {
    try {
      const dataRange = await deps.getProjectDataRange(input);
      deps.patch({ dataRange });
    } catch (error) {
      deps.patch({ status: deps.formatError("loading project data range", error) });
    }
  }

  return { reload, setPinned, setArchived, loadDataRange };
}
