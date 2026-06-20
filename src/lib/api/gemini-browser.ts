import { invoke } from "@tauri-apps/api/core";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  GeminiBridgeSendSingleInput,
  GeminiBrowserProviderStatus,
  GeminiBrowserRunEvent,
  GeminiBrowserRunLogSummary,
  GeminiBrowserRunResult,
} from "$lib/types/gemini-browser";

export const GEMINI_BROWSER_RUN_EVENT = "gemini-browser://run";

export function geminiBridgeStatus() {
  return invoke<GeminiBrowserProviderStatus>("gemini_bridge_status");
}

export function geminiBridgeOpenBrowser() {
  return invoke<GeminiBrowserProviderStatus>("gemini_bridge_open_browser");
}

export function geminiBridgeSendSingle(input: GeminiBridgeSendSingleInput) {
  return invoke<GeminiBrowserRunResult>("gemini_bridge_send_single", { ...input });
}

export function geminiBridgeResume() {
  return invoke<GeminiBrowserProviderStatus>("gemini_bridge_resume");
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
