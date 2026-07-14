# Prompt Pack Runtime Config Extraction Design

## Status

Approved for implementation planning on 2026-07-14.

## Context

`src-tauri/src/prompt_packs/runtime.rs` still owns a small persisted-runtime
configuration block alongside Tauri commands, run lifecycle orchestration, and
provider construction. The block consists of:

- `RunRuntimeProvider` and its persisted-string parser;
- `RunRuntimeConfig`;
- `load_run_runtime_config`, including its SQL query and Browser Provider JSON
  decoding.

The block has one production consumer, `execute_youtube_summary_run`, and one
existing characterization test covering successful API and Gemini Browser
rows. Earlier completion-transport and stage-execution extractions explicitly
left this responsibility in `runtime.rs` so that each refactor had one owner.

This slice now gives persisted runtime configuration its own narrow owner
without moving provider resolution or execution orchestration.

## Goals

- Move persisted runtime-provider parsing, the loaded configuration value, and
  their SQL loader into one private sibling module.
- Preserve SQL, JSON decoding, error classification, and error text.
- Keep LLM profile resolution, model-limit resolution, `RunCompletionRuntime`
  construction, dispatch, and lifecycle orchestration in `runtime.rs`.
- Add characterization coverage for the two currently untested decoding
  failures.
- Protect the new ownership boundary with a CRLF-safe source contract.

## Non-Goals

- No changes to the persisted runtime-provider values `api` and
  `gemini_browser`.
- No database migration or query-semantics change.
- No changes to Browser Provider configuration shape.
- No changes to LLM profile resolution, model selection, or model-limit
  lookup.
- No move of `RunCompletionRuntime` construction or stage dispatch.
- No frontend, command API, event, fixture, or user-visible behavior change.
- No update to `docs/value-registry.md`, because no string value is added,
  removed, or redefined.

## Considered Approaches

### 1. Extract one persisted-runtime-config sibling module

Move `RunRuntimeProvider`, `RunRuntimeConfig`, and
`load_run_runtime_config` into `runtime_config.rs`. Leave provider resolution
and transport construction in `runtime.rs`.

This gives the SQL/configuration responsibility one owner while preserving a
small dependency surface. This is the selected approach.

### 2. Also move provider resolution and `RunCompletionRuntime` construction

This would make `runtime.rs` smaller, but it would mix persistence decoding
with Tauri state, LLM profile resolution, asynchronous model-limit lookup, and
completion-transport construction. Those responsibilities have different
dependencies and failure modes, so this is rejected for this slice.

### 3. Move only the SQL function and keep the types in `runtime.rs`

This would force the new storage module to depend back on the orchestration
module for its return type, or require awkward tuple conversion. It does not
establish a clean ownership boundary and is rejected.

## Selected Architecture

Add one private sibling module:

```text
src-tauri/src/prompt_packs/       # relevant fragment
|-- mod.rs
|-- runtime.rs                    # commands, resolution, construction, lifecycle
|-- runtime_config.rs             # persisted provider/config parsing and loading
`-- ...
```

Register it in `prompt_packs/mod.rs` as:

```rust
mod runtime_config;
```

`runtime_config.rs` owns:

```rust
pub(super) enum RunRuntimeProvider {
    Api,
    GeminiBrowser,
}

pub(super) struct RunRuntimeConfig {
    pub(super) runtime_provider: RunRuntimeProvider,
    pub(super) profile_id: Option<String>,
    pub(super) model_override: Option<String>,
    pub(super) browser_provider_config:
        Option<crate::gemini_browser::GeminiBrowserProviderConfig>,
}

