# Analysis Result-First Redesign Part 2 Workspace State Persistence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire the Part 1 workspace state contract into the current `/analysis` route and persist restorable workspace context without reopening runs automatically.

**Architecture:** Add a focused persistence module for versioned `/analysis` UI state, then connect it to the existing route through pure helpers and workflow callbacks. The current `WorkspaceRail`, `WorkspaceMain`, and `WorkspaceInspector` stay visually intact; this part prepares stable state for later `CompactSourceRail`, `ReportCanvas`, and `RunCompanionTabs` parts.

**Tech Stack:** SvelteKit 2, Svelte 5 runes, Vitest raw-source tests, browser `localStorage`, existing `/analysis` workflow modules, Part 1 `analysis-workspace-state` contract.

---

## Prerequisites

Implement this part only after Part 1 is complete and committed. This plan assumes these Part 1 files exist:

- `src/lib/analysis-workspace-state.ts`
- `src/lib/analysis-workspace-state.test.ts`

This is **Part 2 of 7**. Stop after this part is implemented, verified, and committed. Continue to Part 3 only after explicit user approval.

## Part Boundary

Part 2 may:

- persist last `WorkspaceSelection`;
- persist `canvasMode`, `sourceViewBasis`, and `companionTab`;
- persist existing durable run-history preferences: `historyScope` and `runFilter`;
- normalize restored run-bound state so no run is reopened;
- make source/group selection clear opened-run state through the Part 1 contract;
- make run opening align workspace selection through the Part 1 contract;
- add route-level raw-source tests for persistence wiring.

Part 2 must not:

- create `CompactSourceRail`;
- create `ReportCanvas`;
- create `RunCompanionTabs`;
- move source readers;
- add visible `Report | Source` switching;
- add run search UI;
- persist `OpenRunState`;
- persist selected trace refs, draft chat text, popovers, drawers, or scroll positions.

## File Structure

- Create: `src/lib/analysis-workspace-persistence.ts`
  - Responsibility: serialize, parse, normalize, save, load, and validate versioned restorable `/analysis` workspace state.
- Create: `src/lib/analysis-workspace-persistence.test.ts`
  - Responsibility: verify persistence shape, defensive parsing, run-bound normalization, stale selection fallback, and exclusion of transient state.
- Modify: `src/lib/analysis-run-workflow.ts`
  - Responsibility: notify the route when a run detail has been opened so route-level workspace state can align for manual and automatic open flows.
- Modify: `src/lib/analysis-run-workflow.test.ts`
  - Responsibility: verify `onRunOpened` fires for current run loads and not for stale or missing loads.
- Create: `src/lib/analysis-route-workspace-state.test.ts`
  - Responsibility: raw-source coverage that `/analysis/+page.svelte` imports persistence/state helpers, restores from `localStorage`, saves only durable state, and does not persist run/transient state.
- Modify: `src/routes/analysis/+page.svelte`
  - Responsibility: own `workspaceUiState`, restore persisted workspace context on mount, save durable state after restore, use Part 1 transitions for source/group/run selection, and preserve current visible layout.

## Task 1: Add Workspace Persistence Helpers

**Files:**
- Create: `src/lib/analysis-workspace-persistence.test.ts`
- Create: `src/lib/analysis-workspace-persistence.ts`

- [ ] **Step 1: Write the failing persistence tests**

