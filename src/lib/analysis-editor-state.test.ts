import { describe, expect, it } from "vitest";
import {
  groupCopyCommand,
  groupCreatedStatus,
  groupDeleteDecision,
  groupDeletedStatus,
  groupEditorStateFromGroup,
  groupFallbackSelection,
  groupUpdateCommand,
  groupUpdatedStatus,
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
    source_type: "telegram",
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
      groupSourceType: "telegram",
      groupMemberSourceIds: [],
    });
  });

  it("creates a group editor snapshot from group members in stored order", () => {
    expect(groupEditorStateFromGroup(group({ source_type: "youtube" }))).toEqual({
      editorBoundGroupId: 7,
      groupName: "Research group",
      groupSourceType: "youtube",
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

  it("validates and builds update commands for source groups", () => {
    expect(groupUpdateCommand(null, "Name", [1])).toEqual({
      ok: false,
      status: "Select a source group first.",
    });
    expect(groupUpdateCommand(group({ id: 9 }), " ", [1])).toEqual({
      ok: false,
      status: "Group name cannot be empty.",
    });
    expect(groupUpdateCommand(group({ id: 9 }), "Name", [])).toEqual({
      ok: false,
      status: "Select at least one source for the group.",
    });
    expect(groupUpdateCommand(group({ id: 9 }), " Name ", [3, 1])).toEqual({
      ok: true,
      groupId: 9,
      name: "Name",
      sourceIds: [3, 1],
    });
  });

  it("validates and builds copy commands for source groups", () => {
    expect(groupCopyCommand(" ", [1])).toEqual({
      ok: false,
      status: "Group name cannot be empty.",
    });
    expect(groupCopyCommand("Name", [])).toEqual({
      ok: false,
      status: "Select at least one source for the group.",
    });
    expect(groupCopyCommand(" Name ", [3, 1])).toEqual({
      ok: true,
      name: "Name",
      sourceIds: [3, 1],
    });
  });

  it("validates group deletion and formats group command results", () => {
    expect(groupDeleteDecision(null)).toEqual({
      ok: false,
      status: "Select a source group first.",
    });
    expect(groupDeleteDecision(group({ id: 11, name: "Research" }))).toEqual({
      ok: true,
      groupId: 11,
      name: "Research",
    });
    expect(groupUpdatedStatus(group({ name: "Research" }))).toBe('Source group "Research" saved.');
    expect(groupCreatedStatus(group({ name: "Research" }))).toBe('Source group "Research" created.');
    expect(groupDeletedStatus("Research")).toBe('Source group "Research" deleted.');
    expect(groupFallbackSelection([group({ id: 4 })])).toEqual({
      selectedGroupId: "4",
      group: group({ id: 4 }),
    });
    expect(groupFallbackSelection([])).toEqual({
      selectedGroupId: "",
      group: null,
    });
  });
});
