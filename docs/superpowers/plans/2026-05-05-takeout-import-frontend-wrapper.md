# Takeout Import Frontend Wrapper Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Centralize Takeout import frontend command/event access in `$lib/api/takeout-import.ts` and remove Takeout-specific raw Tauri calls from the `/analysis` route.

**Architecture:** Add a narrow typed wrapper that follows the existing `$lib/api/analysis-runs.ts` pattern. Keep existing Takeout DTO fields and route-local workflow state unchanged, so this refactor only moves the Tauri boundary.

**Tech Stack:** Svelte 5, SvelteKit, TypeScript, Tauri v2 API, Vitest.

---

## Context

Sources Contract V2 already moved core source command calls into
`src/lib/api/sources.ts`. The next cleanup is Takeout import, which still has
raw command names and one raw event listener in `src/routes/analysis/+page.svelte`.

Relevant existing patterns:

- `src/lib/api/analysis-runs.ts`
- `src/lib/api/analysis-runs.test.ts`
- `src/lib/api/sources.ts`
- `src/lib/api/sources.test.ts`

Relevant existing Takeout types:

- `src/lib/types/sources.ts`
  - `TakeoutImportJobRecord`
  - `TakeoutImportEvent`
  - `StartTakeoutImportResponse`
  - `CancelTakeoutImportResponse`

This task is wrapper-only. Do not rename Takeout DTO fields, do not change Rust
commands, and do not extract a Takeout workflow controller.

## File Structure

- Create `src/lib/api/takeout-import.ts`: typed wrapper for Takeout import
  commands and event listener.
- Create `src/lib/api/takeout-import.test.ts`: Vitest coverage for wrapper
  command names, payloads, event constant, and listener forwarding.
- Modify `src/routes/analysis/+page.svelte`: replace only Takeout raw
  `invoke(...)` and `listen(...)` usage with wrapper calls.

## Task 1: Add Takeout Import API Wrapper Tests

**Files:**

- Create: `src/lib/api/takeout-import.test.ts`

- [ ] **Step 1: Create the failing wrapper test file**

Create `src/lib/api/takeout-import.test.ts` with:

```ts
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  TAKEOUT_IMPORT_EVENT,
  cancelTakeoutSourceImport,
  listTakeoutSourceImportJobs,
  listenToTakeoutImportEvents,
  startTakeoutSourceImport,
} from "./takeout-import";
import type { TakeoutImportEvent } from "$lib/types/sources";

const invokeMock = vi.hoisted(() => vi.fn());
const listenMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: listenMock,
}));

describe("takeout import api wrapper", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
  });

  it("lists takeout import jobs with the existing command", async () => {
    invokeMock.mockResolvedValueOnce([]);

    await expect(listTakeoutSourceImportJobs()).resolves.toEqual([]);

    expect(invokeMock).toHaveBeenLastCalledWith("list_takeout_source_import_jobs");
  });

  it("starts a takeout import for a source", async () => {
    invokeMock.mockResolvedValueOnce({ job_id: "takeout-1" });

    await expect(startTakeoutSourceImport(7)).resolves.toEqual({
      job_id: "takeout-1",
    });

    expect(invokeMock).toHaveBeenLastCalledWith("start_takeout_source_import", {
      sourceId: 7,
    });
  });

  it("cancels a takeout import job", async () => {
    invokeMock.mockResolvedValueOnce({ cancelled: true });

    await expect(cancelTakeoutSourceImport("takeout-1")).resolves.toEqual({
      cancelled: true,
    });

    expect(invokeMock).toHaveBeenLastCalledWith("cancel_takeout_source_import", {
      jobId: "takeout-1",
    });
  });

  it("listens on the shared takeout import event name", async () => {
    const unlisten = vi.fn();
    const handler = vi.fn();
    listenMock.mockResolvedValueOnce(unlisten);

    await expect(listenToTakeoutImportEvents(handler)).resolves.toBe(unlisten);
    expect(TAKEOUT_IMPORT_EVENT).toBe("sources://takeout-import");
    expect(listenMock).toHaveBeenCalledWith(TAKEOUT_IMPORT_EVENT, expect.any(Function));

    const payload: TakeoutImportEvent = {
      job_id: "takeout-1",
      source_id: 7,
      account_id: 2,
      status: "running",
      phase: "importing_history",
      message: "Importing",
      inserted: 12,
      skipped: 1,
      progress_current: 12,
      progress_total: 40,
      started_at: 1_700_000,
      finished_at: null,
      warnings: [],
      error: null,
    };
    const event = { payload };

    listenMock.mock.calls[0][1](event);

    expect(handler).toHaveBeenCalledWith(event);
  });
});
```

