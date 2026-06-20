import { expect, test } from "@playwright/test";
import { startMockGeminiServer } from "../mock-gemini/server.mjs";

let server: Awaited<ReturnType<typeof startMockGeminiServer>>;

test.beforeAll(async () => {
  server = await startMockGeminiServer();
});

test.afterAll(async () => {
  await server.stop();
});

test("happy-path renders prompt input and final answer after submit", async ({ page }) => {
  await page.goto(server.url("happy-path"));
  await page.getByRole("textbox").fill("hello");
  await page.getByRole("button", { name: /send/i }).click();
  await expect(page.getByTestId("assistant-answer")).toContainText("Mock final answer");
});

test("ready renders an idle composer without sending", async ({ page }) => {
  await page.goto(server.url("ready"));
  await expect(page.getByRole("textbox")).toBeVisible();
  await expect(page.getByRole("button", { name: /send/i })).toBeVisible();
  await expect(page.getByTestId("assistant-answer")).toHaveText("");
});

test("ready-missing-send renders an incomplete composer", async ({ page }) => {
  await page.goto(server.url("ready-missing-send"));
  await expect(page.getByRole("textbox")).toBeVisible();
  await expect(page.getByRole("button", { name: /send/i })).toHaveCount(0);
});

test("ready-broken renders no usable composer controls", async ({ page }) => {
  await page.goto(server.url("ready-broken"));
  await expect(page.getByRole("textbox")).toHaveCount(0);
  await expect(page.getByRole("button", { name: /send/i })).toHaveCount(0);
});

test("submit emits a mock network telemetry event", async ({ page }) => {
  await page.goto(server.url("happy-path"));
  const responsePromise = page.waitForResponse((response) => response.url().includes("/mock-gemini-event"));
  await page.getByRole("textbox").fill("hello");
  await page.getByRole("button", { name: /send/i }).click();
  const response = await responsePromise;
  expect(response.status()).toBe(204);
});

test("login-required exposes a manual login state", async ({ page }) => {
  await page.goto(server.url("login-required"));
  await expect(page.getByText(/sign in/i)).toBeVisible();
});

test("rate-limit exposes a rate limit banner", async ({ page }) => {
  await page.goto(server.url("rate-limit"));
  await expect(page.getByText(/too many requests/i)).toBeVisible();
});
