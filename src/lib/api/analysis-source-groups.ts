import { invoke } from "@tauri-apps/api/core";
import type { AnalysisSourceGroup } from "$lib/types/analysis";

export function listAnalysisSourceGroups() {
  return invoke<AnalysisSourceGroup[]>("list_analysis_source_groups");
}

export function deleteAnalysisPromptTemplate(templateId: number) {
  return invoke<void>("delete_analysis_prompt_template", { templateId });
}

export function deleteAnalysisSourceGroup(groupId: number) {
  return invoke<void>("delete_analysis_source_group", { groupId });
}
