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
  | "unknown_modal";

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

export interface GeminiBrowserRunResult {
  run_id: string;
  status: GeminiBrowserRunStatus;
  text: string | null;
  message: string | null;
  manual_action: GeminiBrowserManualAction | null;
  artifacts: GeminiBrowserArtifactRefs;
  elapsed_ms: number;
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
}
