import { describe, expect, it } from "vitest";
import rawSource from "./DataGrid.svelte?raw";

const source = rawSource.replace(/\r\n/g, "\n");

describe("Extractum DataGrid", () => {
  it("passes responsive column definitions through to SVAR Grid", () => {
    expect(source).toContain("responsive");
    expect(source).toContain("enhancedResponsive");
    expect(source).toContain("responsive={enhancedResponsive}");
  });
});
