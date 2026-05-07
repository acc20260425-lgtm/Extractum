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
- Telegram account and authentication command access is centralized in
  `src/lib/api/accounts.ts`; the Accounts and Auth routes no longer invoke
  those Tauri commands directly.
- Analysis source group loading and template/group deletion command access and
  route-level orchestration are centralized in
  `src/lib/api/analysis-source-groups.ts` and
  `src/lib/analysis-source-groups-workflow.ts`.
- Analysis prompt-template and source-group create/update command access and
  route-level orchestration are centralized in
  `src/lib/api/analysis-source-groups.ts` and
  `src/lib/analysis-source-groups-workflow.ts`.
- Analysis report start/cancel/delete command access and route-level
  orchestration are centralized in `src/lib/api/analysis-runs.ts` and
  `src/lib/analysis-run-workflow.ts`.
- Boundary-first typed error conversion is complete for the remaining DB,
  Telegram, LLM, and validation command boundaries. Shared helpers now cover
  database, Telegram transport, and LLM network failures while preserving the
  existing `{ kind, message }` frontend wire shape.
- Shared frontend wrapper input contracts for Accounts, Analysis run/chat/source
  group/template, LLM, and source command wrappers now live in domain type
  modules under `src/lib/types/*`; API wrappers no longer export those public
  input interfaces, while wrapper tests continue to pin command payload shapes.
- Obsolete Superpowers plan/spec handoff artifacts for completed cleanup
  workstreams were removed; the current cleanup state now lives in this review
  document, the session handoff, and Git history.

Superpowers plan/spec directories should contain active work only. Completed
cleanup sequencing is preserved in Git history and this review/handoff pair
instead of stale task files.

Deferred by design:

- Rust-to-TypeScript type generation.
- Broad response/event DTO consolidation; the latest pass intentionally
  centralized wrapper input contracts only.
- Secure secret storage, as a separate security backlog item.

## Open Findings

### Major: Analysis route remains a high-context composition surface

`src/routes/analysis/+page.svelte` is smaller than at the start of the review,
and the remaining source group/template editor workflows are now delegated to
the analysis source-groups workflow. The route still owns listener lifecycle,
local Svelte state binding, and UI composition for the Analysis page.

Impact:

- lifecycle and composition changes still require care because the route is a
  broad integration point;
- unrelated UI state can still be touched by future analysis-page changes;
- the route remains a high-context file for new Analysis UI features.

Suggested follow-up:

- keep future changes routed through the existing API and workflow boundaries;
- keep the route as a composition, state binding, and Svelte lifecycle layer;
- only extract listener lifecycle later if it becomes a concrete source of
  defects or test friction.

### Moderate: Remaining response/event frontend/backend DTOs are manually mirrored

Core source command strings and DTO mapping are centralized in
`src/lib/api/sources.ts`, and compact frontend API wrappers now exist for
analysis runs, Analysis chat, Analysis trace, Analysis workspace loading,
Analysis source groups/templates, Takeout import, NotebookLM export, report
start/cancel/delete actions, Telegram accounts/authentication, and LLM
cancellation. A route-level raw Tauri command search now returns no matches
under `src/routes`.

Shared wrapper input contracts for Accounts, Analysis run/chat/source
group/template, LLM, and source wrapper commands are centralized in
`src/lib/types/*`. `AnalysisReportStartCommand.profileId` now matches the Rust
`Option<String>` command boundary as `string | null`.

Several frontend response/event DTOs are still manually maintained beside Rust
serde structs.

Impact:

- response/event DTO drift can still become silent runtime breakage on manually
  mirrored non-source contracts;
- wrapper-local command payloads still need focused tests whenever Rust command
  structs change;
- route files are cleaner, but frontend/backend contract drift remains a
  possible maintenance cost.

Suggested fix:

- audit remaining manually mirrored response/event DTOs only when they show real
  sharing or drift risk;
- keep route files free of raw command access as new command surfaces are added;
- later consider generated TypeScript types from Rust if drift remains a
  recurring problem.

### Low: Some lower-level string errors remain by design

The DB, Telegram, LLM, and validation command boundaries now use explicit typed
`AppError` mappings. A few lower-level and event-oriented paths still keep
`Result<T, String>` intentionally, including LLM streamed event payloads and
compatibility fallbacks through `From<String>` / `classify_message`.

Impact:

- lower-level wording can still matter if a future command boundary forwards a
  string through the compatibility classifier;
- streaming/event payloads remain plain text, so they need small explicit
  conversions when called from typed boundaries;
- broader removal of internal string errors would be a hardening pass rather
  than a current correctness blocker.

Suggested fix:

- keep new command/service boundaries on explicit `AppError` constructors;
- when touching lower-level helpers, avoid introducing new command-facing
  `Result<T, String>` paths;
- reduce `classify_message` fallback reliance opportunistically.

## Recent Verification

Recent verification from the completed boundary-first typed error conversion,
Analysis editor workflow extraction, and Telegram account API wrapper
workstreams:

- focused Cargo checks passed during implementation:
  `cargo test error`, `cargo test accounts`, `cargo test analysis`,
  `cargo test telegram`, and `cargo test llm`;
- focused frontend checks passed during the editor workflow extraction:
  `npm.cmd test -- src/lib/api/analysis-source-groups.test.ts`,
  `npm.cmd test -- src/lib/analysis-source-groups-workflow.test.ts`, and
  `npm.cmd run check`;
- focused frontend checks passed during the Telegram account API wrapper
  workstream:
  `npm.cmd test -- src/lib/api/accounts.test.ts`,
  `npm.cmd test -- src/lib/api/accounts.test.ts src/lib/api/analysis-workspace.test.ts`,
  and `npm.cmd run check`;
- focused frontend checks passed during the wrapper input contract audit:
  `npm.cmd test -- src/lib/api/accounts.test.ts src/lib/api/analysis-runs.test.ts src/lib/api/analysis-chat.test.ts src/lib/api/analysis-source-groups.test.ts src/lib/api/llm.test.ts src/lib/api/sources.test.ts`,
  `npm.cmd run check`, route-level raw Tauri command search under
  `src/routes`, and `git diff --check`;
- full frontend test suite also passed for the latest wrapper input contract
  audit: `npm.cmd test` with 22 test files and 187 tests;
- docs cleanup verification for the latest refresh is recorded in
  `docs/session-context-2026-05-03.md`.

## Recommended Follow-Up Order

1. Audit remaining manually mirrored response/event DTOs only when drift recurs
   or shared usage makes consolidation worthwhile.
2. Opportunistically reduce lower-level `Result<T, String>` and
   `classify_message` fallback reliance when touching nearby backend code.
