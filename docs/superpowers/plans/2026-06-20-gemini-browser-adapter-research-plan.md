# Gemini Browser Adapter Research Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a local, reproducible research harness for the Gemini browser adapter and compare three TypeScript/Node adapter variants against deterministic mock Gemini pages.

**Architecture:** Keep the production app out of this research slice. Add a local mock Gemini server, Playwright tests, TypeScript adapter modules, diagnostic artifact helpers, telemetry helpers, and a matrix report runner under `research/gemini_browser_adapter`. Rust/Tauri integration is not part of this plan.

**Tech Stack:** TypeScript, Node ESM, Vitest, Playwright, local HTTP mock server, existing npm scripts.

---

## File Structure

Create and modify only research/tooling files:

- Modify: `package.json` to add Playwright dependency and research scripts.
- Create: `research/gemini_browser_adapter/playwright.config.ts` for adapter e2e tests.
- Create: `research/gemini_browser_adapter/mock-gemini/variants.mjs` to render deterministic mock Gemini pages.
- Create: `research/gemini_browser_adapter/mock-gemini/server.mjs` to start and stop a local mock server.
- Create: `research/gemini_browser_adapter/src/types.ts` for shared status/result/artifact types.
- Create: `research/gemini_browser_adapter/src/status.test.ts` for type and status guard tests.
- Create: `research/gemini_browser_adapter/src/dom-contract.ts` for adapter variant implementations.
- Create: `research/gemini_browser_adapter/tests/dom-only-baseline.spec.ts` for baseline Playwright tests.
- Create: `research/gemini_browser_adapter/src/scoring.ts` for deterministic locator scoring.
- Create: `research/gemini_browser_adapter/src/scoring.test.ts` for scoring unit tests.
- Create: `research/gemini_browser_adapter/tests/resilient-scoring.spec.ts` for scoring adapter tests.
- Create: `research/gemini_browser_adapter/src/artifacts.ts` for failure bundle writing.
- Create: `research/gemini_browser_adapter/tests/failure-artifacts.spec.ts` for artifact expectations.
- Create: `research/gemini_browser_adapter/src/telemetry.ts` for sanitized network telemetry.
- Create: `research/gemini_browser_adapter/src/telemetry.test.ts` for redaction/unit tests.
- Create: `research/gemini_browser_adapter/tests/telemetry-assisted.spec.ts` for telemetry-assisted behavior.
- Create: `research/gemini_browser_adapter/scripts/write-matrix-report.mjs` for report generation.
- Modify: `research/gemini_browser_adapter/RESILIENCE_TEST_MATRIX.md` to add the executable command and artifact paths.
- Modify: `research/gemini_browser_adapter/TOOLS_AND_METHODS.md` to reference the matrix runner.

Do not touch `src-tauri/*` in this plan.

---

### Task 1: Research Test Tooling

**Files:**
- Modify: `package.json`
- Create: `research/gemini_browser_adapter/playwright.config.ts`

- [ ] **Step 1: Add Playwright test dependency**

Run:

```powershell
npm install --save-dev @playwright/test
```

Expected:

- `package.json` gains `@playwright/test` under `devDependencies`.
- `package-lock.json` updates.
- If the sandbox blocks registry access, rerun this command with escalation.

- [ ] **Step 2: Install Chromium browser for local Playwright runs**

Run:

```powershell
npx playwright install chromium
```

Expected: Playwright downloads or confirms the Chromium browser binary.

- [ ] **Step 3: Add package scripts**

Edit `package.json` and add these scripts inside `"scripts"`:

```json
"test:gemini-browser-adapter:unit": "node scripts/run-vitest.mjs run research/gemini_browser_adapter/**/*.test.ts",
"test:gemini-browser-adapter:e2e": "playwright test -c research/gemini_browser_adapter/playwright.config.ts",
"test:gemini-browser-adapter:report": "node research/gemini_browser_adapter/scripts/write-matrix-report.mjs",
"test:gemini-browser-adapter": "npm run test:gemini-browser-adapter:unit && npm run test:gemini-browser-adapter:e2e && npm run test:gemini-browser-adapter:report"
```

- [ ] **Step 4: Create Playwright config**

Create `research/gemini_browser_adapter/playwright.config.ts`:

```ts
import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  timeout: 30_000,
  expect: {
    timeout: 5_000,
  },
  fullyParallel: false,
  reporter: [
    ["list"],
    [
      "json",
      {
        outputFile: "research/gemini_browser_adapter/artifacts/playwright-results.json",
      },
    ],
  ],
  use: {
    browserName: "chromium",
    headless: true,
    screenshot: "only-on-failure",
    trace: "retain-on-failure",
  },
});
```

- [ ] **Step 5: Verify config parses**

Run:

```powershell
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts --list
```

Expected: command exits `0` and prints no tests yet, or a valid empty test listing.

- [ ] **Step 6: Commit tooling**

Run:

```powershell
git add package.json package-lock.json research/gemini_browser_adapter/playwright.config.ts docs/superpowers/plans/2026-06-20-gemini-browser-adapter-research-plan.md
git commit -m "Add Gemini adapter research test tooling"
```

---

### Task 2: Mock Gemini Server

**Files:**
- Create: `research/gemini_browser_adapter/mock-gemini/variants.mjs`
- Create: `research/gemini_browser_adapter/mock-gemini/server.mjs`
- Create: `research/gemini_browser_adapter/tests/mock-gemini.spec.ts`

- [ ] **Step 1: Write failing Playwright tests for mock variants**

Create `research/gemini_browser_adapter/tests/mock-gemini.spec.ts`:

```ts
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

test("login-required exposes a manual login state", async ({ page }) => {
  await page.goto(server.url("login-required"));
  await expect(page.getByText(/sign in/i)).toBeVisible();
});

test("rate-limit exposes a rate limit banner", async ({ page }) => {
  await page.goto(server.url("rate-limit"));
  await expect(page.getByText(/too many requests/i)).toBeVisible();
});
```

- [ ] **Step 2: Run failing mock tests**

Run:

```powershell
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/mock-gemini.spec.ts
```

Expected: FAIL because `mock-gemini/server.mjs` does not exist.

