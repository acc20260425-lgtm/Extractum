import { describe, expect, it } from "vitest";

describe("analysis group editor props", () => {
  it("keeps group editor selection owned by the report canvas workspace tools", async () => {
    const modulePath = "./analysis-group-editor-props";
    const contract = await import(/* @vite-ignore */ modulePath);

    expect(contract.reportCanvasGroupEditorProps("group-7")).toEqual({
      selectedGroupEditorId: "group-7",
    });
  });
});
