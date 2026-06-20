import type { Locator, Page } from "@playwright/test";
import type { DomContractConfig } from "./config";
import { loadDomContractConfig } from "./config";
import type { GeminiAdapterResult, GeminiAdapterStatus, LocatorAttempt } from "./types";

export type SendSingleOptions = {
  timeoutMs: number;
  quietMs: number;
  configPath?: string;
  contractConfig?: DomContractConfig;
};

async function resolveContractConfig(options: SendSingleOptions): Promise<DomContractConfig> {
  return options.contractConfig ?? (await loadDomContractConfig(options.configPath));
}

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
  config: DomContractConfig,
): Promise<GeminiAdapterResult> {
  let lastText = "";
  let lastChangedAt = Date.now();
  let sawAnswer = false;

  while (Date.now() - startedAt < options.timeoutMs) {
    const critical = await scanCriticalState(page);
    if (critical) return result(critical, startedAt, null, attempts, critical);

    const text = await latestAnswerText(page, config);
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
  const config = await resolveContractConfig(options);

  if (page.isClosed()) return result("browser_crashed", startedAt, null, attempts, "browser_crashed");

  const criticalBefore = await scanCriticalState(page);
  if (criticalBefore) return result(criticalBefore, startedAt, null, attempts, criticalBefore);

  const promptBox = await findPromptBox(page, attempts);
  if (!promptBox) return result("failed", startedAt, null, attempts, "prompt_input_not_found");

  await typePrompt(promptBox, prompt);
  const sendButton = await findSendButton(page, attempts);
  if (!sendButton) return result("failed", startedAt, null, attempts, "send_button_not_found");
  await sendButton.click();

  return await waitForFinalAnswer(page, startedAt, options, attempts, config);
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
