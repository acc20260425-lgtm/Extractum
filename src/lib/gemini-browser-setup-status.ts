import {
  isPartialRiskBrowserResult,
  sanitizeDiagnosticMessage,
} from "./gemini-browser-run-inspector";
import type {
  GeminiBrowserProviderMode,
  GeminiBrowserProviderStatus,
  GeminiBrowserRun,
  GeminiBrowserRunStatus,
} from "./types/gemini-browser";

export type GeminiBrowserSetupCheckState =
  | "ready"
  | "action_needed"
  | "running"
  | "warning"
  | "failed"
  | "unknown"
  | "not_applicable";

export type GeminiBrowserSetupCheckAction =
  | "refresh"
  | "start_chrome"
  | "open"
  | "resume"
  | "send_test"
  | "view_run"
  | "focus_endpoint";

export type GeminiBrowserSetupCheckId =
  | "sidecar"
  | "mode"
  | "chrome_cdp"
  | "gemini_tab"
  | "gemini_readiness"
  | "last_test_run";

export interface GeminiBrowserSetupCheck {
  id: GeminiBrowserSetupCheckId;
  label: string;
  state: GeminiBrowserSetupCheckState;
  message: string;
  action: GeminiBrowserSetupCheckAction | null;
  runId?: string | null;
}

export interface GeminiBrowserSetupStatusInput {
  status: GeminiBrowserProviderStatus | null;
  providerMode: GeminiBrowserProviderMode;
  cdpEndpoint: string;
  runs: GeminiBrowserRun[];
  selectedRun: GeminiBrowserRun | null;
  busy: boolean;
  statusLoadError: string | null;
}

const FAILED_RUN_STATUSES = new Set<GeminiBrowserRunStatus>([
  "failed",
  "timeout",
  "browser_crashed",
  "blocked",
]);

const MANUAL_ACTION_RUN_STATUSES = new Set<GeminiBrowserRunStatus>([
  "needs_login",
  "needs_manual_action",
]);

export function deriveGeminiBrowserSetupChecks(
  input: GeminiBrowserSetupStatusInput,
): GeminiBrowserSetupCheck[] {
  const lastRun = input.selectedRun ?? input.runs[0] ?? null;
  return [
    sidecarCheck(input),
    modeCheck(input),
    chromeCdpCheck(input),
    geminiTabCheck(input),
    geminiReadinessCheck(input, lastRun),
    lastTestRunCheck(lastRun),
  ];
}

export function setupCheckStateLabel(state: GeminiBrowserSetupCheckState): string {
  if (state === "ready") return "Ready";
  if (state === "action_needed") return "Action needed";
  if (state === "running") return "Running";
  if (state === "warning") return "Warning";
  if (state === "failed") return "Failed";
  if (state === "not_applicable") return "Not applicable";
  return "Unknown";
}

export function setupCheckActionLabel(action: GeminiBrowserSetupCheckAction): string {
  if (action === "refresh") return "Refresh";
  if (action === "start_chrome") return "Start Chrome";
  if (action === "open") return "Open";
  if (action === "resume") return "Resume";
  if (action === "send_test") return "Send test";
  if (action === "view_run") return "View run";
  return "Edit endpoint";
}

function sidecarCheck(input: GeminiBrowserSetupStatusInput): GeminiBrowserSetupCheck {
  if (input.statusLoadError) {
    return {
      id: "sidecar",
      label: "Sidecar",
      state: "failed",
      message: `Status failed: ${sanitizeDiagnosticMessage(input.statusLoadError)}`,
      action: "refresh",
    };
  }
  if (!input.status) {
    return {
      id: "sidecar",
      label: "Sidecar",
      state: "unknown",
      message: "Refresh provider status to check the sidecar.",
      action: "refresh",
    };
  }
  return {
    id: "sidecar",
    label: "Sidecar",
    state: "ready",
    message: "Sidecar status responded.",
    action: "refresh",
  };
}

