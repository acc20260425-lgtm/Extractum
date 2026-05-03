import { describe, expect, it } from "vitest";
import {
  groupEditorStateFromGroup,
  isGroupSourceSelected,
  templateEditorStateFromTemplate,
  toggleGroupSourceSelection,
} from "./analysis-editor-state";
import type { AnalysisPromptTemplate, AnalysisSourceGroup } from "./types/analysis";

function template(overrides: Partial<AnalysisPromptTemplate> = {}): AnalysisPromptTemplate {
  return {
    id: 1,
    name: "Weekly report",
    template_kind: "report",
    body: "Summarize this source.",
    version: 1,
    is_builtin: false,
    created_at: 100,
    updated_at: 200,
    ...overrides,
  };
}

function group(overrides: Partial<AnalysisSourceGroup> = {}): AnalysisSourceGroup {
  return {
    id: 7,
    name: "Research group",
    members: [
      { source_id: 3, source_title: "Gamma", item_count: 30 },
      { source_id: 1, source_title: "Alpha", item_count: 10 },
    ],
    created_at: 100,
    updated_at: 200,
    ...overrides,
  };
}

describe("analysis-editor-state", () => {
  it("creates an empty template editor snapshot when no template is selected", () => {
    expect(templateEditorStateFromTemplate(null)).toEqual({
      editorBoundTemplateId: null,
      templateName: "",
      templateBody: "",
    });
  });

  it("creates a template editor snapshot from the selected template", () => {
    expect(templateEditorStateFromTemplate(template({
      id: 5,
      name: "Incident digest",
      body: "Find notable incidents.",
    }))).toEqual({
      editorBoundTemplateId: 5,
      templateName: "Incident digest",
      templateBody: "Find notable incidents.",
    });
  });

  it("creates an empty group editor snapshot when no group is selected", () => {
    expect(groupEditorStateFromGroup(null)).toEqual({
      editorBoundGroupId: null,
      groupName: "",
      groupMemberSourceIds: [],
    });
  });

  it("creates a group editor snapshot from group members in stored order", () => {
    expect(groupEditorStateFromGroup(group())).toEqual({
      editorBoundGroupId: 7,
      groupName: "Research group",
      groupMemberSourceIds: [3, 1],
    });
  });

  it("checks and toggles selected group source ids without mutating the current list", () => {
    const current = [3, 1];

    expect(isGroupSourceSelected(current, 3)).toBe(true);
    expect(isGroupSourceSelected(current, 2)).toBe(false);
    expect(toggleGroupSourceSelection(current, 3)).toEqual([1]);
    expect(toggleGroupSourceSelection(current, 2)).toEqual([1, 2, 3]);
    expect(current).toEqual([3, 1]);
  });
});
