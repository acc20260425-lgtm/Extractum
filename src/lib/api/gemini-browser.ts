import { invoke } from "@tauri-apps/api/core";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  GeminiBridgeSendSingleInput,
  GeminiBrowserProviderConfig,
  GeminiBrowserProviderStatus,
  GeminiBrowserRunEvent,
  GeminiBrowserRunLogSummary,
  GeminiBrowserRunResult,
} from "$lib/types/gemini-browser";

export const GEMINI_BROWSER_RUN_EVENT = "gemini-browser://run";

export function geminiBridgeStatus(browserConfig?: GeminiBrowserProviderConfig | null) {
  if (!browserConfig) return invoke<GeminiBrowserProviderStatus>("gemini_bridge_status");
  return invoke<GeminiBrowserProviderStatus>("gemini_bridge_status", { browserConfig });
}

export function geminiBridgeOpenBrowser(browserConfig?: GeminiBrowserProviderConfig | null) {
  if (!browserConfig) return invoke<GeminiBrowserProviderStatus>("gemini_bridge_open_browser");
  return invoke<GeminiBrowserProviderStatus>("gemini_bridge_open_browser", { browserConfig });
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

export function listenToGeminiBrowserRuns(
  handler: (event: Event<GeminiBrowserRunEvent>) => void,
): Promise<UnlistenFn> {
  return listen<GeminiBrowserRunEvent>(GEMINI_BROWSER_RUN_EVENT, handler);
}