- [ ] **Step 3: Implement mock variants**

Create `research/gemini_browser_adapter/mock-gemini/variants.mjs`:

```js
const ANSWER_TEXT = "Mock final answer from Gemini-like page.";

function basePage(body, script = "") {
  return `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Mock Gemini</title>
  <style>
    body { font-family: system-ui, sans-serif; margin: 24px; }
    main { display: grid; gap: 16px; max-width: 760px; }
    .composer { display: flex; gap: 8px; align-items: center; }
    [role="textbox"], textarea { min-width: 420px; min-height: 40px; }
    .answer { white-space: pre-wrap; border: 1px solid #bbb; padding: 12px; }
    .banner { border: 1px solid #a55; color: #900; padding: 12px; }
  </style>
</head>
<body>
  <main>${body}</main>
  <script>${script}</script>
</body>
</html>`;
}

const submitScript = `
const input = document.querySelector('[role="textbox"], textarea, [contenteditable="true"]');
const button = document.querySelector('[data-send]');
const answer = document.querySelector('[data-testid="assistant-answer"]');
const stop = document.querySelector('[data-testid="stop-control"]');
button?.addEventListener('click', () => {
  if (stop) stop.hidden = false;
  if (input) input.setAttribute('aria-disabled', 'true');
  const chunks = ['Mock ', 'final ', 'answer ', 'from Gemini-like page.'];
  let index = 0;
  const tick = () => {
    answer.textContent += chunks[index] || '';
    index += 1;
    if (index < chunks.length) {
      setTimeout(tick, 120);
      return;
    }
    if (stop) stop.hidden = true;
    if (input) input.removeAttribute('aria-disabled');
  };
  setTimeout(tick, 120);
});
`;

const slowPauseScript = `
const input = document.querySelector('[role="textbox"]');
const button = document.querySelector('[data-send]');
const answer = document.querySelector('[data-testid="assistant-answer"]');
const stop = document.querySelector('[data-testid="stop-control"]');
button?.addEventListener('click', () => {
  stop.hidden = false;
  input.setAttribute('aria-disabled', 'true');
  answer.textContent = 'Mock ';
  setTimeout(() => { answer.textContent += 'final '; }, 1200);
  setTimeout(() => {
    answer.textContent += 'answer after pause.';
    stop.hidden = true;
    input.removeAttribute('aria-disabled');
  }, 2500);
});
`;

const neverStableScript = `
const button = document.querySelector('[data-send]');
const answer = document.querySelector('[data-testid="assistant-answer"]');
button?.addEventListener('click', () => {
  let count = 0;
  setInterval(() => {
    count += 1;
    answer.textContent = 'still generating ' + count;
  }, 150);
});
`;

export function renderMockGeminiPage(variant = "happy-path") {
  if (variant === "login-required") {
    return basePage('<section class="banner"><h1>Sign in</h1><p>Sign in to continue to Gemini.</p></section>');
  }

  if (variant === "captcha") {
    return basePage('<section class="banner"><h1>Verify you are human</h1><p>CAPTCHA required.</p></section>');
  }

  if (variant === "account-picker") {
    return basePage('<section class="banner"><h1>Choose an account</h1><p>Select an account to continue.</p></section>');
  }

  if (variant === "consent") {
    return basePage('<section class="banner"><h1>Before you continue</h1><p>Review privacy and terms.</p></section>');
  }

  if (variant === "rate-limit") {
    return basePage('<section class="banner"><h1>Too many requests</h1><p>Try again later.</p></section>');
  }

  if (variant === "unknown-modal") {
    return basePage('<div role="dialog" aria-label="Gemini notice"><p>Manual review required.</p></div>');
  }

  if (variant === "textarea-input") {
    return basePage(`
      <section class="composer">
        <textarea placeholder="Ask Gemini"></textarea>
        <button data-send aria-label="Send">Send</button>
      </section>
      <article class="answer" data-testid="assistant-answer"></article>
      <button data-testid="stop-control" aria-label="Stop" hidden>Stop</button>
    `, submitScript);
  }

  if (variant === "wrapped-dom") {
    return basePage(`
      <section><div><div><form class="composer">
        <div role="textbox" contenteditable="true" aria-label="Ask Gemini"></div>
        <span role="button" data-send aria-label="Send message">Send</span>
      </form></div></div></section>
      <section><article class="answer" data-testid="assistant-answer"></article></section>
      <button data-testid="stop-control" aria-label="Stop" hidden>Stop</button>
    `, submitScript);
  }

  if (variant === "contenteditable-input") {
    return basePage(`
      <section class="composer">
        <div contenteditable="true" role="textbox" aria-label="Message Gemini"></div>
        <button data-send title="Send">Send</button>
      </section>
      <article class="answer" data-testid="assistant-answer"></article>
      <button data-testid="stop-control" aria-label="Stop" hidden>Stop</button>
    `, submitScript);
  }

  if (variant === "icon-send") {
    return basePage(`
      <section class="composer">
        <div role="textbox" contenteditable="true" aria-label="Ask Gemini"></div>
        <button data-send title="Send"><span aria-hidden="true">↑</span></button>
      </section>
      <article class="answer" data-testid="assistant-answer"></article>
      <button data-testid="stop-control" aria-label="Stop" hidden>Stop</button>
    `, submitScript);
  }

  if (variant === "slow-pauses") {
    return basePage(`
      <section class="composer">
        <div role="textbox" contenteditable="true" aria-label="Ask Gemini"></div>
        <button data-send aria-label="Send">Send</button>
      </section>
      <article class="answer" data-testid="assistant-answer"></article>
      <button data-testid="stop-control" aria-label="Stop" hidden>Stop</button>
    `, slowPauseScript);
  }

  if (variant === "never-stable") {
    return basePage(`
      <section class="composer">
        <div role="textbox" contenteditable="true" aria-label="Ask Gemini"></div>
        <button data-send aria-label="Send">Send</button>
      </section>
      <article class="answer" data-testid="assistant-answer"></article>
      <button data-testid="stop-control" aria-label="Stop">Stop</button>
    `, neverStableScript);
  }

  if (variant === "broken-answer") {
    return basePage(`
      <section class="composer">
        <div role="textbox" contenteditable="true" aria-label="Ask Gemini"></div>
        <button data-send aria-label="Send">Send</button>
      </section>
    `, "");
  }

  return basePage(`
    <section class="composer">
      <div role="textbox" contenteditable="true" aria-label="Ask Gemini"></div>
      <button data-send aria-label="Send">Send</button>
    </section>
    <article class="answer" data-testid="assistant-answer"></article>
    <button data-testid="stop-control" aria-label="Stop" hidden>Stop</button>
  `, submitScript);
}

export { ANSWER_TEXT };
```

- [ ] **Step 4: Implement mock server**

Create `research/gemini_browser_adapter/mock-gemini/server.mjs`:

```js
import http from "node:http";
import { renderMockGeminiPage } from "./variants.mjs";

export async function startMockGeminiServer() {
  const server = http.createServer((request, response) => {
    const url = new URL(request.url || "/", "http://127.0.0.1");
    if (url.pathname !== "/mock-gemini") {
      response.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
      response.end("not found");
      return;
    }

    const variant = url.searchParams.get("variant") || "happy-path";
    response.writeHead(200, { "content-type": "text/html; charset=utf-8" });
    response.end(renderMockGeminiPage(variant));
  });

  await new Promise((resolve) => {
    server.listen(0, "127.0.0.1", resolve);
  });

  const address = server.address();
  if (!address || typeof address === "string") {
    throw new Error("mock_server_address_unavailable");
  }

  return {
    port: address.port,
    url(variant) {
      return `http://127.0.0.1:${address.port}/mock-gemini?variant=${encodeURIComponent(variant)}`;
    },
    async stop() {
      await new Promise((resolve, reject) => {
        server.close((error) => {
          if (error) reject(error);
          else resolve();
        });
      });
    },
  };
}
```

- [ ] **Step 5: Verify mock tests pass**

Run:

```powershell
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/mock-gemini.spec.ts
```

Expected: PASS for 3 tests.

- [ ] **Step 6: Commit mock server**

Run:

```powershell
git add research/gemini_browser_adapter/mock-gemini/variants.mjs research/gemini_browser_adapter/mock-gemini/server.mjs research/gemini_browser_adapter/tests/mock-gemini.spec.ts
git commit -m "Add mock Gemini research server"
```

---

### Task 3: Shared Result Types

**Files:**
- Create: `research/gemini_browser_adapter/src/types.ts`
- Create: `research/gemini_browser_adapter/src/status.test.ts`

- [ ] **Step 1: Write failing unit tests for status guards**

Create `research/gemini_browser_adapter/src/status.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { isTerminalStatus, isManualActionStatus } from "./types";

