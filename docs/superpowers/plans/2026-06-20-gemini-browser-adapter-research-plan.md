# Gemini Browser Adapter Research Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a local, reproducible research harness for the Gemini browser adapter and compare three TypeScript/Node adapter variants against deterministic mock Gemini pages.

**Architecture:** Keep the production app out of this research slice. Add a local mock Gemini server, Playwright tests, TypeScript adapter modules, diagnostic artifact helpers, telemetry helpers, an executable scenario matrix, and a matrix report runner under `research/gemini_browser_adapter`. Rust/Tauri integration is not part of this plan.

**Tech Stack:** TypeScript, Node ESM, Vitest, Playwright, local HTTP mock server, existing npm scripts.

---

## File Structure

Create and modify only research/tooling files:

- Modify: `package.json` to add Playwright dependency and research scripts.
- Create: `research/gemini_browser_adapter/playwright.config.ts` for adapter e2e tests.
- Create: `research/gemini_browser_adapter/tsconfig.json` for strict research TypeScript checks.
- Create: `research/gemini_browser_adapter/mock-gemini/variants.mjs` to render deterministic mock Gemini pages.
- Create: `research/gemini_browser_adapter/mock-gemini/server.mjs` to start and stop a local mock server.
- Create: `research/gemini_browser_adapter/src/types.ts` for shared status/result/artifact types.
- Create: `research/gemini_browser_adapter/src/status.test.ts` for type and status guard tests.
- Create: `research/gemini_browser_adapter/src/dom-contract.ts` for adapter variant implementations.
- Create: `research/gemini_browser_adapter/tests/dom-only-baseline.spec.ts` for baseline Playwright tests.
- Create: `research/gemini_browser_adapter/gemini-dom-contract.config.json` for local selector overrides.
- Create: `research/gemini_browser_adapter/src/config.ts` for selector override loading and defaults.
- Create: `research/gemini_browser_adapter/src/config.test.ts` for config override unit tests.
- Create: `research/gemini_browser_adapter/src/scoring.ts` for deterministic locator scoring.
- Create: `research/gemini_browser_adapter/src/scoring.test.ts` for scoring unit tests.
- Create: `research/gemini_browser_adapter/tests/resilient-scoring.spec.ts` for scoring adapter tests.
- Create: `research/gemini_browser_adapter/src/redaction.ts` for URL and DOM artifact sanitization.
- Create: `research/gemini_browser_adapter/src/artifacts.ts` for failure bundle writing.
- Create: `research/gemini_browser_adapter/tests/failure-artifacts.spec.ts` for artifact expectations.
- Create: `research/gemini_browser_adapter/src/telemetry.ts` for sanitized network telemetry.
- Create: `research/gemini_browser_adapter/src/telemetry.test.ts` for redaction/unit tests.
- Create: `research/gemini_browser_adapter/tests/telemetry-assisted.spec.ts` for telemetry-assisted behavior.
- Create: `research/gemini_browser_adapter/matrix-cases.json` for shared scenario matrix metadata.
- Create: `research/gemini_browser_adapter/src/matrix-cases.ts` for the executable scenario matrix and evidence requirements.
- Create: `research/gemini_browser_adapter/tests/matrix.spec.ts` for the `3 variants x all scenarios` Playwright matrix.
- Create: `research/gemini_browser_adapter/scripts/write-matrix-report.mjs` for coverage-validated report generation.
- Create: `research/gemini_browser_adapter/scripts/run-full-verification.mjs` so matrix reporting runs even after Playwright failures.
- Modify: `research/gemini_browser_adapter/RESILIENCE_TEST_MATRIX.md` to add the executable command and artifact paths.
- Modify: `research/gemini_browser_adapter/TOOLS_AND_METHODS.md` to reference the matrix runner.

Do not touch `src-tauri/*` in this plan.

---

### Task 1: Research Test Tooling

**Files:**
- Modify: `package.json`
- Create: `research/gemini_browser_adapter/playwright.config.ts`
- Create: `research/gemini_browser_adapter/tsconfig.json`
- Create: `research/gemini_browser_adapter/scripts/run-full-verification.mjs`

- [x] **Step 1: Add Playwright test dependency**

Run:

```powershell
npm install --save-dev @playwright/test @types/node
```

Expected:

- `package.json` gains `@playwright/test` and `@types/node` under `devDependencies`.
- `package-lock.json` updates.
- If the sandbox blocks registry access, rerun this command with escalation.

- [x] **Step 2: Install Chromium browser for local Playwright runs**

Run:

```powershell
npx playwright install chromium
```

Expected: Playwright downloads or confirms the Chromium browser binary.

- [x] **Step 3: Add package scripts**

Edit `package.json` and add these scripts inside `"scripts"`:

```json
"test:gemini-browser-adapter:typecheck": "tsc -p research/gemini_browser_adapter/tsconfig.json --noEmit",
"test:gemini-browser-adapter:unit": "node scripts/run-vitest.mjs run research/gemini_browser_adapter/src",
"test:gemini-browser-adapter:e2e": "playwright test -c research/gemini_browser_adapter/playwright.config.ts",
"test:gemini-browser-adapter:report": "node research/gemini_browser_adapter/scripts/write-matrix-report.mjs",
"test:gemini-browser-adapter": "node research/gemini_browser_adapter/scripts/run-full-verification.mjs"
```

- [x] **Step 4: Create full verification wrapper**

Create `research/gemini_browser_adapter/scripts/run-full-verification.mjs`:

```js
import { rmSync } from "node:fs";
import { spawnSync } from "node:child_process";

const npm = process.platform === "win32" ? "npm.cmd" : "npm";
const staleArtifactPaths = [
  "research/gemini_browser_adapter/artifacts/matrix",
  "research/gemini_browser_adapter/artifacts/playwright-results.json",
];

function run(label, args) {
  console.log(`\n== ${label} ==`);
  const result = spawnSync(npm, args, { stdio: "inherit", shell: process.platform === "win32" });
  if (result.error) {
    console.error(`${label} failed to start: ${result.error.message}`);
    return 1;
  }
  return result.status ?? 1;
}

function clearStaleMatrixArtifacts() {
  for (const target of staleArtifactPaths) {
    rmSync(target, { recursive: true, force: true });
  }
}

for (const [label, args] of [
  ["research typecheck", ["run", "test:gemini-browser-adapter:typecheck"]],
  ["research unit tests", ["run", "test:gemini-browser-adapter:unit"]],
]) {
  const code = run(label, args);
  if (code !== 0) process.exit(code);
}

let e2eCode = 1;
let reportCode = 1;
try {
  clearStaleMatrixArtifacts();
  e2eCode = run("research Playwright e2e", ["run", "test:gemini-browser-adapter:e2e"]);
} finally {
  reportCode = run("research matrix report", ["run", "test:gemini-browser-adapter:report"]);
}

process.exit(e2eCode || reportCode);
```

Expected:

- typecheck and unit failures stop the wrapper before e2e;
- stale `artifacts/matrix` result files and stale `playwright-results.json` are removed immediately before e2e;
- after Playwright e2e starts, the matrix report command runs even when Playwright exits non-zero;
- the wrapper exits non-zero when either e2e or report generation fails.

