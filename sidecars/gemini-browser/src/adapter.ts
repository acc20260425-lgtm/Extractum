import { mkdir } from "node:fs/promises";
import { chromium, type BrowserContext, type Locator, type Page } from "@playwright/test";
import type {
  GeminiBrowserProviderStatus,
  GeminiBrowserRunRequest,
  GeminiBrowserRunResult,
} from "./protocol.js";
import { answerCandidates, composerCandidates, sendCandidates } from "./dom-contract.js";
import { captureFailureArtifacts } from "./artifacts.js";

export class GeminiBrowserAdapter {
  private context: BrowserContext | null = null;
  private page: Page | null = null;

  async status(browserProfileDir: string): Promise<GeminiBrowserProviderStatus> {
    return {
      status: this.page ? "ready" : "not_started",
      manual_action: null,
      active_run_id: null,
      queue_depth: 0,
      browser_profile_dir: browserProfileDir,
      latest_message: this.page ? "Browser page is available." : "Browser has not been opened.",
    };
  }

  async openBrowser(browserProfileDir: string): Promise<GeminiBrowserProviderStatus> {
    await mkdir(browserProfileDir, { recursive: true });
    this.context = await chromium.launchPersistentContext(browserProfileDir, {
      headless: false,
      viewport: { width: 1280, height: 900 },
    });
    this.page = this.context.pages()[0] ?? (await this.context.newPage());
    await this.page.goto("https://gemini.google.com/app", { waitUntil: "domcontentloaded" });
    return this.status(browserProfileDir);
  }

  async sendSingle(input: {
    request: GeminiBrowserRunRequest;
    browserProfileDir: string;
    artifactDir: string;
  }): Promise<GeminiBrowserRunResult> {
    const start = Date.now();
    if (!this.page) {
      await this.openBrowser(input.browserProfileDir);
    }
    const page = this.page;
    if (!page) {
      throw new Error("Gemini browser page was not created");
    }

    try {
      const composer = await firstVisible(
        page,
        composerCandidates.map((candidate) => candidate.selector),
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

      const send = await firstVisible(
        page,
        sendCandidates.map((candidate) => candidate.selector),
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
      return this.failure(page, input.request, input.artifactDir, "failed", String(error), start);
    }
  }

  async stop(): Promise<void> {
    await this.context?.close().catch(() => undefined);
    this.context = null;
    this.page = null;
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

async function firstVisible(page: Page, selectors: string[]): Promise<Locator | null> {
  for (const selector of selectors) {
    const locator = page.locator(selector).last();
    if ((await locator.count()) > 0 && (await locator.isVisible().catch(() => false))) {
      return locator;
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