describe("Gemini adapter status helpers", () => {
  it("detects terminal statuses", () => {
    expect(isTerminalStatus("ok")).toBe(true);
    expect(isTerminalStatus("generation_timeout")).toBe(true);
    expect(isTerminalStatus("running")).toBe(false);
  });

  it("detects manual-action statuses", () => {
    expect(isManualActionStatus("login_required")).toBe(true);
    expect(isManualActionStatus("manual_action_required")).toBe(true);
    expect(isManualActionStatus("rate_limited")).toBe(false);
  });
});
```

- [ ] **Step 2: Run unit test to verify it fails**

Run:

```powershell
npm run test:gemini-browser-adapter:unit -- research/gemini_browser_adapter/src/status.test.ts
```

Expected: FAIL because `types.ts` does not exist.

- [ ] **Step 3: Implement shared types**

Create `research/gemini_browser_adapter/src/types.ts`:

```ts
export type AdapterVariant = "dom-only" | "resilient-scoring" | "telemetry-assisted";

export type GeminiAdapterStatus =
  | "ready"
  | "running"
  | "ok"
  | "login_required"
  | "manual_action_required"
  | "captcha_required"
  | "account_picker"
  | "consent_required"
  | "rate_limited"
  | "generation_timeout"
  | "response_parse_failed"
  | "browser_crashed"
  | "failed";

export type LocatorAttempt = {
  name: string;
  strategy: "role" | "label" | "placeholder" | "text" | "css" | "structural" | "fuzzy";
  matched: boolean;
  count?: number;
  error?: string;
  score?: number;
};

export type NetworkEventSummary = {
  at: number;
  kind: "request" | "response" | "websocket-open" | "websocket-frame-received" | "websocket-close";
  url: string;
  method?: string;
  status?: number;
  contentType?: string;
  bytes?: number;
};

export type FailureArtifacts = {
  screenshotPath: string | null;
  htmlPath: string | null;
  telemetryPath: string | null;
  tracePath: string | null;
};

export type GeminiAdapterResult = {
  variant: AdapterVariant;
  status: GeminiAdapterStatus;
  rawText: string | null;
  elapsedMs: number;
  locatorAttempts: LocatorAttempt[];
  networkSummary: NetworkEventSummary[];
  artifacts: FailureArtifacts | null;
  errorReason: string | null;
};

export function isTerminalStatus(status: GeminiAdapterStatus): boolean {
  return status !== "ready" && status !== "running";
}

export function isManualActionStatus(status: GeminiAdapterStatus): boolean {
  return (
    status === "login_required" ||
    status === "manual_action_required" ||
    status === "captcha_required" ||
    status === "account_picker" ||
    status === "consent_required"
  );
}
```

- [ ] **Step 4: Verify unit test passes**

Run:

```powershell
npm run test:gemini-browser-adapter:unit -- research/gemini_browser_adapter/src/status.test.ts
```

Expected: PASS for 2 tests.

- [ ] **Step 5: Commit shared types**

Run:

```powershell
git add research/gemini_browser_adapter/src/types.ts research/gemini_browser_adapter/src/status.test.ts
git commit -m "Add Gemini adapter research result types"
```

---

### Task 4: DOM-Only Baseline Adapter

**Files:**
- Create: `research/gemini_browser_adapter/src/dom-contract.ts`
- Create: `research/gemini_browser_adapter/tests/dom-only-baseline.spec.ts`

- [ ] **Step 1: Write failing baseline e2e tests**

Create `research/gemini_browser_adapter/tests/dom-only-baseline.spec.ts`:

```ts
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
```

- [ ] **Step 2: Run baseline tests to verify failure**

Run:

```powershell
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/dom-only-baseline.spec.ts
```

Expected: FAIL because `dom-contract.ts` does not exist.

- [ ] **Step 3: Implement DOM-only adapter**

Create `research/gemini_browser_adapter/src/dom-contract.ts`:

```ts
import type { Locator, Page } from "@playwright/test";
import type { GeminiAdapterResult, GeminiAdapterStatus, LocatorAttempt } from "./types";

