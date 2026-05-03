import type { AnalysisPromptTemplate, AnalysisSourceGroup } from "$lib/types/analysis";

export type TemplateEditorState = {
  editorBoundTemplateId: number | null;
  templateName: string;
  templateBody: string;
};

export type GroupEditorState = {
  editorBoundGroupId: number | null;
  groupName: string;
  groupMemberSourceIds: number[];
};

export function templateEditorStateFromTemplate(
  template: AnalysisPromptTemplate | null,
): TemplateEditorState {
  if (!template) {
    return {
      editorBoundTemplateId: null,
      templateName: "",
      templateBody: "",
    };
  }

  return {
    editorBoundTemplateId: template.id,
    templateName: template.name,
    templateBody: template.body,
  };
}

export function groupEditorStateFromGroup(
  group: AnalysisSourceGroup | null,
): GroupEditorState {
  if (!group) {
    return {
      editorBoundGroupId: null,
      groupName: "",
      groupMemberSourceIds: [],
    };
  }

  return {
    editorBoundGroupId: group.id,
    groupName: group.name,
    groupMemberSourceIds: group.members.map((member) => member.source_id),
  };
}

export function isGroupSourceSelected(
  groupMemberSourceIds: number[],
  sourceId: number,
) {
  return groupMemberSourceIds.includes(sourceId);
}

export function toggleGroupSourceSelection(
  groupMemberSourceIds: number[],
  sourceId: number,
) {
  if (groupMemberSourceIds.includes(sourceId)) {
    return groupMemberSourceIds.filter((id) => id !== sourceId);
  }

  return [...groupMemberSourceIds, sourceId].sort((a, b) => a - b);
}
