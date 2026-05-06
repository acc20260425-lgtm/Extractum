# Analysis Workspace Loading Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move account/status and analysis source metrics loading out of the `/analysis` route's raw Tauri calls.

**Architecture:** Add a compact `$lib/api/analysis-workspace.ts` wrapper for the three command names, then add a dependency-injected `$lib/analysis-workspace-workflow.ts` that owns account/status and source catalog orchestration. Wire the Svelte route to the workflow while keeping route-local `$state` as the UI composition layer.

**Tech Stack:** Svelte 5, TypeScript, Tauri `invoke`, Vitest.

---

### Task 1: Analysis Workspace API Wrapper

**Files:**
- Create: `src/lib/api/analysis-workspace.ts`
- Create: `src/lib/api/analysis-workspace.test.ts`

- [ ] **Step 1: Write the failing API wrapper tests**

Create `src/lib/api/analysis-workspace.test.ts`:

```ts
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  getWorkspaceAccountStatuses,
  listAnalysisSources,
  listWorkspaceAccounts,
} from "./analysis-workspace";
import type { AccountRuntimeStatus, AccountRecord } from "$lib/types/accounts";
import type { AnalysisSourceOption } from "$lib/types/analysis";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

describe("analysis workspace api wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("loads workspace accounts with the registered command name", async () => {
    const accounts: AccountRecord[] = [{
      id: 1,
      label: "Main",
      api_id: 123,
      phone: "+100",
      created_at: 10,
    }];
    invokeMock.mockResolvedValueOnce(accounts);

    await expect(listWorkspaceAccounts()).resolves.toEqual(accounts);

    expect(invokeMock).toHaveBeenLastCalledWith("list_accounts");
  });

  it("loads account runtime statuses for the given account ids", async () => {
    const statuses: AccountRuntimeStatus[] = [{
      account_id: 1,
      status: "ready",
      message: null,
    }];
    invokeMock.mockResolvedValueOnce(statuses);

    await expect(getWorkspaceAccountStatuses([1, 2])).resolves.toEqual(statuses);

    expect(invokeMock).toHaveBeenLastCalledWith("tg_get_account_statuses", {
      accountIds: [1, 2],
    });
  });

  it("loads analysis source metrics with the registered command name", async () => {
    const sources: AnalysisSourceOption[] = [{
      id: 7,
      account_id: 1,
      title: "Source",
      item_count: 12,
      last_synced_at: 100,
    }];
    invokeMock.mockResolvedValueOnce(sources);

    await expect(listAnalysisSources()).resolves.toEqual(sources);

    expect(invokeMock).toHaveBeenLastCalledWith("list_analysis_sources");
  });
});
```

- [ ] **Step 2: Run the focused API wrapper tests to verify RED**

Run: `npm.cmd test -- src/lib/api/analysis-workspace.test.ts`

Expected: FAIL because `./analysis-workspace` does not exist.

- [ ] **Step 3: Add the minimal API wrapper implementation**

Create `src/lib/api/analysis-workspace.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import type { AccountRecord, AccountRuntimeStatus } from "$lib/types/accounts";
import type { AnalysisSourceOption } from "$lib/types/analysis";

export function listWorkspaceAccounts() {
  return invoke<AccountRecord[]>("list_accounts");
}

export function getWorkspaceAccountStatuses(accountIds: number[]) {
  return invoke<AccountRuntimeStatus[]>("tg_get_account_statuses", { accountIds });
}

export function listAnalysisSources() {
  return invoke<AnalysisSourceOption[]>("list_analysis_sources");
}
```

- [ ] **Step 4: Run the focused API wrapper tests to verify GREEN**

Run: `npm.cmd test -- src/lib/api/analysis-workspace.test.ts`

Expected: PASS.

- [ ] **Step 5: Commit Task 1**

```bash
git add src/lib/api/analysis-workspace.ts src/lib/api/analysis-workspace.test.ts
git commit -m "refactor(analysis): add workspace api wrapper"
```

### Task 2: Analysis Workspace Workflow

**Files:**
- Create: `src/lib/analysis-workspace-workflow.ts`
- Create: `src/lib/analysis-workspace-workflow.test.ts`

- [ ] **Step 1: Write failing workflow tests**

Create tests that exercise these behaviors:

```ts
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  createAnalysisWorkspaceWorkflow,
  type AnalysisWorkspaceWorkflowPatch,
  type AnalysisWorkspaceWorkflowState,
} from "./analysis-workspace-workflow";
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
    accountId: 1,
    kind: "telegram_channel",
    title: "Source",
    username: "source",
    description: null,
    createdAt: 10,
    updatedAt: 20,
    syncEnabled: true,
    lastSyncedAt: null,
    lastSyncStatus: null,
    lastSyncError: null,
    ...overrides,
  };
}

function metric(overrides: Partial<AnalysisSourceOption> = {}): AnalysisSourceOption {
  return {
    id: 7,
    account_id: 1,
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
    selectedSourceId: "",
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
    expect(state.selectedSourceId).toBe("8");
    expect(state.loadingSourceCatalog).toBe(false);
  });

  it("preserves a valid selected source and falls back when stale", async () => {
    const preserved = createHarness({ selectedSourceId: "7" });
    preserved.deps.listSources.mockResolvedValueOnce([source({ id: 7 }), source({ id: 8 })]);
    preserved.deps.listAnalysisSources.mockResolvedValueOnce([metric({ id: 8 })]);

    await preserved.workflow.loadSourceCatalog();

    expect(preserved.state.selectedSourceId).toBe("7");

    const stale = createHarness({ selectedSourceId: "99" });
    stale.deps.listSources.mockResolvedValueOnce([source({ id: 7 })]);
    stale.deps.listAnalysisSources.mockResolvedValueOnce([]);

    await stale.workflow.loadSourceCatalog();

    expect(stale.state.selectedSourceId).toBe("7");
  });

  it("reports source loading errors and clears the loading flag", async () => {
    const { state, deps, workflow } = createHarness();
    deps.listSources.mockRejectedValueOnce("db down");

    await workflow.loadSourceCatalog();

    expect(state.status).toBe("Error loading workspace sources: db down");
    expect(state.loadingSourceCatalog).toBe(false);
  });
});
```

