# NotebookLM Export Frontend Wrapper Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Centralize NotebookLM export frontend command/event access in `$lib/api/notebooklm-export.ts` and remove NotebookLM-specific raw Tauri calls from the `/analysis` route.

**Architecture:** Add a narrow typed wrapper that follows the existing `$lib/api/takeout-import.ts` and `$lib/api/analysis-runs.ts` patterns. Keep existing NotebookLM DTO fields, route-local form state, folder picker behavior, and lifecycle state unchanged, so this refactor only moves the Tauri boundary.

**Tech Stack:** Svelte 5, SvelteKit, TypeScript, Tauri v2 API, Vitest.

---

## Context

Core Sources and Takeout import already have focused frontend API wrappers:

- `src/lib/api/sources.ts`
- `src/lib/api/takeout-import.ts`
- `src/lib/api/analysis-runs.ts`

NotebookLM export still has one raw command and one raw event listener in
`src/routes/analysis/+page.svelte`:

```text
export_source_to_notebooklm
notebooklm://export
```

This task is wrapper-only. Do not rename NotebookLM DTO fields, do not change
Rust commands, do not wrap the folder picker, and do not extract a NotebookLM
workflow controller.

Relevant existing NotebookLM types:

- `src/lib/types/sources.ts`
  - `NotebookLmExportRequest`
  - `NotebookLmExportResult`
  - `NotebookLmExportEvent`

Relevant route-local helpers that must stay unchanged:

- `createNotebookLmExportId()` in `src/routes/analysis/+page.svelte`
- `notebookLmExportRequestFromForm(...)` in `src/lib/analysis-state.ts`
- `notebookLmExportProgressFromEvent(...)` in `src/lib/analysis-state.ts`
- `notebookLmExportInitialProgress()` in `src/lib/analysis-state.ts`
- `notebookLmExportCompleteStatus(...)` in `src/lib/analysis-state.ts`

## File Structure

- Create `src/lib/api/notebooklm-export.ts`: typed wrapper for NotebookLM export
  command and event listener.
- Create `src/lib/api/notebooklm-export.test.ts`: Vitest coverage for wrapper
  command name, payload shape, event constant, and listener forwarding.
- Modify `src/routes/analysis/+page.svelte`: replace only NotebookLM export raw
  `invoke(...)` and `listen(...)` usage with wrapper calls.

## Task 1: Add NotebookLM Export API Wrapper Tests

**Files:**

- Create: `src/lib/api/notebooklm-export.test.ts`

- [ ] **Step 1: Create the failing wrapper test file**

Create `src/lib/api/notebooklm-export.test.ts` with:

```ts
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  NOTEBOOKLM_EXPORT_EVENT,
  exportSourceToNotebookLm,
  listenToNotebookLmExportEvents,
} from "./notebooklm-export";
import type {
  NotebookLmExportEvent,
  NotebookLmExportRequest,
  NotebookLmExportResult,
} from "$lib/types/sources";

const invokeMock = vi.hoisted(() => vi.fn());
const listenMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: listenMock,
}));

function notebookLmExportRequest(
  overrides: Partial<NotebookLmExportRequest> = {},
): NotebookLmExportRequest {
  return {
    export_id: "export-a",
    source_id: 7,
    output_dir: "C:/Exports",
    period_from: 1_700_000,
    period_to: 1_786_000,
    include_media_placeholders: true,
    min_message_length: 5,
    max_words_per_file: 1000,
    max_bytes_per_file: 5000,
    overwrite_existing: false,
    ...overrides,
  };
}

function notebookLmExportResult(
  overrides: Partial<NotebookLmExportResult> = {},
): NotebookLmExportResult {
  return {
    output_dir: "C:/Exports",
    files: [
      {
        path: "C:/Exports/source.md",
        message_count: 12,
        byte_size: 1024,
        approximate_word_count: 300,
      },
    ],
    glossary_file: null,
    exported_message_count: 12,
    skipped_message_count: 2,
    warning_count: 0,
    warnings: [],
    ...overrides,
  };
}

describe("notebooklm export api wrapper", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
  });

  it("exports a source for NotebookLM with the existing command and request payload", async () => {
    const request = notebookLmExportRequest();
    const result = notebookLmExportResult();
    invokeMock.mockResolvedValueOnce(result);

    await expect(exportSourceToNotebookLm(request)).resolves.toBe(result);

    expect(invokeMock).toHaveBeenLastCalledWith("export_source_to_notebooklm", {
      request,
    });
  });

  it("listens on the shared NotebookLM export event name", async () => {
    const unlisten = vi.fn();
    const handler = vi.fn();
    listenMock.mockResolvedValueOnce(unlisten);

    await expect(listenToNotebookLmExportEvents(handler)).resolves.toBe(unlisten);
    expect(NOTEBOOKLM_EXPORT_EVENT).toBe("notebooklm://export");
    expect(listenMock).toHaveBeenCalledWith(NOTEBOOKLM_EXPORT_EVENT, expect.any(Function));

    const payload: NotebookLmExportEvent = {
      export_id: "export-a",
      source_id: 7,
      kind: "progress",
      phase: "writing",
      message: "Writing files",
      progress_current: 2,
      progress_total: 5,
      file_path: "C:/Exports/source.md",
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
npm.cmd test -- notebooklm-export
```

