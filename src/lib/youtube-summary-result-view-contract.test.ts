// @ts-nocheck
import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";

describe("youtube summary result view contract", () => {
  it("loads and displays canonical result structures", () => {
    const source = readFileSync(
      "src/lib/components/research-projects/YoutubeSummaryResultView.svelte",
      "utf8",
    );

    expect(source).toContain("getPromptPackResult");
    expect(source).toContain("claims");
    expect(source).toContain("evidence");
    expect(source).toContain("limitations");
    expect(source).toContain("qualityFlags");
  });

  it("renders the overall readable summary from canonical sections", () => {
    const source = readFileSync(
      "src/lib/components/research-projects/YoutubeSummaryResultView.svelte",
      "utf8",
    );

    expect(source).toContain('arrayAt(recordAt(canonical, "outputs"), "sections")');
    expect(source).toContain('"section_summary"');
    expect(source).toContain('textAt(readableSummarySection ?? {}, "body")');
    expect(source).toContain("summary-box");
  });
});
