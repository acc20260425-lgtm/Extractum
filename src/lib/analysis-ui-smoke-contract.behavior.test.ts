import { describe, expect, it } from "vitest";

describe("analysis UI smoke harness contract", () => {
  it("associates source-group NotebookLM disabled reason through aria-describedby", async () => {
    const modulePath = "./analysis-ui-smoke-contract";
    const contract = await import(/* @vite-ignore */ modulePath);
    const reason = "YouTube source-group NotebookLM export is not implemented yet.";

    expect(contract.notebookLmExportAccessibility(false, reason)).toEqual({
      reasonId: "notebooklm-export-disabled-reason",
      ariaDescribedby: "notebooklm-export-disabled-reason",
      showReason: true,
      buttonSmokeId: "notebooklm-export-button",
      reasonSmokeId: "notebooklm-export-disabled-reason",
    });
    expect(contract.notebookLmExportAccessibility(true, reason)).toMatchObject({
      ariaDescribedby: undefined,
      showReason: false,
    });
    expect(reason).toBe("YouTube source-group NotebookLM export is not implemented yet.");
  });
});
