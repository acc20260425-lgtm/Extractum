import { describe, expect, it } from "vitest";
import {
  groupEditorStateFromGroup,
  isGroupSourceSelected,
  templateCopyCommand,
  templateCreatedStatus,
  templateDeletedStatus,
  templateEditorStateFromTemplate,
  templateUpdateCommand,
  templateUpdatedStatus,
  templateDeleteDecision,
  templateFallbackSelection,
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

  it("validates and builds update commands for editable templates", () => {
    expect(templateUpdateCommand(null, "Name", "Body")).toEqual({
      ok: false,
      status: "Select a template first.",
    });
    expect(templateUpdateCommand(template({ is_builtin: true }), "Name", "Body")).toEqual({
      ok: false,
      status: "Built-in templates cannot be edited directly. Save a copy instead.",
    });
    expect(templateUpdateCommand(template(), " ", "Body")).toEqual({
      ok: false,
      status: "Template name and body cannot be empty.",
    });
    expect(templateUpdateCommand(template({ id: 9 }), " Name ", " Body ")).toEqual({
      ok: true,
      templateId: 9,
      name: "Name",
      body: "Body",
    });
  });

  it("validates and builds copy commands for templates", () => {
    expect(templateCopyCommand(" ", "Body")).toEqual({
      ok: false,
      status: "Template name and body cannot be empty.",
    });
    expect(templateCopyCommand(" Name ", " Body ")).toEqual({
      ok: true,
      name: "Name",
      body: "Body",
    });
  });

  it("validates template deletion and formats template command results", () => {
    expect(templateDeleteDecision(null)).toEqual({
      ok: false,
      status: "Select a template first.",
    });
    expect(templateDeleteDecision(template({ is_builtin: true }))).toEqual({
      ok: false,
      status: "Built-in templates cannot be deleted.",
    });
    expect(templateDeleteDecision(template({ id: 11, name: "Digest" }))).toEqual({
      ok: true,
      templateId: 11,
      name: "Digest",
    });
    expect(templateUpdatedStatus(template({ name: "Digest" }))).toBe('Template "Digest" saved.');
    expect(templateCreatedStatus(template({ name: "Digest" }))).toBe('Template "Digest" created.');
    expect(templateDeletedStatus("Digest")).toBe('Template "Digest" deleted.');
    expect(templateFallbackSelection([template({ id: 4 })])).toEqual({
      selectedTemplateId: "4",
      template: template({ id: 4 }),
    });
    expect(templateFallbackSelection([])).toEqual({
      selectedTemplateId: "",
      template: null,
    });
  });
});
