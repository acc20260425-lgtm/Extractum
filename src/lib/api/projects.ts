import { invoke } from "@tauri-apps/api/core";
import type {
  AddProjectSourcesOutcome,
  ProjectArchivedInput,
  ProjectAnalysisStartCommand,
  ProjectDataRange,
  ProjectDataRangeInput,
  ProjectEditorInput,
  ProjectPinnedInput,
  ProjectRecord,
  ProjectRuns,
  ProjectSourceRecord,
  ProjectSourcesInput,
  ProjectSummary,
  UpdateProjectInput,
} from "$lib/types/projects";

export function listProjects() {
  return invoke<ProjectRecord[]>("list_projects");
}

export function listResearchProjects() {
  return invoke<ProjectSummary[]>("list_research_projects");
}

export function createProject(input: ProjectEditorInput) {
  return invoke<ProjectRecord>("create_project", { ...input });
}

export function updateProject(input: UpdateProjectInput) {
  return invoke<ProjectRecord>("update_project", { ...input });
}

export function deleteProject(projectId: number) {
  return invoke<void>("delete_project", { projectId });
}

export function setProjectPinned(input: ProjectPinnedInput) {
  return invoke<void>("set_project_pinned", { ...input });
}

export function setProjectArchived(input: ProjectArchivedInput) {
  return invoke<void>("set_project_archived", { ...input });
}

export function listProjectSources(projectId: number) {
  return invoke<ProjectSourceRecord[]>("list_project_sources", { projectId });
}

export function addProjectSources(input: ProjectSourcesInput) {
  return invoke<AddProjectSourcesOutcome>("add_project_sources", { ...input });
}

export function removeProjectSources(input: ProjectSourcesInput) {
  return invoke<void>("remove_project_sources", { ...input });
}

export function listProjectRuns(projectId: number) {
  return invoke<ProjectRuns>("list_project_runs", { projectId });
}

export function startProjectAnalysis(command: ProjectAnalysisStartCommand) {
  return invoke<number>("start_project_analysis", { ...command });
}

export function getProjectDataRange(input: ProjectDataRangeInput) {
  return invoke<ProjectDataRange>("get_project_data_range", { ...input });
}