- [x] **Step 5: Create Playwright config**

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
  workers: 1,
  reporter: [
    ["list"],
    [
      "json",
      {
        outputFile: "artifacts/playwright-results.json",
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

- [x] **Step 6: Create research TypeScript config**

Create `research/gemini_browser_adapter/tsconfig.json`:

```json
{
  "compilerOptions": {
    "allowJs": true,
    "checkJs": false,
    "esModuleInterop": true,
    "forceConsistentCasingInFileNames": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "skipLibCheck": true,
    "strict": true,
    "target": "ES2022",
    "types": ["node", "vitest"]
  },
  "include": [
    "playwright.config.ts",
    "src/**/*.ts",
    "tests/**/*.ts",
    "mock-gemini/**/*.mjs"
  ],
  "exclude": ["artifacts", "node_modules"]
}
```

- [x] **Step 6: Verify TypeScript config parses**

Run:

```powershell
npm run test:gemini-browser-adapter:typecheck
```

Expected: command exits `0`. Do not run `playwright --list` yet; the Playwright test directory has no valid spec file at this point and may fail with `No tests found`.

- [x] **Step 7: Commit tooling**

Run:

```powershell
git add package.json package-lock.json research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tsconfig.json research/gemini_browser_adapter/scripts/run-full-verification.mjs docs/superpowers/plans/2026-06-20-gemini-browser-adapter-research-plan.md
git commit -m "Add Gemini adapter research test tooling"
```

---

### Task 2: Mock Gemini Server

**Files:**
- Create: `research/gemini_browser_adapter/mock-gemini/variants.mjs`
- Create: `research/gemini_browser_adapter/mock-gemini/server.mjs`
- Create: `research/gemini_browser_adapter/tests/mock-gemini.spec.ts`

- [x] **Step 1: Write failing Playwright tests for mock variants**

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
```

- [x] **Step 2: Run failing mock tests**

Run:

```powershell
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/mock-gemini.spec.ts
```

Expected: FAIL because `mock-gemini/server.mjs` does not exist.

- [x] **Step 3: Implement mock variants**

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
  fetch('/mock-gemini-event', { method: 'POST', body: 'submitted' }).catch(() => undefined);
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
  if (variant === "ready" || variant === "closed-page") {
    return basePage(`
      <section class="composer">
        <div role="textbox" contenteditable="true" aria-label="Ask Gemini"></div>
        <button data-send aria-label="Send">Send</button>
      </section>
      <article class="answer" data-testid="assistant-answer"></article>
      <button data-testid="stop-control" aria-label="Stop" hidden>Stop</button>
    `, submitScript);
  }

  if (variant === "ready-missing-send") {
    return basePage(`
      <section class="composer">
        <div role="textbox" contenteditable="true" aria-label="Ask Gemini"></div>
      </section>
    `);
  }

  if (variant === "ready-broken") {
    return basePage(`
      <section class="composer">
        <p>Gemini shell rendered, but composer controls are unavailable.</p>
      </section>
    `);
  }

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

- [x] **Step 4: Implement mock server**

Create `research/gemini_browser_adapter/mock-gemini/server.mjs`:

```js
import http from "node:http";
import { renderMockGeminiPage } from "./variants.mjs";

export async function startMockGeminiServer() {
  const server = http.createServer((request, response) => {
    const url = new URL(request.url || "/", "http://127.0.0.1");
    if (url.pathname === "/mock-gemini-event") {
      response.writeHead(204);
      response.end();
      return;
    }

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

- [x] **Step 5: Verify Playwright config lists existing tests**

Run:

```powershell
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/mock-gemini.spec.ts --list
```

Expected: command exits `0` and lists the 7 mock Gemini tests.

- [x] **Step 6: Verify mock tests pass**

Run:

```powershell
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/mock-gemini.spec.ts
```

Expected: PASS for 7 tests.

- [x] **Step 7: Commit mock server**

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

- [x] **Step 1: Write failing unit tests for status guards**

Create `research/gemini_browser_adapter/src/status.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { isManualActionStatus, isSuccessStatus, isTerminalStatus } from "./types";

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

  it("detects success statuses", () => {
    expect(isSuccessStatus("ok")).toBe(true);
    expect(isSuccessStatus("ready")).toBe(true);
    expect(isSuccessStatus("response_parse_failed")).toBe(false);
  });
});
```

- [x] **Step 2: Run unit test to verify it fails**

Run:

```powershell
npm run test:gemini-browser-adapter:unit -- research/gemini_browser_adapter/src/status.test.ts
```

Expected: FAIL because `types.ts` does not exist.

- [x] **Step 3: Implement shared types**

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

export function isSuccessStatus(status: GeminiAdapterStatus): boolean {
  return status === "ok" || status === "ready";
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

- [x] **Step 4: Verify unit test passes**

Run:

```powershell
npm run test:gemini-browser-adapter:unit -- research/gemini_browser_adapter/src/status.test.ts
```

Expected: PASS for 3 tests.

- [x] **Step 5: Commit shared types**

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

- [x] **Step 1: Write failing baseline e2e tests**

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

- [x] **Step 2: Run baseline tests to verify failure**

Run:

```powershell
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/dom-only-baseline.spec.ts
```

Expected: FAIL because `dom-contract.ts` does not exist.

- [x] **Step 3: Implement DOM-only adapter**

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

async function latestAnswerText(page: Page): Promise<string | null> {
  const answer = page.locator(
    '[data-testid="assistant-answer"], [data-testid*="assistant" i], [data-testid*="response" i], article.answer, [data-answer]',
  );
  if ((await answer.count().catch(() => 0)) === 0) return null;
  return (await answer.last().innerText().catch(() => "")).trim();
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
    if (text === null) {
      const answerMissingGraceMs = Math.min(Math.max(options.quietMs * 2, 500), 1500);
      if (Date.now() - startedAt >= answerMissingGraceMs) {
        return result("response_parse_failed", startedAt, null, attempts, "answer_container_not_found");
      }
      await page.waitForTimeout(100);
      continue;
    }

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

  if (page.isClosed()) return result("browser_crashed", startedAt, null, attempts, "browser_crashed");

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

export async function probeReadyDomOnly(page: Page, _options?: SendSingleOptions): Promise<GeminiAdapterResult> {
  const startedAt = Date.now();
  const attempts: LocatorAttempt[] = [];

  if (page.isClosed()) return result("browser_crashed", startedAt, null, attempts, "browser_crashed");

  try {
    const criticalBefore = await scanCriticalState(page);
    if (criticalBefore) return result(criticalBefore, startedAt, null, attempts, criticalBefore);

    const promptBox = await findPromptBox(page, attempts);
    const sendButton = await findSendButton(page, attempts);
    if (promptBox && sendButton) return result("ready", startedAt, null, attempts, null);

    return result("failed", startedAt, null, attempts, "ready_contract_not_satisfied");
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    const status = /closed|crash|target page/i.test(message) ? "browser_crashed" : "failed";
    return result(status, startedAt, null, attempts, message);
  }
}
```

Note: this baseline readiness probe is intentionally artifact-free until Task 7 creates `finalizeResult`; Task 7 replaces it with the artifact-finalizing version.

- [x] **Step 4: Verify baseline tests pass**

Run:

```powershell
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/dom-only-baseline.spec.ts
```

Expected: PASS for 4 tests.

- [x] **Step 5: Commit baseline adapter**

Run:

```powershell
git add research/gemini_browser_adapter/src/dom-contract.ts research/gemini_browser_adapter/tests/dom-only-baseline.spec.ts
git commit -m "Add DOM-only Gemini adapter baseline"
```

---

### Task 5: Selector Config Overrides

**Files:**
- Create: `research/gemini_browser_adapter/gemini-dom-contract.config.json`
- Create: `research/gemini_browser_adapter/src/config.ts`
- Create: `research/gemini_browser_adapter/src/config.test.ts`
- Modify: `research/gemini_browser_adapter/src/dom-contract.ts`

- [x] **Step 1: Write failing config override tests**

Create `research/gemini_browser_adapter/src/config.test.ts`:

```ts
import { mkdtemp, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { DEFAULT_DOM_CONTRACT_CONFIG, loadDomContractConfig } from "./config";

describe("DOM contract config", () => {
  it("returns defaults when config is absent", async () => {
    const config = await loadDomContractConfig("missing-config.json");
    expect(config.answerSelectors).toEqual(DEFAULT_DOM_CONTRACT_CONFIG.answerSelectors);
  });

  it("loads local selector overrides", async () => {
    const dir = await mkdtemp(path.join(os.tmpdir(), "gemini-dom-contract-"));
    const configPath = path.join(dir, "gemini-dom-contract.config.json");
    await writeFile(
      configPath,
      JSON.stringify({
        promptSelectors: ["[data-custom-prompt]"],
        sendSelectors: ["[data-custom-send]"],
        answerSelectors: ["[data-custom-answer]"],
      }),
      "utf8",
    );

    const config = await loadDomContractConfig(configPath);
    expect(config.promptSelectors).toContain("[data-custom-prompt]");
    expect(config.sendSelectors).toContain("[data-custom-send]");
    expect(config.answerSelectors).toContain("[data-custom-answer]");
  });
});
```

- [x] **Step 2: Run config tests to verify failure**

Run:

```powershell
npm run test:gemini-browser-adapter:unit -- research/gemini_browser_adapter/src/config.test.ts
```

Expected: FAIL because `config.ts` does not exist.

- [x] **Step 3: Add default selector override file**

Create `research/gemini_browser_adapter/gemini-dom-contract.config.json`:

```json
{
  "promptSelectors": [],
  "sendSelectors": [],
  "answerSelectors": ["[data-testid=\"assistant-answer\"]"],
  "minPromptScore": 5,
  "minSendScore": 4
}
```

- [x] **Step 4: Implement config loader**

Create `research/gemini_browser_adapter/src/config.ts`:

```ts
import { readFile } from "node:fs/promises";

export type DomContractConfig = {
  promptSelectors: string[];
  sendSelectors: string[];
  answerSelectors: string[];
  minPromptScore: number;
  minSendScore: number;
};

export const DEFAULT_DOM_CONTRACT_CONFIG: DomContractConfig = {
  promptSelectors: [],
  sendSelectors: [],
  answerSelectors: ["[data-testid=\"assistant-answer\"]"],
  minPromptScore: 5,
  minSendScore: 4,
};

export async function loadDomContractConfig(
  configPath = "research/gemini_browser_adapter/gemini-dom-contract.config.json",
): Promise<DomContractConfig> {
  const raw = await readFile(configPath, "utf8").catch(() => null);
  if (!raw) return DEFAULT_DOM_CONTRACT_CONFIG;
  const parsed = JSON.parse(raw) as Partial<DomContractConfig>;
  return {
    promptSelectors: parsed.promptSelectors ?? DEFAULT_DOM_CONTRACT_CONFIG.promptSelectors,
    sendSelectors: parsed.sendSelectors ?? DEFAULT_DOM_CONTRACT_CONFIG.sendSelectors,
    answerSelectors: parsed.answerSelectors ?? DEFAULT_DOM_CONTRACT_CONFIG.answerSelectors,
    minPromptScore: parsed.minPromptScore ?? DEFAULT_DOM_CONTRACT_CONFIG.minPromptScore,
    minSendScore: parsed.minSendScore ?? DEFAULT_DOM_CONTRACT_CONFIG.minSendScore,
  };
}
```

- [x] **Step 5: Wire config into adapter options**

Modify `SendSingleOptions` in `dom-contract.ts`:

```ts
import type { DomContractConfig } from "./config";
import { loadDomContractConfig } from "./config";
```

```ts
export type SendSingleOptions = {
  timeoutMs: number;
  quietMs: number;
  configPath?: string;
  contractConfig?: DomContractConfig;
};
```

Add helper:

```ts
async function resolveContractConfig(options: SendSingleOptions): Promise<DomContractConfig> {
  return options.contractConfig ?? (await loadDomContractConfig(options.configPath));
}
```

Update answer detection to accept config-provided answer selectors before built-in fallbacks:

```ts
async function latestAnswerText(page: Page, config: DomContractConfig): Promise<string | null> {
  const selectors = [
    ...config.answerSelectors,
    '[data-testid*="assistant" i]',
    '[data-testid*="response" i]',
    "article.answer",
    "[data-answer]",
  ];
  const answer = page.locator(selectors.join(", "));
  if ((await answer.count().catch(() => 0)) === 0) return null;
  return (await answer.last().innerText().catch(() => "")).trim();
}
```

Thread `config` through `waitForFinalAnswer` and resolve it once at the start of each adapter call.

- [x] **Step 6: Verify config tests pass**

Run:

```powershell
npm run test:gemini-browser-adapter:unit -- research/gemini_browser_adapter/src/config.test.ts
```

Expected: PASS for 2 tests.

- [x] **Step 7: Commit config overrides**

Run:

```powershell
git add research/gemini_browser_adapter/gemini-dom-contract.config.json research/gemini_browser_adapter/src/config.ts research/gemini_browser_adapter/src/config.test.ts research/gemini_browser_adapter/src/dom-contract.ts
git commit -m "Add Gemini DOM contract selector config"
```

---

### Task 6: Deterministic Locator Scoring

**Files:**
- Create: `research/gemini_browser_adapter/src/scoring.ts`
- Create: `research/gemini_browser_adapter/src/scoring.test.ts`
- Modify: `research/gemini_browser_adapter/src/dom-contract.ts`
- Create: `research/gemini_browser_adapter/tests/resilient-scoring.spec.ts`

- [x] **Step 1: Write failing scoring unit tests**

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

- [x] **Step 2: Run scoring test to verify failure**

Run:

```powershell
npm run test:gemini-browser-adapter:unit -- research/gemini_browser_adapter/src/scoring.test.ts
```

Expected: FAIL because `scoring.ts` does not exist.

- [x] **Step 3: Implement scoring helpers**

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

- [x] **Step 4: Verify scoring unit tests pass**

Run:

```powershell
npm run test:gemini-browser-adapter:unit -- research/gemini_browser_adapter/src/scoring.test.ts
```

Expected: PASS for 3 tests.

- [x] **Step 5: Add resilient-scoring e2e tests**

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
```

- [x] **Step 6: Run resilient tests to verify failure**

Run:

```powershell
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/resilient-scoring.spec.ts
```

Expected: FAIL because `sendSingleResilientScoring` is not exported.

- [x] **Step 7: Add resilient scoring variant**

Modify `research/gemini_browser_adapter/src/dom-contract.ts` by importing scoring helpers:

```ts
import { scoreButtonCandidate, scoreEditableCandidate } from "./scoring";
```

Add these functions below `findSendButton`:

```ts
async function findByConfiguredSelector(
  page: Page,
  selectors: string[],
  attempts: LocatorAttempt[],
  name: string,
): Promise<Locator | null> {
  for (const selector of selectors) {
    const locator = page.locator(selector);
    const count = await locator.count().catch(() => 0);
    const visible = await firstVisible(locator);
    attempts.push({ name, strategy: "css", matched: Boolean(visible), count });
    if (visible) return visible;
  }
  return null;
}

async function findPromptBoxByScoring(page: Page, attempts: LocatorAttempt[], minScore: number): Promise<Locator | null> {
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
    return best;
  });

  const matched = bestIndex.score >= minScore && bestIndex.index >= 0;
  attempts.push({ name: "fuzzy:editable", strategy: "fuzzy", matched, score: bestIndex.score });
  return matched ? page.locator("textarea, input, [contenteditable='true'], [role='textbox']").nth(bestIndex.index) : null;
}

async function findSendButtonByScoring(page: Page, attempts: LocatorAttempt[], minScore: number): Promise<Locator | null> {
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
    return best;
  });

  const matched = bestIndex.score >= minScore && bestIndex.index >= 0;
  attempts.push({ name: "fuzzy:send-button", strategy: "fuzzy", matched, score: bestIndex.score });
  return matched ? page.locator("button, [role='button'], [data-send]").nth(bestIndex.index) : null;
}
```

Add this exported function at the end of the file:

```ts
export async function sendSingleResilientScoring(page: Page, prompt: string, options: SendSingleOptions): Promise<GeminiAdapterResult> {
  const startedAt = Date.now();
  const attempts: LocatorAttempt[] = [];
  void scoreEditableCandidate;
  void scoreButtonCandidate;
  const config = await resolveContractConfig(options);

  if (page.isClosed()) {
    return { ...result("browser_crashed", startedAt, null, attempts, "browser_crashed"), variant: "resilient-scoring" };
  }

  const criticalBefore = await scanCriticalState(page);
  if (criticalBefore) return { ...result(criticalBefore, startedAt, null, attempts, criticalBefore), variant: "resilient-scoring" };

  const promptBox =
    (await findByConfiguredSelector(page, config.promptSelectors, attempts, "config:prompt")) ??
    (await findPromptBox(page, attempts)) ??
    (await findPromptBoxByScoring(page, attempts, config.minPromptScore));
  if (!promptBox) return { ...result("failed", startedAt, null, attempts, "prompt_input_not_found"), variant: "resilient-scoring" };

  await typePrompt(promptBox, prompt);
  const sendButton =
    (await findByConfiguredSelector(page, config.sendSelectors, attempts, "config:send")) ??
    (await findSendButton(page, attempts)) ??
    (await findSendButtonByScoring(page, attempts, config.minSendScore));
  if (!sendButton) return { ...result("failed", startedAt, null, attempts, "send_button_not_found"), variant: "resilient-scoring" };
  await sendButton.click();

  const finalAnswer = await waitForFinalAnswer(page, startedAt, options, attempts, config);
  return {
    ...finalAnswer,
    variant: "resilient-scoring",
    locatorAttempts: attempts,
  };
}

export async function probeReadyResilientScoring(
  page: Page,
  options: SendSingleOptions = { timeoutMs: 1_000, quietMs: 200 },
): Promise<GeminiAdapterResult> {
  const startedAt = Date.now();
  const attempts: LocatorAttempt[] = [];
  void scoreEditableCandidate;
  void scoreButtonCandidate;
  const config = await resolveContractConfig(options);

  if (page.isClosed()) {
    return { ...result("browser_crashed", startedAt, null, attempts, "browser_crashed"), variant: "resilient-scoring" };
  }

  try {
    const criticalBefore = await scanCriticalState(page);
    if (criticalBefore) return { ...result(criticalBefore, startedAt, null, attempts, criticalBefore), variant: "resilient-scoring" };

    const promptBox =
      (await findByConfiguredSelector(page, config.promptSelectors, attempts, "config:prompt")) ??
      (await findPromptBox(page, attempts)) ??
      (await findPromptBoxByScoring(page, attempts, config.minPromptScore));
    const sendButton =
      (await findByConfiguredSelector(page, config.sendSelectors, attempts, "config:send")) ??
      (await findSendButton(page, attempts)) ??
      (await findSendButtonByScoring(page, attempts, config.minSendScore));
    if (promptBox && sendButton) return { ...result("ready", startedAt, null, attempts, null), variant: "resilient-scoring" };

    return { ...result("failed", startedAt, null, attempts, "ready_contract_not_satisfied"), variant: "resilient-scoring" };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    const status = /closed|crash|target page/i.test(message) ? "browser_crashed" : "failed";
    return { ...result(status, startedAt, null, attempts, message), variant: "resilient-scoring" };
  }
}
```

Note: this resilient readiness probe is intentionally artifact-free until Task 7 creates `finalizeResult`; Task 7 replaces it with the artifact-finalizing version.

- [x] **Step 8: Verify scoring adapter tests pass**

Run:

```powershell
npm run test:gemini-browser-adapter:unit -- research/gemini_browser_adapter/src/scoring.test.ts
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/resilient-scoring.spec.ts research/gemini_browser_adapter/tests/dom-only-baseline.spec.ts
```

Expected: PASS for scoring unit tests and both e2e files.

- [x] **Step 9: Commit resilient scoring**

Run:

```powershell
git add research/gemini_browser_adapter/src/scoring.ts research/gemini_browser_adapter/src/scoring.test.ts research/gemini_browser_adapter/src/dom-contract.ts research/gemini_browser_adapter/tests/resilient-scoring.spec.ts
git commit -m "Add resilient scoring Gemini adapter variant"
```

---

### Task 7: Failure Artifacts

**Files:**
- Create: `research/gemini_browser_adapter/src/redaction.ts`
- Create: `research/gemini_browser_adapter/src/artifacts.ts`
- Create: `research/gemini_browser_adapter/tests/failure-artifacts.spec.ts`
- Modify: `research/gemini_browser_adapter/src/dom-contract.ts`

- [x] **Step 1: Write failing artifact e2e test**

Create `research/gemini_browser_adapter/tests/failure-artifacts.spec.ts`:

```ts
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
```

- [x] **Step 2: Run artifact test to verify failure**

Run:

```powershell
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/failure-artifacts.spec.ts
```

Expected: FAIL because `artifactDir` is not part of `SendSingleOptions` and artifact capture is not implemented.

- [x] **Step 3: Implement shared redaction helpers**

Create `research/gemini_browser_adapter/src/redaction.ts`:

```ts
import type { Page } from "@playwright/test";

export function redactUrl(url: string): string {
  return url.replace(/([?&](?:token|key|session|authuser|at|credential)=)[^&]+/gi, "$1<redacted>");
}

export async function reducedDomSnapshot(page: Page): Promise<string> {
  return await page.evaluate(() => {
    const safeAttributes = new Set(["role", "type", "data-testid", "aria-hidden"]);
    const serialize = (element: Element, depth = 0): string => {
      if (depth > 8) return "";
      if (element.matches("script, style, noscript")) return "";
      const attrs = Array.from(element.attributes)
        .filter((attribute) => safeAttributes.has(attribute.name))
        .map((attribute) => ` ${attribute.name}="${attribute.value.replaceAll('"', "&quot;")}"`)
        .join("");
      const children = Array.from(element.children)
        .map((child) => serialize(child, depth + 1))
        .join("");
      const tagName = element.tagName.toLowerCase();
      return `<${tagName}${attrs}>${children}</${tagName}>`;
    };

    return serialize(document.body).slice(0, 200_000);
  }).catch(() => "");
}
```

This reduced snapshot intentionally omits visible text, form values, labels, titles, prompt content, answer text, and account hints. It is for selector-shape diagnostics only.

- [x] **Step 4: Implement artifact writer**

Create `research/gemini_browser_adapter/src/artifacts.ts`:

```ts
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Page } from "@playwright/test";
import { redactUrl, reducedDomSnapshot } from "./redaction";
import type { FailureArtifacts, LocatorAttempt, NetworkEventSummary } from "./types";

export type CaptureFailureInput = {
  page: Page;
  artifactDir: string;
  reason: string;
  locatorAttempts: LocatorAttempt[];
  networkSummary: NetworkEventSummary[];
  artifactMode: "full" | "reduced";
};

function safePageUrl(page: Page): string {
  try {
    return redactUrl(page.url());
  } catch {
    return "about:blank#page-url-unavailable";
  }
}

async function safeHtmlSnapshot(input: CaptureFailureInput): Promise<string> {
  const fallback = '<!doctype html><html><body data-capture-error="page_content_unavailable"></body></html>';
  if (input.artifactMode === "reduced") {
    const reduced = await reducedDomSnapshot(input.page).catch(() => "");
    return reduced || fallback;
  }
  return await input.page.content().catch(() => fallback);
}

export async function captureFailureArtifacts(input: CaptureFailureInput): Promise<FailureArtifacts> {
  const screenshotPath = input.artifactMode === "reduced" ? null : path.join(input.artifactDir, "failure.png");
  const htmlPath = path.join(input.artifactDir, "page.html");
  const telemetryPath = path.join(input.artifactDir, "telemetry.json");
  const artifactDirReady = await mkdir(input.artifactDir, { recursive: true })
    .then(() => true)
    .catch(() => false);

  if (!artifactDirReady) {
    return {
      screenshotPath: null,
      htmlPath: null,
      telemetryPath: null,
      tracePath: null,
    };
  }

  const html = await safeHtmlSnapshot(input);
  const pageUrl = safePageUrl(input.page);

  if (screenshotPath) {
    await input.page.screenshot({ path: screenshotPath, fullPage: true }).catch(() => undefined);
  }
  const htmlWritten = await writeFile(htmlPath, html, "utf8")
    .then(() => true)
    .catch(() => false);
  const telemetryWritten = await writeFile(
    telemetryPath,
    JSON.stringify(
      {
        reason: input.reason,
        url: pageUrl,
        locatorAttempts: input.locatorAttempts,
        networkSummary: input.networkSummary,
        capturedAt: new Date().toISOString(),
      },
      null,
      2,
    ),
    "utf8",
  )
    .then(() => true)
    .catch(() => false);

  return {
    screenshotPath,
    htmlPath: htmlWritten ? htmlPath : null,
    telemetryPath: telemetryWritten ? telemetryPath : null,
    tracePath: null,
  };
}
```

- [x] **Step 5: Integrate artifact capture**

Modify `SendSingleOptions` in `dom-contract.ts`:

```ts
export type SendSingleOptions = {
  timeoutMs: number;
  quietMs: number;
  configPath?: string;
  contractConfig?: DomContractConfig;
  artifactDir?: string;
  artifactMode?: "full" | "reduced";
  networkSummary?: NetworkEventSummary[];
};
```

Import:

```ts
import { captureFailureArtifacts } from "./artifacts";
import type { DomContractConfig } from "./config";
import { isSuccessStatus } from "./types";
import type { NetworkEventSummary } from "./types";
```

Add helper:

```ts
async function withArtifacts(
  page: Page,
  base: GeminiAdapterResult,
  options: SendSingleOptions,
): Promise<GeminiAdapterResult> {
  if (!options.artifactDir || isSuccessStatus(base.status)) return base;
  return {
    ...base,
    artifacts: await captureFailureArtifacts({
      page,
      artifactDir: options.artifactDir,
      reason: base.errorReason ?? base.status,
      locatorAttempts: base.locatorAttempts,
      networkSummary: options.networkSummary ?? base.networkSummary,
      artifactMode: options.artifactMode ?? "full",
    }),
  };
}

async function finalizeResult(
  page: Page,
  base: GeminiAdapterResult,
  options: SendSingleOptions,
): Promise<GeminiAdapterResult> {
  const withNetworkSummary = {
    ...base,
    networkSummary: options.networkSummary ?? base.networkSummary,
  };
  return await withArtifacts(page, withNetworkSummary, options);
}
```

Use `artifactMode: "reduced"` for any live Gemini run. Reduced mode writes a sanitized DOM snapshot and skips screenshot capture. The default `"full"` mode is only for deterministic local mock pages where fixture HTML contains no real account, prompt, or session data.

Replace the resilient-scoring adapter body with this finalizing form so every early return goes through artifact capture:

```ts
export async function sendSingleResilientScoring(page: Page, prompt: string, options: SendSingleOptions): Promise<GeminiAdapterResult> {
  const startedAt = Date.now();
  const attempts: LocatorAttempt[] = [];
  void scoreEditableCandidate;
  void scoreButtonCandidate;
  const config = await resolveContractConfig(options);
  const complete = async (base: GeminiAdapterResult) =>
    await finalizeResult(page, { ...base, variant: "resilient-scoring", locatorAttempts: attempts }, options);

  if (page.isClosed()) {
    return await complete(result("browser_crashed", startedAt, null, attempts, "browser_crashed"));
  }

  const criticalBefore = await scanCriticalState(page);
  if (criticalBefore) return await complete(result(criticalBefore, startedAt, null, attempts, criticalBefore));

  const promptBox =
    (await findByConfiguredSelector(page, config.promptSelectors, attempts, "config:prompt")) ??
    (await findPromptBox(page, attempts)) ??
    (await findPromptBoxByScoring(page, attempts, config.minPromptScore));
  if (!promptBox) return await complete(result("failed", startedAt, null, attempts, "prompt_input_not_found"));

  await typePrompt(promptBox, prompt);
  const sendButton =
    (await findByConfiguredSelector(page, config.sendSelectors, attempts, "config:send")) ??
    (await findSendButton(page, attempts)) ??
    (await findSendButtonByScoring(page, attempts, config.minSendScore));
  if (!sendButton) return await complete(result("failed", startedAt, null, attempts, "send_button_not_found"));
  await sendButton.click();

  return await complete(await waitForFinalAnswer(page, startedAt, options, attempts, config));
}
```

Replace readiness probes with finalizing forms too. A successful `ready` result still skips artifact capture through `isSuccessStatus`; failed readiness probes must write artifacts when `artifactDir` is present.

```ts
export async function probeReadyDomOnly(
  page: Page,
  options: SendSingleOptions = { timeoutMs: 1_000, quietMs: 200 },
): Promise<GeminiAdapterResult> {
  const startedAt = Date.now();
  const attempts: LocatorAttempt[] = [];
  const complete = async (base: GeminiAdapterResult) => await finalizeResult(page, base, options);

  if (page.isClosed()) return await complete(result("browser_crashed", startedAt, null, attempts, "browser_crashed"));

  try {
    const criticalBefore = await scanCriticalState(page);
    if (criticalBefore) return await complete(result(criticalBefore, startedAt, null, attempts, criticalBefore));

    const promptBox = await findPromptBox(page, attempts);
    const sendButton = await findSendButton(page, attempts);
    if (promptBox && sendButton) return await complete(result("ready", startedAt, null, attempts, null));

    return await complete(result("failed", startedAt, null, attempts, "ready_contract_not_satisfied"));
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    const status = /closed|crash|target page/i.test(message) ? "browser_crashed" : "failed";
    return await complete(result(status, startedAt, null, attempts, message));
  }
}

export async function probeReadyResilientScoring(
  page: Page,
  options: SendSingleOptions = { timeoutMs: 1_000, quietMs: 200 },
): Promise<GeminiAdapterResult> {
  const startedAt = Date.now();
  const attempts: LocatorAttempt[] = [];
  void scoreEditableCandidate;
  void scoreButtonCandidate;
  const config = await resolveContractConfig(options);
  const complete = async (base: GeminiAdapterResult) =>
    await finalizeResult(page, { ...base, variant: "resilient-scoring", locatorAttempts: attempts }, options);

  if (page.isClosed()) {
    return await complete(result("browser_crashed", startedAt, null, attempts, "browser_crashed"));
  }

  try {
    const criticalBefore = await scanCriticalState(page);
    if (criticalBefore) return await complete(result(criticalBefore, startedAt, null, attempts, criticalBefore));

    const promptBox =
      (await findByConfiguredSelector(page, config.promptSelectors, attempts, "config:prompt")) ??
      (await findPromptBox(page, attempts)) ??
      (await findPromptBoxByScoring(page, attempts, config.minPromptScore));
    const sendButton =
      (await findByConfiguredSelector(page, config.sendSelectors, attempts, "config:send")) ??
      (await findSendButton(page, attempts)) ??
      (await findSendButtonByScoring(page, attempts, config.minSendScore));
    if (promptBox && sendButton) return await complete(result("ready", startedAt, null, attempts, null));

    return await complete(result("failed", startedAt, null, attempts, "ready_contract_not_satisfied"));
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    const status = /closed|crash|target page/i.test(message) ? "browser_crashed" : "failed";
    return await complete(result(status, startedAt, null, attempts, message));
  }
}
```

Apply the same `complete(...)` pattern to `sendSingleDomOnly` for its `browser_crashed`, critical-state, prompt-missing, send-missing, and final-answer returns. The matrix artifact cases must fail if any non-success send or probe result with `artifactDir` skips `finalizeResult`.

- [x] **Step 6: Verify artifact tests pass**

Run:

```powershell
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/failure-artifacts.spec.ts
```

Expected: PASS and files exist under `research/gemini_browser_adapter/artifacts/test-timeout`.

- [x] **Step 7: Confirm generated artifacts are ignored**

Run:

```powershell
git status --short --untracked-files=all research\\gemini_browser_adapter\\artifacts
```

Expected: no generated artifact files appear because `artifacts` is ignored except the tracked `.gitkeep`.

- [x] **Step 8: Commit artifacts**

Run:

```powershell
git add research/gemini_browser_adapter/src/redaction.ts research/gemini_browser_adapter/src/artifacts.ts research/gemini_browser_adapter/src/dom-contract.ts research/gemini_browser_adapter/tests/failure-artifacts.spec.ts
git commit -m "Add Gemini adapter failure artifacts"
```

---

### Task 8: Telemetry-Assisted Variant

**Files:**
- Create: `research/gemini_browser_adapter/src/telemetry.ts`
- Create: `research/gemini_browser_adapter/src/telemetry.test.ts`
- Modify: `research/gemini_browser_adapter/src/dom-contract.ts`
- Create: `research/gemini_browser_adapter/tests/telemetry-assisted.spec.ts`

- [x] **Step 1: Write failing telemetry unit tests**

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

- [x] **Step 2: Run telemetry test to verify failure**

Run:

```powershell
npm run test:gemini-browser-adapter:unit -- research/gemini_browser_adapter/src/telemetry.test.ts
```

Expected: FAIL because `telemetry.ts` does not exist.

- [x] **Step 3: Implement telemetry collector**

Create `research/gemini_browser_adapter/src/telemetry.ts`:

```ts
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
```

- [x] **Step 4: Verify telemetry unit tests pass**

Run:

```powershell
npm run test:gemini-browser-adapter:unit -- research/gemini_browser_adapter/src/telemetry.test.ts
```

Expected: PASS for 1 test.

- [x] **Step 5: Add telemetry-assisted e2e tests**

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

- [x] **Step 6: Run telemetry e2e tests to verify failure**

Run:

```powershell
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/telemetry-assisted.spec.ts
```

Expected: FAIL because `sendSingleTelemetryAssisted` is not exported.

- [x] **Step 7: Implement telemetry-assisted adapter**

Modify `dom-contract.ts`:

```ts
import { attachNetworkTelemetry } from "./telemetry";
```

Add at the end:

```ts
export async function sendSingleTelemetryAssisted(page: Page, prompt: string, options: SendSingleOptions): Promise<GeminiAdapterResult> {
  const networkSummary: GeminiAdapterResult["networkSummary"] = [];
  attachNetworkTelemetry(page, networkSummary);
  const result = await sendSingleResilientScoring(page, prompt, {
    ...options,
    networkSummary,
  });
  return {
    ...result,
    variant: "telemetry-assisted",
    networkSummary,
  };
}

export async function probeReadyTelemetryAssisted(
  page: Page,
  options: SendSingleOptions = { timeoutMs: 1_000, quietMs: 200 },
): Promise<GeminiAdapterResult> {
  const networkSummary: GeminiAdapterResult["networkSummary"] = [];
  attachNetworkTelemetry(page, networkSummary);
  const result = await probeReadyResilientScoring(page, {
    ...options,
    networkSummary,
  });
  return {
    ...result,
    variant: "telemetry-assisted",
    networkSummary,
  };
}
```

Do not capture telemetry-assisted artifacts in the wrapper after `sendSingleResilientScoring` returns. Pass `networkSummary` through `SendSingleOptions` as shown above so the shared `withArtifacts` call writes the same network summary that the telemetry-assisted result returns.

- [x] **Step 8: Verify telemetry tests pass**

Run:

```powershell
npm run test:gemini-browser-adapter:unit -- research/gemini_browser_adapter/src/telemetry.test.ts
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/telemetry-assisted.spec.ts
```

Expected: PASS for telemetry unit and e2e tests.

- [x] **Step 9: Commit telemetry-assisted variant**

Run:

```powershell
git add research/gemini_browser_adapter/src/telemetry.ts research/gemini_browser_adapter/src/telemetry.test.ts research/gemini_browser_adapter/src/dom-contract.ts research/gemini_browser_adapter/tests/telemetry-assisted.spec.ts
git commit -m "Add telemetry-assisted Gemini adapter variant"
```

---

### Task 9: Executable Matrix and Report Runner

**Files:**
- Create: `research/gemini_browser_adapter/matrix-cases.json`
- Create: `research/gemini_browser_adapter/src/matrix-cases.ts`
- Create: `research/gemini_browser_adapter/tests/matrix.spec.ts`
- Create: `research/gemini_browser_adapter/scripts/write-matrix-report.mjs`
- Modify: `research/gemini_browser_adapter/RESILIENCE_TEST_MATRIX.md`
- Modify: `research/gemini_browser_adapter/TOOLS_AND_METHODS.md`

- [x] **Step 1: Write executable matrix cases**

Create `research/gemini_browser_adapter/matrix-cases.json`:

```json
{
  "adapterVariants": ["dom-only", "resilient-scoring", "telemetry-assisted"],
  "scenarios": [
    {
      "id": "ready",
      "mockVariant": "ready",
      "action": "probe",
      "expectedStatuses": ["ready"],
      "timeoutMs": 1000,
      "quietMs": 200,
      "requiresRawText": false,
      "requiresTelemetryArtifact": false,
      "requiresHtmlArtifact": false,
      "requiresScreenshotArtifact": false,
      "requiresTelemetryNetwork": false,
      "closePageBeforeRun": false
    },
    {
      "id": "ready-missing-send",
      "mockVariant": "ready-missing-send",
      "action": "probe",
      "expectedStatuses": ["failed"],
      "timeoutMs": 1000,
      "quietMs": 200,
      "requiresRawText": false,
      "requiresTelemetryArtifact": true,
      "requiresHtmlArtifact": true,
      "requiresScreenshotArtifact": true,
      "requiresTelemetryNetwork": false,
      "closePageBeforeRun": false
    },
    {
      "id": "ready-broken",
      "mockVariant": "ready-broken",
      "action": "probe",
      "expectedStatuses": ["failed"],
      "timeoutMs": 1000,
      "quietMs": 200,
      "requiresRawText": false,
      "requiresTelemetryArtifact": true,
      "requiresHtmlArtifact": true,
      "requiresScreenshotArtifact": true,
      "requiresTelemetryNetwork": false,
      "closePageBeforeRun": false
    },
    {
      "id": "happy-path",
      "mockVariant": "happy-path",
      "action": "send",
      "expectedStatuses": ["ok"],
      "timeoutMs": 5000,
      "quietMs": 300,
      "requiresRawText": true,
      "requiresTelemetryArtifact": false,
      "requiresHtmlArtifact": false,
      "requiresScreenshotArtifact": false,
      "requiresTelemetryNetwork": true,
      "closePageBeforeRun": false
    },
    {
      "id": "wrapped-dom",
      "mockVariant": "wrapped-dom",
      "action": "send",
      "expectedStatuses": ["ok"],
      "timeoutMs": 5000,
      "quietMs": 300,
      "requiresRawText": true,
      "requiresTelemetryArtifact": false,
      "requiresHtmlArtifact": false,
      "requiresScreenshotArtifact": false,
      "requiresTelemetryNetwork": false,
      "closePageBeforeRun": false
    },
    {
      "id": "textarea-input",
      "mockVariant": "textarea-input",
      "action": "send",
      "expectedStatuses": ["ok"],
      "timeoutMs": 5000,
      "quietMs": 300,
      "requiresRawText": true,
      "requiresTelemetryArtifact": false,
      "requiresHtmlArtifact": false,
      "requiresScreenshotArtifact": false,
      "requiresTelemetryNetwork": false,
      "closePageBeforeRun": false
    },
    {
      "id": "contenteditable-input",
      "mockVariant": "contenteditable-input",
      "action": "send",
      "expectedStatuses": ["ok"],
      "timeoutMs": 5000,
      "quietMs": 300,
      "requiresRawText": true,
      "requiresTelemetryArtifact": false,
      "requiresHtmlArtifact": false,
      "requiresScreenshotArtifact": false,
      "requiresTelemetryNetwork": false,
      "closePageBeforeRun": false
    },
    {
      "id": "icon-send",
      "mockVariant": "icon-send",
      "action": "send",
      "expectedStatuses": ["ok"],
      "timeoutMs": 5000,
      "quietMs": 300,
      "requiresRawText": true,
      "requiresTelemetryArtifact": false,
      "requiresHtmlArtifact": false,
      "requiresScreenshotArtifact": false,
      "requiresTelemetryNetwork": false,
      "closePageBeforeRun": false
    },
    {
      "id": "slow-pauses",
      "mockVariant": "slow-pauses",
      "action": "send",
      "expectedStatuses": ["ok"],
      "timeoutMs": 6000,
      "quietMs": 700,
      "requiresRawText": true,
      "requiresTelemetryArtifact": false,
      "requiresHtmlArtifact": false,
      "requiresScreenshotArtifact": false,
      "requiresTelemetryNetwork": false,
      "closePageBeforeRun": false
    },
    {
      "id": "never-stable",
      "mockVariant": "never-stable",
      "action": "send",
      "expectedStatuses": ["generation_timeout"],
      "timeoutMs": 1500,
      "quietMs": 300,
      "requiresRawText": false,
      "requiresTelemetryArtifact": true,
      "requiresHtmlArtifact": true,
      "requiresScreenshotArtifact": true,
      "requiresTelemetryNetwork": false,
      "closePageBeforeRun": false
    },
    {
      "id": "login-required",
      "mockVariant": "login-required",
      "action": "send",
      "expectedStatuses": ["login_required"],
      "timeoutMs": 2000,
      "quietMs": 300,
      "requiresRawText": false,
      "requiresTelemetryArtifact": true,
      "requiresHtmlArtifact": true,
      "requiresScreenshotArtifact": true,
      "requiresTelemetryNetwork": false,
      "closePageBeforeRun": false
    },
    {
      "id": "captcha",
      "mockVariant": "captcha",
      "action": "send",
      "expectedStatuses": ["captcha_required"],
      "timeoutMs": 2000,
      "quietMs": 300,
      "requiresRawText": false,
      "requiresTelemetryArtifact": true,
      "requiresHtmlArtifact": true,
      "requiresScreenshotArtifact": true,
      "requiresTelemetryNetwork": false,
      "closePageBeforeRun": false
    },
    {
      "id": "account-picker",
      "mockVariant": "account-picker",
      "action": "send",
      "expectedStatuses": ["account_picker"],
      "timeoutMs": 2000,
      "quietMs": 300,
      "requiresRawText": false,
      "requiresTelemetryArtifact": true,
      "requiresHtmlArtifact": true,
      "requiresScreenshotArtifact": true,
      "requiresTelemetryNetwork": false,
      "closePageBeforeRun": false
    },
    {
      "id": "consent",
      "mockVariant": "consent",
      "action": "send",
      "expectedStatuses": ["consent_required"],
      "timeoutMs": 2000,
      "quietMs": 300,
      "requiresRawText": false,
      "requiresTelemetryArtifact": true,
      "requiresHtmlArtifact": true,
      "requiresScreenshotArtifact": true,
      "requiresTelemetryNetwork": false,
      "closePageBeforeRun": false
    },
    {
      "id": "rate-limit",
      "mockVariant": "rate-limit",
      "action": "send",
      "expectedStatuses": ["rate_limited"],
      "timeoutMs": 2000,
      "quietMs": 300,
      "requiresRawText": false,
      "requiresTelemetryArtifact": true,
      "requiresHtmlArtifact": true,
      "requiresScreenshotArtifact": true,
      "requiresTelemetryNetwork": false,
      "closePageBeforeRun": false
    },
    {
      "id": "unknown-modal",
      "mockVariant": "unknown-modal",
      "action": "send",
      "expectedStatuses": ["manual_action_required"],
      "timeoutMs": 2000,
      "quietMs": 300,
      "requiresRawText": false,
      "requiresTelemetryArtifact": true,
      "requiresHtmlArtifact": true,
      "requiresScreenshotArtifact": true,
      "requiresTelemetryNetwork": false,
      "closePageBeforeRun": false
    },
    {
      "id": "broken-answer",
      "mockVariant": "broken-answer",
      "action": "send",
      "expectedStatuses": ["response_parse_failed"],
      "timeoutMs": 1500,
      "quietMs": 300,
      "requiresRawText": false,
      "requiresTelemetryArtifact": true,
      "requiresHtmlArtifact": true,
      "requiresScreenshotArtifact": true,
      "requiresTelemetryNetwork": false,
      "closePageBeforeRun": false
    },
    {
      "id": "closed-page",
      "mockVariant": "closed-page",
      "action": "send",
      "expectedStatuses": ["browser_crashed"],
      "timeoutMs": 1000,
      "quietMs": 200,
      "requiresRawText": false,
      "requiresTelemetryArtifact": true,
      "requiresHtmlArtifact": true,
      "requiresScreenshotArtifact": false,
      "requiresTelemetryNetwork": false,
      "closePageBeforeRun": true
    }
  ]
}
```

Create `research/gemini_browser_adapter/src/matrix-cases.ts`:

```ts
import rawMatrixDefinition from "../matrix-cases.json";
import type { AdapterVariant, GeminiAdapterStatus } from "./types";

export type MatrixAction = "probe" | "send";

export type MatrixScenario = {
  id: string;
  mockVariant: string;
  action: MatrixAction;
  expectedStatuses: GeminiAdapterStatus[];
  timeoutMs: number;
  quietMs: number;
  requiresRawText: boolean;
  requiresTelemetryArtifact: boolean;
  requiresHtmlArtifact: boolean;
  requiresScreenshotArtifact: boolean;
  requiresTelemetryNetwork: boolean;
  closePageBeforeRun: boolean;
};

type MatrixDefinition = {
  adapterVariants: AdapterVariant[];
  scenarios: MatrixScenario[];
};

const matrixDefinition = rawMatrixDefinition as MatrixDefinition;

export const matrixAdapterVariants = matrixDefinition.adapterVariants;
export const matrixScenarios = matrixDefinition.scenarios;

export function expectedMatrixPairTitles(): string[] {
  return matrixAdapterVariants.flatMap((variant) =>
    matrixScenarios.map((scenario) => `${variant} / ${scenario.id}`),
  );
}
```

- [x] **Step 2: Write matrix Playwright spec**

Create `research/gemini_browser_adapter/tests/matrix.spec.ts`:

```ts
import { existsSync } from "node:fs";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { expect, test } from "@playwright/test";
import { startMockGeminiServer } from "../mock-gemini/server.mjs";
import {
  probeReadyDomOnly,
  probeReadyResilientScoring,
  probeReadyTelemetryAssisted,
  sendSingleDomOnly,
  sendSingleResilientScoring,
  sendSingleTelemetryAssisted,
} from "../src/dom-contract";
import { matrixAdapterVariants, matrixScenarios } from "../src/matrix-cases";
import type { AdapterVariant, GeminiAdapterResult } from "../src/types";
import type { SendSingleOptions } from "../src/dom-contract";

type AdapterHarness = {
  send(page: Parameters<typeof sendSingleDomOnly>[0], prompt: string, options: SendSingleOptions): Promise<GeminiAdapterResult>;
  probe(page: Parameters<typeof sendSingleDomOnly>[0], options: SendSingleOptions): Promise<GeminiAdapterResult>;
};

const adapters: Record<AdapterVariant, AdapterHarness> = {
  "dom-only": {
    send: sendSingleDomOnly,
    probe: probeReadyDomOnly,
  },
  "resilient-scoring": {
    send: sendSingleResilientScoring,
    probe: probeReadyResilientScoring,
  },
  "telemetry-assisted": {
    send: sendSingleTelemetryAssisted,
    probe: probeReadyTelemetryAssisted,
  },
};

let server: Awaited<ReturnType<typeof startMockGeminiServer>>;

test.beforeAll(async () => {
  server = await startMockGeminiServer();
});

test.afterAll(async () => {
  await server.stop();
});

test.describe("Gemini adapter executable scenario matrix", () => {
  for (const variant of matrixAdapterVariants) {
    for (const scenario of matrixScenarios) {
      test(`${variant} / ${scenario.id}`, async ({ page }) => {
        await page.goto(server.url(scenario.mockVariant));
        if (scenario.closePageBeforeRun) {
          await page.close();
        }

        const artifactDir = path.join(
          "research/gemini_browser_adapter/artifacts/matrix",
          variant,
          scenario.id,
        );

        const result =
          scenario.action === "probe"
            ? await adapters[variant].probe(page, {
                timeoutMs: scenario.timeoutMs,
                quietMs: scenario.quietMs,
                artifactDir,
              })
            : await adapters[variant].send(page, "hello from matrix", {
                timeoutMs: scenario.timeoutMs,
                quietMs: scenario.quietMs,
                artifactDir,
              });

        const hasScreenshotArtifact = Boolean(result.artifacts?.screenshotPath && existsSync(result.artifacts.screenshotPath));
        const hasHtmlArtifact = Boolean(result.artifacts?.htmlPath && existsSync(result.artifacts.htmlPath));
        const hasTelemetryArtifact = Boolean(result.artifacts?.telemetryPath && existsSync(result.artifacts.telemetryPath));

        await mkdir(artifactDir, { recursive: true });
        await writeFile(
          path.join(artifactDir, "result.json"),
          JSON.stringify(
            {
              variant,
              scenarioId: scenario.id,
              status: result.status,
              expectedStatuses: scenario.expectedStatuses,
              elapsedMs: result.elapsedMs,
              rawTextPresent: Boolean((result.rawText ?? "").trim()),
              artifacts: {
                screenshot: hasScreenshotArtifact,
                html: hasHtmlArtifact,
                telemetry: hasTelemetryArtifact,
              },
              expectedArtifacts: {
                screenshot: scenario.requiresScreenshotArtifact,
                html: scenario.requiresHtmlArtifact,
                telemetry: scenario.requiresTelemetryArtifact,
              },
              falseCompletion: result.status === "ok" && !scenario.expectedStatuses.includes("ok"),
              unexpectedStatus: !scenario.expectedStatuses.includes(result.status),
              timeoutOrHang: result.status === "generation_timeout" || result.elapsedMs >= scenario.timeoutMs,
            },
            null,
            2,
          ),
          "utf8",
        );

        expect(result.variant).toBe(variant);
        expect(scenario.expectedStatuses).toContain(result.status);

        if (scenario.requiresRawText) {
          expect(result.rawText ?? "").toContain("Mock final answer");
        } else {
          expect(result.rawText ?? "").not.toContain("Mock final answer");
        }

        if (scenario.requiresScreenshotArtifact) expect(hasScreenshotArtifact).toBe(true);
        if (scenario.requiresHtmlArtifact) expect(hasHtmlArtifact).toBe(true);
        if (scenario.requiresTelemetryArtifact) expect(hasTelemetryArtifact).toBe(true);

        if (scenario.requiresTelemetryNetwork && variant === "telemetry-assisted") {
          expect(result.networkSummary.some((event) => event.kind === "response")).toBe(true);
          expect(result.networkSummary.some((event) => event.url.includes("/mock-gemini-event"))).toBe(true);
        }
      });
    }
  }
});
```

- [x] **Step 3: Run matrix spec to verify failure**

Run:

```powershell
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/matrix.spec.ts
```

Expected: FAIL until `matrix-cases.ts`, readiness probes, closed-page status handling, and artifact wrapping from previous tasks are implemented.

- [x] **Step 4: Verify matrix spec passes**

Run:

```powershell
npx playwright test -c research/gemini_browser_adapter/playwright.config.ts research/gemini_browser_adapter/tests/matrix.spec.ts
```

Expected:

- exactly `54` matrix cases run (`3` adapter variants x `18` scenarios);
- `ready`, `ready-missing-send`, `ready-broken`, `closed-page`, `account-picker`, `consent`, `unknown-modal`, and `broken-answer` are represented by explicit test titles;
- every case asserts expected adapter status;
- every `requiresRawText` case asserts final answer text;
- every granular artifact flag asserts only the required screenshot, HTML, or telemetry artifact file;
- the `telemetry-assisted / happy-path` case asserts a response event for `/mock-gemini-event`.

- [x] **Step 5: Write coverage-validating matrix report script**

Create `research/gemini_browser_adapter/scripts/write-matrix-report.mjs`:

```js
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";

const artifactDir = "research/gemini_browser_adapter/artifacts";
const inputPath = path.join(artifactDir, "playwright-results.json");
const matrixPath = "research/gemini_browser_adapter/matrix-cases.json";
const outputPath = path.join(artifactDir, "matrix-report.md");
const matrixResultDir = path.join(artifactDir, "matrix");

if (!existsSync(matrixPath)) {
  console.error(`Missing matrix metadata at ${matrixPath}`);
  process.exit(1);
}

const matrixDefinition = JSON.parse(readFileSync(matrixPath, "utf8"));
const expectedPairs = matrixDefinition.adapterVariants.flatMap((variant) =>
  matrixDefinition.scenarios.map((scenario) => `${variant} / ${scenario.id}`),
);

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

function collectResultFiles(dir, files = []) {
  if (!existsSync(dir)) return files;
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const entryPath = path.join(dir, entry.name);
    if (entry.isDirectory()) collectResultFiles(entryPath, files);
    else if (entry.name === "result.json") files.push(entryPath);
  }
  return files;
}

function average(values) {
  if (values.length === 0) return 0;
  return Math.round(values.reduce((sum, value) => sum + value, 0) / values.length);
}

function isSuccessStatus(status) {
  return status === "ok" || status === "ready";
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
const missingPairs = expectedPairs.filter((pair) => !rows.some((row) => row.title.endsWith(pair)));
const observedPairs = expectedPairs.length - missingPairs.length;
const resultRows = collectResultFiles(matrixResultDir).map((filePath) => JSON.parse(readFileSync(filePath, "utf8")));
const resultPairs = new Set(resultRows.map((row) => `${row.variant} / ${row.scenarioId}`));
const missingResultPairs = expectedPairs.filter((pair) => !resultPairs.has(pair));
const successCount = resultRows.filter((row) => isSuccessStatus(row.status)).length;
const okCount = resultRows.filter((row) => row.status === "ok").length;
const readyCount = resultRows.filter((row) => row.status === "ready").length;
const cleanTypedFailureCount = resultRows.filter((row) => !isSuccessStatus(row.status) && row.expectedStatuses.includes(row.status)).length;
const unexpectedFailureCount = resultRows.filter((row) => row.unexpectedStatus).length;
const timeoutOrHangCount = resultRows.filter((row) => row.timeoutOrHang).length;
const falseCompletionCount = resultRows.filter((row) => row.falseCompletion).length;
const artifactIncompleteCount = resultRows.filter((row) =>
  Object.entries(row.expectedArtifacts).some(([name, required]) => required && !row.artifacts[name]),
).length;
const averageElapsedMs = average(resultRows.map((row) => row.elapsedMs));
const worstElapsedMs = resultRows.reduce((max, row) => Math.max(max, row.elapsedMs ?? 0), 0);
const variants = matrixDefinition.adapterVariants.map((variant) => {
  const rowsForVariant = resultRows.filter((row) => row.variant === variant);
  return {
    variant,
    success: rowsForVariant.filter((row) => isSuccessStatus(row.status)).length,
    ok: rowsForVariant.filter((row) => row.status === "ok").length,
    ready: rowsForVariant.filter((row) => row.status === "ready").length,
    typedFailure: rowsForVariant.filter((row) => !isSuccessStatus(row.status) && row.expectedStatuses.includes(row.status)).length,
    unexpected: rowsForVariant.filter((row) => row.unexpectedStatus).length,
    falseCompletion: rowsForVariant.filter((row) => row.falseCompletion).length,
    averageElapsedMs: average(rowsForVariant.map((row) => row.elapsedMs)),
    worstElapsedMs: rowsForVariant.reduce((max, row) => Math.max(max, row.elapsedMs ?? 0), 0),
  };
});

const report = [
  "# Gemini Browser Adapter Matrix Report",
  "",
  `Generated: ${new Date().toISOString()}`,
  "",
  `Total tests: ${rows.length}`,
  `Passed tests: ${passed}`,
  `Failed or unexpected tests: ${failed}`,
  `Expected matrix pairs: ${expectedPairs.length}`,
  `Observed matrix pairs: ${observedPairs}`,
  `Missing matrix pairs: ${missingPairs.length}`,
  `Missing result files: ${missingResultPairs.length}`,
  `Worst Playwright duration ms: ${worst}`,
  "",
  "## Adapter Result Metrics",
  "",
  `Success count: ${successCount}`,
  `OK count: ${okCount}`,
  `Ready count: ${readyCount}`,
  `Clean typed failure count: ${cleanTypedFailureCount}`,
  `Unexpected failure count: ${unexpectedFailureCount}`,
  `Timeout/hang count: ${timeoutOrHangCount}`,
  `Required artifact incomplete count: ${artifactIncompleteCount}`,
  `False completion count: ${falseCompletionCount}`,
  `Average elapsed ms: ${averageElapsedMs}`,
  `Worst elapsed ms: ${worstElapsedMs}`,
  "",
  "| Variant | Success | OK | Ready | Clean Typed Failure | Unexpected | False Completion | Avg ms | Worst ms |",
  "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
  ...variants.map((row) =>
    `| ${row.variant} | ${row.success} | ${row.ok} | ${row.ready} | ${row.typedFailure} | ${row.unexpected} | ${row.falseCompletion} | ${row.averageElapsedMs} | ${row.worstElapsedMs} |`,
  ),
  "",
  "## Matrix Coverage",
  "",
  missingPairs.length === 0 && missingResultPairs.length === 0
    ? "All expected variant/scenario pairs and result files were present."
    : [...missingPairs.map((pair) => `- Missing Playwright row: ${pair}`), ...missingResultPairs.map((pair) => `- Missing result file: ${pair}`)].join("\n"),
  "",
  "| Test | Status | Duration ms |",
  "| --- | --- | ---: |",
  ...rows.map((row) => `| ${row.title.replaceAll("|", "\\|")} | ${row.status} | ${row.duration} |`),
  "",
].join("\n");

await mkdir(artifactDir, { recursive: true });
await writeFile(outputPath, report, "utf8");
console.log(`Wrote ${outputPath}`);
if (failed > 0 || missingPairs.length > 0 || missingResultPairs.length > 0 || unexpectedFailureCount > 0 || artifactIncompleteCount > 0 || falseCompletionCount > 0) process.exit(1);
```

- [x] **Step 6: Run full research verification**

Run:

```powershell
npm run test:gemini-browser-adapter
```

Expected:

- research TypeScript typecheck passes;
- unit tests pass;
- Playwright e2e tests pass;
- `research/gemini_browser_adapter/tests/matrix.spec.ts` runs all `54` matrix cases;
- `research/gemini_browser_adapter/artifacts/playwright-results.json` is written;
- `research/gemini_browser_adapter/artifacts/matrix-report.md` is written;
- the report has `Missing matrix pairs: 0`;
- command exits `0`.

- [x] **Step 7: Update matrix documentation**

Append this section to `research/gemini_browser_adapter/RESILIENCE_TEST_MATRIX.md`:

````markdown
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

The executable matrix is implemented in:

```text
research/gemini_browser_adapter/matrix-cases.json
research/gemini_browser_adapter/src/matrix-cases.ts
research/gemini_browser_adapter/tests/matrix.spec.ts
```

The matrix JSON is the single source of truth for adapter variants and scenario IDs. `matrix-cases.ts` imports it for Playwright tests, and `write-matrix-report.mjs` reads the same file for coverage validation. The matrix covers all `3` adapter variants against all `18` scenarios. Expected statuses and required evidence are asserted in `matrix.spec.ts`; report generation fails when any expected variant/scenario pair is absent from the Playwright JSON output.
````

- [x] **Step 8: Update tools document**

Add this sentence under `## Method` in `research/gemini_browser_adapter/TOOLS_AND_METHODS.md`:

```markdown
The planned executable command is `npm run test:gemini-browser-adapter`. It
should use a wrapper runner so the matrix report is still generated after a
Playwright e2e failure. The wrapper should clear stale matrix `result.json`
files and stale Playwright JSON output immediately before each e2e run.
```

- [x] **Step 9: Verify no generated artifacts are staged**

Run:

```powershell
git status --short --untracked-files=all research\\gemini_browser_adapter
```

Expected: source files and docs may appear, but generated files under `artifacts/` do not appear.

- [x] **Step 10: Commit matrix runner**

Run:

```powershell
git add research/gemini_browser_adapter/matrix-cases.json research/gemini_browser_adapter/src/matrix-cases.ts research/gemini_browser_adapter/tests/matrix.spec.ts research/gemini_browser_adapter/scripts/write-matrix-report.mjs research/gemini_browser_adapter/RESILIENCE_TEST_MATRIX.md research/gemini_browser_adapter/TOOLS_AND_METHODS.md
git commit -m "Add Gemini adapter research matrix report"
```

---

### Task 10: Final Research Harness Verification

**Files:**
- No new files.
- Verify all files created by previous tasks.

- [ ] **Step 1: Run complete research matrix**

Run:

```powershell
npm run test:gemini-browser-adapter
```

Expected: command exits `0`. This includes `npm run test:gemini-browser-adapter:typecheck`.

- [ ] **Step 2: Run research TypeScript check directly**

Run:

```powershell
npm run test:gemini-browser-adapter:typecheck
```

Expected: command exits `0` and typechecks all TypeScript files under `research/gemini_browser_adapter`.

- [ ] **Step 3: Run Svelte/TypeScript project check**

Run:

```powershell
npm run check
```

Expected: command exits `0`. If unrelated pre-existing `src-tauri` changes do not affect this command, keep them out of this research commit.

- [ ] **Step 4: Review generated matrix report**

Run:

```powershell
Get-Content -LiteralPath 'research\\gemini_browser_adapter\\artifacts\\matrix-report.md'
```

Expected: report lists all research tests and has `Failed or unexpected tests: 0` and `Missing matrix pairs: 0`.

- [ ] **Step 5: Confirm production Rust/Tauri was untouched by research commits**

Run:

```powershell
git diff --name-only HEAD -- src-tauri
```

Expected: no output for changes introduced by this research plan. If user-owned `src-tauri` edits exist in the working tree, leave them unstaged.

- [ ] **Step 6: Commit final plan doc update**

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
- The plan includes the local `gemini-dom-contract.config.json` selector override promised by `TOOLS_AND_METHODS.md`.
- The plan typechecks research TypeScript with `tsc -p research/gemini_browser_adapter/tsconfig.json --noEmit`.
- The research TypeScript config uses bundler module resolution, matching the extensionless TS imports in the research files.
- The plan runs all three variants against all eighteen matrix scenarios.
- The matrix report derives expected pairs from `matrix-cases.json`, not a duplicated scenario list.
- The matrix coverage check uses exact title suffix matching, not substring matching.
- The full verification wrapper clears stale matrix result files before Playwright e2e.
- The matrix report includes success, ok, ready, clean typed failure, unexpected failure, timeout/hang, artifact completeness, false completion, average elapsed, and worst elapsed metrics.
- Matrix artifact requirements are granular: telemetry, HTML/reduced DOM, and screenshot are checked independently.
- Closed-page/browser-failure scenarios require telemetry and placeholder HTML artifacts but not screenshot artifacts.
- Readiness probe failures route through `finalizeResult` and produce artifacts when `artifactDir` is set.
- The plan covers success, manual-action, rate-limit, timeout, artifact, and telemetry paths.
- The plan routes non-success adapter returns through `finalizeResult` before returning to the test.
- The artifact writer catches closed-page failures while reading page content and URL.
- The plan treats missing assistant answer containers as `response_parse_failed`, not `ok`.
- The plan redacts artifact URLs and requires reduced, text-free DOM artifacts for live Gemini runs.
- The plan tests that reduced artifacts skip screenshots and strip visible text/form values.
- The matrix artifact directory policy is synchronized with `RESILIENCE_TEST_MATRIX.md`.
- The plan passes telemetry `networkSummary` through shared adapter options before artifact capture.
- The plan writes sanitized artifacts only under ignored `research/gemini_browser_adapter/artifacts`.
- The plan includes exact commands and expected outcomes for every task.
