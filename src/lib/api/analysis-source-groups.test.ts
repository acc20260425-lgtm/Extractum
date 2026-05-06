import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  deleteAnalysisPromptTemplate,
  deleteAnalysisSourceGroup,
  listAnalysisSourceGroups,
} from "./analysis-source-groups";
import type { AnalysisSourceGroup } from "$lib/types/analysis";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

describe("analysis source groups api wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("loads analysis source groups with the registered command name", async () => {
    const groups: AnalysisSourceGroup[] = [{
      id: 10,
      name: "Research",
      members: [{
        source_id: 7,
        source_title: "Source",
        item_count: 12,
      }],
      created_at: 100,
      updated_at: 200,
    }];
    invokeMock.mockResolvedValueOnce(groups);

    await expect(listAnalysisSourceGroups()).resolves.toEqual(groups);

    expect(invokeMock).toHaveBeenLastCalledWith("list_analysis_source_groups");
  });

  it("deletes a prompt template with the expected payload", async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await expect(deleteAnalysisPromptTemplate(42)).resolves.toBeUndefined();

    expect(invokeMock).toHaveBeenLastCalledWith("delete_analysis_prompt_template", {
      templateId: 42,
    });
  });

  it("deletes a source group with the expected payload", async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await expect(deleteAnalysisSourceGroup(9)).resolves.toBeUndefined();

    expect(invokeMock).toHaveBeenLastCalledWith("delete_analysis_source_group", {
      groupId: 9,
    });
  });
});
