import { mkdir } from "node:fs/promises";
import {
  chromium,
  type Browser,
  type BrowserContext,
  type Locator,
  type Page,
} from "@playwright/test";
import type {
  GeminiBrowserDebugErrorStage,
  GeminiBrowserProviderConfig,
  GeminiBrowserProviderStatus,
  GeminiBrowserRunDebugSummary,
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

  async status(
    browserProfileDir: string,
    browserConfig?: GeminiBrowserProviderConfig | null,
  ): Promise<GeminiBrowserProviderStatus> {
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

    const mode = resolveBrowserMode(this.env, browserConfig);
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

  async openBrowser(
    browserProfileDir: string,
    browserConfig?: GeminiBrowserProviderConfig | null,
  ): Promise<GeminiBrowserProviderStatus> {
    const mode = resolveBrowserMode(this.env, browserConfig);
    if (mode.type === "cdp_attach") {
      return this.attachCdpBrowser(browserProfileDir, { createGeminiPage: true, browserConfig });
    }
    return this.openManagedBrowser(browserProfileDir);
  }

  async resumeBrowser(
    browserProfileDir: string,
    browserConfig?: GeminiBrowserProviderConfig | null,
  ): Promise<GeminiBrowserProviderStatus> {
    const mode = resolveBrowserMode(this.env, browserConfig);
    if (mode.type === "cdp_attach") {
      return this.attachCdpBrowser(browserProfileDir, { createGeminiPage: false, browserConfig });
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
    options: { createGeminiPage: boolean; browserConfig?: GeminiBrowserProviderConfig | null },
  ): Promise<GeminiBrowserProviderStatus> {
    const mode = resolveBrowserMode(this.env, options.browserConfig);
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
    browserConfig?: GeminiBrowserProviderConfig | null;
  }): Promise<GeminiBrowserRunResult> {
    const start = Date.now();
    const mode = resolveBrowserMode(this.env, input.browserConfig);
    let debugSummary = emptyDebugSummary(this.session?.type ?? mode.type);
    const hadClosedCdpPage =
      this.session?.type === "cdp_attach" && Boolean(this.session.page?.isClosed());
    if (hadClosedCdpPage) {
      return finalizeRunResult(
        {
          run_id: input.request.run_id,
          status: "browser_crashed",
          text: null,
          message: "Chrome CDP page closed before the run could send.",
          manual_action: null,
          artifacts: emptyArtifacts(input.artifactDir),
          elapsed_ms: Date.now() - start,
        },
        markErrorStage(debugSummary, "transport"),
      );
    }

    let attachStatus: GeminiBrowserProviderStatus | null = null;
    if (!this.session?.page || this.session.page.isClosed()) {
      if (mode.type === "cdp_attach") {
        attachStatus = await this.attachCdpBrowser(input.browserProfileDir, {
          createGeminiPage: false,
          browserConfig: input.browserConfig,
        });
      } else {
        await this.openManagedBrowser(input.browserProfileDir);
      }
    }
    const page = this.session?.page ?? null;
    if (!page || page.isClosed()) {
      const activeType = this.session?.type ?? mode.type;
      if (activeType !== "cdp_attach") {
        debugSummary = emptyDebugSummary(activeType);
        return finalizeRunResult(
          {
            run_id: input.request.run_id,
            status: "failed",
            text: null,
            message: "Gemini browser page was not created.",
            manual_action: null,
            artifacts: emptyArtifacts(input.artifactDir),
            elapsed_ms: Date.now() - start,
          },
          markErrorStage(debugSummary, "setup"),
        );
      }

      if (!this.session?.context && attachStatus) {
        return finalizeRunResult(
          {
            run_id: input.request.run_id,
            status: "needs_manual_action",
            text: null,
            message: attachStatus.latest_message,
            manual_action: attachStatus.manual_action,
            artifacts: emptyArtifacts(input.artifactDir),
            elapsed_ms: Date.now() - start,
          },
          markErrorStage(debugSummary, "setup"),
        );
      }

      return finalizeRunResult(
        {
          run_id: input.request.run_id,
          status: "needs_manual_action",
          text: null,
          message: "Open Gemini in the attached Chrome profile or use Open to create a Gemini tab.",
          manual_action: "start_chrome_cdp",
          artifacts: emptyArtifacts(input.artifactDir),
          elapsed_ms: Date.now() - start,
        },
        markErrorStage(debugSummary, "setup"),
      );
    }

    try {
      const composerResult = await waitForFirstVisibleWithDiagnostics(
        page,
        composerCandidates.map((candidate) => candidate.selector),
        { timeoutMs: 30_000, intervalMs: 500 },
      );
      const composer = composerResult.locator;
      debugSummary = {
        ...debugSummary,
        composer_found: Boolean(composer),
      };
      if (!composer) {
        return this.failure(
          page,
          input.request,
          input.artifactDir,
          "needs_login",
          "Composer was not found.",
          start,
          markErrorStage(debugSummary, "composer"),
        );
      }
      await composer.fill(input.request.prompt).catch(async () => {
        await composer.click();
        await page.keyboard.insertText(input.request.prompt);
      });

      const sendResult = await waitForFirstVisibleWithDiagnostics(
        page,
        sendCandidates.map((candidate) => candidate.selector),
        {
          timeoutMs: 75_000,
          intervalMs: 250,
          keepWaitingWhileVisible: generationBusySelectors,
          idleGraceMs: 10_000,
        },
      );
      const send = sendResult.locator;
      debugSummary = {
        ...debugSummary,
        send_button_found: Boolean(send),
        generation_busy_observed: sendResult.keptWaitingObserved,
        waited_for_send_ms: sendResult.waitedMs,
      };
      if (!send) {
        return this.failure(
          page,
          input.request,
          input.artifactDir,
          "needs_manual_action",
          "Send button was not found.",
          start,
          markErrorStage(debugSummary, "send"),
        );
      }
      const answerBaseline = await captureAnswerState(page, input.request.prompt);
      await send.click();

      const answer = await waitForAnswerText(page, input.request.prompt, answerBaseline);
      debugSummary = {
        ...debugSummary,
        answer_found: Boolean(answer),
        answer_selector: answer?.selector ?? null,
        waited_for_answer_ms: answer?.waitedMs ?? ANSWER_TIMEOUT_MS,
        answer_completion_reason: answer?.completionReason ?? "missing",
        final_text_length: answer?.text.length ?? 0,
      };
      if (!answer) {
        return this.failure(
          page,
          input.request,
          input.artifactDir,
          "timeout",
          "Answer did not appear before timeout.",
          start,
          markErrorStage(debugSummary, "answer"),
        );
      }

      return finalizeRunResult(
        {
          run_id: input.request.run_id,
          status: "ok",
          text: answer.text,
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
        },
        debugSummary,
      );
    } catch (error) {
      if (this.session?.type === "cdp_attach" && isClosedTargetError(error)) {
        return this.failure(
          page,
          input.request,
          input.artifactDir,
          "browser_crashed",
          "Chrome CDP connection closed during the run.",
          start,
          markErrorStage(debugSummary, "transport"),
        );
      }
      return this.failure(
        page,
        input.request,
        input.artifactDir,
        "failed",
        String(error),
        start,
        markErrorStage(debugSummary, "transport"),
      );
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
    debugSummary: GeminiBrowserRunDebugSummary,
  ): Promise<GeminiBrowserRunResult> {
    return finalizeRunResult(
      {
        run_id: request.run_id,
        status,
        text: null,
        message,
        manual_action: status === "needs_login" ? "login" : null,
        artifacts: await captureFailureArtifacts({ page, artifactDir, request, status, message }),
        elapsed_ms: Date.now() - start,
      },
      debugSummary,
    );
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

interface WaitForFirstVisibleResult {
  locator: Locator | null;
  selector: string | null;
  waitedMs: number;
  keptWaitingObserved: boolean;
}

function emptyDebugSummary(mode: GeminiBrowserProviderConfig["mode"]): GeminiBrowserRunDebugSummary {
  return {
    mode,
    composer_found: false,
    send_button_found: false,
    generation_busy_observed: false,
    answer_found: false,
    answer_selector: null,
    waited_for_send_ms: 0,
    waited_for_answer_ms: 0,
    answer_stable_ms: ANSWER_STABLE_MS,
    answer_completion_reason: "missing",
    final_text_length: 0,
    error_stage: null,
  };
}

type RunResultWithoutDebug = Omit<GeminiBrowserRunResult, "debug_summary">;

function finalizeRunResult(
  result: RunResultWithoutDebug,
  debugSummary: GeminiBrowserRunDebugSummary,
): GeminiBrowserRunResult {
  return {
    ...result,
    debug_summary: debugSummary,
  };
}

function markErrorStage(
  debugSummary: GeminiBrowserRunDebugSummary,
  errorStage: GeminiBrowserDebugErrorStage,
): GeminiBrowserRunDebugSummary {
  return {
    ...debugSummary,
    error_stage: errorStage,
  };
}

export async function waitForFirstVisible(
  page: Pick<Page, "locator" | "waitForTimeout">,
  selectors: string[],
  options: {
    timeoutMs?: number;
    intervalMs?: number;
    keepWaitingWhileVisible?: string[];
    idleGraceMs?: number;
  } = {},
): Promise<Locator | null> {
  return (await waitForFirstVisibleWithDiagnostics(page, selectors, options)).locator;
}

async function waitForFirstVisibleWithDiagnostics(
  page: Pick<Page, "locator" | "waitForTimeout">,
  selectors: string[],
  options: {
    timeoutMs?: number;
    intervalMs?: number;
    keepWaitingWhileVisible?: string[];
    idleGraceMs?: number;
  } = {},
): Promise<WaitForFirstVisibleResult> {
  const timeoutMs = options.timeoutMs ?? 20_000;
  const intervalMs = options.intervalMs ?? 250;
  const idleGraceMs = options.idleGraceMs ?? timeoutMs;
  const maxAttempts = Math.max(1, Math.ceil(timeoutMs / Math.max(intervalMs, 1)) + 1);
  let idleElapsedMs = 0;
  let waitedMs = 0;
  let keptWaitingObserved = false;

  for (let attempt = 0; attempt < maxAttempts; attempt += 1) {
    for (const selector of selectors) {
      const locator = page.locator(selector);
      const count = await locator.count();
      for (let index = count - 1; index >= 0; index -= 1) {
        const candidate = locator.nth(index);
        if (await candidate.isVisible().catch(() => false)) {
          return { locator: candidate, selector, waitedMs, keptWaitingObserved };
        }
      }
    }
    const shouldKeepWaiting =
      options.keepWaitingWhileVisible &&
      (await hasVisibleLocator(page, options.keepWaitingWhileVisible));
    if (shouldKeepWaiting) {
      keptWaitingObserved = true;
    } else {
      idleElapsedMs += intervalMs;
      if (idleElapsedMs >= idleGraceMs) {
        return { locator: null, selector: null, waitedMs, keptWaitingObserved };
      }
    }
    if (attempt < maxAttempts - 1) {
      await page.waitForTimeout(intervalMs);
      waitedMs += intervalMs;
    }
  }
  return { locator: null, selector: null, waitedMs, keptWaitingObserved };
}

const ANSWER_TIMEOUT_MS = 60_000;
const ANSWER_POLL_INTERVAL_MS = 500;
const ANSWER_STABLE_MS = 8_000;
const generationBusySelectors = [
  "button[aria-label*='Stop generating' i]",
  "button[aria-label*='Останов' i]",
];

async function hasVisibleLocator(
  page: Pick<Page, "locator">,
  selectors: string[],
): Promise<boolean> {
  for (const selector of selectors) {
    const locator = page.locator(selector);
    const count = await locator.count().catch(() => 0);
    for (let index = count - 1; index >= 0; index -= 1) {
      if (await locator.nth(index).isVisible().catch(() => false)) {
        return true;
      }
    }
  }
  return false;
}

interface AnswerEntry {
  selector: string;
  text: string;
}

interface AnswerState {
  entries: AnswerEntry[];
}

interface AnswerResult {
  text: string;
  selector: string;
  waitedMs: number;
  completionReason: "stable" | "timeout_latest";
}

async function waitForAnswerText(
  page: Page,
  prompt: string,
  baseline: AnswerState,
): Promise<AnswerResult | null> {
  const deadline = Date.now() + ANSWER_TIMEOUT_MS;
  let latestAnswer: AnswerEntry | null = null;
  let lastChangedAt = Date.now();
  let firstSeenAt: number | null = null;
  let waitedMs = 0;

  while (Date.now() < deadline) {
    const state = await captureAnswerState(page, prompt);
    const answer = bestNewAnswerText(state, baseline);
    const now = Date.now();
    if (answer && answer.text !== latestAnswer?.text) {
      latestAnswer = answer;
      lastChangedAt = now;
      firstSeenAt ??= now;
    }
    if (latestAnswer) {
      const stableForMs = now - lastChangedAt;
      if (
        firstSeenAt !== null &&
        stableForMs >= ANSWER_STABLE_MS &&
        now - firstSeenAt >= ANSWER_STABLE_MS
      ) {
        return { ...latestAnswer, waitedMs, completionReason: "stable" };
      }
    }
    await page.waitForTimeout(ANSWER_POLL_INTERVAL_MS);
    waitedMs += ANSWER_POLL_INTERVAL_MS;
  }

  return latestAnswer ? { ...latestAnswer, waitedMs, completionReason: "timeout_latest" } : null;
}

async function captureAnswerState(page: Page, prompt: string): Promise<AnswerState> {
  const entries: AnswerEntry[] = [];
  const seen = new Set<string>();
  for (const selector of answerCandidates.map((candidate) => candidate.selector)) {
    const rawTexts = await page.locator(selector).allTextContents().catch(() => []);
    for (const rawText of rawTexts) {
      const text = rawText.trim();
      if (text.length === 0 || text === prompt || seen.has(text)) continue;
      seen.add(text);
      entries.push({ selector, text });
    }
  }

  return {
    entries,
  };
}

function bestNewAnswerText(current: AnswerState, baseline: AnswerState): AnswerEntry | null {
  const baselineTexts = new Set(baseline.entries.map((entry) => entry.text));
  const newEntries = current.entries.filter((entry) => !baselineTexts.has(entry.text));
  if (newEntries.length === 0) return null;

  return newEntries.reduce((best, entry) => (entry.text.length >= best.text.length ? entry : best));
}
