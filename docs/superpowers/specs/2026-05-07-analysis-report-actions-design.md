# Analysis Report Actions Cleanup Design

## Goal

Extract the remaining raw `/analysis` report action command surface from
`src/routes/analysis/+page.svelte` into typed frontend API wrappers and the
existing analysis run workflow boundary.

The cleanup covers exactly these Tauri commands:

```text
start_analysis_report
cancel_analysis_run
delete_analysis_run
```

## Current State

`src/lib/api/analysis-runs.ts` already owns typed access for listing active and
saved runs, loading run details, and listening to `analysis://run` events.

`src/lib/analysis-run-workflow.ts` already owns orchestration for loading run
history, loading active runs, opening runs, reacting to run events, and guarding
stale detail/chat/trace loads.

`src/routes/analysis/+page.svelte` still directly:

- validates and starts a report with `analysisReportStartCommand`;
- invokes `start_analysis_report`;
- creates the queued live-run entry and focuses the new run;
- invokes `cancel_analysis_run`;
- validates, confirms, invokes, and applies state updates for
  `delete_analysis_run`.

## Recommended Approach

Extend the existing run API and workflow instead of creating a parallel report
actions module.

This keeps one frontend boundary for analysis run commands, avoids a second
controller that would share the same state, and lets route code delegate all run
loading/opening/action behavior through `createAnalysisRunWorkflow`.

## Alternatives Considered

1. Extend `analysis-runs` and `analysis-run-workflow`.

   This is the preferred path. The command surface belongs to the same domain as
   list/open/run-event behavior, and the existing workflow already has the
   dependencies needed to refresh active/saved runs and open run details.

2. Create a new `analysis-report-actions-workflow.ts`.

   This would reduce the size of `analysis-run-workflow.ts`, but it would need
   many of the same dependencies and would split run lifecycle orchestration
   across two controllers. That adds coordination cost without a clear behavior
   boundary.

3. Add only API wrappers and leave orchestration in the route.

   This removes raw command strings but leaves the harder-to-test state mutation
   and destructive action flow in `+page.svelte`. It does not resolve the review
   concern that the route coordinates report actions directly.

## API Boundary

Add these wrappers to `src/lib/api/analysis-runs.ts`:

- `startAnalysisReport(command: AnalysisReportStartCommand): Promise<number>`
- `cancelAnalysisRun(runId: number): Promise<void>`
- `deleteAnalysisRun(runId: number): Promise<void>`

`AnalysisReportStartCommand` currently lives in `src/lib/analysis-state.ts`.
The implementation may either import that type into the API wrapper or move it
to `src/lib/types/analysis.ts` if the plan decides that shared command DTOs
should live with the rest of the analysis DTOs. The implementation should avoid
duplicating the command shape.

## Workflow Boundary

Extend `createAnalysisRunWorkflow` with:

- `startReport(input: AnalysisReportStartState): Promise<void>`
- `cancelRun(runId: number): Promise<void>`
- `deleteSavedRun(run: AnalysisRunSummary): Promise<void>`

The workflow should receive new dependencies for command execution and UI-side
effects:

- `startReport(command: AnalysisReportStartCommand): Promise<number>`
- `cancelRun(runId: number): Promise<void>`
- `deleteRun(runId: number): Promise<void>`
- `confirm(options: RunDeletionDialog): Promise<boolean>`
- `clearOpenedRunState(runId: number): void`
- `clearChatState(): void`
- `clearTraceState(): void`
- `setInitialLiveRun(runId: number): void`

The existing `cancelChatSilently`, `loadRuns`, `loadActiveRuns`, `openRun`, and
`formatError` behavior should be reused where possible.

## Behavior

### Starting a Report

`startReport` should:

1. Use `analysisReportStartCommand` to validate the selected scope, template,
   date range, language, and optional model override.
2. Patch `status` and return when validation fails.
3. Patch `startingReport: true` and `inspectorMode: "active"` before invoking.
4. Cancel the active chat silently when one is in progress.
5. Clear chat state, trace state, and the currently opened run before starting.
6. Call `startReport(command)` and receive the new run id.
7. Create an initial queued live-run state for the new id.
8. Patch `activeRunId` to the new id.
9. Refresh active runs and open the new run.
10. Format errors as `formatError("starting the analysis report", error)`.
11. Always patch `startingReport: false` when complete.

### Cancelling a Run

`cancelRun` should:

1. Call `cancelRun(runId)`.
2. Patch `status` to `Cancelling analysis run ${runId}...` on success.
3. Format errors as `formatError("cancelling the analysis run", error)`.

### Deleting a Saved Run

`deleteSavedRun` should:

1. Use `runDeletionDecision(run)` to block active runs before confirmation.
2. Patch `status` and return when deletion is blocked.
3. Ask for confirmation with the existing dialog text and danger tone.
4. Patch `deletingRunIds` to mark the run as pending.
5. Cancel active chat silently if the active chat belongs to the run being
   deleted.
6. Call `deleteRun(run.id)`.
7. Remove the run from saved and active run lists.
8. Clear opened run/chat/trace/live state when the deleted run is focused.
9. Patch `inspectorMode: "history"` and `status` from `runDeletedStatus(run)`.
10. Reload saved runs.
11. Format errors as `formatError("deleting the saved run", error)`.
12. Always clear the deletion pending flag.

## Route Integration

`src/routes/analysis/+page.svelte` should stop importing `invoke` for these
three commands. It should delegate:

- `runReport()` to `runWorkflow.startReport(...)`;
- `cancelActiveRun(runId)` to `runWorkflow.cancelRun(runId)`;
- `deleteSavedRun(run)` to `runWorkflow.deleteSavedRun(run)`.

The route may keep thin wrappers for event handler readability, but those
wrappers should not contain command strings or action orchestration.

## Testing

Follow TDD during implementation.

Focused tests should cover:

- API wrapper command names and argument shapes for start/cancel/delete.
- `startReport` validation failure without invoking the API.
- Successful `startReport` state transitions, live-run initialization, active
  run reload, and open-run behavior.
- `startReport` error formatting and loading cleanup.
- `cancelRun` success and error statuses.
- `deleteSavedRun` blocked active-run deletion without confirmation.
- `deleteSavedRun` confirmation cancellation without invoking the API.
- Successful `deleteSavedRun` state cleanup, list updates, opened-run reset, run
  reload, and pending flag cleanup.
- `deleteSavedRun` error formatting and pending flag cleanup.

The route cleanup should be verified with:

```text
rg "start_analysis_report|cancel_analysis_run|delete_analysis_run" src/routes/analysis/+page.svelte
```

The command should return no output after route wiring is complete.

## Non-Goals

- Do not change Rust command behavior.
- Do not change report generation semantics.
- Do not redesign analysis run event handling.
- Do not extract template/group create-update actions in this workstream.
- Do not introduce generated TypeScript types in this workstream.
