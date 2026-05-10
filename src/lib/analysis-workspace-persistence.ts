import type { AnalysisRunFilter } from "$lib/analysis-state";
import type { AnalysisHistoryScope } from "$lib/analysis-scope-state";
import {
  defaultAnalysisWorkspaceUiState,
  normalizeRestoredWorkspaceState,
  type AnalysisWorkspaceUiState,
  type CanvasMode,
  type CompanionTab,
  type SourceViewBasis,
  type WorkspaceSelection,
} from "$lib/analysis-workspace-state";
import type { AnalysisSourceGroup } from "$lib/types/analysis";
import type { Source } from "$lib/types/sources";

export const ANALYSIS_WORKSPACE_STATE_KEY = "extractum.analysis.workspace.v1";

export interface PersistedAnalysisWorkspaceRunsState {
  historyScope: AnalysisHistoryScope;
  runFilter: AnalysisRunFilter;
}

export interface PersistedAnalysisWorkspaceState {
  version: 1;
  workspaceSelection: WorkspaceSelection;
  canvasMode: CanvasMode;
  sourceViewBasis: SourceViewBasis;
  companionTab: CompanionTab;
  runs: PersistedAnalysisWorkspaceRunsState;
}

export interface StorageLike {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
  removeItem(key: string): void;
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isPositiveInteger(value: unknown): value is number {
  return typeof value === "number" && Number.isInteger(value) && value > 0;
}

function parseWorkspaceSelection(value: unknown): WorkspaceSelection | null {
  if (!isObject(value) || typeof value.kind !== "string") {
    return null;
  }

  if (value.kind === "none") {
    return { kind: "none" };
  }

  if (value.kind === "source" && isPositiveInteger(value.sourceId)) {
    return { kind: "source", sourceId: value.sourceId };
  }

  if (value.kind === "source_group" && isPositiveInteger(value.sourceGroupId)) {
    return { kind: "source_group", sourceGroupId: value.sourceGroupId };
  }

  return null;
}

function parseCanvasMode(value: unknown): CanvasMode | null {
  return value === "report" || value === "source" ? value : null;
}

function parseSourceViewBasis(value: unknown): SourceViewBasis | null {
  return value === "live_source" || value === "run_snapshot" ? value : null;
}

function parseCompanionTab(value: unknown): CompanionTab | null {
  return value === "evidence" || value === "chat" || value === "runs" ? value : null;
}

function parseHistoryScope(value: unknown): AnalysisHistoryScope | null {
  return value === "all" || value === "current" ? value : null;
}

function parseRunFilter(value: unknown): AnalysisRunFilter | null {
  return value === "all" || value === "completed" || value === "failed" ? value : null;
}

export function persistableAnalysisWorkspaceState(
  uiState: AnalysisWorkspaceUiState,
  runs: PersistedAnalysisWorkspaceRunsState,
): PersistedAnalysisWorkspaceState {
  return {
    version: 1,
    workspaceSelection: uiState.workspaceSelection,
    canvasMode: uiState.canvasMode,
    sourceViewBasis: uiState.sourceViewBasis,
    companionTab: uiState.companionTab,
    runs,
  };
}

export function parsePersistedAnalysisWorkspaceState(
  raw: string | null,
): PersistedAnalysisWorkspaceState | null {
  if (!raw) {
    return null;
  }

  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch {
    return null;
  }

  if (!isObject(parsed) || parsed.version !== 1) {
    return null;
  }

  const workspaceSelection = parseWorkspaceSelection(parsed.workspaceSelection);
  const canvasMode = parseCanvasMode(parsed.canvasMode);
  const sourceViewBasis = parseSourceViewBasis(parsed.sourceViewBasis);
  const companionTab = parseCompanionTab(parsed.companionTab);
  const runs = isObject(parsed.runs) ? parsed.runs : null;
  const historyScope = runs ? parseHistoryScope(runs.historyScope) : null;
  const runFilter = runs ? parseRunFilter(runs.runFilter) : null;

  if (
    !workspaceSelection ||
    !canvasMode ||
    !sourceViewBasis ||
    !companionTab ||
    !historyScope ||
    !runFilter
  ) {
    return null;
  }

  return {
    version: 1,
    workspaceSelection,
    canvasMode,
    sourceViewBasis,
    companionTab,
    runs: {
      historyScope,
      runFilter,
    },
  };
}

export function loadPersistedAnalysisWorkspaceState(
  storage: StorageLike,
): PersistedAnalysisWorkspaceState | null {
  return parsePersistedAnalysisWorkspaceState(
    storage.getItem(ANALYSIS_WORKSPACE_STATE_KEY),
  );
}

export function savePersistedAnalysisWorkspaceState(
  storage: StorageLike,
  state: PersistedAnalysisWorkspaceState,
) {
  storage.setItem(ANALYSIS_WORKSPACE_STATE_KEY, JSON.stringify(state));
}

export function clearPersistedAnalysisWorkspaceState(storage: StorageLike) {
  storage.removeItem(ANALYSIS_WORKSPACE_STATE_KEY);
}

export function restoredUiStateFromPersisted(
  persisted: PersistedAnalysisWorkspaceState,
): AnalysisWorkspaceUiState {
  return normalizeRestoredWorkspaceState({
    ...defaultAnalysisWorkspaceUiState(),
    workspaceSelection: persisted.workspaceSelection,
    openRunState: { kind: "none" },
    canvasMode: persisted.canvasMode,
    sourceViewBasis: persisted.sourceViewBasis,
    companionTab: persisted.companionTab,
    selectedTraceRef: null,
  });
}

export function fallbackWorkspaceSelection(
  preferred: WorkspaceSelection,
  sources: Source[],
  groups: AnalysisSourceGroup[],
): WorkspaceSelection {
  if (
    preferred.kind === "source" &&
    sources.some((source) => source.id === preferred.sourceId)
  ) {
    return preferred;
  }

  if (
    preferred.kind === "source_group" &&
    groups.some((group) => group.id === preferred.sourceGroupId)
  ) {
    return preferred;
  }

  if (preferred.kind === "none" && sources.length === 0 && groups.length === 0) {
    return preferred;
  }

  if (preferred.kind === "source_group" && groups.length > 0) {
    return { kind: "source_group", sourceGroupId: groups[0].id };
  }

  if (preferred.kind === "source" && sources.length > 0) {
    return { kind: "source", sourceId: sources[0].id };
  }

  if (preferred.kind === "none" && sources.length > 0) {
    return { kind: "source", sourceId: sources[0].id };
  }

  if (groups.length > 0) {
    return { kind: "source_group", sourceGroupId: groups[0].id };
  }

  return { kind: "none" };
}
