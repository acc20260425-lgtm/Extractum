import { describe, expect, it } from "vitest";

import {
  loadYoutubeSummaryRuntimePreferences,
  saveYoutubeSummaryBrowserProviderMode,
  saveYoutubeSummaryRuntimeProvider,
  type RuntimePreferenceStorage,
} from "./youtube-summary-runtime-preferences";

class MemoryStorage implements RuntimePreferenceStorage {
  readonly values = new Map<string, string>();
  readonly writtenKeys: string[] = [];

  getItem(key: string): string | null {
    return this.values.get(key) ?? null;
  }

  setItem(key: string, value: string): void {
    this.writtenKeys.push(key);
    this.values.set(key, value);
  }
}

describe("youtube summary runtime preferences", () => {
  it("returns safe defaults without storage", () => {
    expect(loadYoutubeSummaryRuntimePreferences(null)).toEqual({
      runtimeProvider: "api",
      browserProviderMode: "managed",
    });
  });

  it("restores supported preferences from their scoped keys", () => {
    const storage = new MemoryStorage();
    storage.values.set("extractum.youtubeSummary.runtimeProvider", "gemini_browser");
    storage.values.set("extractum.youtubeSummary.browserProviderMode", "cdp_attach");

    expect(loadYoutubeSummaryRuntimePreferences(storage)).toEqual({
      runtimeProvider: "gemini_browser",
      browserProviderMode: "cdp_attach",
    });
  });

  it("normalizes missing and unsupported stored preferences", () => {
    const storage = new MemoryStorage();
    storage.values.set("extractum.youtubeSummary.runtimeProvider", "unsupported");
    storage.values.set("extractum.youtubeSummary.browserProviderMode", "unsupported");

    expect(loadYoutubeSummaryRuntimePreferences(storage)).toEqual({
      runtimeProvider: "api",
      browserProviderMode: "managed",
    });
  });

  it("returns safe defaults when storage reads throw", () => {
    const storage: RuntimePreferenceStorage = {
      getItem() {
        throw new Error("read failed");
      },
      setItem() {},
    };

    expect(loadYoutubeSummaryRuntimePreferences(storage)).toEqual({
      runtimeProvider: "api",
      browserProviderMode: "managed",
    });
  });

  it("saves normalized preferences through the exact scoped keys", () => {
    const storage = new MemoryStorage();

    saveYoutubeSummaryRuntimeProvider(storage, "gemini_browser");
    saveYoutubeSummaryBrowserProviderMode(storage, "cdp_attach");

    expect(storage.writtenKeys).toEqual([
      "extractum.youtubeSummary.runtimeProvider",
      "extractum.youtubeSummary.browserProviderMode",
    ]);
    expect(storage.values.get("extractum.youtubeSummary.runtimeProvider")).toBe("gemini_browser");
    expect(storage.values.get("extractum.youtubeSummary.browserProviderMode")).toBe("cdp_attach");
  });

  it("ignores storage write failures", () => {
    const storage: RuntimePreferenceStorage = {
      getItem() {
        return null;
      },
      setItem() {
        throw new Error("write failed");
      },
    };

    expect(() => saveYoutubeSummaryRuntimeProvider(storage, "api")).not.toThrow();
    expect(() => saveYoutubeSummaryBrowserProviderMode(storage, "managed")).not.toThrow();
  });
});
