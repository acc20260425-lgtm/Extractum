# Gemini Browser CDP Attach Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an operator/debug mode where the Gemini Browser Provider attaches to a user-controlled local Chrome instance through CDP instead of launching Playwright-owned Chromium.

**Architecture:** Keep managed Playwright browser mode as the default. Add a sidecar-only CDP attach mode selected by `EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT`, with strict loopback endpoint validation, deterministic Gemini page selection, CDP-specific lifecycle ownership, and typed operator setup statuses. Rust/Tauri keeps the same command boundary but extends `resume` with `browser_profile_dir` and returns provider status instead of an ack.

**Tech Stack:** Tauri 2, Rust/serde, Node/TypeScript sidecar, Playwright `chromium.connectOverCDP`, Vitest, Cargo tests, Svelte type checking.

---

## File Structure

- Create `sidecars/gemini-browser/src/cdp-endpoint.ts`
  - Validates loopback CDP endpoint URLs.
  - Resolves browser mode from `EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT`.
  - Performs a light `/json/version` reachability/protocol check without attaching or opening pages.
- Create `sidecars/gemini-browser/src/cdp-endpoint.test.ts`
  - Covers accepted and rejected endpoint URLs.
  - Covers reachable, unreachable, and non-Chrome probe results with fake `fetch`.
- Create `sidecars/gemini-browser/src/cdp-pages.ts`
  - Selects a Gemini page deterministically from CDP contexts.
  - Classifies Playwright closed-target errors for `browser_crashed`.
- Create `sidecars/gemini-browser/src/cdp-pages.test.ts`
  - Covers page selection and error classification without Google login state.
- Modify `sidecars/gemini-browser/src/protocol.ts`
  - Add `browser_profile_dir` to the `resume` command shape.
- Modify `sidecars/gemini-browser/src/protocol.test.ts`
  - Add parsing coverage for `resume` with `browser_profile_dir`.
- Modify `sidecars/gemini-browser/src/adapter.ts`
  - Add mode-aware session ownership.
  - Add CDP attach/open/resume/status behavior.
  - Map CDP closed-target errors to `browser_crashed`.
- Modify `sidecars/gemini-browser/src/index.ts`
  - Route `resume` to `adapter.resumeBrowser(browser_profile_dir)`.
- Modify `src-tauri/src/gemini_browser/types.rs`
  - Add `StartChromeCdp` manual action.
  - Add `browser_profile_dir` to `GeminiBrowserSidecarCommand::Resume`.
  - Add serde tests for both.
- Modify `src-tauri/src/gemini_browser/sidecar.rs`
  - Add a `resume` request helper that expects a `Status` response.
- Modify `src-tauri/src/gemini_browser/commands.rs`
  - Make `gemini_bridge_resume` call `sidecar::resume(...)`.
- Modify `src/lib/types/gemini-browser.ts`
  - Add `"start_chrome_cdp"` to `GeminiBrowserManualAction`.
- Modify `src/lib/gemini-browser-provider-panel-contract.ts`
  - Map `start_chrome_cdp` to a compact operator label.
- Modify `src/lib/gemini-browser-provider-panel.test.ts`
  - Cover the new label.
- Modify `scripts/gemini-browser-sidecar-smoke.mjs`
  - Add a `--resume` smoke path that sends the new resume command.
- Modify `package.json`
  - Add optional resume smoke scripts.
- Modify `README.md`
  - Document CDP operator mode and the dedicated Chrome profile command.

---

## Task 1: Shared Protocol And Manual Action Types

**Files:**
- Modify: `sidecars/gemini-browser/src/protocol.ts`
- Modify: `sidecars/gemini-browser/src/protocol.test.ts`
- Modify: `src-tauri/src/gemini_browser/types.rs`
- Modify: `src/lib/types/gemini-browser.ts`
- Modify: `src/lib/gemini-browser-provider-panel-contract.ts`
- Modify: `src/lib/gemini-browser-provider-panel.test.ts`

- [ ] **Step 1: Add the failing TypeScript protocol test**

Append this test to `sidecars/gemini-browser/src/protocol.test.ts`:

```ts
it("parses resume command with browser profile dir", () => {
  const envelope = parseEnvelope(
    JSON.stringify({
      id: "cmd-resume",
      command: {
        type: "resume",
        run_id: null,
        browser_profile_dir: "C:/Extractum/gemini-browser/profile",
      },
    }),
  );

  expect(envelope.command).toEqual({
    type: "resume",
    run_id: null,
    browser_profile_dir: "C:/Extractum/gemini-browser/profile",
  });
});
```

- [ ] **Step 2: Add the failing UI label expectation**

In `src/lib/gemini-browser-provider-panel.test.ts`, inside the `maps provider statuses to compact operator labels` test, add:

```ts
expect(statusLabel("needs_manual_action", "start_chrome_cdp")).toBe("Start Chrome");
```

