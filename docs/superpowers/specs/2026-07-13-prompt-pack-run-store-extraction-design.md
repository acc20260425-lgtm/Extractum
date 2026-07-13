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

Add a private sibling module:

```text
src-tauri/src/prompt_packs/
|-- mod.rs
|-- run_store.rs       # run catalog persistence and read models
|-- runtime.rs         # Tauri adapters and execution orchestration
`-- store.rs           # prompt-pack version lookup; unchanged
```

Register it in `prompt_packs/mod.rs` as `pub(crate) mod run_store;` or the
narrower private equivalent supported by all existing callers. It is not
re-exported from `prompt_packs` and does not create a frontend-visible API.

`runtime.rs` continues to own every Tauri command. Commands obtain the managed
SQLite pool and delegate storage work to functions imported from
`super::run_store`.

## Extraction Boundary

Move these functions and types to `run_store.rs` without changing their SQL or
behavior:

- `list_prompt_pack_runs_in_pool`;
- `update_prompt_pack_run_in_pool`;
- `delete_prompt_pack_run_in_pool`;
- `list_prompt_pack_run_stages_in_pool`;
- `load_run_summary_optional`;
- `normalize_prompt_pack_run_label`;
- `RunSummaryRow`;
- `impl From<RunSummaryRow> for PromptPackRunSummaryDto`.

The pool-level functions used by runtime code or existing tests remain
`pub(crate)`. Helpers that have no external caller remain private to
`run_store.rs`. Visibility must be chosen per actual caller rather than making
the entire module surface public.

The following remain in `runtime.rs`:

- `list_prompt_pack_runs`, `list_active_prompt_pack_runs`, and
  `list_prompt_pack_run_stages` Tauri commands;
- `update_prompt_pack_run` and `delete_prompt_pack_run` Tauri commands;
- run state, cancellation, execution, events, cleanup, and shutdown behavior;
- development cancellation smoke commands and their fixture helpers;
- `now_string`, because it is used broadly throughout runtime execution and
  is not a run-store-specific primitive.

The dev fixture helpers may call `run_store::load_run_summary_optional`, but
they are not moved in this slice because they coordinate both database rows
and `PromptPackRunState`.

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
updates, deletion guards, and missing rows. They may remain in the runtime test
module while the first extraction is performed, importing the pool-level
functions from `run_store`, or move into an in-file `run_store` test module if
their fixtures can move without duplicating large runtime setup. Prefer moving
tests that exercise only storage behavior; do not move mixed lifecycle tests
solely to maximize line-count reduction.

### Source contract

Add or extend a focused source-level contract that verifies:

- `prompt_packs/mod.rs` registers `run_store`;
- `runtime.rs` imports the extracted storage functions;
- `run_store.rs` owns the `RunSummaryRow` mapping and run-catalog SQL;
- the extracted function definitions no longer exist in `runtime.rs`.

The contract should check stable ownership markers, not exact formatting or
line counts. SQL queries remain behaviorally covered by Rust tests.

### Verification commands

Run, at minimum:

1. focused tests for the moved run-store behaviors;
2. all `prompt_packs::runtime::tests` and any new `run_store::tests`;
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

1. Run-catalog SQL and `RunSummaryRow` mapping live in `run_store.rs`.
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
