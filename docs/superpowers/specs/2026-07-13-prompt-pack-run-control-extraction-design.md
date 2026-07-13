# Prompt Pack Run Control Extraction Design

**Status:** Proposed for implementation planning
**Date:** 2026-07-13

## Goal

Reduce the remaining mixed responsibilities in
`src-tauri/src/prompt_packs/runtime.rs` by extracting the in-memory Prompt Pack
run registry and cooperative cancellation policy into one small private
module. Preserve every public path, state transition, cancellation outcome,
error type, and runtime behavior.

## Current Problem

After the run-store and stage-request-policy extractions,
`prompt_packs/runtime.rs` still owns both execution orchestration and the
independent in-memory control state for active Prompt Pack runs. The control
block includes:

- the active-run set;
- the per-run cancellation-token registry;
- registration, cancellation, child-token, completion, and terminal-event
  cleanup methods;
- the generic future-versus-cancellation race helper.

These responsibilities do not require Tauri commands, SQLx, provider
execution, persistence, or browser orchestration. Keeping them in
`runtime.rs` obscures the command and execution flow and makes the run-control
contract harder to review in isolation.

## Selected Architecture

Add this private sibling module:

```text
src-tauri/src/prompt_packs/
|-- mod.rs
|-- run_control.rs             # active-run registry and cooperative cancellation
|-- run_store.rs               # run-catalog persistence
|-- runtime.rs                 # Tauri commands and execution orchestration
|-- stage_request_policy.rs    # prompt construction and token budgets
`-- ...
```

Register it in `prompt_packs/mod.rs` as exactly:

```rust
mod run_control;
```

Do not expose the module itself. Move `PromptPackRunState`, its complete
implementation, and `run_with_prompt_pack_run_cancellation` to
`run_control.rs`.

`PromptPackRunState` remains a public type because it is managed by Tauri and
already exposed through public module paths. `runtime.rs` preserves its
existing path with:

```rust
pub use super::run_control::PromptPackRunState;
```

The existing export from `prompt_packs/mod.rs` remains unchanged:

```rust
pub use runtime::{
    // existing command exports
    PromptPackRunState,
};
```

Therefore both existing paths remain valid:

- `prompt_packs::runtime::PromptPackRunState`;
- `prompt_packs::PromptPackRunState`.

The cancellation helper is not public API. Give it `pub(super)` visibility so
the sibling `runtime` module can call it:

```rust
pub(super) async fn run_with_prompt_pack_run_cancellation<Fut, T>(
    run_cancellation_token: Option<CancellationToken>,
    future: Fut,
) -> Result<T, LlmRequestError>
where
    Fut: Future<Output = Result<T, LlmRequestError>>,
```

`runtime.rs` imports this helper privately. Existing runtime tests remain in
place and may continue to reach the type and helper through the runtime import
surface; their test bodies do not move.

## Extraction Boundary

Move unchanged to `run_control.rs`:

- `PromptPackRunState`;
- its `Default` derivation;
- all existing methods:
  - `new`;
  - `track`;
  - `track_if_absent`;
  - `request_cancel`;
  - `child_token`;
  - `finish`;
  - `active_run_ids`;
  - `apply_event`;
  - private `ensure_cancellation_token`;
- `run_with_prompt_pack_run_cancellation`.

Keep in `runtime.rs`:

- all Tauri commands and `State<'_, PromptPackRunState>` adapters;
- execution spawning and provider calls;
- browser cancellation commands and provenance persistence;
- terminal-event construction and emission;
- interrupted-run database cleanup, including
  `cleanup_interrupted_prompt_pack_runs_in_pool` and
  `cleanup_interrupted_prompt_pack_runs`;
- dev smoke fixtures and their SQL, including
  `seed_prompt_pack_cancellation_smoke_fixture`,
  `clear_prompt_pack_cancellation_smoke_fixture`, their `_in_pool` helpers,
  and `prompt_pack_cancellation_smoke_fixture_run_ids`;
- `PROMPT_PACK_RUN_EVENT` and the fixture label;
- `emit_prompt_pack_run_event`;
- `now_string`.

`emit_prompt_pack_run_event` continues to call `state.apply_event(...)` before
emitting through Tauri. Only the state implementation moves; event creation,
ordering, cloning, and emission remain runtime-owned.

## Dependencies

`run_control.rs` depends only on:

- `std::collections::{HashMap, HashSet}`;
- `std::future::Future`;
- `tokio::sync::Mutex`;
- `tokio_util::sync::CancellationToken`;
- `super::dto::PromptPackRunEvent`;
- `crate::error::AppResult`;
- `crate::llm::LlmRequestError`.

The module must not import Tauri, SQLx, `AppHandle`, emitters, database pool
helpers, provider execution, browser jobs, persistence modules, or Prompt Pack
stores.