function modeCheck(input: GeminiBrowserSetupStatusInput): GeminiBrowserSetupCheck {
  if (input.providerMode === "managed") {
    return {
      id: "mode",
      label: "Mode",
      state: "ready",
      message: "Managed browser profile is selected.",
      action: null,
    };
  }
  if (!isLocalHttpCdpEndpoint(input.cdpEndpoint)) {
    return {
      id: "mode",
      label: "Mode",
      state: "action_needed",
      message: "Attach Chrome requires a local HTTP endpoint such as http://127.0.0.1:9222.",
      action: "focus_endpoint",
    };
  }
  return {
    id: "mode",
    label: "Mode",
    state: "ready",
    message: "Attach Chrome is selected with a local CDP endpoint.",
    action: null,
  };
}

function chromeCdpCheck(input: GeminiBrowserSetupStatusInput): GeminiBrowserSetupCheck {
  if (input.providerMode === "managed") {
    return {
      id: "chrome_cdp",
      label: "Chrome CDP",
      state: "not_applicable",
      message: "Managed mode does not use Chrome CDP attach.",
      action: null,
    };
  }
  if (!input.status) {
    return {
      id: "chrome_cdp",
      label: "Chrome CDP",
      state: "unknown",
      message: "Refresh status to check Chrome CDP.",
      action: "refresh",
    };
  }
  if (input.status.status === "ready") {
    return {
      id: "chrome_cdp",
      label: "Chrome CDP",
      state: "ready",
      message: "Chrome CDP is attached.",
      action: null,
    };
  }
  if (input.status.manual_action === "start_chrome_cdp") {
    return {
      id: "chrome_cdp",
      label: "Chrome CDP",
      state: "action_needed",
      message: sanitizeDiagnosticMessage(input.status.latest_message),
      action: chromeActionFromMessage(input.status.latest_message),
    };
  }
  return {
    id: "chrome_cdp",
    label: "Chrome CDP",
    state: "unknown",
    message: "Resume the provider to attach to Chrome CDP.",
    action: "resume",
  };
}

function geminiTabCheck(input: GeminiBrowserSetupStatusInput): GeminiBrowserSetupCheck {
  if (!input.status) {
    return {
      id: "gemini_tab",
      label: "Gemini tab",
      state: "unknown",
      message: "Refresh status to check the Gemini tab.",
      action: "refresh",
    };
  }
  if (input.status.status === "ready") {
    return {
      id: "gemini_tab",
      label: "Gemini tab",
      state: "ready",
      message: "A usable Gemini page is available.",
      action: null,
    };
  }
  if (input.providerMode === "managed") {
    return {
      id: "gemini_tab",
      label: "Gemini tab",
      state: "unknown",
      message: "Open the managed browser to load Gemini.",
      action: "open",
    };
  }
  if (input.status.manual_action === "start_chrome_cdp") {
    return {
      id: "gemini_tab",
      label: "Gemini tab",
      state: "action_needed",
      message: sanitizeDiagnosticMessage(input.status.latest_message),
      action: chromeActionFromMessage(input.status.latest_message),
    };
  }
  return {
    id: "gemini_tab",
    label: "Gemini tab",
    state: "unknown",
    message: "Open Gemini in the attached browser or resume the provider.",
    action: "open",
  };
}

function geminiReadinessCheck(
  input: GeminiBrowserSetupStatusInput,
  lastRun: GeminiBrowserRun | null,
): GeminiBrowserSetupCheck {
  if (input.status?.status === "needs_login") {
    return {
      id: "gemini_readiness",
      label: "Gemini readiness",
      state: "action_needed",
      message: "Gemini needs login or another browser-side manual step.",
      action: input.providerMode === "cdp_attach" ? "resume" : "open",
    };
  }
  if (lastRun?.result?.manual_action) {
    return {
      id: "gemini_readiness",
      label: "Gemini readiness",
      state: "action_needed",
      message: `Manual action required: ${lastRun.result.manual_action}.`,
      action: "view_run",
      runId: lastRun.run_id,
    };
  }
  const debug = lastRun?.result?.debug_summary;
  if (debug?.composer_found && debug.send_button_found) {
    return {
      id: "gemini_readiness",
      label: "Gemini readiness",
      state: "ready",
      message: "Composer and send button were found in the latest inspected run.",
      action: "send_test",
    };
  }
  if (input.status?.status === "ready") {
    return {
      id: "gemini_readiness",
      label: "Gemini readiness",
      state: "warning",
      message: "Send a test prompt to confirm Gemini is ready.",
      action: "send_test",
    };
  }
  return {
    id: "gemini_readiness",
    label: "Gemini readiness",
    state: "unknown",
    message: "Browser setup must be ready before testing Gemini.",
    action: input.providerMode === "cdp_attach" ? "resume" : "open",
  };
}

