import type { AnalysisRunSummary, YoutubeCorpusMode } from "$lib/types/analysis";
import type { LibrarySourceProvider, LibrarySourceSubtype } from "$lib/types/library-sources";

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
