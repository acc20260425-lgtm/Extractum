import type {
  AnalysisSourceGroup,
  AnalysisSourceOption,
} from "$lib/types/analysis";
import type { Source } from "$lib/types/sources";

export type AnalysisScope = "single_source" | "source_group";
export type AnalysisHistoryScope = "all" | "current";

export type AnalysisHistoryScopeParams = {
  sourceId: number | null;
  sourceGroupId: number | null;
};

export function currentAnalysisSource(
  selectedSourceId: string,
  sourceCatalog: Source[],
) {
  if (!selectedSourceId) return null;
  return sourceCatalog.find((source) => source.id === Number(selectedSourceId)) ?? null;
}

export function currentAnalysisSourceMetric(
  source: Source | null,
  sourceMetrics: Record<number, AnalysisSourceOption>,
) {
  return source ? sourceMetrics[source.id] ?? null : null;
}

export function currentAnalysisGroup(
  selectedGroupId: string,
  groups: AnalysisSourceGroup[],
) {
  if (!selectedGroupId) return null;
  return groups.find((group) => group.id === Number(selectedGroupId)) ?? null;
}

export function currentAnalysisScopeTitle(
  analysisScope: AnalysisScope,
  source: Source | null,
  group: AnalysisSourceGroup | null,
) {
  if (analysisScope === "source_group") {
    return group?.name ?? "Source group";
  }
  return source?.title ?? source?.externalId ?? "Source";
}

export function currentAnalysisScopeSummary(
  analysisScope: AnalysisScope,
  source: Source | null,
  group: AnalysisSourceGroup | null,
  metrics: AnalysisSourceOption | null,
) {
  if (analysisScope === "source_group") {
    if (!group) return "Select a saved source group to run a cross-source report.";
    return `${group.members.length} sources in this group workspace.`;
  }

  if (!source) return "Select a synced source to inspect messages and launch a report.";
  if (metrics) {
    return `${metrics.item_count} synced items available locally for analysis.`;
  }
  return "This source is available in the workspace but has no synced item count yet.";
}

export function analysisHistoryScopeParams(
  historyScope: AnalysisHistoryScope,
  analysisScope: AnalysisScope,
  selectedSourceId: string,
  selectedGroupId: string,
): AnalysisHistoryScopeParams | null {
  if (historyScope === "all") {
    return {
      sourceId: null,
      sourceGroupId: null,
    };
  }

  if (analysisScope === "single_source" && selectedSourceId) {
    return {
      sourceId: Number(selectedSourceId),
      sourceGroupId: null,
    };
  }

  if (analysisScope === "source_group" && selectedGroupId) {
    return {
      sourceId: null,
      sourceGroupId: Number(selectedGroupId),
    };
  }

  return null;
}
