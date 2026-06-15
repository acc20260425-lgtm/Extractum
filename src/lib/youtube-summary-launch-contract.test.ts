// @ts-nocheck
import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";

describe("youtube summary launch contract", () => {
  it("wires launch dialog through the library inspector", () => {
    const inspector = readFileSync("src/lib/components/research-projects/LibraryInspector.svelte", "utf8");
    const dialog = readFileSync("src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte", "utf8");
    const source = `${inspector}\n${dialog}`;

    expect(source).toContain("YoutubeSummaryRunDialog");
    expect(source).toContain("preflightYoutubeSummaryRun");
    expect(source).toContain("startYoutubeSummaryRun");
  });

  it("wires project source launches with the selected project id", () => {
    const inspector = readFileSync("src/lib/components/research-projects/ProjectInspector.svelte", "utf8");
    const dialog = readFileSync("src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte", "utf8");

    expect(inspector).toContain("YoutubeSummaryRunDialog");
    expect(inspector).toContain("projectId={project?.projectId ?? null}");
    expect(inspector).toContain("selectedSource.sourceNumericId");
    expect(dialog).toContain("projectId = null");
    expect(dialog).toContain("projectId,");
    expect(dialog).not.toContain("projectId: null");
  });
});