Expected result:

```text
FAIL src/lib/api/notebooklm-export.test.ts
Cannot find module './notebooklm-export'
```

- [ ] **Step 3: Commit Task 1**

Run:

```powershell
git add src/lib/api/notebooklm-export.test.ts
git commit -m "test(notebooklm): add export api wrapper contract tests"
```

If git writes fail because `.git/index.lock` is denied by the Windows sandbox,
rerun the git command outside the sandbox after approval.

## Task 2: Implement the NotebookLM Export API Wrapper

**Files:**

- Create: `src/lib/api/notebooklm-export.ts`
- Test: `src/lib/api/notebooklm-export.test.ts`

- [ ] **Step 1: Create the wrapper module**

Create `src/lib/api/notebooklm-export.ts` with:

```ts
import { invoke } from "@tauri-apps/api/core";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  NotebookLmExportEvent,
  NotebookLmExportRequest,
  NotebookLmExportResult,
} from "$lib/types/sources";

export const NOTEBOOKLM_EXPORT_EVENT = "notebooklm://export";

export function exportSourceToNotebookLm(request: NotebookLmExportRequest) {
  return invoke<NotebookLmExportResult>("export_source_to_notebooklm", { request });
}

export function listenToNotebookLmExportEvents(
  handler: (event: Event<NotebookLmExportEvent>) => void,
): Promise<UnlistenFn> {
  return listen<NotebookLmExportEvent>(NOTEBOOKLM_EXPORT_EVENT, handler);
}
```

- [ ] **Step 2: Run the focused wrapper test**

Run:

```powershell
npm.cmd test -- notebooklm-export
```

Expected result:

```text
PASS src/lib/api/notebooklm-export.test.ts
```

- [ ] **Step 3: Commit Task 2**

Run:

```powershell
git add src/lib/api/notebooklm-export.ts src/lib/api/notebooklm-export.test.ts
git commit -m "feat(notebooklm): add export api wrapper"
```

If git writes fail because `.git/index.lock` is denied by the Windows sandbox,
rerun the git command outside the sandbox after approval.

## Task 3: Migrate the Analysis Route to the Wrapper

**Files:**

- Modify: `src/routes/analysis/+page.svelte`
- Test: `src/lib/api/notebooklm-export.test.ts`

- [ ] **Step 1: Update imports in the analysis route**

In `src/routes/analysis/+page.svelte`, add this import near the other
`$lib/api/*` imports:

```ts
import {
  exportSourceToNotebookLm,
  listenToNotebookLmExportEvents,
} from "$lib/api/notebooklm-export";
```

Keep this existing import because other non-NotebookLM calls still use it:

```ts
import { invoke } from "@tauri-apps/api/core";
```

Keep this existing import because the chat listener still uses it:

```ts
import { listen } from "@tauri-apps/api/event";
```

Keep this existing import because the folder picker remains route-local:

```ts
import { open as openDialog } from "@tauri-apps/plugin-dialog";
```

