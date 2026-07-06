import type { AddProjectSourcesOutcome, ProjectSourceRecord, ProjectSourcesInput } from "$lib/types/projects";

export type ProjectAddSourceConnectOrigin = "new_source" | "existing_source";
export type EmptyProjectAddSourceBehavior = "missing_source_id_status" | "silent";

export interface ProjectAddSourceWorkflowDeps {
  addProjectSources(input: ProjectSourcesInput): Promise<AddProjectSourcesOutcome>;
  refreshAfterProjectSourceConnect(): Promise<void>;
  setProjectAddSourceSaving(saving: boolean): void;
  setProjectAddSourceStatus(message: string): void;
  formatError(action: string, error: unknown): string;
}

export interface ConnectProjectSourceIdsInput {
  projectId: number | null;
  sourceIds: Array<number | null | undefined>;
  origin: ProjectAddSourceConnectOrigin;
  deps: ProjectAddSourceWorkflowDeps;
  emptyBehavior?: EmptyProjectAddSourceBehavior;
}

export function normalizeProjectSourceIds(sourceIds: Array<number | null | undefined>) {
  return [...new Set(sourceIds.filter((id): id is number => typeof id === "number" && Number.isFinite(id)))];
}

export function connectedSourceIdsForProject(
  projectSources: Pick<ProjectSourceRecord, "project_id" | "source_id">[],
  projectId: number | null,
) {
  if (projectId === null) return new Set<number>();
  return new Set(projectSources.filter((source) => source.project_id === projectId).map((source) => source.source_id));
}

function outcomeStatus(outcome: AddProjectSourcesOutcome, origin: ProjectAddSourceConnectOrigin) {
  if (outcome.added_count > 0) {
    return origin === "existing_source"
      ? "Already in Library. Connected to project."
      : "Source added and connected to project.";
  }
  if (outcome.already_present_count > 0) {
    return "Already connected to this project.";
  }
  return null;
}

export async function connectProjectSourceIds({
  projectId,
  sourceIds,
  origin,
  deps,
  emptyBehavior = "silent",
}: ConnectProjectSourceIdsInput) {
  if (projectId === null) {
    deps.setProjectAddSourceStatus("Select a project");
    return;
  }

  const normalizedSourceIds = normalizeProjectSourceIds(sourceIds);
  if (normalizedSourceIds.length === 0) {
    await deps.refreshAfterProjectSourceConnect();
    if (emptyBehavior === "missing_source_id_status") {
      deps.setProjectAddSourceStatus("Source added to Library, but auto-connect could not be completed.");
    }
    return;
  }

  deps.setProjectAddSourceSaving(true);
  try {
    const outcome = await deps.addProjectSources({ projectId, sourceIds: normalizedSourceIds });
    const status = outcomeStatus(outcome, origin);
    if (status !== null) {
      deps.setProjectAddSourceStatus(status);
    }
    await deps.refreshAfterProjectSourceConnect();
  } catch (error) {
    if (origin === "new_source") {
      deps.setProjectAddSourceStatus(
        `Source added to Library, but connecting it to the project failed: ${deps.formatError(
          "connecting source to project",
          error,
        )}`,
      );
    } else {
      deps.setProjectAddSourceStatus(deps.formatError("connecting source to project", error));
    }
  } finally {
    deps.setProjectAddSourceSaving(false);
  }
}