Create `src/lib/analysis-workspace-persistence.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  ANALYSIS_WORKSPACE_STATE_KEY,
  fallbackWorkspaceSelection,
  loadPersistedAnalysisWorkspaceState,
  persistableAnalysisWorkspaceState,
  restoredUiStateFromPersisted,
  savePersistedAnalysisWorkspaceState,
} from "./analysis-workspace-persistence";
import {
  defaultAnalysisWorkspaceUiState,
  type AnalysisWorkspaceUiState,
  type WorkspaceSelection,
} from "./analysis-workspace-state";
import type { AnalysisSourceGroup } from "./types/analysis";
import type { Source } from "./types/sources";

class MemoryStorage {
  values = new Map<string, string>();

  getItem(key: string) {
    return this.values.get(key) ?? null;
  }

  setItem(key: string, value: string) {
    this.values.set(key, value);
  }

  removeItem(key: string) {
    this.values.delete(key);
  }
}

function source(id: number): Source {
  return {
    id,
    sourceType: "telegram",
    sourceSubtype: "channel",
    telegramSourceKind: "channel",
    accountId: 1,
    externalId: `source-${id}`,
    title: `Source ${id}`,
    lastSyncState: null,
    lastSyncedAt: null,
    isMember: true,
    isActive: true,
    createdAt: 100,
    avatarDataUrl: null,
  };
}

function group(id: number): AnalysisSourceGroup {
  return {
    id,
    name: `Group ${id}`,
    source_type: "telegram",
    members: [],
    created_at: 100,
    updated_at: 100,
  };
}

function uiState(overrides: Partial<AnalysisWorkspaceUiState> = {}): AnalysisWorkspaceUiState {
  return {
    ...defaultAnalysisWorkspaceUiState(),
    ...overrides,
  };
}

describe("analysis-workspace-persistence", () => {
  it("serializes only durable workspace state", () => {
    const persisted = persistableAnalysisWorkspaceState(
      uiState({
        workspaceSelection: { kind: "source", sourceId: 7 },
        openRunState: { kind: "saved", runId: 42 },
        canvasMode: "report",
        sourceViewBasis: "run_snapshot",
        companionTab: "evidence",
        selectedTraceRef: "s7-i1",
      }),
      {
        historyScope: "current",
        runFilter: "completed",
      },
    );

    expect(persisted).toEqual({
      version: 1,
      workspaceSelection: { kind: "source", sourceId: 7 },
      canvasMode: "report",
      sourceViewBasis: "run_snapshot",
      companionTab: "evidence",
      runs: {
        historyScope: "current",
        runFilter: "completed",
      },
    });
    expect(JSON.stringify(persisted)).not.toContain("openRunState");
    expect(JSON.stringify(persisted)).not.toContain("selectedTraceRef");
    expect(JSON.stringify(persisted)).not.toContain("runId");
  });

  it("saves and loads persisted workspace state through the configured key", () => {
    const storage = new MemoryStorage();
    const persisted = persistableAnalysisWorkspaceState(
      uiState({
        workspaceSelection: { kind: "source_group", sourceGroupId: 9 },
        canvasMode: "source",
        sourceViewBasis: "live_source",
        companionTab: "runs",
      }),
      {
        historyScope: "all",
        runFilter: "failed",
      },
    );

    savePersistedAnalysisWorkspaceState(storage, persisted);

    expect(storage.getItem(ANALYSIS_WORKSPACE_STATE_KEY)).toContain('"version":1');
    expect(loadPersistedAnalysisWorkspaceState(storage)).toEqual(persisted);
  });

  it("rejects malformed, unsupported, or invalid persisted state", () => {
    const storage = new MemoryStorage();

    expect(loadPersistedAnalysisWorkspaceState(storage)).toBeNull();

    storage.setItem(ANALYSIS_WORKSPACE_STATE_KEY, "not json");
    expect(loadPersistedAnalysisWorkspaceState(storage)).toBeNull();

    storage.setItem(ANALYSIS_WORKSPACE_STATE_KEY, JSON.stringify({ version: 2 }));
    expect(loadPersistedAnalysisWorkspaceState(storage)).toBeNull();

    storage.setItem(ANALYSIS_WORKSPACE_STATE_KEY, JSON.stringify({
      version: 1,
      workspaceSelection: { kind: "source", sourceId: -1 },
      canvasMode: "report",
      sourceViewBasis: "live_source",
      companionTab: "runs",
      runs: { historyScope: "all", runFilter: "all" },
    }));
    expect(loadPersistedAnalysisWorkspaceState(storage)).toBeNull();
  });

  it("normalizes restored UI state because OpenRunState is never persisted", () => {
    const restored = restoredUiStateFromPersisted({
      version: 1,
      workspaceSelection: { kind: "source", sourceId: 7 },
      canvasMode: "report",
      sourceViewBasis: "run_snapshot",
      companionTab: "chat",
      runs: {
        historyScope: "current",
        runFilter: "completed",
      },
    });

    expect(restored).toEqual({
      workspaceSelection: { kind: "source", sourceId: 7 },
      openRunState: { kind: "none" },
      canvasMode: "report",
      sourceViewBasis: "live_source",
      companionTab: "runs",
      selectedTraceRef: null,
    });
  });

  it("falls back gracefully when persisted source or group ids are stale", () => {
    expect(fallbackWorkspaceSelection(
      { kind: "source", sourceId: 99 },
      [source(7), source(8)],
      [group(9)],
    )).toEqual({ kind: "source", sourceId: 7 });

    expect(fallbackWorkspaceSelection(
      { kind: "source_group", sourceGroupId: 99 },
      [source(7)],
      [group(9)],
    )).toEqual({ kind: "source_group", sourceGroupId: 9 });

    expect(fallbackWorkspaceSelection(
      { kind: "none" },
      [source(7)],
      [group(9)],
    )).toEqual({ kind: "source", sourceId: 7 });

    expect(fallbackWorkspaceSelection(
      { kind: "source", sourceId: 99 },
      [],
      [],
    )).toEqual({ kind: "none" });
  });

  it("preserves valid persisted source and group selections", () => {
    const sourceSelection: WorkspaceSelection = { kind: "source", sourceId: 8 };
    const groupSelection: WorkspaceSelection = { kind: "source_group", sourceGroupId: 9 };

    expect(fallbackWorkspaceSelection(sourceSelection, [source(7), source(8)], [group(9)]))
      .toEqual(sourceSelection);
    expect(fallbackWorkspaceSelection(groupSelection, [source(7)], [group(9)]))
      .toEqual(groupSelection);
  });
});
```

