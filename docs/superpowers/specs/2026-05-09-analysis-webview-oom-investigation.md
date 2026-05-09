# Analysis WebView OOM Investigation

Date: 2026-05-09
Area: Tauri WebView / Svelte analysis workspace
Status: Root cause fixed; explicit loader refactor implemented

## Summary

The intermittent out-of-memory failure was traced to the `/analysis` route in the
WebView renderer process. The Rust backend stayed small and healthy, while the
`msedgewebview2` renderer grew into multi-gigabyte memory usage and eventually
became unresponsive or crashed.

The root cause was a Svelte 5 `$effect` that called `loadRuns()` directly. The
call synchronously entered the run-loading workflow, whose `deps.getState()`
constructed a broader route-state object than the effect intended. Constructing
that object synchronously read route `$state`, including `runs`. Because Svelte
tracks synchronous state reads inside functions called by an effect, the effect
subscribed to `runs` in addition to `historyScopeParams`. When the loader later
patched `runs`, the effect re-ran and scheduled another load, causing repeated
reactive work and rapid JS heap growth on the analysis page.

The original fix wrapped the asynchronous `loadRuns()` call in `untrack`. A
follow-up refactor now makes the dependency explicit: the route effect reads
`historyScopeParams` directly and passes that value to a workflow method that
does not call `deps.getState()`.

```ts
$effect(() => {
  const params = historyScopeParams;
  if (params === null) {
    runs = [];
    return;
  }

  void runWorkflow.loadRunsForScope(params);
});
```

The broad `loadRuns()` wrapper still exists for event-driven and user-action
call sites outside Svelte effect tracking contexts.

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

The failure appeared intermittent because the loop only became relevant when the
analysis history scope was ready. If `historyScopeParams` was `null`, the effect
cleared `runs` and returned. Once the route had a selected global/source/group
history scope, the effect entered `loadRuns()` and exposed the accidental
dependency on `runs`.

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
`list_analysis_runs`. This reduced confidence in a backend-driven polling loop
as the primary cause. It did not rule out frontend-triggered repeated loads; it
only made the backend unlikely to be the source of the loop.

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

`loadRuns()` delegates to the analysis run workflow. That workflow starts by
calling `deps.getState()`, and the route-level `getState` function constructs an
object from `historyScopeParams`, `activeRunId`, `currentRun`, `runs`,
`activeRuns`, and deletion state. The important part is the synchronous reads
that happen while constructing the object; that is when Svelte can register
dependencies. Because the call happened inside the effect's tracking context,
all of those reads were eligible to become effect dependencies.

The feedback edge was `runs`: the effect accidentally subscribed to it, then the
loader later patched `runs` after `listRuns` returned. That patch re-triggered
the effect, which scheduled another run load. The resulting dependency loop
caused repeated reactive work and large retained JS heap on `/analysis`.

The intended dependency is only `historyScopeParams`.

## Route Effect Audit

The current `$effect` blocks in `src/routes/analysis/+page.svelte` were checked
after identifying the root cause:

- The saved-run history effect was the risky pattern before the fix:
  `$effect` -> `loadRuns()` -> workflow `deps.getState()` -> route `$state`
  reads -> later patch of `runs`. It now reads `historyScopeParams` explicitly
  and calls `loadRunsForScope(params)`, which does not call `deps.getState()`.
- The template editor binding effect reads `selectedTemplate` and
  `editorBoundTemplateId`, then writes editor form state only when the selected
  template id changes. It does not call a workflow `getState`.
- The source-group editor binding effect mirrors the template pattern and is
  guarded by `editorBoundGroupId`.
- The status timer effect reads and writes timer/status state, but it is bounded
  by clearing the previous timeout and only schedules one timeout per status
  message.

No other analysis-route effect currently has the `$effect` -> workflow
`deps.getState()` pattern that caused this OOM. Future effects that call route
workflow functions should either keep synchronous reads intentionally narrow,
prefer explicit parameter APIs, or wrap incidental reads in `untrack`.

## Fix

The OOM was first fixed by changing:

```ts
void loadRuns();
```

To:

```ts
void untrack(() => loadRuns());
```

That kept broad workflow state reads out of the effect's tracking context.

The current implementation removes the need for `untrack` in this effect by
using an explicit-scope workflow API:

```ts
$effect(() => {
  const params = historyScopeParams;
  if (params === null) {
    runs = [];
    return;
  }

  void runWorkflow.loadRunsForScope(params);
});
```

`loadRunsForScope(params)` uses only the provided scope argument. The existing
`loadRuns()` method remains as a convenience wrapper for non-effect contexts and
delegates to `loadRunsForScope(deps.getState().historyScopeParams)`.

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
Test Files  17 passed (17)
Tests       163 passed (163)
```

### Whitespace Check

Command:

```powershell
git diff --check
```

Result:

No whitespace errors. Git printed only line-ending warnings for touched files
because the Windows checkout will rewrite LF to CRLF on the next Git write.

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

No automated test currently asserts renderer memory stability. The regression
was validated with the live MCP memory check above.

## Files Changed

- `src/routes/analysis/+page.svelte`
- `src/lib/analysis-run-workflow.ts`
- `src/lib/analysis-run-workflow.test.ts`
- `src/lib/analysis-route-effects.test.ts`

## Risk Assessment

Risk is low:

- The behavior remains the same from the user's perspective.
- Saved runs still load when `historyScopeParams` changes.
- The effect-path loader no longer enters broad workflow state reads, preventing
  accidental Svelte dependency tracking inside the saved-run loader.
- Static checks and analysis tests pass.
- Live memory behavior changed from unbounded growth to stable memory usage.

## Follow-Up Recommendations

1. When adding new `$effect` blocks, treat calls into route workflow functions as
   high-risk if they synchronously call `deps.getState()` and later patch any of
   the same route state. Similar cases may need an explicit parameter API,
   `untrack`, or a different lifecycle pattern.
2. Consider moving one-shot data loads to explicit event handlers or lifecycle
   flows where possible, keeping `$effect` for narrow dependency-driven work.
3. Keep the frontend architecture note on Svelte 5 `$effect` dependency tracking
   updated if more route workflow patterns are added.

## Suggested Commit Message

```text
refactor(analysis): make saved-run loading scope explicit

Add loadRunsForScope(params) so the analysis history effect can pass its
intended dependency directly instead of entering the broad loadRuns()
wrapper inside Svelte tracking.

Keep loadRuns() as the convenience wrapper for event-driven refreshes.
Update the route regression guard to require the explicit-scope loader
and to reject the old broad wrapper from the history effect.

Verification:
- npm.cmd test -- analysis-run-workflow analysis-route-effects
- npm.cmd run check
- npm.cmd test -- analysis
- git diff --check
```
