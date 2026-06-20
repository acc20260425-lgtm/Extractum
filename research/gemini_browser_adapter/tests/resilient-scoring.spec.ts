import { expect, test } from "@playwright/test";
import { startMockGeminiServer } from "../mock-gemini/server.mjs";
import { sendSingleResilientScoring } from "../src/dom-contract";

let server: Awaited<ReturnType<typeof startMockGeminiServer>>;

test.beforeAll(async () => {
  server = await startMockGeminiServer();
});

test.afterAll(async () => {
  await server.stop();
});

test("resilient scoring survives wrapped DOM", async ({ page }) => {
  await page.goto(server.url("wrapped-dom"));
  const result = await sendSingleResilientScoring(page, "hello", { timeoutMs: 5_000, quietMs: 300 });
  expect(result.status).toBe("ok");
  expect(result.rawText).toContain("Mock final answer");
  expect(result.locatorAttempts.some((attempt) => attempt.strategy === "fuzzy" && attempt.matched)).toBe(true);
});

test("resilient scoring reports captcha as manual action", async ({ page }) => {
  await page.goto(server.url("captcha"));
  const result = await sendSingleResilientScoring(page, "hello", { timeoutMs: 2_000, quietMs: 300 });
  expect(result.status).toBe("captcha_required");
});

test("resilient scoring honors selector config overrides", async ({ page }) => {
  await page.setContent(`
    <main>
      <div data-custom-prompt contenteditable="true"></div>
      <button data-custom-send>Go</button>
      <section data-custom-answer></section>
      <script>
        document.querySelector('[data-custom-send]').addEventListener('click', () => {
          setTimeout(() => {
            document.querySelector('[data-custom-answer]').textContent = 'Mock final answer from config override.';
          }, 100);
        });
      </script>
    </main>
  `);

  const result = await sendSingleResilientScoring(page, "hello", {
    timeoutMs: 3_000,
    quietMs: 300,
    contractConfig: {
      promptSelectors: ["[data-custom-prompt]"],
      sendSelectors: ["[data-custom-send]"],
      answerSelectors: ["[data-custom-answer]"],
      minPromptScore: 5,
      minSendScore: 4,
    },
  });

  expect(result.status).toBe("ok");
  expect(result.rawText).toContain("config override");
  expect(result.locatorAttempts.some((attempt) => attempt.name === "config:prompt" && attempt.matched)).toBe(true);
  expect(result.locatorAttempts.some((attempt) => attempt.name === "config:send" && attempt.matched)).toBe(true);
});
