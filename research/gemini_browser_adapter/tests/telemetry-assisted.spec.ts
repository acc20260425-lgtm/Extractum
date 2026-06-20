import { expect, test } from "@playwright/test";
import { startMockGeminiServer } from "../mock-gemini/server.mjs";
import { sendSingleTelemetryAssisted } from "../src/dom-contract";

let server: Awaited<ReturnType<typeof startMockGeminiServer>>;

test.beforeAll(async () => {
  server = await startMockGeminiServer();
});

test.afterAll(async () => {
  await server.stop();
});

test("telemetry-assisted variant records mock network events", async ({ page }) => {
  await page.goto(server.url("happy-path"));
  const result = await sendSingleTelemetryAssisted(page, "hello", { timeoutMs: 5_000, quietMs: 300 });
  expect(result.status).toBe("ok");
  expect(result.networkSummary.length).toBeGreaterThan(0);
  expect(result.networkSummary.some((event) => event.kind === "response")).toBe(true);
});

test("telemetry-assisted variant reports rate limit", async ({ page }) => {
  await page.goto(server.url("rate-limit"));
  const result = await sendSingleTelemetryAssisted(page, "hello", { timeoutMs: 2_000, quietMs: 300 });
  expect(result.status).toBe("rate_limited");
});
