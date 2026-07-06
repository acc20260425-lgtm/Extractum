import type { AnalysisRunSummary, YoutubeCorpusMode } from "$lib/types/analysis";
import type {
  LibraryCatalogStatus,
  LibrarySourceProvider,
  LibrarySourceSubtype,
} from "$lib/types/library-sources";

export type ProjectStatus = "ready" | "running" | "needs_attention" | "empty";

export interface ProjectSummary {
  id: number;
  name: string;
  description: string | null;
  source_count: number;
  material_count: number;
  status: ProjectStatus;
  last_run_at: number | null;
  pinned: boolean;
  archived: boolean;
  updated_at: number;
}

export interface ProjectRecord {
  id: number;
  name: string;
  description: string | null;
  created_at: number;
  updated_at: number;
}

export interface ProjectSourceRecord {
  project_id: number;
  source_id: number;
  provider: LibrarySourceProvider;
  source_subtype: LibrarySourceSubtype;
  title: string | null;
  subtitle: string | null;
  item_count: number;
  added_at: number;
  last_synced_at: number | null;
  sync_status: LibraryCatalogStatus;
  handle: string | null;
}

export interface ProjectDataRange {
  from: number | null;
  to: number | null;
}

export interface ProjectDataRangeInput {
  projectId: number;
  youtubeCorpusMode: YoutubeCorpusMode | null;
  includeMigratedHistory: boolean;
}

export interface ProjectPinnedInput {
  projectId: number;
  pinned: boolean;
}

export interface ProjectArchivedInput {
  projectId: number;
  archived: boolean;
}

export interface AddProjectSourcesOutcome {
  added_count: number;
  already_present_count: number;
}

export interface ProjectEditorInput {
  name: string;
  description: string | null;
}

export interface UpdateProjectInput extends ProjectEditorInput {
  projectId: number;
}

export interface ProjectSourcesInput {
  projectId: number;
  sourceIds: number[];
}

export interface DeleteProjectYoutubeVideoSourceInput {
  projectId: number;
  sourceId: number;
}

export type DeleteProjectYoutubeVideoSourceStatus = "deleted" | "blocked_by_other_projects";

export interface BlockingProjectReference {
  project_id: number;
  title: string;
  archived: boolean;
}

export interface DeleteProjectYoutubeVideoSourceOutcome {
  status: DeleteProjectYoutubeVideoSourceStatus;
  blocking_projects: BlockingProjectReference[];
  remaining_blocking_project_count: number;
}

export interface ProjectAnalysisStartCommand {
  projectId: number;
  periodFrom: number;
  periodTo: number;
  outputLanguage: string;
  promptTemplateId: number;
  modelOverride: string | null;
  profileId: string | null;
  youtubeCorpusMode: YoutubeCorpusMode;
  includeMigratedHistory: boolean;
}

export type ProjectRuns = AnalysisRunSummary[];