- [ ] **Step 2: Run the persistence tests and verify they fail**

Run:

```powershell
npm.cmd test -- src/lib/analysis-workspace-persistence.test.ts
```

Expected: FAIL because `src/lib/analysis-workspace-persistence.ts` does not exist.

- [ ] **Step 3: Add the persistence module**

Create `src/lib/analysis-workspace-persistence.ts`:

```ts
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

  if (!workspaceSelection || !canvasMode || !sourceViewBasis || !companionTab || !historyScope || !runFilter) {
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
```

- [ ] **Step 4: Run the persistence tests and verify they pass**

Run:

```powershell
npm.cmd test -- src/lib/analysis-workspace-persistence.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit the persistence helper**

Run:

```powershell
git add src/lib/analysis-workspace-persistence.ts src/lib/analysis-workspace-persistence.test.ts
git commit -m "feat: add analysis workspace persistence helpers"
```

## Task 2: Notify Route State When Runs Open

**Files:**
- Modify: `src/lib/analysis-run-workflow.ts`
- Modify: `src/lib/analysis-run-workflow.test.ts`

- [ ] **Step 1: Write failing workflow callback tests**

In `src/lib/analysis-run-workflow.test.ts`, add `onRunOpened: vi.fn()` to the `deps` object in `createHarness()`:

```ts
    onRunOpened: vi.fn(),
```

Then add these tests after the existing `"opens a run by loading detail, chat, and trace data"` test:

```ts
  it("notifies route workspace state after a current run opens", async () => {
    const { deps, workflow } = createHarness();
    const run = runDetail({ id: 7, status: "completed" });
    deps.getRun.mockResolvedValueOnce(run);
    deps.loadChatMessages.mockResolvedValueOnce(undefined);

    await workflow.openRun(7);

    expect(deps.onRunOpened).toHaveBeenCalledWith(run);
  });

  it("does not notify route workspace state for a missing run", async () => {
    const { deps, workflow } = createHarness();
    deps.getRun.mockResolvedValueOnce(null);

    await workflow.openRun(7);

    expect(deps.onRunOpened).not.toHaveBeenCalled();
  });

  it("does not notify route workspace state for a stale open run response", async () => {
    const { deps, workflow } = createHarness();
    let resolveFirst: (run: AnalysisRunDetail) => void = () => {};
    deps.getRun
      .mockImplementationOnce(() => new Promise<AnalysisRunDetail>((resolve) => {
        resolveFirst = resolve;
      }))
      .mockResolvedValueOnce(runDetail({ id: 8, result_markdown: "second" }));
    deps.loadChatMessages.mockResolvedValue(undefined);

    const first = workflow.openRun(7);
    const second = workflow.openRun(8);
    resolveFirst(runDetail({ id: 7, result_markdown: "first" }));
    await Promise.all([first, second]);

    expect(deps.onRunOpened).toHaveBeenCalledTimes(1);
    expect(deps.onRunOpened).toHaveBeenCalledWith(expect.objectContaining({ id: 8 }));
  });
```

Update the existing `runSummary()` test fixture with the Part 1 field if it is not already present:

```ts
    youtube_corpus_mode: "transcript_description",
