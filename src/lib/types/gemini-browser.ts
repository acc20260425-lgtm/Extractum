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
