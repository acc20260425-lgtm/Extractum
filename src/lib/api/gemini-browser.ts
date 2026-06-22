import { invoke } from "@tauri-apps/api/core";
import type {
  GeminiBridgeSendSingleInput,
  GeminiBrowserProviderConfig,
  GeminiBrowserProviderStatus,
  GeminiBrowserRun,
  GeminiBrowserRunLogSummary,
  GeminiBrowserRunResult,
  GeminiBrowserStartChromeResult,
} from "$lib/types/gemini-browser";

export function geminiBridgeStatus(browserConfig?: GeminiBrowserProviderConfig | null) {
  if (!browserConfig) return invoke<GeminiBrowserProviderStatus>("gemini_bridge_status");
  return invoke<GeminiBrowserProviderStatus>("gemini_bridge_status", { browserConfig });
}

export function geminiBridgeStatusSnapshot() {
  return invoke<GeminiBrowserProviderStatus>("gemini_bridge_status_snapshot");
}

export function geminiBridgeOpenBrowser(browserConfig?: GeminiBrowserProviderConfig | null) {
  if (!browserConfig) return invoke<GeminiBrowserProviderStatus>("gemini_bridge_open_browser");
  return invoke<GeminiBrowserProviderStatus>("gemini_bridge_open_browser", { browserConfig });
}

export function geminiBridgeStartCdpChrome(browserConfig?: GeminiBrowserProviderConfig | null) {
  if (!browserConfig) return invoke<GeminiBrowserStartChromeResult>("gemini_bridge_start_cdp_chrome");
  return invoke<GeminiBrowserStartChromeResult>("gemini_bridge_start_cdp_chrome", { browserConfig });
}

export function geminiBridgeSendSingle(input: GeminiBridgeSendSingleInput) {
  return invoke<GeminiBrowserRunResult>("gemini_bridge_send_single", { ...input });
}

export function geminiBridgeResume(browserConfig?: GeminiBrowserProviderConfig | null) {
  if (!browserConfig) return invoke<GeminiBrowserProviderStatus>("gemini_bridge_resume");
  return invoke<GeminiBrowserProviderStatus>("gemini_bridge_resume", { browserConfig });
}

export function geminiBridgeStop() {
  return invoke<void>("gemini_bridge_stop");
}

export function geminiBridgeListRuns(limit = 20) {
  return invoke<GeminiBrowserRunLogSummary>("gemini_bridge_list_runs", { limit });
}

export function geminiBridgeGetRun(runId: string) {
  return invoke<GeminiBrowserRun>("gemini_bridge_get_run", { runId });
}

export function geminiBridgeOpenRunFolder(runId: string) {
  return invoke<void>("gemini_bridge_open_run_folder", { runId });
}

// AppErrorKind::NotFound is serialized by Tauri as { kind: "not_found", message }.
export function isGeminiBrowserRunNotFoundError(error: unknown) {
  return (
    typeof error === "object" &&
    error !== null &&
    "kind" in error &&
    (error as { kind?: unknown }).kind === "not_found"
  );
}
