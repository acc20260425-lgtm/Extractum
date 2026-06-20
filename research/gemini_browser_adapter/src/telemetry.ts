import type { Page } from "@playwright/test";
import { redactUrl } from "./redaction";
import type { NetworkEventSummary } from "./types";

export { redactUrl } from "./redaction";

export function attachNetworkTelemetry(page: Page, events: NetworkEventSummary[]): void {
  page.on("request", (request) => {
    const url = request.url();
    if (!/gemini|google|mock-gemini/i.test(url)) return;
    events.push({
      at: Date.now(),
      kind: "request",
      method: request.method(),
      url: redactUrl(url),
    });
  });

  page.on("response", (response) => {
    const url = response.url();
    if (!/gemini|google|mock-gemini/i.test(url)) return;
    events.push({
      at: Date.now(),
      kind: "response",
      status: response.status(),
      contentType: response.headers()["content-type"],
      url: redactUrl(url),
    });
  });

  page.on("websocket", (websocket) => {
    events.push({ at: Date.now(), kind: "websocket-open", url: redactUrl(websocket.url()) });
    websocket.on("framereceived", (frame) => {
      events.push({
        at: Date.now(),
        kind: "websocket-frame-received",
        url: redactUrl(websocket.url()),
        bytes: typeof frame.payload === "string" ? frame.payload.length : frame.payload.byteLength,
      });
    });
    websocket.on("close", () => {
      events.push({ at: Date.now(), kind: "websocket-close", url: redactUrl(websocket.url()) });
    });
  });
}
