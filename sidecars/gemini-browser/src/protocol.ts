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
    artifact_write_error: string | null;
  };
  elapsed_ms: number;
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

export type SidecarCommand =
  | { type: "status"; browser_profile_dir: string }
  | { type: "open_browser"; browser_profile_dir: string }
  | {
      type: "send_single";
      request: GeminiBrowserRunRequest;
      browser_profile_dir: string;
      artifact_dir: string;
    }
  | { type: "resume"; run_id: string | null; browser_profile_dir: string }
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
