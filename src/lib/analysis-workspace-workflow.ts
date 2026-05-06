import type { AccountRecord, AccountRuntimeStatus } from "$lib/types/accounts";
import type { AnalysisSourceOption } from "$lib/types/analysis";
import type { Source } from "$lib/types/sources";

export interface AnalysisWorkspaceWorkflowState {
  selectedSourceId: string;
}

export type AnalysisWorkspaceWorkflowPatch = Partial<{
  accounts: AccountRecord[];
  accountStatuses: Record<number, AccountRuntimeStatus>;
  sourceCatalog: Source[];
  sourceMetrics: Record<number, AnalysisSourceOption>;
  selectedSourceId: string;
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

function nextSelectedSourceId(
  selectedSourceId: string,
  allSources: Source[],
  analysisSources: AnalysisSourceOption[],
) {
  if (!selectedSourceId && allSources.length > 0) {
    return String(analysisSources[0]?.id ?? allSources[0].id);
  }

  if (
    selectedSourceId &&
    !allSources.some((source) => source.id === Number(selectedSourceId))
  ) {
    return allSources[0] ? String(allSources[0].id) : "";
  }

  return selectedSourceId;
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
        selectedSourceId: nextSelectedSourceId(
          deps.getState().selectedSourceId,
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
