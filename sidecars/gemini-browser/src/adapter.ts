import { mkdir } from "node:fs/promises";
import {
  chromium,
  type Browser,
  type BrowserContext,
  type Locator,
  type Page,
} from "@playwright/test";
import type {
  GeminiBrowserProviderStatus,
  GeminiBrowserRunRequest,
  GeminiBrowserRunResult,
} from "./protocol.js";
import { answerCandidates, composerCandidates, sendCandidates } from "./dom-contract.js";
import { captureFailureArtifacts } from "./artifacts.js";
import {
  cdpSetupStatus,
  resolveBrowserMode,
  validateCdpEndpoint,
  type FetchLike,
} from "./cdp-endpoint.js";
import { isClosedTargetError, selectGeminiPage } from "./cdp-pages.js";

type BrowserSession =
  | { type: "managed"; context: BrowserContext; page: Page | null }
  | {
      type: "cdp_attach";
      browser: Browser | null;
      context: BrowserContext | null;
      page: Page | null;
    };

type ConnectOverCdp = (endpoint: string) => Promise<Browser>;

interface GeminiBrowserAdapterOptions {
  env?: Record<string, string | undefined>;
  fetchLike?: FetchLike;
  connectOverCdp?: ConnectOverCdp;
}

export class GeminiBrowserAdapter {
  private session: BrowserSession | null = null;
  private readonly env: Record<string, string | undefined>;
  private readonly fetchLike: FetchLike;
  private readonly connectOverCdp: ConnectOverCdp;

  constructor(options: GeminiBrowserAdapterOptions = {}) {
    this.env = options.env ?? process.env;
    this.fetchLike = options.fetchLike ?? fetch;
    this.connectOverCdp = options.connectOverCdp ?? ((endpoint) => chromium.connectOverCDP(endpoint));
  }

  __setTestPage(page: Page, mode: BrowserSession["type"] = "managed") {
    if (mode === "managed") {
      this.session = { type: "managed", context: null as unknown as BrowserContext, page };
      return;
    }
    this.session = { type: "cdp_attach", browser: null, context: null, page };
  }

  __setTestSession(session: BrowserSession) {
    this.session = session;
  }

