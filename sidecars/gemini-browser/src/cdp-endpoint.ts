export type BrowserMode =
  | { type: "managed" }
  | { type: "cdp_attach"; rawEndpoint: string };

export type CdpEndpointValidation =
  | { ok: true; endpoint: string }
  | { ok: false; message: string };

export interface FetchResponseLike {
  ok: boolean;
  json: () => Promise<unknown>;
}

export type FetchLike = (
  input: string | URL,
  init?: { signal?: AbortSignal },
) => Promise<FetchResponseLike>;

const LOOPBACK_HOSTS = new Set(["127.0.0.1", "localhost", "[::1]"]);

export function resolveBrowserMode(env: Record<string, string | undefined>): BrowserMode {
  const raw = env.EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT?.trim();
  if (!raw) return { type: "managed" };
  return { type: "cdp_attach", rawEndpoint: raw };
}

export function validateCdpEndpoint(raw: string | undefined): CdpEndpointValidation {
  if (!raw?.trim()) {
    return { ok: false, message: "Chrome CDP endpoint is not configured." };
  }

  let url: URL;
  try {
    url = new URL(raw.trim());
  } catch {
    return { ok: false, message: "Chrome CDP endpoint must be a loopback HTTP URL." };
  }

  if (url.protocol !== "http:") {
    return { ok: false, message: "Chrome CDP endpoint must use http." };
  }
  if (url.username || url.password) {
    return { ok: false, message: "Chrome CDP endpoint must not contain credentials." };
  }
  if (!LOOPBACK_HOSTS.has(url.hostname)) {
    return { ok: false, message: "Chrome CDP endpoint must use localhost or 127.0.0.1." };
  }

  const port = Number(url.port);
  if (!Number.isInteger(port) || port <= 0 || port > 65535) {
    return { ok: false, message: "Chrome CDP endpoint must include a non-zero port." };
  }
  if (url.pathname !== "/" || url.search || url.hash) {
    return {
      ok: false,
      message: "Chrome CDP endpoint must be a base URL without path, query, or hash.",
    };
  }

  return { ok: true, endpoint: `${url.protocol}//${url.host}` };
}

export async function cdpSetupStatus(
  endpoint: string,
  fetchLike: FetchLike = fetch,
): Promise<{ ok: true; message: string } | { ok: false; message: string }> {
  const validation = validateCdpEndpoint(endpoint);
  if (!validation.ok) {
    return { ok: false, message: validation.message };
  }

  try {
    const controller = new AbortController();
    let timeout: ReturnType<typeof setTimeout> | null = null;
    try {
      const versionUrl = new URL("/json/version", validation.endpoint);
      const response = await Promise.race([
        fetchLike(versionUrl, { signal: controller.signal }),
        new Promise<never>((_, reject) => {
          timeout = setTimeout(() => {
            controller.abort();
            reject(new Error("Chrome CDP endpoint probe timed out."));
          }, 1_500);
        }),
      ]);

      if (!response.ok) {
        return {
          ok: false,
          message: "Chrome CDP endpoint did not expose a Chrome debugging protocol.",
        };
      }

      let payload: unknown;
      try {
        payload = await response.json();
      } catch {
        return {
          ok: false,
          message: "Chrome CDP endpoint did not expose a Chrome debugging protocol.",
        };
      }

      if (isChromeVersionPayload(payload)) {
        return { ok: true, message: "Chrome CDP endpoint is reachable." };
      }
      return {
        ok: false,
        message: "Chrome CDP endpoint did not expose a Chrome debugging protocol.",
      };
    } finally {
      if (timeout) clearTimeout(timeout);
    }
  } catch {
    return {
      ok: false,
      message: "Chrome CDP endpoint is unavailable. Start Chrome with remote debugging enabled.",
    };
  }
}

function isChromeVersionPayload(value: unknown): value is {
  Browser?: string;
  webSocketDebuggerUrl?: string;
} {
  if (!value || typeof value !== "object") return false;
  const payload = value as { Browser?: unknown; webSocketDebuggerUrl?: unknown };
  return typeof payload.Browser === "string" || typeof payload.webSocketDebuggerUrl === "string";
}
