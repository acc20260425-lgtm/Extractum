import { describe, expect, it } from "vitest";

describe("analysis group editor props", () => {
  it("keeps group editor selection owned by the report canvas workspace tools", async () => {
    const modulePath = "./analysis-group-editor-props";
    const contract = await import(/* @vite-ignore */ modulePath);

    const selections: string[] = [];
    const props = contract.reportCanvasGroupEditorProps("group-7", (value: string) => selections.push(value));
    expect(props).toEqual({
      selectedGroupEditorId: "group-7",
      onChangeSelectedGroupId: expect.any(Function),
    });
    props.onChangeSelectedGroupId("group-8");
    expect(selections).toEqual(["group-8"]);
  });
});
