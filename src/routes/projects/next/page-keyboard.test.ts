import { describe, expect, it } from "vitest";
import rawPageSource from "./+page.svelte?raw";

const source = rawPageSource.replace(/\r\n/g, "\n");

describe("/projects/next source keyboard integration", () => {
  it("passes keyboard navigation callbacks to the source grid", () => {
    expect(source).toContain("keyboardNavigationEnabled=");
    expect(source).toContain("!connectOpen && !addSourceOpen && !disconnectOpen");
    expect(source).toContain('keyboardHint: "↑↓ строка · Enter инспектор"');
    expect(source).toContain("onKeyboardActivateSource");
    expect(source).toContain("onKeyboardInspectSource");
    expect(source).toContain("onKeyboardEscape");
  });
});