  async status(browserProfileDir: string): Promise<GeminiBrowserProviderStatus> {
    if (this.session?.type === "cdp_attach") {
      const page = this.session.page;
      if (page && !page.isClosed()) {
        return providerStatus({
          status: "ready",
          browserProfileDir,
          message: "Chrome CDP attached.",
        });
      }
      if (this.session.context) {
        return providerStatus({
          status: "needs_manual_action",
          manualAction: "start_chrome_cdp",
          browserProfileDir,
          message: "Chrome CDP attached, but no Gemini tab is available.",
        });
      }
    }

    const page = this.session?.page ?? null;
    if (page && !page.isClosed()) {
      return providerStatus({
        status: "ready",
        browserProfileDir,
        message: "Browser page is available.",
      });
    }

    const mode = resolveBrowserMode(this.env);
    if (mode.type === "cdp_attach") {
      const probe = await cdpSetupStatus(mode.rawEndpoint, this.fetchLike);
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

    const validation = validateCdpEndpoint(mode.rawEndpoint);
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
      browser = await this.connectOverCdp(validation.endpoint);
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
      this.session = null;
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

  async sendSingle(input: {
    request: GeminiBrowserRunRequest;
    browserProfileDir: string;
    artifactDir: string;
  }): Promise<GeminiBrowserRunResult> {
    const start = Date.now();
    const mode = resolveBrowserMode(this.env);
    const hadClosedCdpPage =
      this.session?.type === "cdp_attach" && Boolean(this.session.page?.isClosed());
    if (hadClosedCdpPage) {
      return {
        run_id: input.request.run_id,
        status: "browser_crashed",
        text: null,
        message: "Chrome CDP page closed before the run could send.",
        manual_action: null,
        artifacts: emptyArtifacts(input.artifactDir),
        elapsed_ms: Date.now() - start,
      };
    }

    let attachStatus: GeminiBrowserProviderStatus | null = null;
    if (!this.session?.page || this.session.page.isClosed()) {
      if (mode.type === "cdp_attach") {
        attachStatus = await this.attachCdpBrowser(input.browserProfileDir, {
          createGeminiPage: false,
        });
      } else {
        await this.openManagedBrowser(input.browserProfileDir);
      }
    }
    const page = this.session?.page ?? null;
    if (!page || page.isClosed()) {
      const activeType = this.session?.type ?? mode.type;
      if (activeType !== "cdp_attach") {
        return {
          run_id: input.request.run_id,
          status: "failed",
          text: null,
          message: "Gemini browser page was not created.",
          manual_action: null,
          artifacts: emptyArtifacts(input.artifactDir),
          elapsed_ms: Date.now() - start,
        };
      }

      if (!this.session?.context && attachStatus) {
        return {
          run_id: input.request.run_id,
          status: "needs_manual_action",
          text: null,
          message: attachStatus.latest_message,
          manual_action: attachStatus.manual_action,
          artifacts: emptyArtifacts(input.artifactDir),
          elapsed_ms: Date.now() - start,
        };
      }

      return {
        run_id: input.request.run_id,
        status: "needs_manual_action",
        text: null,
        message: "Open Gemini in the attached Chrome profile or use Open to create a Gemini tab.",
        manual_action: "start_chrome_cdp",
        artifacts: emptyArtifacts(input.artifactDir),
        elapsed_ms: Date.now() - start,
      };
    }

    try {
      const composer = await waitForFirstVisible(
        page,
        composerCandidates.map((candidate) => candidate.selector),
        { timeoutMs: 30_000, intervalMs: 500 },
      );
      if (!composer) {
        return this.failure(
          page,
          input.request,
          input.artifactDir,
          "needs_login",
          "Composer was not found.",
          start,
        );
      }
      await composer.fill(input.request.prompt).catch(async () => {
        await composer.click();
        await page.keyboard.insertText(input.request.prompt);
      });

      const send = await waitForFirstVisible(
        page,
        sendCandidates.map((candidate) => candidate.selector),
        { timeoutMs: 10_000, intervalMs: 250 },
      );
      if (!send) {
        return this.failure(
          page,
          input.request,
          input.artifactDir,
          "needs_manual_action",
          "Send button was not found.",
          start,
        );
      }
      await send.click();

      const answer = await waitForAnswerText(page, input.request.prompt);
      if (!answer) {
        return this.failure(
          page,
          input.request,
          input.artifactDir,
          "timeout",
          "Answer did not appear before timeout.",
          start,
        );
      }

      return {
        run_id: input.request.run_id,
        status: "ok",
        text: answer,
        message: null,
        manual_action: null,
        artifacts: {
          run_dir: input.artifactDir,
          html: null,
          screenshot: null,
          telemetry: null,
          artifact_write_error: null,
        },
        elapsed_ms: Date.now() - start,
      };
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
  }

  async stop(): Promise<void> {
    if (this.session?.type === "managed") {
      await this.session.context?.close().catch(() => undefined);
    }
    this.session = null;
  }

  private async failure(
    page: Page,
    request: GeminiBrowserRunRequest,
    artifactDir: string,
    status: GeminiBrowserRunResult["status"],
    message: string,
    start: number,
  ): Promise<GeminiBrowserRunResult> {
    return {
      run_id: request.run_id,
      status,
      text: null,
      message,
      manual_action: status === "needs_login" ? "login" : null,
      artifacts: await captureFailureArtifacts({ page, artifactDir, request, status, message }),
      elapsed_ms: Date.now() - start,
    };
  }
}

function providerStatus(input: {
  status: GeminiBrowserProviderStatus["status"];
  browserProfileDir: string;
  message: string;
  manualAction?: GeminiBrowserProviderStatus["manual_action"];
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

function emptyArtifacts(artifactDir: string): GeminiBrowserRunResult["artifacts"] {
  return {
    run_dir: artifactDir,
    html: null,
    screenshot: null,
    telemetry: null,
    artifact_write_error: null,
  };
}

export async function waitForFirstVisible(
  page: Pick<Page, "locator" | "waitForTimeout">,
  selectors: string[],
  options: { timeoutMs?: number; intervalMs?: number } = {},
): Promise<Locator | null> {
  const timeoutMs = options.timeoutMs ?? 20_000;
  const intervalMs = options.intervalMs ?? 250;
  const maxAttempts = Math.max(1, Math.ceil(timeoutMs / Math.max(intervalMs, 1)) + 1);

  for (let attempt = 0; attempt < maxAttempts; attempt += 1) {
    for (const selector of selectors) {
      const locator = page.locator(selector);
      const count = await locator.count();
      for (let index = count - 1; index >= 0; index -= 1) {
        const candidate = locator.nth(index);
        if (await candidate.isVisible().catch(() => false)) {
          return candidate;
        }
      }
    }
    if (attempt < maxAttempts - 1) {
      await page.waitForTimeout(intervalMs);
    }
  }
  return null;
}

async function waitForAnswerText(page: Page, prompt: string): Promise<string | null> {
  const deadline = Date.now() + 60_000;
  while (Date.now() < deadline) {
    for (const selector of answerCandidates.map((candidate) => candidate.selector)) {
      const texts = await page.locator(selector).allTextContents().catch(() => []);
      const answer = texts
        .map((text) => text.trim())
        .filter((text) => text.length > 0 && text !== prompt)
        .at(-1);
      if (answer) return answer;
    }
    await page.waitForTimeout(500);
  }
  return null;
}
