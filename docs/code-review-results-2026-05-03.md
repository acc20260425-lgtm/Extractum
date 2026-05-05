# Code Review Results - 2026-05-03

## Scope

This review covered the whole Extractum codebase with security findings intentionally out of scope.
The review focus was maintainability, consistency, extensibility, testability, and avoiding duplication.

The repository was clean at the start of the review. CodeRabbit could not be used because
`coderabbit --version` failed in this environment with `Wsl/Service/E_ACCESSDENIED`, so the results
below are from a manual review.

## Contract V2 Update

Source Contract V2 is complete and merged into `main`.

Resolved for core sources:

- source workflows in `/analysis` now call `$lib/api/sources` instead of raw
  core source Tauri commands;
- core source UI domain objects now use camelCase fields;
- raw source DTO mapping is centralized in `src/lib/api/sources.ts`;
- `get_items` was replaced by the registered `list_source_items` command;
- source request DTOs use camelCase Tauri wire fields;
- Telegram source-kind validation is centralized;
- source command and service boundaries use explicit `AppError` constructors
  for source-local user-visible failures;
- repeated source SQLite test setup is consolidated in
  `src-tauri/src/sources/test_support.rs`.

Deferred by design:

- Rust-to-TypeScript type generation.

## Frontend Wrapper And Controller Update - 2026-05-05

Takeout import frontend API wrapping is now complete and merged into `main`.
NotebookLM export frontend API wrapping is also complete and merged into
`main`. Analysis chat API wrapping and route-level chat orchestration extraction
are also complete and merged into `main`:

- design: `docs/superpowers/specs/2026-05-05-notebooklm-export-frontend-wrapper-design.md`;
- plan: `docs/superpowers/plans/2026-05-05-notebooklm-export-frontend-wrapper.md`;
- chat plan: `docs/superpowers/plans/2026-05-05-analysis-chat-wrapper-controller.md`.

The NotebookLM workstream is intentionally wrapper-only. It centralizes
`export_source_to_notebooklm` and `notebooklm://export` in
`$lib/api/notebooklm-export.ts`, while leaving backend Rust code, DTO field
names, the folder picker, and route lifecycle state unchanged.

Resolved for Takeout import, NotebookLM export, and Analysis chat:

- `src/lib/api/takeout-import.ts` owns the `list_takeout_source_import_jobs`,
  `start_takeout_source_import`, and `cancel_takeout_source_import` commands
  plus the `sources://takeout-import` event name;
- `src/lib/api/notebooklm-export.ts` owns the
  `export_source_to_notebooklm` command and `notebooklm://export` event name;
- `src/lib/api/analysis-chat.ts` owns the `list_analysis_chat_messages`,
  `ask_analysis_run_question`, and `clear_analysis_chat_messages` commands
  plus the `analysis://chat` event name;
- `src/lib/analysis-chat-workflow.ts` owns chat loading, asking, cancellation,
  clearing, event handling, stale request guards, and state reset;
- wrapper/controller tests cover command payload shapes, event constants,
  listener forwarding, and route-level chat behavior;
- `/analysis` no longer owns Takeout import, NotebookLM export, or Analysis chat
  raw Tauri command/event strings.

## Open Findings

### Major: Analysis route still owns several non-run workflows

`src/routes/analysis/+page.svelte` has been reduced by extracting analysis run loading, opening, and
run-event orchestration into a tested route-local workflow controller. Analysis
chat command/event access and chat orchestration have also been extracted.

The remaining route responsibilities still include source/group/template
editing coordination, account/status loading, analysis source metrics loading,
report start/cancel/delete actions, NotebookLM export lifecycle/form state,
Takeout job state, trace loading/resolution and presentation state, listener
lifecycle, and UI composition.

Impact:

- remaining feature areas are still difficult to test in isolation;
- unrelated workflow state can still be touched by future analysis-page changes;
- the route remains a high-context file for new analysis features.

Suggested follow-up:

- extract focused wrappers/controllers for trace, account/status loading,
  analysis templates, analysis source groups, analysis report actions, Takeout
  job state, and NotebookLM export lifecycle;
- keep the route as a composition and Svelte lifecycle layer;
- add focused tests around extracted pure reducers/controllers before broader UI refactors.

### Moderate: Remaining non-source frontend/backend contracts are manually mirrored

Core source command strings and DTO mapping are centralized in
`src/lib/api/sources.ts`, and source UI code uses camelCase domain types. Compact
frontend API wrappers now also exist for analysis runs, Analysis chat, Takeout
import, NotebookLM export, and LLM cancellation.

Several remaining frontend TypeScript DTOs and raw Tauri command strings are
still manually maintained beside Rust serde structs. Current notable raw
`/analysis` command surfaces include trace loading/resolution, account/status
loading, analysis source metrics, analysis templates, analysis source groups,
and report start/cancel/delete actions.

Impact:

- DTO drift can still become silent runtime breakage on the remaining raw
  command surfaces;
- the remaining raw command names are harder to search and refactor safely;
- route files can still carry non-source infrastructure detail they do not need.

Suggested fix:

- introduce typed `$lib/api/*` wrappers for the remaining compact Tauri command
  surfaces;
- move any remaining route-local DTOs to shared frontend type modules;
- later consider generated TypeScript types from Rust if drift remains a recurring problem.

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

Recent verification from the completed Analysis chat wrapper/controller
workstream:

- route cleanup search found no raw Analysis chat command/event strings in
  `src/routes/analysis/+page.svelte`;
- `npm.cmd test -- analysis-chat analysis-chat-workflow analysis-chat-state`:
  passed with 3 test files and 23 tests;
- `npm.cmd test -- analysis-chat analysis-chat-workflow analysis-chat-state analysis-runs llm notebooklm-export takeout-import sources`:
  passed with 8 test files and 41 tests;
- `npm.cmd test`: passed with 15 test files and 124 tests;
- `npm.cmd run check`: passed with 0 errors and 0 warnings;
- `git diff --check`: passed with exit code 0.

## Recommended Follow-Up Order

1. Extract wrappers/controllers for the remaining raw `/analysis` command
   surfaces: trace, accounts/statuses, source metrics, templates, source groups,
   and report actions.
2. Extract NotebookLM export lifecycle/form state if export behavior is expected
   to keep growing beyond the current wrapper-only boundary.
3. Extract Takeout job state if the import workflow is expected to gain more
   route-local behavior.
4. Improve typed error conversion for remaining DB, Telegram, LLM, and
   validation paths.
5. Continue with secure secret storage as a separate backlog item, not mixed
   into stabilization work.
