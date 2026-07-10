import { describe, expect, it } from "vitest";
import rawSource from "./DataGrid.svelte?raw";

const source = rawSource.replace(/\r\n/g, "\n");

describe("Extractum DataGrid", () => {
  it("passes responsive column definitions through to SVAR Grid", () => {
    expect(source).toContain("responsive");
    expect(source).toContain("enhancedResponsive");
    expect(source).toContain("responsive={enhancedResponsive}");
  });

  it("uses the v11 active-row focus ring", () => {
    expect(source).toContain("inset 2px 0 0 var(--extractum-primary)");
    expect(source).toContain("inset 0 0 0 1px");
  });

  it("accepts a rowHeight prop with a stable sizes object (svar clears sortMarks on reactive prop changes)", () => {
    expect(source).toContain("rowHeight = 34");
    expect(source).toContain("rowHeight?: number");
    // sizes must be built once from the initial prop value, not recreated
    // reactively — and the prop must actually feed GRID_SIZES
    expect(source).toContain("const GRID_SIZES = { rowHeight: untrack(() => rowHeight)");
    expect(source).toContain("sizes={GRID_SIZES}");
  });
});
