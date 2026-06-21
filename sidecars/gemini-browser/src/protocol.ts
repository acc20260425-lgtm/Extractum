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

export interface GeminiBrowserRunRequest {
  run_id: string;
  prompt: string;
  source: string;
  artifact_mode: "reduced" | "full";
}

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
  manual_action: string | null;
  artifacts: {
    run_dir: string | null;
    html: string | null;
    screenshot: string | null;
    telemetry: string | null;
    answer_extraction?: string | null;
    artifact_write_error: string | null;
  };
  elapsed_ms: number;
  debug_summary?: GeminiBrowserRunDebugSummary | null;
}

export interface GeminiBrowserProviderStatus {
  status:
    | "not_started"
    | "ready"
    | "needs_login"
    | "needs_manual_action"
    | "running"
    | "stopped"
    | "failed";
  manual_action: string | null;
  active_run_id: string | null;
  queue_depth: number;
  browser_profile_dir: string;
  latest_message: string | null;
}

export type GeminiBrowserProviderMode = "managed" | "cdp_attach";

export interface GeminiBrowserProviderConfig {
  mode: GeminiBrowserProviderMode;
  cdp_endpoint?: string | null;
}

export type SidecarCommand =
  | {
      type: "status";
      browser_profile_dir: string;
      browser_config?: GeminiBrowserProviderConfig | null;
    }
  | {
      type: "open_browser";
      browser_profile_dir: string;
      browser_config?: GeminiBrowserProviderConfig | null;
    }
  | {
      type: "send_single";
      request: GeminiBrowserRunRequest;
      browser_profile_dir: string;
      artifact_dir: string;
      browser_config?: GeminiBrowserProviderConfig | null;
    }
  | {
      type: "resume";
      run_id: string | null;
      browser_profile_dir: string;
      browser_config?: GeminiBrowserProviderConfig | null;
    }
  | { type: "stop" };

export interface SidecarEnvelope {
  id: string;
  command: SidecarCommand;
}

export type SidecarResponse =
  | { type: "status"; status: GeminiBrowserProviderStatus }
  | { type: "run_result"; result: GeminiBrowserRunResult }
  | { type: "ack" }
  | { type: "error"; message: string };

export function parseEnvelope(line: string): SidecarEnvelope {
  const value = JSON.parse(line) as SidecarEnvelope;
  if (!value.id || typeof value.id !== "string") {
    throw new Error("Sidecar envelope id is required");
  }
  if (!value.command || typeof value.command.type !== "string") {
    throw new Error("Sidecar command type is required");
  }
  return value;
}
