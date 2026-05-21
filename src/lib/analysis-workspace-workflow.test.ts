import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  createAnalysisWorkspaceWorkflow,
  type AnalysisWorkspaceWorkflowPatch,
  type AnalysisWorkspaceWorkflowState,
} from "./analysis-workspace-workflow";
import type { WorkspaceSelection } from "./analysis-workspace-state";
import type { AccountRecord, AccountRuntimeStatus } from "./types/accounts";
import type { AnalysisSourceOption } from "./types/analysis";
import type { Source } from "./types/sources";

function account(overrides: Partial<AccountRecord> = {}): AccountRecord {
  return {
    id: 1,
    label: "Main",
    api_id: 123,
    phone: "+100",
    created_at: 10,
    ...overrides,
  };
}

function runtimeStatus(overrides: Partial<AccountRuntimeStatus> = {}): AccountRuntimeStatus {
  return {
    account_id: 1,
    status: "ready",
    message: null,
    ...overrides,
  };
}

function source(overrides: Partial<Source> = {}): Source {
  return {
    id: 7,
    sourceType: "telegram",
    sourceSubtype: "channel",
    accountId: 1,
    externalId: "source-7",
    title: "Source",
    lastSyncState: null,
    lastSyncedAt: null,
    isMember: true,
    isActive: true,
    createdAt: 10,
    telegramUsername: null,
    avatarDataUrl: null,
    ...overrides,
  };
}

function metric(overrides: Partial<AnalysisSourceOption> = {}): AnalysisSourceOption {
  return {
    id: 7,
    account_id: 1,
    source_type: "telegram",
    title: "Source",
    item_count: 12,
    last_synced_at: 100,
    ...overrides,
  };
}

type HarnessState = AnalysisWorkspaceWorkflowState & {
  accounts: AccountRecord[];
  accountStatuses: Record<number, AccountRuntimeStatus>;
  sourceCatalog: Source[];
  sourceMetrics: Record<number, AnalysisSourceOption>;
  loadingSourceCatalog: boolean;
  status: string;
};

function createHarness(initial: Partial<HarnessState> = {}) {
  const state: HarnessState = {
    workspaceSelection: { kind: "none" },
    accounts: [],
    accountStatuses: {},
    sourceCatalog: [],
    sourceMetrics: {},
    loadingSourceCatalog: false,
    status: "",
    ...initial,
  };

  const deps = {
    getState: () => state,
    patch: vi.fn((patch: AnalysisWorkspaceWorkflowPatch) => Object.assign(state, patch)),
    listAccounts: vi.fn(),
    getAccountStatuses: vi.fn(),
    listSources: vi.fn(),
    listAnalysisSources: vi.fn(),
    formatError: vi.fn((action: string, error: unknown) => `Error ${action}: ${String(error)}`),
  };

  return { state, deps, workflow: createAnalysisWorkspaceWorkflow(deps) };
}

describe("analysis-workspace-workflow", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("loads accounts and maps runtime statuses by account id", async () => {
    const { state, deps, workflow } = createHarness();
    deps.listAccounts.mockResolvedValueOnce([account({ id: 1 }), account({ id: 2 })]);
    deps.getAccountStatuses.mockResolvedValueOnce([
      runtimeStatus({ account_id: 1 }),
      runtimeStatus({ account_id: 2, status: "restoring" }),
    ]);

    await workflow.loadAccounts();

    expect(deps.getAccountStatuses).toHaveBeenCalledWith([1, 2]);
    expect(state.accounts.map((entry) => entry.id)).toEqual([1, 2]);
    expect(state.accountStatuses[2]?.status).toBe("restoring");
  });

  it("clears account statuses when there are no accounts", async () => {
    const { state, deps, workflow } = createHarness({
      accountStatuses: { 1: runtimeStatus() },
    });
    deps.listAccounts.mockResolvedValueOnce([]);

    await workflow.loadAccounts();

    expect(deps.getAccountStatuses).not.toHaveBeenCalled();
    expect(state.accounts).toEqual([]);
    expect(state.accountStatuses).toEqual({});
  });

  it("reports account loading errors", async () => {
    const { state, deps, workflow } = createHarness();
    deps.listAccounts.mockRejectedValueOnce("backend down");

    await workflow.loadAccounts();

    expect(state.status).toBe("Error loading workspace accounts: backend down");
  });

  it("loads source catalog and analysis metrics while selecting the first analysis source", async () => {
    const { state, deps, workflow } = createHarness();
    deps.listSources.mockResolvedValueOnce([source({ id: 7 }), source({ id: 8 })]);
    deps.listAnalysisSources.mockResolvedValueOnce([metric({ id: 8 })]);

    await workflow.loadSourceCatalog();

    expect(deps.listSources).toHaveBeenCalledWith(null);
    expect(state.sourceCatalog.map((entry) => entry.id)).toEqual([7, 8]);
    expect(state.sourceMetrics[8]?.item_count).toBe(12);
    expect(state.workspaceSelection).toEqual({ kind: "source", sourceId: 8 });
    expect(state.loadingSourceCatalog).toBe(false);
  });

  it("preserves a valid selected source and falls back when stale", async () => {
    const preserved = createHarness({
      workspaceSelection: { kind: "source", sourceId: 7 },
    });
    preserved.deps.listSources.mockResolvedValueOnce([source({ id: 7 }), source({ id: 8 })]);
    preserved.deps.listAnalysisSources.mockResolvedValueOnce([metric({ id: 8 })]);

    await preserved.workflow.loadSourceCatalog();

    expect(preserved.state.workspaceSelection).toEqual({ kind: "source", sourceId: 7 });

    const stale = createHarness({
      workspaceSelection: { kind: "source", sourceId: 99 },
    });
    stale.deps.listSources.mockResolvedValueOnce([source({ id: 7 })]);
    stale.deps.listAnalysisSources.mockResolvedValueOnce([]);

    await stale.workflow.loadSourceCatalog();

    expect(stale.state.workspaceSelection).toEqual({ kind: "source", sourceId: 7 });
  });

  it("preserves a selected source group when source catalog refreshes", async () => {
    const workspaceSelection: WorkspaceSelection = { kind: "source_group", sourceGroupId: 12 };
    const { state, deps, workflow } = createHarness({ workspaceSelection });
    deps.listSources.mockResolvedValueOnce([source({ id: 7 })]);
    deps.listAnalysisSources.mockResolvedValueOnce([metric({ id: 7 })]);

    await workflow.loadSourceCatalog();

    expect(state.workspaceSelection).toEqual(workspaceSelection);
  });

  it("reports source loading errors and clears the loading flag", async () => {
    const { state, deps, workflow } = createHarness();
    deps.listSources.mockRejectedValueOnce("db down");

    await workflow.loadSourceCatalog();

    expect(state.status).toBe("Error loading workspace sources: db down");
    expect(state.loadingSourceCatalog).toBe(false);
  });
});