```

- [ ] **Step 2: Run the workflow tests and verify they fail**

Run:

```powershell
npm.cmd test -- src/lib/analysis-run-workflow.test.ts
```

Expected: FAIL because `AnalysisRunWorkflowDeps` has no `onRunOpened` callback and `openRun()` does not call it.

- [ ] **Step 3: Add the workflow callback**

In `src/lib/analysis-run-workflow.ts`, add this property to `AnalysisRunWorkflowDeps`:

```ts
  onRunOpened(run: AnalysisRunDetail): void;
```

In `openRun()`, immediately after:

```ts
      deps.patch({ currentRun: run });
```

add:

```ts
      deps.onRunOpened(run);
```

The callback must stay after the stale guard and after the not-found branch so only the current loaded run updates route workspace state.

- [ ] **Step 4: Run the workflow tests and verify they pass**

Run:

```powershell
npm.cmd test -- src/lib/analysis-run-workflow.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit the workflow callback**

Run:

```powershell
git add src/lib/analysis-run-workflow.ts src/lib/analysis-run-workflow.test.ts
git commit -m "feat: notify analysis workspace when runs open"
```

## Task 3: Wire Workspace State And Persistence Into `/analysis`

**Files:**
- Create: `src/lib/analysis-route-workspace-state.test.ts`
- Modify: `src/routes/analysis/+page.svelte`

- [ ] **Step 1: Add failing route raw-source tests**

Create `src/lib/analysis-route-workspace-state.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";

function functionSlice(name: string, nextName: string) {
  const start = analysisPageSource.indexOf(`  ${name}`);
  const end = analysisPageSource.indexOf(`\n  ${nextName}`, start + 1);

  expect(start, `${name} should exist`).toBeGreaterThan(-1);
  expect(end, `${nextName} should follow ${name}`).toBeGreaterThan(start);

  return analysisPageSource.slice(start, end);
}

describe("analysis route workspace state", () => {
  it("imports workspace state and persistence helpers", () => {
    expect(analysisPageSource).toContain("defaultAnalysisWorkspaceUiState");
    expect(analysisPageSource).toContain("openRunWorkspaceState");
    expect(analysisPageSource).toContain("selectSourceWorkspace");
    expect(analysisPageSource).toContain("selectSourceGroupWorkspace");
    expect(analysisPageSource).toContain("loadPersistedAnalysisWorkspaceState");
    expect(analysisPageSource).toContain("savePersistedAnalysisWorkspaceState");
    expect(analysisPageSource).toContain("fallbackWorkspaceSelection");
  });

  it("owns result-first workspace state without rendering the new layout yet", () => {
    expect(analysisPageSource).toContain("let workspaceUiState = $state<AnalysisWorkspaceUiState>(");
    expect(analysisPageSource).toContain("let workspacePersistenceReady = $state(false);");
    expect(analysisPageSource).toContain("let restoredWorkspaceSelection = $state<WorkspaceSelection | null>(null);");
    expect(analysisPageSource).toContain("defaultAnalysisWorkspaceUiState()");
  });

  it("restores persisted workspace state before loading active runs", () => {
    const mount = analysisPageSource.slice(
      analysisPageSource.indexOf("  onMount(() => {"),
      analysisPageSource.indexOf("</script>"),
    );

    expect(mount).toContain("restorePersistedWorkspaceState();");
    expect(mount).toContain("await Promise.all([loadSourceCatalog(), loadGroups()]);");
    expect(mount).toContain("await applyRestoredWorkspaceSelection();");
    expect(mount.indexOf("restorePersistedWorkspaceState();"))
      .toBeLessThan(mount.indexOf("await Promise.all([loadSourceCatalog(), loadGroups()]);"));
    expect(mount.indexOf("await applyRestoredWorkspaceSelection();"))
      .toBeLessThan(mount.indexOf("void loadActiveRuns();"));
  });

  it("persists durable workspace state and excludes run-bound transient state", () => {
    const saveFunction = functionSlice(
      "function persistWorkspaceState()",
      "function applyWorkspaceUiState",
    );

    expect(saveFunction).toContain("savePersistedAnalysisWorkspaceState(window.localStorage");
    expect(saveFunction).toContain("persistableAnalysisWorkspaceState(workspaceUiState");
    expect(saveFunction).toContain("historyScope");
    expect(saveFunction).toContain("runFilter");
    expect(saveFunction).not.toContain("currentRun");
    expect(saveFunction).not.toContain("activeRunId");
    expect(saveFunction).not.toContain("selectedTraceRef");
    expect(saveFunction).not.toContain("chatQuestion");
    expect(saveFunction).not.toContain("sourceManagerOpen");
  });

  it("uses workspace transition helpers for source, group, and run opening", () => {
    const sourceFunction = functionSlice(
      "async function selectSource",
      "function selectGroup",
    );
    const groupFunction = functionSlice(
      "function selectGroup",
      "async function changeSelectedTopicKey",
    );
    const runFunction = functionSlice(
      "function alignWorkspaceToOpenedRun",
      "async function loadChatMessages",
    );

    expect(sourceFunction).toContain("selectSourceWorkspace(workspaceUiState, sourceId)");
    expect(sourceFunction).toContain("clearCurrentRunForWorkspaceSwitch();");
    expect(groupFunction).toContain("selectSourceGroupWorkspace(workspaceUiState, groupId)");
    expect(groupFunction).toContain("clearCurrentRunForWorkspaceSwitch();");
    expect(runFunction).toContain("openRunWorkspaceState(workspaceUiState");
    expect(runFunction).toContain("legacyScopeFromWorkspaceSelection");
  });

  it("saves workspace state from a guarded effect after restore is complete", () => {
    expect(analysisPageSource).toContain("if (!workspacePersistenceReady) {");
    expect(analysisPageSource).toContain("persistWorkspaceState();");
  });
});
```