- [ ] **Step 3: Add the failing Rust serde expectations**

In `src-tauri/src/gemini_browser/types.rs`, inside `mod tests`, add:

```rust
#[test]
fn manual_action_serializes_start_chrome_cdp() {
    let value = serde_json::to_value(GeminiBrowserManualAction::StartChromeCdp)
        .expect("serialize manual action");

    assert_eq!(value, "start_chrome_cdp");
}

#[test]
fn resume_command_serializes_browser_profile_dir() {
    let command = GeminiBrowserSidecarEnvelope {
        id: "cmd-resume".to_string(),
        command: GeminiBrowserSidecarCommand::Resume {
            run_id: None,
            browser_profile_dir: "C:/Extractum/gemini-browser/profile".to_string(),
        },
    };

    let json = serde_json::to_value(command).expect("serialize command");
    assert_eq!(json["command"]["type"], "resume");
    assert_eq!(json["command"]["run_id"], serde_json::Value::Null);
    assert_eq!(
        json["command"]["browser_profile_dir"],
        "C:/Extractum/gemini-browser/profile"
    );
}
```

- [ ] **Step 4: Run the targeted tests and verify they fail for the expected reasons**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar:typecheck
npm.cmd run test:gemini-browser-sidecar:unit
npm.cmd run test -- src/lib/gemini-browser-provider-panel.test.ts
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-cdp --lib gemini_browser::types
```

Expected:
- TypeScript typecheck fails because `browser_profile_dir` is not in the `resume` command union. The unit test alone is not sufficient because `parseEnvelope()` casts parsed JSON to `SidecarEnvelope`.
- UI test fails because `start_chrome_cdp` is not a known manual action label.
- Rust test fails because `StartChromeCdp` and `Resume.browser_profile_dir` are not implemented.

- [ ] **Step 5: Update the TypeScript sidecar protocol**

Change the `resume` command member in `sidecars/gemini-browser/src/protocol.ts` to:

```ts
  | { type: "resume"; run_id: string | null; browser_profile_dir: string }
```

- [ ] **Step 6: Update the frontend manual action union**

Change `GeminiBrowserManualAction` in `src/lib/types/gemini-browser.ts` to:

```ts
export type GeminiBrowserManualAction =
  | "login"
  | "account_picker"
  | "consent"
  | "captcha"
  | "unknown_modal"
  | "start_chrome_cdp";
```

- [ ] **Step 7: Update the provider label mapping**

In `src/lib/gemini-browser-provider-panel-contract.ts`, add the `start_chrome_cdp` branch before the generic `needs_manual_action` branch:

```ts
  if (status === "needs_manual_action" && manualAction === "start_chrome_cdp") {
    return "Start Chrome";
  }
```

- [ ] **Step 8: Update Rust shared types**

In `src-tauri/src/gemini_browser/types.rs`, add the enum variant:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeminiBrowserManualAction {
    Login,
    AccountPicker,
    Consent,
    Captcha,
    UnknownModal,
    StartChromeCdp,
}
```

Change the `Resume` command variant to:

```rust
    Resume {
        run_id: Option<String>,
        browser_profile_dir: String,
    },
```

