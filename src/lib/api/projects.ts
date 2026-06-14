import { invoke } from "@tauri-apps/api/core";
import type {
  AddProjectSourcesOutcome,
  ProjectAnalysisStartCommand,
  ProjectEditorInput,
  ProjectRecord,
  ProjectRuns,
  ProjectSourceRecord,
  ProjectSourcesInput,
  UpdateProjectInput,
} from "$lib/types/projects";

export function listProjects() {
  return invoke<ProjectRecord[]>("list_projects");
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
