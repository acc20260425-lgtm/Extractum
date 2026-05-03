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

export type TemplateUpdateCommand =
  | { ok: false; status: string }
  | { ok: true; templateId: number; name: string; body: string };

export type TemplateCopyCommand =
  | { ok: false; status: string }
  | { ok: true; name: string; body: string };

export type TemplateDeleteDecision =
  | { ok: false; status: string }
  | { ok: true; templateId: number; name: string };

export type GroupUpdateCommand =
  | { ok: false; status: string }
  | { ok: true; groupId: number; name: string; sourceIds: number[] };

export type GroupCopyCommand =
  | { ok: false; status: string }
  | { ok: true; name: string; sourceIds: number[] };

export type GroupDeleteDecision =
  | { ok: false; status: string }
  | { ok: true; groupId: number; name: string };

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

export function templateUpdateCommand(
  template: AnalysisPromptTemplate | null,
  name: string,
  body: string,
): TemplateUpdateCommand {
  if (!template) {
    return { ok: false, status: "Select a template first." };
  }

  if (template.is_builtin) {
    return {
      ok: false,
      status: "Built-in templates cannot be edited directly. Save a copy instead.",
    };
  }

  const trimmedName = name.trim();
  const trimmedBody = body.trim();
  if (!trimmedName || !trimmedBody) {
    return { ok: false, status: "Template name and body cannot be empty." };
  }

  return {
    ok: true,
    templateId: template.id,
    name: trimmedName,
    body: trimmedBody,
  };
}

export function templateCopyCommand(
  name: string,
  body: string,
): TemplateCopyCommand {
  const trimmedName = name.trim();
  const trimmedBody = body.trim();
  if (!trimmedName || !trimmedBody) {
    return { ok: false, status: "Template name and body cannot be empty." };
  }

  return {
    ok: true,
    name: trimmedName,
    body: trimmedBody,
  };
}

export function templateDeleteDecision(
  template: AnalysisPromptTemplate | null,
): TemplateDeleteDecision {
  if (!template) {
    return { ok: false, status: "Select a template first." };
  }

  if (template.is_builtin) {
    return { ok: false, status: "Built-in templates cannot be deleted." };
  }

  return {
    ok: true,
    templateId: template.id,
    name: template.name,
  };
}

export function templateUpdatedStatus(template: Pick<AnalysisPromptTemplate, "name">) {
  return `Template "${template.name}" saved.`;
}

export function templateCreatedStatus(template: Pick<AnalysisPromptTemplate, "name">) {
  return `Template "${template.name}" created.`;
}

export function templateDeletedStatus(name: string) {
  return `Template "${name}" deleted.`;
}

export function templateFallbackSelection(templates: AnalysisPromptTemplate[]) {
  const template = templates[0] ?? null;
  return {
    selectedTemplateId: template ? String(template.id) : "",
    template,
  };
}

export function groupUpdateCommand(
  group: AnalysisSourceGroup | null,
  name: string,
  sourceIds: number[],
): GroupUpdateCommand {
  if (!group) {
    return { ok: false, status: "Select a source group first." };
  }

  const trimmedName = name.trim();
  if (!trimmedName) {
    return { ok: false, status: "Group name cannot be empty." };
  }

  if (sourceIds.length === 0) {
    return { ok: false, status: "Select at least one source for the group." };
  }

  return {
    ok: true,
    groupId: group.id,
    name: trimmedName,
    sourceIds,
  };
}

export function groupCopyCommand(
  name: string,
  sourceIds: number[],
): GroupCopyCommand {
  const trimmedName = name.trim();
  if (!trimmedName) {
    return { ok: false, status: "Group name cannot be empty." };
  }

  if (sourceIds.length === 0) {
    return { ok: false, status: "Select at least one source for the group." };
  }

  return {
    ok: true,
    name: trimmedName,
    sourceIds,
  };
}

export function groupDeleteDecision(
  group: AnalysisSourceGroup | null,
): GroupDeleteDecision {
  if (!group) {
    return { ok: false, status: "Select a source group first." };
  }

  return {
    ok: true,
    groupId: group.id,
    name: group.name,
  };
}

export function groupUpdatedStatus(group: Pick<AnalysisSourceGroup, "name">) {
  return `Source group "${group.name}" saved.`;
}

export function groupCreatedStatus(group: Pick<AnalysisSourceGroup, "name">) {
  return `Source group "${group.name}" created.`;
}

export function groupDeletedStatus(name: string) {
  return `Source group "${name}" deleted.`;
}

export function groupFallbackSelection(groups: AnalysisSourceGroup[]) {
  const group = groups[0] ?? null;
  return {
    selectedGroupId: group ? String(group.id) : "",
    group,
  };
}
