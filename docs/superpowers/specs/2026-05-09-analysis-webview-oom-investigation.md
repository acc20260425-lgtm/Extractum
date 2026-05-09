# Analysis WebView OOM Investigation

Date: 2026-05-09
Area: Tauri WebView / Svelte analysis workspace
Status: Root cause found and patched locally

## Summary

The intermittent out-of-memory failure was traced to the `/analysis` route in the
WebView renderer process. The Rust backend stayed small and healthy, while the
`msedgewebview2` renderer grew into multi-gigabyte memory usage and eventually
became unresponsive or crashed.

The root cause was a Svelte 5 `$effect` that called `loadRuns()` directly. Because
Svelte tracks synchronous state reads inside functions called by an effect, the
effect subscribed to additional state read by the run-loading workflow. The
workflow also patches route state (`loadingRuns`, `runs`, and related state),
which caused repeated reactive work and rapid JS heap growth on the analysis
page.

The local fix wraps the asynchronous `loadRuns()` call in `untrack`, keeping the
effect dependent only on `historyScopeParams`:

```ts
import { onMount, untrack } from "svelte";

$effect(() => {
  if (historyScopeParams === null) {
    runs = [];
    return;
  }

  void untrack(() => loadRuns());
});
```

## Environment

- Workspace: `G:\Develop\Extractum`
- App: `org.ai.extractum`
- Tauri: `2.10.3`
- Runtime: Windows / Edge WebView2
- Route under investigation: `http://localhost:1420/analysis`
- WebView process: `msedgewebview2.exe`
- Backend process: `extractum.exe`

## Initial Symptoms

The user reported an intermittent "out of memory" error while the desktop app was
running. The error was not tied to a single visible user action at first.

Initial MCP connection showed:

- App window was open at `/analysis`.
- DOM was small, around 487-578 elements.
- The WebView JS heap and renderer process memory were very large.
- The Rust backend memory stayed low.

## Evidence Collected

### Process-Level Memory

During the failure investigation, the renderer process grew to multi-gigabyte
usage:

- `msedgewebview2`: approximately 4.8 GB working set during the first capture.
- Later grew to approximately 6.4 GB before becoming unresponsive/crashing.
- `extractum.exe`: approximately 50-60 MB during the same period.

This separated the issue from the Rust backend and pointed to the WebView
renderer/frontend.

### DOM Size

The DOM was not large enough to explain the memory use:

- `/analysis` DOM: roughly 487-578 nodes.
- Body text length: roughly 11 KB.

This made a pure DOM blow-up unlikely.

### Database Size and Stored Artifacts

The local database was inspected to rule out giant persisted reports, traces, or
chat messages being loaded by the page.

Observed:

- Database size: about 60.7 MB.
- `items`: 89,488 rows.
- `analysis_runs`: 8 rows.
- `analysis_chat_messages`: 0 rows.
- Largest compressed item payloads: around 3 KB each.
- Largest saved report/trace artifacts: only a few KB.

Conclusion: the OOM was not caused by loading a giant saved report, trace, or
chat history into the UI.

### Route Comparison

The `/settings` route was used as a control:

- `/settings` JS heap: about 8-12 MB.
- `/settings` renderer working set: around 150-210 MB.

On `/analysis`, before the fix:

- JS heap reached about 3.0 GB.
- Renderer working set reached 5+ GB.

This localized the issue to the analysis route rather than WebView2 globally.

### Passive Memory Reproduction

After navigating to `/analysis`, memory grew without clicks or user actions:

Before fix, passive sampling showed growth over about one minute:

```text
07:20:21  WorkingSetMB 1235.7  PrivateMB 1195.6
07:20:26  WorkingSetMB 1345.5  PrivateMB 1305.6
07:20:31  WorkingSetMB 1659.7  PrivateMB 1620.5
07:20:36  WorkingSetMB 2009.1  PrivateMB 1972.0
07:20:41  WorkingSetMB 2351.9  PrivateMB 2316.8
07:20:46  WorkingSetMB 2690.6  PrivateMB 2656.2
07:20:51  WorkingSetMB 3018.3  PrivateMB 2983.6
07:20:56  WorkingSetMB 3297.8  PrivateMB 3266.2
07:21:16  WorkingSetMB 3455.8  PrivateMB 3420.5
```

CPU was effectively idle, indicating retained reactive/runtime work rather than
active compute.

### IPC Monitoring

IPC monitoring did not show a flood of backend commands such as
`list_analysis_runs`. This reduced confidence in a backend polling loop as the
primary cause.

The issue was therefore narrowed to frontend reactivity/runtime memory, not
backend request volume.

