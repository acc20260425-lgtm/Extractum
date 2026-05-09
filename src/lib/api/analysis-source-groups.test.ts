import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  createAnalysisPromptTemplate,
  createAnalysisSourceGroup,
  deleteAnalysisPromptTemplate,
  deleteAnalysisSourceGroup,
  listAnalysisPromptTemplates,
  listAnalysisSourceGroups,
  updateAnalysisPromptTemplate,
  updateAnalysisSourceGroup,
} from "./analysis-source-groups";
import type { AnalysisPromptTemplate, AnalysisSourceGroup } from "$lib/types/analysis";

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
      source_type: "telegram",
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

  it("loads prompt templates with the expected template kind payload", async () => {
    const templates: AnalysisPromptTemplate[] = [{
      id: 5,
      name: "Report",
      template_kind: "report",
      body: "Summarize",
      version: 1,
      is_builtin: false,
      created_at: 100,
      updated_at: 200,
    }];
    invokeMock.mockResolvedValueOnce(templates);

    await expect(listAnalysisPromptTemplates("report")).resolves.toEqual(templates);

    expect(invokeMock).toHaveBeenLastCalledWith("list_analysis_prompt_templates", {
      templateKind: "report",
    });
  });

  it("creates a prompt template with the expected payload", async () => {
    const created: AnalysisPromptTemplate = {
      id: 6,
      name: "Copy",
      template_kind: "report",
      body: "Body",
      version: 1,
      is_builtin: false,
      created_at: 100,
      updated_at: 200,
    };
    invokeMock.mockResolvedValueOnce(created);

    await expect(createAnalysisPromptTemplate({
      name: "Copy",
      templateKind: "report",
      body: "Body",
    })).resolves.toEqual(created);

    expect(invokeMock).toHaveBeenLastCalledWith("create_analysis_prompt_template", {
      name: "Copy",
      templateKind: "report",
      body: "Body",
    });
  });

  it("updates a prompt template with the expected payload", async () => {
    const updated: AnalysisPromptTemplate = {
      id: 6,
      name: "Updated",
      template_kind: "report",
      body: "Body",
      version: 2,
      is_builtin: false,
      created_at: 100,
      updated_at: 300,
    };
    invokeMock.mockResolvedValueOnce(updated);

    await expect(updateAnalysisPromptTemplate({
      templateId: 6,
      name: "Updated",
      body: "Body",
    })).resolves.toEqual(updated);

    expect(invokeMock).toHaveBeenLastCalledWith("update_analysis_prompt_template", {
      templateId: 6,
      name: "Updated",
      body: "Body",
    });
  });

  it("deletes a prompt template with the expected payload", async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await expect(deleteAnalysisPromptTemplate(42)).resolves.toBeUndefined();

    expect(invokeMock).toHaveBeenLastCalledWith("delete_analysis_prompt_template", {
      templateId: 42,
    });
  });

  it("creates a source group with the expected payload", async () => {
    const created: AnalysisSourceGroup = {
      id: 12,
      name: "New group",
      source_type: "youtube",
      members: [],
      created_at: 100,
      updated_at: 100,
    };
    invokeMock.mockResolvedValueOnce(created);

    await expect(createAnalysisSourceGroup({
      name: "New group",
      sourceType: "youtube",
      sourceIds: [3, 7],
    })).resolves.toEqual(created);

    expect(invokeMock).toHaveBeenLastCalledWith("create_analysis_source_group", {
      name: "New group",
      sourceType: "youtube",
      sourceIds: [3, 7],
    });
  });

  it("updates a source group with the expected payload", async () => {
    const updated: AnalysisSourceGroup = {
      id: 12,
      name: "Updated group",
      source_type: "telegram",
      members: [],
      created_at: 100,
      updated_at: 200,
    };
    invokeMock.mockResolvedValueOnce(updated);

    await expect(updateAnalysisSourceGroup({
      groupId: 12,
      name: "Updated group",
      sourceType: "telegram",
      sourceIds: [7],
    })).resolves.toEqual(updated);

    expect(invokeMock).toHaveBeenLastCalledWith("update_analysis_source_group", {
      groupId: 12,
      name: "Updated group",
      sourceType: "telegram",
      sourceIds: [7],
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
