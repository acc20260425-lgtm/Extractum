# Prompt Pack Run Store Extraction Design

**Status:** Proposed for implementation planning
**Date:** 2026-07-13

## Goal

Reduce the size and mixed responsibilities of
`src-tauri/src/prompt_packs/runtime.rs` by extracting its run-catalog SQLite
operations into a focused private module. Preserve all runtime behavior,
Tauri commands, SQL, DTOs, persisted values, and public interfaces.

## Current Problem

`prompt_packs/runtime.rs` is approximately 3,270 lines and currently combines
several responsibilities:

- Tauri command adapters and application-state access;
- run lifecycle and cancellation orchestration;
- API and browser LLM execution;
- prompt construction and token-budget calculation;
- run-catalog SQL queries and row-to-DTO mapping;
- development smoke-fixture management;
- a large in-file test module.

The run-catalog SQL block is a useful first extraction because it already has
a coherent data boundary and does not own execution lifecycle. Keeping it in
`runtime.rs` makes the runtime module harder to navigate and encourages future
SQL additions to accumulate beside unrelated orchestration code.

## Selected Architecture

Add a private sibling module. This diagram is a fragment of the existing
`prompt_packs` tree, not its complete contents:

```text
src-tauri/src/prompt_packs/
|-- mod.rs
|-- run_store.rs       # run catalog persistence and read models
|-- runtime.rs         # Tauri adapters and execution orchestration
|-- ...                # dto, projections, youtube_summary, and other modules
`-- store.rs           # prompt-pack version lookup; unchanged
```

Register it in `prompt_packs/mod.rs` as private `mod run_store;`. It is not
re-exported from `prompt_packs` and does not create a frontend-visible API.

`runtime.rs` continues to own every Tauri command. Commands obtain the managed
SQLite pool and delegate storage work to functions imported from
`super::run_store`.

## Extraction Boundary

Move this catalog read/manage API to `run_store.rs` without changing its SQL or
behavior:

- `list_prompt_pack_runs_in_pool`;
- `update_prompt_pack_run_in_pool`;
- `delete_prompt_pack_run_in_pool`;
- `list_prompt_pack_run_stages_in_pool`;
- `load_run_summary_optional`;
- `normalize_prompt_pack_run_label`;
- `RunSummaryRow`;
- `impl From<RunSummaryRow> for PromptPackRunSummaryDto`.

The five pool-level functions called by `runtime.rs` use `pub(super)`:
`list_prompt_pack_runs_in_pool`, `update_prompt_pack_run_in_pool`,
`delete_prompt_pack_run_in_pool`, `list_prompt_pack_run_stages_in_pool`, and
`load_run_summary_optional`. This intentionally narrows the three existing
`pub(crate)` functions and expands the two currently private functions only as
far as their new sibling-module caller requires. `normalize_prompt_pack_run_label`,
`RunSummaryRow`, and its DTO conversion remain private to `run_store.rs`.

The following remain in `runtime.rs`:

- `list_prompt_pack_runs`, `list_active_prompt_pack_runs`, and
  `list_prompt_pack_run_stages` Tauri commands;
- `update_prompt_pack_run` and `delete_prompt_pack_run` Tauri commands;
- run state, cancellation, execution, events, cleanup, and shutdown behavior;
- development cancellation smoke commands and their fixture helpers;
- `now_string`, because it is used broadly throughout runtime execution and
  is not a run-store-specific primitive.

`update_prompt_pack_run_in_pool` currently calls the runtime-local
`now_string()`. After extraction it calls `crate::time::now_rfc3339_utc()`
directly. This is the same implementation used by `now_string`; it avoids both
a reverse `run_store -> runtime` dependency and another duplicated timestamp
helper. Consolidating the other timestamp helpers remains a separate
follow-up.

The dev fixture helpers call `run_store::load_run_summary_optional`, but
they are not moved in this slice because they coordinate both database rows
and `PromptPackRunState`.

Lifecycle and execution SQL intentionally remains in `runtime.rs`, including
cancellation updates, stage provenance/status updates,
`mark_prompt_pack_run_failed`, interrupted-run cleanup, and dev-fixture
INSERT/DELETE operations. The new module owns only the enumerated catalog
read/manage API; it does not own every query that references
`prompt_pack_runs` or `prompt_pack_stage_runs`.

## Existing `store.rs`

Do not merge this extraction into `prompt_packs/store.rs`. That module has a
different responsibility: resolving prompt-pack definitions and versions.
Using a separate `run_store.rs` prevents it from becoming a generic container
for unrelated SQL helpers and gives future run persistence a clear owner.

## Data and API Compatibility

This is a structural refactor only:

- Tauri command names and parameters are unchanged;
- Rust command exports from `prompt_packs/mod.rs` are unchanged;
- DTO field names and serialization are unchanged;
- SQL text, ordering, limits, status checks, and error messages are unchanged;
- database schema and migrations are unchanged;
- status, state, kind, mode, phase, provider, and other registered string
  values are unchanged;
- no update to `docs/value-registry.md` is required.

The move must not introduce transactions, retries, caching, query
consolidation, timestamp changes, or altered label normalization. Those would
be behavioral changes and belong in separate designs.

## Error Handling

Preserve existing error behavior exactly:

- SQLx failures continue through `AppError::database`;
- missing runs continue to return the same not-found messages;
- deletion of queued or running runs continues to return the same conflict;
- empty or whitespace-only run labels continue to normalize to `None`.

The new module does not log errors. Tauri adapters and lifecycle code retain
responsibility for user-facing propagation or logging.

## Testing Strategy

### Behavioral tests

Keep the existing tests for listing, stage mapping, label normalization,
updates, deletion guards, and missing rows in `runtime::tests` for this slice.
Only their imports change to reference `run_store`. They share the runtime
module's substantial database fixture setup; moving or duplicating that setup
would make this structural diff wider without improving behavior coverage.
Test-module decomposition is a separate follow-up.

### Source contract

Add or extend a focused source-level contract that verifies:

- `prompt_packs/mod.rs` registers `run_store`;
- `runtime.rs` imports the extracted storage functions;
- `run_store.rs` owns `RunSummaryRow` and the five enumerated catalog
  read/manage functions;
- those five function definitions no longer exist in `runtime.rs`.

The contract must not assert that all SQL mentioning `prompt_pack_runs` or
`prompt_pack_stage_runs` moved out of `runtime.rs`; lifecycle and fixture SQL is
explicitly out of scope. It should check stable function/type ownership
markers, not exact formatting or line counts. SQL behavior remains covered by
Rust tests.

### Verification commands

Run, at minimum:

1. focused tests for the moved run-store behaviors;
2. all `prompt_packs::runtime::tests`;
3. the complete Rust test suite once;
4. `npm.cmd run check:rustfmt`;
5. `cargo check --manifest-path src-tauri/Cargo.toml --all-targets` with zero
   warnings.

## Scope Constraints

Implementation changes are limited to:

- `src-tauri/src/prompt_packs/mod.rs`;
- `src-tauri/src/prompt_packs/runtime.rs`;
- new `src-tauri/src/prompt_packs/run_store.rs`;
- one focused source-contract test if the repository's existing contract-test
  conventions require it.

Do not change frontend files, migrations, DTO definitions, Tauri command
registration, dependencies, prompt assets, or unrelated formatting.

## Acceptance Criteria

1. The five enumerated catalog read/manage functions and `RunSummaryRow`
   mapping live in `run_store.rs`.
2. `runtime.rs` retains Tauri and lifecycle orchestration but no longer defines
   the extracted storage functions or row type.
3. Public Tauri and Rust interfaces remain unchanged.
4. Existing storage and runtime behavior tests pass without weakened
   assertions.
5. The complete Rust suite passes.
6. Rust formatting and all-target compilation pass with zero warnings.
7. The implementation diff contains no persisted-value or behavioral change.

## Deferred Follow-ups

This slice does not attempt to fully decompose `runtime.rs`. Later independent
designs may consider:

- extracting LLM request construction and budget calculations;
- extracting API/browser provider execution adapters;
- separating the large runtime test module by responsibility;
- consolidating duplicated timestamp helpers across prompt-pack modules.

Each follow-up must preserve behavior and be reviewed as its own bounded
slice.