export type SendSingleOptions = {
  timeoutMs: number;
  quietMs: number;
};

function emptyArtifacts() {
  return null;
}

function result(
  status: GeminiAdapterStatus,
  startedAt: number,
  rawText: string | null,
  locatorAttempts: LocatorAttempt[],
  errorReason: string | null,
): GeminiAdapterResult {
  return {
    variant: "dom-only",
    status,
    rawText,
    elapsedMs: Date.now() - startedAt,
    locatorAttempts,
    networkSummary: [],
    artifacts: emptyArtifacts(),
    errorReason,
  };
}

async function firstVisible(locator: Locator): Promise<Locator | null> {
  const count = await locator.count().catch(() => 0);
  for (let index = 0; index < count; index += 1) {
    const candidate = locator.nth(index);
    if (await candidate.isVisible().catch(() => false)) {
      return candidate;
    }
  }
  return null;
}

export async function scanCriticalState(page: Page): Promise<GeminiAdapterStatus | null> {
  const bodyText = await page.locator("body").innerText({ timeout: 500 }).catch(() => "");
  const url = page.url();
  if (/accounts\.google\.com|signin|sign in/i.test(`${url}\n${bodyText}`)) return "login_required";
  if (/captcha|verify you are human|robot/i.test(bodyText)) return "captcha_required";
  if (/choose an account|select an account/i.test(bodyText)) return "account_picker";
  if (/before you continue|privacy|terms|consent/i.test(bodyText)) return "consent_required";
  if (/too many requests|rate limit|try again later/i.test(bodyText)) return "rate_limited";
  const dialogVisible = await page.getByRole("dialog").first().isVisible().catch(() => false);
  if (dialogVisible) return "manual_action_required";
  return null;
}

export async function findPromptBox(page: Page, attempts: LocatorAttempt[]): Promise<Locator | null> {
  const candidates: Array<[LocatorAttempt["strategy"], string, Locator]> = [
    ["role", "role:textbox", page.getByRole("textbox")],
    ["label", "label:prompt", page.getByLabel(/prompt|message|ask|gemini/i)],
    ["placeholder", "placeholder:prompt", page.getByPlaceholder(/ask|message|prompt|gemini/i)],
    ["css", "css:textarea", page.locator("textarea")],
    ["css", "css:contenteditable", page.locator('[contenteditable="true"]')],
  ];

  for (const [strategy, name, locator] of candidates) {
    const count = await locator.count().catch(() => 0);
    const visible = await firstVisible(locator);
    attempts.push({ name, strategy, matched: Boolean(visible), count });
    if (visible) return visible;
  }
  return null;
}

export async function findSendButton(page: Page, attempts: LocatorAttempt[]): Promise<Locator | null> {
  const candidates: Array<[LocatorAttempt["strategy"], string, Locator]> = [
    ["role", "role:send", page.getByRole("button", { name: /send|submit/i })],
    ["css", "css:data-send", page.locator("[data-send]")],
    ["css", "css:title-send", page.locator('[title*="Send" i]')],
  ];

  for (const [strategy, name, locator] of candidates) {
    const count = await locator.count().catch(() => 0);
    const visible = await firstVisible(locator);
    attempts.push({ name, strategy, matched: Boolean(visible), count });
    if (visible) return visible;
  }
  return null;
}

async function typePrompt(promptBox: Locator, prompt: string): Promise<void> {
  await promptBox.click();
  const tagName = await promptBox.evaluate((element) => element.tagName.toLowerCase());
  if (tagName === "textarea" || tagName === "input") {
    await promptBox.fill(prompt);
    return;
  }
  await promptBox.evaluate((element, value) => {
    element.textContent = value;
    element.dispatchEvent(new InputEvent("input", { bubbles: true, inputType: "insertText", data: value }));
  }, prompt);
}

async function latestAnswerText(page: Page): Promise<string> {
  const answer = page.locator('[data-testid="assistant-answer"], article, main section').last();
  return (await answer.innerText().catch(() => "")).trim();
}

async function generationControls(page: Page, attempts: LocatorAttempt[]) {
  const stop = page.getByRole("button", { name: /stop|cancel|interrupt/i }).first();
  const stopVisible = await stop.isVisible().catch(() => false);
  const promptBox = await findPromptBox(page, attempts);
  const inputUsable = promptBox ? !(await promptBox.getAttribute("aria-disabled").then((value) => value === "true").catch(() => false)) : false;
  return { stopVisible, inputUsable };
}

export async function waitForFinalAnswer(
  page: Page,
  startedAt: number,
  options: SendSingleOptions,
  attempts: LocatorAttempt[],
): Promise<GeminiAdapterResult> {
  let lastText = "";
  let lastChangedAt = Date.now();
  let sawAnswer = false;

  while (Date.now() - startedAt < options.timeoutMs) {
    const critical = await scanCriticalState(page);
    if (critical) return result(critical, startedAt, null, attempts, critical);

    const text = await latestAnswerText(page);
    if (text.length > 0) sawAnswer = true;
    if (text !== lastText) {
      lastText = text;
      lastChangedAt = Date.now();
    }

    const controls = await generationControls(page, attempts);
    const quietEnough = Date.now() - lastChangedAt >= options.quietMs;
    if (sawAnswer && quietEnough && !controls.stopVisible && controls.inputUsable) {
      return result("ok", startedAt, lastText, attempts, null);
    }

    await page.waitForTimeout(100);
  }

  return result("generation_timeout", startedAt, lastText || null, attempts, "generation_timeout");
}

