export interface AnalysisSourceOption {
  id: number;
  account_id: number | null;
  title: string | null;
  item_count: number;
  last_synced_at: number | null;
}

export type AnalysisPromptTemplateKind = "report" | "chat";

export interface AnalysisPromptTemplate {
  id: number;
  name: string;
  template_kind: string;
  body: string;
  version: number;
  is_builtin: boolean;
  created_at: number;
  updated_at: number;
}

export interface AnalysisSourceGroupMember {
  source_id: number;
  source_title: string | null;
  item_count: number;
}

export interface AnalysisSourceGroup {
  id: number;
  name: string;
  members: AnalysisSourceGroupMember[];
  created_at: number;
  updated_at: number;
}

export interface AnalysisRunSummary {
  id: number;
  run_type: string;
  scope_type: string;
  source_id: number | null;
  source_title: string | null;
  source_group_id: number | null;
  source_group_name: string | null;
  scope_label: string;
  period_from: number;
  period_to: number;
  output_language: string;
  prompt_template_id: number | null;
  prompt_template_name: string | null;
  prompt_template_version: number;
  provider_profile: string;
  provider: string;
  model: string;
  status: string;
  error: string | null;
  has_trace_data: boolean;
  created_at: number;
  completed_at: number | null;
}

export interface AnalysisRunDetail extends AnalysisRunSummary {
  result_markdown: string | null;
  error: string | null;
}

export interface ListAnalysisRunsInput {
  sourceId: number | null;
  sourceGroupId: number | null;
  limit: number;
}

export interface AnalysisReportStartCommand {
  sourceId: number | null;
  sourceGroupId: number | null;
  periodFrom: number;
  periodTo: number;
  outputLanguage: string;
  promptTemplateId: number;
  modelOverride: string | null;
  profileId: string | null;
}

export interface AnalysisTraceRef {
  ref: string;
  item_id: number;
  source_id: number;
  external_id: string;
  published_at: number;
  excerpt: string;
}

export interface AnalysisTraceData {
  refs: AnalysisTraceRef[];
}

export interface AnalysisChunkSummaryEvent {
  index: number;
  total: number;
  message_count: number;
  summary: string;
  topics: string[];
  notable_points: string[];
  candidate_refs: string[];
}

export interface AnalysisRunEvent {
  run_id: number;
  request_id: string | null;
  kind: "queued" | "started" | "progress" | "delta" | "completed" | "failed" | "cancelled";
  phase: string;
  queue_position: number | null;
  message: string | null;
  progress_current: number | null;
  progress_total: number | null;
  delta: string | null;
  chunk_summary: AnalysisChunkSummaryEvent | null;
  error: string | null;
}

export interface AnalysisChatTurn {
  role: "user" | "assistant";
  content: string;
}

export interface AnalysisChatMessage {
  id: number;
  run_id: number;
  role: "user" | "assistant";
  content: string;
  created_at: number;
}

export interface AskAnalysisRunQuestionInput {
  runId: number;
  question: string;
  modelOverride: string | null;
  profileId: string | null;
}

export interface AnalysisChatEvent {
  request_id: string;
  run_id: number;
  kind: "queued" | "started" | "delta" | "completed" | "failed" | "cancelled";
  queue_position: number | null;
  delta: string | null;
  message: string | null;
  error: string | null;
}

export interface EventEnvelope<T> {
  payload: T;
}

export type ReportSegment =
  | { type: "text"; value: string; key: string }
  | { type: "ref"; value: string; key: string };

export interface CreateAnalysisPromptTemplateInput {
  name: string;
  templateKind: AnalysisPromptTemplateKind;
  body: string;
}

export interface UpdateAnalysisPromptTemplateInput {
  templateId: number;
  name: string;
  body: string;
}

export interface CreateAnalysisSourceGroupInput {
  name: string;
  sourceIds: number[];
}

export interface UpdateAnalysisSourceGroupInput extends CreateAnalysisSourceGroupInput {
  groupId: number;
}
