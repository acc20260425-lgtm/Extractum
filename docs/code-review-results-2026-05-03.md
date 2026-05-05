# Code Review Results - 2026-05-03

## Scope

This review covered the whole Extractum codebase with security findings intentionally out of scope.
The review focus was maintainability, consistency, extensibility, testability, and avoiding duplication.

The repository was clean at the start of the review. CodeRabbit could not be used because
`coderabbit --version` failed in this environment with `Wsl/Service/E_ACCESSDENIED`, so the results
below are from a manual review.

## Contract V2 Update

Source Contract V2 is complete on branch `sources-contract-v2`.

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

- NotebookLM export frontend API wrapper;
- Rust-to-TypeScript type generation.

## Frontend Wrapper Planning Update - 2026-05-05

Takeout import frontend API wrapping is now complete and merged into `main`.
The next selected workstream is a matching NotebookLM export frontend API
wrapper:

- design: `docs/superpowers/specs/2026-05-05-notebooklm-export-frontend-wrapper-design.md`;
- plan: `docs/superpowers/plans/2026-05-05-notebooklm-export-frontend-wrapper.md`.

The NotebookLM workstream is intentionally wrapper-only. It centralizes
`export_source_to_notebooklm` and `notebooklm://export` in
`$lib/api/notebooklm-export.ts`, while leaving backend Rust code, DTO field
names, the folder picker, and route lifecycle state unchanged.

## Open Findings

### Major: Analysis route still owns several non-run workflows

`src/routes/analysis/+page.svelte` has been reduced by extracting analysis run loading, opening, and
run-event orchestration into a tested route-local workflow controller.

The remaining route responsibilities still include source/group/template editing, chat orchestration,
NotebookLM export, Takeout job state, trace presentation state, listener lifecycle, and UI composition.

Impact:

- remaining feature areas are still difficult to test in isolation;
- unrelated workflow state can still be touched by future analysis-page changes;
- the route remains a high-context file for new analysis features.

Suggested follow-up:

- extract focused controllers/helpers for chat, sources, Takeout import, and NotebookLM export;
- keep the route as a composition and Svelte lifecycle layer;
- add focused tests around extracted pure reducers/controllers before broader UI refactors.

### Moderate: Some non-source frontend/backend contracts remain manually mirrored

Core source command strings and DTO mapping are now centralized in
`src/lib/api/sources.ts`, and source UI code uses camelCase domain types.
Several non-source frontend TypeScript DTOs and raw Tauri command/event strings
are still manually maintained beside Rust serde structs.

Impact:

- DTO drift can still become silent runtime breakage outside core sources;
- non-source command and event names are harder to search and refactor safely;
- route files can still carry non-source infrastructure detail they do not need.

Suggested fix:

- introduce typed `$lib/api/*` wrappers for NotebookLM export and other compact
  Tauri command/event surfaces;
- move remaining route-local DTOs to shared frontend type modules;
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

- `cargo test sources --lib`: passed with 41 tests.
- `cargo test`: passed with 141 tests.
- `npm.cmd test`: passed with 11 test files and 102 tests when run outside the
  sandbox so Vite/esbuild could spawn.
- `npm.cmd run check`: passed with 0 errors and 0 warnings when run outside the
  sandbox so Vite/esbuild could spawn.
- `git diff --check`: passed with no real whitespace errors.

## Recommended Follow-Up Order

1. Add the planned typed wrapper for NotebookLM export.
2. Extract the remaining non-run analysis route controllers/helpers.
3. Improve typed error conversion for remaining DB, Telegram, LLM, and validation paths.
4. Continue with secure secret storage as a separate backlog item, not mixed into stabilization work.
