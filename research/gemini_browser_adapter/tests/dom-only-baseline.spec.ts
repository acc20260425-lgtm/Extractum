import { expect, test } from "@playwright/test";
import { startMockGeminiServer } from "../mock-gemini/server.mjs";
import { sendSingleDomOnly } from "../src/dom-contract";

let server: Awaited<ReturnType<typeof startMockGeminiServer>>;

test.beforeAll(async () => {
  server = await startMockGeminiServer();
});

test.afterAll(async () => {
  await server.stop();
});

test("dom-only baseline captures happy-path answer", async ({ page }) => {
  await page.goto(server.url("happy-path"));
  const result = await sendSingleDomOnly(page, "hello", { timeoutMs: 5_000, quietMs: 300 });
  expect(result.status).toBe("ok");
  expect(result.rawText).toContain("Mock final answer");
});

test("dom-only baseline handles textarea input", async ({ page }) => {
  await page.goto(server.url("textarea-input"));
  const result = await sendSingleDomOnly(page, "hello", { timeoutMs: 5_000, quietMs: 300 });
  expect(result.status).toBe("ok");
  expect(result.rawText).toContain("Mock final answer");
});

test("dom-only baseline reports login-required state", async ({ page }) => {
  await page.goto(server.url("login-required"));
  const result = await sendSingleDomOnly(page, "hello", { timeoutMs: 2_000, quietMs: 300 });
  expect(result.status).toBe("login_required");
  expect(result.rawText).toBeNull();
});

test("dom-only baseline reports generation timeout", async ({ page }) => {
  await page.goto(server.url("never-stable"));
  const result = await sendSingleDomOnly(page, "hello", { timeoutMs: 1_000, quietMs: 300 });
  expect(result.status).toBe("generation_timeout");
});
