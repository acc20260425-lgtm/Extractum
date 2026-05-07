import { invoke } from "@tauri-apps/api/core";
import type {
  AnalysisPromptTemplate,
  AnalysisPromptTemplateKind,
  AnalysisSourceGroup,
  CreateAnalysisPromptTemplateInput,
  CreateAnalysisSourceGroupInput,
  UpdateAnalysisPromptTemplateInput,
  UpdateAnalysisSourceGroupInput,
} from "$lib/types/analysis";

export function listAnalysisSourceGroups() {
  return invoke<AnalysisSourceGroup[]>("list_analysis_source_groups");
}

export function listAnalysisPromptTemplates(templateKind: AnalysisPromptTemplateKind) {
  return invoke<AnalysisPromptTemplate[]>("list_analysis_prompt_templates", { templateKind });
}

export function createAnalysisPromptTemplate(input: CreateAnalysisPromptTemplateInput) {
  return invoke<AnalysisPromptTemplate>("create_analysis_prompt_template", { ...input });
}

export function updateAnalysisPromptTemplate(input: UpdateAnalysisPromptTemplateInput) {
  return invoke<AnalysisPromptTemplate>("update_analysis_prompt_template", { ...input });
}

export function deleteAnalysisPromptTemplate(templateId: number) {
  return invoke<void>("delete_analysis_prompt_template", { templateId });
}

export function createAnalysisSourceGroup(input: CreateAnalysisSourceGroupInput) {
  return invoke<AnalysisSourceGroup>("create_analysis_source_group", { ...input });
}

export function updateAnalysisSourceGroup(input: UpdateAnalysisSourceGroupInput) {
  return invoke<AnalysisSourceGroup>("update_analysis_source_group", { ...input });
}

export function deleteAnalysisSourceGroup(groupId: number) {
  return invoke<void>("delete_analysis_source_group", { groupId });
}