- [ ] **Step 2: Run the focused workflow tests to verify RED**

Run: `npm.cmd test -- src/lib/analysis-workspace-workflow.test.ts`

Expected: FAIL because `./analysis-workspace-workflow` does not exist.

- [ ] **Step 3: Add the minimal workflow implementation**

Create `src/lib/analysis-workspace-workflow.ts` with:

```ts
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
```

- [ ] **Step 4: Run focused workflow tests to verify GREEN**

Run: `npm.cmd test -- src/lib/analysis-workspace-workflow.test.ts`

Expected: PASS.

- [ ] **Step 5: Commit Task 2**

```bash
git add src/lib/analysis-workspace-workflow.ts src/lib/analysis-workspace-workflow.test.ts
git commit -m "refactor(analysis): extract workspace loading workflow"
```

### Task 3: Wire `/analysis` Route To Workspace Workflow

**Files:**
- Modify: `src/routes/analysis/+page.svelte`

- [ ] **Step 1: Write a route cleanup search that currently fails**

Run:

```bash
rg "list_accounts|tg_get_account_statuses|list_analysis_sources" src/routes/analysis/+page.svelte
```

Expected: output still contains all three raw command strings.

- [ ] **Step 2: Wire imports and workflow dependencies**

Modify `src/routes/analysis/+page.svelte`:

```ts
import {
  getWorkspaceAccountStatuses,
  listAnalysisSources,
  listWorkspaceAccounts,
} from "$lib/api/analysis-workspace";
import {
  createAnalysisWorkspaceWorkflow,
  type AnalysisWorkspaceWorkflowPatch,
} from "$lib/analysis-workspace-workflow";
```

Add:

```ts
function applyWorkspaceWorkflowPatch(patch: AnalysisWorkspaceWorkflowPatch) {
  if ("accounts" in patch) accounts = patch.accounts ?? [];
  if ("accountStatuses" in patch) accountStatuses = patch.accountStatuses ?? {};
  if ("sourceCatalog" in patch) sourceCatalog = patch.sourceCatalog ?? [];
  if ("sourceMetrics" in patch) sourceMetrics = patch.sourceMetrics ?? {};
  if ("selectedSourceId" in patch) selectedSourceId = patch.selectedSourceId ?? "";
  if ("loadingSourceCatalog" in patch) loadingSourceCatalog = patch.loadingSourceCatalog ?? false;
  if ("status" in patch && patch.status !== undefined) status = patch.status;
}

const workspaceWorkflow = createAnalysisWorkspaceWorkflow({
  getState: () => ({ selectedSourceId }),
  patch: applyWorkspaceWorkflowPatch,
  listAccounts: listWorkspaceAccounts,
  getAccountStatuses: getWorkspaceAccountStatuses,
  listSources,
  listAnalysisSources,
  formatError: formatAppError,
});
```

- [ ] **Step 3: Replace route-local loading bodies**

Change:

```ts
async function loadAccounts() {
  await workspaceWorkflow.loadAccounts();
}

async function loadSourceCatalog() {
  await workspaceWorkflow.loadSourceCatalog();
}
```

- [ ] **Step 4: Verify the raw command search is clean**

Run:

```bash
rg "list_accounts|tg_get_account_statuses|list_analysis_sources" src/routes/analysis/+page.svelte
```

Expected: no output and exit code 1.

- [ ] **Step 5: Run focused tests**

Run:

```bash
npm.cmd test -- src/lib/api/analysis-workspace.test.ts src/lib/analysis-workspace-workflow.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit Task 3**

```bash
git add src/routes/analysis/+page.svelte
git commit -m "refactor(analysis): use workspace loading workflow"
```

### Task 4: Verification And Review Document Refresh

**Files:**
- Modify: `docs/code-review-results-2026-05-03.md`
- Modify: `docs/session-context-2026-05-03.md`

- [ ] **Step 1: Run full frontend verification**

Run:

```bash
npm.cmd test
npm.cmd run check
git diff --check
```

Expected: all pass.

- [ ] **Step 2: Update the review document**

Remove `list_accounts`, `tg_get_account_statuses`, and `list_analysis_sources`
from the remaining raw command surface in `docs/code-review-results-2026-05-03.md`.
Add a resolved bullet that account/status loading and analysis source metrics
are centralized in `$lib/api/analysis-workspace.ts` and
`$lib/analysis-workspace-workflow.ts`.

- [ ] **Step 3: Update the session handoff**

Update `docs/session-context-2026-05-03.md` so completed workstreams include
Analysis workspace loading extraction and the remaining raw command surface
only lists:

```text
list_analysis_source_groups
start_analysis_report
cancel_analysis_run
delete_analysis_run
delete_analysis_prompt_template
delete_analysis_source_group
```

- [ ] **Step 4: Commit Task 4**

```bash
git add docs/code-review-results-2026-05-03.md docs/session-context-2026-05-03.md
git commit -m "docs(analysis): refresh workspace loading cleanup context"
```
