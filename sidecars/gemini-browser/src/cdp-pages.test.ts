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

  it("returns the first usable Gemini page in page order", () => {
    const first = page("https://gemini.google.com/app");
    const second = page("https://gemini.google.com/app/second");

    expect(selectGeminiPage([first, second])).toBe(first);
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
