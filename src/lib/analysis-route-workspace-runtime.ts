import {
  persistableAnalysisWorkspaceState,
  type PersistedAnalysisWorkspaceRunsState,
} from "$lib/analysis-workspace-persistence";
import type { AnalysisWorkspaceUiState } from "$lib/analysis-workspace-state";

export async function restoreWorkspaceBeforeActiveRuns({
  restore,
  loadSourcesAndGroups,
  applyRestoredSelection,
  loadActiveRuns,
}: {
  restore: () => void;
  loadSourcesAndGroups: () => Promise<void>;
  applyRestoredSelection: () => Promise<boolean>;
  loadActiveRuns: () => Promise<void>;
}) {
  restore();
  await loadSourcesAndGroups();
  const restoredSelectionApplied = await applyRestoredSelection();
  await loadActiveRuns();
  return { restoredSelectionApplied };
}

export function persistWorkspaceWhenReady({
  ready,
  state,
  runs,
  save,
}: {
  ready: boolean;
  state: AnalysisWorkspaceUiState;
  runs: PersistedAnalysisWorkspaceRunsState;
  save: (state: ReturnType<typeof persistableAnalysisWorkspaceState>) => void;
}) {
  if (!ready) return null;
  const persisted = persistableAnalysisWorkspaceState(state, runs);
  save(persisted);
  return persisted;
}
