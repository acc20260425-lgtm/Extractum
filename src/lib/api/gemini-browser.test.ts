import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  GEMINI_BROWSER_RUN_EVENT,
  geminiBridgeListRuns,
  geminiBridgeOpenBrowser,
  geminiBridgeResume,
  geminiBridgeSendSingle,
  geminiBridgeStartCdpChrome,
  geminiBridgeStatus,
  geminiBridgeStop,
  listenToGeminiBrowserRuns,
} from "./gemini-browser";
import type { GeminiBrowserProviderConfig } from "$lib/types/gemini-browser";

const invokeMock = vi.hoisted(() => vi.fn());
const listenMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: listenMock,
}));

describe("gemini browser api wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
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

  it("lists runs and subscribes to run events", async () => {
    await geminiBridgeListRuns(5);
    expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_list_runs", { limit: 5 });

    const unlisten = vi.fn();
    const handler = vi.fn();
    listenMock.mockResolvedValueOnce(unlisten);
    await expect(listenToGeminiBrowserRuns(handler)).resolves.toBe(unlisten);
    expect(GEMINI_BROWSER_RUN_EVENT).toBe("gemini-browser://run");
    expect(listenMock).toHaveBeenCalledWith(GEMINI_BROWSER_RUN_EVENT, handler);
  });
});