function lastTestRunCheck(lastRun: GeminiBrowserRun | null): GeminiBrowserSetupCheck {
  if (!lastRun) {
    return {
      id: "last_test_run",
      label: "Last test run",
      state: "unknown",
      message: "No Browser Provider test run is loaded yet.",
      action: null,
      runId: null,
    };
  }
  if (lastRun.status === "running" || lastRun.status === "queued") {
    return {
      id: "last_test_run",
      label: "Last test run",
      state: "running",
      message: `Run ${lastRun.run_id} is ${lastRun.status}.`,
      action: "view_run",
      runId: lastRun.run_id,
    };
  }
  const result = lastRun.result;
  const effectiveStatus = result?.status ?? lastRun.status;
  if (MANUAL_ACTION_RUN_STATUSES.has(effectiveStatus) || result?.manual_action) {
    return {
      id: "last_test_run",
      label: "Last test run",
      state: "action_needed",
      message: `Run ${lastRun.run_id} needs manual action.`,
      action: "view_run",
      runId: lastRun.run_id,
    };
  }
  if (FAILED_RUN_STATUSES.has(effectiveStatus)) {
    return {
      id: "last_test_run",
      label: "Last test run",
      state: "failed",
      message: `Run ${lastRun.run_id} ended with ${effectiveStatus}.`,
      action: "view_run",
      runId: lastRun.run_id,
    };
  }
  if (!result?.debug_summary) {
    return {
      id: "last_test_run",
      label: "Last test run",
      state: result ? "unknown" : "running",
      message: result ? `Run ${lastRun.run_id} has no debug summary.` : `Run ${lastRun.run_id} is pending.`,
      action: "view_run",
      runId: lastRun.run_id,
    };
  }
  if (result.status === "ok" && result.debug_summary.answer_completion_reason === "stable") {
    return {
      id: "last_test_run",
      label: "Last test run",
      state: "ready",
      message: `Run ${lastRun.run_id} completed with a stable answer.`,
      action: "view_run",
      runId: lastRun.run_id,
    };
  }
  if (isPartialRiskBrowserResult(result)) {
    return {
      id: "last_test_run",
      label: "Last test run",
      state: "warning",
      message: `Run ${lastRun.run_id} is partial-risk (timeout_latest).`,
      action: "view_run",
      runId: lastRun.run_id,
    };
  }
  return {
    id: "last_test_run",
    label: "Last test run",
    state: "unknown",
    message: `Run ${lastRun.run_id} needs inspection.`,
    action: "view_run",
    runId: lastRun.run_id,
  };
}

function chromeActionFromMessage(message: string | null): GeminiBrowserSetupCheckAction {
  const normalized = message?.toLowerCase() ?? "";
  if (normalized.includes("attached") || normalized.includes("open gemini")) return "open";
  if (normalized.includes("configured but not attached")) return "resume";
  return "start_chrome";
}

function isLocalHttpCdpEndpoint(value: string): boolean {
  const trimmed = value.trim();
  if (!trimmed) return false;
  try {
    const url = new URL(trimmed);
    const host = url.hostname;
    return (
      url.protocol === "http:" &&
      (host === "127.0.0.1" || host === "localhost" || host === "[::1]" || host === "::1") &&
      Boolean(url.port) &&
      url.pathname === "/" &&
      !url.search &&
      !url.hash &&
      !url.username &&
      !url.password
    );
  } catch {
    return false;
  }
}
