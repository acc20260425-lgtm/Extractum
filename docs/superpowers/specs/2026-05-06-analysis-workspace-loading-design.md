# Analysis Workspace Loading Design

## Goal

Extract the remaining read-only `/analysis` workspace loading command surface
from `src/routes/analysis/+page.svelte` into typed frontend wrappers and a small
dependency-injected workflow.

## Scope

This slice covers only:

- `list_accounts`
- `tg_get_account_statuses`
- `list_analysis_sources`

It does not cover source groups, template deletion, report start/cancel/delete
actions, source topics/items, or Takeout/NotebookLM workflows.

## Architecture

Add `$lib/api/analysis-workspace.ts` for compact Tauri command wrappers:

- `listWorkspaceAccounts()`
- `getWorkspaceAccountStatuses(accountIds: number[])`
- `listAnalysisSources()`

Add `$lib/analysis-workspace-workflow.ts` for route-level orchestration that is
still framework-independent and testable. The workflow receives dependencies for
state access, patching, source catalog loading, account/status loading, analysis
source metrics loading, and error formatting.

The Svelte route remains responsible for wiring `$state` variables into the
workflow and applying patches. It should no longer import or call these raw
Tauri command names directly.

## Data Flow

Account loading:

1. Load accounts.
2. Patch `accounts`.
3. If no accounts exist, patch `accountStatuses` to `{}` and stop.
4. Load statuses for the returned account ids.
5. Patch `accountStatuses` as a record keyed by `account_id`.
6. On error, patch `status` with `formatError("loading workspace accounts", error)`.

Source catalog loading:

1. Patch `loadingSourceCatalog: true`.
2. Load all source records with existing `$lib/api/sources.listSources(null)`.
3. Load analysis source metrics with `listAnalysisSources()`.
4. Patch `sourceCatalog` and `sourceMetrics`.
5. Preserve the current selected source id when it still exists.
6. Select the first analysis source id when nothing is selected.
7. Fall back to the first source id when the selected source no longer exists.
8. Clear the selection when no sources exist.
9. On error, patch `status` with `formatError("loading workspace sources", error)`.
10. Always patch `loadingSourceCatalog: false`.

## Error Handling

The workflow does not interpret backend error shapes. It delegates all formatting
to the injected `formatError` dependency, matching existing analysis workflow
patterns.

## Testing

Use TDD:

- API wrapper tests verify command names and argument shapes for all three
  commands.
- Workflow tests verify empty-account handling, status record mapping, account
  error reporting, source metric mapping, selected source preservation/fallback,
  and source-loading error/loading state behavior.

## Success Criteria

- `src/routes/analysis/+page.svelte` no longer contains raw command strings for
  `list_accounts`, `tg_get_account_statuses`, or `list_analysis_sources`.
- Focused wrapper and workflow tests pass.
- Full frontend tests and Svelte check pass before the workstream is reported
  complete.
