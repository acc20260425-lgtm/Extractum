import type { AccountRecord, AccountRuntimeStatus } from "$lib/types/accounts";
import type { AnalysisSourceOption } from "$lib/types/analysis";
import type { Source } from "$lib/types/sources";
import type { WorkspaceSelection } from "$lib/analysis-workspace-state";

export interface AnalysisWorkspaceWorkflowState {
  workspaceSelection: WorkspaceSelection;
}

export type AnalysisWorkspaceWorkflowPatch = Partial<{
  accounts: AccountRecord[];
  accountStatuses: Record<number, AccountRuntimeStatus>;
  sourceCatalog: Source[];
  sourceMetrics: Record<number, AnalysisSourceOption>;
  workspaceSelection: WorkspaceSelection;
  loadingSourceCatalog: boolean;
  status: string;
}>;

export interface AnalysisWorkspaceWorkflowDeps {
  getState(): AnalysisWorkspaceWorkflowState;
  patch(patch: AnalysisWorkspaceWorkflowPatch): void;
  listAccounts(): Promise<AccountRecord[]>;
  getAccountStatuses(accountIds: number[]): Promise<AccountRuntimeStatus[]>;
  listSources(accountId: number | null): Promise<Source[]>;
  listAnalysisSources(): Promise<AnalysisSourceOption[]>;
  formatError(action: string, error: unknown): string;
}

function accountStatusesById(statuses: AccountRuntimeStatus[]) {
  return Object.fromEntries(
    statuses.map((runtimeStatus) => [runtimeStatus.account_id, runtimeStatus]),
  );
}

function sourceMetricsById(sources: AnalysisSourceOption[]) {
  return Object.fromEntries(sources.map((source) => [source.id, source]));
}

function fallbackSourceSelection(
  allSources: Source[],
  analysisSources: AnalysisSourceOption[],
): WorkspaceSelection {
  const fallbackSourceId = analysisSources[0]?.id ?? allSources[0]?.id ?? null;
  return fallbackSourceId === null
    ? { kind: "none" }
    : { kind: "source", sourceId: fallbackSourceId };
}

function nextWorkspaceSelection(
  workspaceSelection: WorkspaceSelection,
  allSources: Source[],
  analysisSources: AnalysisSourceOption[],
): WorkspaceSelection {
  if (workspaceSelection.kind !== "source") {
    return workspaceSelection.kind === "none"
      ? fallbackSourceSelection(allSources, analysisSources)
      : workspaceSelection;
  }

  if (
    !allSources.some((source) => source.id === workspaceSelection.sourceId)
  ) {
    return fallbackSourceSelection(allSources, analysisSources);
  }

  return workspaceSelection;
}

export function createAnalysisWorkspaceWorkflow(deps: AnalysisWorkspaceWorkflowDeps) {
  async function loadAccounts() {
    try {
      const accounts = await deps.listAccounts();
      deps.patch({ accounts });
      if (accounts.length === 0) {
        deps.patch({ accountStatuses: {} });
        return;
      }

      const statuses = await deps.getAccountStatuses(accounts.map((account) => account.id));
      deps.patch({ accountStatuses: accountStatusesById(statuses) });
    } catch (error) {
      deps.patch({ status: deps.formatError("loading workspace accounts", error) });
    }
  }

  async function loadSourceCatalog() {
    deps.patch({ loadingSourceCatalog: true });
    try {
      const [allSources, analysisSources] = await Promise.all([
        deps.listSources(null),
        deps.listAnalysisSources(),
      ]);
      deps.patch({
        sourceCatalog: allSources,
        sourceMetrics: sourceMetricsById(analysisSources),
        workspaceSelection: nextWorkspaceSelection(
          deps.getState().workspaceSelection,
          allSources,
          analysisSources,
        ),
      });
    } catch (error) {
      deps.patch({ status: deps.formatError("loading workspace sources", error) });
    } finally {
      deps.patch({ loadingSourceCatalog: false });
    }
  }

  return {
    loadAccounts,
    loadSourceCatalog,
  };
}