export async function sendSingleDomOnly(page: Page, prompt: string, options: SendSingleOptions): Promise<GeminiAdapterResult> {
  const startedAt = Date.now();
  const attempts: LocatorAttempt[] = [];

  const criticalBefore = await scanCriticalState(page);
  if (criticalBefore) return result(criticalBefore, startedAt, null, attempts, criticalBefore);

  const promptBox = await findPromptBox(page, attempts);
  if (!promptBox) return result("failed", startedAt, null, attempts, "prompt_input_not_found");

  await typePrompt(promptBox, prompt);
  const sendButton = await findSendButton(page, attempts);
  if (!sendButton) return result("failed", startedAt, null, attempts, "send_button_not_found");
  await sendButton.click();

  return await waitForFinalAnswer(page, startedAt, options, attempts);
}
```

- [ ] **Step 4: Verify baseline tests pass**

Run:

```powershell
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/dom-only-baseline.spec.ts
```

Expected: PASS for 4 tests.

- [ ] **Step 5: Commit baseline adapter**

Run:

```powershell
git add research/gemini_browser_adapter/src/dom-contract.ts research/gemini_browser_adapter/tests/dom-only-baseline.spec.ts
git commit -m "Add DOM-only Gemini adapter baseline"
```

---

### Task 5: Deterministic Locator Scoring

**Files:**
- Create: `research/gemini_browser_adapter/src/scoring.ts`
- Create: `research/gemini_browser_adapter/src/scoring.test.ts`
- Modify: `research/gemini_browser_adapter/src/dom-contract.ts`
- Create: `research/gemini_browser_adapter/tests/resilient-scoring.spec.ts`

- [ ] **Step 1: Write failing scoring unit tests**

Create `research/gemini_browser_adapter/src/scoring.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { scoreEditableCandidate, scoreButtonCandidate } from "./scoring";

describe("locator scoring", () => {
  it("scores visible lower-page editable prompt candidates highly", () => {
    const score = scoreEditableCandidate({
      aria: "ask gemini textbox",
      topRatio: 0.8,
      width: 480,
      height: 48,
      visible: true,
      editable: true,
    });
    expect(score).toBeGreaterThanOrEqual(8);
  });

  it("rejects hidden editable candidates", () => {
    const score = scoreEditableCandidate({
      aria: "ask gemini",
      topRatio: 0.8,
      width: 480,
      height: 48,
      visible: false,
      editable: true,
    });
    expect(score).toBe(0);
  });

  it("scores send-like buttons by label and position", () => {
    const score = scoreButtonCandidate({
      label: "send message",
      topRatio: 0.75,
      rightRatio: 0.85,
      width: 44,
      height: 36,
      visible: true,
      enabled: true,
    });
    expect(score).toBeGreaterThanOrEqual(8);
  });
});
```

- [ ] **Step 2: Run scoring test to verify failure**

Run:

```powershell
npm run test:gemini-browser-adapter:unit -- research/gemini_browser_adapter/src/scoring.test.ts
```

Expected: FAIL because `scoring.ts` does not exist.

- [ ] **Step 3: Implement scoring helpers**

Create `research/gemini_browser_adapter/src/scoring.ts`:

```ts
export type EditableCandidateScoreInput = {
  aria: string;
  topRatio: number;
  width: number;
  height: number;
  visible: boolean;
  editable: boolean;
};

export type ButtonCandidateScoreInput = {
  label: string;
  topRatio: number;
  rightRatio: number;
  width: number;
  height: number;
  visible: boolean;
  enabled: boolean;
};

export function scoreEditableCandidate(input: EditableCandidateScoreInput): number {
  if (!input.visible || !input.editable) return 0;
  let score = 0;
  if (input.width >= 300) score += 2;
  if (input.height >= 24 && input.height <= 240) score += 2;
  if (input.topRatio >= 0.45) score += 2;
  if (/ask|message|prompt|gemini|enter|type/i.test(input.aria)) score += 3;
  if (input.editable) score += 1;
  return score;
}

export function scoreButtonCandidate(input: ButtonCandidateScoreInput): number {
  if (!input.visible || !input.enabled) return 0;
  let score = 0;
  if (input.width >= 24 && input.height >= 24) score += 2;
  if (/send|submit|run|arrow|message/i.test(input.label)) score += 5;
  if (input.topRatio >= 0.45) score += 1;
  if (input.rightRatio >= 0.55) score += 2;
  return score;
}
```

- [ ] **Step 4: Verify scoring unit tests pass**

Run:

```powershell
npm run test:gemini-browser-adapter:unit -- research/gemini_browser_adapter/src/scoring.test.ts
```

Expected: PASS for 3 tests.

- [ ] **Step 5: Add resilient-scoring e2e tests**

Create `research/gemini_browser_adapter/tests/resilient-scoring.spec.ts`:

```ts
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
```

- [ ] **Step 6: Run resilient tests to verify failure**

Run:

```powershell
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/resilient-scoring.spec.ts
```

Expected: FAIL because `sendSingleResilientScoring` is not exported.

- [ ] **Step 7: Add resilient scoring variant**

Modify `research/gemini_browser_adapter/src/dom-contract.ts` by importing scoring helpers:

```ts
import { scoreButtonCandidate, scoreEditableCandidate } from "./scoring";
```

Add these functions below `findSendButton`:

```ts
async function findPromptBoxByScoring(page: Page, attempts: LocatorAttempt[]): Promise<Locator | null> {
  const bestIndex = await page.locator("textarea, input, [contenteditable='true'], [role='textbox']").evaluateAll((elements) => {
    let best = { index: -1, score: 0 };
    elements.forEach((element, index) => {
      const rect = element.getBoundingClientRect();
      const style = window.getComputedStyle(element);
      const aria = [
        element.getAttribute("aria-label"),
        element.getAttribute("placeholder"),
        element.getAttribute("role"),
      ].filter(Boolean).join(" ");
      const editable =
        element instanceof HTMLTextAreaElement ||
        element instanceof HTMLInputElement ||
        element.getAttribute("contenteditable") === "true" ||
        element.getAttribute("role") === "textbox";
      const visible = style.display !== "none" && style.visibility !== "hidden" && rect.width > 0 && rect.height > 0;
      let score = 0;
      if (visible && editable) {
        if (rect.width >= 300) score += 2;
        if (rect.height >= 24 && rect.height <= 240) score += 2;
        if (rect.top / window.innerHeight >= 0.45) score += 2;
        if (/ask|message|prompt|gemini|enter|type/i.test(aria)) score += 3;
        score += 1;
      }
      if (score > best.score) best = { index, score };
    });
    return best.score >= 5 ? best.index : -1;
  });

  attempts.push({ name: "fuzzy:editable", strategy: "fuzzy", matched: bestIndex >= 0, score: bestIndex >= 0 ? 5 : 0 });
  return bestIndex >= 0 ? page.locator("textarea, input, [contenteditable='true'], [role='textbox']").nth(bestIndex) : null;
}