- [ ] **Step 2: Run the focused test and confirm it fails because the wrapper does not exist**

Run:

```powershell
npm.cmd test -- takeout-import
```

Expected result:

```text
FAIL src/lib/api/takeout-import.test.ts
Cannot find module './takeout-import'
```

## Task 2: Implement the Takeout Import API Wrapper

**Files:**

- Create: `src/lib/api/takeout-import.ts`
- Test: `src/lib/api/takeout-import.test.ts`

- [ ] **Step 1: Create the wrapper module**

Create `src/lib/api/takeout-import.ts` with:

```ts
import { invoke } from "@tauri-apps/api/core";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  CancelTakeoutImportResponse,
  StartTakeoutImportResponse,
  TakeoutImportEvent,
  TakeoutImportJobRecord,
} from "$lib/types/sources";

export const TAKEOUT_IMPORT_EVENT = "sources://takeout-import";

export function listTakeoutSourceImportJobs() {
  return invoke<TakeoutImportJobRecord[]>("list_takeout_source_import_jobs");
}

export function startTakeoutSourceImport(sourceId: number) {
  return invoke<StartTakeoutImportResponse>("start_takeout_source_import", { sourceId });
}

export function cancelTakeoutSourceImport(jobId: string) {
  return invoke<CancelTakeoutImportResponse>("cancel_takeout_source_import", { jobId });
}

export function listenToTakeoutImportEvents(
  handler: (event: Event<TakeoutImportEvent>) => void,
): Promise<UnlistenFn> {
  return listen<TakeoutImportEvent>(TAKEOUT_IMPORT_EVENT, handler);
}
```

- [ ] **Step 2: Run the focused wrapper test**

Run:

```powershell
npm.cmd test -- takeout-import
```

Expected result:

```text
PASS src/lib/api/takeout-import.test.ts
```

## Task 3: Migrate the Analysis Route to the Wrapper

**Files:**

- Modify: `src/routes/analysis/+page.svelte`
- Test: `src/lib/api/takeout-import.test.ts`

- [ ] **Step 1: Update imports in the analysis route**

In `src/routes/analysis/+page.svelte`, add this import near the other
`$lib/api/*` imports:

```ts
import {
  cancelTakeoutSourceImport,
  listTakeoutSourceImportJobs,
  listenToTakeoutImportEvents,
  startTakeoutSourceImport,
} from "$lib/api/takeout-import";
```

Keep this existing import because other non-Takeout calls still use it:

```ts
import { invoke } from "@tauri-apps/api/core";
```

Keep this existing import because chat and NotebookLM listeners still use it:

```ts
import { listen } from "@tauri-apps/api/event";
```

Remove these type imports from the `$lib/types/sources` import list:

```ts
CancelTakeoutImportResponse,
StartTakeoutImportResponse,
```

- [ ] **Step 2: Replace Takeout job listing**

Change `loadTakeoutImportJobs` from:

```ts
async function loadTakeoutImportJobs() {
  try {
    const jobs = await invoke<TakeoutImportJobRecord[]>("list_takeout_source_import_jobs");
    applyTakeoutJobs(jobs);
  } catch (error) {
    status = formatAppError("loading Takeout import jobs", error);
  }
}
```

to:

