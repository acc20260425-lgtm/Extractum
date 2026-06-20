export type AdapterVariant = "dom-only" | "resilient-scoring" | "telemetry-assisted";

export type GeminiAdapterStatus =
  | "ready"
  | "running"
  | "ok"
  | "login_required"
  | "manual_action_required"
  | "captcha_required"
  | "account_picker"
  | "consent_required"
  | "rate_limited"
  | "generation_timeout"
  | "response_parse_failed"
  | "browser_crashed"
  | "failed";

export type LocatorAttempt = {
  name: string;
  strategy: "role" | "label" | "placeholder" | "text" | "css" | "structural" | "fuzzy";
  matched: boolean;
  count?: number;
  error?: string;
  score?: number;
};

export type NetworkEventSummary = {
  at: number;
  kind: "request" | "response" | "websocket-open" | "websocket-frame-received" | "websocket-close";
  url: string;
  method?: string;
  status?: number;
  contentType?: string;
  bytes?: number;
};

export type FailureArtifacts = {
  screenshotPath: string | null;
  htmlPath: string | null;
  telemetryPath: string | null;
  tracePath: string | null;
};

export type GeminiAdapterResult = {
  variant: AdapterVariant;
  status: GeminiAdapterStatus;
  rawText: string | null;
  elapsedMs: number;
  locatorAttempts: LocatorAttempt[];
  networkSummary: NetworkEventSummary[];
  artifacts: FailureArtifacts | null;
  errorReason: string | null;
};

export function isTerminalStatus(status: GeminiAdapterStatus): boolean {
  return status !== "ready" && status !== "running";
}

export function isSuccessStatus(status: GeminiAdapterStatus): boolean {
  return status === "ok" || status === "ready";
}

export function isManualActionStatus(status: GeminiAdapterStatus): boolean {
  return (
    status === "login_required" ||
    status === "manual_action_required" ||
    status === "captcha_required" ||
    status === "account_picker" ||
    status === "consent_required"
  );
}