async function findSendButtonByScoring(page: Page, attempts: LocatorAttempt[]): Promise<Locator | null> {
  const bestIndex = await page.locator("button, [role='button'], [data-send]").evaluateAll((elements) => {
    let best = { index: -1, score: 0 };
    elements.forEach((element, index) => {
      const rect = element.getBoundingClientRect();
      const style = window.getComputedStyle(element);
      const label = [
        element.getAttribute("aria-label"),
        element.getAttribute("title"),
        element.textContent,
      ].filter(Boolean).join(" ");
      const visible = style.display !== "none" && style.visibility !== "hidden" && rect.width > 0 && rect.height > 0;
      const enabled = !element.hasAttribute("disabled") && element.getAttribute("aria-disabled") !== "true";
      let score = 0;
      if (visible && enabled) {
        if (rect.width >= 24 && rect.height >= 24) score += 2;
        if (/send|submit|run|arrow|message/i.test(label)) score += 5;
        if (rect.top / window.innerHeight >= 0.45) score += 1;
        if (rect.right / window.innerWidth >= 0.55) score += 2;
      }
      if (score > best.score) best = { index, score };
    });
    return best.score >= 4 ? best.index : -1;
  });

  attempts.push({ name: "fuzzy:send-button", strategy: "fuzzy", matched: bestIndex >= 0, score: bestIndex >= 0 ? 4 : 0 });
  return bestIndex >= 0 ? page.locator("button, [role='button'], [data-send]").nth(bestIndex) : null;
}
```

Add this exported function at the end of the file:

```ts
export async function sendSingleResilientScoring(page: Page, prompt: string, options: SendSingleOptions): Promise<GeminiAdapterResult> {
  const startedAt = Date.now();
  const attempts: LocatorAttempt[] = [];
  void scoreEditableCandidate;
  void scoreButtonCandidate;

  const criticalBefore = await scanCriticalState(page);
  if (criticalBefore) return { ...result(criticalBefore, startedAt, null, attempts, criticalBefore), variant: "resilient-scoring" };

  const promptBox = (await findPromptBox(page, attempts)) ?? (await findPromptBoxByScoring(page, attempts));
  if (!promptBox) return { ...result("failed", startedAt, null, attempts, "prompt_input_not_found"), variant: "resilient-scoring" };

  await typePrompt(promptBox, prompt);
  const sendButton = (await findSendButton(page, attempts)) ?? (await findSendButtonByScoring(page, attempts));
  if (!sendButton) return { ...result("failed", startedAt, null, attempts, "send_button_not_found"), variant: "resilient-scoring" };
  await sendButton.click();

  const finalAnswer = await waitForFinalAnswer(page, startedAt, options, attempts);
  return {
    ...finalAnswer,
    variant: "resilient-scoring",
    locatorAttempts: attempts,
  };
}
```

- [ ] **Step 8: Verify scoring adapter tests pass**

Run:

```powershell
npm run test:gemini-browser-adapter:unit -- research/gemini_browser_adapter/src/scoring.test.ts
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/resilient-scoring.spec.ts research/gemini_browser_adapter/tests/dom-only-baseline.spec.ts
```

Expected: PASS for scoring unit tests and both e2e files.

- [ ] **Step 9: Commit resilient scoring**

Run:

```powershell
git add research/gemini_browser_adapter/src/scoring.ts research/gemini_browser_adapter/src/scoring.test.ts research/gemini_browser_adapter/src/dom-contract.ts research/gemini_browser_adapter/tests/resilient-scoring.spec.ts
git commit -m "Add resilient scoring Gemini adapter variant"
```

---

### Task 6: Failure Artifacts

**Files:**
- Create: `research/gemini_browser_adapter/src/artifacts.ts`
- Create: `research/gemini_browser_adapter/tests/failure-artifacts.spec.ts`
- Modify: `research/gemini_browser_adapter/src/dom-contract.ts`

- [ ] **Step 1: Write failing artifact e2e test**

Create `research/gemini_browser_adapter/tests/failure-artifacts.spec.ts`:

```ts
import { existsSync, readFileSync } from "node:fs";
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

test("timeout writes screenshot, html, and telemetry artifacts", async ({ page }) => {
  await page.goto(server.url("never-stable"));
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
  expect(Array.isArray(telemetry.locatorAttempts)).toBe(true);
});
```

- [ ] **Step 2: Run artifact test to verify failure**

Run:

```powershell
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/failure-artifacts.spec.ts
```

Expected: FAIL because `artifactDir` is not part of `SendSingleOptions` and artifact capture is not implemented.

- [ ] **Step 3: Implement artifact writer**

Create `research/gemini_browser_adapter/src/artifacts.ts`:

```ts
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Page } from "@playwright/test";
import type { FailureArtifacts, LocatorAttempt, NetworkEventSummary } from "./types";

export type CaptureFailureInput = {
  page: Page;
  artifactDir: string;
  reason: string;
  locatorAttempts: LocatorAttempt[];
  networkSummary: NetworkEventSummary[];
};