- [ ] **Step 2: Run the route raw-source test and verify it fails**

Run:

```powershell
npm.cmd test -- src/lib/analysis-route-workspace-state.test.ts
```

Expected: FAIL because `/analysis/+page.svelte` has not been wired yet.

- [ ] **Step 3: Import workspace state helpers**

In `src/routes/analysis/+page.svelte`, add these imports near the existing analysis workflow imports:

```ts
  import {
    defaultAnalysisWorkspaceUiState,
    legacyScopeFromWorkspaceSelection,
    openRunWorkspaceState,
    selectSourceGroupWorkspace,
    selectSourceWorkspace,
    type AnalysisWorkspaceUiState,
    type WorkspaceSelection,
  } from "$lib/analysis-workspace-state";
  import {
    fallbackWorkspaceSelection,
    loadPersistedAnalysisWorkspaceState,
    persistableAnalysisWorkspaceState,
    restoredUiStateFromPersisted,
    savePersistedAnalysisWorkspaceState,
  } from "$lib/analysis-workspace-persistence";
```

- [ ] **Step 4: Add route-owned workspace state**

After the existing `inspectorMode` state, add:

```ts
  let workspaceUiState = $state<AnalysisWorkspaceUiState>(
    defaultAnalysisWorkspaceUiState(),
  );
  let workspacePersistenceReady = $state(false);
  let restoredWorkspaceSelection = $state<WorkspaceSelection | null>(null);
```

- [ ] **Step 5: Add route helpers for persistence and workspace transitions**

Add these helpers before `applyRunWorkflowPatch()`:

