export interface AnalysisSourceOption {
  id: number;
  account_id: number | null;
  title: string | null;
  item_count: number;
  last_synced_at: number | null;
}

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

export interface AnalysisRunEvent {
  run_id: number;
  kind: "started" | "progress" | "delta" | "completed" | "failed";
  phase: string;
  message: string | null;
  progress_current: number | null;
  progress_total: number | null;
  delta: string | null;
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

export interface AnalysisChatEvent {
  request_id: string;
  run_id: number;
  kind: "started" | "delta" | "completed" | "failed";
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
