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
});