```ts
  function restorePersistedWorkspaceState() {
    if (typeof window === "undefined") {
      return;
    }

    const persisted = loadPersistedAnalysisWorkspaceState(window.localStorage);
    if (!persisted) {
      workspacePersistenceReady = true;
      return;
    }

    const restored = restoredUiStateFromPersisted(persisted);
    workspaceUiState = restored;
    restoredWorkspaceSelection = restored.workspaceSelection;
    historyScope = persisted.runs.historyScope;
    runFilter = persisted.runs.runFilter;
    workspacePersistenceReady = true;
  }

  function persistWorkspaceState() {
    if (typeof window === "undefined" || !workspacePersistenceReady) {
      return;
    }

    savePersistedAnalysisWorkspaceState(
      window.localStorage,
      persistableAnalysisWorkspaceState(workspaceUiState, {
        historyScope,
        runFilter,
      }),
    );
  }

  function applyWorkspaceUiState(next: AnalysisWorkspaceUiState) {
    workspaceUiState = next;
  }

  function clearCurrentRunForWorkspaceSwitch() {
    if (activeRunId !== null || currentRun !== null) {
      runWorkflow.invalidateOpenRunRequests();
    }

    activeRunId = null;
    currentRun = null;
    traceData = { refs: [] };
    savedTraceRefs = [];
    resolvedTraceRefs = [];
    selectedTraceRef = null;
    chatMessages = [];
    chatQuestion = "";
    chatting = false;
    activeChatRequestId = null;
    activeChatRunId = null;
  }

  function liveScopeExistsForRun(run: AnalysisRunDetail) {
    if (run.source_id !== null) {
      return sourceCatalog.some((source) => source.id === run.source_id);
    }

    if (run.source_group_id !== null) {
      return groups.some((group) => group.id === run.source_group_id);
    }

    return false;
  }

  function alignWorkspaceToOpenedRun(run: AnalysisRunDetail) {
    const next = openRunWorkspaceState(workspaceUiState, {
      runId: run.id,
      status: run.status,
      sourceId: run.source_id,
      sourceGroupId: run.source_group_id,
      liveScopeExists: liveScopeExistsForRun(run),
    });

    applyWorkspaceUiState(next);

    const legacy = legacyScopeFromWorkspaceSelection(next.workspaceSelection);
    analysisScope = legacy.analysisScope;
    selectedSourceId = legacy.selectedSourceId;
    selectedGroupId = legacy.selectedGroupId;
  }

  async function applyRestoredWorkspaceSelection() {
    if (!restoredWorkspaceSelection) {
      return false;
    }

    const selection = fallbackWorkspaceSelection(
      restoredWorkspaceSelection,
      sourceCatalog,
      groups,
    );
    restoredWorkspaceSelection = null;

    if (selection.kind === "source") {
      await selectSource(selection.sourceId, { preserveRestoredCanvasState: true });
      return true;
    }

    if (selection.kind === "source_group") {
      selectGroup(selection.sourceGroupId, { preserveRestoredCanvasState: true });
      return true;
    }

    applyWorkspaceUiState({
      ...workspaceUiState,
      workspaceSelection: { kind: "none" },
    });
    return false;
  }
```

- [ ] **Step 6: Wire `onRunOpened` in the run workflow**

In the `createAnalysisRunWorkflow({ ... })` dependency object, add:

```ts
    onRunOpened: alignWorkspaceToOpenedRun,
```

- [ ] **Step 7: Update source and group selection to use transition helpers**

Change `selectSource` signature:

```ts
  async function selectSource(
    sourceId: number,
    { preserveRestoredCanvasState = false }: { preserveRestoredCanvasState?: boolean } = {},
  ) {
```

At the start of `selectSource`, add:

```ts
    const previousWorkspaceState = workspaceUiState;
    applyWorkspaceUiState(selectSourceWorkspace(workspaceUiState, sourceId));
    historyScope = "current";
    if (activeChatRequestId !== null) {
      void cancelChat({ silent: true });
    }
    clearCurrentRunForWorkspaceSwitch();
```

At the end of `selectSource`, after the `await Promise.all([...])` block, add:

```ts
    if (preserveRestoredCanvasState) {
      applyWorkspaceUiState({
        ...workspaceUiState,
        canvasMode: previousWorkspaceState.canvasMode,
        sourceViewBasis: previousWorkspaceState.sourceViewBasis,
        companionTab: previousWorkspaceState.companionTab,
      });
    }
```

Change `selectGroup` signature:

```ts
  function selectGroup(
    groupId: number,
    { preserveRestoredCanvasState = false }: { preserveRestoredCanvasState?: boolean } = {},
  ) {
```

At the start of `selectGroup`, add:

```ts
    const previousWorkspaceState = workspaceUiState;
    applyWorkspaceUiState(selectSourceGroupWorkspace(workspaceUiState, groupId));
    historyScope = "current";
    if (activeChatRequestId !== null) {
      void cancelChat({ silent: true });
    }
    clearCurrentRunForWorkspaceSwitch();
```

At the end of `selectGroup`, add:

```ts
    if (preserveRestoredCanvasState) {
      applyWorkspaceUiState({
        ...workspaceUiState,
        canvasMode: previousWorkspaceState.canvasMode,
        sourceViewBasis: previousWorkspaceState.sourceViewBasis,
        companionTab: previousWorkspaceState.companionTab,
      });
    }
```

- [ ] **Step 8: Restore before loading active runs**

Replace the initial loading block in `onMount()`:

