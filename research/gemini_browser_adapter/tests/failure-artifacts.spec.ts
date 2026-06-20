import { existsSync, readFileSync } from "node:fs";
import { expect, test } from "@playwright/test";
import { startMockGeminiServer } from "../mock-gemini/server.mjs";
import { probeReadyResilientScoring, sendSingleResilientScoring } from "../src/dom-contract";

let server: Awaited<ReturnType<typeof startMockGeminiServer>>;

test.beforeAll(async () => {
  server = await startMockGeminiServer();
});

test.afterAll(async () => {
  await server.stop();
});

test("timeout writes screenshot, html, and telemetry artifacts", async ({ page }) => {
  await page.goto(`${server.url("never-stable")}&authuser=0&token=secret`);
  const result = await sendSingleResilientScoring(page, "hello", {
    timeoutMs: 1_000,
    quietMs: 300,
    artifactDir: "research/gemini_browser_adapter/artifacts/test-timeout",
  });

  expect(result.status).toBe("generation_timeout");
  expect(result.artifacts?.screenshotPath).toBeTruthy();
  expect(result.artifacts?.htmlPath).toBeTruthy();
  expect(result.artifacts?.telemetryPath).toBeTruthy();
  expect(existsSync(result.artifacts!.screenshotPath!)).toBe(true);
  expect(existsSync(result.artifacts!.htmlPath!)).toBe(true);
  expect(existsSync(result.artifacts!.telemetryPath!)).toBe(true);

  const telemetry = JSON.parse(readFileSync(result.artifacts!.telemetryPath!, "utf8"));
  expect(telemetry.reason).toBe("generation_timeout");
  expect(telemetry.url).toContain("authuser=<redacted>");
  expect(telemetry.url).toContain("token=<redacted>");
  expect(Array.isArray(telemetry.locatorAttempts)).toBe(true);
});

test("failed readiness probe writes artifacts", async ({ page }) => {
  await page.goto(server.url("ready-missing-send"));
  const result = await probeReadyResilientScoring(page, {
    timeoutMs: 1_000,
    quietMs: 200,
    artifactDir: "research/gemini_browser_adapter/artifacts/test-ready-missing-send",
  });

  expect(result.status).toBe("failed");
  expect(result.errorReason).toBe("ready_contract_not_satisfied");
  expect(result.artifacts?.htmlPath).toBeTruthy();
  expect(result.artifacts?.telemetryPath).toBeTruthy();
  expect(existsSync(result.artifacts!.htmlPath!)).toBe(true);
  expect(existsSync(result.artifacts!.telemetryPath!)).toBe(true);
});

test("reduced artifact mode skips screenshot and strips visible text and form values", async ({ page }) => {
  await page.setContent(`
    <main>
      <p>Visible Secret Account Hint</p>
      <textarea>secret prompt value</textarea>
      <div contenteditable="true">private editable value</div>
    </main>
  `);

  const result = await sendSingleResilientScoring(page, "typed secret prompt", {
    timeoutMs: 500,
    quietMs: 100,
    artifactDir: "research/gemini_browser_adapter/artifacts/test-reduced",
    artifactMode: "reduced",
    contractConfig: {
      promptSelectors: ["textarea"],
      sendSelectors: ["[data-missing-send]"],
      answerSelectors: ["[data-missing-answer]"],
      minPromptScore: 5,
      minSendScore: 4,
    },
  });

  expect(result.status).toBe("failed");
  expect(result.artifacts?.screenshotPath).toBeNull();
  expect(result.artifacts?.htmlPath).toBeTruthy();
  expect(existsSync(result.artifacts!.htmlPath!)).toBe(true);

  const html = readFileSync(result.artifacts!.htmlPath!, "utf8");
  expect(html).toContain("<textarea");
  expect(html).not.toContain("Visible Secret Account Hint");
  expect(html).not.toContain("secret prompt value");
  expect(html).not.toContain("private editable value");
  expect(html).not.toContain("typed secret prompt");
});