export async function captureFailureArtifacts(input: CaptureFailureInput): Promise<FailureArtifacts> {
  await mkdir(input.artifactDir, { recursive: true });
  const screenshotPath = path.join(input.artifactDir, "failure.png");
  const htmlPath = path.join(input.artifactDir, "page.html");
  const telemetryPath = path.join(input.artifactDir, "telemetry.json");

  await input.page.screenshot({ path: screenshotPath, fullPage: true }).catch(() => undefined);
  await writeFile(htmlPath, await input.page.content(), "utf8").catch(() => undefined);
  await writeFile(
    telemetryPath,
    JSON.stringify(
      {
        reason: input.reason,
        url: input.page.url(),
        locatorAttempts: input.locatorAttempts,
        networkSummary: input.networkSummary,
        capturedAt: new Date().toISOString(),
      },
      null,
      2,
    ),
    "utf8",
  );

  return {
    screenshotPath,
    htmlPath,
    telemetryPath,
    tracePath: null,
  };
}
```

- [ ] **Step 4: Integrate artifact capture**

Modify `SendSingleOptions` in `dom-contract.ts`:

```ts
export type SendSingleOptions = {
  timeoutMs: number;
  quietMs: number;
  artifactDir?: string;
};
```

Import:

```ts
import { captureFailureArtifacts } from "./artifacts";
```

Add helper:

```ts
async function withArtifacts(
  page: Page,
  base: GeminiAdapterResult,
  options: SendSingleOptions,
): Promise<GeminiAdapterResult> {
  if (!options.artifactDir || base.status === "ok") return base;
  return {
    ...base,
    artifacts: await captureFailureArtifacts({
      page,
      artifactDir: options.artifactDir,
      reason: base.errorReason ?? base.status,
      locatorAttempts: base.locatorAttempts,
      networkSummary: base.networkSummary,
    }),
  };
}
```

Wrap every returned non-`ok` result in both adapter variants with `await withArtifacts(page, baseResult, options)`. For example:

```ts
const baseResult = result("generation_timeout", startedAt, lastText || null, attempts, "generation_timeout");
return await withArtifacts(page, baseResult, options);
```

- [ ] **Step 5: Verify artifact tests pass**

Run:

```powershell
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/failure-artifacts.spec.ts
```

Expected: PASS and files exist under `research/gemini_browser_adapter/artifacts/test-timeout`.

- [ ] **Step 6: Confirm generated artifacts are ignored**

Run:

```powershell
git status --short --untracked-files=all research\\gemini_browser_adapter\\artifacts
```

Expected: no generated artifact files appear because `artifacts` is ignored except the tracked `.gitkeep`.

- [ ] **Step 7: Commit artifacts**

Run:

```powershell
git add research/gemini_browser_adapter/src/artifacts.ts research/gemini_browser_adapter/src/dom-contract.ts research/gemini_browser_adapter/tests/failure-artifacts.spec.ts
git commit -m "Add Gemini adapter failure artifacts"
```

---

### Task 7: Telemetry-Assisted Variant

**Files:**
- Create: `research/gemini_browser_adapter/src/telemetry.ts`
- Create: `research/gemini_browser_adapter/src/telemetry.test.ts`
- Modify: `research/gemini_browser_adapter/src/dom-contract.ts`
- Create: `research/gemini_browser_adapter/tests/telemetry-assisted.spec.ts`

- [ ] **Step 1: Write failing telemetry unit tests**

Create `research/gemini_browser_adapter/src/telemetry.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { redactUrl } from "./telemetry";

describe("telemetry redaction", () => {
  it("redacts token-like query parameters", () => {
    const url = redactUrl("https://gemini.google.com/app?token=secret&authuser=0&safe=yes");
    expect(url).toContain("token=<redacted>");
    expect(url).toContain("authuser=<redacted>");
    expect(url).toContain("safe=yes");
  });
});
```

- [ ] **Step 2: Run telemetry test to verify failure**

Run:

```powershell
npm run test:gemini-browser-adapter:unit -- research/gemini_browser_adapter/src/telemetry.test.ts
```

Expected: FAIL because `telemetry.ts` does not exist.

- [ ] **Step 3: Implement telemetry collector**

Create `research/gemini_browser_adapter/src/telemetry.ts`:

```ts
import type { Page } from "@playwright/test";
import type { NetworkEventSummary } from "./types";

export function redactUrl(url: string): string {
  return url.replace(/([?&](?:token|key|session|authuser|at|credential)=)[^&]+/gi, "$1<redacted>");
}

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
```

- [ ] **Step 4: Verify telemetry unit tests pass**

Run:

```powershell
npm run test:gemini-browser-adapter:unit -- research/gemini_browser_adapter/src/telemetry.test.ts
```

Expected: PASS for 1 test.

- [ ] **Step 5: Add telemetry-assisted e2e tests**

Create `research/gemini_browser_adapter/tests/telemetry-assisted.spec.ts`:

```ts
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
```

- [ ] **Step 6: Run telemetry e2e tests to verify failure**

Run:

```powershell
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/telemetry-assisted.spec.ts
```

Expected: FAIL because `sendSingleTelemetryAssisted` is not exported.

- [ ] **Step 7: Implement telemetry-assisted adapter**

Modify `dom-contract.ts`:

```ts
import { attachNetworkTelemetry } from "./telemetry";
```

Add at the end:

```ts
export async function sendSingleTelemetryAssisted(page: Page, prompt: string, options: SendSingleOptions): Promise<GeminiAdapterResult> {
  const networkSummary: GeminiAdapterResult["networkSummary"] = [];
  attachNetworkTelemetry(page, networkSummary);
  const result = await sendSingleResilientScoring(page, prompt, options);
  return {
    ...result,
    variant: "telemetry-assisted",
    networkSummary,
  };
}
```

Ensure `withArtifacts` receives the result's `networkSummary` when writing telemetry files.

- [ ] **Step 8: Verify telemetry tests pass**

Run:

```powershell
npm run test:gemini-browser-adapter:unit -- research/gemini_browser_adapter/src/telemetry.test.ts
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/telemetry-assisted.spec.ts
```

Expected: PASS for telemetry unit and e2e tests.

- [ ] **Step 9: Commit telemetry-assisted variant**

Run:

```powershell
git add research/gemini_browser_adapter/src/telemetry.ts research/gemini_browser_adapter/src/telemetry.test.ts research/gemini_browser_adapter/src/dom-contract.ts research/gemini_browser_adapter/tests/telemetry-assisted.spec.ts
git commit -m "Add telemetry-assisted Gemini adapter variant"
```

---

### Task 8: Matrix Report Runner

**Files:**
- Create: `research/gemini_browser_adapter/scripts/write-matrix-report.mjs`
- Modify: `research/gemini_browser_adapter/RESILIENCE_TEST_MATRIX.md`
- Modify: `research/gemini_browser_adapter/TOOLS_AND_METHODS.md`

- [ ] **Step 1: Write matrix report script**

Create `research/gemini_browser_adapter/scripts/write-matrix-report.mjs`:

```js
import { existsSync, readFileSync } from "node:fs";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";