Remove `NotebookLmExportResult` from the `$lib/types/sources` import list if
the route no longer needs it after the command call is migrated. Keep
`NotebookLmExportEvent` if it is still needed by `applyNotebookLmExportEvent`.

- [ ] **Step 2: Replace the NotebookLM export command**

Change the command call inside `exportNotebookLm` from:

```ts
const result = await invoke<NotebookLmExportResult>("export_source_to_notebooklm", {
  request,
});
```

to:

```ts
const result = await exportSourceToNotebookLm(request);
```

- [ ] **Step 3: Replace only the NotebookLM export event listener**

Change the NotebookLM export listener in `onMount` from:

```ts
void listen<NotebookLmExportEvent>("notebooklm://export", ({ payload }: EventEnvelope<NotebookLmExportEvent>) => {
  if (disposed) {
    return;
  }

  applyNotebookLmExportEvent(payload);
}).then((unlisten) => {
  if (disposed) {
    unlisten();
    return;
  }
  detachNotebookLmExportListener = unlisten;
});
```

to:

```ts
void listenToNotebookLmExportEvents(({ payload }) => {
  if (disposed) {
    return;
  }

  applyNotebookLmExportEvent(payload);
}).then((unlisten) => {
  if (disposed) {
    unlisten();
    return;
  }
  detachNotebookLmExportListener = unlisten;
});
```

Do not change the chat listener in this task.

- [ ] **Step 4: Verify no raw NotebookLM export command or event strings remain in the route**

Run:

```powershell
rg -n "export_source_to_notebooklm|notebooklm://export" src\routes\analysis\+page.svelte
```

Expected result:

```text
no matches
```

- [ ] **Step 5: Run the focused wrapper test**

Run:

```powershell
npm.cmd test -- notebooklm-export
```

Expected result:

```text
1 test file passed
```

- [ ] **Step 6: Commit Task 3**

Run:

```powershell
git add src/routes/analysis/+page.svelte src/lib/api/notebooklm-export.ts src/lib/api/notebooklm-export.test.ts
git commit -m "refactor(notebooklm): use export api wrapper in analysis route"
```

If git writes fail because `.git/index.lock` is denied by the Windows sandbox,
rerun the git command outside the sandbox after approval.

## Task 4: Verification and Commit

**Files:**

- Verify: `src/lib/api/notebooklm-export.ts`
- Verify: `src/lib/api/notebooklm-export.test.ts`
- Verify: `src/routes/analysis/+page.svelte`

- [ ] **Step 1: Run focused and nearby frontend tests**

Run:

```powershell
npm.cmd test -- analysis-state notebooklm-export takeout-import analysis-runs sources
```

Expected result:

```text
all selected test files passed
```

- [ ] **Step 2: Run full frontend tests**

Run:

```powershell
npm.cmd test
```

Expected result:

```text
all test files passed
```

- [ ] **Step 3: Run Svelte and TypeScript checks**

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

- [ ] **Step 4: Run whitespace check**

Run:

```powershell
git diff --check
```

Expected result:

```text
no output
```

- [ ] **Step 5: Commit Task 4**

If verification produced file changes, commit those files. If verification did
not produce file changes, create an empty verification commit because the user
requested one commit at the end of each top-level task:

```powershell
git commit --allow-empty -m "test(notebooklm): verify export wrapper integration"
```

If git writes fail because `.git/index.lock` is denied by the Windows sandbox,
rerun the git command outside the sandbox after approval.

## Self-Review Checklist

- The wrapper is the only new frontend API surface.
- The route no longer owns the NotebookLM export command name or event name.
- Existing NotebookLM DTO field names stay unchanged.
- `openDialog(...)` remains route-local.
- No Rust files are modified.
- No chat, template, source group, Takeout, or source management workflows are
  refactored in this task.
- Tests cover command name, request payload shape, event name, and listener
  forwarding.

## Commit Messages

```text
test(notebooklm): add export api wrapper contract tests
feat(notebooklm): add export api wrapper
refactor(notebooklm): use export api wrapper in analysis route
test(notebooklm): verify export wrapper integration
```
