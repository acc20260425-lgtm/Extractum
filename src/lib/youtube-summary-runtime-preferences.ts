import type { GeminiBrowserProviderMode } from "./types/gemini-browser";
import type { PromptPackRuntimeProvider } from "./types/prompt-packs";

const RUNTIME_PROVIDER_KEY = "extractum.youtubeSummary.runtimeProvider";
const BROWSER_PROVIDER_MODE_KEY = "extractum.youtubeSummary.browserProviderMode";

const DEFAULT_PREFERENCES: YoutubeSummaryRuntimePreferences = {
  runtimeProvider: "api",
  browserProviderMode: "managed",
};

export interface YoutubeSummaryRuntimePreferences {
  runtimeProvider: PromptPackRuntimeProvider;
  browserProviderMode: GeminiBrowserProviderMode;
}

export interface RuntimePreferenceStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

export function loadYoutubeSummaryRuntimePreferences(
  storage: RuntimePreferenceStorage | null,
): YoutubeSummaryRuntimePreferences {
  if (!storage) {
    return { ...DEFAULT_PREFERENCES };
  }

  try {
    return {
      runtimeProvider:
        storage.getItem(RUNTIME_PROVIDER_KEY) === "gemini_browser" ? "gemini_browser" : "api",
      browserProviderMode:
        storage.getItem(BROWSER_PROVIDER_MODE_KEY) === "cdp_attach" ? "cdp_attach" : "managed",
    };
  } catch {
    return { ...DEFAULT_PREFERENCES };
  }
}

export function saveYoutubeSummaryRuntimeProvider(
  storage: RuntimePreferenceStorage,
  runtimeProvider: PromptPackRuntimeProvider,
): void {
  try {
    storage.setItem(
      RUNTIME_PROVIDER_KEY,
      runtimeProvider === "gemini_browser" ? "gemini_browser" : "api",
    );
  } catch {
    // Storage availability must not block UI behavior.
  }
}

export function saveYoutubeSummaryBrowserProviderMode(
  storage: RuntimePreferenceStorage,
  browserProviderMode: GeminiBrowserProviderMode,
): void {
  try {
    storage.setItem(
      BROWSER_PROVIDER_MODE_KEY,
      browserProviderMode === "cdp_attach" ? "cdp_attach" : "managed",
    );
  } catch {
    // Storage availability must not block UI behavior.
  }
}
