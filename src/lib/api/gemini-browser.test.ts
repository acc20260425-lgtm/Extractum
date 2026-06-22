import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  geminiBridgeGetRun,
  geminiBridgeListRuns,
  geminiBridgeOpenBrowser,
  geminiBridgeOpenRunFolder,
  geminiBridgeResume,
  geminiBridgeSendSingle,
  geminiBridgeStartCdpChrome,
  geminiBridgeStatus,
  geminiBridgeStatusSnapshot,
  geminiBridgeStop,
  isGeminiBrowserRunNotFoundError,
} from "./gemini-browser";
import type { GeminiBrowserProviderConfig } from "$lib/types/gemini-browser";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

describe("gemini browser api wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("wraps provider commands with stable command names", async () => {
    await geminiBridgeStatus();
    expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_status");

    await geminiBridgeOpenBrowser();
    expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_open_browser");

    await geminiBridgeResume();
    expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_resume");

    await geminiBridgeStop();
    expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_stop");

    await geminiBridgeOpenRunFolder("run-1");
    expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_open_run_folder", {
      runId: "run-1",
    });
  });

  it("forwards browser provider config to provider commands", async () => {
    const browserConfig: GeminiBrowserProviderConfig = {
      mode: "cdp_attach",
      cdpEndpoint: "http://127.0.0.1:9222",
    };

    await geminiBridgeStatus(browserConfig);
    expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_status", { browserConfig });

    await geminiBridgeOpenBrowser(browserConfig);
    expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_open_browser", { browserConfig });

    await geminiBridgeResume(browserConfig);
    expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_resume", { browserConfig });

    await geminiBridgeStartCdpChrome(browserConfig);
    expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_start_cdp_chrome", {
      browserConfig,
    });
  });

  it("sends single prompt with camelCase frontend keys", async () => {
    await geminiBridgeSendSingle({
      runId: "run-1",
      prompt: "hello",
      source: "settings_test",
      artifactMode: "reduced",
      browserConfig: {
        mode: "cdp_attach",
        cdpEndpoint: "http://127.0.0.1:9222",
      },
    });

    expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_send_single", {
      runId: "run-1",
      prompt: "hello",
      source: "settings_test",
      artifactMode: "reduced",
      browserConfig: {
        mode: "cdp_attach",
        cdpEndpoint: "http://127.0.0.1:9222",
      },
    });
  });

  it("wraps pull read-model commands", async () => {
    await geminiBridgeStatusSnapshot();
    expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_status_snapshot");

    await geminiBridgeListRuns(5);
    expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_list_runs", { limit: 5 });

    await geminiBridgeGetRun("run-1");
    expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_get_run", { runId: "run-1" });
  });

  it("detects typed not-found run detail errors", () => {
    expect(isGeminiBrowserRunNotFoundError({ kind: "not_found", message: "missing" })).toBe(true);
    expect(isGeminiBrowserRunNotFoundError({ kind: "network", message: "not found upstream" })).toBe(false);
    expect(isGeminiBrowserRunNotFoundError(new Error("not found text only"))).toBe(false);
  });

  it("does not expose Gemini Browser run event public names", async () => {
    const api = await import("./gemini-browser");
    expect("listenToGeminiBrowserRuns" in api).toBe(false);
    expect("GEMINI_BROWSER_RUN_EVENT" in api).toBe(false);
    expect("listenToGeminiBrowserRunChanges" in api).toBe(false);
    expect("GEMINI_BROWSER_RUN_CHANGE_EVENT" in api).toBe(false);
  });
});