```ts
    void loadAccounts();
    void loadSourceCatalog().then(() => {
      if (selectedSourceId) {
        const sourceId = Number(selectedSourceId);
        const selected = sourceCatalog.find((source) => source.id === sourceId);
        void Promise.all([
          selected && sourceCapabilities(selected).hasTopics
            ? loadSourceTopics(sourceId)
            : Promise.resolve(),
          loadItems(sourceId),
          selected?.sourceType === "youtube" ? loadYoutubeDetail(selected) : Promise.resolve(),
        ]);
      }
    });
    void loadTemplates();
    void loadGroups();
    void loadActiveRuns();
```

with:

```ts
    restorePersistedWorkspaceState();
    void loadAccounts();
    void (async () => {
      await Promise.all([loadSourceCatalog(), loadGroups()]);
      const restoredSelectionApplied = await applyRestoredWorkspaceSelection();
      if (!restoredSelectionApplied && selectedSourceId) {
        const sourceId = Number(selectedSourceId);
        const selected = sourceCatalog.find((source) => source.id === sourceId);
        void Promise.all([
          selected && sourceCapabilities(selected).hasTopics
            ? loadSourceTopics(sourceId)
            : Promise.resolve(),
          loadItems(sourceId),
          selected?.sourceType === "youtube" ? loadYoutubeDetail(selected) : Promise.resolve(),
        ]);
      }
      void loadActiveRuns();
    })();
    void loadTemplates();
```

Do not call `void loadGroups();` elsewhere in this mount block after moving it into the async initialization block.

- [ ] **Step 9: Add the persistence effect**

Add this effect before the existing status auto-clear `$effect`:

```ts
  $effect(() => {
    workspaceUiState;
    historyScope;
    runFilter;

    if (!workspacePersistenceReady) {
      return;
    }

    persistWorkspaceState();
  });
```

This effect intentionally reads only durable workspace state and run-history preferences.

- [ ] **Step 10: Run the route raw-source test and verify it passes**

Run:

```powershell
npm.cmd test -- src/lib/analysis-route-workspace-state.test.ts
```

Expected: PASS.

- [ ] **Step 11: Run focused route/workflow/persistence tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-route-workspace-state.test.ts src/lib/analysis-run-workflow.test.ts src/lib/analysis-workspace-persistence.test.ts src/lib/analysis-workspace-state.test.ts
```

Expected: PASS.

- [ ] **Step 12: Commit route persistence wiring**

Run:

```powershell
git add src/routes/analysis/+page.svelte src/lib/analysis-route-workspace-state.test.ts
git commit -m "feat: persist analysis workspace state"
```

## Task 4: Run Part 2 Verification

**Files:**
- Verify all Part 2 files changed in Tasks 1-3.

- [ ] **Step 1: Run focused frontend tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-workspace-persistence.test.ts src/lib/analysis-run-workflow.test.ts src/lib/analysis-route-workspace-state.test.ts src/lib/analysis-workspace-state.test.ts
```

Expected: PASS.

- [ ] **Step 2: Run the full frontend test suite**

Run:

```powershell
npm.cmd test
```

Expected: PASS.

- [ ] **Step 3: Run Svelte and TypeScript checks**

Run:

```powershell
npm.cmd run check
```

Expected: PASS.

- [ ] **Step 4: Check whitespace**

Run:

```powershell
git diff --check
```

Expected: no output and exit code 0.

- [ ] **Step 5: Commit any final fixes**

If verification required fixes, commit them:

```powershell
git add src
git commit -m "test: verify analysis workspace persistence"
```

Skip this commit if there are no additional changes after Tasks 1-3.

- [ ] **Step 6: Stop for review**

Run:

```powershell
git status --short
```

Expected: clean working tree.

Report:

```text
Part 2 workspace state persistence is implemented and verified. Stopping before Part 3.
```

Do not begin Part 3 until the user explicitly approves continuing.

## Self-Review

- Spec coverage: this plan covers workspace persistence, restored-state normalization, no automatic run reopening, stale source/group fallback, source/group switches clearing run-bound state, and run opening aligning workspace selection.
- Placeholder scan: all tasks include concrete files, test code, implementation code, commands, expected outcomes, and commit commands.
- Type consistency: Part 2 uses Part 1 `AnalysisWorkspaceUiState`, `WorkspaceSelection`, `CanvasMode`, `SourceViewBasis`, `CompanionTab`, and existing `AnalysisHistoryScope`/`AnalysisRunFilter` names consistently.