- [ ] **Step 9: Run targeted tests and verify they pass**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar:unit
npm.cmd run test -- src/lib/gemini-browser-provider-panel.test.ts
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-cdp --lib gemini_browser::types
```

Expected:
- Sidecar protocol tests pass.
- Provider panel label test passes.
- Rust Gemini browser type tests pass.

- [ ] **Step 10: Commit shared type changes**

Run:

```powershell
git add sidecars/gemini-browser/src/protocol.ts sidecars/gemini-browser/src/protocol.test.ts src-tauri/src/gemini_browser/types.rs src/lib/types/gemini-browser.ts src/lib/gemini-browser-provider-panel-contract.ts src/lib/gemini-browser-provider-panel.test.ts
git commit -m "feat: add Gemini CDP manual action protocol"
```

Expected: commit contains only shared protocol/type/label updates.

---

## Task 2: CDP Endpoint Validation And Light Status Probe

**Files:**
- Create: `sidecars/gemini-browser/src/cdp-endpoint.ts`
- Create: `sidecars/gemini-browser/src/cdp-endpoint.test.ts`

- [ ] **Step 1: Write endpoint validation and probe tests**

Create `sidecars/gemini-browser/src/cdp-endpoint.test.ts`:

```ts
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

  it("reports unreachable endpoint as operator setup action", async () => {
    const fetchMock = vi.fn<FetchLike>(async () => {
      throw new Error("ECONNREFUSED");
    });

    await expect(cdpSetupStatus("http://127.0.0.1:9222", fetchMock)).resolves.toEqual({
      ok: false,
      message: "Chrome CDP endpoint is unavailable. Start Chrome with remote debugging enabled.",
    });
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar:unit
```

Expected: FAIL because `sidecars/gemini-browser/src/cdp-endpoint.ts` does not exist.

- [ ] **Step 3: Implement CDP endpoint helpers**

Create `sidecars/gemini-browser/src/cdp-endpoint.ts`:

```ts
export type BrowserMode =
  | { type: "managed" }
  | { type: "cdp_attach"; endpoint: string };

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

  const validation = validateCdpEndpoint(raw);
  if (!validation.ok) {
    return { type: "cdp_attach", endpoint: raw };
  }
  return { type: "cdp_attach", endpoint: validation.endpoint };
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
    return { ok: false, message: "Chrome CDP endpoint must be a base URL without path, query, or hash." };
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
    const timeout = setTimeout(() => controller.abort(), 1500);
    try {
      const versionUrl = new URL("/json/version", validation.endpoint);
      const response = await fetchLike(versionUrl, { signal: controller.signal });
      if (!response.ok) {
        return {
          ok: false,
          message: "Chrome CDP endpoint did not expose a Chrome debugging protocol.",
        };
      }
      const payload = await response.json();
      if (isChromeVersionPayload(payload)) {
        return { ok: true, message: "Chrome CDP endpoint is reachable." };
      }
      return {
        ok: false,
        message: "Chrome CDP endpoint did not expose a Chrome debugging protocol.",
      };
    } finally {
      clearTimeout(timeout);
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
```

- [ ] **Step 4: Run endpoint tests**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar:unit
```

Expected: PASS for `cdp-endpoint.test.ts` and existing sidecar tests.

- [ ] **Step 5: Commit endpoint helpers**

Run:

```powershell
git add sidecars/gemini-browser/src/cdp-endpoint.ts sidecars/gemini-browser/src/cdp-endpoint.test.ts
git commit -m "feat: validate Gemini Chrome CDP endpoint"
```

Expected: commit contains only CDP endpoint helper and tests.

---

## Task 3: CDP Page Selection And Closed-Target Classification

**Files:**
- Create: `sidecars/gemini-browser/src/cdp-pages.ts`
- Create: `sidecars/gemini-browser/src/cdp-pages.test.ts`

- [ ] **Step 1: Write page selection and closed-target tests**

Create `sidecars/gemini-browser/src/cdp-pages.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { isClosedTargetError, selectGeminiPage, type CdpPageLike } from "./cdp-pages.js";

function page(url: string, closed = false): CdpPageLike {
  return {
    isClosed: () => closed,
    url: () => url,
  };
}

describe("CDP Gemini page selection", () => {
  it("selects only gemini.google.com pages", () => {
    const gemini = page("https://gemini.google.com/app");
    const selected = selectGeminiPage([
      page("https://accounts.google.com/signin"),
      page("https://google.com/search?q=gemini"),
      gemini,
    ]);

    expect(selected).toBe(gemini);
  });

  it("ignores closed and unreadable pages", () => {
    const gemini = page("https://gemini.google.com/app");
    const unreadable: CdpPageLike = {
      isClosed: () => false,
      url: () => {
        throw new Error("closed");
      },
    };

    expect(selectGeminiPage([page("https://gemini.google.com/app", true), unreadable, gemini])).toBe(
      gemini,
    );
  });

  it("prefers active Gemini page when provided", () => {
    const first = page("https://gemini.google.com/app");
    const active = page("https://gemini.google.com/app/active");

    expect(selectGeminiPage([first, active], active)).toBe(active);
  });
});

describe("closed target error classification", () => {
  it("matches Playwright closed-target and disconnect phrases", () => {
    for (const message of [
      "Target closed",
      "Page closed",
      "Browser has been closed",
      "Context closed",
      "Protocol error: Connection closed",
      "browserContext.close: Target page, context or browser has been closed",
    ]) {
      expect(isClosedTargetError(new Error(message)), message).toBe(true);
    }
  });

  it("does not match ordinary DOM failures", () => {
    expect(isClosedTargetError(new Error("Composer was not found."))).toBe(false);
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar:unit
```

Expected: FAIL because `sidecars/gemini-browser/src/cdp-pages.ts` does not exist.

- [ ] **Step 3: Implement page helpers**

Create `sidecars/gemini-browser/src/cdp-pages.ts`:

```ts
export interface CdpPageLike {
  isClosed: () => boolean;
  url: () => string;
}

export function selectGeminiPage<T extends CdpPageLike>(
  pages: T[],
  activePage: T | null = null,
): T | null {
  const candidates = pages.filter(isUsableGeminiPage);
  if (activePage && candidates.includes(activePage)) {
    return activePage;
  }
  return candidates[0] ?? null;
}

export function isClosedTargetError(error: unknown): boolean {
  const message = error instanceof Error ? error.message : String(error);
  return /target closed|page closed|browser has been closed|context closed|connection closed|target page, context or browser has been closed/i.test(
    message,
  );
}

function isUsableGeminiPage(page: CdpPageLike): boolean {
  if (page.isClosed()) return false;
  let rawUrl: string;
  try {
    rawUrl = page.url();
  } catch {
    return false;
  }
  try {
    const parsed = new URL(rawUrl);
    return parsed.protocol === "https:" && parsed.hostname === "gemini.google.com";
  } catch {
    return false;
  }
}
```

- [ ] **Step 4: Run page helper tests**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar:unit
```

Expected: PASS for endpoint, page helper, protocol, and adapter tests.

- [ ] **Step 5: Commit page helper changes**

Run:

```powershell
git add sidecars/gemini-browser/src/cdp-pages.ts sidecars/gemini-browser/src/cdp-pages.test.ts
git commit -m "feat: add Gemini CDP page selection helpers"
```

Expected: commit contains only CDP page helper and tests.

---

## Task 4: Mode-Aware Sidecar Adapter

**Files:**
- Modify: `sidecars/gemini-browser/src/adapter.ts`
- Modify: `sidecars/gemini-browser/src/adapter.test.ts`

- [ ] **Step 1: Add adapter tests for CDP setup status and closed-target mapping**

Append these tests to `sidecars/gemini-browser/src/adapter.test.ts`:

```ts
import { GeminiBrowserAdapter } from "./adapter.js";

it("reports CDP endpoint setup action before long-lived attach", async () => {
  const adapter = new GeminiBrowserAdapter({
    env: { EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT: "http://127.0.0.1:9222" },
    fetchLike: async () => {
      throw new Error("ECONNREFUSED");
    },
  });

  await expect(adapter.status("C:/Extractum/gemini-browser/profile")).resolves.toMatchObject({
    status: "needs_manual_action",
    manual_action: "start_chrome_cdp",
    browser_profile_dir: "C:/Extractum/gemini-browser/profile",
    latest_message: "Chrome CDP endpoint is unavailable. Start Chrome with remote debugging enabled.",
  });
});

it("maps CDP closed-target send failures to browser_crashed", async () => {
  const adapter = new GeminiBrowserAdapter({
    env: {},
  });
  const page = {
    isClosed: () => false,
    locator: () => {
      throw new Error("Target closed");
    },
    waitForTimeout: async () => undefined,
  };
  adapter.__setTestPage(page as never, "cdp_attach");

  await expect(
    adapter.sendSingle({
      browserProfileDir: "C:/Extractum/gemini-browser/profile",
      artifactDir: "C:/Extractum/gemini-browser/runs/run-1",
      request: {
        run_id: "run-1",
        prompt: "hello",
        source: "settings_test",
        artifact_mode: "reduced",
      },
    }),
  ).resolves.toMatchObject({
    status: "browser_crashed",
    manual_action: null,
    message: "Chrome CDP connection closed during the run.",
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar:unit
```

Expected: FAIL because `GeminiBrowserAdapter` does not accept injected dependencies and does not expose `__setTestPage`.

- [ ] **Step 3: Update adapter imports and session types**

In `sidecars/gemini-browser/src/adapter.ts`, replace the Playwright import with:

```ts
import {
  chromium,
  type Browser,
  type BrowserContext,
  type Locator,
  type Page,
} from "@playwright/test";
```

Add these imports:

```ts
import {
  cdpSetupStatus,
  resolveBrowserMode,
  validateCdpEndpoint,
  type FetchLike,
} from "./cdp-endpoint.js";
import { isClosedTargetError, selectGeminiPage } from "./cdp-pages.js";
```

Add these types above the class:

```ts
type BrowserSession =
  | { type: "managed"; context: BrowserContext; page: Page | null }
  | { type: "cdp_attach"; browser: Browser | null; context: BrowserContext | null; page: Page | null };

interface GeminiBrowserAdapterOptions {
  env?: Record<string, string | undefined>;
  fetchLike?: FetchLike;
}
```

- [ ] **Step 4: Replace adapter fields and constructor**

Replace the current fields:

```ts
  private context: BrowserContext | null = null;
  private page: Page | null = null;
```

with:

```ts
  private session: BrowserSession | null = null;
  private readonly env: Record<string, string | undefined>;
  private readonly fetchLike: FetchLike;

  constructor(options: GeminiBrowserAdapterOptions = {}) {
    this.env = options.env ?? process.env;
    this.fetchLike = options.fetchLike ?? fetch;
  }
```

Add this test-only helper inside the class:

```ts
  __setTestPage(page: Page, mode: BrowserSession["type"] = "managed") {
    if (mode === "managed") {
      this.session = { type: "managed", context: null as unknown as BrowserContext, page };
      return;
    }
    this.session = { type: "cdp_attach", browser: null, context: null, page };
  }
```

- [ ] **Step 5: Replace status implementation**

Replace `status(browserProfileDir: string)` with:

```ts
  async status(browserProfileDir: string): Promise<GeminiBrowserProviderStatus> {
    const page = this.session?.page ?? null;
    if (page && !page.isClosed()) {
      return providerStatus({
        status: "ready",
        browserProfileDir,
        message:
          this.session?.type === "cdp_attach"
            ? "Chrome CDP attached."
            : "Browser page is available.",
      });
    }

    const mode = resolveBrowserMode(this.env);
    if (mode.type === "cdp_attach") {
      const probe = await cdpSetupStatus(mode.endpoint, this.fetchLike);
      if (!probe.ok) {
        return providerStatus({
          status: "needs_manual_action",
          manualAction: "start_chrome_cdp",
          browserProfileDir,
          message: probe.message,
        });
      }
      return providerStatus({
        status: "not_started",
        browserProfileDir,
        message: "Chrome CDP endpoint is configured but not attached.",
      });
    }

    return providerStatus({
      status: "not_started",
      browserProfileDir,
      message: "Browser has not been opened.",
    });
  }
```

Add this helper below the class:

```ts
function providerStatus(input: {
  status: GeminiBrowserProviderStatus["status"];
  browserProfileDir: string;
  message: string;
  manualAction?: string | null;
}): GeminiBrowserProviderStatus {
  return {
    status: input.status,
    manual_action: input.manualAction ?? null,
    active_run_id: null,
    queue_depth: 0,
    browser_profile_dir: input.browserProfileDir,
    latest_message: input.message,
  };
}
```

- [ ] **Step 6: Add managed and CDP open/resume helpers**

Replace `openBrowser(browserProfileDir: string)` with:

```ts
  async openBrowser(browserProfileDir: string): Promise<GeminiBrowserProviderStatus> {
    const mode = resolveBrowserMode(this.env);
    if (mode.type === "cdp_attach") {
      return this.attachCdpBrowser(browserProfileDir, { createGeminiPage: true });
    }
    return this.openManagedBrowser(browserProfileDir);
  }

  async resumeBrowser(browserProfileDir: string): Promise<GeminiBrowserProviderStatus> {
    const mode = resolveBrowserMode(this.env);
    if (mode.type === "cdp_attach") {
      return this.attachCdpBrowser(browserProfileDir, { createGeminiPage: false });
    }
    return this.openManagedBrowser(browserProfileDir);
  }

  private async openManagedBrowser(browserProfileDir: string): Promise<GeminiBrowserProviderStatus> {
    await mkdir(browserProfileDir, { recursive: true });
    const context = await chromium.launchPersistentContext(browserProfileDir, {
      headless: false,
      viewport: { width: 1280, height: 900 },
    });
    const page = context.pages()[0] ?? (await context.newPage());
    this.session = { type: "managed", context, page };
    await page.goto("https://gemini.google.com/app", { waitUntil: "domcontentloaded" });
    return this.status(browserProfileDir);
  }

  private async attachCdpBrowser(
    browserProfileDir: string,
    options: { createGeminiPage: boolean },
  ): Promise<GeminiBrowserProviderStatus> {
    const mode = resolveBrowserMode(this.env);
    if (mode.type !== "cdp_attach") {
      return this.openManagedBrowser(browserProfileDir);
    }

    const validation = validateCdpEndpoint(mode.endpoint);
    if (!validation.ok) {
      return providerStatus({
        status: "needs_manual_action",
        manualAction: "start_chrome_cdp",
        browserProfileDir,
        message: validation.message,
      });
    }

    let browser: Browser;
    try {
      browser = await chromium.connectOverCDP(validation.endpoint);
    } catch {
      return providerStatus({
        status: "needs_manual_action",
        manualAction: "start_chrome_cdp",
        browserProfileDir,
        message: "Chrome CDP endpoint is unavailable. Start Chrome with remote debugging enabled.",
      });
    }

    const context = browser.contexts()[0] ?? null;
    if (!context) {
      this.session = { type: "cdp_attach", browser, context: null, page: null };
      return providerStatus({
        status: "needs_manual_action",
        manualAction: "start_chrome_cdp",
        browserProfileDir,
        message:
          "Chrome CDP connected but no user profile context was available. Restart Chrome with a dedicated user data directory.",
      });
    }

    let page = selectGeminiPage(context.pages());
    if (!page && options.createGeminiPage) {
      page = await context.newPage();
      await page.goto("https://gemini.google.com/app", { waitUntil: "domcontentloaded" });
    }

    this.session = { type: "cdp_attach", browser, context, page };
    if (!page) {
      return providerStatus({
        status: "needs_manual_action",
        manualAction: "start_chrome_cdp",
        browserProfileDir,
        message: "Open Gemini in the attached Chrome profile or use Open to create a Gemini tab.",
      });
    }

    return providerStatus({
      status: "ready",
      browserProfileDir,
      message: "Chrome CDP attached.",
    });
  }
```

- [ ] **Step 7: Update sendSingle page access and closed-target mapping**

In `sendSingle`, replace:

```ts
    if (!this.page) {
      await this.openBrowser(input.browserProfileDir);
    }
    const page = this.page;
```

with:

```ts
    if (!this.session?.page || this.session.page.isClosed()) {
      const mode = resolveBrowserMode(this.env);
      if (mode.type === "cdp_attach") {
        await this.attachCdpBrowser(input.browserProfileDir, { createGeminiPage: false });
      } else {
        await this.openManagedBrowser(input.browserProfileDir);
      }
    }
    const page = this.session?.page ?? null;
    if (!page || page.isClosed()) {
      return {
        run_id: input.request.run_id,
        status: "needs_manual_action",
        text: null,
        message: "Open Gemini in the attached Chrome profile or use Open to create a Gemini tab.",
        manual_action: "start_chrome_cdp",
        artifacts: {
          run_dir: input.artifactDir,
          html: null,
          screenshot: null,
          telemetry: null,
          artifact_write_error: null,
        },
        elapsed_ms: Date.now() - start,
      };
    }
```

Replace the catch block:

```ts
    } catch (error) {
      return this.failure(page, input.request, input.artifactDir, "failed", String(error), start);
    }
```

with:

```ts
    } catch (error) {
      if (this.session?.type === "cdp_attach" && isClosedTargetError(error)) {
        return this.failure(
          page,
          input.request,
          input.artifactDir,
          "browser_crashed",
          "Chrome CDP connection closed during the run.",
          start,
        );
      }
      return this.failure(page, input.request, input.artifactDir, "failed", String(error), start);
    }
```

- [ ] **Step 8: Update stop lifecycle**

Replace `stop()` with:

```ts
  async stop(): Promise<void> {
    if (this.session?.type === "managed") {
      await this.session.context?.close().catch(() => undefined);
    }
    this.session = null;
  }
```

This intentionally drops CDP references only. Do not call `context.close()` or `browser.close()` in CDP mode; Playwright does not provide a safer v1 detach operation here that is worth risking user tabs. Managed mode still closes the context it owns.

- [ ] **Step 9: Run sidecar tests**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar
```

Expected: typecheck, unit tests, and sidecar build pass.

- [ ] **Step 10: Commit adapter changes**

Run:

```powershell
git add sidecars/gemini-browser/src/adapter.ts sidecars/gemini-browser/src/adapter.test.ts
git commit -m "feat: add Gemini browser CDP adapter mode"
```

Expected: commit contains only adapter CDP behavior and tests.

---

## Task 5: Wire Resume Through Sidecar And Rust

**Files:**
- Modify: `sidecars/gemini-browser/src/index.ts`
- Modify: `src-tauri/src/gemini_browser/sidecar.rs`
- Modify: `src-tauri/src/gemini_browser/commands.rs`
- Modify: `src-tauri/src/gemini_browser/types.rs`

- [ ] **Step 1: Add the Rust sidecar resume test**

In `src-tauri/src/gemini_browser/types.rs`, the Task 1 `resume_command_serializes_browser_profile_dir` test already covers serialization. Add one more assertion to that same test:

```rust
assert!(json["command"].get("browser_profile_dir").is_some());
```

- [ ] **Step 2: Run Rust test and verify current call sites fail to compile**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-cdp --lib gemini_browser
```

Expected: FAIL until Rust command construction is updated to provide `browser_profile_dir` for `Resume`.

- [ ] **Step 3: Wire Node sidecar resume to adapter**

In `sidecars/gemini-browser/src/index.ts`, replace:

```ts
      if (command.type === "resume") {
        writeResponse(id, { type: "ack" });
        return;
      }
```

with:

```ts
      if (command.type === "resume") {
        writeResponse(id, {
          type: "status",
          status: await adapter.resumeBrowser(command.browser_profile_dir),
        });
        return;
      }
```

- [ ] **Step 4: Add Rust sidecar resume helper**

In `src-tauri/src/gemini_browser/sidecar.rs`, add this function after `open_browser`:

```rust
pub(crate) async fn resume(
    handle: &AppHandle,
    state: &GeminiBrowserState,
    browser_profile_dir: String,
) -> AppResult<GeminiBrowserProviderStatus> {
    match request_sidecar(
        handle,
        state,
        GeminiBrowserSidecarCommand::Resume {
            run_id: None,
            browser_profile_dir,
        },
    )
    .await?
    {
        GeminiBrowserSidecarResponse::Status { status } => Ok(status),
        _ => Err(AppError::internal(
            "Unexpected Gemini sidecar resume response",
        )),
    }
}
```

- [ ] **Step 5: Update Tauri resume command**

In `src-tauri/src/gemini_browser/commands.rs`, replace the current `gemini_bridge_resume` body:

```rust
pub async fn gemini_bridge_resume(
    handle: AppHandle,
    state: State<'_, GeminiBrowserState>,
) -> AppResult<GeminiBrowserProviderStatus> {
    sidecar::open_browser(&handle, &state, path_string(&profile_dir(&handle)?)).await
}
```

with:

```rust
pub async fn gemini_bridge_resume(
    handle: AppHandle,
    state: State<'_, GeminiBrowserState>,
) -> AppResult<GeminiBrowserProviderStatus> {
    sidecar::resume(&handle, &state, path_string(&profile_dir(&handle)?)).await
}
```

- [ ] **Step 6: Run TypeScript and Rust checks**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-cdp --lib gemini_browser
```

Expected:
- Sidecar typecheck/build passes with `resumeBrowser`.
- Rust Gemini browser tests pass.

- [ ] **Step 7: Commit resume wiring**

Run:

```powershell
git add sidecars/gemini-browser/src/index.ts src-tauri/src/gemini_browser/sidecar.rs src-tauri/src/gemini_browser/commands.rs src-tauri/src/gemini_browser/types.rs
git commit -m "feat: route Gemini browser resume through sidecar"
```

Expected: commit contains Node/Rust resume wiring and serialization test update.

---

## Task 6: Resume Smoke And Operator Negative Check

**Files:**
- Modify: `scripts/gemini-browser-sidecar-smoke.mjs`
- Modify: `package.json`

- [ ] **Step 1: Add sidecar smoke support for resume**

In `scripts/gemini-browser-sidecar-smoke.mjs`, after the existing mode parsing, add:

```js
const resumeSmoke = process.argv.includes("--resume");
```

In the non-Playwright branch, replace the `request` constant with:

```js
  const profileDir = path.join(repoRoot, "artifacts", "gemini-browser-smoke-profile");
  const request = {
    id: "smoke-1",
    command: resumeSmoke
      ? {
          type: "resume",
          run_id: null,
          browser_profile_dir: profileDir,
        }
      : {
          type: "status",
          browser_profile_dir: profileDir,
        },
  };
```

Replace the response type assertion:

```js
    if (parsed.response?.type !== "status") {
```

with the same line. Both status and resume smokes should receive `response.type === "status"`.

- [ ] **Step 2: Add package scripts**

In `package.json`, add scripts next to the existing sidecar smoke scripts:

```json
"smoke:gemini-browser-sidecar:resume:node": "node scripts/gemini-browser-sidecar-smoke.mjs --resume",
"smoke:gemini-browser-sidecar:resume:binary": "node scripts/gemini-browser-sidecar-smoke.mjs --binary --resume"
```

- [ ] **Step 3: Run normal resume smoke in managed mode**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar
npm.cmd run smoke:gemini-browser-sidecar:resume:node
```

Expected: smoke returns a JSON response with `response.type: "status"`.

- [ ] **Step 4: Run CDP negative smoke without Chrome**

Run in PowerShell:

```powershell
$env:EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT = "http://127.0.0.1:9222"
npm.cmd run smoke:gemini-browser-sidecar:resume:node
Remove-Item Env:\EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT
```

Expected: smoke returns `response.type: "status"`, `response.status.status: "needs_manual_action"`, and `response.status.manual_action: "start_chrome_cdp"`. If port `9222` is already running Chrome on the machine, use a closed loopback port such as `127.0.0.1:65530` for this negative smoke.

- [ ] **Step 5: Commit smoke changes**

Run:

```powershell
git add scripts/gemini-browser-sidecar-smoke.mjs package.json
git commit -m "test: add Gemini CDP resume smoke"
```

Expected: commit contains only smoke script and package script updates.

---

## Task 7: Documentation And Operator Instructions

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/specs/2026-06-20-gemini-browser-cdp-attach-design.md`
- Modify: `docs/superpowers/plans/2026-06-20-gemini-browser-cdp-attach-plan.md`

- [ ] **Step 1: Document CDP operator mode in README**

Append this section under the existing Gemini Browser Sidecar Packaging section in `README.md`:

````markdown
## Gemini Browser User-Controlled Chrome CDP Mode

For Google accounts that reject Playwright-owned browser login, operators can run
Gemini Browser Provider in user-controlled Chrome CDP mode.

Start ordinary Chrome with a dedicated Extractum profile:

```powershell
$profile = "$env:APPDATA\org.ai.extractum\gemini-browser\chrome-cdp-profile"
Start-Process chrome.exe -ArgumentList @(
  "--remote-debugging-port=9222",
  "--user-data-dir=$profile",
  "https://gemini.google.com/app"
)
```

Do not use the normal Chrome `Default` profile. Complete Google/Gemini login
manually in this Chrome window, then start Extractum with:

```powershell
$env:EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT = "http://127.0.0.1:9222"
npm.cmd run tauri dev
```

In CDP mode, `Resume` attaches only to an existing Gemini tab. `Open` may create
a Gemini tab but never performs Google account, phone, CAPTCHA, consent, or
other security actions. `Stop` detaches Extractum and does not close Chrome.
Only loopback base HTTP endpoints are accepted.
````

- [ ] **Step 2: Add implementation note to the CDP spec**

Append this section to `docs/superpowers/specs/2026-06-20-gemini-browser-cdp-attach-design.md`:

```markdown
## Implementation Plan

Implementation plan:
`docs/superpowers/plans/2026-06-20-gemini-browser-cdp-attach-plan.md`.
```

- [ ] **Step 3: Mark docs task complete in this plan**

Update Task 7 checkboxes to `[x]` after README and spec are updated.

- [ ] **Step 4: Commit docs**

Run:

```powershell
git add README.md docs/superpowers/specs/2026-06-20-gemini-browser-cdp-attach-design.md docs/superpowers/plans/2026-06-20-gemini-browser-cdp-attach-plan.md
git commit -m "docs: document Gemini CDP attach operator mode"
```

Expected: commit contains operator docs and plan checkbox updates.

---

## Task 8: Final Verification

**Files:**
- Modify: `docs/superpowers/plans/2026-06-20-gemini-browser-cdp-attach-plan.md`

- [ ] **Step 1: Run sidecar verification**

Run:

```powershell
npm.cmd run test:gemini-browser-sidecar
```

Expected: sidecar typecheck, unit tests, and build pass.

- [ ] **Step 2: Run frontend verification**

Run:

```powershell
npm.cmd run test -- src/lib/gemini-browser-provider-panel.test.ts src/lib/api/gemini-browser.test.ts
npm.cmd run check
```

Expected: targeted Vitest tests pass and Svelte check reports `0 errors and 0 warnings`.

- [ ] **Step 3: Run Rust verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-cdp --lib gemini_browser
```

Expected: all Gemini browser Rust tests pass.

- [ ] **Step 4: Run managed sidecar smokes**

Run:

```powershell
npm.cmd run smoke:gemini-browser-sidecar:node
npm.cmd run smoke:gemini-browser-sidecar:resume:node
npm.cmd run smoke:gemini-browser-sidecar:playwright:node
```

Expected: status and resume smokes return `response.type: "status"`; Playwright smoke returns `ok: true`.

- [ ] **Step 5: Run CDP negative smoke**

Run:

```powershell
$env:EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT = "http://127.0.0.1:65530"
npm.cmd run smoke:gemini-browser-sidecar:resume:node
Remove-Item Env:\EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT
```

Expected: response status is `needs_manual_action`, manual action is `start_chrome_cdp`, and latest message tells the operator to start Chrome with remote debugging enabled.

- [ ] **Step 6: Run optional manual CDP happy path**

Run Chrome manually:

```powershell
$profile = "$env:APPDATA\org.ai.extractum\gemini-browser\chrome-cdp-profile"
Start-Process chrome.exe -ArgumentList @(
  "--remote-debugging-port=9222",
  "--user-data-dir=$profile",
  "https://gemini.google.com/app"
)
```

Then log in manually and run Extractum with:

```powershell
$env:EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT = "http://127.0.0.1:9222"
npm.cmd run tauri dev
```

Expected: `Settings -> Browser Providers -> Resume` attaches to the existing Gemini tab; sending the one-sentence test prompt creates a new run with status `ok`. If Google blocks the account, record the observed manual action in the verification notes instead of changing code.

- [ ] **Step 7: Append verification notes**

Append this section to the end of this plan:

```markdown
## Verification Notes

- Sidecar verification: pending until Task 8 Step 1 executes.
- Frontend verification: pending until Task 8 Step 2 executes.
- Rust verification: pending until Task 8 Step 3 executes.
- Managed sidecar smokes: pending until Task 8 Step 4 executes.
- CDP negative smoke: pending until Task 8 Step 5 executes.
- Manual CDP happy path: pending until Task 8 Step 6 executes.
```

During Task 8 execution, replace each pending line with the observed result and concrete command evidence.

- [ ] **Step 8: Commit final verification note**

Run:

```powershell
git add docs/superpowers/plans/2026-06-20-gemini-browser-cdp-attach-plan.md
git commit -m "docs: record Gemini CDP attach verification"
```

Expected: final commit contains only plan checkbox and verification-note updates.

---

## Self-Review

**Spec coverage:** This plan covers user-controlled Chrome CDP attach, endpoint allowlist and normalization, dedicated profile guidance, mode ownership, `resume` protocol shape, page selection, CDP status probing, closed-target mapping, manual action type updates, smoke tests, docs, and manual validation.

**Security boundary:** CDP attach remains loopback-only, rejects non-base URLs and credentials, does not automate Google login/security flows, and does not close the user's Chrome process.

**Known limitation:** The existing DOM adapter still treats no-composer states as `needs_login`. The CDP spec records this as a known limitation and follow-up; this plan does not build a full Gemini manual-action classifier.

**Out of scope:** This plan does not add packaged Settings UI for CDP endpoint configuration, automatic Chrome launching, remote CDP support, Chrome installation discovery, or bundled Chromium delivery.
