export type GeminiBrowserProviderStatusKind =
  | "not_started"
  | "ready"
  | "needs_login"
  | "needs_manual_action"
  | "running"
  | "stopped"
  | "failed";

export type GeminiBrowserManualAction =
  | "login"
  | "account_picker"
  | "consent"
  | "captcha"
  | "unknown_modal"
  | "start_chrome_cdp";

export type GeminiBrowserProviderMode = "managed" | "cdp_attach";

export type GeminiBrowserDebugErrorStage =
  | "setup"
  | "composer"
  | "send"
  | "answer"
  | "artifacts"
  | "transport";

export type GeminiBrowserAnswerCompletionReason = "stable" | "timeout_latest" | "missing";

export type GeminiBrowserCandidateRejectReason =
  | "baseline"
  | "composer"
  | "prompt_container"
  | "navigation"
  | "account_or_login"
  | "controls"
  | "multi_turn"
  | "not_visible"
  | "empty"
  | "lower_score";

export type GeminiBrowserAnswerGrouping = "assistant_turn" | "single_node" | "unknown";

export interface GeminiBrowserAnswerExtractionDebug {
  raw_candidate_count: number;
  grouped_candidate_count: number;
  selected_candidate_length: number;
  returned_text_length: number;
  selected_grouping: GeminiBrowserAnswerGrouping;
  selected_candidate_rank: number | null;
  selected_score: number | null;
  largest_candidate_length: number;
  larger_valid_candidate_available: boolean;
  larger_rejected_candidate_count: number;
  larger_rejected_reasons: GeminiBrowserCandidateRejectReason[];
  top_candidate_lengths: number[];
  busy_visible_at_completion: boolean;
  last_growth_elapsed_ms: number | null;
  candidate_signature_changed_count: number;
  stable_poll_count_after_last_candidate_change: number;
}

export interface GeminiBrowserProviderConfig {
  mode: GeminiBrowserProviderMode;
  cdpEndpoint?: string | null;
}

export interface GeminiBrowserStartChromeResult {
  browser_profile_dir: string;
  cdp_endpoint: string;
  message: string;
}

export interface GeminiBrowserProviderStatus {
  status: GeminiBrowserProviderStatusKind;
  manual_action: GeminiBrowserManualAction | null;
  active_run_id: string | null;
  queue_depth: number;
  browser_profile_dir: string;
  latest_message: string | null;
}

export type GeminiBrowserRunStatus =
  | "queued"
  | "running"
  | "ok"
  | "ready"
  | "needs_login"
  | "needs_manual_action"
  | "blocked"
  | "timeout"
  | "browser_crashed"
  | "failed"
  | "cancelled";

export interface GeminiBrowserArtifactRefs {
  run_dir: string | null;
  html: string | null;
  screenshot: string | null;
  telemetry: string | null;
  answer_extraction?: string | null;
  artifact_write_error: string | null;
}

export interface GeminiBrowserRunDebugSummary {
  mode: GeminiBrowserProviderMode;
  composer_found: boolean;
  send_button_found: boolean;
  generation_busy_observed: boolean;
  answer_found: boolean;
  answer_selector: string | null;
  waited_for_send_ms: number;
  waited_for_answer_ms: number;
  answer_stable_ms: number;
  answer_completion_reason: GeminiBrowserAnswerCompletionReason;
  final_text_length: number;
  error_stage: GeminiBrowserDebugErrorStage | null;
  extraction?: GeminiBrowserAnswerExtractionDebug | null;
}

export interface GeminiBrowserRunResult {
  run_id: string;
  status: GeminiBrowserRunStatus;
  text: string | null;
  message: string | null;
  manual_action: GeminiBrowserManualAction | null;
  artifacts: GeminiBrowserArtifactRefs;
  elapsed_ms: number;
  debug_summary?: GeminiBrowserRunDebugSummary | null;
}

export interface GeminiBrowserRun {
  run_id: string;
  source: string;
  status: GeminiBrowserRunStatus;
  prompt_preview: string;
  created_at: string;
  updated_at: string;
  result: GeminiBrowserRunResult | null;
}

export interface GeminiBrowserRunLogSummary {
  runs: GeminiBrowserRun[];
}

export interface GeminiBrowserRunEvent {
  run_id: string;
  status: GeminiBrowserRunStatus;
  message: string | null;
  queue_position: number | null;
}

export interface GeminiBridgeSendSingleInput {
  runId: string;
  prompt: string;
  source?: string | null;
  artifactMode?: "reduced" | "full" | null;
  browserConfig?: GeminiBrowserProviderConfig | null;
}
