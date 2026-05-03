# Code Review Results - 2026-05-03

## Scope

This review covered the whole Extractum codebase with security findings intentionally out of scope.
The review focus was maintainability, consistency, extensibility, testability, and avoiding duplication.

The repository was clean at the start of the review. CodeRabbit could not be used because
`coderabbit --version` failed in this environment with `Wsl/Service/E_ACCESSDENIED`, so the results
below are from a manual review.

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

### Major: Large backend modules mix unrelated behavior

`src-tauri/src/sources.rs` mixes DTOs, sync settings, avatar cache, Telegram peer resolution, sync,
forum topic refresh, DB row mapping, and tests.

Takeout import has been split into focused `state`, `pagination`, and `export_dc` modules under
`src-tauri/src/takeout_import/`. The remaining Takeout orchestration in `mod.rs` is intentional for
this first slice, with peer validation and history import flow still owned by the facade.

Impact:

- larger change blast radius;
- more difficult targeted tests;
- more context required for safe edits.

Suggested fix:

- split by existing behavior boundaries, not by abstract architecture;
- next likely modules: `sources/peer_resolution`, `sources/sync`, `sources/items`, and
  `sources/settings`.

Planning status:

- first Takeout implementation slice is documented in
  `docs/superpowers/plans/2026-05-03-takeout-import-backend-split.md` and has been implemented;
- `sources.rs` split is documented in
  `docs/superpowers/plans/2026-05-03-sources-backend-split.md` and has been implemented;
- any remaining Takeout orchestration in `mod.rs` is intentional for this first pass.

### Major: Some frontend/backend contracts remain manually mirrored

Several frontend TypeScript DTOs and raw Tauri command/event strings are still manually maintained
beside Rust serde structs.

Impact:

- DTO drift can become silent runtime breakage;
- command and event names are harder to search and refactor safely;
- route files can still carry infrastructure detail they do not need.

Suggested fix:

- introduce typed `$lib/api/*` wrappers for the remaining compact Tauri command/event surfaces;
- move route-local DTOs to shared frontend type modules;
- later consider generated TypeScript types from Rust if drift remains a recurring problem.

### Major: Error typing is only partial

The backend exposes `AppError`, but many lower-level helpers still return `Result<T, String>`.
`src-tauri/src/error.rs` also classifies arbitrary strings into error kinds by substring matching.

Impact:

- changing wording can change the frontend-visible error kind;
- tests for failure modes are weaker than the apparent typed API suggests;
- behavior is harder to reason about across DB, Telegram, LLM, and validation paths.

Suggested fix:

- keep `AppError` at command/service boundaries;
- add small typed conversion helpers for DB, Telegram, LLM, and validation paths;
- reduce reliance on message heuristics over time.

## Recent Verification

- `npm.cmd test`: passed with 10 test files and 97 tests.
- `npm.cmd run check`: passed with 0 errors and 0 warnings when run outside the sandbox so
  Vite/esbuild could spawn.
- `git diff --check`: passed with no real whitespace errors. Windows LF-to-CRLF warnings may appear
  for edited Markdown files in this worktree.

## Recommended Follow-Up Order

1. Extract the remaining non-run analysis route controllers/helpers.
2. Add typed wrappers for the next compact Tauri command/event surface.
3. Split `sources.rs` only along behavior boundaries already covered by tests.
4. Improve typed error conversion for DB, Telegram, LLM, and validation paths.
5. Continue with secure secret storage as a separate backlog item, not mixed into stabilization work.