pub(super) async fn load_run_runtime_config(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<RunRuntimeConfig>;
```

`RunRuntimeProvider::parse` remains private because only the loader uses it.
The module and its exported surface are limited to the parent
`prompt_packs` module; there is no crate-wide or public API.

The existing enum, struct, parser body, SQL, bind order, `fetch_one`, JSON
decoding, and error mapping move without semantic rewriting.

## Runtime Ownership After the Move

`execute_youtube_summary_run` in `runtime.rs` continues to:

1. obtain the database pool;
2. call `load_run_runtime_config`;
3. match `RunRuntimeProvider`;
4. resolve the API profile and effective model when required;
5. resolve the model input-token limit;
6. construct `RunCompletionRuntime`;
7. prepare execution options, emit lifecycle events, and dispatch stages.

This intentionally keeps persisted configuration separate from the resolved
runtime objects that depend on Tauri state and backend integrations.

## Dependencies

`runtime_config.rs` may depend only on:

- `sqlx::SqlitePool`;
- `crate::error::{AppError, AppResult}`;
- `crate::gemini_browser::GeminiBrowserProviderConfig`;
- `serde_json` for the existing snapshot decoding.

It must not depend on:

- Tauri types or commands;
- `RunCompletionRuntime` or completion transport;
- LLM profile/model resolution;
- stage execution or request policy;
- prompt-pack lifecycle commands, events, or state;
- `super::runtime`.

After the move, `runtime.rs` imports the loader and provider enum from
`super::runtime_config`. Its other imports are cleaned only where the moved
block makes them unused; unrelated imports are not reorganized.

## Error Handling

The extraction preserves the current failure behavior exactly:

- SQL failures, including a missing `fetch_one` row, continue through
  `AppError::database`;
- an unsupported persisted provider continues to return
  `AppError::validation` with
  `Unsupported prompt-pack runtime provider: {value}`;
- malformed `browser_provider_config_json` continues to return
  `AppError::internal` with
  `parse Browser Provider config snapshot: {error}`.

The slice does not introduce an optional loader, a new not-found mapping,
fallback/default provider behavior, or recovery from malformed snapshots.

## Test Placement

The existing storage fixture and related runtime tests already live in
`runtime::tests`. They stay there to avoid moving or duplicating the shared
database setup in a mechanical extraction.

Before moving production code, add two characterization cases beside
`load_run_runtime_config_reads_api_and_browser_rows`:

- an unsupported provider produces the existing validation error;
- malformed Browser Provider JSON produces the existing internal error.

The tests initially call the current private loader through `super`. After the
move, only their imports change to `super::super::runtime_config`; their test
data and assertions remain unchanged.

## Source Contract

Add `src/lib/prompt-pack-runtime-config-contract.test.ts`. Like the existing
Prompt Pack ownership contracts, it normalizes CRLF before matching Rust
source.

The contract verifies that:

- `prompt_packs/mod.rs` registers exactly a private `mod runtime_config;`;
- `RunRuntimeProvider`, `RunRuntimeConfig`, and
  `load_run_runtime_config` are defined in `runtime_config.rs` with the
  intended sibling visibility;
- `RunRuntimeProvider::parse` remains private;
- those definitions and the loader's `SELECT provider_profile_id, model,
  runtime_provider, browser_provider_config_json` query no longer live in the
  production portion of `runtime.rs`; test-fixture INSERTs may still name the
  same columns;
- the portion of `runtime.rs` before `#[cfg(test)] mod tests` contains exactly
  one call to `load_run_runtime_config`; test calls are counted separately;
- `runtime.rs` retains the provider match, API profile resolution, effective
  model resolution, model input-limit resolution, and
  `RunCompletionRuntime` construction;
- `runtime_config.rs` contains none of the forbidden orchestration,
  transport, stage, Tauri-command, lifecycle, or `super::runtime` markers.

The contract should use precise definition and call-site markers rather than
forbidding provider names globally, because the legitimate provider match
remains in `runtime.rs`.

## Verification Strategy

Implementation verification runs, in order:

1. repository preconditions: clean worktree and passing Rust formatting check;
2. the three runtime-config Rust characterization tests;
3. the focused new Vitest source contract;
4. all `prompt_packs::runtime::tests`;
5. the full Rust test suite;
6. the full Vitest suite;
7. `npm.cmd run check:rustfmt`;
8. `cargo check --manifest-path src-tauri/Cargo.toml --all-targets`;
9. `git diff --check` and a scope review.

The implementation diff is limited to:

- `src-tauri/src/prompt_packs/mod.rs`;
- `src-tauri/src/prompt_packs/runtime.rs`;
- new `src-tauri/src/prompt_packs/runtime_config.rs`;
- new `src/lib/prompt-pack-runtime-config-contract.test.ts`.

The design and plan documents are separate workflow artifacts and are not
counted as implementation files.

## Acceptance Criteria

- Persisted runtime-provider parsing and configuration loading have one owner:
  `runtime_config.rs`.
- `runtime.rs` retains resolution, transport construction, dispatch, and
  lifecycle orchestration.
- Successful API and Gemini Browser rows decode exactly as before.
- Unsupported providers and malformed Browser Provider snapshots retain their
  existing error kinds and messages.
- The new module is private and has no Tauri, transport, stage, or lifecycle
  dependency.
- Focused and full Rust and Vitest suites pass.
- Rust formatting and all-target compilation checks pass.
- The implementation changes only the four declared files.
