# Analysis `loadRuns` API Refactor Task

Date: 2026-05-09
Status: Ready for implementation
Area: Svelte analysis route / analysis run workflow

## Context

The `/analysis` WebView OOM was fixed by wrapping the saved-run loader call in
`untrack`. The root cause was that the history-scope `$effect` called
`loadRuns()`, which synchronously entered `deps.getState()` and read route
`$state` beyond the intended `historyScopeParams` dependency.

The current regression guard protects the exact fixed pattern, but the workflow
API still hides the scope dependency inside `loadRuns()`. This task is a
separate refactor to make the saved-run loader API more explicit and easier to
test. It is not required for the OOM fix itself.

## Goal

Introduce an explicit saved-run loading API that accepts
`AnalysisHistoryScopeParams | null` as an argument, so the route history effect
can pass its intended dependency directly instead of entering broad
`deps.getState()` tracking.

## Recommended Design

Add a new workflow method:

```ts
loadRunsForScope(params: AnalysisHistoryScopeParams | null): Promise<void>
```

Move the current saved-run loading body into `loadRunsForScope(params)`.

Keep the existing `loadRuns()` method as a convenience wrapper for non-effect
contexts:

```ts
async function loadRuns() {
  await loadRunsForScope(deps.getState().historyScopeParams);
}
```

This preserves existing event-driven and user-action callers, while allowing
the route history effect to use the explicit API.

## Route Change

Update the history-scope `$effect` in `src/routes/analysis/+page.svelte` to read
only `historyScopeParams` synchronously and call the explicit loader:

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

The current `untrack(() => loadRuns())` wrapper should be removed from this
history effect as part of the refactor. Once `loadRunsForScope(params)` avoids
`deps.getState()`, the effect's only intended reactive read is the explicit
`historyScopeParams` read above, so `untrack` is no longer needed there.

If a local route wrapper is preferred for consistency, it must accept `params`
as an argument and must not call the broad `loadRuns()` wrapper from inside the
effect.

## Current Call Sites To Preserve

The existing `loadRuns()` wrapper is still useful outside Svelte tracking
contexts:

- refresh actions from the analysis UI;
- source sync and source-management refresh flows;
- `deleteSavedRun()` after deleting a saved run;
- `handleRunEvent()` after completed / failed / cancelled run events.

Those call sites do not need to pass scope manually unless a later refactor makes
that clearer.

## Tests

Add or update tests before implementation:

1. `analysis-run-workflow.test.ts`
   - `loadRunsForScope(params)` loads saved runs using the provided params.
   - `loadRunsForScope(null)` clears saved runs and does not call `listRuns`.
   - `loadRunsForScope(...)` does not call `deps.getState()`; use a throwing or
     counted `getState` mock for this test.
   - existing `loadRuns()` tests continue to cover the convenience wrapper.

2. `analysis-route-effects.test.ts`
   - update the guard so the history effect uses the explicit-scope loader.
   - reject direct `void loadRuns();` in the history effect.
   - reject calling the broad wrapper from the history effect.
   - no longer require `untrack` for this history effect once the explicit loader
     is in place.

3. Run:

```powershell
npm.cmd test -- analysis
npm.cmd run check
git diff --check
```

## Acceptance Criteria

- The history-scope `$effect` no longer calls the broad saved-run loader wrapper.
- `loadRunsForScope(params)` has no hidden `deps.getState()` dependency.
- The history-scope `$effect` no longer needs or uses `untrack`.
- Existing saved-run refresh behavior is unchanged from the user's perspective.
- Analysis tests and Svelte check pass.
- The WebView OOM investigation report remains accurate; update it if the code
  shape changes enough to make the current fixed snippet stale.

## Non-Goals

- Do not introduce a general static analyzer for `$effect` usage.
- Do not refactor unrelated analysis workflows.
- Do not remove `untrack` from unrelated code paths. This task only removes the
  history-effect `untrack` if `loadRunsForScope(params)` has no hidden
  `deps.getState()` dependency and the regression guard is updated accordingly.
