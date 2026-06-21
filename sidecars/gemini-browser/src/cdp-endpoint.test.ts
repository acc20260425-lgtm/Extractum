import { describe, expect, it, vi } from "vitest";
import {
  cdpSetupStatus,
  resolveBrowserMode,
  validateCdpEndpoint,
  type FetchLike,
} from "./cdp-endpoint.js";

describe("CDP endpoint validation", () => {
  it("keeps managed mode when the CDP endpoint env var is absent", () => {
    expect(resolveBrowserMode({})).toEqual({ type: "managed" });
  });

  it("uses command browser config before env fallback", () => {
    expect(
      resolveBrowserMode(
        { EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT: "http://127.0.0.1:9222" },
        { mode: "managed", cdp_endpoint: null },
      ),
    ).toEqual({ type: "managed" });

    expect(resolveBrowserMode({}, { mode: "cdp_attach", cdp_endpoint: null })).toEqual({
      type: "cdp_attach",
      rawEndpoint: "http://127.0.0.1:9222",
    });
  });

  it("keeps the configured CDP endpoint as raw operator input", () => {
    expect(
      resolveBrowserMode({
        EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT: " http://127.0.0.1:9222 ",
      }),
    ).toEqual({ type: "cdp_attach", rawEndpoint: "http://127.0.0.1:9222" });

    expect(
      resolveBrowserMode({
        EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT: "http://192.168.1.20:9222",
      }),
    ).toEqual({ type: "cdp_attach", rawEndpoint: "http://192.168.1.20:9222" });
  });

  it("accepts only base loopback HTTP endpoints with a non-zero port", () => {
    expect(validateCdpEndpoint("http://127.0.0.1:9222")).toEqual({
      ok: true,
      endpoint: "http://127.0.0.1:9222",
    });
    expect(validateCdpEndpoint("http://localhost:9222")).toEqual({
      ok: true,
      endpoint: "http://localhost:9222",
    });
    expect(validateCdpEndpoint("http://[::1]:9222")).toEqual({
      ok: true,
      endpoint: "http://[::1]:9222",
    });
  });

  it("rejects remote, unspecified, malformed, credentialed, and non-base endpoints", () => {
    const invalid = [
      "http://192.168.1.20:9222",
      "http://0.0.0.0:9222",
      "http://127.0.0.1:0",
      "http://127.0.0.1:9222/json/version",
      "http://127.0.0.1:9222?token=x",
      "http://user:pass@127.0.0.1:9222",
      "https://127.0.0.1:9222",
      "127.0.0.1:9222",
      "http://chrome.local:9222",
    ];

    for (const value of invalid) {
      expect(validateCdpEndpoint(value), value).toMatchObject({ ok: false });
    }
  });
});

describe("CDP status probe", () => {
  it("reports reachable Chrome debugging endpoint", async () => {
    const fetchMock = vi.fn<FetchLike>(async () => ({
      ok: true,
      json: async () => ({
        Browser: "Chrome/126",
        webSocketDebuggerUrl: "ws://127.0.0.1:9222/devtools/browser/id",
      }),
    }));

    await expect(cdpSetupStatus("http://127.0.0.1:9222", fetchMock)).resolves.toEqual({
      ok: true,
      message: "Chrome CDP endpoint is reachable.",
    });
  });

  it("reports non-Chrome or incompatible endpoint as operator setup action", async () => {
    const fetchMock = vi.fn<FetchLike>(async () => ({
      ok: true,
      json: async () => ({ hello: "world" }),
    }));

    await expect(cdpSetupStatus("http://127.0.0.1:9222", fetchMock)).resolves.toEqual({
      ok: false,
      message: "Chrome CDP endpoint did not expose a Chrome debugging protocol.",
    });
  });

  it("reports invalid /json/version JSON as non-Chrome protocol", async () => {
    const fetchMock = vi.fn<FetchLike>(async () => ({
      ok: true,
      json: async () => {
        throw new Error("invalid json");
      },
    }));

    await expect(cdpSetupStatus("http://127.0.0.1:9222", fetchMock)).resolves.toEqual({
      ok: false,
      message: "Chrome CDP endpoint did not expose a Chrome debugging protocol.",
    });
  });

  it("reports unreachable endpoint as operator setup action", async () => {
    const fetchMock = vi.fn<FetchLike>(async () => {
      throw new Error("ECONNREFUSED");
    });

    await expect(cdpSetupStatus("http://127.0.0.1:9222", fetchMock)).resolves.toEqual({
      ok: false,
      message: "Chrome CDP endpoint is unavailable. Start Chrome with remote debugging enabled.",
    });
  });

  it("times out endpoint probe even if fetch never settles", async () => {
    vi.useFakeTimers();
    try {
      const fetchMock = vi.fn<FetchLike>(() => new Promise(() => undefined));
      const probe = cdpSetupStatus("http://127.0.0.1:9222", fetchMock);

      await vi.advanceTimersByTimeAsync(1_500);

      await expect(probe).resolves.toEqual({
        ok: false,
        message: "Chrome CDP endpoint is unavailable. Start Chrome with remote debugging enabled.",
      });
    } finally {
      vi.useRealTimers();
    }
  });
});
