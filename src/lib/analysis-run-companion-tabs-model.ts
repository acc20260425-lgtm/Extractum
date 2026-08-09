import { runsFilterDefaults, type CompanionRunsFilterState } from "$lib/analysis-run-companion-state";
import type { CompanionTab } from "$lib/analysis-workspace-state";

export function runCompanionTabId(tab: CompanionTab) {
  return `run-companion-tab-${tab}`;
}

export function runCompanionPanelId() {
  return "run-companion-panel";
}

export function runCompanionTabLabel(tab: CompanionTab, chunkLabel: string) {
  if (tab === "chunks") return chunkLabel;
  return tab === "evidence" ? "Evidence" : tab === "chat" ? "Chat" : "Runs";
}

export function companionChunkPresentation<T extends { total?: number | null }>(
  summaries: T[],
  running: boolean,
  hasRun: boolean,
) {
  const count = summaries.length;
  const total = summaries.at(-1)?.total ?? null;
  return {
    framed: false,
    running,
    disabled: !hasRun,
    label: count === 0 ? "Chunks" : total && total > 0 ? `Chunks ${count}/${total}` : `Chunks ${count}`,
  };
}

export function clearCompanionRunsFilter(_filter: CompanionRunsFilterState) {
  return runsFilterDefaults();
}
