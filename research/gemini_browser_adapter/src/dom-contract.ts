import type { Locator, Page } from "@playwright/test";
import { captureFailureArtifacts } from "./artifacts";
import type { DomContractConfig } from "./config";
import { loadDomContractConfig } from "./config";
import { scoreButtonCandidate, scoreEditableCandidate } from "./scoring";
import { isSuccessStatus } from "./types";
import type { GeminiAdapterResult, GeminiAdapterStatus, LocatorAttempt, NetworkEventSummary } from "./types";

export type SendSingleOptions = {
  timeoutMs: number;
  quietMs: number;
  configPath?: string;
  contractConfig?: DomContractConfig;
  artifactDir?: string;
  artifactMode?: "full" | "reduced";
  networkSummary?: NetworkEventSummary[];
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
  const tagName = await promptBox.evaluate((element) => element.tagName.toLowerCase());
  if (tagName === "textarea" || tagName === "input") {
    await promptBox.click();
    await promptBox.fill(prompt);
    return;
  }
  if (await promptBox.isVisible().catch(() => false)) {
    await promptBox.click().catch(() => undefined);
  }
  await promptBox.evaluate((element, value) => {
    element.textContent = value;
    element.dispatchEvent(new InputEvent("input", { bubbles: true, inputType: "insertText", data: value }));
  }, prompt);
}

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
    const fallback = count > 0 ? locator.first() : null;
    attempts.push({ name, strategy: "css", matched: Boolean(visible ?? fallback), count });
    if (visible ?? fallback) return visible ?? fallback;
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
  const complete = async (base: GeminiAdapterResult) => await finalizeResult(page, base, options);

  if (page.isClosed()) return await complete(result("browser_crashed", startedAt, null, attempts, "browser_crashed"));

  const criticalBefore = await scanCriticalState(page);
  if (criticalBefore) return await complete(result(criticalBefore, startedAt, null, attempts, criticalBefore));

  const promptBox = await findPromptBox(page, attempts);
  if (!promptBox) return await complete(result("failed", startedAt, null, attempts, "prompt_input_not_found"));

  await typePrompt(promptBox, prompt);
  const sendButton = await findSendButton(page, attempts);
  if (!sendButton) return await complete(result("failed", startedAt, null, attempts, "send_button_not_found"));
  await sendButton.click();

  return await complete(await waitForFinalAnswer(page, startedAt, options, attempts, config));
}

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
    (await findPromptBoxByScoring(page, attempts, config.minPromptScore)) ??
    (await findPromptBox(page, attempts));
  if (!promptBox) return await complete(result("failed", startedAt, null, attempts, "prompt_input_not_found"));

  await typePrompt(promptBox, prompt);
  const sendButton =
    (await findByConfiguredSelector(page, config.sendSelectors, attempts, "config:send")) ??
    (await findSendButtonByScoring(page, attempts, config.minSendScore)) ??
    (await findSendButton(page, attempts));
  if (!sendButton) return await complete(result("failed", startedAt, null, attempts, "send_button_not_found"));
  await sendButton.click();

  return await complete(await waitForFinalAnswer(page, startedAt, options, attempts, config));
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
      (await findPromptBoxByScoring(page, attempts, config.minPromptScore)) ??
      (await findPromptBox(page, attempts));
    const sendButton =
      (await findByConfiguredSelector(page, config.sendSelectors, attempts, "config:send")) ??
      (await findSendButtonByScoring(page, attempts, config.minSendScore)) ??
      (await findSendButton(page, attempts));
    if (promptBox && sendButton) return await complete(result("ready", startedAt, null, attempts, null));

    return await complete(result("failed", startedAt, null, attempts, "ready_contract_not_satisfied"));
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    const status = /closed|crash|target page/i.test(message) ? "browser_crashed" : "failed";
    return await complete(result(status, startedAt, null, attempts, message));
  }
}
