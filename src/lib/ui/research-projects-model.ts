import type { AnalysisRunSummary } from "$lib/types/analysis";
import type {
  LibraryCatalogRecord,
  LibrarySourceProvider,
  LibrarySourceSubtype,
} from "$lib/types/library-sources";
import type {
  DeleteProjectYoutubeVideoSourceOutcome,
  ProjectRecord,
  ProjectSourceRecord,
} from "$lib/types/projects";
import { librarySourceTypeLabel } from "./library-catalog-model";

export type ProjectStatus = "ready" | "running" | "needs_attention" | "empty";
export type LibrarySourceStatus = "active" | "needs_account" | "syncing" | "error" | "unavailable";

export type ResearchProjectBacking =
  | { kind: "project"; projectId: number }
  | { kind: "source_group"; groupId: number; sourceType: LibrarySourceProvider };

export type ResearchProjectView = {
  id: string;
  projectId: number;
  title: string;
  description: string | null;
  periodLabel: string;
  sourceCount: number;
  evidenceCount: number;
  materialCount: number;
  lastRunLabel: string | null;
  status: ProjectStatus;
  backing: ResearchProjectBacking;
};

export type LibrarySourceView = {
  id: string;
  sourceId: number;
  provider: LibrarySourceProvider;
  typeLabel: string;
  title: string;
  subtitle: string | null;
  projectCount: number;
  lastCollectedAt: number | null;
  lastCollectedLabel: string | null;
  localCopyLabel: string | null;
  status: LibrarySourceStatus;
  disabledReason: string | null;
  alreadyConnected: boolean;
  connectable: boolean;
};

export type ProjectSourceLinkView = {
  projectId: string;
  sourceId: string;
  sourceNumericId: number;
  provider: LibrarySourceProvider;
  subtype: LibrarySourceSubtype;
  typeLabel: string;
  title: string;
  subtitle: string | null;
  itemCount: number;
  localCopyLabel: string;
  addedAt: number;
  addedAtLabel: string | null;
  connectionStatus: "connected";
  filterSummary: string;
};

export type LibraryFilterState = {
  query: string;
  providers: LibrarySourceProvider[];
};

export const PROJECT_PERIOD_LABEL = "All time";

export function projectViewId(projectId: number) {
  return `project:${projectId}`;
}

export function projectIdFromViewId(viewId: string | null) {
  if (!viewId?.startsWith("project:")) return null;
  const value = Number(viewId.slice("project:".length));
  return Number.isFinite(value) ? value : null;
}

function sourceRowId(sourceId: number) {
  return `source:${sourceId}`;
}

function materialLabel(count: number) {
  if (count === 1) return "1 material";
  return `${count} materials`;
}

function dateLabel(unixSeconds: number | null) {
  if (!unixSeconds) return null;
  return new Intl.DateTimeFormat("en", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(unixSeconds * 1000));
}

export function buildResearchProjectsView(
  projects: ProjectRecord[],
  projectSources: ProjectSourceRecord[],
  runs: AnalysisRunSummary[] = [],
): ResearchProjectView[] {
  return projects.map((project) => {
    const sources = projectSources.filter((source) => source.project_id === project.id);
    const materialCount = sources.reduce((total, source) => total + source.item_count, 0);
    const latestRun = runs
      .filter((run) => run.project_id === project.id)
      .sort((left, right) => right.created_at - left.created_at)[0];
    const running = runs.some(
      (run) => run.project_id === project.id && (run.status === "queued" || run.status === "running"),
    );
    return {
      id: projectViewId(project.id),
      projectId: project.id,
      title: project.name,
      description: project.description,
      periodLabel: PROJECT_PERIOD_LABEL,
      sourceCount: sources.length,
      evidenceCount: materialCount,
      materialCount,
      lastRunLabel: latestRun ? dateLabel(latestRun.created_at) : null,
      status: running ? "running" : sources.length === 0 ? "empty" : "ready",
      backing: { kind: "project", projectId: project.id },
    };
  });
}