## Root Cause

The problematic code was in `src/routes/analysis/+page.svelte`:

```ts
$effect(() => {
  if (historyScopeParams === null) {
    runs = [];
    return;
  }

  void loadRuns();
});
```

Svelte 5 tracks state and derived values synchronously read inside `$effect`,
including reads that happen indirectly through called functions.

`loadRuns()` delegates to the analysis run workflow. That workflow reads current
route/workflow state and patches route state while loading saved runs. Because
the call happened inside the effect's tracking context, the effect subscribed to
more state than intended. The resulting dependency loop caused repeated reactive
work and large retained JS heap on `/analysis`.

The intended dependency is only `historyScopeParams`.

## Fix

Changed:

```ts
void loadRuns();
```

To:

```ts
void untrack(() => loadRuns());
```

And imported `untrack`:

```ts
import { onMount, untrack } from "svelte";
```

This prevents state reads inside `loadRuns()` from becoming dependencies of the
effect, while preserving the intended behavior: load saved runs whenever
`historyScopeParams` changes.

## Verification

### Svelte Check

Command:

```powershell
npm.cmd run check
```

Result:

```text
svelte-check found 0 errors and 0 warnings
```

### Frontend Tests

Command:

```powershell
npm.cmd test -- analysis
```

Result:

```text
Test Files  16 passed (16)
Tests       160 passed (160)
```

### Whitespace Check

Command:

```powershell
git diff --check
```

Result:

No whitespace errors. Git printed only the existing line-ending warning:

```text
warning: in the working copy of 'src/routes/analysis/+page.svelte',
LF will be replaced by CRLF the next time Git touches it
```

### Live Memory Check After Fix

After navigating from `/settings` back to `/analysis`, the renderer stayed
stable:

Initial post-fix `/analysis` sample:

- JS heap: about 16-17 MB.
- Renderer working set: about 216 MB, then settled lower.

Passive sampling after fix:

```text
07:24:46  WorkingSetMB 179.3  PrivateMB 136.3
07:24:51  WorkingSetMB 162.8  PrivateMB 120.1
07:24:56  WorkingSetMB 162.8  PrivateMB 120.1
07:25:01  WorkingSetMB 162.8  PrivateMB 120.3
07:25:06  WorkingSetMB 157.9  PrivateMB 115.4
07:25:11  WorkingSetMB 157.9  PrivateMB 115.4
07:25:16  WorkingSetMB 157.9  PrivateMB 115.4
07:25:21  WorkingSetMB 157.9  PrivateMB 115.4
07:25:26  WorkingSetMB 157.9  PrivateMB 115.4
07:25:31  WorkingSetMB 157.9  PrivateMB 115.4
07:25:36  WorkingSetMB 157.5  PrivateMB 115.0
07:25:41  WorkingSetMB 157.5  PrivateMB 115.0
```

Final direct WebView check:

```text
JS heap total: about 21 MB
JS heap used:  about 17 MB
Renderer working set: about 172.8 MB
Renderer private memory: about 131.7 MB
```

## Files Changed

- `src/routes/analysis/+page.svelte`

## Risk Assessment

Risk is low:

- The behavior remains the same from the user's perspective.
- Saved runs still load when `historyScopeParams` changes.
- The change only prevents accidental Svelte dependency tracking inside the
  asynchronous loader.
- Static checks and analysis tests pass.
- Live memory behavior changed from unbounded growth to stable memory usage.

## Follow-Up Recommendations

1. Review other `$effect` blocks that call functions which read and write route
   state. Similar cases may need `untrack` or a different lifecycle pattern.
2. Consider moving one-shot data loads to explicit event handlers or lifecycle
   flows where possible, keeping `$effect` for narrow dependency-driven work.
3. Add a lightweight developer note about Svelte 5 `$effect` dependency tracking
   in the frontend architecture docs if this pattern appears elsewhere.

## Suggested Commit Message

```text
fix(analysis): prevent WebView OOM from tracked run loading

Wrap the analysis saved-run loader in Svelte's untrack() when it is
called from the history-scope effect. The effect only needs to depend on
historyScopeParams, but calling loadRuns() directly allowed synchronous
state reads inside the workflow to become tracked dependencies.

That accidental dependency tracking caused repeated reactive work on the
/analysis route and rapidly grew the WebView renderer heap into multiple
gigabytes while the Rust backend stayed small.

Verification:
- npm.cmd run check
- npm.cmd test -- analysis
- git diff --check
- live MCP memory check: /analysis renderer stabilized around 157-173 MB
  instead of growing past 3-5 GB
```