After extraction, `HashMap`, `HashSet`, and `tokio::sync::Mutex` leave
`runtime.rs`. `Future` and `CancellationToken` remain imported there because
other runtime-owned browser and stage-execution functions still use them.

## Behavioral Compatibility

This is a structural refactor only. Preserve exactly:

- `track` inserts the run ID and ensures a cancellation token exists;
- `track_if_absent` returns whether the active-run insertion was new while
  still ensuring a token exists;
- `request_cancel` creates a missing token before cancelling it;
- `child_token` returns a child of the stored token and returns `None` when the
  run has no stored token;
- `finish` removes both active membership and the stored token;
- `active_run_ids` returns a sorted ascending vector;
- `apply_event` calls `finish` only for `completed`, `partial`, `failed`,
  `cancelled`, and `interrupted` event kinds;
- events with any other kind leave the state unchanged;
- the cancellation helper awaits the supplied future directly when no token
  is present;
- an already-cancelled token returns `LlmRequestError::Cancelled` before the
  supplied future is polled to completion;
- otherwise `tokio::select!` races the future against token cancellation and
  returns the winning result.

Do not replace the two mutexes with a combined lock, change lock ordering,
introduce atomics, add timeouts, add cleanup-on-drop behavior, or change token
parent/child relationships in this slice.

## Error Handling

Keep the existing `AppResult` signatures on `track`, `track_if_absent`, and
`request_cancel`, even though their current in-memory operations do not create
an error. Removing those wrappers would change the public method contract and
would widen the refactor into its callers.

The generic cancellation helper continues to return only the supplied
future's `LlmRequestError` or `LlmRequestError::Cancelled`. Do not add logging,
error mapping, context strings, or a new error layer.

## Tests

Keep all existing behavioral tests in `runtime::tests` for this slice. They
already cover:

- active-run registration and duplicate detection;
- cancellation of child tokens;
- removal of finished state;
- terminal event cleanup;
- a future without a cancellation token;
- an already-cancelled token;
- queued and active browser-stage cancellation;
- dev smoke-fixture registration and cancellation cleanup.

Do not move these tests into `run_control.rs`; doing so would mix a small
ownership refactor with test-fixture restructuring. Test bodies and assertions
remain unchanged. Only imports may change if required by the final module
wiring.

Add one raw-source ownership contract under `src/lib/` that verifies:

- `prompt_packs/mod.rs` registers private `run_control`;
- `run_control.rs` defines public `PromptPackRunState`;
- `run_control.rs` defines
  `pub(super) run_with_prompt_pack_run_cancellation`;
- neither definition remains in `runtime.rs`;
- `runtime.rs` publicly re-exports `PromptPackRunState` from `run_control`;
- `prompt_packs/mod.rs` continues to export `PromptPackRunState` through
  `runtime`;
- `run_control.rs` contains the existing terminal-event kind set;
- `run_control.rs` contains no Tauri, SQLx, AppHandle, emitter, database, or
  persistence dependency.

The source contract checks ownership and compatibility paths. Existing Rust
tests remain the authority for behavior and cancellation semantics.

## Verification

Run, at minimum:

1. the new raw-source contract;
2. the focused state and cancellation Rust tests;
3. all `prompt_packs::runtime::tests`;
4. all Prompt Pack Rust tests;
5. the complete Vitest suite;
6. the complete Rust suite;
7. `npm.cmd run check:rustfmt`;
8. `cargo check --manifest-path src-tauri/Cargo.toml --all-targets` with zero
   warnings.

## Scope

Implementation changes are limited to:

- `src-tauri/src/prompt_packs/mod.rs`;
- `src-tauri/src/prompt_packs/runtime.rs`;
- new `src-tauri/src/prompt_packs/run_control.rs`;
- one new focused raw-source contract under `src/lib/`.

Do not modify DTOs, persistence code, migrations, Prompt Pack assets,
dependencies, Tauri command registration, frontend behavior,
`docs/project.md`, or `docs/value-registry.md`. No registered status, state,
kind, mode, phase, provider, or persisted value changes.

## Acceptance Criteria

1. `PromptPackRunState`, its complete implementation, and the generic
   cancellation helper live in private `run_control.rs`.
2. Both existing public paths to `PromptPackRunState` remain valid.
3. `runtime.rs` continues to own commands, provider/browser orchestration,
   event emission, SQL cleanup, and dev fixtures.
4. Active-run membership, token lifecycle, terminal cleanup, cancellation
   races, errors, and public signatures are unchanged.
5. Existing behavioral test bodies remain in `runtime::tests` unchanged.
6. Source ownership, focused behavior, full Vitest, full Rust, rustfmt, and
   all-target checks pass with zero warnings.
7. The implementation diff contains only the four files permitted by scope.

## Deferred Follow-ups

This slice does not extract browser-provider orchestration, terminal-event
construction, interrupted-run persistence, dev smoke fixtures, or Tauri
commands. Those remain independent future designs.