export function buildLibrarySourcesView(
  catalogRecords: LibraryCatalogRecord[],
  projectSources: ProjectSourceRecord[],
  selectedProjectId: string | null,
): LibrarySourceView[] {
  const projectId = projectIdFromViewId(selectedProjectId);
  const connectedIds = new Set(
    projectSources
      .filter((source) => projectId !== null && source.project_id === projectId)
      .map((source) => source.source_id),
  );

  return catalogRecords.map((record) => {
    const source = record.source;
    const alreadyConnected = connectedIds.has(source.source_id);
    const catalogDisabledReason = record.disabled_reasons.connect_to_project;
    const disabledReason = alreadyConnected ? "Already in project" : catalogDisabledReason;
    const connectable = disabledReason === null && record.capabilities.can_connect_to_project;

    return {
      id: sourceRowId(source.source_id),
      sourceId: source.source_id,
      provider: source.provider,
      typeLabel: librarySourceTypeLabel(source.provider, source.source_subtype),
      title: source.title ?? `Source #${source.source_id}`,
      subtitle: source.subtitle,
      projectCount: source.project_count,
      lastCollectedAt: source.last_synced_at,
      lastCollectedLabel: dateLabel(source.last_synced_at),
      localCopyLabel: materialLabel(source.item_count),
      status: record.status,
      disabledReason,
      alreadyConnected,
      connectable,
    };
  });
}

export function filterLibrarySources(sources: LibrarySourceView[], filters: LibraryFilterState) {
  const query = filters.query.trim().toLocaleLowerCase();
  const providers = new Set(filters.providers);
  return sources.filter((source) => {
    const matchesQuery =
      !query || `${source.title} ${source.subtitle ?? ""}`.toLocaleLowerCase().includes(query);
    const matchesProvider = providers.size === 0 || providers.has(source.provider);
    return matchesQuery && matchesProvider;
  });
}

export function connectableSelection(sources: LibrarySourceView[], selectedIds: Set<string>) {
  return sources.filter((source) => selectedIds.has(source.id) && source.connectable);
}

export function reconcileProjectSourceSelection(
  selectedIds: string[],
  rows: Pick<ProjectSourceLinkView, "sourceId">[],
) {
  const visibleIds = new Set(rows.map((row) => row.sourceId));
  return selectedIds.filter((id) => visibleIds.has(id));
}

export function selectedProjectSourcesSyncDisabledReason(
  rows: Pick<ProjectSourceLinkView, "provider" | "subtype">[],
) {
  if (rows.length === 0) return "Select sources to sync";
  const hasUnsupported = rows.some(
    (row) => row.provider !== "youtube" || row.subtype !== "video",
  );
  if (hasUnsupported) return "Selected sources include unsupported sync types";
  return null;
}

export const PROJECT_YOUTUBE_VIDEO_LIBRARY_DELETE_CONFIRM =
  "Delete this YouTube video from the project and Library? The app will cancel the deletion if another project still uses it. This will remove its transcript, comments, and stored materials.";

export function selectedProjectSourceLibraryDeleteDisabledReason(
  rows: Pick<ProjectSourceLinkView, "provider" | "subtype">[],
) {
  if (rows.length !== 1) return "Select one YouTube video source";
  const [row] = rows;
  if (row.provider !== "youtube" || row.subtype !== "video") {
    return "Only YouTube videos can be deleted from Library here";
  }
  return null;
}

export function projectSourceLibraryDeleteStatus(
  outcome: DeleteProjectYoutubeVideoSourceOutcome,
) {
  if (outcome.status === "deleted") return "Source deleted from project and Library.";
  const names = outcome.blocking_projects.map((project) => project.title).join(", ");
  const suffix =
    outcome.remaining_blocking_project_count > 0
      ? `, and ${outcome.remaining_blocking_project_count} more`
      : "";
  return `Cannot delete from Library: source is used by other projects: ${names}${suffix}.`;
}

export function buildProjectSourceLinksView(
  projectId: string | null,
  projectSources: ProjectSourceRecord[],
): ProjectSourceLinkView[] {
  const numericProjectId = projectIdFromViewId(projectId);
  if (!projectId || numericProjectId === null) return [];
  return projectSources
    .filter((source) => source.project_id === numericProjectId)
    .map((source) => ({
      projectId,
      sourceId: sourceRowId(source.source_id),
      sourceNumericId: source.source_id,
      provider: source.provider,
      subtype: source.source_subtype,
      typeLabel: librarySourceTypeLabel(source.provider, source.source_subtype),
      title: source.title ?? `Source #${source.source_id}`,
      subtitle: source.subtitle,
      itemCount: source.item_count,
      localCopyLabel: materialLabel(source.item_count),
      addedAt: source.added_at,
      addedAtLabel: dateLabel(source.added_at),
      connectionStatus: "connected",
      filterSummary: source.subtitle ?? source.provider,
    }));
}

export function projectRunDisabledReason(
  project: ProjectRecord | ResearchProjectView | null,
  sources: Pick<ProjectSourceRecord, "provider">[],
) {
  if (!project) return "Select a project";
  if (sources.length === 0) return "Add sources to run analysis";
  const providers = new Set(sources.map((source) => source.provider));
  if (providers.size > 1) return "Mixed-provider project analysis runs are not supported yet.";
  return null;
}
