import { describe, expect, it, vi } from "vitest";
import type { Page } from "@playwright/test";
import { GeminiBrowserAdapter, waitForFirstVisible } from "./adapter.js";
import {
  GEMINI_DOM_CONTRACT_VERSION,
  answerCandidates,
  composerCandidates,
  sendCandidates,
} from "./dom-contract.js";

describe("production Gemini DOM contract", () => {
  it("keeps the selected resilient-scoring contract version explicit", () => {
    expect(GEMINI_DOM_CONTRACT_VERSION).toBe("2026-06-20-resilient-scoring");
  });

  it("has candidates for composer, send, and answer extraction", () => {
    expect(composerCandidates.length).toBeGreaterThan(0);
    expect(sendCandidates.length).toBeGreaterThan(0);
    expect(answerCandidates.length).toBeGreaterThan(0);
    expect(answerCandidates.some((candidate) => candidate.selector === "main section")).toBe(false);
  });

  it("covers the localized Russian Gemini send button label", () => {
    expect(sendCandidates.map((candidate) => candidate.selector)).toContain(
      "button[aria-label*='Отправ' i]",
    );
  });

  it("waits for a delayed composer candidate before reporting it missing", async () => {
    let attempts = 0;
    const locator = {
      count: async () => {
        attempts += 1;
        return attempts >= 3 ? 1 : 0;
      },
      isVisible: async () => true,
    };
    const locatorList = {
      count: locator.count,
      nth: () => locator,
      last: () => locator,
    };
    const page = {
      locator: (selector: string) => {
        expect(selector).toBe("[contenteditable='true']");
        return locatorList;
      },
      waitForTimeout: async () => undefined,
    } as unknown as Pick<Page, "locator" | "waitForTimeout">;

    await expect(
      waitForFirstVisible(page, ["[contenteditable='true']"], { timeoutMs: 1_000, intervalMs: 0 }),
    ).resolves.toBe(locator);
    expect(attempts).toBe(3);
  });

  it("returns an earlier visible match when the last matching element is hidden", async () => {
    const visible = {
      count: async () => 1,
      isVisible: async () => true,
    };
    const hidden = {
      count: async () => 1,
      isVisible: async () => false,
    };
    const locatorList = {
      count: async () => 2,
      nth: (index: number) => (index === 0 ? visible : hidden),
      last: () => hidden,
    };
    const page = {
      locator: (selector: string) => {
        expect(selector).toBe("[contenteditable='true']");
        return locatorList;
      },
      waitForTimeout: async () => undefined,
    } as unknown as Pick<Page, "locator" | "waitForTimeout">;

    await expect(
      waitForFirstVisible(page, ["[contenteditable='true']"], { timeoutMs: 1_000, intervalMs: 0 }),
    ).resolves.toBe(visible);
  });

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

  it("reports attached CDP session without a Gemini page distinctly", async () => {
    const adapter = new GeminiBrowserAdapter({
      env: { EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT: "http://127.0.0.1:9222" },
    });

    adapter.__setTestSession({
      type: "cdp_attach",
      browser: null,
      context: { pages: () => [] } as never,
      page: null,
    });

    await expect(adapter.status("C:/Extractum/gemini-browser/profile")).resolves.toMatchObject({
      status: "needs_manual_action",
      manual_action: "start_chrome_cdp",
      latest_message: "Chrome CDP attached, but no Gemini tab is available.",
    });
  });

  it("opens Gemini in an existing CDP context without creating a browser context", async () => {
    const page = {
      goto: vi.fn(async () => undefined),
      isClosed: () => false,
    };
    const context = {
      pages: () => [],
      newPage: vi.fn(async () => page),
    };
    const browser = {
      contexts: () => [context],
      newContext: vi.fn(),
    };
    const adapter = new GeminiBrowserAdapter({
      env: { EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT: "http://127.0.0.1:9222" },
      connectOverCdp: async () => browser as never,
    });

    await expect(adapter.openBrowser("C:/Extractum/gemini-browser/profile")).resolves.toMatchObject({
      status: "ready",
      latest_message: "Chrome CDP attached.",
    });
    expect(context.newPage).toHaveBeenCalledOnce();
    expect(browser.newContext).not.toHaveBeenCalled();
    expect(page.goto).toHaveBeenCalledWith("https://gemini.google.com/app", {
      waitUntil: "domcontentloaded",
    });
  });

  it("does not create a Gemini page from sendSingle in CDP attach-only mode", async () => {
    const context = {
      pages: () => [],
      newPage: vi.fn(),
    };
    const browser = {
      contexts: () => [context],
    };
    const adapter = new GeminiBrowserAdapter({
      env: { EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT: "http://127.0.0.1:9222" },
      connectOverCdp: async () => browser as never,
    });

    await expect(
      adapter.sendSingle({
        browserProfileDir: "C:/Extractum/gemini-browser/profile",
        artifactDir: "artifacts/gemini-browser-adapter-test/run-1",
        request: {
          run_id: "run-1",
          prompt: "hello",
          source: "settings_test",
          artifact_mode: "reduced",
        },
      }),
    ).resolves.toMatchObject({
      status: "needs_manual_action",
      manual_action: "start_chrome_cdp",
      message: "Open Gemini in the attached Chrome profile or use Open to create a Gemini tab.",
      debug_summary: {
        mode: "cdp_attach",
        composer_found: false,
        send_button_found: false,
        answer_found: false,
        answer_completion_reason: "missing",
        final_text_length: 0,
        error_stage: "setup",
      },
    });
    expect(context.newPage).not.toHaveBeenCalled();
  });

  it("preserves CDP attach setup errors from sendSingle", async () => {
    const adapter = new GeminiBrowserAdapter({
      env: { EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT: "http://127.0.0.1:9222" },
      connectOverCdp: async () => {
        throw new Error("ECONNREFUSED");
      },
    });

    await expect(
      adapter.sendSingle({
        browserProfileDir: "C:/Extractum/gemini-browser/profile",
        artifactDir: "artifacts/gemini-browser-adapter-test/run-1",
        request: {
          run_id: "run-1",
          prompt: "hello",
          source: "settings_test",
          artifact_mode: "reduced",
        },
      }),
    ).resolves.toMatchObject({
      status: "needs_manual_action",
      manual_action: "start_chrome_cdp",
      message: "Chrome CDP endpoint is unavailable. Start Chrome with remote debugging enabled.",
      debug_summary: {
        mode: "cdp_attach",
        composer_found: false,
        send_button_found: false,
        answer_found: false,
        answer_completion_reason: "missing",
        final_text_length: 0,
        error_stage: "setup",
      },
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
        artifactDir: "artifacts/gemini-browser-adapter-test/run-1",
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
      debug_summary: {
        mode: "cdp_attach",
        answer_completion_reason: "missing",
        final_text_length: 0,
        error_stage: "transport",
      },
    });
  });

  it("maps an already closed attached CDP page before send to browser_crashed", async () => {
    const adapter = new GeminiBrowserAdapter({
      env: { EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT: "http://127.0.0.1:9222" },
    });
    const page = {
      isClosed: () => true,
    };
    adapter.__setTestPage(page as never, "cdp_attach");

    await expect(
      adapter.sendSingle({
        browserProfileDir: "C:/Extractum/gemini-browser/profile",
        artifactDir: "artifacts/gemini-browser-adapter-test/run-1",
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
      message: "Chrome CDP page closed before the run could send.",
      debug_summary: {
        mode: "cdp_attach",
        composer_found: false,
        send_button_found: false,
        answer_found: false,
        answer_completion_reason: "missing",
        final_text_length: 0,
        error_stage: "transport",
      },
    });
  });

  it("adds sanitized debug summary to send-button failures", async () => {
    const prompt = "private prompt must not appear in debug summary";
    const composer = {
      count: async () => 1,
      nth: () => composer,
      isVisible: async () => true,
      fill: vi.fn(async () => undefined),
    };
    const empty = {
      count: async () => 0,
      nth: () => empty,
      isVisible: async () => false,
      allTextContents: async () => [],
    };
    const page = {
      isClosed: () => false,
      locator: (selector: string) => {
        if (selector === "rich-textarea textarea") return composer;
        return empty;
      },
      waitForTimeout: async () => undefined,
    };
    const adapter = new GeminiBrowserAdapter({ env: {} });
    adapter.__setTestPage(page as never);

    const result = await adapter.sendSingle({
      browserProfileDir: "C:/Extractum/gemini-browser/profile",
      artifactDir: "artifacts/gemini-browser-adapter-test/run-send-fail",
      request: {
        run_id: "run-send-fail",
        prompt,
        source: "settings_test",
        artifact_mode: "reduced",
      },
    });

    expect(result).toMatchObject({
      status: "needs_manual_action",
      message: "Send button was not found.",
      debug_summary: {
        mode: "managed",
        composer_found: true,
        send_button_found: false,
        answer_found: false,
        answer_selector: null,
        answer_completion_reason: "missing",
        final_text_length: 0,
        error_stage: "send",
      },
    });
    expect(JSON.stringify(result.debug_summary)).not.toContain(prompt);
  });

  it("adds sanitized debug summary to composer failures", async () => {
    const empty = {
      count: async () => 0,
      nth: () => empty,
      isVisible: async () => false,
      allTextContents: async () => [],
    };
    const page = {
      isClosed: () => false,
      locator: () => empty,
      waitForTimeout: async () => undefined,
    };
    const adapter = new GeminiBrowserAdapter({ env: {} });
    adapter.__setTestPage(page as never);

    await expect(
      adapter.sendSingle({
        browserProfileDir: "C:/Extractum/gemini-browser/profile",
        artifactDir: "artifacts/gemini-browser-adapter-test/run-composer-missing",
        request: {
          run_id: "run-composer-missing",
          prompt: "private prompt",
          source: "settings_test",
          artifact_mode: "reduced",
        },
      }),
    ).resolves.toMatchObject({
      status: "needs_login",
      debug_summary: {
        mode: "managed",
        composer_found: false,
        send_button_found: false,
        answer_found: false,
        answer_completion_reason: "missing",
        final_text_length: 0,
        error_stage: "composer",
      },
    });
  });

  it("adds sanitized debug summary to answer timeouts", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-06-21T00:00:00Z"));
    try {
      const composer = {
        count: async () => 1,
        nth: () => composer,
        isVisible: async () => true,
        fill: vi.fn(async () => undefined),
      };
      const send = {
        count: async () => 1,
        nth: () => send,
        isVisible: async () => true,
        click: vi.fn(async () => undefined),
      };
      const empty = {
        count: async () => 0,
        nth: () => empty,
        isVisible: async () => false,
        allTextContents: async () => [],
      };
      const page = {
        isClosed: () => false,
        locator: (selector: string) => {
          if (selector === "rich-textarea textarea") return composer;
          if (selector === "button[aria-label*='send' i]") return send;
          return empty;
        },
        waitForTimeout: async (ms: number) => {
          vi.advanceTimersByTime(ms);
        },
      };
      const adapter = new GeminiBrowserAdapter({ env: {} });
      adapter.__setTestPage(page as never);

      await expect(
        adapter.sendSingle({
          browserProfileDir: "C:/Extractum/gemini-browser/profile",
          artifactDir: "artifacts/gemini-browser-adapter-test/run-answer-timeout",
          request: {
            run_id: "run-answer-timeout",
            prompt: "private prompt",
            source: "settings_test",
            artifact_mode: "reduced",
          },
        }),
      ).resolves.toMatchObject({
        status: "timeout",
        debug_summary: {
          mode: "managed",
          composer_found: true,
          send_button_found: true,
          answer_found: false,
          answer_completion_reason: "missing",
          final_text_length: 0,
          error_stage: "answer",
        },
      });
    } finally {
      vi.useRealTimers();
    }
  });

  it("waits for a streaming Gemini answer to stabilize before returning text", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-06-21T00:00:00Z"));
    try {
      const prompt = "Ты знаешь последние новости ЧМ по футболу?";
      const finalAnswer =
        "Да, конечно! Прямо сейчас в США, Канаде и Мексике в самом разгаре групповой этап ЧМ-2026. Турнир преподносит немало сюрпризов.";
      let submitted = false;
      const composer = {
        count: async () => 1,
        nth: () => composer,
        isVisible: async () => true,
        fill: vi.fn(async () => undefined),
      };
      const send = {
        count: async () => 1,
        nth: () => send,
        isVisible: async () => true,
        click: vi.fn(async () => {
          submitted = true;
        }),
      };
      const answer = {
        count: async () => (submitted ? 1 : 0),
        nth: () => answer,
        isVisible: async () => true,
        allTextContents: vi.fn(async () => {
          if (!submitted) return [];
          const elapsed = Date.now() - new Date("2026-06-21T00:00:00Z").getTime();
          if (elapsed < 500) return ["Да,"];
          return [finalAnswer];
        }),
      };
      const completionAction = {
        count: async () => (submitted && Date.now() - new Date("2026-06-21T00:00:00Z").getTime() >= 500 ? 1 : 0),
        nth: () => completionAction,
        isVisible: async () => true,
      };
      const empty = {
        count: async () => 0,
        nth: () => empty,
        isVisible: async () => false,
        allTextContents: async () => [],
      };
      const page = {
        isClosed: () => false,
        locator: (selector: string) => {
          if (selector === "rich-textarea textarea") return composer;
          if (selector === "button[aria-label*='send' i]") return send;
          if (selector === "[data-response-index]") return answer;
          if (selector === "[data-test-id='copy-button']") return completionAction;
          return empty;
        },
        waitForTimeout: async (ms: number) => {
          vi.advanceTimersByTime(ms);
        },
      };
      const adapter = new GeminiBrowserAdapter({ env: {} });
      adapter.__setTestPage(page as never);

      await expect(
        adapter.sendSingle({
          browserProfileDir: "C:/Extractum/gemini-browser/profile",
          artifactDir: "artifacts/gemini-browser-adapter-test/run-1",
          request: {
            run_id: "run-1",
            prompt,
            source: "settings_test",
            artifact_mode: "reduced",
          },
        }),
      ).resolves.toMatchObject({
        status: "ok",
        text: finalAnswer,
        debug_summary: {
          mode: "managed",
          composer_found: true,
          send_button_found: true,
          answer_found: true,
          answer_selector: "[data-response-index]",
          answer_completion_reason: "stable",
          final_text_length: finalAnswer.length,
          error_stage: null,
        },
      });
    } finally {
      vi.useRealTimers();
    }
  });

  it("ignores previous answers and waits through a mid-generation pause", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-06-21T00:00:00Z"));
    try {
      const startedAt = new Date("2026-06-21T00:00:00Z").getTime();
      const finalAnswer =
        "Португалия, Франция и Бразилия остаются среди главных фаворитов, но групповой этап уже принес несколько неожиданных результатов.";
      const previousAnswer = "Предыдущий ответ из старого сообщения.";
      let submitted = false;
      const composer = {
        count: async () => 1,
        nth: () => composer,
        isVisible: async () => true,
        fill: vi.fn(async () => undefined),
      };
      const send = {
        count: async () => 1,
        nth: () => send,
        isVisible: async () => true,
        click: vi.fn(async () => {
          submitted = true;
        }),
      };
      const answer = {
        count: async () => (submitted ? 2 : 1),
        nth: () => answer,
        isVisible: async () => true,
        allTextContents: vi.fn(async () => {
          if (!submitted) return [previousAnswer];
          const elapsed = Date.now() - startedAt;
          if (elapsed < 5_000) return [previousAnswer, "П"];
          return [previousAnswer, finalAnswer];
        }),
      };
      const completionAction = {
        count: async () => (submitted && Date.now() - startedAt >= 5_000 ? 2 : 1),
        nth: () => completionAction,
        isVisible: async () => true,
      };
      const empty = {
        count: async () => 0,
        nth: () => empty,
        isVisible: async () => false,
        allTextContents: async () => [],
      };
      const page = {
        isClosed: () => false,
        locator: (selector: string) => {
          if (selector === "rich-textarea textarea") return composer;
          if (selector === "button[aria-label*='send' i]") return send;
          if (selector === "message-content") return answer;
          if (selector === "[data-test-id='copy-button']") return completionAction;
          return empty;
        },
        waitForTimeout: async (ms: number) => {
          vi.advanceTimersByTime(ms);
        },
      };
      const adapter = new GeminiBrowserAdapter({ env: {} });
      adapter.__setTestPage(page as never);

      await expect(
        adapter.sendSingle({
          browserProfileDir: "C:/Extractum/gemini-browser/profile",
          artifactDir: "artifacts/gemini-browser-adapter-test/run-2",
          request: {
            run_id: "run-2",
            prompt: "кто фаворит на чм по футболу?",
            source: "settings_test",
            artifact_mode: "reduced",
          },
        }),
      ).resolves.toMatchObject({
        status: "ok",
        text: finalAnswer,
      });
    } finally {
      vi.useRealTimers();
    }
  });

  it("does not treat early answer action buttons as completion", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-06-21T00:00:00Z"));
    try {
      const startedAt = new Date("2026-06-21T00:00:00Z").getTime();
      const partialAnswer = "Текущий Чемпионат мира 2026 в самом разгаре,";
      const finalAnswer =
        "Текущий Чемпионат мира 2026 в самом разгаре, и это исторический турнир с несколькими неожиданными результатами, интригой в группах и плотной борьбой фаворитов.";
      let submitted = false;
      const composer = {
        count: async () => 1,
        nth: () => composer,
        isVisible: async () => true,
        fill: vi.fn(async () => undefined),
      };
      const send = {
        count: async () => 1,
        nth: () => send,
        isVisible: async () => true,
        click: vi.fn(async () => {
          submitted = true;
        }),
      };
      const answer = {
        count: async () => (submitted ? 1 : 0),
        nth: () => answer,
        isVisible: async () => true,
        allTextContents: vi.fn(async () => {
          if (!submitted) return [];
          const elapsed = Date.now() - startedAt;
          if (elapsed < 5_000) return [partialAnswer];
          return [finalAnswer];
        }),
      };
      const earlyCompletionAction = {
        count: async () => (submitted ? 1 : 0),
        nth: () => earlyCompletionAction,
        isVisible: async () => true,
      };
      const empty = {
        count: async () => 0,
        nth: () => empty,
        isVisible: async () => false,
        allTextContents: async () => [],
      };
      const page = {
        isClosed: () => false,
        locator: (selector: string) => {
          if (selector === "rich-textarea textarea") return composer;
          if (selector === "button[aria-label*='send' i]") return send;
          if (selector === "message-content") return answer;
          if (selector === "[data-test-id='copy-button']") return earlyCompletionAction;
          return empty;
        },
        waitForTimeout: async (ms: number) => {
          vi.advanceTimersByTime(ms);
        },
      };
      const adapter = new GeminiBrowserAdapter({ env: {} });
      adapter.__setTestPage(page as never);

      await expect(
        adapter.sendSingle({
          browserProfileDir: "C:/Extractum/gemini-browser/profile",
          artifactDir: "artifacts/gemini-browser-adapter-test/run-3",
          request: {
            run_id: "run-3",
            prompt: "что происходит на чемпионате мира?",
            source: "settings_test",
            artifact_mode: "reduced",
          },
        }),
      ).resolves.toMatchObject({
        status: "ok",
        text: finalAnswer,
      });
    } finally {
      vi.useRealTimers();
    }
  });

  it("marks answer completion as timeout_latest when visible text never stabilizes", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-06-21T00:00:00Z"));
    try {
      const startedAt = Date.now();
      let submitted = false;
      const composer = {
        count: async () => 1,
        nth: () => composer,
        isVisible: async () => true,
        fill: vi.fn(async () => undefined),
      };
      const send = {
        count: async () => 1,
        nth: () => send,
        isVisible: async () => true,
        click: vi.fn(async () => {
          submitted = true;
        }),
      };
      const answer = {
        count: async () => (submitted ? 1 : 0),
        nth: () => answer,
        isVisible: async () => true,
        allTextContents: async () => {
          if (!submitted) return [];
          return [`partial answer ${Date.now() - startedAt}`];
        },
      };
      const empty = {
        count: async () => 0,
        nth: () => empty,
        isVisible: async () => false,
        allTextContents: async () => [],
      };
      const page = {
        isClosed: () => false,
        locator: (selector: string) => {
          if (selector === "rich-textarea textarea") return composer;
          if (selector === "button[aria-label*='send' i]") return send;
          if (selector === "message-content") return answer;
          return empty;
        },
        waitForTimeout: async (ms: number) => {
          vi.advanceTimersByTime(ms);
        },
      };
      const adapter = new GeminiBrowserAdapter({ env: {} });
      adapter.__setTestPage(page as never);

      const result = await adapter.sendSingle({
        browserProfileDir: "C:/Extractum/gemini-browser/profile",
        artifactDir: "artifacts/gemini-browser-adapter-test/run-timeout-latest",
        request: {
          run_id: "run-timeout-latest",
          prompt: "slow prompt",
          source: "settings_test",
          artifact_mode: "reduced",
        },
      });

      expect(result).toMatchObject({
        status: "ok",
        debug_summary: {
          answer_found: true,
          answer_completion_reason: "timeout_latest",
        },
      });
      expect(result.text).toContain("partial answer");
    } finally {
      vi.useRealTimers();
    }
  });

  it("waits for Gemini to finish a previous generation before sending", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-06-21T00:00:00Z"));
    try {
      const startedAt = Date.now();
      const finalAnswer = "Теперь отвечаю на прошлый вопрос полностью.";
      let submitted = false;
      const composer = {
        count: async () => 1,
        nth: () => composer,
        isVisible: async () => true,
        fill: vi.fn(async () => undefined),
      };
      const send = {
        count: async () => (Date.now() - startedAt >= 15_000 ? 1 : 0),
        nth: () => send,
        isVisible: async () => Date.now() - startedAt >= 15_000,
        click: vi.fn(async () => {
          submitted = true;
        }),
      };
      const stopGenerating = {
        count: async () => (Date.now() - startedAt < 15_000 ? 1 : 0),
        nth: () => stopGenerating,
        isVisible: async () => Date.now() - startedAt < 15_000,
      };
      const answer = {
        count: async () => (submitted ? 1 : 0),
        nth: () => answer,
        isVisible: async () => true,
        allTextContents: async () => (submitted ? [finalAnswer] : []),
      };
      const empty = {
        count: async () => 0,
        nth: () => empty,
        isVisible: async () => false,
        allTextContents: async () => [],
      };
      const page = {
        isClosed: () => false,
        locator: (selector: string) => {
          if (selector === "rich-textarea textarea") return composer;
          if (selector === "button[aria-label*='send' i]") return send;
          if (selector === "button[aria-label*='Останов' i]") return stopGenerating;
          if (selector === "message-content") return answer;
          return empty;
        },
        waitForTimeout: async (ms: number) => {
          vi.advanceTimersByTime(ms);
        },
      };
      const adapter = new GeminiBrowserAdapter({ env: {} });
      adapter.__setTestPage(page as never);

      await expect(
        adapter.sendSingle({
          browserProfileDir: "C:/Extractum/gemini-browser/profile",
          artifactDir: "artifacts/gemini-browser-adapter-test/run-4",
          request: {
            run_id: "run-4",
            prompt: "ответь на прошлый вопрос",
            source: "settings_test",
            artifact_mode: "reduced",
          },
        }),
      ).resolves.toMatchObject({
        status: "ok",
        text: finalAnswer,
        debug_summary: {
          generation_busy_observed: true,
          send_button_found: true,
        },
      });
    } finally {
      vi.useRealTimers();
    }
  });
});