```ts
async function loadTakeoutImportJobs() {
  try {
    const jobs = await listTakeoutSourceImportJobs();
    applyTakeoutJobs(jobs);
  } catch (error) {
    status = formatAppError("loading Takeout import jobs", error);
  }
}
```

- [ ] **Step 3: Replace Takeout start command**

Change the command call inside `startTakeoutImport` from:

```ts
await invoke<StartTakeoutImportResponse>("start_takeout_source_import", { sourceId });
```

to:

```ts
await startTakeoutSourceImport(sourceId);
```

- [ ] **Step 4: Replace Takeout cancel command**

Change the command call inside `cancelTakeoutImport` from:

```ts
const result = await invoke<CancelTakeoutImportResponse>(
  "cancel_takeout_source_import",
  { jobId },
);
```

to:

```ts
const result = await cancelTakeoutSourceImport(jobId);
```

- [ ] **Step 5: Replace only the Takeout event listener**

Change the Takeout listener in `onMount` from:

```ts
void listen<TakeoutImportEvent>("sources://takeout-import", ({ payload }: EventEnvelope<TakeoutImportEvent>) => {
  if (disposed) {
    return;
  }

  applyTakeoutImportEvent(payload);
}).then((unlisten) => {
  if (disposed) {
    unlisten();
    return;
  }
  detachTakeoutImportListener = unlisten;
});
```

to:

```ts
void listenToTakeoutImportEvents(({ payload }) => {
  if (disposed) {
    return;
  }

  applyTakeoutImportEvent(payload);
}).then((unlisten) => {
  if (disposed) {
    unlisten();
    return;
  }
  detachTakeoutImportListener = unlisten;
});
```

Do not change the chat listener or NotebookLM listener in this task.

- [ ] **Step 6: Verify no raw Takeout command or event strings remain in the route**

Run:

```powershell
rg -n "list_takeout_source_import_jobs|start_takeout_source_import|cancel_takeout_source_import|sources://takeout-import" src\routes\analysis\+page.svelte
```

Expected result:

```text
no matches
```

## Task 4: Verification and Commit

**Files:**

- Verify: `src/lib/api/takeout-import.ts`
- Verify: `src/lib/api/takeout-import.test.ts`
- Verify: `src/routes/analysis/+page.svelte`

- [ ] **Step 1: Run focused wrapper tests**

Run:

```powershell
npm.cmd test -- takeout-import
```

Expected result:

```text
1 test file passed
```

- [ ] **Step 2: Run nearby frontend API wrapper tests**

Run:

```powershell
npm.cmd test -- analysis-runs sources takeout-import
```

Expected result:

```text
3 test files passed
```

- [ ] **Step 3: Run full frontend tests**

Run:

```powershell
npm.cmd test
```

Expected result:

```text
all test files passed
```

- [ ] **Step 4: Run Svelte and TypeScript checks**

Run:

```powershell
npm.cmd run check
```

Expected result:

```text
0 errors and 0 warnings
```

If this command fails in the default sandbox with `spawn EPERM`, rerun it
outside the sandbox. This is a known environment issue in the repo notes.

- [ ] **Step 5: Run whitespace check**

Run:

```powershell
git diff --check
```

Expected result:

```text
no output
```

- [ ] **Step 6: Commit**

Run:

```powershell
git add src/lib/api/takeout-import.ts src/lib/api/takeout-import.test.ts src/routes/analysis/+page.svelte
git commit -m "refactor(takeout): add frontend api wrapper"
```

If git writes fail because `.git/index.lock` is denied by the Windows sandbox,
rerun the git command outside the sandbox after approval.

## Self-Review Checklist

- The wrapper is the only new API surface.
- The route no longer owns Takeout command names or the Takeout event name.
- Existing Takeout DTO field names stay unchanged.
- No Rust files are modified.
- No NotebookLM, chat, source group, template, or source management workflows
  are refactored in this task.
- Tests cover commands, payloads, event name, and listener forwarding.

## Commit Message

```text
refactor(takeout): add frontend api wrapper
```