const artifactDir = "research/gemini_browser_adapter/artifacts";
const inputPath = path.join(artifactDir, "playwright-results.json");
const outputPath = path.join(artifactDir, "matrix-report.md");

function collectSpecs(suite, rows = []) {
  for (const spec of suite.specs || []) {
    const tests = spec.tests || [];
    for (const test of tests) {
      const result = test.results?.[0];
      rows.push({
        title: [...(suite.titlePath || []), spec.title].filter(Boolean).join(" / "),
        status: test.outcome || result?.status || "unknown",
        duration: result?.duration || 0,
      });
    }
  }
  for (const child of suite.suites || []) {
    collectSpecs({ ...child, titlePath: [...(suite.titlePath || []), child.title] }, rows);
  }
  return rows;
}

if (!existsSync(inputPath)) {
  console.error(`Missing Playwright JSON results at ${inputPath}`);
  process.exit(1);
}

const json = JSON.parse(readFileSync(inputPath, "utf8"));
const rows = collectSpecs({ suites: json.suites || [], titlePath: [] });
const passed = rows.filter((row) => row.status === "expected" || row.status === "passed").length;
const failed = rows.length - passed;
const worst = rows.reduce((max, row) => Math.max(max, row.duration), 0);

const report = [
  "# Gemini Browser Adapter Matrix Report",
  "",
  `Generated: ${new Date().toISOString()}`,
  "",
  `Total tests: ${rows.length}`,
  `Passed tests: ${passed}`,
  `Failed or unexpected tests: ${failed}`,
  `Worst duration ms: ${worst}`,
  "",
  "| Test | Status | Duration ms |",
  "| --- | --- | ---: |",
  ...rows.map((row) => `| ${row.title.replaceAll("|", "\\|")} | ${row.status} | ${row.duration} |`),
  "",
].join("\n");

await mkdir(artifactDir, { recursive: true });
await writeFile(outputPath, report, "utf8");
console.log(`Wrote ${outputPath}`);
if (failed > 0) process.exit(1);
```

- [ ] **Step 2: Run full research verification**

Run:

```powershell
npm run test:gemini-browser-adapter
```

Expected:

- unit tests pass;
- Playwright e2e tests pass;
- `research/gemini_browser_adapter/artifacts/playwright-results.json` is written;
- `research/gemini_browser_adapter/artifacts/matrix-report.md` is written;
- command exits `0`.

- [ ] **Step 3: Update matrix documentation**

Append this section to `research/gemini_browser_adapter/RESILIENCE_TEST_MATRIX.md`:

```markdown
## Execution

Run the complete research matrix with:

```powershell
npm run test:gemini-browser-adapter
```

The Playwright JSON output is written to:

```text
research/gemini_browser_adapter/artifacts/playwright-results.json
```

The summarized matrix report is written to:

```text
research/gemini_browser_adapter/artifacts/matrix-report.md
```
```

- [ ] **Step 4: Update tools document**

Add this sentence under `## Method` in `research/gemini_browser_adapter/TOOLS_AND_METHODS.md`:

```markdown
The executable matrix command is `npm run test:gemini-browser-adapter`; it runs unit tests, Playwright tests, and the matrix report writer.
```

- [ ] **Step 5: Verify no generated artifacts are staged**

Run:

```powershell
git status --short --untracked-files=all research\\gemini_browser_adapter
```

Expected: source files and docs may appear, but generated files under `artifacts/` do not appear.

- [ ] **Step 6: Commit matrix runner**

Run:

```powershell
git add research/gemini_browser_adapter/scripts/write-matrix-report.mjs research/gemini_browser_adapter/RESILIENCE_TEST_MATRIX.md research/gemini_browser_adapter/TOOLS_AND_METHODS.md
git commit -m "Add Gemini adapter research matrix report"
```

---

### Task 9: Final Research Harness Verification

**Files:**
- No new files.
- Verify all files created by previous tasks.

- [ ] **Step 1: Run complete research matrix**

Run:

```powershell
npm run test:gemini-browser-adapter
```

Expected: command exits `0`.

- [ ] **Step 2: Run Svelte/TypeScript project check**

Run:

```powershell
npm run check
```

Expected: command exits `0`. If unrelated pre-existing `src-tauri` changes do not affect this command, keep them out of this research commit.

- [ ] **Step 3: Review generated matrix report**

Run:

```powershell
Get-Content -LiteralPath 'research\\gemini_browser_adapter\\artifacts\\matrix-report.md'
```

Expected: report lists all research tests and has `Failed or unexpected tests: 0`.

- [ ] **Step 4: Confirm production Rust/Tauri was untouched by research commits**

Run:

```powershell
git diff --name-only HEAD -- src-tauri
```

Expected: no output for changes introduced by this research plan. If user-owned `src-tauri` edits exist in the working tree, leave them unstaged.

- [ ] **Step 5: Commit final plan doc update**

Run:

```powershell
git add docs/superpowers/plans/2026-06-20-gemini-browser-adapter-research-plan.md
git commit -m "Add Gemini adapter research implementation plan"
```

---

## Self-Review Checklist

- The plan keeps Python out of production runtime.
- The plan keeps Rust/Tauri out of the first research harness.
- The plan creates a local mock Gemini page before adapter code.
- The plan compares three variants from `TOOLS_AND_METHODS.md`.
- The plan covers success, manual-action, rate-limit, timeout, artifact, and telemetry paths.
- The plan writes sanitized artifacts only under ignored `research/gemini_browser_adapter/artifacts`.
- The plan includes exact commands and expected outcomes for every task.
