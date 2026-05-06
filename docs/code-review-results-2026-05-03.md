# Code Review Results - 2026-05-03

## Scope

This review covered the whole Extractum codebase with security findings
intentionally out of scope. The review focus was maintainability, consistency,
extensibility, testability, and avoiding duplication.

CodeRabbit could not be used because `coderabbit --version` failed in this
environment with `Wsl/Service/E_ACCESSDENIED`, so the results below are from a
manual review.

## Consolidated Resolved Work

The following maintainability work is complete in the cleanup history or current
cleanup branch:

- Analysis run loading, opening, and run-event orchestration were extracted from
  `src/routes/analysis/+page.svelte` into a tested workflow controller.
- Core source workflows in `/analysis` now call `$lib/api/sources` instead of
  raw core source Tauri commands.
- Source UI domain objects now use camelCase fields, and raw source DTO mapping
  is centralized in `src/lib/api/sources.ts`.
- `get_items` was replaced by the registered `list_source_items` command.
- Source request DTOs use camelCase Tauri wire fields.
- Telegram source-kind validation is centralized.
- Source command and service boundaries use explicit `AppError` constructors
  for source-local user-visible failures.
- Repeated source SQLite test setup is consolidated in
  `src-tauri/src/sources/test_support.rs`.
- Takeout import command/event access is centralized in
  `src/lib/api/takeout-import.ts`.
- NotebookLM export command/event access is centralized in
  `src/lib/api/notebooklm-export.ts`.
- Analysis chat command/event access and route-level orchestration are
  centralized in `src/lib/api/analysis-chat.ts` and
  `src/lib/analysis-chat-workflow.ts`.
- Analysis trace command access and route-level orchestration are centralized in
  `src/lib/api/analysis-trace.ts` and
  `src/lib/analysis-trace-workflow.ts`.
- Analysis account/status loading and analysis source metrics command access are
  centralized in `src/lib/api/analysis-workspace.ts` and
  `src/lib/analysis-workspace-workflow.ts`.
- Analysis source group loading and template/group deletion command access and
  route-level orchestration are centralized in
  `src/lib/api/analysis-source-groups.ts` and
  `src/lib/analysis-source-groups-workflow.ts`.

Historical Superpowers plan/spec files for these completed workstreams were
removed after this consolidation. Future files under `docs/superpowers/plans`
and `docs/superpowers/specs` should represent only active work.

Deferred by design:

- Rust-to-TypeScript type generation.
- Secure secret storage, as a separate security backlog item.

## Open Findings

### Major: Analysis route still owns several non-run workflows

`src/routes/analysis/+page.svelte` is smaller than at the start of the review,
but it still coordinates several feature areas directly. The remaining route
responsibilities include source group editing, template deletion, report
start/cancel/delete actions, listener lifecycle, and UI composition.

Current raw `/analysis` command surfaces found in the route:

```text
start_analysis_report
cancel_analysis_run
delete_analysis_run
```

Impact:

- remaining feature areas are still difficult to test in isolation;
- unrelated workflow state can still be touched by future analysis-page changes;
- the route remains a high-context file for new analysis features.

Suggested follow-up:

- extract analysis source group, template deletion, and report action
  surfaces in similarly small slices;
- keep the route as a composition and Svelte lifecycle layer;
- add focused tests around extracted wrappers/controllers before broader UI
  refactors.

### Moderate: Remaining non-source frontend/backend contracts are manually mirrored

Core source command strings and DTO mapping are centralized in
`src/lib/api/sources.ts`, and compact frontend API wrappers now exist for
analysis runs, Analysis chat, Analysis trace, Analysis workspace loading,
Takeout import, NotebookLM export, and LLM cancellation.

Several remaining frontend TypeScript DTOs and raw Tauri command strings are
still manually maintained beside Rust serde structs. Current notable raw
`/analysis` command surfaces are report start/cancel/delete actions.

Impact:

- DTO drift can still become silent runtime breakage on the remaining raw
  command surfaces;
- the remaining raw command names are harder to search and refactor safely;
- route files can still carry infrastructure detail they do not need.

Suggested fix:

- introduce typed `$lib/api/*` wrappers for the remaining compact Tauri command
  surfaces;
- move any remaining route-local DTOs to shared frontend type modules when they
  become shared across wrappers/controllers;
- later consider generated TypeScript types from Rust if drift remains a
  recurring problem.

### Moderate: Error typing is still partial outside source boundaries

Core source command and service boundaries now use explicit typed `AppError`
constructors for source-local user-visible failures. Elsewhere, the backend
still exposes `AppError` while some lower-level helpers return `Result<T,
String>`. `src-tauri/src/error.rs` also still classifies arbitrary strings into
error kinds by substring matching for compatibility paths.

Impact:

- outside the tightened source paths, changing wording can still change the
  frontend-visible error kind;
- tests for some non-source failure modes are weaker than the apparent typed API
  suggests;
- behavior is harder to reason about across DB, Telegram, LLM, and validation
  paths that have not been tightened yet.

Suggested fix:

- keep `AppError` at command/service boundaries;
- add small typed conversion helpers for remaining DB, Telegram, LLM, and
  validation paths;
- reduce reliance on message heuristics over time.

## Recent Verification

Recent verification from the completed Analysis source groups and template
deletion wrapper/controller workstream:

- route cleanup search found no raw `list_analysis_source_groups`,
  `delete_analysis_prompt_template`, or `delete_analysis_source_group` command
  strings in `src/routes/analysis/+page.svelte`;
- focused tests passed for `analysis-source-groups` and
  `analysis-source-groups-workflow`;
- `npm.cmd test` passed with 21 test files and 156 tests;
- `npm.cmd run check` passed with 0 errors and 0 warnings;
- `git diff --check` passed with exit code 0.

## Recommended Follow-Up Order

1. Extract wrappers/controllers for report start/cancel/delete actions.
2. Improve typed error conversion for remaining DB, Telegram, LLM, and
   validation paths.
